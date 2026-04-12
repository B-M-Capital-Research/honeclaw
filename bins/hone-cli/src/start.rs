use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use reqwest::StatusCode;
use tokio::process::{Child, Command};

use crate::common::{ResolvedRuntimePaths, load_cli_config, resolve_runtime_paths};

fn executable_name(binary: &str) -> String {
    if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    }
}

fn current_exe_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.to_path_buf());
            if parent.file_name().and_then(OsStr::to_str) == Some("deps") {
                if let Some(grandparent) = parent.parent() {
                    dirs.push(grandparent.to_path_buf());
                }
            }
        }
    }
    if let Some(root) = env::var_os("HONE_INSTALL_ROOT").map(PathBuf::from) {
        dirs.push(root.join("bin"));
    }
    dirs
}

pub(crate) fn locate_binary(binary: &str) -> Option<PathBuf> {
    let name = executable_name(binary);
    for dir in current_exe_dirs() {
        let candidate = dir.join(&name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn child_envs(paths: &ResolvedRuntimePaths) -> Vec<(String, String)> {
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

async fn wait_for_http_ready(url: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    for _ in 0..60 {
        match client.get(url).send().await {
            Ok(response) if response.status() == StatusCode::OK => return Ok(()),
            Ok(_) | Err(_) => tokio::time::sleep(Duration::from_millis(500)).await,
        }
    }
    Err(format!("服务未在预期时间内就绪: {url}"))
}

async fn spawn_binary(
    binary: &str,
    paths: &ResolvedRuntimePaths,
    extra_envs: &[(String, String)],
) -> Result<Child, String> {
    let path = locate_binary(binary)
        .ok_or_else(|| format!("找不到 {binary} 二进制；请确认它与 hone-cli 一起安装/构建"))?;

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
        .map_err(|e| format!("启动 {binary} 失败: {e}"))
}

async fn spawn_channel(
    binary: &str,
    label: &str,
    paths: &ResolvedRuntimePaths,
) -> Result<Child, String> {
    println!("[INFO] starting {label}...");
    let mut child = spawn_binary(binary, paths, &[]).await?;
    ensure_child_alive(binary, &mut child).await?;
    println!("[INFO] {label} running");
    Ok(child)
}

async fn ensure_child_alive(binary: &str, child: &mut Child) -> Result<(), String> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    match child.try_wait() {
        Ok(Some(status)) => Err(format!("{binary} 启动后立即退出（status={status}）")),
        Ok(None) => Ok(()),
        Err(error) => Err(format!("检查 {binary} 进程状态失败: {error}")),
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

pub(crate) async fn run_start(explicit_config: Option<&Path>) -> Result<(), String> {
    let (config, paths) = load_cli_config(explicit_config, true).map_err(|e| e.to_string())?;
    let _ = resolve_runtime_paths(explicit_config, true).map_err(|e| e.to_string())?;

    let mut children = Vec::new();

    println!(
        "[INFO] starting backend using {}",
        paths.effective_config_path.to_string_lossy()
    );
    let mut backend = spawn_binary("hone-console-page", &paths, &[]).await?;
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

    if config.imessage.enabled {
        children.push(spawn_channel("hone-imessage", "iMessage", &paths).await?);
    }
    if config.discord.enabled {
        children.push(spawn_channel("hone-discord", "Discord", &paths).await?);
    }
    if config.feishu.enabled {
        children.push(spawn_channel("hone-feishu", "Feishu", &paths).await?);
    }
    if config.telegram.enabled {
        children.push(spawn_channel("hone-telegram", "Telegram", &paths).await?);
    }

    println!("[INFO] frontend disabled. use the web console or desktop separately if needed.");
    println!("[INFO] press Ctrl-C to stop.");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("[INFO] shutdown requested");
        }
        result = async {
            if let Some(first) = children.first_mut() {
                first.wait().await.map_err(|e| e.to_string())
            } else {
                Ok(std::process::ExitStatus::from_raw(0))
            }
        } => {
            match result {
                Ok(status) => {
                    shutdown_children(&mut children).await;
                    return Err(format!("backend exited unexpectedly: {status}"));
                }
                Err(error) => {
                    shutdown_children(&mut children).await;
                    return Err(format!("等待 backend 退出时失败: {error}"));
                }
            }
        }
    }

    shutdown_children(&mut children).await;
    Ok(())
}

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(windows)]
trait ExitStatusExtCompat {
    fn from_raw(code: u32) -> std::process::ExitStatus;
}

#[cfg(windows)]
impl ExitStatusExtCompat for std::process::ExitStatus {
    fn from_raw(code: u32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }
}
