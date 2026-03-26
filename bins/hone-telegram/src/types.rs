use std::collections::HashMap;
use std::sync::Arc;

use teloxide::types::Message;
use tokio::sync::Mutex;

#[derive(Clone)]
pub(crate) struct TelegramAppState {
    pub(crate) dedup: hone_channels::ingress::MessageDeduplicator,
    pub(crate) session_locks: hone_channels::ingress::SessionLockRegistry,
    pub(crate) scope_resolver: hone_channels::ingress::ActorScopeResolver,
    pub(crate) pretrigger: hone_channels::ingress::GroupPretriggerWindowRegistry,
    pub(crate) media_groups: MediaGroupBuffer,
}

#[derive(Clone, Default)]
pub(crate) struct MediaGroupBuffer {
    pending: Arc<Mutex<HashMap<String, Vec<Message>>>>,
}

impl MediaGroupBuffer {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) async fn push(&self, group_id: &str, message: Message) -> bool {
        let mut guard = self.pending.lock().await;
        let entry = guard.entry(group_id.to_string()).or_default();
        let is_first = entry.is_empty();
        entry.push(message);
        is_first
    }

    pub(crate) async fn take(&self, group_id: &str) -> Vec<Message> {
        let mut guard = self.pending.lock().await;
        let mut messages = guard.remove(group_id).unwrap_or_default();
        messages.sort_by_key(|message| message.id.0);
        messages
    }
}
