use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use chrono::Utc;
use serde_json::json;

use hone_core::config::HoneConfig;
use hone_core::{
    HEARTBEAT_STALE_AFTER_SECS, ProcessHeartbeatSnapshot, read_process_heartbeat,
    runtime_heartbeat_path,
};

use crate::state::AppState;
use crate::types::{ChannelStatusInfo, MetaInfo};

const API_VERSION: &str = "desktop-v1";

pub(crate) async fn handle_meta(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(json!(MetaInfo {
        name: "Hone".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        channel: "imessage".to_string(),
        supports_imessage: cfg!(target_os = "macos"),
        api_version: API_VERSION.to_string(),
        capabilities: meta_capabilities(&state.core.config, &state.deployment_mode),
        deployment_mode: state.deployment_mode.clone(),
    }))
}

pub(crate) async fn handle_channels(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = &state.core.config;
    let channels = vec![
        ChannelStatusInfo {
            id: "web".to_string(),
            label: "Web".to_string(),
            enabled: true,
            running: true,
            status: "running".to_string(),
            pid: Some(std::process::id()),
            last_heartbeat_at: None,
            detail: "Web 控制台与 API 服务".to_string(),
        },
        external_channel_status(
            "imessage",
            "iMessage",
            config.imessage.enabled,
            "消息数据库轮询监听",
            (!cfg!(target_os = "macos")).then_some("当前系统非 macOS"),
        ),
        external_channel_status(
            "discord",
            "Discord",
            config.discord.enabled,
            "Discord Gateway 监听中",
            None,
        ),
        external_channel_status(
            "feishu",
            "Feishu",
            config.feishu.enabled,
            "Feishu 长连接渠道",
            None,
        ),
        external_channel_status(
            "telegram",
            "Telegram",
            config.telegram.enabled,
            "Telegram Bot 监听中",
            None,
        ),
    ];

    Json(serde_json::to_value(&channels).unwrap_or(json!([])))
}

fn meta_capabilities(config: &HoneConfig, deployment_mode: &str) -> Vec<String> {
    let mut capabilities = vec![
        "channels".to_string(),
        "users".to_string(),
        "history".to_string(),
        "chat".to_string(),
        "sse.events".to_string(),
        "logs".to_string(),
        "skills".to_string(),
        "cron_jobs".to_string(),
        "portfolio".to_string(),
        "research".to_string(),
        "llm_audit".to_string(),
    ];

    if deployment_mode == "local" {
        capabilities.push("local_file_proxy".to_string());
    }

    if cfg!(target_os = "macos") && config.imessage.enabled {
        capabilities.push("imessage".to_string());
    }

    capabilities.sort();
    capabilities
}

fn runtime_dir() -> std::path::PathBuf {
    let dir = std::env::var("HONE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("./data/runtime"));
    dir
}

fn read_channel_heartbeat(channel: &str) -> Option<ProcessHeartbeatSnapshot> {
    let path = runtime_heartbeat_path(&runtime_dir(), channel);
    read_process_heartbeat(&path).ok()
}

fn heartbeat_is_fresh(snapshot: &ProcessHeartbeatSnapshot) -> bool {
    let age = Utc::now().signed_duration_since(snapshot.updated_at);
    age >= chrono::TimeDelta::zero()
        && age
            <= chrono::TimeDelta::from_std(Duration::from_secs(HEARTBEAT_STALE_AFTER_SECS))
                .unwrap_or_else(|_| chrono::TimeDelta::seconds(HEARTBEAT_STALE_AFTER_SECS as i64))
}

fn external_channel_status(
    id: &str,
    label: &str,
    enabled: bool,
    detail: &str,
    unsupported_reason: Option<&str>,
) -> ChannelStatusInfo {
    if let Some(reason) = unsupported_reason {
        return ChannelStatusInfo {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            running: false,
            status: "unsupported".to_string(),
            pid: None,
            last_heartbeat_at: None,
            detail: reason.to_string(),
        };
    }

    if !enabled {
        return ChannelStatusInfo {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            running: false,
            status: "disabled".to_string(),
            pid: None,
            last_heartbeat_at: None,
            detail: "config.yaml 中未启用".to_string(),
        };
    }

    match read_channel_heartbeat(id) {
        Some(snapshot) if heartbeat_is_fresh(&snapshot) => ChannelStatusInfo {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            running: true,
            status: "running".to_string(),
            pid: Some(snapshot.pid),
            last_heartbeat_at: Some(snapshot.updated_at.to_rfc3339()),
            detail: format!("{detail}（pid={}）", snapshot.pid),
        },
        Some(snapshot) => ChannelStatusInfo {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            running: false,
            status: "stopped".to_string(),
            pid: Some(snapshot.pid),
            last_heartbeat_at: Some(snapshot.updated_at.to_rfc3339()),
            detail: format!(
                "心跳超时（pid={}，last_seen={}）",
                snapshot.pid, snapshot.updated_at
            ),
        },
        None => ChannelStatusInfo {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            running: false,
            status: "stopped".to_string(),
            pid: None,
            last_heartbeat_at: None,
            detail: "未检测到渠道心跳".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration as ChronoDuration, Utc};

    use super::*;

    fn heartbeat_with_age(seconds: i64) -> ProcessHeartbeatSnapshot {
        let now = Utc::now();
        ProcessHeartbeatSnapshot {
            channel: "discord".to_string(),
            pid: 1234,
            started_at: now - ChronoDuration::seconds(seconds + 10),
            updated_at: now - ChronoDuration::seconds(seconds),
        }
    }

    #[test]
    fn fresh_heartbeat_is_running() {
        assert!(heartbeat_is_fresh(&heartbeat_with_age(30)));
    }

    #[test]
    fn stale_heartbeat_is_not_running() {
        assert!(!heartbeat_is_fresh(&heartbeat_with_age(
            HEARTBEAT_STALE_AFTER_SECS as i64 + 1
        )));
    }
}
