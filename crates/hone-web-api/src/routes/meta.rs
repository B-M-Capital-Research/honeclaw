use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use serde_json::json;

use hone_core::config::HoneConfig;
use hone_core::{
    HEARTBEAT_STALE_AFTER_SECS, ProcessHeartbeatSnapshot, read_process_heartbeat,
    runtime_heartbeat_path, scan_channel_processes,
};

use crate::state::AppState;
use crate::types::{ChannelProcessInfo, ChannelStatusInfo, MetaInfo};

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
            processes: vec![ChannelProcessInfo {
                pid: std::process::id(),
                running: true,
                started_at: Some(Utc::now().to_rfc3339()),
                last_heartbeat_at: Some(Utc::now().to_rfc3339()),
                managed_by_desktop: Some(true),
                source: Some("self".to_string()),
            }],
        },
        external_channel_status(
            &state,
            "imessage",
            "iMessage",
            config.imessage.enabled,
            "消息数据库轮询监听",
            config,
            (!cfg!(target_os = "macos")).then_some("当前系统非 macOS"),
        ),
        external_channel_status(
            &state,
            "discord",
            "Discord",
            config.discord.enabled,
            "Discord Gateway 监听中",
            config,
            None,
        ),
        external_channel_status(
            &state,
            "feishu",
            "Feishu",
            config.feishu.enabled,
            "Feishu 长连接渠道",
            config,
            None,
        ),
        external_channel_status(
            &state,
            "telegram",
            "Telegram",
            config.telegram.enabled,
            "Telegram Bot 监听中",
            config,
            None,
        ),
    ];

    Json(serde_json::to_value(&channels).unwrap_or(json!([])))
}

pub(crate) async fn handle_runtime_heartbeat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::state::RuntimeHeartbeatRequest>,
) -> impl IntoResponse {
    if payload.channel.trim().is_empty() || payload.pid == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid heartbeat payload" })),
        )
            .into_response();
    }

    state.heartbeat_registry.record(payload);
    (StatusCode::NO_CONTENT, Json(json!({}))).into_response()
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

fn read_channel_heartbeat(config: &HoneConfig, channel: &str) -> Option<ProcessHeartbeatSnapshot> {
    let path = runtime_heartbeat_path(&hone_core::runtime_heartbeat_dir(config), channel);
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
    state: &Arc<AppState>,
    id: &str,
    label: &str,
    enabled: bool,
    detail: &str,
    _config: &HoneConfig,
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
            processes: Vec::new(),
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
            processes: Vec::new(),
        };
    }

    let snapshots = collect_channel_heartbeats(state, id);
    if snapshots.is_empty() {
        return ChannelStatusInfo {
            id: id.to_string(),
            label: label.to_string(),
            enabled,
            running: false,
            status: "stopped".to_string(),
            pid: None,
            last_heartbeat_at: None,
            detail: "未检测到渠道心跳".to_string(),
            processes: Vec::new(),
        };
    }

    let processes = snapshots
        .iter()
        .map(snapshot_to_process_info)
        .collect::<Vec<_>>();
    let mut processes = processes;
    merge_os_processes(id, &mut processes);
    processes.sort_by_key(|process| process.pid);
    let running_processes = processes.iter().filter(|process| process.running).count();
    let stale_processes = processes.len().saturating_sub(running_processes);
    let latest = snapshots.iter().max_by_key(|snapshot| snapshot.updated_at);
    let primary_pid = latest.map(|snapshot| snapshot.pid);
    let latest_heartbeat_at = latest.map(|snapshot| snapshot.updated_at.to_rfc3339());

    let (running, status, detail) = if running_processes > 0 && stale_processes == 0 {
        let pids = running_pid_summary(&processes);
        (
            true,
            "running".to_string(),
            format!(
                "{detail}（{} 个进程在线，pids={}）",
                running_processes, pids
            ),
        )
    } else if running_processes > 0 {
        let pids = running_pid_summary(&processes);
        (
            false,
            "degraded".to_string(),
            format!(
                "{detail}（{} / {} 个进程在线，在线 pids={}）",
                running_processes,
                processes.len(),
                pids
            ),
        )
    } else {
        (false, "stopped".to_string(), stopped_detail(&processes))
    };

    ChannelStatusInfo {
        id: id.to_string(),
        label: label.to_string(),
        enabled,
        running,
        status,
        pid: primary_pid,
        last_heartbeat_at: latest_heartbeat_at,
        detail,
        processes,
    }
}

fn collect_channel_heartbeats(
    state: &Arc<AppState>,
    channel: &str,
) -> Vec<ProcessHeartbeatSnapshot> {
    let mut snapshots = BTreeMap::<u32, ProcessHeartbeatSnapshot>::new();

    for process in state.heartbeat_registry.channel_processes(channel) {
        snapshots.insert(
            process.pid,
            ProcessHeartbeatSnapshot {
                channel: process.channel,
                pid: process.pid,
                started_at: process.started_at,
                updated_at: process.updated_at,
            },
        );
    }

    if let Some(snapshot) = read_channel_heartbeat(&state.core.config, channel) {
        match snapshots.get(&snapshot.pid) {
            Some(existing) if existing.updated_at >= snapshot.updated_at => {}
            _ => {
                snapshots.insert(snapshot.pid, snapshot);
            }
        }
    }

    snapshots.into_values().collect()
}

fn snapshot_to_process_info(snapshot: &ProcessHeartbeatSnapshot) -> ChannelProcessInfo {
    ChannelProcessInfo {
        pid: snapshot.pid,
        running: heartbeat_is_fresh(snapshot),
        started_at: Some(snapshot.started_at.to_rfc3339()),
        last_heartbeat_at: Some(snapshot.updated_at.to_rfc3339()),
        managed_by_desktop: None,
        source: Some("heartbeat".to_string()),
    }
}

fn merge_os_processes(channel: &str, processes: &mut Vec<ChannelProcessInfo>) {
    let observed = scan_channel_processes(channel);
    for process in observed {
        if let Some(existing) = processes.iter_mut().find(|item| item.pid == process.pid) {
            existing.running = true;
            existing.source = match existing.source.as_deref() {
                Some("heartbeat") => Some("heartbeat+process_scan".to_string()),
                Some(current) => Some(current.to_string()),
                None => Some("process_scan".to_string()),
            };
        } else {
            processes.push(ChannelProcessInfo {
                pid: process.pid,
                running: true,
                started_at: None,
                last_heartbeat_at: None,
                managed_by_desktop: None,
                source: Some("process_scan".to_string()),
            });
        }
    }
}

fn running_pid_summary(processes: &[ChannelProcessInfo]) -> String {
    processes
        .iter()
        .filter(|process| process.running)
        .map(|process| process.pid.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn stopped_detail(processes: &[ChannelProcessInfo]) -> String {
    let stale = processes
        .iter()
        .map(|process| {
            let last_seen = process
                .last_heartbeat_at
                .as_deref()
                .unwrap_or("no-heartbeat");
            format!("{}@{}", process.pid, last_seen)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("心跳超时（{stale}）")
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
