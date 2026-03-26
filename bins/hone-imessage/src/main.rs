//! Hone iMessage Bot 入口
//!
//! macOS 上通过轮询 iMessage SQLite 数据库来获取消息，
//! 使用 AppleScript 发送回复。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use hone_channels::agent_session::{
    AgentRunOptions, AgentSession, AgentSessionEvent, AgentSessionListener, MessageMetadata,
};
use hone_channels::ingress::{
    ActorScopeResolver, GroupTrigger, IncomingEnvelope, MessageDeduplicator, SessionLockRegistry,
};
use hone_channels::outbound::{attach_stream_activity_probe, split_segments};
use hone_channels::prompt::PromptOptions;
use hone_channels::runtime::DEFAULT_MAX_SEGMENT_SIZE;
use hone_core::SessionIdentity;
use rusqlite::{Connection, OpenFlags};
use tracing::{error, info, warn};

// ── 控制台推送常量 ─────────────────────────────────────────────────────────────
/// Web 控制台接收 iMessage 事件的 API 地址（默认本地地址，可通过环境变量覆盖）
fn console_event_url() -> String {
    std::env::var("HONE_CONSOLE_URL").unwrap_or_else(|_| "http://127.0.0.1:8077".to_string())
        + "/api/imessage-event"
}

// ── 常量 ─────────────────────────────────────────────────────────────────────

/// AppleScript 发送后的间隔，避免过快连发
const SEND_DELAY_MS: u64 = 800;

const RECENT_MESSAGE_TTL_SECS: u64 = 120;
const RECENT_MESSAGE_MAX_ENTRIES: usize = 200;

#[derive(Clone)]
struct ImessageAppState {
    core: Arc<hone_channels::HoneBotCore>,
    dedup: MessageDeduplicator,
    session_locks: SessionLockRegistry,
    scope_resolver: ActorScopeResolver,
}

// ── 辅助函数 ──────────────────────────────────────────────────────────────────

/// 展开 ~ 为绝对路径
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs_fallback() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn dirs_fallback() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// 向 Web 控制台推送 iMessage 事件（fire-and-forget，失败不影响主流程）
///
/// event_type 取值：
/// - "imessage_user_message"       用户消息已写入 session
/// - "imessage_processing_start"   开始生成回复
/// - "imessage_assistant_message"  回复已生成完毕
/// - "imessage_processing_error"   处理失败
fn push_console_event(handle: &str, event_type: &str, data: serde_json::Value) {
    let handle = handle.to_string();
    let event_type = event_type.to_string();
    let url = console_event_url();
    tokio::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
        {
            Ok(c) => c,
            Err(_) => return,
        };
        let body = serde_json::json!({
            "channel": "imessage",
            "user_id": handle,
            "event_type": event_type,
            "data": data,
        });
        let _ = client.post(&url).json(&body).send().await;
    });
}

/// 通过 AppleScript 发送 iMessage
fn send_imessage(handle: &str, text: &str) -> bool {
    // 转义 AppleScript 字符串中的特殊字符
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");

    let script = format!(
        r#"tell application "Messages"
    set targetService to 1st account whose service type = iMessage
    set targetBuddy to participant "{handle}" of targetService
    send "{escaped}" to targetBuddy
end tell"#
    );

    match std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("AppleScript 发送失败: {}", stderr.trim());
                false
            }
        }
        Err(e) => {
            error!("无法执行 osascript: {}", e);
            false
        }
    }
}

fn imessage_admin_prompt(project_root: &str) -> String {
    format!(
        "【管理员权限】\
        \n你正在与管理员用户交互，具有以下特殊能力：\
        \n1. 可以直接查看和修改 Hone 项目的源代码与配置文件\
        \n   项目根目录：{project_root}\
        \n   主要目录：src/bins（渠道入口）、crates/（核心库）、skills/（技能）、config.yaml（配置）\
        \n2. 修改完源代码后，可调用 restart_hone(confirm=\"yes\") 工具重启 Hone（将重新编译并启动）\
        \n3. 管理员操作须谨慎，修改源码后建议先确认变更内容再执行重启\
        \n如需使用管理员功能，请先 load_skill(\"hone_admin\") 获取详细操作指引。"
    )
}

fn imessage_error_reason(
    kind: hone_channels::agent_session::AgentSessionErrorKind,
) -> &'static str {
    use hone_channels::agent_session::AgentSessionErrorKind as Kind;
    match kind {
        Kind::SpawnFailed => "spawn_failed",
        Kind::StdoutUnavailable => "stdout_unavailable",
        Kind::TimeoutOverall => "timeout",
        Kind::TimeoutPerLine => "per_line_timeout",
        Kind::GeminiError => "gemini_error_event",
        Kind::ContextWindowOverflow => "context_window_overflow",
        Kind::ExitFailure => "exit_failure",
        Kind::Io => "stdout_read_error",
        Kind::AgentTimeout => "timeout",
        Kind::AgentFailed => "agent_error",
    }
}

fn parse_iteration(detail: &Option<String>) -> Option<u32> {
    let raw = detail.as_deref()?.trim();
    raw.strip_prefix("iteration=")?.parse::<u32>().ok()
}

struct ImessageConsoleListener {
    handle: String,
}

#[async_trait]
impl AgentSessionListener for ImessageConsoleListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        match event {
            AgentSessionEvent::UserMessage { content } => {
                push_console_event(
                    &self.handle,
                    "imessage_user_message",
                    serde_json::json!({ "text": content }),
                );
            }
            AgentSessionEvent::Progress { stage, detail } => match stage {
                "agent.run" => {
                    push_console_event(
                        &self.handle,
                        "imessage_processing_start",
                        serde_json::json!({ "agent": detail.unwrap_or_default() }),
                    );
                }
                "gemini.spawn" | "gemini.final_response" => {
                    let payload = if let Some(iteration) = parse_iteration(&detail) {
                        serde_json::json!({ "stage": stage, "iteration": iteration })
                    } else {
                        serde_json::json!({ "stage": stage })
                    };
                    push_console_event(&self.handle, "imessage_progress", payload);
                }
                "tool.execute" => {
                    let payload =
                        serde_json::json!({ "stage": stage, "tool": detail.unwrap_or_default() });
                    push_console_event(&self.handle, "imessage_progress", payload);
                }
                _ => {}
            },
            AgentSessionEvent::Error { error } => {
                push_console_event(
                    &self.handle,
                    "imessage_processing_error",
                    serde_json::json!({
                        "reason": imessage_error_reason(error.kind),
                        "error": error.message
                    }),
                );
            }
            AgentSessionEvent::Done { response } => {
                if response.success {
                    push_console_event(
                        &self.handle,
                        "imessage_assistant_message",
                        serde_json::json!({
                            "text": response.content,
                            "iterations": response.iterations
                        }),
                    );
                }
            }
            _ => {}
        }
    }
}

struct ImessageStreamListener {
    handle: String,
    core: Arc<hone_channels::HoneBotCore>,
    session_id: String,
    buffer: tokio::sync::Mutex<String>,
    sent_segments: tokio::sync::Mutex<usize>,
}

impl ImessageStreamListener {
    async fn flush_buffer(&self, allow_empty: bool) {
        let mut leftover = String::new();
        {
            let mut guard = self.buffer.lock().await;
            let trimmed = guard.trim().to_string();
            if allow_empty || !trimmed.is_empty() {
                leftover = trimmed;
                guard.clear();
            }
        }

        if leftover.is_empty() {
            return;
        }

        send_imessage(&self.handle, &leftover);
        let mut sent = self.sent_segments.lock().await;
        *sent += 1;
        tokio::time::sleep(Duration::from_millis(SEND_DELAY_MS)).await;
    }

    async fn send_segments(&self, segments: Vec<String>) {
        let total = segments.len();
        for (index, segment) in segments.into_iter().enumerate() {
            send_imessage(&self.handle, &segment);
            let mut sent = self.sent_segments.lock().await;
            *sent += 1;
            drop(sent);
            if index + 1 < total {
                tokio::time::sleep(Duration::from_millis(SEND_DELAY_MS)).await;
            }
        }
    }
}

#[async_trait]
impl AgentSessionListener for ImessageStreamListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        match event {
            AgentSessionEvent::StreamDelta { content } => {
                let mut segments = Vec::new();
                {
                    let mut guard = self.buffer.lock().await;
                    guard.push_str(&content);
                    while guard.chars().count() >= 100 {
                        let cut = char_boundary_at(&guard, 100);
                        let segment = guard[..cut].to_string();
                        *guard = guard[cut..].to_string();
                        segments.push(segment);
                    }
                }
                if !segments.is_empty() {
                    self.send_segments(segments).await;
                }
            }
            AgentSessionEvent::Error { error } => {
                if matches!(
                    error.kind,
                    hone_channels::agent_session::AgentSessionErrorKind::TimeoutOverall
                ) {
                    self.flush_buffer(false).await;
                }
            }
            AgentSessionEvent::Done { .. } => {
                self.flush_buffer(false).await;
                let sent = *self.sent_segments.lock().await;
                if sent > 0 {
                    self.core.log_message_step(
                        "imessage",
                        &self.handle,
                        &self.session_id,
                        "reply.send",
                        &format!("segments.sent={sent}/{sent}"),
                        None,
                        None,
                    );
                }
            }
            _ => {}
        }
    }
}

/// 构建 system prompt（含历史总结、模型名、中文指令、管理员权限）
/// 从 chat.db 查询新消息
struct IncomingMessage {
    rowid: i64,
    handle: String,
    text: String,
}

fn poll_new_messages(
    db_path: &PathBuf,
    last_rowid: i64,
    target_handle: &str,
) -> Option<Vec<IncomingMessage>> {
    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(_) => {
            return None;
        }
    };

    // is_from_me=0 表示来自对方的消息
    // m.date 存储有两种可能：秒级或纳秒级 Mac Epoch 时间
    // macOS Epoch: 2001-01-01 00:00:00 UTC
    let sql = r#"
        SELECT
            m.ROWID,
            h.id AS handle_id,
            COALESCE(m.text, '') AS text
        FROM message m
        JOIN handle h ON m.handle_id = h.ROWID
        WHERE m.ROWID > ?1
          AND m.is_from_me = 0
          AND m.text IS NOT NULL
          AND m.text != ''
          AND m.service = 'iMessage'
          -- 只处理最近 5 分钟 (300 秒) 的消息，忽略 iCloud 同步进来的几年/几个月前的旧数据
          AND (
            m.date >= (strftime('%s', 'now') - 978307200 - 300) -- 秒级格式 (Mac OS 10.12-)
            OR
            m.date >= ((strftime('%s', 'now') - 978307200 - 300) * 1000000000) -- 纳秒级格式 (Mac OS 10.13+)
          )
        ORDER BY m.ROWID ASC
        LIMIT 50
    "#;

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => {
            error!("SQL prepare 失败: {}", e);
            return Some(vec![]);
        }
    };

    let rows = match stmt.query_map([last_rowid], |row| {
        Ok(IncomingMessage {
            rowid: row.get(0)?,
            handle: row.get(1)?,
            text: row.get(2)?,
        })
    }) {
        Ok(r) => r,
        Err(e) => {
            error!("SQL 查询失败: {}", e);
            return Some(vec![]);
        }
    };

    let mut messages = Vec::new();
    for row in rows {
        match row {
            Ok(msg) => {
                // 过滤 target_handle
                if !target_handle.is_empty() && msg.handle != target_handle {
                    continue;
                }
                messages.push(msg);
            }
            Err(e) => {
                warn!("读取消息行失败: {}", e);
            }
        }
    }

    Some(messages)
}

/// 获取 chat.db 中当前最大的 ROWID
fn get_max_rowid(db_path: &PathBuf) -> i64 {
    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            warn!("获取 max ROWID 失败（可能无权限）: {}", e);
            return 0;
        }
    };

    conn.query_row("SELECT COALESCE(MAX(ROWID), 0) FROM message", [], |row| {
        row.get(0)
    })
    .unwrap_or(0)
}

// ── 主入口 ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let (config, config_path) = match hone_channels::load_runtime_config() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("❌ 配置加载失败: {e}");
            std::process::exit(1);
        }
    };
    let core = Arc::new(hone_channels::HoneBotCore::new(config));
    let app_state = Arc::new(ImessageAppState {
        core: core.clone(),
        dedup: MessageDeduplicator::new(
            Duration::from_secs(RECENT_MESSAGE_TTL_SECS),
            RECENT_MESSAGE_MAX_ENTRIES,
        ),
        session_locks: SessionLockRegistry::new(),
        scope_resolver: ActorScopeResolver::new("imessage"),
    });

    hone_core::logging::setup_logging(&core.config.logging);

    // 确保 gemini CLI 可被子进程找到（安装在 ~/local/node/bin）
    if let Some(home) = dirs_fallback() {
        let gemini_bin = home.join("local/node/bin");
        if gemini_bin.exists() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            // Safety: called at startup before spawning threads
            unsafe {
                std::env::set_var("PATH", format!("{}:{}", gemini_bin.display(), current_path));
            }
            info!("🔧 已将 {} 加入 PATH", gemini_bin.display());
        }
    }

    info!("🍎 Hone iMessage Bot 启动");
    info!(
        "   model={} timeout={} agent={}",
        core.config.llm.openrouter.model,
        core.config.llm.openrouter.timeout,
        core.config.agent.runner
    );
    info!(
        "   LLM provider: {}",
        if core.llm.is_some() {
            "已就绪"
        } else {
            "未配置（压缩功能不可用）"
        }
    );
    core.log_startup_routing("imessage", &config_path);

    if !core.config.imessage.enabled {
        warn!("imessage.enabled=false，iMessage Bot 不会启动。");
        return;
    }

    let _heartbeat = match hone_core::spawn_process_heartbeat(&core.config, "imessage") {
        Ok(heartbeat) => heartbeat,
        Err(err) => {
            error!("无法启动 iMessage heartbeat: {err}");
            std::process::exit(1);
        }
    };

    let imessage_cfg = &core.config.imessage;
    let db_path = expand_tilde(&imessage_cfg.db_path);
    let poll_interval = Duration::from_secs(imessage_cfg.poll_interval);
    let target_handle = imessage_cfg.target_handle.clone();

    // 检查数据库可访问性
    if !db_path.exists() {
        error!("❌ chat.db 不存在: {:?}", db_path);
        error!("   请确保路径正确且已授予「完全磁盘访问权限」");
        std::process::exit(1);
    }

    info!("📂 chat.db 路径: {:?}", db_path);
    info!("⏱  轮询间隔: {}s", imessage_cfg.poll_interval);
    if !target_handle.is_empty() {
        info!("🎯 仅监听: {}", target_handle);
    } else {
        info!("📨 监听所有联系人");
    }

    // 初始化：记录当前最大 ROWID，只处理启动后的新消息
    let mut last_rowid = get_max_rowid(&db_path);
    info!("📌 起始 ROWID: {}", last_rowid);
    info!("✅ iMessage Bot 已就绪，等待新消息...");

    // 启动内置 HTTP 服务器，供 hone-console-page 定时任务回调使用
    let listen_addr = core.config.imessage.listen_addr.clone();
    start_http_server(listen_addr);

    // 连续打开 chat.db 失败的次数（用于抑制重复日志）
    let mut db_open_fail_count: u32 = 0;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("👋 iMessage Bot 已停止");
                break;
            }
            _ = tokio::time::sleep(poll_interval) => {
                let poll_result = poll_new_messages(&db_path, last_rowid, &target_handle);

                let new_messages = match poll_result {
                    None => {
                        // DB 打开失败：首次打 error，后续每 60 次（约 2 分钟）打一次 warn，避免刷屏
                        if db_open_fail_count == 0 {
                            error!(
                                "无法打开 chat.db：{:?}\n  → macOS 需要「完全磁盘访问权限」\n  → 请前往：系统设置 → 隐私与安全性 → 完全磁盘访问权限\n  → 将运行本程序的终端 App（如 Terminal / iTerm2 / Cursor）加入列表后重启",
                                db_path
                            );
                        } else if db_open_fail_count % 60 == 0 {
                            warn!("chat.db 仍无法打开（已失败 {} 次），请检查「完全磁盘访问权限」", db_open_fail_count);
                        }
                        db_open_fail_count = db_open_fail_count.saturating_add(1);
                        continue;
                    }
                    Some(msgs) => {
                        if db_open_fail_count > 0 {
                            info!("✅ chat.db 已恢复可访问（之前失败 {} 次）", db_open_fail_count);
                            db_open_fail_count = 0;
                        }
                        msgs
                    }
                };

                if new_messages.is_empty() {
                    continue;
                }

                for msg in new_messages {
                    info!("📩 [{}] {}", msg.handle, msg.text);
                    last_rowid = last_rowid.max(msg.rowid);

                    let dedupe_key = format!("{}::{}::{}", msg.rowid, msg.handle, msg.text);
                    if app_state.dedup.is_duplicate(&dedupe_key) {
                        warn!(
                            "[iMessage] [{}] 检测到短期重复消息，已跳过处理",
                            msg.handle
                        );
                        continue;
                    }
                    let state = Arc::clone(&app_state);

                    // 在独立 task 中处理，避免阻塞轮询
                    tokio::spawn(async move {
                        process_message(state, msg).await;
                    });
                }
            }
        }
    }
}

/// 处理一条收到的 iMessage 消息（分发至流式或非流式路径）
async fn process_message(state: Arc<ImessageAppState>, msg: IncomingMessage) {
    let handle = msg.handle.clone();
    let text = msg.text.clone();
    let (actor, channel_target, chat_mode) =
        match state.scope_resolver.direct(&handle, handle.clone()) {
            Ok(value) => value,
            Err(err) => {
                error!("[iMessage] [{}] actor 构建失败: {}", handle, err);
                return;
            }
        };
    let envelope = IncomingEnvelope {
        message_id: Some(msg.rowid.to_string()),
        actor: actor.clone(),
        session_identity: SessionIdentity::from_actor(&actor)
            .expect("imessage actor should always map to a session identity"),
        session_id: SessionIdentity::from_actor(&actor)
            .expect("imessage actor should always map to a session identity")
            .session_id(),
        channel_target,
        chat_mode,
        text: text.clone(),
        attachments: Vec::new(),
        trigger: GroupTrigger::default(),
        recv_extra: None,
        session_metadata: None,
        message_metadata: MessageMetadata::default(),
    };
    if state
        .core
        .try_intercept_admin_registration(&envelope.actor, &envelope.text)
    {
        send_imessage(&handle, hone_channels::core::REGISTER_ADMIN_INTERCEPT_ACK);
        return;
    }
    let session_id = envelope.actor.session_id();
    let _session_guard = state.session_locks.lock(&session_id).await;
    let is_admin = state.core.is_admin_actor(&envelope.actor);

    info!(
        "[iMessage] [{}] 开始处理: {} (agent={})",
        handle, text, state.core.config.agent.runner
    );

    let _ = send_imessage(&handle, "收到，Hone正在处理...");
    process_message_session(state, envelope, is_admin).await;
}

/// 统一路径：按 AgentSession 事件决定是否出现流式增量，不再按 runner 名称分支。
async fn process_message_session(
    state: Arc<ImessageAppState>,
    envelope: IncomingEnvelope,
    is_admin: bool,
) {
    let handle = envelope.actor.user_id.clone();
    let session_id = envelope.actor.session_id();
    let mut prompt_options = PromptOptions {
        is_admin,
        model_hint: Some("基础模型由当前运行配置决定".to_string()),
        force_chinese: true,
        ..PromptOptions::default()
    };
    if is_admin {
        let project_root = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string());
        prompt_options.admin_prompt = Some(imessage_admin_prompt(&project_root));
    }

    let mut session = AgentSession::new(
        state.core.clone(),
        envelope.actor.clone(),
        envelope.channel_target.clone(),
    )
    .with_message_id(envelope.message_id.clone())
    .with_message_metadata(envelope.message_metadata.clone())
    .with_prompt_options(prompt_options)
    .with_cron_allowed(envelope.cron_allowed());

    session.add_listener(Arc::new(ImessageConsoleListener {
        handle: handle.to_string(),
    }));
    session.add_listener(Arc::new(ImessageStreamListener {
        handle: handle.to_string(),
        core: state.core.clone(),
        session_id: session_id.clone(),
        buffer: tokio::sync::Mutex::new(String::new()),
        sent_segments: tokio::sync::Mutex::new(0),
    }));
    let stream_probe = attach_stream_activity_probe(&mut session);

    let timeout_secs = state.core.config.llm.openrouter.timeout.clamp(180, 360);
    let run_options = AgentRunOptions {
        timeout: Some(Duration::from_secs(timeout_secs)),
        segmenter: None,
        quota_mode: hone_channels::agent_session::AgentRunQuotaMode::UserConversation,
        model_override: None,
    };
    let result = session.run(&envelope.text, run_options).await;
    let response = result.response;

    if stream_probe.saw_stream_delta() {
        if response.success {
            info!(
                "[iMessage] [{}] 回复成功，迭代 {} 次",
                handle, response.iterations
            );
        } else if let Some(err) = response.error.clone() {
            error!("[iMessage] [{}] 处理失败: {}", handle, err);
            let truncated: String = err.chars().take(120).collect();
            let _ = send_imessage(&handle, &format!("(处理中断：{})", truncated));
        }
        return;
    }

    if response.success {
        info!(
            "[iMessage] [{}] 回复成功，迭代 {} 次",
            handle, response.iterations
        );
        let full = if response.content.trim().is_empty() {
            "收到。".to_string()
        } else {
            response.content.trim().to_string()
        };
        let segments = split_segments(&full, DEFAULT_MAX_SEGMENT_SIZE, DEFAULT_MAX_SEGMENT_SIZE);
        let total_segments = segments.len();
        let mut sent_segments = 0usize;
        for (index, segment) in segments.iter().enumerate() {
            if send_imessage(&handle, segment) {
                sent_segments += 1;
            }
            if index + 1 < total_segments {
                tokio::time::sleep(Duration::from_millis(SEND_DELAY_MS)).await;
            }
        }
        state.core.log_message_step(
            "imessage",
            &handle,
            &session_id,
            "reply.send",
            &format!("segments.sent={sent_segments}/{total_segments}"),
            None,
            None,
        );
    } else if let Some(err) = response.error.clone() {
        error!("[iMessage] [{}] 处理失败: {}", handle, err);
        let truncated: String = err.chars().take(100).collect();
        if err.contains("agent_timeout") {
            let _ = send_imessage(&handle, "抱歉，处理超时了。请稍后再试。");
        } else {
            let _ = send_imessage(&handle, &format!("抱歉，处理出错了: {}", truncated));
        }
    }
}

/// 在 text 中按字符数找到字节边界（用于安全截断 UTF-8 字符串）
fn char_boundary_at(s: &str, char_count: usize) -> usize {
    s.char_indices()
        .nth(char_count)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

// ── 内置 HTTP 服务器（供 hone-console-page 定时任务回调） ─────────────────────

/// 请求体：从 hone-console-page 发来的"请向此 handle 发送 iMessage"命令
#[derive(serde::Deserialize)]
struct SendRequest {
    /// iMessage handle（电话号码或邮箱）
    handle: String,
    /// 要发送的文本内容
    text: String,
    /// 可选：任务名称，仅用于日志
    #[serde(default)]
    job_name: String,
}

/// POST /api/send — 接收来自 hone-console-page 的回调，通过 AppleScript 发送 iMessage
async fn handle_send(State(_): State<()>, Json(req): Json<SendRequest>) -> axum::http::StatusCode {
    if req.handle.is_empty() || req.text.is_empty() {
        warn!("[iMessage/HTTP] 收到空 handle 或空内容，忽略");
        return axum::http::StatusCode::BAD_REQUEST;
    }

    info!(
        "[iMessage/HTTP] 定时任务投递: handle={} job={} text_len={}",
        req.handle,
        req.job_name,
        req.text.len()
    );

    // 推送到 Web 控制台（让用户在控制台页面也能看到）
    push_console_event(
        &req.handle,
        "imessage_assistant_message",
        serde_json::json!({ "text": req.text, "source": "scheduled", "job_name": req.job_name }),
    );

    // 按段发送 iMessage（每段 ≤ DEFAULT_MAX_SEGMENT_SIZE 字符）
    let segments = split_segments(
        &req.text,
        DEFAULT_MAX_SEGMENT_SIZE,
        DEFAULT_MAX_SEGMENT_SIZE,
    );
    let total = segments.len();
    let mut ok_count = 0usize;
    for seg in &segments {
        if send_imessage(&req.handle, seg) {
            ok_count += 1;
        } else {
            warn!(
                "[iMessage/HTTP] 发送失败: handle={} seg_len={}",
                req.handle,
                seg.len()
            );
        }
    }

    if ok_count == total {
        info!(
            "[iMessage/HTTP] 投递完成: handle={} segments={}/{}",
            req.handle, ok_count, total
        );
        axum::http::StatusCode::OK
    } else {
        error!(
            "[iMessage/HTTP] 部分投递失败: handle={} ok={}/{}",
            req.handle, ok_count, total
        );
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// 启动内置 HTTP 服务器（非阻塞，在 tokio 任务中运行）
pub fn start_http_server(listen_addr: String) {
    let app = Router::new()
        .route("/api/send", post(handle_send))
        .with_state(());

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&listen_addr).await {
            Ok(l) => {
                info!("[iMessage/HTTP] 内置服务已启动: http://{}", listen_addr);
                l
            }
            Err(e) => {
                error!(
                    "[iMessage/HTTP] 无法绑定 {}: {}，定时任务将无法通过 iMessage 回调",
                    listen_addr, e
                );
                return;
            }
        };
        if let Err(e) = axum::serve(listener, app).await {
            error!("[iMessage/HTTP] 服务异常退出: {}", e);
        }
    });
}
