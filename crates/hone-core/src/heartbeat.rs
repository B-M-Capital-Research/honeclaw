use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::config::HoneConfig;

pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;
pub const HEARTBEAT_STALE_AFTER_SECS: u64 = 75;
const HEARTBEAT_ENDPOINT_PATH: &str = "/api/runtime/heartbeat";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHeartbeatSnapshot {
    pub channel: String,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ProcessHeartbeat {
    path: PathBuf,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for ProcessHeartbeat {
    fn drop(&mut self) {
        self.task.abort();
        let _ = fs::remove_file(&self.path);
    }
}

pub fn runtime_heartbeat_dir(config: &HoneConfig) -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("HONE_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir);
    }

    let sessions_dir = PathBuf::from(&config.storage.sessions_dir);
    let base_dir = sessions_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("./data"));
    base_dir.join("runtime")
}

pub fn runtime_heartbeat_path(runtime_dir: &Path, channel: &str) -> PathBuf {
    runtime_dir.join(format!("{channel}.heartbeat.json"))
}

pub fn read_process_heartbeat(path: &Path) -> io::Result<ProcessHeartbeatSnapshot> {
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(io::Error::other)
}

pub fn spawn_process_heartbeat(config: &HoneConfig, channel: &str) -> io::Result<ProcessHeartbeat> {
    let runtime_dir = runtime_heartbeat_dir(config);
    fs::create_dir_all(&runtime_dir)?;

    let path = runtime_heartbeat_path(&runtime_dir, channel);
    let channel = channel.to_string();
    let pid = std::process::id();
    let started_at = Utc::now();

    write_snapshot(
        &path,
        &ProcessHeartbeatSnapshot {
            channel: channel.clone(),
            pid,
            started_at,
            updated_at: started_at,
        },
    )?;

    let console_url = std::env::var("HONE_CONSOLE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty());
    let console_token = std::env::var("HONE_CONSOLE_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let task_path = path.clone();
    let task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let http_client = heartbeat_http_client(console_token.as_deref());

        loop {
            ticker.tick().await;
            let now = Utc::now();
            let snapshot = ProcessHeartbeatSnapshot {
                channel: channel.clone(),
                pid,
                started_at,
                updated_at: now,
            };
            if let Err(err) = write_snapshot(&task_path, &snapshot) {
                tracing::warn!(
                    channel = %channel,
                    pid,
                    path = %task_path.display(),
                    "failed to write heartbeat: {err}"
                );
            }
            if let (Some(base_url), Some(client)) = (console_url.as_deref(), http_client.as_ref()) {
                if let Err(err) = post_remote_heartbeat(client, base_url, &snapshot).await {
                    tracing::warn!(
                        channel = %channel,
                        pid,
                        url = %base_url,
                        "failed to post heartbeat: {err}"
                    );
                }
            }
        }
    });

    Ok(ProcessHeartbeat { path, task })
}

fn write_snapshot(path: &Path, snapshot: &ProcessHeartbeatSnapshot) -> io::Result<()> {
    let content = serde_json::to_vec_pretty(snapshot).map_err(io::Error::other)?;
    fs::write(path, content)
}

fn heartbeat_http_client(bearer_token: Option<&str>) -> Option<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Some(token) = bearer_token {
        let value = HeaderValue::from_str(&format!("Bearer {token}")).ok()?;
        headers.insert(AUTHORIZATION, value);
    }

    reqwest::Client::builder()
        // Heartbeat posts target the local desktop backend and should not be routed
        // through system HTTP/SOCKS proxies such as Clash/Surge.
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .default_headers(headers)
        .build()
        .ok()
}

async fn post_remote_heartbeat(
    client: &reqwest::Client,
    base_url: &str,
    snapshot: &ProcessHeartbeatSnapshot,
) -> Result<(), reqwest::Error> {
    client
        .post(format!("{base_url}{HEARTBEAT_ENDPOINT_PATH}"))
        .json(snapshot)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_path_uses_json_suffix() {
        let path = runtime_heartbeat_path(Path::new("/tmp/runtime"), "discord");
        assert_eq!(path, PathBuf::from("/tmp/runtime/discord.heartbeat.json"));
    }
}
