use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use clap::Args;
use reqwest::StatusCode;
use tokio::process::{Child, Command};

use crate::common::{ResolvedRuntimePaths, load_cli_config, resolve_runtime_paths};

const SOURCE_RUNTIME_PACKAGES: &[&str] = &[
    "hone-cli",
    "hone-mcp",
    "hone-console-page",
    "hone-imessage",
    "hone-discord",
    "hone-feishu",
    "hone-telegram",
];
const DEFAULT_PUBLIC_WEB_PORT: u16 = 8088;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct StartArgs {
    /// 在源码 checkout 中先构建 hone-cli 和运行时二进制，再从本地 target 启动。
    #[arg(long)]
    pub(crate) build: bool,
    /// 源码 checkout 根目录；默认从当前目录向上查找，或读取 HONE_SOURCE_ROOT。
    #[arg(long, value_name = "DIR")]
    pub(crate) source_root: Option<PathBuf>,
}

fn executable_name(binary: &str) -> String {
    if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    }
}

pub(crate) fn source_root_from_env_or_cwd(explicit: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return Some(path.to_path_buf());
    }
    if let Some(path) = env::var_os("HONE_SOURCE_ROOT").map(PathBuf::from) {
        return Some(path);
    }
    let mut current = env::current_dir().ok()?;
    loop {
        if current.join("Cargo.toml").is_file() && current.join("bins/hone-cli").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn source_target_debug_dir(source_root: &Path) -> PathBuf {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| source_root.join("target"));
    let target_dir = if target_dir.is_absolute() {
        target_dir
    } else {
        source_root.join(target_dir)
    };
    target_dir.join("debug")
}

fn public_web_port_from_env() -> u16 {
    env::var("HONE_PUBLIC_WEB_PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PUBLIC_WEB_PORT)
}

fn binary_search_dirs(source_root: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(root) = source_root {
        dirs.push(source_target_debug_dir(root));
    } else if let Some(root) = env::var_os("HONE_SOURCE_ROOT").map(PathBuf::from) {
        dirs.push(source_target_debug_dir(&root));
    }
    dirs.extend(current_exe_dirs());
    dirs
}

fn current_exe_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        dirs.push(parent.to_path_buf());
        if parent.file_name().and_then(OsStr::to_str) == Some("deps")
            && let Some(grandparent) = parent.parent()
        {
            dirs.push(grandparent.to_path_buf());
        }
    }
    if let Some(root) = env::var_os("HONE_INSTALL_ROOT").map(PathBuf::from) {
        dirs.push(root.join("bin"));
    }
    dirs
}

pub(crate) fn locate_binary(binary: &str) -> Option<PathBuf> {
    locate_binary_with_source(binary, None)
}

pub(crate) fn locate_binary_with_source(
    binary: &str,
    source_root: Option<&Path>,
) -> Option<PathBuf> {
    let name = executable_name(binary);
    for dir in binary_search_dirs(source_root) {
        let candidate = dir.join(&name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(crate) fn child_envs(paths: &ResolvedRuntimePaths) -> Vec<(String, String)> {
    let mut envs = vec![
        (
            "HONE_CONFIG_PATH".to_string(),
            paths.effective_config_path.to_string_lossy().to_string(),
        ),
        (
            "HONE_USER_CONFIG_PATH".to_string(),
            paths.canonical_config_path.to_string_lossy().to_string(),
        ),
        (
            "HONE_DATA_DIR".to_string(),
            paths.data_dir.to_string_lossy().to_string(),
        ),
        (
            "HONE_SKILLS_DIR".to_string(),
            paths.skills_dir.to_string_lossy().to_string(),
        ),
        ("HONE_WEB_PORT".to_string(), paths.web_port.to_string()),
        ("HONE_DISABLE_AUTO_OPEN".to_string(), "1".to_string()),
    ];
    if let Some(root) = env::var_os("HONE_HOME") {
        envs.push((
            "HONE_HOME".to_string(),
            PathBuf::from(root).to_string_lossy().to_string(),
        ));
    }
    envs
}

pub(crate) async fn wait_for_http_ready(url: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let mut last_observation = None;
    for _ in 0..60 {
        match client.get(url).send().await {
            Ok(response) if response.status() == StatusCode::OK => return Ok(()),
            Ok(response) => {
                last_observation = Some(format!("HTTP {}", response.status()));
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(error) => {
                last_observation = Some(error.to_string());
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    Err(http_ready_failure_message(url, last_observation.as_deref()))
}

fn http_ready_failure_message(url: &str, last_observation: Option<&str>) -> String {
    match last_observation.filter(|value| !value.trim().is_empty()) {
        Some(detail) => format!("服务未在预期时间内就绪: {url}（最后一次检查: {detail}）"),
        None => format!("服务未在预期时间内就绪: {url}"),
    }
}

pub(crate) fn missing_binary_message(binary: &str, source_root: Option<&Path>) -> String {
    let executable = executable_name(binary);
    let search_dirs = binary_search_dirs(source_root);
    let checked = if search_dirs.is_empty() {
        "<none>".to_string()
    } else {
        search_dirs
            .iter()
            .map(|dir| dir.to_string_lossy())
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "找不到 {binary} 二进制（文件名: {executable}）。请确认它与 hone-cli 一起安装/构建；源码模式可先运行 `hone-cli start --build` 或设置 HONE_SOURCE_ROOT。已检查: {checked}"
    )
}

pub(crate) fn unexpected_child_exit_message(
    binary: &str,
    status: &std::process::ExitStatus,
) -> String {
    format!("{binary} 异常退出（status={status}）。请查看上方 {binary} 日志。")
}

pub(crate) async fn spawn_binary(
    binary: &str,
    paths: &ResolvedRuntimePaths,
    extra_envs: &[(String, String)],
    source_root: Option<&Path>,
) -> Result<Child, String> {
    let path = locate_binary_with_source(binary, source_root)
        .ok_or_else(|| missing_binary_message(binary, source_root))?;

    let mut command = Command::new(&path);
    command
        .current_dir(&paths.root_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (key, value) in child_envs(paths)
        .into_iter()
        .chain(extra_envs.iter().cloned())
    {
        command.env(key, value);
    }

    command
        .spawn()
        .map_err(|e| format!("启动 {binary} 失败：{e}"))
}

async fn spawn_channel(
    binary: &str,
    label: &str,
    paths: &ResolvedRuntimePaths,
    source_root: Option<&Path>,
) -> Result<Child, String> {
    println!("[INFO] starting {label}...");
    let mut child = spawn_binary(binary, paths, &[], source_root).await?;
    ensure_child_alive(binary, &mut child).await?;
    println!("[INFO] {label} running");
    Ok(child)
}

pub(crate) async fn ensure_child_alive(binary: &str, child: &mut Child) -> Result<(), String> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    match child.try_wait() {
        Ok(Some(status)) => Err(format!(
            "{binary} 启动后立即退出（status={status}）。请查看上方 {binary} 日志。"
        )),
        Ok(None) => Ok(()),
        Err(error) => Err(format!("检查 {binary} 进程状态失败：{error}")),
    }
}

async fn wait_for_any_child_exit(
    children: &mut [Child],
) -> Result<(usize, std::process::ExitStatus), String> {
    loop {
        for (idx, child) in children.iter_mut().enumerate() {
            match child.try_wait() {
                Ok(Some(status)) => return Ok((idx, status)),
                Ok(None) => {}
                Err(error) => return Err(format!("检查子进程状态失败：{error}")),
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

fn unexpected_exit_hint(binary: &str) -> Option<&'static str> {
    match binary {
        "hone-discord" => Some(
            "Discord 子进程异常退出。若日志含“invalid authentication / 4004”，请检查 Discord bot token 是否重复粘贴或已失效；可运行 `hone-cli configure --section channels` 重新配置。",
        ),
        _ => None,
    }
}

async fn shutdown_children(children: &mut [Child]) {
    for child in children.iter_mut().rev() {
        if let Ok(Some(_)) = child.try_wait() {
            continue;
        }
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
}

async fn build_source_runtime_binaries(source_root: &Path) -> Result<(), String> {
    println!(
        "[INFO] building local source CLI/runtime binaries in {}",
        source_root.display()
    );
    let mut command = Command::new("cargo");
    command.arg("build");
    for package in SOURCE_RUNTIME_PACKAGES {
        command.arg("-p").arg(package);
    }
    command
        .current_dir(source_root)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = command
        .status()
        .await
        .map_err(|e| format!("运行 cargo build 失败：{e}"))?;
    if !status.success() {
        return Err(format!("本地 CLI/runtime 构建失败（status={status}）"));
    }
    Ok(())
}

fn write_current_pid(paths: &ResolvedRuntimePaths) -> Result<(), String> {
    fs::create_dir_all(&paths.runtime_dir)
        .map_err(|e| format!("无法创建 runtime 目录 {}: {e}", paths.runtime_dir.display()))?;
    fs::write(
        paths.runtime_dir.join("current.pid"),
        std::process::id().to_string(),
    )
    .map_err(|e| format!("无法写入 current.pid: {e}"))
}

fn clear_current_pid(paths: &ResolvedRuntimePaths) {
    let path = paths.runtime_dir.join("current.pid");
    let current = fs::read_to_string(&path).ok();
    let pid = std::process::id().to_string();
    if current.as_deref().map(str::trim) == Some(pid.as_str()) {
        let _ = fs::remove_file(path);
    }
}

pub(crate) async fn run_start(
    explicit_config: Option<&Path>,
    args: StartArgs,
) -> Result<(), String> {
    let source_root = if args.build {
        Some(source_root_from_env_or_cwd(args.source_root.as_deref()).ok_or_else(|| {
            "找不到源码 checkout；请在项目目录运行 `hone-cli start --build`，或传入 --source-root"
                .to_string()
        })?)
    } else {
        source_root_from_env_or_cwd(args.source_root.as_deref())
    };

    if args.build {
        build_source_runtime_binaries(source_root.as_deref().expect("source root")).await?;
    }

    let (config, paths) = load_cli_config(explicit_config, true).map_err(|e| e.to_string())?;
    let _ = resolve_runtime_paths(explicit_config, true).map_err(|e| e.to_string())?;

    let lock_names = hone_core::enabled_process_lock_names(&config);
    let mut on_warn = |message: &str| println!("[WARN] {message}");
    if let Err(error) = hone_core::preflight_process_locks_with_cleanup(
        &paths.runtime_dir,
        &lock_names,
        &mut on_warn,
    ) {
        return Err(hone_core::format_lock_failure_message(
            &error.process,
            &error.path,
            &error.source,
            "Hone CLI",
        ));
    }

    let mut children = Vec::new();
    let mut labels = Vec::new();

    println!(
        "[INFO] starting backend using {}",
        paths.effective_config_path.to_string_lossy()
    );
    let mut backend =
        spawn_binary("hone-console-page", &paths, &[], source_root.as_deref()).await?;
    ensure_child_alive("hone-console-page", &mut backend).await?;
    let meta_url = format!("http://127.0.0.1:{}/api/meta", paths.web_port);
    if let Err(error) = wait_for_http_ready(&meta_url).await {
        let _ = backend.kill().await;
        let _ = backend.wait().await;
        return Err(error);
    }
    println!(
        "[INFO] backend ready at http://127.0.0.1:{}",
        paths.web_port
    );
    children.push(backend);
    labels.push("hone-console-page");

    if config.imessage.enabled {
        children.push(
            spawn_channel("hone-imessage", "iMessage", &paths, source_root.as_deref()).await?,
        );
        labels.push("hone-imessage");
    }
    if config.discord.enabled {
        children
            .push(spawn_channel("hone-discord", "Discord", &paths, source_root.as_deref()).await?);
        labels.push("hone-discord");
    }
    if config.feishu.enabled {
        children
            .push(spawn_channel("hone-feishu", "Feishu", &paths, source_root.as_deref()).await?);
        labels.push("hone-feishu");
    }
    if config.telegram.enabled {
        children.push(
            spawn_channel("hone-telegram", "Telegram", &paths, source_root.as_deref()).await?,
        );
        labels.push("hone-telegram");
    }

    write_current_pid(&paths)?;
    println!(
        "[INFO] admin UI available at http://127.0.0.1:{}",
        paths.web_port
    );
    println!(
        "[INFO] user UI available at http://127.0.0.1:{}",
        public_web_port_from_env()
    );
    println!(
        "[INFO] source frontend helpers: hone-cli web admin-ui --dev / hone-cli web user-ui --dev"
    );
    println!("[INFO] press Ctrl-C to stop.");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("[INFO] shutdown requested");
        }
        result = wait_for_any_child_exit(&mut children) => {
            match result {
                Ok((idx, status)) => {
                    shutdown_children(&mut children).await;
                    clear_current_pid(&paths);
                    let binary = labels.get(idx).copied().unwrap_or("unknown");
                    let base_error = unexpected_child_exit_message(binary, &status);
                    if let Some(hint) = unexpected_exit_hint(binary) {
                        return Err(format!("{base_error}\n{hint}"));
                    }
                    return Err(base_error);
                }
                Err(error) => {
                    shutdown_children(&mut children).await;
                    clear_current_pid(&paths);
                    return Err(format!("等待子进程退出时失败：{error}"));
                }
            }
        }
    }

    shutdown_children(&mut children).await;
    clear_current_pid(&paths);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unexpected_exit_hint_includes_discord_token_guidance() {
        let hint = unexpected_exit_hint("hone-discord").unwrap_or_default();
        assert!(hint.contains("token"));
        assert!(hint.contains("configure --section channels"));
    }

    #[test]
    fn unexpected_exit_hint_is_none_for_other_processes() {
        assert!(unexpected_exit_hint("hone-feishu").is_none());
    }

    #[test]
    fn locate_binary_with_source_root_prefers_source_target() {
        let root = std::env::temp_dir().join(format!(
            "hone_source_target_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let debug_dir = root.join("target/debug");
        std::fs::create_dir_all(&debug_dir).unwrap();
        let binary = debug_dir.join(executable_name("hone-console-page"));
        std::fs::write(&binary, "mock").unwrap();

        let resolved = locate_binary_with_source("hone-console-page", Some(&root))
            .expect("source binary should resolve");
        assert_eq!(resolved, binary);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn missing_binary_message_lists_checked_source_dir() {
        let root = std::env::temp_dir().join(format!(
            "hone_missing_binary_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let message = missing_binary_message("hone-console-page", Some(&root));

        assert!(message.contains("hone-console-page"));
        assert!(message.contains(&source_target_debug_dir(&root).to_string_lossy().to_string()));
        assert!(message.contains("HONE_SOURCE_ROOT"));
    }

    #[test]
    fn http_ready_failure_message_keeps_last_observation() {
        let message =
            http_ready_failure_message("http://127.0.0.1:8077/api/meta", Some("HTTP 503"));

        assert!(message.contains("http://127.0.0.1:8077/api/meta"));
        assert!(message.contains("HTTP 503"));
    }

    #[test]
    fn unexpected_child_exit_message_points_to_process_logs() {
        #[cfg(unix)]
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg("exit 7")
            .status()
            .expect("quick command should run");
        #[cfg(windows)]
        let status = std::process::Command::new("cmd")
            .args(["/C", "exit /B 7"])
            .status()
            .expect("quick command should run");
        let message = unexpected_child_exit_message("hone-console-page", &status);

        assert!(message.contains("hone-console-page"));
        assert!(message.contains("status="));
        assert!(message.contains("日志"));
    }

    fn long_running_command() -> Command {
        #[cfg(unix)]
        {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg("sleep 3");
            cmd
        }
        #[cfg(windows)]
        {
            let mut cmd = Command::new("cmd");
            cmd.args([
                "/C",
                "powershell -NoLogo -NoProfile -Command \"Start-Sleep -Seconds 3\"",
            ]);
            cmd
        }
    }

    fn quick_exit_command(exit_code: i32) -> Command {
        #[cfg(unix)]
        {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(format!("exit {exit_code}"));
            cmd
        }
        #[cfg(windows)]
        {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", &format!("exit /B {exit_code}")]);
            cmd
        }
    }

    #[tokio::test]
    async fn wait_for_any_child_exit_returns_exited_child_index_and_status() {
        let mut slow = long_running_command();
        slow.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let slow_child = slow.spawn().expect("spawn slow child");

        let mut fast = quick_exit_command(7);
        fast.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let fast_child = fast.spawn().expect("spawn fast child");

        let mut children = vec![slow_child, fast_child];
        let (idx, status) = wait_for_any_child_exit(&mut children)
            .await
            .expect("wait_for_any_child_exit should succeed");

        assert_eq!(idx, 1);
        assert_eq!(status.code(), Some(7));
        assert!(
            children[0].try_wait().expect("query slow child").is_none(),
            "the first child should still be running when the second child exits"
        );

        children[0].kill().await.expect("kill slow child");
        children[0].wait().await.expect("wait slow child");
        children[1].wait().await.expect("wait fast child");
    }
}
