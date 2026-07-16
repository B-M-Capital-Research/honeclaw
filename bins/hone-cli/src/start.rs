use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use clap::Args;
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::process::{Child, Command};

use crate::common::{ResolvedRuntimePaths, load_cli_config, resolve_runtime_paths};
use hone_core::cloud_runtime::RuntimeRole;

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
const ACTIVE_CHAT_DRAIN_MAX_WAIT: Duration = Duration::from_secs(6 * 60);
const ACTIVE_CHAT_DRAIN_GRACE: Duration = Duration::from_secs(15);
const ACTIVE_CHAT_DRAIN_POLL_INTERVAL: Duration = Duration::from_secs(1);
const ACTIVE_CHAT_DRAIN_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const ACTIVE_CHAT_DRAIN_MAX_CONSECUTIVE_FAILURES: usize = 3;

#[derive(Debug, Deserialize)]
struct ActiveChatRunsResponse {
    count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveChatDrainPollDecision {
    Drained,
    Waiting { active_count: usize },
    RetryUnavailable { consecutive_failures: usize },
    ProceedUnavailable,
}

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
    if let Some(hone_mcp_bin) = locate_binary_with_source("hone-mcp", Some(&paths.root_dir)) {
        envs.push((
            "HONE_MCP_BIN".to_string(),
            hone_mcp_bin.to_string_lossy().to_string(),
        ));
    }
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

fn active_chat_drain_timeout(agent_overall_timeout: Duration) -> Duration {
    agent_overall_timeout
        .saturating_add(ACTIVE_CHAT_DRAIN_GRACE)
        .min(ACTIVE_CHAT_DRAIN_MAX_WAIT)
}

fn classify_active_chat_drain_poll(
    active_count: Result<usize, ()>,
    previous_consecutive_failures: usize,
) -> ActiveChatDrainPollDecision {
    match active_count {
        Ok(0) => ActiveChatDrainPollDecision::Drained,
        Ok(active_count) => ActiveChatDrainPollDecision::Waiting { active_count },
        Err(()) => {
            let consecutive_failures = previous_consecutive_failures.saturating_add(1);
            if consecutive_failures >= ACTIVE_CHAT_DRAIN_MAX_CONSECUTIVE_FAILURES {
                ActiveChatDrainPollDecision::ProceedUnavailable
            } else {
                ActiveChatDrainPollDecision::RetryUnavailable {
                    consecutive_failures,
                }
            }
        }
    }
}

fn parse_active_chat_runs_response(body: &str) -> Result<usize, String> {
    serde_json::from_str::<ActiveChatRunsResponse>(body)
        .map(|response| response.count)
        .map_err(|error| format!("活动聊天任务响应格式无效：{error}"))
}

async fn fetch_active_chat_run_count(
    client: &reqwest::Client,
    url: &str,
    auth_token: Option<&str>,
) -> Result<usize, String> {
    let mut request = client.get(url);
    if let Some(auth_token) = auth_token.filter(|token| !token.trim().is_empty()) {
        request = request.bearer_auth(auth_token.trim());
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("请求失败：{error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("读取响应失败：{error}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    parse_active_chat_runs_response(&body)
}

async fn wait_for_active_chat_drain(url: &str, auth_token: Option<&str>, timeout: Duration) {
    let client = match reqwest::Client::builder()
        .timeout(ACTIVE_CHAT_DRAIN_REQUEST_TIMEOUT)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            println!("[WARN] 无法创建活动聊天任务检查客户端，将继续关闭服务：{error}");
            return;
        }
    };
    let started_at = tokio::time::Instant::now();
    let deadline = started_at + timeout;
    let mut consecutive_failures = 0usize;
    let mut last_active_count = None;

    loop {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            let active_detail = last_active_count
                .map(|count| format!("，最后一次检测到 {count} 个活动任务"))
                .unwrap_or_default();
            println!(
                "[WARN] 等待活动聊天任务结束已达到 {} 秒上限{active_detail}，将继续关闭服务",
                timeout.as_secs()
            );
            return;
        }

        let remaining = deadline.saturating_duration_since(now);
        let observation = match tokio::time::timeout(
            remaining.min(ACTIVE_CHAT_DRAIN_REQUEST_TIMEOUT),
            fetch_active_chat_run_count(&client, url, auth_token),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err("请求超时".to_string()),
        };
        let observation_error = observation.as_ref().err().cloned();
        let decision = classify_active_chat_drain_poll(
            observation.as_ref().map(|count| *count).map_err(|_| ()),
            consecutive_failures,
        );

        match decision {
            ActiveChatDrainPollDecision::Drained => {
                if last_active_count.is_some() {
                    println!("[INFO] 活动聊天任务已全部结束，继续关闭服务");
                } else {
                    println!("[INFO] 当前没有活动聊天任务，继续关闭服务");
                }
                return;
            }
            ActiveChatDrainPollDecision::Waiting { active_count } => {
                consecutive_failures = 0;
                if last_active_count != Some(active_count) {
                    println!(
                        "[INFO] 正在等待 {active_count} 个活动聊天任务结束（最多等待 {} 秒）...",
                        timeout.as_secs()
                    );
                    last_active_count = Some(active_count);
                }
            }
            ActiveChatDrainPollDecision::RetryUnavailable {
                consecutive_failures: failures,
            } => {
                consecutive_failures = failures;
                println!(
                    "[WARN] 暂时无法查询活动聊天任务（{failures}/{ACTIVE_CHAT_DRAIN_MAX_CONSECUTIVE_FAILURES}）：{}",
                    observation_error.as_deref().unwrap_or("未知错误")
                );
            }
            ActiveChatDrainPollDecision::ProceedUnavailable => {
                println!(
                    "[WARN] 连续 {ACTIVE_CHAT_DRAIN_MAX_CONSECUTIVE_FAILURES} 次无法查询活动聊天任务，将继续关闭服务：{}",
                    observation_error.as_deref().unwrap_or("未知错误")
                );
                return;
            }
        }

        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            continue;
        }
        tokio::time::sleep(remaining.min(ACTIVE_CHAT_DRAIN_POLL_INTERVAL)).await;
    }
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
    isolate_runtime_child_from_terminal_interrupt(&mut command);
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

fn isolate_runtime_child_from_terminal_interrupt(command: &mut Command) {
    // Interactive Ctrl-C targets the terminal's foreground process group. Keep
    // runtime children in their own groups so only the CLI supervisor receives
    // SIGINT, queries the Web drain endpoint, and then terminates children in a
    // controlled order after active chats reach zero.
    #[cfg(unix)]
    command.process_group(0);
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
    let runtime_role = RuntimeRole::from_env();
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

    if runtime_role.runs_worker_tasks() && config.imessage.enabled {
        children.push(
            spawn_channel("hone-imessage", "iMessage", &paths, source_root.as_deref()).await?,
        );
        labels.push("hone-imessage");
    }
    if runtime_role.runs_worker_tasks() && config.discord.enabled {
        children
            .push(spawn_channel("hone-discord", "Discord", &paths, source_root.as_deref()).await?);
        labels.push("hone-discord");
    }
    if runtime_role.runs_worker_tasks() && config.feishu.enabled {
        children
            .push(spawn_channel("hone-feishu", "Feishu", &paths, source_root.as_deref()).await?);
        labels.push("hone-feishu");
    }
    if runtime_role.runs_worker_tasks() && config.telegram.enabled {
        children.push(
            spawn_channel("hone-telegram", "Telegram", &paths, source_root.as_deref()).await?,
        );
        labels.push("hone-telegram");
    }
    if !runtime_role.runs_worker_tasks() {
        println!("[INFO] HONE_RUNTIME_ROLE=web: channel sidecars disabled");
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

    let active_chat_runs_url = format!(
        "http://127.0.0.1:{}/api/runtime/active-chat-runs",
        paths.web_port
    );
    wait_for_active_chat_drain(
        &active_chat_runs_url,
        Some(config.web.auth_token.as_str()),
        active_chat_drain_timeout(config.agent.overall_timeout()),
    )
    .await;
    shutdown_children(&mut children).await;
    clear_current_pid(&paths);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_chat_drain_timeout_uses_agent_timeout_with_grace() {
        assert_eq!(
            active_chat_drain_timeout(Duration::from_secs(120)),
            Duration::from_secs(135)
        );
    }

    #[test]
    fn active_chat_drain_timeout_is_capped_at_six_minutes() {
        assert_eq!(
            active_chat_drain_timeout(Duration::from_secs(20 * 60)),
            ACTIVE_CHAT_DRAIN_MAX_WAIT
        );
    }

    #[test]
    fn active_chat_run_response_requires_numeric_count() {
        assert_eq!(
            parse_active_chat_runs_response(r#"{"count":2}"#).expect("valid response"),
            2
        );
        assert!(parse_active_chat_runs_response(r#"{"count":"2"}"#).is_err());
        assert!(parse_active_chat_runs_response(r#"{}"#).is_err());
    }

    #[test]
    fn active_chat_drain_poll_waits_until_count_reaches_zero() {
        assert_eq!(
            classify_active_chat_drain_poll(Ok(2), 1),
            ActiveChatDrainPollDecision::Waiting { active_count: 2 }
        );
        assert_eq!(
            classify_active_chat_drain_poll(Ok(0), 0),
            ActiveChatDrainPollDecision::Drained
        );
    }

    #[test]
    fn active_chat_drain_poll_gives_up_after_bounded_query_failures() {
        assert_eq!(
            classify_active_chat_drain_poll(Err(()), 0),
            ActiveChatDrainPollDecision::RetryUnavailable {
                consecutive_failures: 1
            }
        );
        assert_eq!(
            classify_active_chat_drain_poll(
                Err(()),
                ACTIVE_CHAT_DRAIN_MAX_CONSECUTIVE_FAILURES - 1
            ),
            ActiveChatDrainPollDecision::ProceedUnavailable
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn runtime_child_isolated_from_supervisor_process_group() {
        let mut command = Command::new("sh");
        isolate_runtime_child_from_terminal_interrupt(&mut command);
        let status = command
            .arg("-c")
            .arg(
                "child_pgid=$(ps -o pgid= -p $$ | tr -d ' '); \
                 parent_pgid=$(ps -o pgid= -p $PPID | tr -d ' '); \
                 test -n \"$child_pgid\" -a -n \"$parent_pgid\" \
                   -a \"$child_pgid\" != \"$parent_pgid\"",
            )
            .status()
            .await
            .expect("spawn isolated child");

        assert!(status.success());
    }

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
        let debug_dir = source_target_debug_dir(&root);
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
    fn child_envs_exports_hone_mcp_bin_from_source_root() {
        let root = std::env::temp_dir().join(format!(
            "hone_child_envs_mcp_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let debug_dir = source_target_debug_dir(&root);
        let runtime_dir = root.join("data/runtime");
        std::fs::create_dir_all(&debug_dir).unwrap();
        std::fs::create_dir_all(&runtime_dir).unwrap();
        let hone_mcp = debug_dir.join(executable_name("hone-mcp"));
        std::fs::write(&hone_mcp, "mock").unwrap();

        let paths = ResolvedRuntimePaths {
            canonical_config_path: root.join("config.yaml"),
            effective_config_path: runtime_dir.join("effective-config.yaml"),
            data_dir: root.join("data"),
            runtime_dir,
            skills_dir: root.join("skills"),
            root_dir: root.clone(),
            web_port: 8077,
        };
        let envs = child_envs(&paths);

        assert!(envs.iter().any(|(key, value)| {
            key == "HONE_MCP_BIN" && value == &hone_mcp.to_string_lossy().to_string()
        }));

        let _ = std::fs::remove_dir_all(root);
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
