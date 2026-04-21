//! hone-web-api — Hone 控制台 HTTP 服务库
//!
//! 将原 `hone-console-page` 二进制的服务逻辑提取为库，
//! 供 `hone-desktop` 在 Tauri 主进程内直接嵌入启动，无需子进程 sidecar。

pub mod logging;
mod public_auth;
pub mod routes;
pub mod runtime;
pub mod state;
pub mod types;

pub use logging::{LogBuffer, LogCaptureLayer, LogEntry};
pub use routes::{build_admin_app, build_public_app};
pub use state::{AppState, AuthState, PushEvent};

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use hone_core::config::{EventEngineConfig, HoneConfig};
use hone_event_engine::{BodyPolisher, LlmPolisher, parse_polish_levels};
use hone_llm::{LlmProvider, OpenRouterProvider};
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::routes::events::handle_scheduler_events;
use crate::runtime::{runtime_port, runtime_public_port};
use crate::state::AppState as InnerAppState;

const PUSH_CHANNEL_CAPACITY: usize = 64;

/// 按 config 决定是否装配 LlmPolisher。
/// 失败路径统一回退到 `None`（引擎继续用 NoopPolisher，走默认模板）。
fn build_event_engine_polisher(
    core_cfg: &HoneConfig,
    engine_cfg: &EventEngineConfig,
) -> Option<Arc<dyn BodyPolisher>> {
    let levels = parse_polish_levels(&engine_cfg.renderer.llm_polish_for);
    if levels.is_empty() {
        return None;
    }
    match OpenRouterProvider::from_config(core_cfg) {
        Ok(provider) => {
            let provider: Arc<dyn LlmProvider> = Arc::new(provider);
            let polisher = LlmPolisher::new(provider, levels)
                .with_model(core_cfg.llm.openrouter.auxiliary_model());
            info!("event engine: LlmPolisher 已装配");
            Some(Arc::new(polisher) as Arc<dyn BodyPolisher>)
        }
        Err(e) => {
            tracing::warn!("event engine: llm provider 不可用，跳过 polish: {e}");
            None
        }
    }
}

pub struct StartedServer {
    pub state: Arc<InnerAppState>,
    pub admin_port: u16,
    pub public_port: Option<u16>,
}

// ── 全局唯一 LogBuffer ──────────────────────────────────────────────────────
// tracing 订阅者在进程内只能设置一次，因此 LogBuffer 也必须全局唯一，
// 否则重连后 AppState 持有新 buffer 但订阅者仍向旧 buffer 写入，导致日志消失。
static GLOBAL_LOG_BUFFER: OnceLock<LogBuffer> = OnceLock::new();
static FILE_LOG_STARTED: AtomicBool = AtomicBool::new(false);

fn global_log_buffer() -> &'static LogBuffer {
    GLOBAL_LOG_BUFFER.get_or_init(LogBuffer::new)
}

#[cfg(test)]
pub(crate) fn test_env_lock() -> &'static Mutex<()> {
    static TEST_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_ENV_LOCK.get_or_init(|| Mutex::new(()))
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
/// 返回启动后的共享状态与监听端口。服务在后台 Tokio task 中运行，进程退出时自然结束。
pub async fn start_server(
    config_path: &str,
    data_dir: Option<&Path>,
    skills_dir: Option<&Path>,
    deployment_mode: &str,
) -> Result<StartedServer, String> {
    let mut config =
        HoneConfig::from_file(config_path).map_err(|e| format!("配置加载失败: {e}"))?;
    config.apply_runtime_overrides(data_dir, skills_dir, Some(Path::new(config_path)));
    config.ensure_runtime_dirs();

    let core = Arc::new(hone_channels::HoneBotCore::new(config));
    let web_auth = Arc::new(
        hone_memory::WebAuthStorage::new(&core.config.storage.session_sqlite_db_path)
            .map_err(|e| format!("Web Auth 存储初始化失败: {e}"))?,
    );

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
        web_auth,
        public_auth_limiter: Default::default(),
        push_tx,
        http_client,
        log_buffer: log_buffer.clone(),
        deployment_mode: deployment_mode.to_string(),
        auth: AuthState {
            bearer_token,
            sse_tickets: Mutex::new(HashMap::new()),
        },
        heartbeat_registry: Default::default(),
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

    // ── 事件引擎（主动消息 feed，默认 enabled=false；config 开启后启动）──
    {
        let engine_cfg = state.core.config.event_engine.clone();
        let fmp_cfg = state.core.config.fmp.clone();
        let portfolio_dir = state.core.config.storage.portfolio_dir.clone();
        let notif_prefs_dir = state.core.config.storage.notif_prefs_dir.clone();
        let (events_db, events_jsonl, digest_dir) = {
            // 与 sessions.sqlite3 同目录：events.sqlite3 + events.jsonl + digest_buffer/
            let session_db = std::path::PathBuf::from(
                &state.core.config.storage.session_sqlite_db_path,
            );
            let base = session_db
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("./data"));
            (
                base.join("events.sqlite3"),
                base.join("events.jsonl"),
                base.join("digest_buffer"),
            )
        };
        // 可选 LLM 润色：当 llm_polish_for 非空且 llm provider 可用时装配 LlmPolisher。
        let polisher = build_event_engine_polisher(&state.core.config, &engine_cfg);
        tokio::spawn(async move {
            let mut engine = hone_event_engine::EventEngine::new(engine_cfg, fmp_cfg)
                .with_store_path(events_db)
                .with_events_jsonl_path(Some(events_jsonl))
                .with_portfolio_dir(portfolio_dir)
                .with_prefs_dir(notif_prefs_dir)
                .with_digest_dir(digest_dir);
            if let Some(p) = polisher {
                engine = engine.with_polisher(p);
            }
            if let Err(e) = engine.start().await {
                tracing::warn!("event engine start failed: {e}");
            }
        });
    }

    // ── 调度器 ─────────────────────────────────────────────────────
    let mut scheduler_channels = vec!["web".to_string()];
    if state.core.config.imessage.enabled {
        scheduler_channels.insert(0, "imessage".to_string());
    }
    let (scheduler, event_rx) = state.core.create_scheduler(scheduler_channels);
    tokio::spawn(async move { scheduler.start().await });
    let state_for_scheduler = state.clone();
    tokio::spawn(async move { handle_scheduler_events(state_for_scheduler, event_rx).await });

    // ── 绑定管理端口（默认 8077，可通过 HONE_WEB_PORT 覆盖）─────────────
    let bind_addr = format!("127.0.0.1:{}", runtime_port());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("无法绑定端口 {bind_addr}: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("获取端口失败: {e}"))?
        .port();

    // ── 启动管理端 Axum 服务 ───────────────────────────────────────
    let admin_app = build_admin_app(state.clone());
    tokio::spawn(async move {
        axum::serve(listener, admin_app).await.ok();
    });

    let public_port = if let Some(configured_public_port) = runtime_public_port() {
        let public_bind_addr = format!("127.0.0.1:{configured_public_port}");
        let public_listener = tokio::net::TcpListener::bind(&public_bind_addr)
            .await
            .map_err(|e| format!("无法绑定用户端口 {public_bind_addr}: {e}"))?;
        let public_port = public_listener
            .local_addr()
            .map_err(|e| format!("获取用户端口失败: {e}"))?
            .port();
        let public_app = build_public_app(state.clone());
        tokio::spawn(async move {
            axum::serve(public_listener, public_app).await.ok();
        });
        Some(public_port)
    } else {
        None
    };

    info!("Hone Web API 管理端已启动，端口 {port}");
    if let Some(public_port) = public_port {
        info!("Hone Web API 用户端已启动，端口 {public_port}");
    }
    Ok(StartedServer {
        state,
        admin_port: port,
        public_port,
    })
}
