use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::HoneConfig;

pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;
pub const HEARTBEAT_STALE_AFTER_SECS: u64 = 75;

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

    let task_path = path.clone();
    let task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

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
        }
    });

    Ok(ProcessHeartbeat { path, task })
}

fn write_snapshot(path: &Path, snapshot: &ProcessHeartbeatSnapshot) -> io::Result<()> {
    let content = serde_json::to_vec_pretty(snapshot).map_err(io::Error::other)?;
    fs::write(path, content)
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
