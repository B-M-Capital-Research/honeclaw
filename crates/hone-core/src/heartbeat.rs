use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
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

/// Persistent error sidecar written when `write_snapshot` fails repeatedly.
/// Read by web-api's status routes so a backend can flag a channel as
/// degraded even when the channel's own heartbeat file is frozen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatErrorRecord {
    pub channel: String,
    pub pid: u32,
    pub consecutive_failures: u32,
    pub last_error: String,
    pub last_failure_at: DateTime<Utc>,
}

/// In-process counters for the heartbeat task. The owning process can read
/// these directly via `ProcessHeartbeat::metrics()`. Cross-process consumers
/// must read the on-disk `.heartbeat.error` sidecar instead.
#[derive(Debug, Default)]
pub struct HeartbeatMetrics {
    consecutive_failures: AtomicU32,
    last_error: Mutex<Option<String>>,
    last_failure_at: Mutex<Option<DateTime<Utc>>>,
}

impl HeartbeatMetrics {
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|guard| guard.clone())
    }

    pub fn last_failure_at(&self) -> Option<DateTime<Utc>> {
        self.last_failure_at.lock().ok().and_then(|guard| *guard)
    }

    pub fn is_degraded(&self) -> bool {
        self.consecutive_failures() > 0
    }

    fn record_failure(&self, err: &str, when: DateTime<Utc>) -> u32 {
        let next = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = Some(err.to_string());
        }
        if let Ok(mut guard) = self.last_failure_at.lock() {
            *guard = Some(when);
        }
        next
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.last_failure_at.lock() {
            *guard = None;
        }
    }
}

pub struct ProcessHeartbeat {
    path: PathBuf,
    error_path: PathBuf,
    task: tokio::task::JoinHandle<()>,
    metrics: Arc<HeartbeatMetrics>,
}

impl ProcessHeartbeat {
    pub fn metrics(&self) -> Arc<HeartbeatMetrics> {
        self.metrics.clone()
    }
}

impl Drop for ProcessHeartbeat {
    fn drop(&mut self) {
        self.task.abort();
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_file(&self.error_path);
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

/// Sidecar written when the main heartbeat write fails. Web-api consumers
/// read this to surface a degraded state even when the heartbeat file is
/// frozen at a still-fresh timestamp.
pub fn runtime_heartbeat_error_path(runtime_dir: &Path, channel: &str) -> PathBuf {
    runtime_dir.join(format!("{channel}.heartbeat.error"))
}

pub fn read_process_heartbeat(path: &Path) -> io::Result<ProcessHeartbeatSnapshot> {
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(io::Error::other)
}

pub fn read_heartbeat_error(path: &Path) -> Option<HeartbeatErrorRecord> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn spawn_process_heartbeat(config: &HoneConfig, channel: &str) -> io::Result<ProcessHeartbeat> {
    let runtime_dir = runtime_heartbeat_dir(config);
    fs::create_dir_all(&runtime_dir)?;

    let path = runtime_heartbeat_path(&runtime_dir, channel);
    let error_path = runtime_heartbeat_error_path(&runtime_dir, channel);
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
    let _ = fs::remove_file(&error_path);

    let console_url = std::env::var("HONE_CONSOLE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty());
    let console_token = std::env::var("HONE_CONSOLE_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let metrics = Arc::new(HeartbeatMetrics::default());
    let task_metrics = metrics.clone();
    let task_path = path.clone();
    let task_error_path = error_path.clone();
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
            match write_snapshot(&task_path, &snapshot) {
                Ok(()) => {
                    if task_metrics.is_degraded() {
                        tracing::info!(
                            channel = %channel,
                            pid,
                            "heartbeat write recovered after failures"
                        );
                    }
                    task_metrics.record_success();
                    let _ = fs::remove_file(&task_error_path);
                }
                Err(err) => {
                    let err_msg = format!("{err}");
                    let count = task_metrics.record_failure(&err_msg, now);
                    tracing::warn!(
                        channel = %channel,
                        pid,
                        path = %task_path.display(),
                        consecutive_failures = count,
                        "failed to write heartbeat: {err_msg}"
                    );
                    let record = HeartbeatErrorRecord {
                        channel: channel.clone(),
                        pid,
                        consecutive_failures: count,
                        last_error: err_msg,
                        last_failure_at: now,
                    };
                    if let Err(side_err) = write_error_sidecar(&task_error_path, &record) {
                        tracing::warn!(
                            channel = %channel,
                            pid,
                            path = %task_error_path.display(),
                            "failed to write heartbeat error sidecar: {side_err}"
                        );
                    }
                }
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

    Ok(ProcessHeartbeat {
        path,
        error_path,
        task,
        metrics,
    })
}

fn write_snapshot(path: &Path, snapshot: &ProcessHeartbeatSnapshot) -> io::Result<()> {
    let content = serde_json::to_vec_pretty(snapshot).map_err(io::Error::other)?;
    fs::write(path, content)
}

fn write_error_sidecar(path: &Path, record: &HeartbeatErrorRecord) -> io::Result<()> {
    let content = serde_json::to_vec(record).map_err(io::Error::other)?;
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

    #[test]
    fn heartbeat_error_path_uses_error_suffix() {
        let path = runtime_heartbeat_error_path(Path::new("/tmp/runtime"), "discord");
        assert_eq!(path, PathBuf::from("/tmp/runtime/discord.heartbeat.error"));
    }

    #[test]
    fn metrics_record_and_recover() {
        let m = HeartbeatMetrics::default();
        assert!(!m.is_degraded());
        let when = Utc::now();
        assert_eq!(m.record_failure("disk full", when), 1);
        assert_eq!(m.record_failure("disk full", when), 2);
        assert!(m.is_degraded());
        assert_eq!(m.consecutive_failures(), 2);
        assert_eq!(m.last_error().as_deref(), Some("disk full"));
        m.record_success();
        assert!(!m.is_degraded());
        assert_eq!(m.consecutive_failures(), 0);
        assert!(m.last_error().is_none());
    }

    #[test]
    fn read_heartbeat_error_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = runtime_heartbeat_error_path(dir.path(), "telegram");
        let record = HeartbeatErrorRecord {
            channel: "telegram".into(),
            pid: 4242,
            consecutive_failures: 3,
            last_error: "No space left on device".into(),
            last_failure_at: Utc::now(),
        };
        write_error_sidecar(&path, &record).unwrap();
        let loaded = read_heartbeat_error(&path).expect("error file should round-trip");
        assert_eq!(loaded.channel, "telegram");
        assert_eq!(loaded.pid, 4242);
        assert_eq!(loaded.consecutive_failures, 3);
        assert_eq!(loaded.last_error, "No space left on device");
    }

    #[test]
    fn read_heartbeat_error_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = runtime_heartbeat_error_path(dir.path(), "telegram");
        assert!(read_heartbeat_error(&path).is_none());
    }
}
