//! ACP child-process lifecycle helpers.
//!
//! ACP runners hand `hone-mcp` to the upstream CLI as a stdio MCP server. That
//! makes the MCP server a grandchild of Hone, so cleanup must target the whole
//! ACP process group instead of only the direct child.

use std::time::Duration;

use tokio::process::{Child, Command};

pub(crate) fn configure_acp_command_process_group(command: &mut Command) {
    command.kill_on_drop(true);
    #[cfg(unix)]
    {
        command.process_group(0);
    }
}

pub(crate) struct AcpChildGuard {
    runner_label: &'static str,
    child: Option<Child>,
    process_group_id: Option<u32>,
    stderr_task: Option<tokio::task::JoinHandle<()>>,
}

impl AcpChildGuard {
    pub(crate) fn new(
        runner_label: &'static str,
        child: Child,
        stderr_task: Option<tokio::task::JoinHandle<()>>,
    ) -> Self {
        Self {
            runner_label,
            process_group_id: child.id(),
            child: Some(child),
            stderr_task,
        }
    }

    pub(crate) fn child_mut(&mut self) -> Option<&mut Child> {
        self.child.as_mut()
    }

    pub(crate) fn set_stderr_task(&mut self, task: Option<tokio::task::JoinHandle<()>>) {
        self.stderr_task = task;
    }

    pub(crate) async fn terminate(&mut self) {
        self.abort_stderr_task();

        let Some(child) = self.child.as_mut() else {
            return;
        };

        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(err) => {
                tracing::warn!(
                    "[AgentRunner/{}] failed to inspect ACP child before cleanup: {}",
                    self.runner_label,
                    err
                );
            }
        }

        terminate_process_tree(self.process_group_id, child).await;
        let _ = self.child.take();
    }

    fn abort_stderr_task(&mut self) {
        if let Some(task) = self.stderr_task.take() {
            task.abort();
        }
    }
}

impl Drop for AcpChildGuard {
    fn drop(&mut self) {
        self.abort_stderr_task();
        if let Some(child) = self.child.as_mut() {
            let _ = child.start_kill();
        }
        #[cfg(unix)]
        if let Some(process_group_id) = self.process_group_id {
            send_process_group_signal(process_group_id, "KILL");
        }
    }
}

async fn terminate_process_tree(process_group_id: Option<u32>, child: &mut Child) {
    #[cfg(unix)]
    if let Some(process_group_id) = process_group_id {
        send_process_group_signal(process_group_id, "TERM");
    }

    #[cfg(not(unix))]
    {
        let _ = child.start_kill();
    }

    if tokio::time::timeout(Duration::from_secs(2), child.wait())
        .await
        .is_ok()
    {
        return;
    }

    #[cfg(unix)]
    if let Some(process_group_id) = process_group_id {
        send_process_group_signal(process_group_id, "KILL");
    }
    let _ = child.start_kill();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

#[cfg(unix)]
fn send_process_group_signal(process_group_id: u32, signal: &str) {
    let target = format!("-{process_group_id}");
    let _ = std::process::Command::new("kill")
        .arg(format!("-{signal}"))
        // Linux's external `kill` may otherwise parse a negative PGID as
        // another option instead of the process-group target.
        .arg("--")
        .arg(target)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;

    #[cfg(unix)]
    #[tokio::test]
    async fn acp_child_guard_terminates_grandchild_process_group() {
        let temp = tempfile::tempdir().expect("tempdir");
        let pid_path = temp.path().join("grandchild.pid");
        let script = format!(
            "sleep 30 & echo $! > {}; wait",
            shell_quote(&pid_path.to_string_lossy())
        );

        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(script)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        configure_acp_command_process_group(&mut command);
        let child = command.spawn().expect("spawn shell");
        let mut guard = AcpChildGuard::new("test", child, None);

        let grandchild_pid = wait_for_pid_file(&pid_path).await;
        assert!(
            process_is_alive(grandchild_pid),
            "grandchild should be alive before cleanup"
        );

        guard.terminate().await;
        wait_until_process_exits(grandchild_pid).await;
        assert!(
            !process_is_alive(grandchild_pid),
            "grandchild should be killed with the process group"
        );
    }

    #[cfg(unix)]
    async fn wait_for_pid_file(path: &std::path::Path) -> u32 {
        for _ in 0..50 {
            if let Ok(raw) = std::fs::read_to_string(path)
                && let Ok(pid) = raw.trim().parse::<u32>()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("pid file was not written: {}", path.display());
    }

    #[cfg(unix)]
    async fn wait_until_process_exits(pid: u32) {
        for _ in 0..50 {
            if !process_is_alive(pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[cfg(unix)]
    fn process_is_alive(pid: u32) -> bool {
        #[cfg(target_os = "linux")]
        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat"))
            && stat
                .rsplit_once(") ")
                .and_then(|(_, fields)| fields.chars().next())
                == Some('Z')
        {
            // `kill -0` reports a zombie as present even though it cannot run.
            // Minimal CI containers may leave orphaned grandchildren in this
            // state until PID 1 eventually reaps them.
            return false;
        }
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    fn shell_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
