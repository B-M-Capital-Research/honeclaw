#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use hone_core::HoneConfig;
use hone_core::config::{diff_yaml_value, read_yaml_value, runtime_overlay_path};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tauri::async_runtime;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::{
    ShellExt,
    process::{CommandChild, CommandEvent},
};

#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

const API_VERSION: &str = "desktop-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BackendConfig {
    mode: String,
    #[serde(default, alias = "base_url")]
    base_url: String,
    #[serde(default, alias = "bearer_token")]
    bearer_token: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            mode: "bundled".to_string(),
            base_url: String::new(),
            bearer_token: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopChannelSettings {
    config_path: String,
    imessage_enabled: bool,
    feishu_enabled: bool,
    telegram_enabled: bool,
    discord_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopChannelSettingsInput {
    imessage_enabled: bool,
    feishu_enabled: bool,
    telegram_enabled: bool,
    discord_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopChannelSettingsUpdateResult {
    settings: DesktopChannelSettings,
    restarted_bundled_backend: bool,
    message: String,
    backend_status: Option<BackendStatusInfo>,
}

/// Agent 基础设置（写入运行时覆盖层）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSettings {
    /// function_calling | gemini_cli | gemini_acp | codex_cli | codex_acp | opencode_acp
    runner: String,
    /// codex_cli 专用，其他 provider 忽略
    codex_model: String,
    /// OpenAI 协议渠道 Base URL（agent.opencode.api_base_url）
    #[serde(default)]
    openai_url: String,
    /// OpenAI 协议渠道模型名（agent.opencode.model）
    #[serde(default)]
    openai_model: String,
    /// OpenRouter 子模型（llm.openrouter.sub_model），用于心跳/压缩等辅助链路
    #[serde(default)]
    openai_sub_model: String,
    /// OpenAI 协议渠道 API Key（agent.opencode.api_key）
    #[serde(default)]
    openai_api_key: String,
}

/// CLI 联通检测结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliCheckResult {
    ok: bool,
    message: String,
}

/// OpenRouter API Key 设置（写入运行时覆盖层的 llm.openrouter.api_keys）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenRouterSettings {
    /// 多 Key 列表（支持 fallback）
    api_keys: Vec<String>,
}

/// FMP API Key 设置（写入运行时覆盖层的 fmp.api_keys）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FmpSettings {
    /// 多 Key 列表（支持 fallback）
    api_keys: Vec<String>,
}

/// Tavily API Key 设置（写入运行时覆盖层的 search.api_keys）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TavilySettings {
    /// 多 Key 列表（支持 fallback）
    api_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaInfo {
    name: String,
    version: String,
    channel: String,
    supports_imessage: bool,
    api_version: String,
    capabilities: Vec<String>,
    deployment_mode: String,
}

fn desktop_meta_from_config(config: &HoneConfig, deployment_mode: &str) -> MetaInfo {
    let mut capabilities = vec![
        "channels".to_string(),
        "users".to_string(),
        "history".to_string(),
        "chat".to_string(),
        "sse.events".to_string(),
        "logs".to_string(),
        "skills".to_string(),
        "cron_jobs".to_string(),
        "portfolio".to_string(),
        "research".to_string(),
        "llm_audit".to_string(),
    ];

    if deployment_mode == "local" {
        capabilities.push("local_file_proxy".to_string());
    }

    if cfg!(target_os = "macos") && config.imessage.enabled {
        capabilities.push("imessage".to_string());
    }

    capabilities.sort();

    MetaInfo {
        name: "Hone".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        channel: "imessage".to_string(),
        supports_imessage: cfg!(target_os = "macos"),
        api_version: API_VERSION.to_string(),
        capabilities,
        deployment_mode: deployment_mode.to_string(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BackendStatusInfo {
    config: BackendConfig,
    resolved_base_url: Option<String>,
    connected: bool,
    last_error: Option<String>,
    meta: Option<MetaInfo>,
    diagnostics: DiagnosticPaths,
}

#[derive(Default)]
pub(crate) struct DesktopState {
    inner: Mutex<DesktopBackendManager>,
    transition_lock: tokio::sync::Mutex<()>,
}

#[derive(Default)]
struct DesktopBackendManager {
    config: BackendConfig,
    resolved_base_url: Option<String>,
    meta: Option<MetaInfo>,
    last_error: Option<String>,
    /// 内嵌 Axum 服务任务句柄（bundled 模式）
    web_server_task: Option<tokio::task::JoinHandle<()>>,
    /// 各 channel sidecar 子进程（imessage / discord / feishu / telegram）
    channel_children: BTreeMap<String, CommandChild>,
    diagnostics: Option<DiagnosticPaths>,
}

impl Drop for DesktopBackendManager {
    fn drop(&mut self) {
        stop_managed_children(self);
    }
}

struct RuntimePaths {
    config_path: PathBuf,
    data_dir: PathBuf,
    runtime_dir: PathBuf,
    skills_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct DiagnosticPaths {
    config_dir: String,
    data_dir: String,
    logs_dir: String,
    desktop_log: String,
    sidecar_log: String,
}

fn normalize_base_url(raw: &str) -> String {
    raw.trim().trim_end_matches('/').to_string()
}

fn timestamp_string() -> String {
    // GMT+8 (Asia/Shanghai) 可读格式，如 2026-03-15 10:30:00.123
    let tz = chrono::FixedOffset::east_opt(8 * 3600).expect("valid tz");
    chrono::Utc::now()
        .with_timezone(&tz)
        .format("%Y-%m-%d %H:%M:%S%.3f")
        .to_string()
}

fn append_log(path: &PathBuf, level: &str, message: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "[{}] {:<5} {}", timestamp_string(), level, message);
    }
}

fn diagnostic_paths(app: &AppHandle) -> Result<DiagnosticPaths, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    // 与 ensure_runtime_paths 保持一致：优先使用 HONE_DESKTOP_DATA_DIR 覆盖
    let data_dir = if let Ok(override_dir) = std::env::var("HONE_DESKTOP_DATA_DIR") {
        PathBuf::from(override_dir)
    } else {
        app.path().app_data_dir().map_err(|e| e.to_string())?
    };
    // 日志统一写到 data/runtime/logs/，与 hone-web-api 及各 sidecar 保持一致
    let logs_dir = data_dir.join("runtime").join("logs");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;

    Ok(DiagnosticPaths {
        config_dir: config_dir.to_string_lossy().to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        logs_dir: logs_dir.to_string_lossy().to_string(),
        desktop_log: logs_dir.join("desktop.log").to_string_lossy().to_string(),
        sidecar_log: logs_dir.join("sidecar.log").to_string_lossy().to_string(),
    })
}

fn log_desktop(app: &AppHandle, level: &str, message: impl AsRef<str>) {
    if let Ok(paths) = diagnostic_paths(app) {
        append_log(&PathBuf::from(paths.desktop_log), level, message.as_ref());
    }
}

fn config_store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("backend.json"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn resource_or_repo_path(app: &AppHandle, resource: &str) -> PathBuf {
    app.path()
        .resource_dir()
        .ok()
        .map(|dir| dir.join(resource))
        .filter(|path| path.exists())
        .unwrap_or_else(|| repo_root().join(resource))
}

fn ensure_runtime_paths(app: &AppHandle) -> Result<RuntimePaths, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;

    let data_dir = if let Ok(override_dir) = std::env::var("HONE_DESKTOP_DATA_DIR") {
        // 显式覆盖（优先级最高）
        PathBuf::from(override_dir)
    } else {
        // 开发模式回退：若编译时的项目根目录下有 data/ 目录（说明当前在开发机上），
        // 则直接使用项目的 ./data/，与 launch.sh 共享同一份数据。
        // 打包分发后，用户机器上不存在该编译时路径，自动降级到 Tauri 默认数据目录。
        let dev_data = repo_root().join("data");
        if dev_data.is_dir() {
            dev_data
        } else {
            app.path().app_data_dir().map_err(|e| e.to_string())?
        }
    };
    let runtime_dir = data_dir.join("runtime");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&runtime_dir).map_err(|e| e.to_string())?;

    // ── 运行时配置文件 ──────────────────────────────────────────────────
    // 实际运行中读写的是 data/runtime/config_runtime.yaml（不是项目根的 config.yaml）。
    // 首次启动时，若 config_runtime.yaml 不存在，则从以下来源复制（优先级依次降低）：
    //   1. 项目根目录 config.yaml        ← 开发者手动维护的种子配置
    //   2. 项目根目录 config.example.yaml ← 兜底示例
    // 运行时覆盖层固定写入 data/runtime/config_runtime.overrides.yaml；
    // 这样可以保证项目根的 config.yaml 只作为「种子/模板」不被运行时直接修改，
    // 而 config_runtime.yaml 保持有效运行时基底，覆盖变更独立落盘。
    let config_path = runtime_dir.join("config_runtime.yaml");
    if !config_path.exists() {
        let seed = {
            let root_config = resource_or_repo_path(app, "config.yaml");
            if root_config.exists() {
                root_config
            } else {
                resource_or_repo_path(app, "config.example.yaml")
            }
        };
        fs::copy(&seed, &config_path)
            .map_err(|e| format!("无法初始化 config_runtime.yaml（来源: {seed:?}）: {e}"))?;
        let overlay_path = runtime_overlay_path(&config_path);
        if overlay_path.exists() {
            let _ = fs::remove_file(&overlay_path);
        }
    }

    // soul.md 必须与 config_runtime.yaml 同目录（runtime_dir），
    // 因为 system_prompt_path: "./soul.md" 是相对于 config 文件所在目录解析的。
    let soul_dest = runtime_dir.join("soul.md");
    if !soul_dest.exists() {
        let soul_src = resource_or_repo_path(app, "soul.md");
        if soul_src.exists() {
            fs::copy(&soul_src, &soul_dest)
                .map_err(|e| format!("无法复制 soul.md 到 runtime 目录: {e}"))?;
        }
    }

    let skills_dir = resource_or_repo_path(app, "skills");
    Ok(RuntimePaths {
        config_path,
        data_dir,
        runtime_dir,
        skills_dir,
    })
}

fn load_persisted_config(app: &AppHandle) -> Result<BackendConfig, String> {
    let path = config_store_path(app)?;
    if !path.exists() {
        return Ok(BackendConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_persisted_config(app: &AppHandle, config: &BackendConfig) -> Result<(), String> {
    let path = config_store_path(app)?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn atomic_write_yaml(path: &Path, yaml: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("覆盖层路径缺少父目录: {}", path.display()))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("config");
    let tmp_path = parent.join(format!(".{file_name}.{stamp}.tmp"));

    fs::write(&tmp_path, yaml).map_err(|e| e.to_string())?;
    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(first_err) => {
            let _ = fs::remove_file(path);
            match fs::rename(&tmp_path, path) {
                Ok(()) => Ok(()),
                Err(second_err) => {
                    let _ = fs::remove_file(&tmp_path);
                    Err(format!(
                        "无法写入覆盖层 {}: {second_err}（初次重命名错误: {first_err}）",
                        path.display()
                    ))
                }
            }
        }
    }
}

fn write_overlay_patch(path: &Path, patch: Option<serde_yaml::Value>) -> Result<(), String> {
    match patch {
        None => {
            if path.exists() {
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        Some(serde_yaml::Value::Mapping(map)) if map.is_empty() => {
            if path.exists() {
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        Some(value) => {
            let yaml = serde_yaml::to_string(&value).map_err(|e| e.to_string())?;
            atomic_write_yaml(path, &yaml)
        }
    }
}

fn save_runtime_config_overlay<F>(config_path: &Path, mutate: F) -> Result<HoneConfig, String>
where
    F: FnOnce(&mut HoneConfig),
{
    let base_value = read_yaml_value(config_path).map_err(|e| e.to_string())?;
    let mut config = HoneConfig::from_file(config_path).map_err(|e| e.to_string())?;
    mutate(&mut config);
    let current_value = serde_yaml::to_value(&config).map_err(|e| e.to_string())?;
    let patch = diff_yaml_value(&base_value, &current_value);
    let overlay_path = runtime_overlay_path(config_path);
    write_overlay_patch(&overlay_path, patch)?;
    Ok(config)
}

fn load_channel_settings(app: &AppHandle) -> Result<DesktopChannelSettings, String> {
    let runtime = ensure_runtime_paths(app)?;
    let config_path = runtime.config_path;
    let config = HoneConfig::from_file(&config_path).map_err(|e| e.to_string())?;
    Ok(DesktopChannelSettings {
        config_path: config_path.to_string_lossy().to_string(),
        imessage_enabled: config.imessage.enabled,
        feishu_enabled: config.feishu.enabled,
        telegram_enabled: config.telegram.enabled,
        discord_enabled: config.discord.enabled,
    })
}

fn save_channel_settings(
    app: &AppHandle,
    settings: &DesktopChannelSettingsInput,
) -> Result<DesktopChannelSettings, String> {
    let runtime = ensure_runtime_paths(app)?;
    let config_path = runtime.config_path;
    let config = save_runtime_config_overlay(&config_path, |config| {
        config.imessage.enabled = settings.imessage_enabled;
        config.feishu.enabled = settings.feishu_enabled;
        config.telegram.enabled = settings.telegram_enabled;
        config.discord.enabled = settings.discord_enabled;
    })?;

    Ok(DesktopChannelSettings {
        config_path: config_path.to_string_lossy().to_string(),
        imessage_enabled: config.imessage.enabled,
        feishu_enabled: config.feishu.enabled,
        telegram_enabled: config.telegram.enabled,
        discord_enabled: config.discord.enabled,
    })
}

fn resolved_base_url(config: &BackendConfig, manager: &DesktopBackendManager) -> Option<String> {
    match config.mode.as_str() {
        "bundled" => manager.resolved_base_url.clone(),
        "remote" => Some(normalize_base_url(&config.base_url)),
        _ => None,
    }
}

fn backend_status_snapshot(manager: &DesktopBackendManager) -> BackendStatusInfo {
    let resolved = resolved_base_url(&manager.config, manager);
    BackendStatusInfo {
        config: manager.config.clone(),
        resolved_base_url: resolved,
        connected: manager.meta.is_some() && manager.last_error.is_none(),
        last_error: manager.last_error.clone(),
        meta: manager.meta.clone(),
        diagnostics: manager
            .diagnostics
            .clone()
            .unwrap_or_else(DesktopBackendManager::fallback_diagnostics),
    }
}

fn validate_meta(meta: MetaInfo) -> Result<MetaInfo, String> {
    if meta.api_version == API_VERSION {
        Ok(meta)
    } else {
        Err(format!(
            "unsupported backend api_version: {} (expected {API_VERSION})",
            meta.api_version
        ))
    }
}

/// 停止内嵌 web 服务 task（abort）
fn stop_web_server(manager: &mut DesktopBackendManager) {
    if let Some(handle) = manager.web_server_task.take() {
        handle.abort();
    }
}

fn stop_managed_children(manager: &mut DesktopBackendManager) {
    stop_web_server(manager);
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

impl DesktopBackendManager {
    fn fallback_diagnostics() -> DiagnosticPaths {
        DiagnosticPaths {
            config_dir: String::new(),
            data_dir: String::new(),
            logs_dir: String::new(),
            desktop_log: String::new(),
            sidecar_log: String::new(),
        }
    }
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<String>,
}

impl ScopedEnvVar {
    fn remove(key: &'static str) -> Self {
        let previous = env::var(key).ok();
        unsafe {
            env::remove_var(key);
        }
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => unsafe {
                env::set_var(self.key, value);
            },
            None => unsafe {
                env::remove_var(self.key);
            },
        }
    }
}

async fn probe_meta(base_url: &str, bearer_token: &str) -> Result<MetaInfo, String> {
    let mut headers = HeaderMap::new();
    if !bearer_token.trim().is_empty() {
        let value = HeaderValue::from_str(&format!("Bearer {}", bearer_token.trim()))
            .map_err(|e| e.to_string())?;
        headers.insert(AUTHORIZATION, value);
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(format!("{}/api/meta", normalize_base_url(base_url)))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("backend probe failed: {}", response.status()));
    }

    response.json::<MetaInfo>().await.map_err(|e| e.to_string())
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
    vec![
        (
            "HONE_CONFIG_PATH",
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
    ]
}

fn start_enabled_channels(
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
        envs.extend(extra_envs);

        let child = start_logged_sidecar(app, binary, channel, envs, sidecar_log.clone())?;
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

async fn connect_backend_inner(
    app: &AppHandle,
    desktop: &DesktopState,
) -> Result<BackendStatusInfo, String> {
    let config = {
        let mut guard = desktop.inner.lock().unwrap();
        let config = load_persisted_config(app)?;
        guard.config = config.clone();
        guard.diagnostics = diagnostic_paths(app).ok();
        config
    };

    match config.mode.as_str() {
        "remote" => {
            log_desktop(app, "INFO", "connecting remote backend");
            {
                let mut guard = desktop.inner.lock().unwrap();
                stop_managed_children(&mut guard);
                guard.resolved_base_url = Some(normalize_base_url(&config.base_url));
                guard.meta = None;
                guard.last_error = None;
            }

            let resolved = normalize_base_url(&config.base_url);
            if resolved.is_empty() {
                let mut guard = desktop.inner.lock().unwrap();
                guard.last_error = Some("remote base URL 不能为空".to_string());
                log_desktop(app, "ERROR", "remote base URL is empty");
                return Ok(backend_status_snapshot(&guard));
            }

            match probe_meta(&resolved, &config.bearer_token)
                .await
                .and_then(validate_meta)
            {
                Ok(meta) => {
                    let mut guard = desktop.inner.lock().unwrap();
                    guard.meta = Some(meta);
                    guard.last_error = None;
                    log_desktop(app, "INFO", format!("remote backend connected: {resolved}"));
                    Ok(backend_status_snapshot(&guard))
                }
                Err(error) => {
                    let mut guard = desktop.inner.lock().unwrap();
                    guard.meta = None;
                    guard.last_error = Some(error.clone());
                    log_desktop(
                        app,
                        "ERROR",
                        format!("remote backend probe failed: {error}"),
                    );
                    Ok(backend_status_snapshot(&guard))
                }
            }
        }
        _ => {
            // ── bundled 模式：在主进程内启动 Axum HTTP 服务 ─────────────────
            //
            // 开发场景优化：若 HONE_WEB_PORT（默认 8077）已有后端在运行，
            // 则直接切换为 remote 模式连接到该后端，避免重复启动嵌入式进程、
            // 产生数据目录冲突以及日志/会话与 Web UI 不一致的问题。
            {
                let dev_port: u16 = std::env::var("HONE_WEB_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8077);
                let candidate = format!("http://127.0.0.1:{dev_port}");
                if let Ok(meta) = probe_meta(&candidate, "").await.and_then(validate_meta) {
                    let mut guard = desktop.inner.lock().unwrap();
                    stop_managed_children(&mut guard);
                    guard.resolved_base_url = Some(candidate.clone());
                    guard.meta = Some(meta);
                    guard.last_error = None;
                    guard.diagnostics = diagnostic_paths(app).ok();
                    log_desktop(
                        app,
                        "INFO",
                        format!("existing backend detected at {candidate}, using remote mode"),
                    );
                    return Ok(backend_status_snapshot(&guard));
                }
            }

            let runtime = ensure_runtime_paths(app)?;
            let diagnostics = diagnostic_paths(app)?;

            // 先停掉旧任务
            {
                let mut guard = desktop.inner.lock().unwrap();
                stop_managed_children(&mut guard);
            }

            log_desktop(
                app,
                "INFO",
                format!(
                    "starting embedded web server data_dir={}",
                    runtime.data_dir.display()
                ),
            );

            // 启动 Axum 服务（port=0，OS 分配可用端口）
            let config_path_str = runtime.config_path.to_string_lossy().to_string();
            let data_dir = runtime.data_dir.clone();
            let skills_dir = runtime.skills_dir.clone();
            let runtime_config =
                HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;

            let _hone_web_port_guard = ScopedEnvVar::remove("HONE_WEB_PORT");
            match hone_web_api::start_server(
                &config_path_str,
                Some(&data_dir),
                Some(&skills_dir),
                "local",
            )
            .await
            {
                Ok((_web_state, port)) => {
                    let base_url = format!("http://127.0.0.1:{port}");
                    let meta = desktop_meta_from_config(&runtime_config, "local");
                    {
                        let mut guard = desktop.inner.lock().unwrap();
                        guard.resolved_base_url = Some(base_url.clone());
                        guard.meta = Some(meta);
                        guard.last_error = None;
                        guard.diagnostics = Some(diagnostics.clone());
                    }

                    // 对于同进程内嵌服务，绑定成功本身已经足够说明 API 已就绪；
                    // 继续自 probe 反而会把短暂的启动抖动放大成误报。
                    let mut guard = desktop.inner.lock().unwrap();
                    if let Err(e) =
                        start_enabled_channels(app, &mut guard, &runtime, &diagnostics, &base_url)
                    {
                        log_desktop(app, "ERROR", format!("channel sidecar startup failed: {e}"));
                    }

                    log_desktop(
                        app,
                        "INFO",
                        format!("embedded web server ready: {base_url}"),
                    );
                    Ok(backend_status_snapshot(&guard))
                }
                Err(error) => {
                    let mut guard = desktop.inner.lock().unwrap();
                    guard.meta = None;
                    guard.last_error = Some(error.clone());
                    guard.diagnostics = Some(diagnostics);
                    log_desktop(
                        app,
                        "ERROR",
                        format!("embedded web server start failed: {error}"),
                    );
                    Ok(backend_status_snapshot(&guard))
                }
            }
        }
    }
}

pub(crate) fn get_backend_config_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendConfig, String> {
    let config = load_persisted_config(&app)?;
    state.inner.lock().unwrap().config = config.clone();
    Ok(config)
}

pub(crate) fn set_backend_config_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    config: BackendConfig,
) -> Result<(), String> {
    save_persisted_config(&app, &config)?;
    log_desktop(
        &app,
        "INFO",
        format!(
            "saved backend config mode={} base_url={}",
            config.mode, config.base_url
        ),
    );
    let mut guard = state.inner.lock().unwrap();
    if config.mode == "remote" {
        stop_managed_children(&mut guard);
        guard.resolved_base_url = Some(normalize_base_url(&config.base_url));
        guard.meta = None;
        guard.last_error = None;
    }
    guard.config = config;
    Ok(())
}

async fn connect_backend_serialized(
    app: &AppHandle,
    state: &DesktopState,
) -> Result<BackendStatusInfo, String> {
    let _guard = state.transition_lock.lock().await;
    connect_backend_inner(app, state).await
}

pub(crate) async fn connect_backend_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    connect_backend_serialized(&app, &state).await
}

pub(crate) async fn start_bundled_backend_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    {
        let mut guard = state.inner.lock().unwrap();
        guard.config.mode = "bundled".to_string();
        guard.config.base_url.clear();
        save_persisted_config(&app, &guard.config)?;
    }
    connect_backend_serialized(&app, &state).await
}

pub(crate) async fn stop_bundled_backend_impl(
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    let _guard = state.transition_lock.lock().await;
    let mut guard = state.inner.lock().unwrap();
    stop_managed_children(&mut guard);
    guard.meta = None;
    guard.last_error = Some("bundled backend stopped".to_string());
    Ok(backend_status_snapshot(&guard))
}

pub(crate) fn get_channel_settings_impl(app: AppHandle) -> Result<DesktopChannelSettings, String> {
    load_channel_settings(&app)
}

pub(crate) async fn set_channel_settings_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: DesktopChannelSettingsInput,
) -> Result<DesktopChannelSettingsUpdateResult, String> {
    let saved = save_channel_settings(&app, &settings)?;
    log_desktop(
        &app,
        "INFO",
        format!(
            "saved channel settings imessage={} feishu={} telegram={} discord={}",
            saved.imessage_enabled,
            saved.feishu_enabled,
            saved.telegram_enabled,
            saved.discord_enabled
        ),
    );

    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        let status = connect_backend_serialized(&app, &state).await?;
        let message = if status.connected {
            "已保存到运行时覆盖层，并已重启内置后端".to_string()
        } else {
            format!(
                "已保存到运行时覆盖层，但内置后端重启后未连接：{}",
                status
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "未知错误".to_string())
            )
        };
        return Ok(DesktopChannelSettingsUpdateResult {
            settings: saved,
            restarted_bundled_backend: true,
            message,
            backend_status: Some(status),
        });
    }

    Ok(DesktopChannelSettingsUpdateResult {
        settings: saved,
        restarted_bundled_backend: false,
        message: "已保存到本地运行时覆盖层。当前为远程模式，下次切回内置后端时生效".to_string(),
        backend_status: None,
    })
}

pub(crate) fn backend_status_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> BackendStatusInfo {
    let mut guard = state.inner.lock().unwrap();
    if guard.diagnostics.is_none() {
        guard.diagnostics = diagnostic_paths(&app).ok();
    }
    backend_status_snapshot(&guard)
}

// ── Agent 基础设置 commands ─────────────────────────────────────────────────

pub(crate) fn get_agent_settings_impl(app: AppHandle) -> Result<AgentSettings, String> {
    let runtime = ensure_runtime_paths(&app)?;
    let config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    Ok(AgentSettings {
        runner: config.agent.runner,
        codex_model: config.agent.codex_model,
        openai_url: config.agent.opencode.api_base_url,
        openai_model: config.agent.opencode.model,
        openai_sub_model: config.llm.openrouter.sub_model,
        openai_api_key: config.agent.opencode.api_key,
    })
}

pub(crate) async fn set_agent_settings_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: AgentSettings,
) -> Result<(), String> {
    let runtime = ensure_runtime_paths(&app)?;
    save_runtime_config_overlay(&runtime.config_path, |config| {
        config.agent.runner = settings.runner.clone();
        config.agent.codex_model = settings.codex_model.clone();
        config.agent.opencode.api_base_url = settings.openai_url.clone();
        config.agent.opencode.model = settings.openai_model.clone();
        config.llm.openrouter.sub_model = settings.openai_sub_model.clone();
        config.agent.opencode.api_key = settings.openai_api_key.clone();
    })?;
    log_desktop(
        &app,
        "INFO",
        format!(
            "saved agent settings runner={} codex_model={} openai_url={}",
            settings.runner, settings.codex_model, settings.openai_url
        ),
    );
    // 内置后端模式下重启以立即生效
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        let _ = connect_backend_serialized(&app, &state).await;
    }
    Ok(())
}

/// 测试 OpenAI 协议渠道连通性：发送一个最小 chat/completions 请求，验证 URL + API Key + 模型是否有效。
pub(crate) async fn test_openai_channel_impl(
    url: String,
    model: String,
    api_key: String,
) -> Result<CliCheckResult, String> {
    use reqwest::Client;
    use serde_json::json;

    let base = url.trim_end_matches('/');
    let endpoint = format!("{}/chat/completions", base);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1
    });

    let resp = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let status = r.status();
            if status.is_success() {
                Ok(CliCheckResult {
                    ok: true,
                    message: format!("连通成功（HTTP {}）", status.as_u16()),
                })
            } else {
                let text = r.text().await.unwrap_or_default();
                let preview: String = text.chars().take(120).collect();
                Ok(CliCheckResult {
                    ok: false,
                    message: format!("HTTP {} — {}", status.as_u16(), preview),
                })
            }
        }
        Err(e) => Ok(CliCheckResult {
            ok: false,
            message: format!("请求失败：{}", e),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn run_with_transition_lock(
        state: Arc<DesktopState>,
        concurrent: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    ) {
        let _guard = state.transition_lock.lock().await;
        let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
        peak.fetch_max(current, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(25)).await;
        concurrent.fetch_sub(1, Ordering::SeqCst);
    }

    #[tokio::test]
    async fn backend_transition_lock_serializes_concurrent_calls() {
        let state = Arc::new(DesktopState::default());
        let concurrent = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let mut tasks = Vec::new();
        for _ in 0..3 {
            tasks.push(tokio::spawn(run_with_transition_lock(
                state.clone(),
                concurrent.clone(),
                peak.clone(),
            )));
        }

        for task in tasks {
            task.await.expect("task should join");
        }

        assert_eq!(
            peak.load(Ordering::SeqCst),
            1,
            "backend transition should never run concurrently"
        );
    }
}

/// 检测本地 CLI/ACP runner 是否可用（运行 --version）。
/// 仅检查二进制是否存在且可执行，不发送真实请求，通常在 1～2s 内完成。
pub(crate) async fn check_agent_cli_impl(runner: String) -> Result<CliCheckResult, String> {
    let binary = match runner.as_str() {
        "gemini_cli" | "gemini_acp" => "gemini",
        "codex_cli" => "codex",
        "codex_acp" => "codex-acp",
        other => return Err(format!("不支持的 runner: {other}")),
    };

    let mut command = tokio::process::Command::new(binary);
    if runner == "codex_acp" {
        command.arg("--help");
    } else {
        command.arg("--version");
    }
    let result = tokio::time::timeout(std::time::Duration::from_secs(8), command.output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let version_hint = if !stdout.is_empty() {
                stdout
            } else if !stderr.is_empty() {
                stderr
            } else {
                "(无版本输出)".to_string()
            };
            Ok(CliCheckResult {
                ok: true,
                message: format!("{binary} 已就绪  {version_hint}"),
            })
        }
        Ok(Err(e)) => Ok(CliCheckResult {
            ok: false,
            message: format!("找不到 {binary} 命令：{e}"),
        }),
        Err(_) => Ok(CliCheckResult {
            ok: false,
            message: format!("{binary} 检测超时（8 秒）"),
        }),
    }
}

/// 读取运行时覆盖层中的 OpenRouter API Key 设置（多 Key）
pub(crate) fn get_openrouter_settings_impl(app: AppHandle) -> Result<OpenRouterSettings, String> {
    let runtime = ensure_runtime_paths(&app)?;
    let config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    // 合并 api_key（旧格式）和 api_keys（新格式）
    let pool = config.llm.openrouter.effective_key_pool();
    Ok(OpenRouterSettings {
        api_keys: pool.keys().to_vec(),
    })
}

/// 保存 OpenRouter API Keys 到运行时覆盖层，并重启内置后端立即生效
pub(crate) async fn set_openrouter_settings_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: OpenRouterSettings,
) -> Result<(), String> {
    let runtime = ensure_runtime_paths(&app)?;
    let valid_keys: Vec<String> = settings
        .api_keys
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .collect();
    save_runtime_config_overlay(&runtime.config_path, |config| {
        // 过滤空 key，写入 api_keys，清空旧的单 api_key 字段（避免重复）
        config.llm.openrouter.api_keys = valid_keys.clone();
        config.llm.openrouter.api_key = String::new(); // 清空旧字段，由 api_keys 主导
    })?;
    log_desktop(
        &app,
        "INFO",
        format!("saved openrouter settings keys_count={}", valid_keys.len()),
    );
    // 内置后端模式下重启以立即生效
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        let _ = connect_backend_serialized(&app, &state).await;
    }
    Ok(())
}

/// 读取运行时覆盖层中的 FMP API Key 设置（多 Key）
pub(crate) fn get_fmp_settings_impl(app: AppHandle) -> Result<FmpSettings, String> {
    let runtime = ensure_runtime_paths(&app)?;
    let config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    let pool = config.fmp.effective_key_pool();
    Ok(FmpSettings {
        api_keys: pool.keys().to_vec(),
    })
}

/// 保存 FMP API Keys 到运行时覆盖层，并重启内置后端立即生效
pub(crate) async fn set_fmp_settings_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: FmpSettings,
) -> Result<(), String> {
    let runtime = ensure_runtime_paths(&app)?;
    let valid_keys: Vec<String> = settings
        .api_keys
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .collect();
    save_runtime_config_overlay(&runtime.config_path, |config| {
        config.fmp.api_keys = valid_keys.clone();
        config.fmp.api_key = String::new(); // 清空旧字段
    })?;
    log_desktop(
        &app,
        "INFO",
        format!("saved fmp settings keys_count={}", valid_keys.len()),
    );
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        let _ = connect_backend_serialized(&app, &state).await;
    }
    Ok(())
}

/// 读取运行时覆盖层中的 Tavily API Key 设置（多 Key）
pub(crate) fn get_tavily_settings_impl(app: AppHandle) -> Result<TavilySettings, String> {
    let runtime = ensure_runtime_paths(&app)?;
    let config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    // 过滤空 key
    let valid_keys: Vec<String> = config
        .search
        .api_keys
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .collect();
    Ok(TavilySettings {
        api_keys: valid_keys,
    })
}

/// 保存 Tavily API Keys 到运行时覆盖层，并重启内置后端立即生效
pub(crate) async fn set_tavily_settings_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: TavilySettings,
) -> Result<(), String> {
    let runtime = ensure_runtime_paths(&app)?;
    let valid_keys: Vec<String> = settings
        .api_keys
        .into_iter()
        .filter(|k| !k.trim().is_empty())
        .collect();
    save_runtime_config_overlay(&runtime.config_path, |config| {
        config.search.api_keys = valid_keys.clone();
    })?;
    log_desktop(
        &app,
        "INFO",
        format!("saved tavily settings keys_count={}", valid_keys.len()),
    );
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        let _ = connect_backend_serialized(&app, &state).await;
    }
    Ok(())
}

pub(crate) fn run_desktop_app() {
    crate::commands::run_desktop_app();
}
