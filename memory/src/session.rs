//! 会话存储 — JSON 文件

use chrono::{DateTime, FixedOffset};
use hone_core::agent::ToolCallMade;
use hone_core::{ActorIdentity, SessionIdentity, beijing_now};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// 会话存储管理器
pub struct SessionStorage {
    data_dir: PathBuf,
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
    let mut metadata = HashMap::new();
    metadata.insert("tool_name".to_string(), Value::String(call.name.clone()));
    if let Some(tool_call_id) = &call.tool_call_id {
        metadata.insert(
            "tool_call_id".to_string(),
            Value::String(tool_call_id.clone()),
        );
    }
    metadata.insert("tool_arguments".to_string(), call.arguments.clone());
    metadata
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
        let dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir).ok();
        Self { data_dir: dir }
    }

    /// 创建新会话
    pub fn create_session(
        &self,
        session_id: Option<&str>,
        actor: Option<ActorIdentity>,
        session_identity: Option<SessionIdentity>,
    ) -> hone_core::HoneResult<String> {
        let id = session_id.map(|s| s.to_string()).unwrap_or_else(|| {
            session_identity
                .as_ref()
                .map(SessionIdentity::session_id)
                .or_else(|| actor.as_ref().map(ActorIdentity::session_id))
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
        });

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

        let path = self.data_dir.join(format!("{id}.json"));
        let content = serde_json::to_string_pretty(&session)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        std::fs::write(&path, content)?;

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
        let path = self.data_dir.join(format!("{session_id}.json"));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&content)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        Ok(Some(session))
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

        let path = self.data_dir.join(format!("{session_id}.json"));
        let json = serde_json::to_string_pretty(&session)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        std::fs::write(&path, json)?;

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
        let path = self.data_dir.join(format!("{session_id}.json"));
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| hone_core::HoneError::Serialization(e.to_string()))?;
        std::fs::write(&path, json)?;
        Ok(())
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
}
