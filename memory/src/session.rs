//! 会话存储 — JSON 文件

use crate::session_sqlite::SqliteSessionMirror;
use chrono::{DateTime, FixedOffset};
use hone_core::agent::ToolCallMade;
use hone_core::{ActorIdentity, SessionIdentity, beijing_now};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// 会话存储管理器
pub struct SessionStorage {
    data_dir: PathBuf,
    sqlite_storage: Option<SqliteSessionMirror>,
    runtime_backend: SessionRuntimeBackend,
    shadow_sqlite_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SessionStorageOptions {
    pub shadow_sqlite_db_path: Option<PathBuf>,
    pub shadow_sqlite_enabled: bool,
    pub runtime_backend: SessionRuntimeBackend,
}

impl Default for SessionStorageOptions {
    fn default() -> Self {
        Self {
            shadow_sqlite_db_path: None,
            shadow_sqlite_enabled: false,
            runtime_backend: SessionRuntimeBackend::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRuntimeBackend {
    Json,
    Sqlite,
}

impl SessionRuntimeBackend {
    fn from_config_value(value: &str) -> Self {
        match value.trim() {
            "sqlite" => Self::Sqlite,
            _ => Self::Json,
        }
    }
}

/// 全局共享的会话锁注册表，以防止并发修改同一 session_id 引起的 Last Writer Wins 数据覆盖问题
static SESSION_LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();

fn get_session_lock(session_id: &str) -> Arc<Mutex<()>> {
    let mutex_map = SESSION_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = mutex_map.lock().unwrap();
    if let Some(lock) = map.get(session_id) {
        lock.clone()
    } else {
        let lock = Arc::new(Mutex::new(()));
        map.insert(session_id.to_string(), lock.clone());
        lock
    }
}

fn default_session_version() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SessionPromptState {
    #[serde(default)]
    pub frozen_time_beijing: String,
}

impl SessionPromptState {
    pub fn ensure_frozen_time(mut self) -> Self {
        if self.frozen_time_beijing.trim().is_empty() {
            self.frozen_time_beijing = hone_core::beijing_now_rfc3339();
        }
        self
    }

    pub fn frozen_datetime(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(&self.frozen_time_beijing).unwrap_or_else(|_| beijing_now())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SessionRuntimeState {
    #[serde(default)]
    pub prompt: SessionPromptState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSummary {
    pub content: String,
    pub updated_at: String,
}

impl SessionSummary {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            updated_at: hone_core::beijing_now_rfc3339(),
        }
    }
}

/// 会话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default = "default_session_version")]
    pub version: u32,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<ActorIdentity>,
    /// `actor` 表示最近一次以谁的身份创建 / 更新了这个 session；
    /// `session_identity` 才是这份历史真正按谁归属。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_identity: Option<SessionIdentity>,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<SessionMessage>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
    #[serde(default)]
    pub runtime: SessionRuntimeState,
    #[serde(default)]
    pub summary: Option<SessionSummary>,
}

/// 会话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub metadata: Option<HashMap<String, Value>>,
}

pub fn session_message_in_context(role: &str) -> bool {
    matches!(role, "user" | "assistant" | "tool")
}

pub fn select_context_messages<'a>(
    messages: &'a [SessionMessage],
    limit: Option<usize>,
) -> Vec<&'a SessionMessage> {
    let mut filtered: Vec<_> = messages
        .iter()
        .filter(|message| session_message_in_context(&message.role))
        .collect();

    if let Some(limit) = limit {
        if filtered.len() > limit {
            filtered = filtered
                .into_iter()
                .rev()
                .take(limit)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
        }
    }

    filtered
}

pub fn build_tool_message_metadata(call: &ToolCallMade) -> HashMap<String, Value> {
    build_tool_message_metadata_parts(
        &call.name,
        call.tool_call_id.as_deref(),
        Some(call.arguments.clone()),
    )
}

pub const INVOKED_SKILLS_METADATA_KEY: &str = "skill_runtime.invoked_skills";
pub const SLASH_SKILL_METADATA_KEY: &str = "skill_runtime.slash_skill";
pub const COMPACT_BOUNDARY_METADATA_KEY: &str = "session.compact_boundary";
pub const COMPACT_SUMMARY_METADATA_KEY: &str = "session.compact_summary";
pub const COMPACT_SKILL_SNAPSHOT_METADATA_KEY: &str = "session.compact_skill_snapshot";
pub const ASSISTANT_TOOL_CALLS_METADATA_KEY: &str = "assistant.tool_calls";

pub fn build_tool_message_metadata_parts(
    tool_name: &str,
    tool_call_id: Option<&str>,
    tool_arguments: Option<Value>,
) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "tool_name".to_string(),
        Value::String(tool_name.to_string()),
    );
    if let Some(tool_call_id) = tool_call_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        metadata.insert(
            "tool_call_id".to_string(),
            Value::String(tool_call_id.to_string()),
        );
    }
    if let Some(arguments) = tool_arguments {
        metadata.insert("tool_arguments".to_string(), arguments);
    }
    metadata
}

pub fn build_assistant_message_metadata(tool_calls: &[Value]) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    if !tool_calls.is_empty() {
        metadata.insert(
            ASSISTANT_TOOL_CALLS_METADATA_KEY.to_string(),
            Value::Array(tool_calls.to_vec()),
        );
    }
    metadata
}

pub fn assistant_tool_calls_from_metadata(
    metadata: Option<&HashMap<String, Value>>,
) -> Option<Vec<Value>> {
    metadata
        .and_then(|items| items.get(ASSISTANT_TOOL_CALLS_METADATA_KEY))
        .and_then(|value| value.as_array())
        .map(|items| items.to_vec())
        .filter(|items| !items.is_empty())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InvokedSkillRecord {
    pub skill_name: String,
    pub display_name: String,
    pub path: String,
    pub prompt: String,
    pub execution_context: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub loaded_from: String,
    pub updated_at: String,
}

pub fn invoked_skills_from_metadata(metadata: &HashMap<String, Value>) -> Vec<InvokedSkillRecord> {
    metadata
        .get(INVOKED_SKILLS_METADATA_KEY)
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<InvokedSkillRecord>>(value).ok())
        .unwrap_or_default()
}

pub fn message_is_slash_skill(metadata: Option<&HashMap<String, Value>>) -> bool {
    metadata
        .and_then(|items| items.get(SLASH_SKILL_METADATA_KEY))
        .and_then(|value| value.as_str())
        .is_some_and(|value| !value.trim().is_empty())
}

pub fn message_is_compact_boundary(metadata: Option<&HashMap<String, Value>>) -> bool {
    metadata
        .and_then(|items| items.get(COMPACT_BOUNDARY_METADATA_KEY))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub fn message_is_compact_summary(metadata: Option<&HashMap<String, Value>>) -> bool {
    metadata
        .and_then(|items| items.get(COMPACT_SUMMARY_METADATA_KEY))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub fn message_is_compact_skill_snapshot(metadata: Option<&HashMap<String, Value>>) -> bool {
    metadata
        .and_then(|items| items.get(COMPACT_SKILL_SNAPSHOT_METADATA_KEY))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub fn build_compact_boundary_metadata(
    trigger: &str,
    messages_summarized: usize,
    pre_summary_messages: usize,
) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert(COMPACT_BOUNDARY_METADATA_KEY.to_string(), Value::Bool(true));
    metadata.insert("trigger".to_string(), Value::String(trigger.to_string()));
    metadata.insert(
        "messages_summarized".to_string(),
        Value::Number(messages_summarized.into()),
    );
    metadata.insert(
        "pre_summary_messages".to_string(),
        Value::Number(pre_summary_messages.into()),
    );
    metadata
}

pub fn build_compact_summary_metadata(source: &str) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert(COMPACT_SUMMARY_METADATA_KEY.to_string(), Value::Bool(true));
    metadata.insert("source".to_string(), Value::String(source.to_string()));
    metadata
}

pub fn build_compact_skill_snapshot_metadata(skill_name: &str) -> HashMap<String, Value> {
    let mut metadata = HashMap::new();
    metadata.insert(
        COMPACT_SKILL_SNAPSHOT_METADATA_KEY.to_string(),
        Value::Bool(true),
    );
    metadata.insert(
        "skill_name".to_string(),
        Value::String(skill_name.to_string()),
    );
    metadata
}

pub fn find_last_compact_boundary_index(messages: &[SessionMessage]) -> Option<usize> {
    messages
        .iter()
        .rposition(|message| message_is_compact_boundary(message.metadata.as_ref()))
}

pub fn select_messages_after_compact_boundary<'a>(
    messages: &'a [SessionMessage],
    limit: Option<usize>,
) -> Vec<&'a SessionMessage> {
    let sliced = if let Some(index) = find_last_compact_boundary_index(messages) {
        &messages[index..]
    } else {
        messages
    };
    select_context_messages(sliced, limit)
}

pub fn latest_compact_summary(messages: &[SessionMessage]) -> Option<&SessionMessage> {
    messages
        .iter()
        .rev()
        .find(|message| message_is_compact_summary(message.metadata.as_ref()))
}

pub fn has_compact_skill_snapshot(messages: &[&SessionMessage]) -> bool {
    messages
        .iter()
        .any(|message| message_is_compact_skill_snapshot(message.metadata.as_ref()))
}

pub fn restore_tool_message(
    content: &str,
    metadata: Option<&HashMap<String, Value>>,
) -> Option<(String, String, String)> {
    let metadata = metadata?;
    let tool_name = metadata.get("tool_name")?.as_str()?.trim().to_string();
    if tool_name.is_empty() {
        return None;
    }
    let tool_call_id = metadata
        .get("tool_call_id")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    Some((tool_call_id, tool_name, content.to_string()))
}

impl SessionStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self::with_options(data_dir, SessionStorageOptions::default())
    }

    pub fn with_options(data_dir: impl AsRef<Path>, options: SessionStorageOptions) -> Self {
        let dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir).ok();
        let sqlite_storage = if options.shadow_sqlite_enabled
            || matches!(options.runtime_backend, SessionRuntimeBackend::Sqlite)
        {
            options.shadow_sqlite_db_path.as_ref().and_then(|path| {
                match SqliteSessionMirror::new(path) {
                    Ok(storage) => Some(storage),
                    Err(err) => {
                        tracing::warn!(
                            path = %path.display(),
                            "failed to initialize session shadow sqlite: {err}"
                        );
                        None
                    }
                }
            })
        } else {
            None
        };

        Self {
            data_dir: dir,
            sqlite_storage,
            runtime_backend: options.runtime_backend,
            shadow_sqlite_enabled: options.shadow_sqlite_enabled,
        }
    }

    pub fn from_storage_config(config: &hone_core::config::StorageConfig) -> Self {
        Self::with_options(
            &config.sessions_dir,
            SessionStorageOptions {
                shadow_sqlite_db_path: Some(PathBuf::from(&config.session_sqlite_db_path)),
                shadow_sqlite_enabled: config.session_sqlite_shadow_write_enabled,
                runtime_backend: SessionRuntimeBackend::from_config_value(
                    &config.session_runtime_backend,
                ),
            },
        )
    }

    /// 创建新会话
    pub fn create_session(
        &self,
        session_id: Option<&str>,
        actor: Option<ActorIdentity>,
        session_identity: Option<SessionIdentity>,
    ) -> hone_core::HoneResult<String> {
        let id = if let Some(session_id) = session_id {
            validate_storage_component(session_id).ok_or_else(|| {
                hone_core::HoneError::Config("session_id 包含非法路径组件".to_string())
            })?
        } else {
            session_identity
                .as_ref()
                .map(SessionIdentity::session_id)
                .or_else(|| actor.as_ref().map(ActorIdentity::session_id))
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
        };

        let session_identity = session_identity.or_else(|| {
            actor
                .as_ref()
                .and_then(|actor| SessionIdentity::from_actor(actor).ok())
        });

        let now = hone_core::beijing_now_rfc3339();
        let session = Session {
            version: default_session_version(),
            id: id.clone(),
            actor,
            session_identity,
            created_at: now.clone(),
            updated_at: now,
            messages: Vec::new(),
            metadata: HashMap::new(),
            runtime: SessionRuntimeState {
                prompt: SessionPromptState {
                    frozen_time_beijing: hone_core::beijing_now_rfc3339(),
                },
            },
            summary: None,
        };

        self.write_session(&id, &session)?;

        Ok(id)
    }

    pub fn create_session_for_actor(&self, actor: &ActorIdentity) -> hone_core::HoneResult<String> {
        self.create_session(
            Some(&actor.session_id()),
            Some(actor.clone()),
            SessionIdentity::from_actor(actor).ok(),
        )
    }

    pub fn create_session_for_identity(
        &self,
        session_identity: &SessionIdentity,
        actor: Option<&ActorIdentity>,
    ) -> hone_core::HoneResult<String> {
        self.create_session(
            Some(&session_identity.session_id()),
            actor.cloned(),
            Some(session_identity.clone()),
        )
    }

    /// 加载会话
    pub fn load_session(&self, session_id: &str) -> hone_core::HoneResult<Option<Session>> {
        match self.runtime_backend {
            SessionRuntimeBackend::Json => self.load_session_from_json(session_id),
            SessionRuntimeBackend::Sqlite => {
                if let Some(storage) = &self.sqlite_storage {
                    if let Some(session) = storage.load_session(session_id)? {
                        return Ok(Some(session));
                    }
                }

                let fallback = self.load_session_from_json(session_id)?;
                if let Some(session) = &fallback {
                    if let Ok(path) = self.session_json_path(session_id) {
                        let _ = self.write_session_to_sqlite(&path, session);
                    }
                }
                Ok(fallback)
            }
        }
    }

    /// 添加消息
    pub fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        metadata: Option<HashMap<String, Value>>,
    ) -> hone_core::HoneResult<bool> {
        // 先获取当前 session 会话的全局锁，再执行「读 -> 内存追加 -> 写」，保证同一 session 绝对序列化
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(false);
        };

        session.messages.push(SessionMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: hone_core::beijing_now_rfc3339(),
            metadata,
        });
        session.updated_at = hone_core::beijing_now_rfc3339();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    /// 获取消息列表
    pub fn get_messages(
        &self,
        session_id: &str,
        limit: Option<usize>,
    ) -> hone_core::HoneResult<Vec<SessionMessage>> {
        let Some(session) = self.load_session(session_id)? else {
            return Ok(Vec::new());
        };

        let messages = match limit {
            Some(n) => session
                .messages
                .into_iter()
                .rev()
                .take(n)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            None => session.messages,
        };

        Ok(messages)
    }

    pub fn list_sessions(&self) -> hone_core::HoneResult<Vec<Session>> {
        match self.runtime_backend {
            SessionRuntimeBackend::Json => self.list_sessions_from_json(),
            SessionRuntimeBackend::Sqlite => {
                if let Some(storage) = &self.sqlite_storage {
                    let sessions = storage.list_sessions()?;
                    if !sessions.is_empty() {
                        return Ok(sessions);
                    }
                }
                self.list_sessions_from_json()
            }
        }
    }

    /// 获取或初始化 session 级 prompt 状态。
    pub fn ensure_prompt_state(
        &self,
        session_id: &str,
    ) -> hone_core::HoneResult<Option<SessionPromptState>> {
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(None);
        };

        let prompt = session.runtime.prompt.clone().ensure_frozen_time();
        if prompt != session.runtime.prompt {
            session.runtime.prompt = prompt.clone();
            session.version = default_session_version();
            session.updated_at = hone_core::beijing_now_rfc3339();
            self.write_session(session_id, &session)?;
        }

        Ok(Some(prompt))
    }

    /// 替换整个消息列表（用于上下文压缩）
    pub fn replace_messages(
        &self,
        session_id: &str,
        messages: Vec<SessionMessage>,
    ) -> hone_core::HoneResult<bool> {
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(false);
        };

        session.messages = messages;
        session.updated_at = hone_core::beijing_now_rfc3339();
        session.version = default_session_version();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    pub fn replace_messages_with_summary(
        &self,
        session_id: &str,
        messages: Vec<SessionMessage>,
        summary: Option<SessionSummary>,
    ) -> hone_core::HoneResult<bool> {
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(false);
        };

        session.messages = messages;
        session.summary = summary;
        session.version = default_session_version();
        session.updated_at = hone_core::beijing_now_rfc3339();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    /// 更新会话级 metadata（合并写入）
    pub fn update_metadata(
        &self,
        session_id: &str,
        metadata: HashMap<String, Value>,
    ) -> hone_core::HoneResult<bool> {
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(false);
        };

        for (key, value) in metadata {
            session.metadata.insert(key, value);
        }
        session.updated_at = hone_core::beijing_now_rfc3339();
        session.version = default_session_version();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    fn write_session(&self, session_id: &str, session: &Session) -> hone_core::HoneResult<()> {
        let path = self.session_json_path(session_id)?;
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        std::fs::write(&path, json)?;

        match self.runtime_backend {
            SessionRuntimeBackend::Json => self.shadow_write_session(&path, session),
            SessionRuntimeBackend::Sqlite => self.write_session_to_sqlite(&path, session)?,
        }
        Ok(())
    }

    fn session_json_path(&self, session_id: &str) -> hone_core::HoneResult<PathBuf> {
        let normalized = validate_storage_component(session_id).ok_or_else(|| {
            hone_core::HoneError::Config("session_id 包含非法路径组件".to_string())
        })?;
        Ok(self.data_dir.join(format!("{normalized}.json")))
    }

    fn load_session_from_json(&self, session_id: &str) -> hone_core::HoneResult<Option<Session>> {
        let path = self.session_json_path(session_id)?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&content)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        Ok(Some(session))
    }

    fn list_sessions_from_json(&self) -> hone_core::HoneResult<Vec<Session>> {
        let mut sessions = Vec::new();
        let entries = match std::fs::read_dir(&self.data_dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err.into()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue,
            };
            let session: Session = match serde_json::from_str(&content) {
                Ok(session) => session,
                Err(_) => continue,
            };
            sessions.push(session);
        }

        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }

    fn write_session_to_sqlite(&self, path: &Path, session: &Session) -> hone_core::HoneResult<()> {
        let Some(sqlite_storage) = &self.sqlite_storage else {
            self.shadow_write_session(path, session);
            return Ok(());
        };

        if let Err(err) = sqlite_storage.upsert_session(path, session) {
            tracing::error!(
                session_id = %session.id,
                path = %path.display(),
                "failed to write session into sqlite runtime backend: {err}"
            );
            return Err(err);
        }

        Ok(())
    }

    fn shadow_write_session(&self, path: &Path, session: &Session) {
        if !self.shadow_sqlite_enabled {
            return;
        }

        let Some(shadow_sqlite) = &self.sqlite_storage else {
            return;
        };

        if let Err(err) = shadow_sqlite.upsert_session(path, session) {
            tracing::warn!(
                session_id = %session.id,
                path = %path.display(),
                "failed to shadow-write session into sqlite: {err}"
            );
        }
    }
}

fn validate_storage_component(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut components = Path::new(trimmed).components();
    let component = match components.next() {
        Some(Component::Normal(component)) => component.to_str()?.to_string(),
        _ => return None,
    };
    if components.next().is_some() {
        return None;
    }
    if component.is_empty() {
        None
    } else {
        Some(component)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn update_metadata_merges_existing_values() {
        let root = make_temp_dir("hone_memory_test");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "alice", None::<String>).expect("actor");
        let session_id = storage
            .create_session(
                Some("Actor_test"),
                Some(actor),
                Some(SessionIdentity::direct("feishu", "alice").expect("session")),
            )
            .expect("create");
        storage
            .update_metadata(
                &session_id,
                HashMap::from([("channel".to_string(), Value::String("feishu".to_string()))]),
            )
            .expect("first update");
        storage
            .update_metadata(
                &session_id,
                HashMap::from([(
                    "email".to_string(),
                    Value::String("alice@example.com".to_string()),
                )]),
            )
            .expect("second update");

        let session = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(
            session.metadata.get("channel"),
            Some(&Value::String("feishu".to_string()))
        );
        assert_eq!(
            session.metadata.get("email"),
            Some(&Value::String("alice@example.com".to_string()))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn create_session_for_actor_persists_actor_identity() {
        let root = make_temp_dir("hone_memory_test");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");

        let session = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(session.actor, Some(actor));
        assert_eq!(
            session.session_identity,
            Some(SessionIdentity::group("discord", "g:1:c:2").expect("session"))
        );
        assert_eq!(session.version, 3);
        assert!(!session.runtime.prompt.frozen_time_beijing.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_add_message_does_not_lose_data() {
        let root = make_temp_dir("hone_memory_test_concurrent");
        let storage = Arc::new(SessionStorage::new(&root));
        let session_id = storage.create_session(None, None, None).expect("create");

        let mut handles = vec![];
        let num_threads = 50;

        for i in 0..num_threads {
            let storage_clone = storage.clone();
            let sid = session_id.clone();
            handles.push(std::thread::spawn(move || {
                storage_clone
                    .add_message(&sid, "user", &format!("Msg {}", i), None)
                    .unwrap();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let msgs = storage
            .get_messages(&session_id, None)
            .expect("get_messages");
        assert_eq!(msgs.len(), num_threads);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn load_session_missing_returns_none() {
        let root = make_temp_dir("hone_memory_test_missing");
        let storage = SessionStorage::new(&root);
        let session = storage.load_session("does-not-exist").expect("load");
        assert!(session.is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn add_message_missing_session_returns_false() {
        let root = make_temp_dir("hone_memory_test_missing_add");
        let storage = SessionStorage::new(&root);
        let ok = storage
            .add_message("does-not-exist", "user", "hi", None)
            .expect("add_message");
        assert!(!ok);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn get_messages_missing_session_returns_empty() {
        let root = make_temp_dir("hone_memory_test_missing_get");
        let storage = SessionStorage::new(&root);
        let msgs = storage.get_messages("does-not-exist", None).expect("get");
        assert!(msgs.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn get_messages_limit_returns_latest_in_order() {
        let root = make_temp_dir("hone_memory_test_limit");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        storage
            .add_message(&session_id, "user", "m1", None)
            .expect("add1");
        storage
            .add_message(&session_id, "assistant", "m2", None)
            .expect("add2");
        storage
            .add_message(&session_id, "user", "m3", None)
            .expect("add3");

        let msgs = storage.get_messages(&session_id, Some(2)).expect("get");
        let contents: Vec<_> = msgs.iter().map(|m| m.content.as_str()).collect();
        assert_eq!(contents, vec!["m2", "m3"]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn replace_messages_overwrites_existing() {
        let root = make_temp_dir("hone_memory_test_replace");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        storage
            .add_message(&session_id, "user", "before", None)
            .expect("add");

        let new_messages = vec![SessionMessage {
            role: "assistant".to_string(),
            content: "after".to_string(),
            timestamp: hone_core::beijing_now_rfc3339(),
            metadata: None,
        }];

        let ok = storage
            .replace_messages(&session_id, new_messages)
            .expect("replace");
        assert!(ok);

        let msgs = storage.get_messages(&session_id, None).expect("get");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "after");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn replace_messages_missing_session_returns_false() {
        let root = make_temp_dir("hone_memory_test_replace_missing");
        let storage = SessionStorage::new(&root);
        let ok = storage
            .replace_messages("does-not-exist", Vec::new())
            .expect("replace");
        assert!(!ok);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn create_session_rejects_parent_dir_component() {
        let root = make_temp_dir("hone_memory_test_invalid_session_id");
        let storage = SessionStorage::new(&root);
        let err = storage
            .create_session(Some("../escape"), None, None)
            .expect_err("invalid session id should fail");
        assert!(
            err.to_string().contains("session_id"),
            "unexpected error: {err}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ensure_prompt_state_sets_frozen_time_once() {
        let root = make_temp_dir("hone_memory_test_prompt_state");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        let first = storage
            .ensure_prompt_state(&session_id)
            .expect("ensure first")
            .expect("prompt");
        let second = storage
            .ensure_prompt_state(&session_id)
            .expect("ensure second")
            .expect("prompt");

        assert_eq!(first, second);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn replace_messages_with_summary_updates_both() {
        let root = make_temp_dir("hone_memory_test_replace_summary");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        let new_messages = vec![SessionMessage {
            role: "assistant".to_string(),
            content: "after".to_string(),
            timestamp: hone_core::beijing_now_rfc3339(),
            metadata: None,
        }];

        storage
            .replace_messages_with_summary(
                &session_id,
                new_messages,
                Some(SessionSummary::new("summary text")),
            )
            .expect("replace");

        let session = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(
            session.summary.as_ref().map(|item| item.content.as_str()),
            Some("summary text")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn select_messages_after_compact_boundary_slices_to_latest_boundary() {
        let messages = vec![
            SessionMessage {
                role: "user".to_string(),
                content: "before".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: None,
            },
            SessionMessage {
                role: "system".to_string(),
                content: "Conversation compacted".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: Some(build_compact_boundary_metadata("auto", 3, 5)),
            },
            SessionMessage {
                role: "user".to_string(),
                content: "【Compact Summary】\nsummary".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: Some(build_compact_summary_metadata("auto")),
            },
            SessionMessage {
                role: "assistant".to_string(),
                content: "after".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: None,
            },
        ];

        let selected = select_messages_after_compact_boundary(&messages, None);
        let contents: Vec<_> = selected.iter().map(|m| m.content.as_str()).collect();
        assert_eq!(contents, vec!["【Compact Summary】\nsummary", "after"]);
        assert_eq!(
            latest_compact_summary(&messages).map(|message| message.content.as_str()),
            Some("【Compact Summary】\nsummary")
        );
    }

    #[test]
    fn select_context_messages_keeps_tool_role() {
        let messages = vec![
            SessionMessage {
                role: "system".to_string(),
                content: "ignore".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: None,
            },
            SessionMessage {
                role: "user".to_string(),
                content: "u1".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: None,
            },
            SessionMessage {
                role: "tool".to_string(),
                content: "t1".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: Some(HashMap::from([(
                    "tool_name".to_string(),
                    Value::String("web_search".to_string()),
                )])),
            },
            SessionMessage {
                role: "assistant".to_string(),
                content: "a1".to_string(),
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: None,
            },
        ];

        let selected = select_context_messages(&messages, Some(3));
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].role, "user");
        assert_eq!(selected[1].role, "tool");
        assert_eq!(selected[2].role, "assistant");
    }

    #[test]
    fn tool_message_metadata_round_trips() {
        let call = ToolCallMade {
            name: "web_search".to_string(),
            arguments: serde_json::json!({ "query": "AAPL" }),
            result: serde_json::json!({ "ok": true }),
            tool_call_id: Some("call_1".to_string()),
        };
        let metadata = build_tool_message_metadata(&call);
        let restored =
            restore_tool_message("{\"ok\":true}", Some(&metadata)).expect("restore tool");
        assert_eq!(restored.0, "call_1");
        assert_eq!(restored.1, "web_search");
        assert_eq!(restored.2, "{\"ok\":true}");
    }

    #[test]
    fn update_metadata_missing_session_returns_false() {
        let root = make_temp_dir("hone_memory_test_metadata_missing");
        let storage = SessionStorage::new(&root);
        let ok = storage
            .update_metadata(
                "does-not-exist",
                HashMap::from([("k".to_string(), Value::String("v".to_string()))]),
            )
            .expect("update");
        assert!(!ok);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn shadow_sqlite_writes_without_affecting_json_flow() {
        let root = make_temp_dir("hone_memory_test_shadow_sqlite");
        let db_path = root.join("sessions.sqlite3");
        let storage = SessionStorage::with_options(
            root.join("sessions"),
            SessionStorageOptions {
                shadow_sqlite_db_path: Some(db_path.clone()),
                shadow_sqlite_enabled: true,
                runtime_backend: SessionRuntimeBackend::Json,
            },
        );

        let actor = ActorIdentity::new("feishu", "alice", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        storage
            .add_message(&session_id, "user", "hello shadow", None)
            .expect("append");

        let conn = rusqlite::Connection::open(&db_path).expect("sqlite");
        let session_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .expect("session count");
        let message_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_messages WHERE session_id = ?1",
                [&session_id],
                |row| row.get(0),
            )
            .expect("message count");

        assert_eq!(session_count, 1);
        assert_eq!(message_count, 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn sqlite_runtime_backend_reads_from_sqlite() {
        let root = make_temp_dir("hone_memory_test_sqlite_runtime");
        let db_path = root.join("sessions.sqlite3");
        let storage = SessionStorage::with_options(
            root.join("sessions"),
            SessionStorageOptions {
                shadow_sqlite_db_path: Some(db_path.clone()),
                shadow_sqlite_enabled: true,
                runtime_backend: SessionRuntimeBackend::Sqlite,
            },
        );

        let actor = ActorIdentity::new("feishu", "bob", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        storage
            .add_message(&session_id, "user", "hello sqlite", None)
            .expect("append");

        std::fs::remove_file(root.join("sessions").join(format!("{session_id}.json")))
            .expect("remove json fallback");

        let session = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "hello sqlite");

        let _ = std::fs::remove_dir_all(root);
    }
}
