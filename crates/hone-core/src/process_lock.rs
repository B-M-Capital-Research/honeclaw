use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use fs2::FileExt;

use crate::config::HoneConfig;

pub const PROCESS_LOCK_DESKTOP: &str = "hone-desktop";
pub const PROCESS_LOCK_CONSOLE_PAGE: &str = "hone-console-page";
pub const PROCESS_LOCK_IMESSAGE: &str = "hone-imessage";
pub const PROCESS_LOCK_DISCORD: &str = "hone-discord";
pub const PROCESS_LOCK_FEISHU: &str = "hone-feishu";
pub const PROCESS_LOCK_TELEGRAM: &str = "hone-telegram";

#[derive(Debug)]
pub struct ProcessLockGuard {
    pub name: String,
    pub path: PathBuf,
    file: File,
}

impl Drop for ProcessLockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub fn runtime_lock_dir(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("locks")
}

pub fn process_lock_path(runtime_dir: &Path, name: &str) -> PathBuf {
    runtime_lock_dir(runtime_dir).join(format!("{name}.lock"))
}

pub fn acquire_process_lock(runtime_dir: &Path, name: &str) -> io::Result<ProcessLockGuard> {
    let lock_dir = runtime_lock_dir(runtime_dir);
    fs::create_dir_all(&lock_dir)?;

    let path = process_lock_path(runtime_dir, name);
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)?;
    file.try_lock_exclusive()?;

    file.set_len(0)?;
    file.write_all(
        format!(
            "pid={}\nprocess={}\nlocked_at={}\n",
            std::process::id(),
            name,
            Utc::now().to_rfc3339()
        )
        .as_bytes(),
    )?;
    file.sync_data()?;

    Ok(ProcessLockGuard {
        name: name.to_string(),
        path,
        file,
    })
}

pub fn acquire_runtime_process_lock(
    config: &HoneConfig,
    name: &str,
) -> io::Result<ProcessLockGuard> {
    acquire_process_lock(&crate::heartbeat::runtime_heartbeat_dir(config), name)
}

pub fn preflight_process_locks(runtime_dir: &Path, names: &[&str]) -> Result<(), ProcessLockError> {
    let mut guards = Vec::with_capacity(names.len());
    for name in names {
        match acquire_process_lock(runtime_dir, name) {
            Ok(guard) => guards.push(guard),
            Err(source) => {
                return Err(ProcessLockError {
                    process: (*name).to_string(),
                    path: process_lock_path(runtime_dir, name),
                    source,
                });
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct ProcessLockError {
    pub process: String,
    pub path: PathBuf,
    pub source: io::Error,
}

impl ProcessLockError {
    pub fn is_conflict(&self) -> bool {
        self.source.kind() == io::ErrorKind::WouldBlock
    }
}

impl std::fmt::Display for ProcessLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} startup lock unavailable at {}: {}",
            self.process,
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for ProcessLockError {}

pub fn format_lock_failure_message(
    process: &str,
    path: &Path,
    error: &io::Error,
    scope: &str,
) -> String {
    if error.kind() == io::ErrorKind::WouldBlock {
        format!(
            "检测到旧的 {scope} 进程仍占用启动锁，{process} 不会启动。请先清理之前的进程后再重试。\n锁文件: {}",
            path.display()
        )
    } else {
        format!(
            "无法为 {process} 创建启动锁，当前不会继续启动。请先清理之前的进程后再重试。\n锁文件: {}\n错误: {error}",
            path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_lock_path_lives_under_runtime_locks_dir() {
        let path = process_lock_path(Path::new("/tmp/runtime"), PROCESS_LOCK_DISCORD);
        assert_eq!(path, PathBuf::from("/tmp/runtime/locks/hone-discord.lock"));
    }

    #[test]
    fn same_lock_cannot_be_acquired_twice() {
        let runtime_dir = std::env::temp_dir().join(format!(
            "hone-process-lock-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::create_dir_all(&runtime_dir).expect("runtime dir");

        {
            let _guard =
                acquire_process_lock(&runtime_dir, PROCESS_LOCK_TELEGRAM).expect("first lock");
            let err = acquire_process_lock(&runtime_dir, PROCESS_LOCK_TELEGRAM)
                .expect_err("second lock should fail");
            assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
        }

        let _ = fs::remove_dir_all(&runtime_dir);
    }
}
