use std::sync::Arc;

use hone_channels::HoneBotCore;
use hone_channels::ingress::{ActorScopeResolver, MessageDeduplicator, SessionLockRegistry};
use serde::Deserialize;

use super::client::FeishuApiClient;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) core: Arc<HoneBotCore>,
    pub(crate) facade: FeishuApiClient,
    pub(crate) dedup: MessageDeduplicator,
    pub(crate) session_locks: SessionLockRegistry,
    pub(crate) scope_resolver: ActorScopeResolver,
}

pub(crate) struct FeishuEventHandler {
    pub(crate) state: Arc<AppState>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FeishuIncomingMessage {
    pub(crate) message_id: String,
    pub(crate) chat_id: String,
    pub(crate) open_id: String,
    #[serde(default)]
    pub(crate) message_type: Option<String>,
    #[serde(default)]
    pub(crate) email: Option<String>,
    #[serde(default)]
    pub(crate) mobile: Option<String>,
    #[serde(default)]
    pub(crate) attachments: Vec<FeishuIncomingAttachment>,
    pub(crate) text: String,
    #[serde(default)]
    pub(crate) chat_type: Option<String>,
    pub(crate) has_mention: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FeishuIncomingAttachment {
    pub(crate) filename: String,
    #[serde(default)]
    pub(crate) content_type: Option<String>,
    #[serde(default)]
    pub(crate) size: Option<u32>,
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) local_path: Option<String>,
    #[serde(default)]
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderedMessage {
    pub(crate) msg_type: &'static str,
    pub(crate) content: String,
}
