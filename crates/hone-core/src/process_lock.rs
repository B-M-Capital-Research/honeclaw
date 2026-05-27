use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::thread;
use std::time::Duration;

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
    fs::create_dir_all(&lock_dir)
        .map_err(|err| process_lock_io_error("create process lock directory", &lock_dir, err))?;

    let path = process_lock_path(runtime_dir, name);
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|err| process_lock_io_error("open process lock", &path, err))?;
    file.try_lock_exclusive()
        .map_err(|err| process_lock_io_error("acquire process lock", &path, err))?;

    file.set_len(0)
        .map_err(|err| process_lock_io_error("truncate process lock", &path, err))?;
    file.write_all(
        format!(
            "pid={}\nprocess={}\nlocked_at={}\n",
            std::process::id(),
            name,
            Utc::now().to_rfc3339()
        )
        .as_bytes(),
    )
    .map_err(|err| process_lock_io_error("write process lock metadata", &path, err))?;
    file.sync_data()
        .map_err(|err| process_lock_io_error("sync process lock", &path, err))?;

    Ok(ProcessLockGuard {
        name: name.to_string(),
        path,
        file,
    })
}

fn process_lock_io_error(action: &str, path: &Path, err: io::Error) -> io::Error {
    io::Error::new(err.kind(), format!("{action} ({}): {err}", path.display()))
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

fn lock_pid_mismatch_warning(process: &str, pid: u32) -> String {
    format!(
        "{process} 启动锁记录的 pid={pid} 仍存在，但命令行已不匹配；为避免误杀进程，已跳过自动清理。"
    )
}

fn lock_cleanup_attempt_warning(process: &str, pid: u32, path: &Path) -> String {
    format!(
        "检测到 {process} 启动锁冲突，正在尝试自动结束旧进程 pid={pid} 并清理锁文件: {}",
        path.display()
    )
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    let content = fs::read_to_string(path).ok()?;
    content.lines().find_map(|line| {
        line.strip_prefix("pid=")
            .and_then(|raw| raw.trim().parse::<u32>().ok())
    })
}

fn pid_command_line(pid: u32) -> Option<String> {
    let output = StdCommand::new("ps")
        .args(["-p", &pid.to_string(), "-o", "args="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let rendered = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if rendered.is_empty() {
        None
    } else {
        Some(rendered)
    }
}

fn pid_matches_expected_process(pid: u32, process: &str) -> bool {
    let Some(command) = pid_command_line(pid) else {
        return false;
    };
    let executable = command.split_whitespace().next().unwrap_or_default();
    Path::new(executable)
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value == process)
        .unwrap_or(false)
        || command.contains(process)
}

fn pid_is_alive(pid: u32) -> bool {
    StdCommand::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pid="])
        .output()
        .map(|output| {
            output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
        })
        .unwrap_or(false)
}

fn wait_for_pid_exit(pid: u32, timeout_ms: u64) -> bool {
    let polls = timeout_ms / 100;
    for _ in 0..polls.max(1) {
        if !pid_is_alive(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    !pid_is_alive(pid)
}

fn terminate_pid_with_retry(pid: u32) -> bool {
    let _ = StdCommand::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status();
    if wait_for_pid_exit(pid, 2_000) {
        return true;
    }
    let _ = StdCommand::new("kill")
        .arg("-KILL")
        .arg(pid.to_string())
        .status();
    wait_for_pid_exit(pid, 1_500)
}

/// Try to free a stuck startup lock by terminating the PID recorded in the
/// lock file, when that PID still matches the expected process name.
///
/// `on_warn` receives one-line, human-readable progress notes so callers can
/// route them into whatever logger they use (println, tauri event log, etc.).
/// Returns `true` if the conflicting process was terminated and the stale
/// lock file was removed; `false` if the conflict could not be resolved.
pub fn try_cleanup_conflicting_process(
    error: &ProcessLockError,
    on_warn: &mut dyn FnMut(&str),
) -> bool {
    let Some(pid) = read_lock_pid(&error.path) else {
        return false;
    };
    if !pid_matches_expected_process(pid, &error.process) {
        on_warn(&lock_pid_mismatch_warning(&error.process, pid));
        return false;
    }

    on_warn(&lock_cleanup_attempt_warning(
        &error.process,
        pid,
        &error.path,
    ));

    let stopped = terminate_pid_with_retry(pid);
    if stopped {
        let _ = fs::remove_file(&error.path);
    }
    stopped
}

/// Like [`preflight_process_locks`], but on conflict tries to clean up the
/// stale owner once per lock name before giving up.
pub fn preflight_process_locks_with_cleanup(
    runtime_dir: &Path,
    names: &[&str],
    on_warn: &mut dyn FnMut(&str),
) -> Result<(), ProcessLockError> {
    let mut cleanup_attempts = 0usize;
    loop {
        match preflight_process_locks(runtime_dir, names) {
            Ok(()) => return Ok(()),
            Err(error) => {
                if cleanup_attempts < names.len()
                    && try_cleanup_conflicting_process(&error, on_warn)
                {
                    cleanup_attempts += 1;
                    continue;
                }
                return Err(error);
            }
        }
    }
}

/// Returns the list of bundled-runtime process lock names that should be held
/// for a given config (console-page is always required; channel listeners are
/// included only when enabled and supported on the current OS).
pub fn enabled_process_lock_names(config: &HoneConfig) -> Vec<&'static str> {
    let mut names = vec![PROCESS_LOCK_CONSOLE_PAGE];
    if cfg!(target_os = "macos") && config.imessage.enabled {
        names.push(PROCESS_LOCK_IMESSAGE);
    }
    if config.discord.enabled {
        names.push(PROCESS_LOCK_DISCORD);
    }
    if config.feishu.enabled {
        names.push(PROCESS_LOCK_FEISHU);
    }
    if config.telegram.enabled {
        names.push(PROCESS_LOCK_TELEGRAM);
    }
    names
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
            assert!(err.to_string().contains("acquire process lock"));
            assert!(
                err.to_string().contains(
                    &process_lock_path(&runtime_dir, PROCESS_LOCK_TELEGRAM)
                        .display()
                        .to_string()
                )
            );
        }

        let _ = fs::remove_dir_all(&runtime_dir);
    }

    #[test]
    fn acquire_process_lock_reports_lock_directory_path() {
        let runtime_dir = std::env::temp_dir().join(format!(
            "hone-process-lock-dir-error-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        fs::write(&runtime_dir, "plain file").expect("file as runtime dir");

        let err = acquire_process_lock(&runtime_dir, PROCESS_LOCK_TELEGRAM)
            .expect_err("file runtime dir should fail");

        assert!(err.to_string().contains("create process lock directory"));
        assert!(
            err.to_string()
                .contains(&runtime_lock_dir(&runtime_dir).display().to_string())
        );
        let _ = fs::remove_file(&runtime_dir);
    }

    #[test]
    fn process_lock_cleanup_warnings_are_actionable() {
        let mismatch = lock_pid_mismatch_warning(PROCESS_LOCK_DISCORD, 123);
        assert!(mismatch.contains(PROCESS_LOCK_DISCORD));
        assert!(mismatch.contains("pid=123"));
        assert!(mismatch.contains("跳过自动清理"));

        let cleanup =
            lock_cleanup_attempt_warning(PROCESS_LOCK_DISCORD, 123, Path::new("/tmp/hone.lock"));
        assert!(cleanup.contains("启动锁冲突"));
        assert!(cleanup.contains("pid=123"));
        assert!(cleanup.contains("/tmp/hone.lock"));
    }
}
