#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Mutex;
use std::time::Duration;

use hone_core::HoneConfig;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use serde::{Deserialize, Serialize};
use tauri::async_runtime;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::{
    ShellExt,
    process::{CommandChild, CommandEvent},
};

mod processes;
mod runtime_env;
mod settings;

use self::{processes::*, runtime_env::*, settings::*};

#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const API_VERSION: &str = "desktop-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopChannelSettings {
    config_path: String,
    imessage_enabled: bool,
    feishu_enabled: bool,
    #[serde(default)]
    feishu_app_id: String,
    #[serde(default)]
    feishu_app_secret: String,
    telegram_enabled: bool,
    #[serde(default)]
    telegram_bot_token: String,
    discord_enabled: bool,
    #[serde(default)]
    discord_bot_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopChannelSettingsInput {
    imessage_enabled: bool,
    feishu_enabled: bool,
    #[serde(default)]
    feishu_app_id: String,
    #[serde(default)]
    feishu_app_secret: String,
    telegram_enabled: bool,
    #[serde(default)]
    telegram_bot_token: String,
    discord_enabled: bool,
    #[serde(default)]
    discord_bot_token: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopChannelSettingsUpdateResult {
    settings: DesktopChannelSettings,
    restarted_bundled_backend: bool,
    message: String,
    backend_status: Option<BackendStatusInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSettingsUpdateResult {
    settings: AgentSettings,
    restarted_bundled_backend: bool,
    message: String,
    backend_status: Option<BackendStatusInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelProcessCleanupEntry {
    channel: String,
    kept_pid: Option<u32>,
    removed_pids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelProcessCleanupResult {
    entries: Vec<ChannelProcessCleanupEntry>,
    message: String,
}

/// Agent 基础设置（写入运行时覆盖层）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultiAgentSearchSettings {
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    max_iterations: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultiAgentAnswerSettings {
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    variant: String,
    #[serde(default)]
    max_tool_calls: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultiAgentSettings {
    search: MultiAgentSearchSettings,
    answer: MultiAgentAnswerSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuxiliarySettings {
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentSettings {
    /// function_calling | gemini_cli | gemini_acp | codex_cli | codex_acp | opencode_acp | multi-agent
    runner: String,
    /// codex_cli 专用，其他 provider 忽略
    codex_model: String,
    /// OpenAI 协议渠道 Base URL（agent.opencode.api_base_url）
    #[serde(default)]
    openai_url: String,
    /// OpenAI 协议渠道模型名（agent.opencode.model）
    #[serde(default)]
    openai_model: String,
    /// OpenAI 协议渠道 API Key（agent.opencode.api_key）
    #[serde(default)]
    openai_api_key: String,
    #[serde(default)]
    auxiliary: Option<AuxiliarySettings>,
    #[serde(default)]
    multi_agent: Option<MultiAgentSettings>,
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
    desktop_lock: Mutex<Option<hone_core::ProcessLockGuard>>,
    inner: Mutex<DesktopBackendManager>,
    config_write_lock: tokio::sync::Mutex<()>,
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
    /// bundled 模式下的 hone-console-page 生命周期锁
    bundled_web_lock: Option<hone_core::ProcessLockGuard>,
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
    effective_config_path: PathBuf,
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

fn bundled_runtime_ready(manager: &DesktopBackendManager) -> bool {
    manager.last_error.is_none()
        && manager.bundled_web_lock.is_some()
        && manager.meta.is_some()
        && manager.resolved_base_url.is_some()
}

fn mark_bundled_runtime_dirty(manager: &mut DesktopBackendManager, reason: &str) {
    manager.meta = None;
    manager.last_error = Some(reason.to_string());
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

pub(crate) fn show_startup_error_dialog(message: &str) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("Hone Startup Blocked")
        .set_description(message)
        .set_buttons(MessageButtons::Ok)
        .show();
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
        .map_err(|e| format!("client build failed: {e}; debug={e:?}"))?;

    let response = client
        .get(format!("{}/api/meta", normalize_base_url(base_url)))
        .send()
        .await
        .map_err(|e| format!("request send failed: {e}; debug={e:?}"))?;

    if !response.status().is_success() {
        return Err(format!("backend probe failed: {}", response.status()));
    }

    response
        .json::<MetaInfo>()
        .await
        .map_err(|e| format!("response json failed: {e}; debug={e:?}"))
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
                    let should_log = guard.last_error.as_deref() != Some(error.as_str());
                    guard.last_error = Some(error.clone());
                    if should_log {
                        log_desktop(
                            app,
                            "ERROR",
                            format!("remote backend probe failed url={resolved} error={error}"),
                        );
                    }
                    Ok(backend_status_snapshot(&guard))
                }
            }
        }
        _ => {
            let runtime = ensure_runtime_paths(app)?;
            let diagnostics = diagnostic_paths(app)?;
            let runtime_config =
                HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;

            {
                let guard = desktop.inner.lock().unwrap();
                if bundled_runtime_ready(&guard) {
                    return Ok(backend_status_snapshot(&guard));
                }
            }

            if let Err(message) = preflight_bundled_runtime_locks(app) {
                log_desktop(app, "ERROR", &message);
                show_startup_error_dialog(&message);
                let mut guard = desktop.inner.lock().unwrap();
                stop_managed_children(&mut guard);
                guard.meta = None;
                guard.last_error = Some(message);
                guard.diagnostics = Some(diagnostics);
                return Ok(backend_status_snapshot(&guard));
            }
            log_desktop(app, "INFO", "bundled runtime preflight locks passed");

            // 先停掉旧任务
            {
                let mut guard = desktop.inner.lock().unwrap();
                stop_managed_children(&mut guard);
            }
            log_desktop(
                app,
                "INFO",
                "bundled runtime previous managed children stopped",
            );

            // 启动 Axum 服务（port=0，OS 分配可用端口）
            let config_path_str = runtime.config_path.to_string_lossy().to_string();
            let data_dir = runtime.data_dir.clone();
            let skills_dir = runtime.skills_dir.clone();
            let web_lock = match hone_core::acquire_process_lock(
                &runtime.runtime_dir,
                hone_core::PROCESS_LOCK_CONSOLE_PAGE,
            )
            .map_err(|error| {
                hone_core::format_lock_failure_message(
                    hone_core::PROCESS_LOCK_CONSOLE_PAGE,
                    &hone_core::process_lock_path(
                        &runtime.runtime_dir,
                        hone_core::PROCESS_LOCK_CONSOLE_PAGE,
                    ),
                    &error,
                    "Hone bundled runtime",
                )
            }) {
                Ok(lock) => lock,
                Err(message) => {
                    log_desktop(app, "ERROR", &message);
                    let mut guard = desktop.inner.lock().unwrap();
                    guard.meta = None;
                    guard.last_error = Some(message);
                    guard.diagnostics = Some(diagnostics);
                    return Ok(backend_status_snapshot(&guard));
                }
            };
            log_desktop(app, "INFO", "bundled runtime console lock acquired");

            log_desktop(
                app,
                "INFO",
                format!(
                    "starting embedded web server data_dir={}",
                    runtime.data_dir.display()
                ),
            );

            let _hone_web_port_guard = ScopedEnvVar::remove("HONE_WEB_PORT");
            log_desktop(
                app,
                "INFO",
                "bundled runtime calling hone_web_api::start_server",
            );
            match hone_web_api::start_server(
                &config_path_str,
                Some(&data_dir),
                Some(&skills_dir),
                "local",
            )
            .await
            {
                Ok((_web_state, port)) => {
                    log_desktop(
                        app,
                        "INFO",
                        format!("hone_web_api::start_server returned port={port}"),
                    );
                    let base_url = format!("http://127.0.0.1:{port}");
                    let meta = desktop_meta_from_config(&runtime_config, "local");
                    {
                        let mut guard = desktop.inner.lock().unwrap();
                        guard.resolved_base_url = Some(base_url.clone());
                        guard.meta = Some(meta);
                        guard.last_error = None;
                        guard.diagnostics = Some(diagnostics.clone());
                        guard.bundled_web_lock = Some(web_lock);
                    }

                    // 对于同进程内嵌服务，绑定成功本身已经足够说明 API 已就绪；
                    // 继续自 probe 反而会把短暂的启动抖动放大成误报。
                    let mut guard = desktop.inner.lock().unwrap();
                    log_desktop(app, "INFO", "starting managed bundled channels");
                    if let Err(e) =
                        start_enabled_channels(app, &mut guard, &runtime, &diagnostics, &base_url)
                    {
                        let message =
                            format!("bundled channel sidecar startup failed, runtime aborted: {e}");
                        log_desktop(app, "ERROR", &message);
                        stop_managed_children(&mut guard);
                        guard.meta = None;
                        guard.last_error = Some(message);
                        return Ok(backend_status_snapshot(&guard));
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

async fn with_config_write_lock<T, F>(state: &DesktopState, op: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let _guard = state.config_write_lock.lock().await;
    op()
}

pub(crate) async fn connect_backend_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    match connect_backend_serialized(&app, &state).await {
        Ok(status) => Ok(status),
        Err(error) => {
            log_desktop(
                &app,
                "ERROR",
                format!("connect_backend command failed: {error}"),
            );
            Err(error)
        }
    }
}

pub(crate) fn bootstrap_backend_on_startup(app: AppHandle) {
    async_runtime::spawn(async move {
        let state = app.state::<DesktopState>();
        if let Err(error) = connect_backend_serialized(&app, &state).await {
            log_desktop(
                &app,
                "ERROR",
                format!("startup backend bootstrap failed: {error}"),
            );
        }
    });
}

pub(crate) fn prepare_desktop_startup(app: AppHandle) -> Result<(), String> {
    let runtime = ensure_runtime_paths(&app)?;
    configure_desktop_runtime_env(&app, &runtime);
    preflight_startup_locks(&app)
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
    let saved = with_config_write_lock(&state, || save_channel_settings(&app, &settings)).await?;
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
        {
            let mut guard = state.inner.lock().unwrap();
            mark_bundled_runtime_dirty(
                &mut guard,
                "channel settings updated; bundled runtime restart required",
            );
        }
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
    guard.config = load_persisted_config(&app).unwrap_or_default();
    if guard.diagnostics.is_none() {
        guard.diagnostics = diagnostic_paths(&app).ok();
    }
    backend_status_snapshot(&guard)
}

pub(crate) async fn cleanup_channel_processes_impl(
    state: State<'_, DesktopState>,
) -> Result<ChannelProcessCleanupResult, String> {
    let _guard = state.transition_lock.lock().await;
    let mut guard = state.inner.lock().unwrap();
    Ok(cleanup_duplicate_channel_processes_inner(&mut guard))
}

// ── Agent 基础设置 commands ─────────────────────────────────────────────────

pub(crate) fn get_agent_settings_impl(app: AppHandle) -> Result<AgentSettings, String> {
    let runtime = ensure_runtime_paths(&app)?;
    let config = HoneConfig::from_file(&runtime.config_path).map_err(|e| e.to_string())?;
    Ok(AgentSettings {
        runner: config.agent.runner.clone(),
        codex_model: config.agent.codex_model.clone(),
        openai_url: config.agent.opencode.api_base_url.clone(),
        openai_model: config.agent.opencode.model.clone(),
        openai_api_key: config.agent.opencode.api_key.clone(),
        auxiliary: Some(seed_auxiliary_settings(&config)),
        multi_agent: Some(seed_multi_agent_settings(&config)),
    })
}

fn build_agent_setting_updates(settings: &AgentSettings) -> Vec<(&'static str, serde_yaml::Value)> {
    let mut updates = vec![
        (
            "agent.runner",
            serde_yaml::Value::String(settings.runner.clone()),
        ),
        (
            "agent.codex_model",
            serde_yaml::Value::String(settings.codex_model.clone()),
        ),
        (
            "agent.opencode.api_base_url",
            serde_yaml::Value::String(settings.openai_url.clone()),
        ),
        (
            "agent.opencode.model",
            serde_yaml::Value::String(settings.openai_model.clone()),
        ),
        (
            "agent.opencode.api_key",
            serde_yaml::Value::String(settings.openai_api_key.clone()),
        ),
    ];
    if let Some(auxiliary) = &settings.auxiliary {
        updates.extend([
            (
                "llm.auxiliary.base_url",
                serde_yaml::Value::String(auxiliary.base_url.clone()),
            ),
            (
                "llm.auxiliary.api_key",
                serde_yaml::Value::String(auxiliary.api_key.clone()),
            ),
            (
                "llm.auxiliary.api_key_env",
                serde_yaml::Value::String("MINIMAX_API_KEY".to_string()),
            ),
            (
                "llm.auxiliary.model",
                serde_yaml::Value::String(auxiliary.model.clone()),
            ),
            (
                "llm.openrouter.sub_model",
                serde_yaml::Value::String(auxiliary.model.clone()),
            ),
        ]);
    }
    if let Some(multi_agent) = &settings.multi_agent {
        updates.extend([
            (
                "agent.multi_agent.search.base_url",
                serde_yaml::Value::String(multi_agent.search.base_url.clone()),
            ),
            (
                "agent.multi_agent.search.api_key",
                serde_yaml::Value::String(multi_agent.search.api_key.clone()),
            ),
            (
                "agent.multi_agent.search.model",
                serde_yaml::Value::String(multi_agent.search.model.clone()),
            ),
            (
                "agent.multi_agent.search.max_iterations",
                serde_yaml::Value::Number(serde_yaml::Number::from(
                    multi_agent.search.max_iterations,
                )),
            ),
            (
                "agent.multi_agent.answer.api_base_url",
                serde_yaml::Value::String(multi_agent.answer.base_url.clone()),
            ),
            (
                "agent.multi_agent.answer.api_key",
                serde_yaml::Value::String(multi_agent.answer.api_key.clone()),
            ),
            (
                "agent.multi_agent.answer.model",
                serde_yaml::Value::String(multi_agent.answer.model.clone()),
            ),
            (
                "agent.multi_agent.answer.variant",
                serde_yaml::Value::String(multi_agent.answer.variant.clone()),
            ),
            (
                "agent.multi_agent.answer.max_tool_calls",
                serde_yaml::Value::Number(serde_yaml::Number::from(
                    multi_agent.answer.max_tool_calls,
                )),
            ),
        ]);
    }
    updates
}

fn agent_settings_require_save(current: &AgentSettings, next: &AgentSettings) -> bool {
    current != next
}

fn build_agent_settings_update_result(
    settings: AgentSettings,
    backend_status: Option<BackendStatusInfo>,
) -> AgentSettingsUpdateResult {
    match backend_status {
        Some(status) => {
            let message = if status.connected {
                "已保存 Agent 设置，并已重启内置后端".to_string()
            } else {
                format!(
                    "已保存 Agent 设置，但当前 runtime 尚未生效：{}",
                    status
                        .last_error
                        .clone()
                        .unwrap_or_else(|| "内置后端重启后未连接".to_string())
                )
            };
            AgentSettingsUpdateResult {
                settings,
                restarted_bundled_backend: true,
                message,
                backend_status: Some(status),
            }
        }
        None => AgentSettingsUpdateResult {
            settings,
            restarted_bundled_backend: false,
            message: "已保存 Agent 设置。当前为远程模式，下次切回内置后端时生效".to_string(),
            backend_status: None,
        },
    }
}

pub(crate) async fn set_agent_settings_impl(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: AgentSettings,
) -> Result<AgentSettingsUpdateResult, String> {
    let current_settings = get_agent_settings_impl(app.clone())?;
    if !agent_settings_require_save(&current_settings, &settings) {
        log_desktop(
            &app,
            "INFO",
            format!(
                "agent settings unchanged; skip save/restart runner={}",
                settings.runner
            ),
        );
        return Ok(AgentSettingsUpdateResult {
            settings,
            restarted_bundled_backend: false,
            message: "Agent 设置未变化".to_string(),
            backend_status: None,
        });
    }
    let runtime = ensure_runtime_paths(&app)?;
    let updates = build_agent_setting_updates(&settings);
    with_config_write_lock(&state, || {
        apply_setting_updates(
            &runtime.config_path,
            &runtime.effective_config_path,
            updates,
        )
        .map(|_| ())
    })
    .await?;
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
        {
            let mut guard = state.inner.lock().unwrap();
            mark_bundled_runtime_dirty(
                &mut guard,
                "agent settings updated; bundled runtime restart required",
            );
        }
        let status = connect_backend_serialized(&app, &state).await?;
        return Ok(build_agent_settings_update_result(settings, Some(status)));
    }
    Ok(build_agent_settings_update_result(settings, None))
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

    async fn run_with_config_write_lock(
        state: Arc<DesktopState>,
        concurrent: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
    ) {
        let _ = with_config_write_lock(&state, || {
            let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(25));
            concurrent.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        })
        .await;
    }

    #[tokio::test]
    async fn config_write_lock_serializes_concurrent_calls() {
        let state = Arc::new(DesktopState::default());
        let concurrent = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        let mut tasks = Vec::new();
        for _ in 0..3 {
            tasks.push(tokio::spawn(run_with_config_write_lock(
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
            "config writes should never run concurrently"
        );
    }

    #[tokio::test]
    async fn config_write_lock_preserves_updates_from_concurrent_saves() {
        let state = Arc::new(DesktopState::default());
        let dir = std::env::temp_dir().join(format!(
            "hone-desktop-config-write-lock-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp test dir should exist");
        let config_path = dir.join("config.yaml");
        let effective_config_path = dir.join("effective-config.yaml");
        std::fs::write(
            &config_path,
            r#"
agent:
  runner: opencode_acp
  codex_model: ""
  opencode:
    api_base_url: "https://openrouter.ai/api/v1"
    model: "google/gemini-2.5-pro-preview"
    api_key: ""
search:
  api_keys: []
fmp:
  api_keys: []
  api_key: ""
"#,
        )
        .expect("seed config should write");

        let first_started = Arc::new(AtomicBool::new(false));
        let state_for_agent = state.clone();
        let config_for_agent = config_path.clone();
        let effective_for_agent = effective_config_path.clone();
        let first_started_for_agent = first_started.clone();
        let agent_task = tokio::spawn(async move {
            with_config_write_lock(&state_for_agent, || {
                first_started_for_agent.store(true, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(30));
                apply_setting_updates(
                    &config_for_agent,
                    &effective_for_agent,
                    vec![
                        (
                            "agent.runner",
                            serde_yaml::Value::String("multi-agent".to_string()),
                        ),
                        (
                            "agent.codex_model",
                            serde_yaml::Value::String("ignored-model".to_string()),
                        ),
                    ],
                )
                .map(|_| ())
            })
            .await
        });

        while !first_started.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }

        let state_for_fmp = state.clone();
        let config_for_fmp = config_path.clone();
        let effective_for_fmp = effective_config_path.clone();
        let fmp_task = tokio::spawn(async move {
            with_config_write_lock(&state_for_fmp, || {
                apply_setting_updates(
                    &config_for_fmp,
                    &effective_for_fmp,
                    vec![
                        (
                            "fmp.api_keys",
                            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String(
                                "fmp-key-1".to_string(),
                            )]),
                        ),
                        ("fmp.api_key", serde_yaml::Value::String(String::new())),
                    ],
                )
                .map(|_| ())
            })
            .await
        });

        agent_task
            .await
            .expect("agent task should join")
            .expect("agent save should succeed");
        fmp_task
            .await
            .expect("fmp task should join")
            .expect("fmp save should succeed");

        let config = HoneConfig::from_file(&config_path).expect("final config should load");
        assert_eq!(config.agent.runner, "multi-agent");
        assert_eq!(config.agent.codex_model, "ignored-model");
        assert_eq!(config.fmp.api_keys, vec!["fmp-key-1".to_string()]);
    }

    #[test]
    fn seed_multi_agent_settings_prefers_existing_multi_agent_values() {
        let mut config = HoneConfig::default();
        config.agent.opencode.api_base_url = "https://openrouter.ai/api/v1".to_string();
        config.agent.opencode.api_key = "sk-or-old".to_string();
        config.agent.opencode.model = "google/gemini-3.1-pro-preview".to_string();
        config.agent.opencode.variant = "high".to_string();
        config.agent.multi_agent.search.base_url = "https://api.minimaxi.com/v1".to_string();
        config.agent.multi_agent.search.api_key = "sk-cp-new".to_string();
        config.agent.multi_agent.search.model = "MiniMax-M2.7-highspeed".to_string();
        config.agent.multi_agent.search.max_iterations = 9;
        config.agent.multi_agent.answer.api_base_url = "https://custom.example/v1".to_string();
        config.agent.multi_agent.answer.api_key = "sk-answer".to_string();
        config.agent.multi_agent.answer.model = "google/gemini-3.1-pro-preview".to_string();
        config.agent.multi_agent.answer.variant = "xhigh".to_string();
        config.agent.multi_agent.answer.max_tool_calls = 2;

        let seeded = seed_multi_agent_settings(&config);
        assert_eq!(seeded.search.api_key, "sk-cp-new");
        assert_eq!(seeded.search.max_iterations, 9);
        assert_eq!(seeded.answer.base_url, "https://custom.example/v1");
        assert_eq!(seeded.answer.api_key, "sk-answer");
        assert_eq!(seeded.answer.variant, "xhigh");
        assert_eq!(seeded.answer.max_tool_calls, 2);
    }

    #[test]
    fn seed_multi_agent_settings_falls_back_to_opencode_answer() {
        let mut config = HoneConfig::default();
        config.agent.opencode.api_base_url = "https://openrouter.ai/api/v1".to_string();
        config.agent.opencode.api_key = "sk-or-fallback".to_string();
        config.agent.opencode.model = "google/gemini-3.1-pro-preview".to_string();
        config.agent.opencode.variant = "high".to_string();
        config.agent.multi_agent.answer = hone_core::config::MultiAgentAnswerConfig::default();

        let seeded = seed_multi_agent_settings(&config);
        assert_eq!(seeded.answer.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(seeded.answer.api_key, "sk-or-fallback");
        assert_eq!(seeded.answer.model, "google/gemini-3.1-pro-preview");
        assert_eq!(seeded.answer.variant, "high");
        assert_eq!(seeded.answer.max_tool_calls, 1);
    }

    #[test]
    fn seed_auxiliary_settings_prefers_explicit_auxiliary_config() {
        let mut config = HoneConfig::default();
        config.llm.auxiliary.base_url = "https://api.minimaxi.com/v1".to_string();
        config.llm.auxiliary.api_key = "sk-cp-aux".to_string();
        config.llm.auxiliary.model = "MiniMax-M2.7-highspeed".to_string();
        config.agent.multi_agent.search.api_key = "sk-cp-search".to_string();

        let seeded = seed_auxiliary_settings(&config);
        assert_eq!(seeded.base_url, "https://api.minimaxi.com/v1");
        assert_eq!(seeded.api_key, "sk-cp-aux");
        assert_eq!(seeded.model, "MiniMax-M2.7-highspeed");
    }

    #[test]
    fn seed_auxiliary_settings_falls_back_to_multi_agent_search() {
        let mut config = HoneConfig::default();
        config.agent.multi_agent.search.base_url = "https://api.minimaxi.com/v1".to_string();
        config.agent.multi_agent.search.api_key = "sk-cp-search".to_string();
        config.agent.multi_agent.search.model = "MiniMax-M2.7".to_string();

        let seeded = seed_auxiliary_settings(&config);
        assert_eq!(seeded.base_url, "https://api.minimaxi.com/v1");
        assert_eq!(seeded.api_key, "sk-cp-search");
        assert_eq!(seeded.model, "MiniMax-M2.7");
    }

    #[test]
    fn build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated() {
        let settings = AgentSettings {
            runner: "multi-agent".to_string(),
            codex_model: String::new(),
            openai_url: "https://opencode.example/v1".to_string(),
            openai_model: "openai/gpt-5.4".to_string(),
            openai_api_key: "sk-opencode".to_string(),
            auxiliary: None,
            multi_agent: Some(MultiAgentSettings {
                search: MultiAgentSearchSettings {
                    base_url: "https://search.example/v1".to_string(),
                    api_key: "sk-search".to_string(),
                    model: "search-model".to_string(),
                    max_iterations: 6,
                },
                answer: MultiAgentAnswerSettings {
                    base_url: "https://answer.example/v1".to_string(),
                    api_key: "sk-answer".to_string(),
                    model: "answer-model".to_string(),
                    variant: "high".to_string(),
                    max_tool_calls: 2,
                },
            }),
        };

        let updates = build_agent_setting_updates(&settings);
        let update_map = updates
            .into_iter()
            .map(|(path, value)| (path, value))
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(
            update_map
                .get("agent.opencode.api_base_url")
                .and_then(serde_yaml::Value::as_str),
            Some("https://opencode.example/v1")
        );
        assert_eq!(
            update_map
                .get("agent.opencode.model")
                .and_then(serde_yaml::Value::as_str),
            Some("openai/gpt-5.4")
        );
        assert_eq!(
            update_map
                .get("agent.opencode.api_key")
                .and_then(serde_yaml::Value::as_str),
            Some("sk-opencode")
        );
        assert!(!update_map.contains_key("agent.opencode.variant"));
        assert_eq!(
            update_map
                .get("agent.multi_agent.answer.api_base_url")
                .and_then(serde_yaml::Value::as_str),
            Some("https://answer.example/v1")
        );
        assert_eq!(
            update_map
                .get("agent.multi_agent.answer.api_key")
                .and_then(serde_yaml::Value::as_str),
            Some("sk-answer")
        );
        assert_eq!(
            update_map
                .get("agent.multi_agent.answer.model")
                .and_then(serde_yaml::Value::as_str),
            Some("answer-model")
        );
        assert_eq!(
            update_map
                .get("agent.multi_agent.answer.variant")
                .and_then(serde_yaml::Value::as_str),
            Some("high")
        );
    }

    #[test]
    fn agent_settings_require_save_skips_identical_runner_payloads() {
        let settings = AgentSettings {
            runner: "opencode_acp".to_string(),
            codex_model: String::new(),
            openai_url: "https://openrouter.ai/api/v1".to_string(),
            openai_model: "google/gemini-2.5-pro-preview".to_string(),
            openai_api_key: "sk-or".to_string(),
            auxiliary: Some(AuxiliarySettings {
                base_url: "https://api.minimaxi.com/v1".to_string(),
                api_key: "sk-cp-aux".to_string(),
                model: "MiniMax-M2.7-highspeed".to_string(),
            }),
            multi_agent: Some(MultiAgentSettings {
                search: MultiAgentSearchSettings {
                    base_url: "https://api.minimaxi.com/v1".to_string(),
                    api_key: "sk-cp-search".to_string(),
                    model: "MiniMax-M2.7-highspeed".to_string(),
                    max_iterations: 8,
                },
                answer: MultiAgentAnswerSettings {
                    base_url: "https://openrouter.ai/api/v1".to_string(),
                    api_key: "sk-or-answer".to_string(),
                    model: "google/gemini-2.5-pro-preview".to_string(),
                    variant: "high".to_string(),
                    max_tool_calls: 1,
                },
            }),
        };

        assert!(
            !agent_settings_require_save(&settings, &settings.clone()),
            "unchanged settings should not trigger a fresh save/restart"
        );

        let changed = AgentSettings {
            runner: "multi-agent".to_string(),
            ..settings.clone()
        };
        assert!(
            agent_settings_require_save(&settings, &changed),
            "changing the runner should still trigger save/restart"
        );
    }

    #[test]
    fn bundled_agent_settings_update_result_surfaces_runtime_not_applied() {
        let settings = AgentSettings {
            runner: "multi-agent".to_string(),
            codex_model: String::new(),
            openai_url: "https://openrouter.ai/api/v1".to_string(),
            openai_model: "google/gemini-2.5-pro-preview".to_string(),
            openai_api_key: String::new(),
            auxiliary: None,
            multi_agent: None,
        };
        let status = BackendStatusInfo {
            config: BackendConfig {
                mode: "bundled".to_string(),
                base_url: String::new(),
                bearer_token: String::new(),
            },
            resolved_base_url: Some("http://127.0.0.1:8077".to_string()),
            connected: false,
            last_error: Some("bundle restart failed".to_string()),
            meta: None,
            diagnostics: DiagnosticPaths {
                config_dir: "/tmp/config".to_string(),
                data_dir: "/tmp/data".to_string(),
                logs_dir: "/tmp/logs".to_string(),
                desktop_log: "/tmp/logs/desktop.log".to_string(),
                sidecar_log: "/tmp/logs/sidecar.log".to_string(),
            },
        };

        let result = build_agent_settings_update_result(settings.clone(), Some(status));

        assert!(result.restarted_bundled_backend);
        assert_eq!(result.settings.runner, "multi-agent");
        assert!(
            result.message.contains("当前 runtime 尚未生效"),
            "should explicitly surface that runtime did not apply the new runner"
        );
        assert_eq!(
            result
                .backend_status
                .as_ref()
                .and_then(|status| status.last_error.as_deref()),
            Some("bundle restart failed")
        );
    }

    #[test]
    fn desktop_channel_settings_input_accepts_secret_fields_from_camel_case_payload() {
        let input: DesktopChannelSettingsInput = serde_json::from_value(serde_json::json!({
            "imessageEnabled": true,
            "feishuEnabled": true,
            "feishuAppId": "cli_test",
            "feishuAppSecret": "secret-value",
            "telegramEnabled": true,
            "telegramBotToken": "tg-token",
            "discordEnabled": true,
            "discordBotToken": "discord-token"
        }))
        .expect("desktop channel payload should deserialize");

        assert!(input.imessage_enabled);
        assert!(input.feishu_enabled);
        assert_eq!(input.feishu_app_id, "cli_test");
        assert_eq!(input.feishu_app_secret, "secret-value");
        assert!(input.telegram_enabled);
        assert_eq!(input.telegram_bot_token, "tg-token");
        assert!(input.discord_enabled);
        assert_eq!(input.discord_bot_token, "discord-token");
    }

    #[test]
    fn desktop_channel_settings_serializes_secret_fields_to_camel_case_payload() {
        let settings = DesktopChannelSettings {
            config_path: "/tmp/config.yaml".to_string(),
            imessage_enabled: false,
            feishu_enabled: true,
            feishu_app_id: "cli_test".to_string(),
            feishu_app_secret: "secret-value".to_string(),
            telegram_enabled: true,
            telegram_bot_token: "tg-token".to_string(),
            discord_enabled: true,
            discord_bot_token: "discord-token".to_string(),
        };

        let json =
            serde_json::to_value(&settings).expect("desktop channel settings should serialize");
        assert_eq!(json["feishuAppId"], "cli_test");
        assert_eq!(json["feishuAppSecret"], "secret-value");
        assert_eq!(json["telegramBotToken"], "tg-token");
        assert_eq!(json["discordBotToken"], "discord-token");
    }

    #[test]
    fn mark_bundled_runtime_dirty_clears_cached_meta_and_sets_restart_reason() {
        let mut manager = DesktopBackendManager::default();
        manager.resolved_base_url = Some("http://127.0.0.1:3000".to_string());
        manager.meta = Some(MetaInfo {
            name: "Hone".to_string(),
            version: "0.1.13".to_string(),
            channel: "imessage".to_string(),
            supports_imessage: true,
            api_version: API_VERSION.to_string(),
            capabilities: vec!["chat".to_string()],
            deployment_mode: "local".to_string(),
        });
        manager.last_error = None;

        mark_bundled_runtime_dirty(&mut manager, "channel settings updated");

        assert!(manager.meta.is_none());
        assert_eq!(
            manager.last_error.as_deref(),
            Some("channel settings updated")
        );
        assert_eq!(
            manager.resolved_base_url.as_deref(),
            Some("http://127.0.0.1:3000")
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
        "opencode_acp" | "multi-agent" => "opencode",
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
    with_config_write_lock(&state, || {
        apply_setting_updates(
            &runtime.config_path,
            &runtime.effective_config_path,
            vec![
                (
                    "llm.openrouter.api_keys",
                    serde_yaml::Value::Sequence(
                        valid_keys
                            .iter()
                            .cloned()
                            .map(serde_yaml::Value::String)
                            .collect(),
                    ),
                ),
                (
                    "llm.openrouter.api_key",
                    serde_yaml::Value::String(String::new()),
                ),
            ],
        )
        .map(|_| ())
    })
    .await?;
    log_desktop(
        &app,
        "INFO",
        format!("saved openrouter settings keys_count={}", valid_keys.len()),
    );
    // 内置后端模式下重启以立即生效
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        {
            let mut guard = state.inner.lock().unwrap();
            mark_bundled_runtime_dirty(
                &mut guard,
                "openrouter settings updated; bundled runtime restart required",
            );
        }
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
    with_config_write_lock(&state, || {
        apply_setting_updates(
            &runtime.config_path,
            &runtime.effective_config_path,
            vec![
                (
                    "fmp.api_keys",
                    serde_yaml::Value::Sequence(
                        valid_keys
                            .iter()
                            .cloned()
                            .map(serde_yaml::Value::String)
                            .collect(),
                    ),
                ),
                ("fmp.api_key", serde_yaml::Value::String(String::new())),
            ],
        )
        .map(|_| ())
    })
    .await?;
    log_desktop(
        &app,
        "INFO",
        format!("saved fmp settings keys_count={}", valid_keys.len()),
    );
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        {
            let mut guard = state.inner.lock().unwrap();
            mark_bundled_runtime_dirty(
                &mut guard,
                "fmp settings updated; bundled runtime restart required",
            );
        }
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
    with_config_write_lock(&state, || {
        apply_setting_updates(
            &runtime.config_path,
            &runtime.effective_config_path,
            vec![(
                "search.api_keys",
                serde_yaml::Value::Sequence(
                    valid_keys
                        .iter()
                        .cloned()
                        .map(serde_yaml::Value::String)
                        .collect(),
                ),
            )],
        )
        .map(|_| ())
    })
    .await?;
    log_desktop(
        &app,
        "INFO",
        format!("saved tavily settings keys_count={}", valid_keys.len()),
    );
    let backend_config = load_persisted_config(&app).unwrap_or_default();
    if backend_config.mode == "bundled" {
        {
            let mut guard = state.inner.lock().unwrap();
            mark_bundled_runtime_dirty(
                &mut guard,
                "tavily settings updated; bundled runtime restart required",
            );
        }
        let _ = connect_backend_serialized(&app, &state).await;
    }
    Ok(())
}

pub(crate) fn run_desktop_app() {
    crate::commands::run_desktop_app();
}
