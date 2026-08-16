//! 会话存储 — PostgreSQL 权威后端。

use chrono::{DateTime, FixedOffset};
use hone_core::agent::{
    AgentMessage, NormalizedConversationMessage, NormalizedConversationPart, ToolCallMade,
    denormalize_normalized_message,
};
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::cloud_sync::{ensure_cloud_schema_once, run_cloud_sync};
use hone_core::{ActorIdentity, HoneResult, SessionIdentity, compare_rfc3339, local_now};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Session that was in-flight when the process was killed, identified by a recent
/// last user message without a following assistant reply.
#[derive(Debug, Clone)]
pub struct InterruptedSessionInfo {
    pub session_id: String,
    pub actor_user_id: String,
    pub actor_channel_scope: Option<String>,
}

/// Session 的持久化后端抽象。
///
/// PostgreSQL 是权威存储；这个 trait 允许测试注入同语义的内存实现。
///
/// 语义约定：
/// - `upsert`：必须幂等；`source_path` 是对应 JSON 文件的绝对路径，
///   便于实现层在自己的 schema 里保留路径元信息
/// - `load` 返回 `None` 表示「index 里没有」，调用方会自行 fallback 到 JSON
/// - `list` 返回的顺序由实现决定；`SessionStorage` 只在 JSON 扫出空时才信任它
/// - `find_interrupted` 是面向 channel recovery 的窄接口，纯 JSON 场景下
///   可以返回空 Vec
pub trait SessionIndex: Send + Sync {
    /// 把 `session` 全量 upsert 进索引。`source_path` 是对应 JSON 文件路径。
    fn upsert(&self, source_path: &Path, session: &Session) -> HoneResult<()>;

    /// 按 session_id 查询。未命中返回 `Ok(None)`。
    fn load(&self, session_id: &str) -> HoneResult<Option<Session>>;

    /// 列出索引里已知的所有 session。顺序由实现决定。
    fn list(&self) -> HoneResult<Vec<Session>>;

    /// 查询某渠道在给定时间窗内被中断（最后一条是 user、没有后续 assistant）的 session。
    /// 仅供 channel recovery 使用；纯 JSON 后端返回空切片是合理默认。
    fn find_interrupted(
        &self,
        channel: &str,
        updated_after_rfc3339: &str,
        updated_before_rfc3339: &str,
    ) -> HoneResult<Vec<InterruptedSessionInfo>>;
}

/// 会话存储管理器
///
/// 字段说明：
/// - `data_dir`：session JSON 文件的根目录
/// `storage` 是唯一的 PostgreSQL 会话索引；测试可注入同语义的内存实现。
pub struct SessionStorage {
    data_dir: PathBuf,
    storage: Arc<dyn SessionIndex>,
    _test_postgres_lease: Option<Arc<crate::test_postgres::TestPostgresLease>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRuntimeBackend {
    CloudPg,
}

pub struct CloudPgSessionIndex {
    postgres: CloudPgRuntime,
}

impl CloudPgSessionIndex {
    pub fn new(postgres: CloudPgRuntime) -> HoneResult<Self> {
        ensure_cloud_schema_once(postgres.clone(), None)?;
        Ok(Self { postgres })
    }
}

impl SessionIndex for CloudPgSessionIndex {
    fn upsert(&self, _source_path: &Path, session: &Session) -> HoneResult<()> {
        let postgres = self.postgres.clone();
        let session_id = session.id.clone();
        let actor_storage_key = session_actor_storage_key(session);
        let content = serde_json::to_value(session)
            .map_err(|err| hone_core::HoneError::Serialization(err.to_string()))?;
        run_cloud_session(async move {
            postgres
                .upsert_session_record(&session_id, &actor_storage_key, content)
                .await
        })
    }

    fn load(&self, session_id: &str) -> HoneResult<Option<Session>> {
        let postgres = self.postgres.clone();
        let session_id = session_id.to_string();
        run_cloud_session(async move { postgres.load_session_record(&session_id).await })?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|err| hone_core::HoneError::Serialization(err.to_string()))
    }

    fn list(&self) -> HoneResult<Vec<Session>> {
        let postgres = self.postgres.clone();
        let values = run_cloud_session(async move { postgres.list_session_records().await })?;
        let mut sessions = Vec::new();
        for value in values {
            match serde_json::from_value::<Session>(value) {
                Ok(session) => sessions.push(session),
                Err(err) => {
                    tracing::warn!("failed to parse cloud session record: {err}");
                }
            }
        }
        Ok(sessions)
    }

    fn find_interrupted(
        &self,
        channel: &str,
        updated_after_rfc3339: &str,
        updated_before_rfc3339: &str,
    ) -> HoneResult<Vec<InterruptedSessionInfo>> {
        let channel = channel.to_string();
        let updated_after = updated_after_rfc3339.to_string();
        let updated_before = updated_before_rfc3339.to_string();
        Ok(self
            .list()?
            .into_iter()
            .filter(|session| {
                compare_rfc3339(&session.updated_at, &updated_after).is_gt()
                    && compare_rfc3339(&session.updated_at, &updated_before).is_lt()
                    && session
                        .actor
                        .as_ref()
                        .map(|actor| {
                            actor.channel == channel
                                && actor.channel_scope.is_none()
                                && session
                                    .messages
                                    .last()
                                    .map(|message| message.role.as_str() == "user")
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false)
            })
            .filter_map(|session| {
                let actor = session.actor?;
                Some(InterruptedSessionInfo {
                    session_id: session.id,
                    actor_user_id: actor.user_id,
                    actor_channel_scope: actor.channel_scope,
                })
            })
            .collect())
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

fn session_actor_storage_key(session: &Session) -> String {
    session
        .actor
        .as_ref()
        .map(ActorIdentity::storage_key)
        .or_else(|| {
            session
                .session_identity
                .as_ref()
                .map(SessionIdentity::session_id)
        })
        .unwrap_or_else(|| session.id.clone())
}

fn run_cloud_session<T, F>(future: F) -> HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = HoneResult<T>> + Send + 'static,
{
    run_cloud_sync(future, None, "cloud session operation")
}

fn default_session_version() -> u32 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SessionPromptState {
    #[serde(default, alias = "frozen_time_beijing")]
    pub frozen_time_local: String,
}

impl SessionPromptState {
    pub fn ensure_frozen_time(mut self) -> Self {
        if self.frozen_time_local.trim().is_empty() {
            self.frozen_time_local = hone_core::local_now_rfc3339();
        }
        self
    }

    pub fn frozen_datetime(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(&self.frozen_time_local).unwrap_or_else(|_| local_now())
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
            updated_at: hone_core::local_now_rfc3339(),
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
#[derive(Debug, Clone, Serialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: Vec<NormalizedConversationPart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub timestamp: String,
    #[serde(default)]
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SessionMessageContentCompat {
    Text(String),
    Parts(Vec<NormalizedConversationPart>),
}

#[derive(Debug, Clone, Deserialize)]
struct SessionMessageCompat {
    role: String,
    content: SessionMessageContentCompat,
    #[serde(default)]
    status: Option<String>,
    timestamp: String,
    #[serde(default)]
    metadata: Option<HashMap<String, Value>>,
}

impl<'de> Deserialize<'de> for SessionMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = SessionMessageCompat::deserialize(deserializer)?;
        let content = match raw.content {
            SessionMessageContentCompat::Parts(parts) => parts,
            SessionMessageContentCompat::Text(text) => {
                session_message_parts_from_legacy(&raw.role, &text, raw.metadata.as_ref())
            }
        };
        Ok(Self {
            role: raw.role,
            content,
            status: raw.status,
            timestamp: raw.timestamp,
            metadata: raw.metadata,
        })
    }
}

pub fn session_message_in_context(role: &str) -> bool {
    matches!(role, "user" | "assistant" | "tool")
}

fn message_metadata_clone(
    metadata: Option<&HashMap<String, Value>>,
) -> Option<HashMap<String, Value>> {
    metadata.cloned()
}

fn text_part(part_type: &str, text: &str) -> Option<NormalizedConversationPart> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(NormalizedConversationPart {
        part_type: part_type.to_string(),
        text: Some(trimmed.to_string()),
        id: None,
        name: None,
        args: None,
        result: None,
        metadata: None,
    })
}

fn text_part_with_metadata(
    part_type: &str,
    text: &str,
    metadata: Option<&HashMap<String, Value>>,
) -> Option<NormalizedConversationPart> {
    text_part(part_type, text).map(|mut part| {
        part.metadata = message_metadata_clone(metadata);
        part
    })
}

fn parse_json_or_string(input: &str) -> Value {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Value::String(String::new())
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(trimmed.to_string()))
    }
}

fn normalized_tool_call_parts(
    metadata: Option<&HashMap<String, Value>>,
) -> Vec<NormalizedConversationPart> {
    assistant_tool_calls_from_metadata(metadata)
        .into_iter()
        .flatten()
        .map(|tool_call| NormalizedConversationPart {
            part_type: "tool_call".to_string(),
            text: None,
            id: tool_call
                .get("id")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            name: tool_call
                .get("function")
                .and_then(|value| value.get("name"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            args: tool_call
                .get("function")
                .and_then(|value| value.get("arguments"))
                .map(|value| {
                    value
                        .as_str()
                        .map(parse_json_or_string)
                        .unwrap_or_else(|| value.clone())
                }),
            result: None,
            metadata: None,
        })
        .collect()
}

fn session_message_parts_from_legacy(
    role: &str,
    content: &str,
    metadata: Option<&HashMap<String, Value>>,
) -> Vec<NormalizedConversationPart> {
    match role {
        "user" | "system" => text_part_with_metadata("text", content, metadata)
            .into_iter()
            .collect(),
        "assistant" => {
            let mut parts = normalized_tool_call_parts(metadata);
            let part_type = if parts.is_empty() { "text" } else { "final" };
            parts.extend(text_part_with_metadata(part_type, content, metadata));
            parts
        }
        "tool" => {
            let tool_name = metadata
                .and_then(|items| items.get("tool_name"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string);
            let tool_call_id = metadata
                .and_then(|items| items.get("tool_call_id"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string);
            vec![NormalizedConversationPart {
                part_type: "tool_result".to_string(),
                text: None,
                id: tool_call_id,
                name: tool_name,
                args: None,
                result: Some(parse_json_or_string(content)),
                metadata: message_metadata_clone(metadata),
            }]
        }
        _ => text_part_with_metadata("text", content, metadata)
            .into_iter()
            .collect(),
    }
}

pub fn session_message_text(message: &SessionMessage) -> String {
    let text = message
        .content
        .iter()
        .filter_map(|part| part.text.as_deref())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if !text.is_empty() {
        return text;
    }

    message
        .content
        .iter()
        .find(|part| part.part_type == "tool_result")
        .map(tool_result_text)
        .unwrap_or_default()
}

pub fn session_message_to_normalized(message: &SessionMessage) -> NormalizedConversationMessage {
    NormalizedConversationMessage {
        role: message.role.clone(),
        content: message.content.clone(),
        status: message.status.clone(),
        metadata: message.metadata.clone(),
    }
}

pub fn session_message_to_agent_messages(message: &SessionMessage) -> Vec<AgentMessage> {
    if message.role == "tool" {
        return message
            .content
            .iter()
            .find(|part| part.part_type == "tool_result")
            .map(|part| {
                vec![AgentMessage {
                    role: "tool".to_string(),
                    content: Some(tool_result_text(part)),
                    tool_calls: None,
                    tool_call_id: part.id.clone(),
                    name: part.name.clone(),
                    metadata: part.metadata.clone().or_else(|| message.metadata.clone()),
                }]
            })
            .or_else(|| {
                restore_tool_message(message).map(|(tool_call_id, tool_name, content)| {
                    vec![AgentMessage {
                        role: "tool".to_string(),
                        content: Some(content),
                        tool_calls: None,
                        tool_call_id: Some(tool_call_id),
                        name: Some(tool_name),
                        metadata: message.metadata.clone(),
                    }]
                })
            })
            .unwrap_or_default();
    }

    denormalize_normalized_message(&session_message_to_normalized(message))
}

pub fn session_message_from_normalized(
    message: &NormalizedConversationMessage,
    timestamp: impl Into<String>,
) -> SessionMessage {
    SessionMessage {
        role: message.role.clone(),
        content: message.content.clone(),
        status: message.status.clone(),
        timestamp: timestamp.into(),
        metadata: message.metadata.clone(),
    }
}

pub fn session_message_from_text(
    role: &str,
    content: &str,
    timestamp: impl Into<String>,
    metadata: Option<HashMap<String, Value>>,
) -> SessionMessage {
    SessionMessage {
        role: role.to_string(),
        content: session_message_parts_from_legacy(role, content, metadata.as_ref()),
        status: Some("completed".to_string()),
        timestamp: timestamp.into(),
        metadata,
    }
}

pub fn select_context_messages(
    messages: &[SessionMessage],
    limit: Option<usize>,
) -> Vec<&SessionMessage> {
    let mut filtered: Vec<_> = messages
        .iter()
        .filter(|message| session_message_in_context(&message.role))
        .collect();

    if let Some(limit) = limit
        && filtered.len() > limit
    {
        filtered = filtered
            .into_iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
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

pub fn select_messages_after_compact_boundary(
    messages: &[SessionMessage],
    limit: Option<usize>,
) -> Vec<&SessionMessage> {
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

fn tool_result_text(part: &NormalizedConversationPart) -> String {
    match part.result.as_ref() {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Object(map)) => {
            for key in ["formatted_output", "aggregated_output", "stdout", "text"] {
                if let Some(text) = map.get(key).and_then(|value| value.as_str())
                    && !text.trim().is_empty()
                {
                    return text.to_string();
                }
            }
            serde_json::to_string(&Value::Object(map.clone()))
                .unwrap_or_else(|_| "null".to_string())
        }
        Some(value) => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
        None => String::new(),
    }
}

pub fn restore_tool_message(message: &SessionMessage) -> Option<(String, String, String)> {
    if message.role == "tool" {
        let metadata = message.metadata.as_ref()?;
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
        return Some((tool_call_id, tool_name, session_message_text(message)));
    }

    message.content.iter().find_map(|part| {
        if part.part_type != "tool_result" {
            return None;
        }
        Some((
            part.id.clone().unwrap_or_default(),
            part.name.clone().unwrap_or_default(),
            tool_result_text(part),
        ))
    })
}

impl SessionStorage {
    /// PostgreSQL-backed test constructor. The path is only an isolation namespace.
    #[doc(hidden)]
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let (postgres, lease) = crate::test_postgres::isolated_postgres(&data_dir)
            .expect("SessionStorage PostgreSQL test runtime");
        let mut storage =
            Self::new_cloud(&data_dir, postgres).expect("SessionStorage PostgreSQL test schema");
        storage._test_postgres_lease = Some(lease);
        storage
    }

    pub fn new_cloud(data_dir: impl AsRef<Path>, postgres: CloudPgRuntime) -> HoneResult<Self> {
        let dir = data_dir.as_ref().to_path_buf();
        let storage = Arc::new(CloudPgSessionIndex::new(postgres)?) as Arc<dyn SessionIndex>;
        Ok(Self {
            data_dir: dir,
            storage,
            _test_postgres_lease: None,
        })
    }

    /// 测试 / runtime 层可以注入自定义的 `SessionIndex`（例如 mock）。
    /// 生产流程走 `with_options` 即可，不需要用这个构造器。
    pub fn with_custom_index(data_dir: impl AsRef<Path>, index: Arc<dyn SessionIndex>) -> Self {
        let dir = data_dir.as_ref().to_path_buf();
        Self {
            data_dir: dir,
            storage: index,
            _test_postgres_lease: None,
        }
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

        let now = hone_core::local_now_rfc3339();
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
                    frozen_time_local: hone_core::local_now_rfc3339(),
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

    /// 从权威后端加载会话。
    pub fn load_session(&self, session_id: &str) -> hone_core::HoneResult<Option<Session>> {
        self.storage.load(session_id)
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

        session.messages.push(session_message_from_text(
            role,
            content,
            hone_core::local_now_rfc3339(),
            metadata,
        ));
        session.updated_at = hone_core::local_now_rfc3339();
        session.version = default_session_version();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    /// 仅当最后一条消息的 `role` 与文本都匹配时，将其原子回滚。
    ///
    /// 主要用于 scheduler 在判定“本轮不发送”后，撤回刚刚通过通用成功路径
    /// 落入 direct session 的 assistant final，避免污染真实用户上下文。
    pub fn remove_last_message_if_matches(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> hone_core::HoneResult<bool> {
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(false);
        };

        let Some(last) = session.messages.last() else {
            return Ok(false);
        };

        if last.role != role || session_message_text(last).trim() != content.trim() {
            return Ok(false);
        }

        session.messages.pop();
        session.updated_at = hone_core::local_now_rfc3339();
        session.version = default_session_version();
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
        self.storage.list()
    }

    /// 查询某渠道内「最后一条是 user、没有 assistant 回复」的会话，
    /// 用于 channel 重启时恢复未完成请求。
    /// 只有启用了 `SessionIndex` 实现才能给出结果，否则返回空切片。
    pub fn find_interrupted_sessions(
        &self,
        channel: &str,
        updated_after_rfc3339: &str,
        updated_before_rfc3339: &str,
    ) -> hone_core::HoneResult<Vec<InterruptedSessionInfo>> {
        self.storage
            .find_interrupted(channel, updated_after_rfc3339, updated_before_rfc3339)
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
            session.updated_at = hone_core::local_now_rfc3339();
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
        session.updated_at = hone_core::local_now_rfc3339();
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
        session.updated_at = hone_core::local_now_rfc3339();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    pub fn append_session_messages(
        &self,
        session_id: &str,
        messages: Vec<SessionMessage>,
    ) -> hone_core::HoneResult<bool> {
        let lock = get_session_lock(session_id);
        let _guard = lock.lock().unwrap();

        let Some(mut session) = self.load_session(session_id)? else {
            return Ok(false);
        };

        session.messages.extend(messages);
        session.updated_at = hone_core::local_now_rfc3339();
        session.version = default_session_version();
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
        session.updated_at = hone_core::local_now_rfc3339();
        session.version = default_session_version();
        self.write_session(session_id, &session)?;

        Ok(true)
    }

    fn write_session(&self, session_id: &str, session: &Session) -> hone_core::HoneResult<()> {
        let normalized = validate_storage_component(session_id).ok_or_else(|| {
            hone_core::HoneError::Config("session_id 包含非法路径组件".to_string())
        })?;
        self.storage.upsert(
            &self
                .data_dir
                .join("cloud_sessions")
                .join(format!("{normalized}.json")),
            session,
        )
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
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct RecordingSessionIndex {
        sessions: Mutex<BTreeMap<String, Session>>,
    }

    impl SessionIndex for RecordingSessionIndex {
        fn upsert(&self, _source_path: &Path, session: &Session) -> HoneResult<()> {
            self.sessions
                .lock()
                .expect("recording index lock")
                .insert(session.id.clone(), session.clone());
            Ok(())
        }

        fn load(&self, session_id: &str) -> HoneResult<Option<Session>> {
            Ok(self
                .sessions
                .lock()
                .expect("recording index lock")
                .get(session_id)
                .cloned())
        }

        fn list(&self) -> HoneResult<Vec<Session>> {
            Ok(self
                .sessions
                .lock()
                .expect("recording index lock")
                .values()
                .cloned()
                .collect())
        }

        fn find_interrupted(
            &self,
            _channel: &str,
            _updated_after_rfc3339: &str,
            _updated_before_rfc3339: &str,
        ) -> HoneResult<Vec<InterruptedSessionInfo>> {
            Ok(Vec::new())
        }
    }

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()))
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
        assert_eq!(session.version, 4);
        assert!(!session.runtime.prompt.frozen_time_local.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn load_session_missing_returns_none() {
        let root = make_temp_dir("hone_memory_test_missing");
        let storage = SessionStorage::new(&root);
        let session = storage.load_session("does-not-exist").expect("load");
        assert!(session.is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn get_messages_missing_session_returns_empty() {
        let root = make_temp_dir("hone_memory_test_missing_get");
        let storage = SessionStorage::new(&root);
        let msgs = storage.get_messages("does-not-exist", None).expect("get");
        assert!(msgs.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
        let contents: Vec<_> = msgs.iter().map(session_message_text).collect();
        assert_eq!(contents, vec!["m2", "m3"]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn remove_last_message_if_matches_only_removes_matching_tail() {
        let root = make_temp_dir("hone_memory_test_remove_last_matching");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        storage
            .add_message(&session_id, "user", "[定时任务触发] TEM", None)
            .expect("add user");
        storage
            .add_message(
                &session_id,
                "assistant",
                "TEM 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送",
                None,
            )
            .expect("add assistant");

        let removed = storage
            .remove_last_message_if_matches(
                &session_id,
                "assistant",
                "TEM 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送",
            )
            .expect("remove");
        assert!(removed);

        let msgs = storage.get_messages(&session_id, None).expect("get");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(session_message_text(&msgs[0]), "[定时任务触发] TEM");

        let removed_again = storage
            .remove_last_message_if_matches(
                &session_id,
                "assistant",
                "TEM 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送",
            )
            .expect("remove again");
        assert!(!removed_again);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn replace_messages_overwrites_existing() {
        let root = make_temp_dir("hone_memory_test_replace");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        storage
            .add_message(&session_id, "user", "before", None)
            .expect("add");

        let new_messages = vec![session_message_from_text(
            "assistant",
            "after",
            hone_core::local_now_rfc3339(),
            None,
        )];

        let ok = storage
            .replace_messages(&session_id, new_messages)
            .expect("replace");
        assert!(ok);

        let msgs = storage.get_messages(&session_id, None).expect("get");
        assert_eq!(msgs.len(), 1);
        assert_eq!(session_message_text(&msgs[0]), "after");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn replace_messages_with_summary_updates_both() {
        let root = make_temp_dir("hone_memory_test_replace_summary");
        let storage = SessionStorage::new(&root);
        let session_id = storage.create_session(None, None, None).expect("create");

        let new_messages = vec![session_message_from_text(
            "assistant",
            "after",
            hone_core::local_now_rfc3339(),
            None,
        )];

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
            session_message_from_text("user", "before", hone_core::local_now_rfc3339(), None),
            session_message_from_text(
                "system",
                "Conversation compacted",
                hone_core::local_now_rfc3339(),
                Some(build_compact_boundary_metadata("auto", 3, 5)),
            ),
            session_message_from_text(
                "user",
                "【Compact Summary】\nsummary",
                hone_core::local_now_rfc3339(),
                Some(build_compact_summary_metadata("auto")),
            ),
            session_message_from_text("assistant", "after", hone_core::local_now_rfc3339(), None),
        ];

        let selected = select_messages_after_compact_boundary(&messages, None);
        let contents: Vec<_> = selected.iter().map(|m| session_message_text(m)).collect();
        assert_eq!(contents, vec!["【Compact Summary】\nsummary", "after"]);
        assert_eq!(
            latest_compact_summary(&messages).map(session_message_text),
            Some("【Compact Summary】\nsummary".to_string())
        );
    }

    #[test]
    fn select_context_messages_keeps_tool_role() {
        let messages = vec![
            session_message_from_text("system", "ignore", hone_core::local_now_rfc3339(), None),
            session_message_from_text("user", "u1", hone_core::local_now_rfc3339(), None),
            session_message_from_text(
                "tool",
                "t1",
                hone_core::local_now_rfc3339(),
                Some(HashMap::from([(
                    "tool_name".to_string(),
                    Value::String("web_search".to_string()),
                )])),
            ),
            session_message_from_text("assistant", "a1", hone_core::local_now_rfc3339(), None),
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
        let message = session_message_from_text(
            "tool",
            "{\"ok\":true}",
            hone_core::local_now_rfc3339(),
            Some(metadata),
        );
        let restored = restore_tool_message(&message).expect("restore tool");
        assert_eq!(restored.0, "call_1");
        assert_eq!(restored.1, "web_search");
        assert_eq!(restored.2, "{\"ok\":true}");
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_write_is_visible_on_read() {
        let root = make_temp_dir("hone_memory_test_postgres_visibility");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "alice", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        storage
            .add_message(&session_id, "user", "hello postgres", None)
            .expect("append");
        let loaded = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(session_message_text(&loaded.messages[0]), "hello postgres");
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_sessions_survive_a_second_storage_handle() {
        let root = make_temp_dir("hone_memory_test_postgres_second_handle");
        let first = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "alice", None::<String>).expect("actor");
        let session_id = first.create_session_for_actor(&actor).expect("create");
        first
            .add_message(&session_id, "user", "persisted", None)
            .expect("append");
        let second = SessionStorage::new(&root);
        let loaded = second
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(session_message_text(&loaded.messages[0]), "persisted");
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_list_reflects_updates_from_another_handle() {
        let root = make_temp_dir("hone_memory_test_postgres_list");
        let first = SessionStorage::new(&root);
        let second = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "postgres-list", None::<String>).expect("actor");
        let session_id = first.create_session_for_actor(&actor).expect("create");
        first
            .add_message(&session_id, "assistant", "visible", None)
            .expect("append");
        let listed = second.list_sessions().expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(session_message_text(&listed[0].messages[0]), "visible");
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_runtime_does_not_read_local_json() {
        let root = make_temp_dir("hone_memory_test_postgres_authority");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "bob", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        storage
            .add_message(&session_id, "user", "hello postgres", None)
            .expect("append");
        std::fs::create_dir_all(&root).expect("local dir");
        std::fs::write(root.join(format!("{session_id}.json")), "not valid json")
            .expect("local poison file");
        let session = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session_message_text(&session.messages[0]), "hello postgres");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn cloud_runtime_backend_creates_no_local_shadow_artifact() {
        let root = make_temp_dir("hone_memory_test_no_local_shadow");
        let runtime_index = Arc::new(RecordingSessionIndex::default()) as Arc<dyn SessionIndex>;
        let storage = SessionStorage::with_custom_index(&root, runtime_index.clone());
        let actor = ActorIdentity::new("web", "cloud-shadow", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        storage
            .add_message(&session_id, "user", "hello postgres only", None)
            .expect("append");
        let runtime_session = runtime_index
            .load(&session_id)
            .expect("runtime load")
            .expect("runtime session");
        assert_eq!(runtime_session.messages.len(), 1);
        assert!(
            !root.exists(),
            "PG-only session writes must not create a local artifact"
        );
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn upsert_session_persists_rows() {
        let root = make_temp_dir("hone_session_pg_upsert");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "alice", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        storage
            .add_message(&session_id, "user", "hello", None)
            .expect("append");
        let loaded = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(loaded.id, session_id);
        assert_eq!(loaded.messages.len(), 1);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn upsert_session_replaces_old_rows_and_stores_message_metadata_columns() {
        let root = make_temp_dir("hone_session_pg_replace");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("feishu", "replace", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        let metadata = HashMap::from([(
            "tool_name".to_string(),
            Value::String("web_search".to_string()),
        )]);
        storage
            .add_message(&session_id, "tool", "first", Some(metadata.clone()))
            .expect("append");
        storage
            .replace_messages(
                &session_id,
                vec![session_message_from_text(
                    "tool",
                    "replacement",
                    hone_core::local_now_rfc3339(),
                    Some(metadata),
                )],
            )
            .expect("replace");
        let loaded = storage
            .load_session(&session_id)
            .expect("load")
            .expect("session");
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(session_message_text(&loaded.messages[0]), "replacement");
        assert_eq!(
            loaded.messages[0]
                .metadata
                .as_ref()
                .and_then(|value| value.get("tool_name"))
                .and_then(Value::as_str),
            Some("web_search")
        );
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn upsert_session_accepts_cloud_shadow_source_path_without_local_file() {
        let root = make_temp_dir("hone_session_pg_no_local_source");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("web", "no-local-source", None::<String>).expect("actor");
        let session_id = storage.create_session_for_actor(&actor).expect("create");
        assert!(storage.load_session(&session_id).expect("load").is_some());
        assert!(!root.exists());
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn list_sessions_orders_by_updated_at_desc() {
        let root = make_temp_dir("hone_session_pg_list_order");
        let storage = SessionStorage::new(&root);
        for (id, updated_at) in [
            ("older", "2026-01-01T00:00:00+08:00"),
            ("newer", "2026-01-02T00:00:00+08:00"),
        ] {
            let session = Session {
                version: default_session_version(),
                id: id.to_string(),
                actor: None,
                session_identity: None,
                created_at: updated_at.to_string(),
                updated_at: updated_at.to_string(),
                messages: Vec::new(),
                metadata: HashMap::new(),
                runtime: SessionRuntimeState::default(),
                summary: None,
            };
            storage.write_session(id, &session).expect("write");
        }
        let listed = storage.list_sessions().expect("list");
        assert_eq!(
            listed
                .iter()
                .map(|session| session.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn list_sessions_skips_unreadable_rows() {
        let root = make_temp_dir("hone_session_pg_invalid_json");
        let (postgres, _lease) = crate::test_postgres::isolated_postgres(&root).expect("postgres");
        let invalid_postgres = postgres.clone();
        run_cloud_session(async move {
            invalid_postgres
                .upsert_session_record(
                    "invalid",
                    "invalid",
                    serde_json::json!({"not": "a session"}),
                )
                .await
        })
        .expect("insert malformed session value");
        let storage = SessionStorage::new_cloud(&root, postgres).expect("storage");
        assert!(storage.list_sessions().expect("list").is_empty());
    }
}
