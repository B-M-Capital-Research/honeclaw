use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;

use crate::logging::LogBuffer;

/// 调度器 → 浏览器的主动推送事件
#[derive(Debug, Clone, Serialize)]
pub struct PushEvent {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub event: String,
    pub data: Value,
}

/// iMessage Bot → Web Console 事件推送请求体
#[derive(Debug, Deserialize)]
pub struct IMessageEventRequest {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub event_type: String,
    pub data: Value,
}

pub struct AppState {
    pub core: Arc<hone_channels::HoneBotCore>,
    pub push_tx: broadcast::Sender<PushEvent>,
    pub http_client: reqwest::Client,
    pub log_buffer: LogBuffer,
    pub deployment_mode: String,
    pub auth: AuthState,
}

pub struct AuthState {
    pub bearer_token: Option<String>,
    pub sse_tickets: Mutex<HashMap<String, Instant>>,
}
