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
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use hone_core::config::{EventEngineConfig, HoneConfig};
use hone_event_engine::{
    BodyPolisher, DiscordSink, FeishuSink, IMessageSink, LlmPolisher, LogSink, MultiChannelSink,
    OutboundSink, TelegramSink, parse_polish_levels,
};
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

const DEFAULT_EVENT_ENGINE_NEWS_CLASSIFIER_MODEL: &str = "amazon/nova-lite-v1";

/// 装配"不确定来源 NewsCritical → LLM 仲裁"分类器。
/// 走 OpenRouter,key 复用 llm.openrouter.api_key。
/// 失败一律退化为 `None`(router 跳过 LLM 路径,uncertain 源新闻保持 Low)。
fn build_event_engine_news_classifier(
    core_cfg: &HoneConfig,
) -> Option<Arc<dyn hone_event_engine::NewsClassifier>> {
    match OpenRouterProvider::from_config(core_cfg) {
        Ok(provider) => {
            let provider: Arc<dyn LlmProvider> = Arc::new(provider);
            let model = core_cfg.event_engine.news_classifier_model.trim();
            let model = if model.is_empty() {
                DEFAULT_EVENT_ENGINE_NEWS_CLASSIFIER_MODEL
            } else {
                model
            };
            let classifier = hone_event_engine::LlmNewsClassifier::new(provider, model);
            info!("event engine: news LLM classifier 装配 (model={model})");
            Some(Arc::new(classifier) as Arc<dyn hone_event_engine::NewsClassifier>)
        }
        Err(e) => {
            tracing::warn!(
                "event engine: news LLM classifier 不可用,uncertain 源新闻将维持 Low: {e}"
            );
            None
        }
    }
}

/// 按 config 组装真实 OutboundSink(事件引擎的渠道出口)。
///
/// - 按每个 channel 的 `enabled` 以及必要凭据是否就绪,逐个 attach 真 sink
///   到 MultiChannelSink 上;未 attach 的渠道 fall back 到 LogSink
/// - 没有 channel 启用(极端情况) → 退化成纯 LogSink,语义不变
fn build_event_engine_sink(core_cfg: &HoneConfig) -> Arc<dyn OutboundSink> {
    let mut multi = MultiChannelSink::with_log_fallback();
    if core_cfg.telegram.enabled && !core_cfg.telegram.bot_token.trim().is_empty() {
        multi = multi.with_channel(
            "telegram",
            Arc::new(TelegramSink::new(core_cfg.telegram.bot_token.clone())),
        );
    }
    if core_cfg.discord.enabled && !core_cfg.discord.bot_token.trim().is_empty() {
        multi = multi.with_channel(
            "discord",
            Arc::new(DiscordSink::new(core_cfg.discord.bot_token.clone())),
        );
    }
    if core_cfg.feishu.enabled
        && !core_cfg.feishu.app_id.trim().is_empty()
        && !core_cfg.feishu.app_secret.trim().is_empty()
    {
        multi = multi.with_channel(
            "feishu",
            Arc::new(FeishuSink::new(
                core_cfg.feishu.app_id.clone(),
                core_cfg.feishu.app_secret.clone(),
            )),
        );
    }
    if core_cfg.imessage.enabled {
        multi = multi.with_channel("imessage", Arc::new(IMessageSink::new()));
    }
    let registered = multi.channels_registered();
    if registered.is_empty() {
        info!("event engine sink: 没有渠道启用,回退到 LogSink");
        return Arc::new(LogSink);
    }
    info!(
        channels = ?registered,
        "event engine sink: MultiChannelSink 已装配"
    );
    Arc::new(multi)
}

pub struct StartedServer {
    pub state: Arc<InnerAppState>,
    pub admin_port: u16,
    pub public_port: Option<u16>,
    pub task_handles: Vec<tokio::task::JoinHandle<()>>,
}

// ── 全局唯一 LogBuffer ──────────────────────────────────────────────────────
// tracing 订阅者在进程内只能设置一次，因此 LogBuffer 也必须全局唯一，
// 否则重连后 AppState 持有新 buffer 但订阅者仍向旧 buffer 写入，导致日志消失。
static GLOBAL_LOG_BUFFER: OnceLock<LogBuffer> = OnceLock::new();
static FILE_LOG_STARTED: AtomicBool = AtomicBool::new(false);
// tracing-appender 的 NonBlocking writer 依赖 WorkerGuard 存活;guard drop 即停止
// 后台 flush 线程,因此必须 hold 在静态变量里直到进程退出。
static FILE_LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

fn global_log_buffer() -> &'static LogBuffer {
    GLOBAL_LOG_BUFFER.get_or_init(LogBuffer::new)
}

/// 维护 `acp-events.log` 的简单按日轮转 + 15 天清理。
/// acp-events.log 的写入方在 `hone-channels` 里每写一行重新 open,所以这里
/// 直接 rename 不会撞到打开的 FD;无需协调锁。
fn spawn_acp_events_log_rotator(logs_dir: PathBuf) {
    const RETENTION_DAYS: i64 = 15;
    const ACTIVE_LOG: &str = "acp-events.log";
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(3600));
        loop {
            ticker.tick().await;
            let today = chrono::Local::now().date_naive();
            let active = logs_dir.join(ACTIVE_LOG);
            if let Ok(meta) = std::fs::metadata(&active) {
                if let Ok(mtime) = meta.modified() {
                    let mtime_local: chrono::DateTime<chrono::Local> = mtime.into();
                    let mtime_date = mtime_local.date_naive();
                    if mtime_date < today {
                        let rotated = logs_dir
                            .join(format!("{ACTIVE_LOG}.{}", mtime_date.format("%Y-%m-%d")));
                        if let Err(e) = std::fs::rename(&active, &rotated) {
                            tracing::warn!("acp-events.log 轮转失败: {e}");
                        } else {
                            tracing::info!(
                                "acp-events.log 已轮转 → {}",
                                rotated.file_name().and_then(|s| s.to_str()).unwrap_or("")
                            );
                        }
                    }
                }
            }
            let cutoff = today - chrono::Duration::days(RETENTION_DAYS);
            let Ok(entries) = std::fs::read_dir(&logs_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                    continue;
                };
                let Some((_prefix, date_str)) = name.rsplit_once('.') else {
                    continue;
                };
                let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
                    continue;
                };
                if date < cutoff {
                    if let Err(e) = std::fs::remove_file(entry.path()) {
                        tracing::warn!("旧日志清理失败 {name}: {e}");
                    }
                }
            }
        }
    });
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
    let mut task_handles = Vec::new();
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
    // web.log:走 tracing-appender DAILY rolling,保留最近 15 天,文件名形如
    //   web.log.YYYY-MM-DD(沿用 web.log. 前缀,既有 grep 仍能匹配)。
    // acp-events.log:每写一次重新 open,所以无需 FD 协调;另起一个 tick 任务
    //   每小时巡一次,把昨日 mtime 的 acp-events.log 重命名归档,并删 15 天前的归档。
    if let Some(data) = data_dir {
        let logs_dir = data.join("runtime").join("logs");
        let _ = std::fs::create_dir_all(&logs_dir);
        if FILE_LOG_STARTED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            match tracing_appender::rolling::Builder::new()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .filename_prefix("web.log")
                .max_log_files(15)
                .build(&logs_dir)
            {
                Ok(appender) => {
                    let (writer, guard) = tracing_appender::non_blocking(appender);
                    let _ = FILE_LOG_GUARD.set(guard);
                    let buf = log_buffer.clone();
                    tokio::spawn(async move {
                        use std::io::Write;
                        let mut rx = buf.tx.subscribe();
                        let mut writer = writer;
                        loop {
                            match rx.recv().await {
                                Ok(entry) => {
                                    let line = format!(
                                        "[{}] {:<5} {}\n",
                                        entry.timestamp, entry.level, entry.message
                                    );
                                    let _ = writer.write_all(line.as_bytes());
                                }
                                Err(broadcast::error::RecvError::Closed) => break,
                                Err(broadcast::error::RecvError::Lagged(n)) => {
                                    let _ = writer.write_all(
                                        format!("[WARN ] 日志追赶：跳过 {n} 条\n").as_bytes(),
                                    );
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("无法初始化 web.log RollingFileAppender: {e}");
                }
            }

            spawn_acp_events_log_rotator(logs_dir.clone());
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
    task_handles.push(tokio::spawn(async move {
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
    }));

    // ── 事件引擎（主动消息 feed，默认 enabled=false；config 开启后启动）──
    {
        let engine_cfg = state.core.config.event_engine.clone();
        let fmp_cfg = state.core.config.fmp.clone();
        let portfolio_dir = state.core.config.storage.portfolio_dir.clone();
        let notif_prefs_dir = state.core.config.storage.notif_prefs_dir.clone();
        // task_runs.jsonl 跟 heartbeat sidecar 同级 (data/runtime/);
        // 启动时清理 14 天前的旧文件 (failure 只 warn,不影响启动)。
        let task_runs_dir = hone_core::task_observer::task_runs_dir(&state.core.config);
        hone_core::task_observer::purge_old_task_runs(
            &task_runs_dir,
            hone_core::TASK_RUNS_RETENTION_DAYS,
        );
        let task_runs_dir_arc = std::sync::Arc::new(task_runs_dir.clone());
        let (events_db, events_jsonl, digest_dir) = {
            // 与 sessions.sqlite3 同目录：events.sqlite3 + events.jsonl + digest_buffer/
            let session_db =
                std::path::PathBuf::from(&state.core.config.storage.session_sqlite_db_path);
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
        let sink = build_event_engine_sink(&state.core.config);
        let news_classifier = build_event_engine_news_classifier(&state.core.config);
        // global_digest 也走 OpenRouter,与 news_classifier 用同一 provider
        let global_digest_provider: Option<Arc<dyn LlmProvider>> =
            match OpenRouterProvider::from_config(&state.core.config) {
                Ok(p) => Some(Arc::new(p)),
                Err(e) => {
                    tracing::warn!("global_digest LLM provider 不可用: {e}");
                    None
                }
            };

        // ── Thesis 蒸馏 cron(每 7 天扫一次,独立 task,挂掉不影响 digest)──
        if let Some(p) = global_digest_provider.clone() {
            let distill_model = state
                .core
                .config
                .event_engine
                .global_digest
                .event_dedupe_model
                .clone();
            let prefs_dir_clone = notif_prefs_dir.clone();
            let portfolio_dir_clone = portfolio_dir.clone();
            let thesis_task_runs_dir = task_runs_dir_arc.clone();
            task_handles.push(tokio::spawn(async move {
                let prefs_storage =
                    match hone_event_engine::prefs::FilePrefsStorage::new(&prefs_dir_clone) {
                        Ok(s) => Arc::new(s) as Arc<dyn hone_event_engine::prefs::PrefsProvider>,
                        Err(e) => {
                            tracing::warn!(
                                "thesis distill cron: prefs storage 打开失败: {e},cron 不启动"
                            );
                            return;
                        }
                    };
                let portfolio_storage =
                    Arc::new(hone_memory::PortfolioStorage::new(&portfolio_dir_clone));
                let sandbox_base = hone_channels::sandbox_base_dir();
                let distiller =
                    Arc::new(hone_event_engine::global_digest::LlmThesisDistiller::new(
                        p,
                        distill_model.clone(),
                    ));
                tracing::info!(
                    model = %distill_model,
                    sandbox_base = %sandbox_base.display(),
                    interval_hours = hone_event_engine::global_digest::DEFAULT_DISTILL_INTERVAL_HOURS,
                    "thesis distill cron starting"
                );
                hone_event_engine::global_digest::distill_cron_loop(
                    distiller,
                    prefs_storage,
                    portfolio_storage,
                    sandbox_base,
                    hone_event_engine::global_digest::DEFAULT_DISTILL_INTERVAL_HOURS,
                    Some(thesis_task_runs_dir),
                )
                .await;
            }));
        }

        let engine_task_runs_dir = task_runs_dir.clone();
        task_handles.push(tokio::spawn(async move {
            let mut engine = hone_event_engine::EventEngine::new(engine_cfg, fmp_cfg)
                .with_store_path(events_db)
                .with_events_jsonl_path(Some(events_jsonl))
                .with_portfolio_dir(portfolio_dir)
                .with_prefs_dir(notif_prefs_dir)
                .with_digest_dir(digest_dir)
                .with_task_runs_dir(Some(engine_task_runs_dir))
                .with_sink(sink);
            if let Some(p) = polisher {
                engine = engine.with_polisher(p);
            }
            if let Some(c) = news_classifier {
                engine = engine.with_news_classifier(c);
            }
            if let Some(p) = global_digest_provider {
                engine = engine.with_global_digest_provider(p);
            }
            if let Err(e) = engine.start().await {
                tracing::warn!("event engine start failed: {e}");
            }
        }));
    }

    // ── 调度器 ─────────────────────────────────────────────────────
    let mut scheduler_channels = vec!["web".to_string()];
    if state.core.config.imessage.enabled {
        scheduler_channels.insert(0, "imessage".to_string());
    }
    let (scheduler, event_rx) = state.core.create_scheduler(scheduler_channels);
    task_handles.push(tokio::spawn(async move { scheduler.start().await }));
    let state_for_scheduler = state.clone();
    task_handles.push(tokio::spawn(async move {
        handle_scheduler_events(state_for_scheduler, event_rx).await
    }));

    // ── 绑定管理端口（默认 8077，可通过 HONE_WEB_PORT 覆盖）─────────────
    let bind_addr = format!("127.0.0.1:{}", runtime_port());
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            for handle in task_handles {
                handle.abort();
            }
            return Err(format!("无法绑定端口 {bind_addr}: {e}"));
        }
    };
    let port = listener
        .local_addr()
        .map_err(|e| format!("获取端口失败: {e}"))?
        .port();

    let public_listener = if let Some(configured_public_port) = runtime_public_port() {
        let public_bind_addr = format!("127.0.0.1:{configured_public_port}");
        let public_listener = match tokio::net::TcpListener::bind(&public_bind_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                for handle in task_handles {
                    handle.abort();
                }
                return Err(format!("无法绑定用户端口 {public_bind_addr}: {e}"));
            }
        };
        let public_port = public_listener
            .local_addr()
            .map_err(|e| format!("获取用户端口失败: {e}"))?
            .port();
        Some((public_listener, public_port))
    } else {
        None
    };
    let public_port = public_listener
        .as_ref()
        .map(|(_, public_port)| *public_port);

    // ── 启动管理端 Axum 服务 ───────────────────────────────────────
    let admin_app = build_admin_app(state.clone());
    task_handles.push(tokio::spawn(async move {
        axum::serve(listener, admin_app).await.ok();
    }));

    if let Some((public_listener, _)) = public_listener {
        let public_app = build_public_app(state.clone());
        task_handles.push(tokio::spawn(async move {
            axum::serve(public_listener, public_app).await.ok();
        }));
    }

    info!("Hone Web API 管理端已启动，端口 {port}");
    if let Some(public_port) = public_port {
        info!("Hone Web API 用户端已启动，端口 {public_port}");
    }
    Ok(StartedServer {
        state,
        admin_port: port,
        public_port,
        task_handles,
    })
}
