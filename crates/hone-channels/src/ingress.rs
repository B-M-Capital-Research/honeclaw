use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use hone_core::{ActorIdentity, SessionIdentity};
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
