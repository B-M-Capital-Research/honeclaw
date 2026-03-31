use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeHeartbeatRequest {
    pub channel: String,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeHeartbeatProcess {
    pub channel: String,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct HeartbeatRegistry {
    entries: Mutex<HashMap<String, HashMap<u32, RuntimeHeartbeatProcess>>>,
}

impl HeartbeatRegistry {
    pub fn record(&self, request: RuntimeHeartbeatRequest) {
        let mut entries = self.entries.lock().unwrap();
        let channel_entries = entries.entry(request.channel.clone()).or_default();
        channel_entries.insert(
            request.pid,
            RuntimeHeartbeatProcess {
                channel: request.channel,
                pid: request.pid,
                started_at: request.started_at,
                updated_at: request.updated_at,
            },
        );
    }

    pub fn channel_processes(&self, channel: &str) -> Vec<RuntimeHeartbeatProcess> {
        let mut entries = self.entries.lock().unwrap();
        prune_registry(&mut entries);
        entries
            .get(channel)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default()
    }
}

fn prune_registry(entries: &mut HashMap<String, HashMap<u32, RuntimeHeartbeatProcess>>) {
    let cutoff = Utc::now() - chrono::Duration::hours(24);
    entries.retain(|_, processes| {
        processes.retain(|_, process| process.updated_at >= cutoff);
        !processes.is_empty()
    });
}

pub struct AppState {
    pub core: Arc<hone_channels::HoneBotCore>,
    pub push_tx: broadcast::Sender<PushEvent>,
    pub http_client: reqwest::Client,
    pub log_buffer: LogBuffer,
    pub deployment_mode: String,
    pub auth: AuthState,
    pub heartbeat_registry: HeartbeatRegistry,
}

pub struct AuthState {
    pub bearer_token: Option<String>,
    pub sse_tickets: Mutex<HashMap<String, Instant>>,
}
