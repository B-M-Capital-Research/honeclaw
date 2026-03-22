//! hone-web-api — Hone 控制台 HTTP 服务库
//!
//! 将原 `hone-console-page` 二进制的服务逻辑提取为库，
//! 供 `hone-desktop` 在 Tauri 主进程内直接嵌入启动，无需子进程 sidecar。

pub mod logging;
pub mod routes;
pub mod runtime;
pub mod state;
pub mod types;

pub use logging::{LogBuffer, LogCaptureLayer, LogEntry};
pub use routes::build_app;
pub use state::{AppState, AuthState, PushEvent};

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use hone_core::config::HoneConfig;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::routes::events::handle_scheduler_events;
use crate::state::AppState as InnerAppState;

const PUSH_CHANNEL_CAPACITY: usize = 64;

// ── 全局唯一 LogBuffer ──────────────────────────────────────────────────────
// tracing 订阅者在进程内只能设置一次，因此 LogBuffer 也必须全局唯一，
// 否则重连后 AppState 持有新 buffer 但订阅者仍向旧 buffer 写入，导致日志消失。
static GLOBAL_LOG_BUFFER: OnceLock<LogBuffer> = OnceLock::new();
static FILE_LOG_STARTED: AtomicBool = AtomicBool::new(false);

fn global_log_buffer() -> &'static LogBuffer {
    GLOBAL_LOG_BUFFER.get_or_init(LogBuffer::new)
}

/// 初始化全局 tracing 订阅者（含内存捕获层）。
/// 调用一次即可；重复调用安全（会静默失败）。
pub fn init_logging(log_buffer: &LogBuffer, log_level: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_env("HONE_LOG_LEVEL")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(false);

    let capture_layer = LogCaptureLayer::new(log_buffer.clone());

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(capture_layer);

    // try_init 失败时安静忽略（已初始化）
    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// 在当前进程内启动 Axum HTTP 服务（含调度器 & UDP 日志接收）。
///
/// 参数：
/// - `config_path`：`config.yaml` 路径  
/// - `data_dir`：数据根目录（覆盖 config 中的相对路径），`None` 时使用 config 原始值  
/// - `skills_dir`：技能目录，`None` 时使用 config 原始值  
/// - `deployment_mode`：`"local"` / `"cloud"` 等，写入 `/api/meta` 响应  
///
/// 返回 `(Arc<AppState>, actual_port)`。服务在后台 Tokio task 中运行，进程退出时自然结束。
pub async fn start_server(
    config_path: &str,
    data_dir: Option<&Path>,
    skills_dir: Option<&Path>,
    deployment_mode: &str,
) -> Result<(Arc<InnerAppState>, u16), String> {
    let mut config =
        HoneConfig::from_file(config_path).map_err(|e| format!("配置加载失败: {e}"))?;

    // 覆盖存储路径（当 Tauri 运行时路径与 config.yaml 中相对路径不同时）
    if let Some(data) = data_dir {
        let base = data.to_string_lossy();
        config.storage.sessions_dir = data.join("sessions").to_string_lossy().to_string();
        config.storage.portfolio_dir = data.join("portfolio").to_string_lossy().to_string();
        config.storage.cron_jobs_dir = data.join("cron_jobs").to_string_lossy().to_string();
        config.storage.reports_dir = data.join("reports").to_string_lossy().to_string();
        config.storage.x_drafts_dir = data.join("x_drafts").to_string_lossy().to_string();
        config.storage.gen_images_dir = data.join("gen_images").to_string_lossy().to_string();
        config.storage.kb_dir = data.join("kb").to_string_lossy().to_string();
        config.storage.llm_audit_db_path =
            data.join("llm_audit.sqlite3").to_string_lossy().to_string();
        let _ = base; // suppress unused warning
    }
    if let Some(sd) = skills_dir {
        config.extra.insert(
            "skills_dir".to_string(),
            serde_yaml::Value::String(sd.to_string_lossy().to_string()),
        );
    }

    runtime::ensure_runtime_dirs(&config);

    let core = Arc::new(hone_channels::HoneBotCore::new(config));

    // ── 日志系统（全局唯一 buffer，订阅者只初始化一次）──────────────
    // 必须使用 global_log_buffer()：tracing 全局订阅者只能 set 一次，
    // 若每次 start_server 创建新 buffer，重连后 AppState 持有新 buffer
    // 但订阅者仍写入旧 buffer，造成 UI 日志消失。
    let log_buffer = global_log_buffer().clone();
    let log_level = core.config.logging.level.clone();
    init_logging(&log_buffer, &log_level);

    // ── 文件日志（仅首次 start_server 时启动写入任务）────────────────
    if let Some(data) = data_dir {
        let log_file_path = data.join("runtime").join("logs").join("web.log");
        if FILE_LOG_STARTED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let buf = log_buffer.clone();
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                let mut rx = buf.tx.subscribe();
                match tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file_path)
                    .await
                {
                    Ok(mut file) => loop {
                        match rx.recv().await {
                            Ok(entry) => {
                                let line = format!(
                                    "[{}] {:<5} {}\n",
                                    entry.timestamp, entry.level, entry.message
                                );
                                let _ = file.write_all(line.as_bytes()).await;
                                let _ = file.flush().await;
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                let _ = file
                                    .write_all(
                                        format!("[WARN ] 日志追赶：跳过 {n} 条\n").as_bytes(),
                                    )
                                    .await;
                            }
                        }
                    },
                    Err(e) => {
                        tracing::warn!("无法打开 web.log: {e}");
                    }
                }
            });
        }
    }

    // ── 构建 AppState ─────────────────────────────────────────────
    let (push_tx, _) = broadcast::channel::<PushEvent>(PUSH_CHANNEL_CAPACITY);
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client 构建失败: {e}"))?;
    let bearer_token = {
        let v = core.config.web.auth_token.trim().to_string();
        if v.is_empty() { None } else { Some(v) }
    };
    let state = Arc::new(InnerAppState {
        core,
        push_tx,
        http_client,
        log_buffer: log_buffer.clone(),
        deployment_mode: deployment_mode.to_string(),
        auth: AuthState {
            bearer_token,
            sse_tickets: Mutex::new(HashMap::new()),
        },
    });

    // ── UDP 日志接收（收集各 channel sidecar 的日志）────────────────
    let udp_port = state.core.config.logging.udp_port.unwrap_or(18118);
    let udp_log_buffer = log_buffer.clone();
    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{udp_port}");
        if let Ok(socket) = tokio::net::UdpSocket::bind(&addr).await {
            info!("UDP log server listening on {addr}");
            let mut buf = [0u8; 65536];
            loop {
                if let Ok((len, _)) = socket.recv_from(&mut buf).await {
                    if let Ok(entry) = serde_json::from_slice::<LogEntry>(&buf[..len]) {
                        udp_log_buffer.push(entry);
                    }
                }
            }
        }
    });

    // ── 调度器 ─────────────────────────────────────────────────────
    let mut scheduler_channels = vec!["web".to_string()];
    if state.core.config.imessage.enabled {
        scheduler_channels.insert(0, "imessage".to_string());
    }
    let (scheduler, event_rx) = state.core.create_scheduler(scheduler_channels);
    tokio::spawn(async move { scheduler.start().await });
    let state_for_scheduler = state.clone();
    tokio::spawn(async move { handle_scheduler_events(state_for_scheduler, event_rx).await });

    // ── 绑定端口（默认随机；若设置 HONE_WEB_PORT 则绑定指定端口）────────
    let bind_addr = std::env::var("HONE_WEB_PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .map(|port| format!("127.0.0.1:{port}"))
        .unwrap_or_else(|| "127.0.0.1:0".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("无法绑定端口 {bind_addr}: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("获取端口失败: {e}"))?
        .port();

    // ── 启动 Axum 服务 ────────────────────────────────────────────
    let app = build_app(state.clone());
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    info!("Hone Web API 服务已启动，端口 {port}");
    Ok((state, port))
}
