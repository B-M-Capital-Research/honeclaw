use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hone_core::{ActorIdentity, HoneResult, SessionIdentity};
use hone_memory::SessionStorage;
use serde_json::Value;

use crate::agent_session::MessageMetadata;
use crate::attachments::ReceivedAttachment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatMode {
    /// 这条入站消息来自私聊。
    /// 仅表示“消息形态”，不表示持久化时一定写入用户独占 session。
    Direct,
    /// 这条入站消息来自群聊。
    /// 它只影响入口期策略（如是否允许 cron、是否启用群 guard），
    /// 不直接决定最终 session_id；session 归属应由独立的 session identity 解析。
    Group,
}

impl ChatMode {
    pub fn cron_allowed(self) -> bool {
        matches!(self, Self::Direct)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GroupTrigger {
    pub direct_mention: bool,
    pub reply_to_bot: bool,
    pub question_signal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupTriggerMode {
    MentionOnly,
    MentionOrQuestion,
    All,
}

impl GroupTriggerMode {
    pub fn from_config_value(raw: &str) -> Self {
        match raw.trim() {
            "mention_only" => Self::MentionOnly,
            "all" => Self::All,
            _ => Self::MentionOrQuestion,
        }
    }

    pub fn should_trigger(self, trigger: &GroupTrigger) -> bool {
        match self {
            Self::MentionOnly => trigger.direct_mention || trigger.reply_to_bot,
            Self::MentionOrQuestion => {
                trigger.direct_mention || trigger.reply_to_bot || trigger.question_signal
            }
            Self::All => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IncomingEnvelope {
    pub message_id: Option<String>,
    pub actor: ActorIdentity,
    pub session_identity: SessionIdentity,
    pub session_id: String,
    pub channel_target: String,
    pub chat_mode: ChatMode,
    pub text: String,
    pub attachments: Vec<ReceivedAttachment>,
    pub trigger: GroupTrigger,
    pub recv_extra: Option<String>,
    pub session_metadata: Option<HashMap<String, Value>>,
    pub message_metadata: MessageMetadata,
}

impl IncomingEnvelope {
    pub fn cron_allowed(&self) -> bool {
        self.chat_mode.cron_allowed()
    }

    pub fn is_group(&self) -> bool {
        matches!(self.chat_mode, ChatMode::Group)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferedGroupMessage {
    pub channel: String,
    pub message_id: String,
    pub speaker_label: String,
    pub text: String,
    pub received_at: SystemTime,
}

impl BufferedGroupMessage {
    pub fn new(
        channel: impl Into<String>,
        message_id: impl Into<String>,
        speaker_label: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            channel: channel.into(),
            message_id: message_id.into(),
            speaker_label: speaker_label.into(),
            text: text.into(),
            received_at: SystemTime::now(),
        }
    }

    pub fn rendered_text(&self) -> String {
        format!("[{}] {}", self.speaker_label.trim(), self.text.trim())
    }
}

#[derive(Clone)]
pub struct GroupPretriggerWindowRegistry {
    inner: Arc<tokio::sync::Mutex<HashMap<String, VecDeque<BufferedGroupMessage>>>>,
    max_messages: usize,
    max_age: Duration,
}

impl GroupPretriggerWindowRegistry {
    pub fn new(max_messages: usize, max_age: Duration) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            max_messages: max_messages.max(1),
            max_age,
        }
    }

    pub async fn push(&self, group_session_id: &str, message: BufferedGroupMessage) {
        let mut guard = self.inner.lock().await;
        let entry = guard.entry(group_session_id.to_string()).or_default();
        prune_buffer(entry, self.max_age);
        if entry
            .iter()
            .any(|existing| existing.message_id == message.message_id)
        {
            return;
        }
        entry.push_back(message);
        while entry.len() > self.max_messages {
            let _ = entry.pop_front();
        }
    }

    pub async fn take_recent(
        &self,
        group_session_id: &str,
        exclude_message_id: Option<&str>,
    ) -> Vec<BufferedGroupMessage> {
        let mut guard = self.inner.lock().await;
        let Some(mut entry) = guard.remove(group_session_id) else {
            return Vec::new();
        };
        prune_buffer(&mut entry, self.max_age);
        entry
            .into_iter()
            .filter(|message| Some(message.message_id.as_str()) != exclude_message_id)
            .collect()
    }
}

fn prune_buffer(buffer: &mut VecDeque<BufferedGroupMessage>, max_age: Duration) {
    let now = SystemTime::now();
    while let Some(front) = buffer.front() {
        let age = now
            .duration_since(front.received_at)
            .unwrap_or_else(|_| Duration::from_secs(0));
        if age > max_age {
            let _ = buffer.pop_front();
        } else {
            break;
        }
    }
}

pub fn persist_buffered_group_messages(
    storage: &SessionStorage,
    session_id: &str,
    messages: &[BufferedGroupMessage],
) -> HoneResult<usize> {
    if messages.is_empty() {
        return Ok(0);
    }

    let mut existing_message_ids = HashSet::new();
    for message in storage.get_messages(session_id, None)? {
        if let Some(message_id) = message
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("channel_message_id"))
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            existing_message_ids.insert(message_id.to_string());
        }
    }

    let mut persisted = 0usize;
    for message in messages {
        if !existing_message_ids.insert(message.message_id.clone()) {
            continue;
        }

        let mut metadata = HashMap::new();
        metadata.insert("pretrigger_buffered".to_string(), Value::Bool(true));
        metadata.insert(
            "channel_message_id".to_string(),
            Value::String(message.message_id.clone()),
        );
        metadata.insert(
            "speaker_label".to_string(),
            Value::String(message.speaker_label.clone()),
        );
        metadata.insert(
            "channel".to_string(),
            Value::String(message.channel.clone()),
        );
        if let Ok(duration) = message.received_at.duration_since(UNIX_EPOCH) {
            metadata.insert(
                "buffered_at_unix_ms".to_string(),
                Value::Number(serde_json::Number::from(duration.as_millis() as u64)),
            );
        }

        let _ =
            storage.add_message(session_id, "user", &message.rendered_text(), Some(metadata))?;
        persisted += 1;
    }

    Ok(persisted)
}

#[derive(Debug, Clone)]
pub struct ActorScopeResolver {
    channel: String,
}

impl ActorScopeResolver {
    pub fn new(channel: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
        }
    }

    pub fn direct(
        &self,
        user_id: &str,
        channel_target: impl Into<String>,
    ) -> hone_core::HoneResult<(ActorIdentity, String, ChatMode)> {
        Ok((
            ActorIdentity::new(&self.channel, user_id, None::<String>)?,
            channel_target.into(),
            ChatMode::Direct,
        ))
    }

    pub fn group(
        &self,
        user_id: &str,
        channel_scope: impl Into<String>,
        channel_target: impl Into<String>,
    ) -> hone_core::HoneResult<(ActorIdentity, String, ChatMode)> {
        Ok((
            ActorIdentity::new(&self.channel, user_id, Some(channel_scope.into()))?,
            channel_target.into(),
            ChatMode::Group,
        ))
    }
}

struct DedupState {
    ttl: Duration,
    max_entries: usize,
    order: VecDeque<(String, Instant)>,
    seen: HashMap<String, Instant>,
}

impl DedupState {
    fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            ttl,
            max_entries,
            order: VecDeque::new(),
            seen: HashMap::new(),
        }
    }

    fn check_and_store(&mut self, key: &str) -> bool {
        let now = Instant::now();
        self.prune(now);
        if let Some(ts) = self.seen.get(key)
            && now.duration_since(*ts) <= self.ttl
        {
            return true;
        }
        self.seen.insert(key.to_string(), now);
        self.order.push_back((key.to_string(), now));
        self.prune(now);
        false
    }

    fn prune(&mut self, now: Instant) {
        loop {
            let (key_owned, ts_owned) = match self.order.front() {
                Some((key, ts)) => (key.clone(), *ts),
                None => break,
            };
            let expired = now.duration_since(ts_owned) > self.ttl;
            let stale = match self.seen.get(&key_owned) {
                Some(current) => *current != ts_owned,
                None => true,
            };
            let overflow = self.seen.len() > self.max_entries;
            if expired || stale || overflow {
                let _ = self.order.pop_front();
                if !stale {
                    self.seen.remove(&key_owned);
                }
                continue;
            }
            break;
        }
    }
}

#[derive(Clone)]
pub struct MessageDeduplicator {
    inner: Arc<Mutex<DedupState>>,
}

impl MessageDeduplicator {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DedupState::new(ttl, max_entries.max(1)))),
        }
    }

    pub fn is_duplicate(&self, key: &str) -> bool {
        let mut guard = self.inner.lock().expect("dedup state poisoned");
        guard.check_and_store(key)
    }
}

#[derive(Clone, Default)]
pub struct SessionLockRegistry {
    inner: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl SessionLockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn lock(&self, session_id: &str) -> tokio::sync::OwnedMutexGuard<()> {
        let lock = {
            let mut guard = self.inner.lock().expect("session lock registry poisoned");
            guard
                .entry(session_id.to_string())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };
        lock.lock_owned().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::{ActorIdentity, SessionIdentity};

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()))
    }

    #[tokio::test]
    async fn pretrigger_window_keeps_latest_messages() {
        let registry = GroupPretriggerWindowRegistry::new(2, Duration::from_secs(300));
        registry
            .push(
                "group:1",
                BufferedGroupMessage::new("discord", "m1", "a", "one"),
            )
            .await;
        registry
            .push(
                "group:1",
                BufferedGroupMessage::new("discord", "m2", "b", "two"),
            )
            .await;
        registry
            .push(
                "group:1",
                BufferedGroupMessage::new("discord", "m3", "c", "three"),
            )
            .await;

        let buffered = registry.take_recent("group:1", None).await;
        let ids: Vec<_> = buffered
            .iter()
            .map(|message| message.message_id.as_str())
            .collect();
        assert_eq!(ids, vec!["m2", "m3"]);
    }

    #[tokio::test]
    async fn pretrigger_window_prunes_expired_messages() {
        let registry = GroupPretriggerWindowRegistry::new(10, Duration::from_secs(300));
        let mut old = BufferedGroupMessage::new("telegram", "m1", "a", "old");
        old.received_at = SystemTime::now()
            .checked_sub(Duration::from_secs(301))
            .expect("time");
        registry.push("group:1", old).await;
        registry
            .push(
                "group:1",
                BufferedGroupMessage::new("telegram", "m2", "b", "fresh"),
            )
            .await;

        let buffered = registry.take_recent("group:1", None).await;
        let ids: Vec<_> = buffered
            .iter()
            .map(|message| message.message_id.as_str())
            .collect();
        assert_eq!(ids, vec!["m2"]);
    }

    #[test]
    fn persist_buffered_messages_deduplicates_by_channel_message_id() {
        let root = make_temp_dir("hone_channels_buffered_persist");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
        let session_identity = SessionIdentity::group("discord", "g:1:c:2").expect("session");
        let session_id = storage
            .create_session(
                Some(&session_identity.session_id()),
                Some(actor),
                Some(session_identity),
            )
            .expect("create session");

        let mut metadata = HashMap::new();
        metadata.insert(
            "channel_message_id".to_string(),
            Value::String("m1".to_string()),
        );
        storage
            .add_message(&session_id, "user", "[a] existing", Some(metadata))
            .expect("seed message");

        let persisted = persist_buffered_group_messages(
            &storage,
            &session_id,
            &[
                BufferedGroupMessage::new("discord", "m1", "a", "dup"),
                BufferedGroupMessage::new("discord", "m2", "b", "fresh"),
            ],
        )
        .expect("persist buffered");

        assert_eq!(persisted, 1);
        let messages = storage
            .get_messages(&session_id, None)
            .expect("get messages");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].content, "[b] fresh");

        let _ = std::fs::remove_dir_all(root);
    }
}
