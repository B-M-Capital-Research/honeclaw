use super::*;

fn bundled_lock_failure_message(error: &hone_core::ProcessLockError) -> String {
    format!(
        "检测到旧的 Hone bundled runtime 进程仍占用启动锁，桌面主进程不会继续启动相关 backend/listener。请先清理之前的进程后再重试。\n冲突组件: {}\n锁文件: {}",
        error.process,
        error.path.display()
    )
}

pub(super) fn preflight_bundled_runtime_locks(app: &AppHandle) -> Result<(), String> {
    let runtime = ensure_runtime_paths(app)?;
    let runtime_config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    let lock_names = hone_core::enabled_process_lock_names(&runtime_config);

    let mut on_warn = |message: &str| log_desktop(app, "WARN", message.to_string());
    hone_core::preflight_process_locks_with_cleanup(
        &runtime.runtime_dir,
        &lock_names,
        &mut on_warn,
    )
    .map_err(|error| bundled_lock_failure_message(&error))
}

fn ensure_desktop_process_lock(app: &AppHandle) -> Result<(), String> {
    let runtime = ensure_runtime_paths(app)?;
    let desktop_lock_path =
        hone_core::process_lock_path(&runtime.runtime_dir, hone_core::PROCESS_LOCK_DESKTOP);
    let state = app.state::<DesktopState>();
    let mut guard = state.desktop_lock.lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }

    let lock =
        hone_core::acquire_process_lock(&runtime.runtime_dir, hone_core::PROCESS_LOCK_DESKTOP)
            .map_err(|error| {
                hone_core::format_lock_failure_message(
                    hone_core::PROCESS_LOCK_DESKTOP,
                    &desktop_lock_path,
                    &error,
                    "Hone Desktop",
                )
            })?;
    *guard = Some(lock);
    Ok(())
}

pub(super) fn preflight_startup_locks(app: &AppHandle) -> Result<(), String> {
    ensure_desktop_process_lock(app)?;
    let config = load_persisted_config(app).unwrap_or_default();
    if config.mode != "bundled" {
        return Ok(());
    }
    preflight_bundled_runtime_locks(app)
}

fn stop_web_server(manager: &mut DesktopBackendManager) {
    for handle in std::mem::take(&mut manager.web_server_tasks) {
        handle.abort();
    }
}

pub(super) fn stop_managed_children(manager: &mut DesktopBackendManager) {
    stop_web_server(manager);
    manager.bundled_web_lock = None;
    for (_, child) in std::mem::take(&mut manager.channel_children) {
        let _ = child.kill();
    }
    if let Some(runtime_dir) = manager
        .diagnostics
        .as_ref()
        .map(|paths| PathBuf::from(&paths.data_dir).join("runtime"))
    {
        clear_runtime_heartbeats(&runtime_dir);
    }
}

fn runtime_heartbeat_path(runtime_dir: &std::path::Path, channel: &str) -> PathBuf {
    hone_core::runtime_heartbeat_path(runtime_dir, channel)
}

fn remove_runtime_heartbeat(runtime_dir: &std::path::Path, channel: &str) {
    let _ = fs::remove_file(runtime_heartbeat_path(runtime_dir, channel));
}

fn clear_runtime_heartbeats(runtime_dir: &std::path::Path) {
    for channel in ["imessage", "discord", "feishu", "telegram"] {
        remove_runtime_heartbeat(runtime_dir, channel);
    }
}

fn start_logged_sidecar(
    app: &AppHandle,
    binary: &str,
    log_label: &str,
    envs: Vec<(&str, String)>,
    log_path: PathBuf,
) -> Result<CommandChild, String> {
    let command = app.shell().sidecar(binary).map_err(|e| e.to_string())?;
    let command = envs
        .into_iter()
        .fold(command, |command, (key, value)| command.env(key, value));

    let (mut rx, child) = command.spawn().map_err(|e| e.to_string())?;
    let log_label = log_label.to_string();

    append_log(
        &log_path,
        "INFO",
        &format!("spawned {binary} pid={}", child.pid()),
    );

    async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => {
                    let line = String::from_utf8_lossy(&bytes);
                    append_log(
                        &log_path,
                        "INFO",
                        &format!("[{log_label}] {}", line.trim_end()),
                    );
                }
                CommandEvent::Stderr(bytes) => {
                    let line = String::from_utf8_lossy(&bytes);
                    append_log(
                        &log_path,
                        "ERROR",
                        &format!("[{log_label}] {}", line.trim_end()),
                    );
                }
                CommandEvent::Error(error) => {
                    append_log(
                        &log_path,
                        "ERROR",
                        &format!("[{log_label}] shell event error: {error}"),
                    );
                }
                CommandEvent::Terminated(payload) => {
                    append_log(
                        &log_path,
                        "INFO",
                        &format!(
                            "[{log_label}] sidecar terminated code={:?} signal={:?}",
                            payload.code, payload.signal
                        ),
                    );
                }
                _ => {}
            }
        }
    });

    Ok(child)
}

fn common_runtime_envs(runtime: &RuntimePaths) -> Vec<(&'static str, String)> {
    let mut envs = vec![
        (
            "HONE_CONFIG_PATH",
            runtime.effective_config_path.to_string_lossy().to_string(),
        ),
        (
            "HONE_USER_CONFIG_PATH",
            runtime.config_path.to_string_lossy().to_string(),
        ),
        (
            "HONE_DATA_DIR",
            runtime.data_dir.to_string_lossy().to_string(),
        ),
        (
            "HONE_SKILLS_DIR",
            runtime.skills_dir.to_string_lossy().to_string(),
        ),
        (
            "HONE_RUNTIME_DIR",
            runtime.runtime_dir.to_string_lossy().to_string(),
        ),
        (
            "HONE_AGENT_SANDBOX_DIR",
            runtime
                .data_dir
                .join("agent-sandboxes")
                .to_string_lossy()
                .to_string(),
        ),
    ];

    for key in ["HONE_MCP_BIN", "HONE_BUNDLED_OPENCODE_BIN"] {
        if let Ok(value) = env::var(key) {
            if !value.trim().is_empty() {
                envs.push((key, value));
            }
        }
    }

    envs
}

pub(super) fn start_enabled_channels(
    app: &AppHandle,
    manager: &mut DesktopBackendManager,
    runtime: &RuntimePaths,
    diagnostics: &DiagnosticPaths,
    base_url: &str,
) -> Result<(), String> {
    clear_runtime_heartbeats(&runtime.runtime_dir);

    let config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    let sidecar_log = PathBuf::from(&diagnostics.sidecar_log);

    let mut spawn_channel = |channel: &str,
                             binary: &str,
                             enabled: bool,
                             supported: bool,
                             extra_envs: Vec<(&'static str, String)>|
     -> Result<(), String> {
        if !enabled || !supported {
            remove_runtime_heartbeat(&runtime.runtime_dir, channel);
            return Ok(());
        }

        let mut envs = common_runtime_envs(runtime);
        envs.push(("HONE_CONSOLE_URL", base_url.to_string()));
        if !config.web.auth_token.trim().is_empty() {
            envs.push((
                "HONE_CONSOLE_TOKEN",
                config.web.auth_token.trim().to_string(),
            ));
        }
        envs.extend(extra_envs);

        let child = match start_logged_sidecar(app, binary, channel, envs, sidecar_log.clone()) {
            Ok(child) => child,
            Err(error) => {
                remove_runtime_heartbeat(&runtime.runtime_dir, channel);
                log_desktop(
                    app,
                    "WARN",
                    format!("managed channel {channel} skipped because spawn failed: {error}"),
                );
                return Ok(());
            }
        };
        std::thread::sleep(Duration::from_millis(400));
        let still_running = hone_core::scan_channel_processes(channel)
            .into_iter()
            .any(|process| process.pid == child.pid());
        if !still_running {
            remove_runtime_heartbeat(&runtime.runtime_dir, channel);
            log_desktop(
                app,
                "WARN",
                format!(
                    "managed channel {channel} skipped because it exited during startup; an older process may still exist or the sidecar failed before acquiring its runtime lock"
                ),
            );
            return Ok(());
        }
        manager.channel_children.insert(channel.to_string(), child);
        log_desktop(app, "INFO", format!("started managed channel {channel}"));
        Ok(())
    };

    spawn_channel(
        "imessage",
        "hone-imessage",
        config.imessage.enabled,
        cfg!(target_os = "macos"),
        Vec::new(),
    )?;
    spawn_channel(
        "discord",
        "hone-discord",
        config.discord.enabled,
        true,
        Vec::new(),
    )?;
    spawn_channel(
        "feishu",
        "hone-feishu",
        config.feishu.enabled,
        true,
        Vec::new(),
    )?;
    spawn_channel(
        "telegram",
        "hone-telegram",
        config.telegram.enabled,
        true,
        Vec::new(),
    )?;

    Ok(())
}

fn kill_pid(pid: u32) {
    let _ = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status();
}

pub(super) fn cleanup_duplicate_channel_processes_inner(
    manager: &mut DesktopBackendManager,
) -> ChannelProcessCleanupResult {
    let mut entries = Vec::new();

    for channel in ["imessage", "discord", "feishu", "telegram"] {
        let mut observed = hone_core::scan_channel_processes(channel)
            .into_iter()
            .map(|process| process.pid)
            .collect::<Vec<_>>();
        observed.sort_unstable();
        observed.dedup();

        if observed.is_empty() {
            entries.push(ChannelProcessCleanupEntry {
                channel: channel.to_string(),
                kept_pid: None,
                removed_pids: Vec::new(),
            });
            continue;
        }

        let managed_pid = manager
            .channel_children
            .get(channel)
            .map(|child| child.pid());
        let keep_pid = managed_pid
            .filter(|pid| observed.contains(pid))
            .or_else(|| observed.iter().copied().max());

        let removed_pids = observed
            .into_iter()
            .filter(|pid| Some(*pid) != keep_pid)
            .collect::<Vec<_>>();

        for pid in &removed_pids {
            kill_pid(*pid);
        }

        entries.push(ChannelProcessCleanupEntry {
            channel: channel.to_string(),
            kept_pid: keep_pid,
            removed_pids,
        });
    }

    let removed_total = entries
        .iter()
        .map(|entry| entry.removed_pids.len())
        .sum::<usize>();
    let message = if removed_total == 0 {
        "未发现需要清理的多余渠道进程".to_string()
    } else {
        format!("已清理 {removed_total} 个多余渠道进程，并为每个渠道保留 1 个实例")
    };

    ChannelProcessCleanupResult { entries, message }
}
