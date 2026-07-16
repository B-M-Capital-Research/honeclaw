use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde_json::{Value, json};
use tokio_stream::StreamExt;
use tracing::{error, info};

use hone_channels::agent_session::{
    AgentRunOptions, AgentRunQuotaMode, AgentSession, AgentSessionEvent, AgentSessionListener,
    run_with_progress_ticks,
};
use hone_channels::prompt::PromptOptions;
use hone_channels::run_event::RunEvent;
use hone_channels::runtime::{clean_msg_markers, should_skip_buffer};
use hone_core::ActorIdentity;

use crate::state::{ActiveChatRunHandle, AppState};
use crate::types::ChatRequest;

const WEB_CHAT_PROGRESS_TICK: Duration = Duration::from_secs(10);
const WEB_CHAT_PROGRESS_MIN_SILENCE: Duration = Duration::from_secs(15);
const WEB_CHAT_PROGRESS_STATUS: &str = "仍在处理中，正在完成核验与分析";

pub(crate) struct SseSessionListener {
    tx: tokio::sync::mpsc::Sender<(String, Value)>,
    user_id: String,
    sent_segments: Arc<tokio::sync::Mutex<usize>>,
    active_run: Option<ActiveChatRunHandle>,
    terminal_sent: Arc<AtomicBool>,
}

#[async_trait]
impl AgentSessionListener for SseSessionListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        if !matches!(&event, AgentSessionEvent::Done { .. })
            && self.terminal_sent.load(Ordering::Acquire)
        {
            return;
        }
        match event {
            AgentSessionEvent::Segment { text } => {
                if let Some(active_run) = &self.active_run {
                    let _ = active_run.update("running", "正在输出最终回答");
                }
                let _ = self
                    .tx
                    .send(("assistant_delta".into(), json!({ "content": text })))
                    .await;
                let mut guard = self.sent_segments.lock().await;
                *guard += 1;
            }
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => {
                if let Some(active_run) = &self.active_run {
                    let _ = active_run.update("running", "正在输出最终回答");
                }
                let _ = self
                    .tx
                    .send(("assistant_delta".into(), json!({ "content": content })))
                    .await;
                let mut guard = self.sent_segments.lock().await;
                *guard += 1;
            }
            AgentSessionEvent::Run(RunEvent::StreamReset) => {
                if let Some(active_run) = &self.active_run {
                    let _ = active_run.update("running", "正在复核结果，确保信息准确");
                }
                let _ = self.tx.send(("assistant_reset".into(), json!({}))).await;
                let mut guard = self.sent_segments.lock().await;
                *guard = 0;
            }
            AgentSessionEvent::Run(RunEvent::ToolStatus {
                status,
                tool,
                reasoning,
                message,
            }) => {
                let status_text = public_tool_status_text(&status);
                if let Some(active_run) = &self.active_run {
                    let _ = active_run.update("running", status_text);
                }
                let payload = json!({
                    "tool": tool,
                    "status": status,
                    "text": message,
                    "reasoning": reasoning,
                    // Public chat must only render this fixed, user-safe field.
                    "public_status_text": status_text,
                });
                let _ = self.tx.send(("tool_call".into(), payload)).await;
            }
            AgentSessionEvent::Run(RunEvent::Progress { stage, detail: _ }) => {
                let (phase, status_text) = public_progress_status(stage);
                let run = self
                    .active_run
                    .as_ref()
                    .and_then(|active_run| active_run.update(phase, status_text));
                let payload = if let Some(run) = run {
                    json!({
                        "run_id": run.run_id,
                        "started_at_ms": run.started_at_ms,
                        "updated_at_ms": run.updated_at_ms,
                        "phase": run.phase,
                        "status_text": run.status_text,
                    })
                } else {
                    json!({
                        "phase": phase,
                        "status_text": status_text,
                    })
                };
                let _ = self.tx.send(("run_progress".into(), payload)).await;
            }
            AgentSessionEvent::Run(RunEvent::Error { error }) => {
                let mut i = error.message.len().min(120);
                while i > 0 && !error.message.is_char_boundary(i) {
                    i -= 1;
                }
                let snippet = &error.message[..i];
                let _ = self
                    .tx
                    .send((
                        "run_error".into(),
                        json!({ "message": format!("抱歉，处理出错: {snippet}") }),
                    ))
                    .await;
            }
            AgentSessionEvent::Done { response } => {
                if self.terminal_sent.swap(true, Ordering::AcqRel) {
                    return;
                }
                if let Some(active_run) = &self.active_run {
                    // The assistant turn is persisted before Done is emitted.
                    // Clear recovery state before any terminal bytes reach the
                    // browser so a concurrent refresh cannot append a second
                    // thinking card behind the completed answer.
                    active_run.finish();
                }
                let sent = *self.sent_segments.lock().await;
                // ── 安全刷新：仅当流式阶段完全没有发送过内容时，才补发全量，
                // 防止 SSE 连接建立前丢失第一帧。若已发过内容则跳过，避免重复渲染。
                if response.success && sent == 0 {
                    let cleaned = clean_msg_markers(&response.content);
                    if !cleaned.is_empty() && !should_skip_buffer(&cleaned) {
                        let _ = self
                            .tx
                            .send(("assistant_delta".into(), json!({ "content": cleaned })))
                            .await;
                    }
                }
                if !response.success {
                    error!(
                        "[Console] [{}] 处理失败: {}",
                        self.user_id,
                        response
                            .error
                            .clone()
                            .unwrap_or_else(|| "未知错误".to_string())
                    );
                }
                let _ = self
                    .tx
                    .send((
                        "run_finished".into(),
                        json!({ "success": response.success }),
                    ))
                    .await;
            }
            _ => {}
        }
    }
}

fn emit_web_progress_heartbeat(
    tx: &tokio::sync::mpsc::Sender<(String, Value)>,
    active_run: &ActiveChatRunHandle,
    terminal_sent: &AtomicBool,
    min_silence: Duration,
) {
    if terminal_sent.load(Ordering::Acquire) {
        return;
    }
    let Some(run) = active_run.heartbeat("running", WEB_CHAT_PROGRESS_STATUS, min_silence) else {
        return;
    };
    // A slow or disconnected browser must never stall the detached Agent run.
    let _ = tx.try_send((
        "run_progress".into(),
        json!({
            "run_id": run.run_id,
            "started_at_ms": run.started_at_ms,
            "updated_at_ms": run.updated_at_ms,
            "phase": run.phase,
            "status_text": run.status_text,
        }),
    ));
}

fn public_progress_status(stage: &str) -> (&'static str, &'static str) {
    match stage {
        "session.compress" => ("thinking", "正在整理会话上下文"),
        "entity_resolution.preflight" => ("running", "正在识别当前问题中的公司或证券实体"),
        "entity_resolution.preflight.done" => ("running", "实体范围已确认，正在整理回答"),
        "entity_resolution.preflight.failed" => {
            ("running", "实体或数据核验未完成，正在安全结束本轮")
        }
        "market_data.preflight.done" => ("running", "实体与行情已核验，正在生成完整分析"),
        "agent.run.retry" => ("running", "正在复核结果，确保信息准确"),
        "agent.run.progress" => ("running", "仍在处理中，正在完成核验与分析"),
        "agent.run" => ("running", "正在检索、核验并生成回答"),
        _ => ("thinking", "正在准备并核验所需信息"),
    }
}

fn public_tool_status_text(status: &str) -> &'static str {
    match status.trim().to_ascii_lowercase().as_str() {
        "completed" | "complete" | "success" | "succeeded" | "finished" | "done" => {
            "数据核验完成，正在组织回答"
        }
        _ => "正在查询并核验所需数据",
    }
}

pub(crate) fn build_chat_sse(
    state: Arc<AppState>,
    actor_result: Result<ActorIdentity, hone_core::HoneError>,
    message: String,
    attachments_count: usize,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // mpsc channel 连接 spawn task ↔ SSE stream
    let (tx, rx) = tokio::sync::mpsc::channel::<(String, Value)>(64);

    let arc = state.clone();
    let msg = message.clone();
    let att_count = attachments_count;
    // 在返回 SSE 之前登记，避免用户刚发送就刷新时出现“任务不存在”的竞态。
    let active_run_guard_result = actor_result
        .as_ref()
        .ok()
        .map(|actor| state.active_chat_runs.try_begin(actor.session_id()));

    tokio::spawn(async move {
        let actor_clone = match actor_result {
            Ok(actor) => actor,
            Err(error) => {
                let _ = tx
                    .send(("error".into(), json!({ "text": error.to_string() })))
                    .await;
                let _ = tx.send(("done".into(), json!({}))).await;
                return;
            }
        };
        let active_run_guard = match active_run_guard_result {
            Some(Ok(guard)) => guard,
            Some(Err(_)) => {
                let _ = tx
                    .send((
                        "run_error".into(),
                        json!({ "message": "当前会话已有任务正在处理，请稍后查看" }),
                    ))
                    .await;
                let _ = tx
                    .send(("run_finished".into(), json!({ "success": false })))
                    .await;
                return;
            }
            None => unreachable!("a valid actor always has a pre-registered chat run"),
        };
        let active_run = active_run_guard
            .run()
            .expect("active chat run must exist while its guard is alive");
        let run_id = active_run.run_id.clone();

        // 立即发送 ack；时间与任务 id 来自服务端，页面刷新后可稳定恢复。
        let _ = tx
            .send((
                "run_started".into(),
                json!({
                    "runner": arc.core.config.agent.runner,
                    "text": "",
                    "run_id": active_run.run_id,
                    "started_at_ms": active_run.started_at_ms,
                    "updated_at_ms": active_run.updated_at_ms,
                    "phase": active_run.phase,
                    "status_text": active_run.status_text,
                }),
            ))
            .await;

        if let Some(reply) = arc
            .core
            .try_handle_intercept_command(&actor_clone, &msg)
            .await
        {
            let _ = tx
                .send(("assistant_delta".into(), json!({ "content": reply })))
                .await;
            // Do not expose a still-active run after the terminal frame. This
            // is the same ordering guarantee used by the session listener.
            drop(active_run_guard);
            let _ = tx
                .send(("run_finished".into(), json!({ "success": true })))
                .await;
            return;
        }

        let recv_extra = format!("attachments={att_count}");
        let prompt_options = PromptOptions {
            is_admin: arc.core.is_admin_actor(&actor_clone),
            ..PromptOptions::default()
        };

        let mut session = AgentSession::new(
            arc.core.clone(),
            actor_clone.clone(),
            actor_clone.user_id.clone(),
        )
        .with_restore_max_messages(None)
        .with_message_id(Some(run_id))
        .with_prompt_options(prompt_options)
        .with_recv_extra(Some(recv_extra));

        let sent_segments = Arc::new(tokio::sync::Mutex::new(0usize));
        let terminal_sent = Arc::new(AtomicBool::new(false));
        let active_run_handle = active_run_guard.handle();
        session.add_listener(Arc::new(SseSessionListener {
            tx: tx.clone(),
            user_id: actor_clone.user_id.clone(),
            sent_segments: sent_segments.clone(),
            active_run: Some(active_run_handle.clone()),
            terminal_sent: terminal_sent.clone(),
        }));

        info!(
            channel = %actor_clone.channel,
            attachments = att_count,
            message_len = msg.chars().count(),
            "[Console] 收到消息"
        );
        eprintln!("[Console] 收到消息，开始处理...");

        let run_options = AgentRunOptions {
            timeout: Some(state.core.config.agent.overall_timeout()),
            segmenter: None,
            quota_mode: AgentRunQuotaMode::UserConversation,
            model_override: None,
            ..AgentRunOptions::default()
        };
        let heartbeat_tx = tx.clone();
        let heartbeat_terminal = terminal_sent.clone();
        let _ = run_with_progress_ticks(
            session.run(&msg, run_options),
            WEB_CHAT_PROGRESS_TICK,
            move |_, _| {
                emit_web_progress_heartbeat(
                    &heartbeat_tx,
                    &active_run_handle,
                    &heartbeat_terminal,
                    WEB_CHAT_PROGRESS_MIN_SILENCE,
                );
                std::future::ready(())
            },
        )
        .await;
        drop(active_run_guard);
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(|(event, data)| {
        let data_str = serde_json::to_string(&data).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().event(event).data(data_str))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// POST /api/chat — 接收消息，以 SSE 流式返回 Agent 响应
pub(crate) async fn handle_chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let actor_result = hone_channels::HoneBotCore::create_actor(
        req.channel.trim(),
        req.user_id.trim(),
        req.channel_scope.as_deref(),
    );
    let mut message = req.message.unwrap_or_default().trim().to_string();
    let mut attachments_count = 0usize;

    if let Some(attachments) = req.attachments {
        attachments_count = attachments.len();
        if !attachments.is_empty() {
            let att = attachments
                .iter()
                .map(|a| format!("[附件: {a}]"))
                .collect::<Vec<_>>()
                .join("\n");
            message = if message.is_empty() {
                att
            } else {
                format!("{message}\n{att}")
            };
        }
    }

    build_chat_sse(state, actor_result, message, attachments_count)
}

/// 部署脚本在终止进程前轮询此端点，避免把仍在生成的用户请求直接杀掉。
pub(crate) async fn handle_active_chat_runs(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "count": state.active_chat_runs.count() }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use hone_channels::agent_session::{AgentSessionEvent, AgentSessionListener};
    use hone_channels::run_event::RunEvent;
    use hone_core::agent::AgentResponse;
    use serde_json::json;

    use crate::state::ActiveChatRunRegistry;

    use super::{SseSessionListener, emit_web_progress_heartbeat};

    #[tokio::test]
    async fn stream_reset_clears_sent_count_and_emits_assistant_reset() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);
        let sent_segments = Arc::new(tokio::sync::Mutex::new(1));
        let listener = SseSessionListener {
            tx,
            user_id: "u1".to_string(),
            sent_segments: sent_segments.clone(),
            active_run: None,
            terminal_sent: Arc::new(AtomicBool::new(false)),
        };

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::StreamReset))
            .await;

        let (event, _) = rx.recv().await.expect("reset event");
        assert_eq!(event, "assistant_reset");
        assert_eq!(*sent_segments.lock().await, 0);
    }

    #[tokio::test]
    async fn final_stream_delta_and_done_emit_one_delta_and_one_run_finished() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);
        let sent_segments = Arc::new(tokio::sync::Mutex::new(0));
        let listener = SseSessionListener {
            tx,
            user_id: "u1".to_string(),
            sent_segments: sent_segments.clone(),
            active_run: None,
            terminal_sent: Arc::new(AtomicBool::new(false)),
        };

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::StreamDelta {
                content: "最终答案".to_string(),
            }))
            .await;
        listener
            .on_event(AgentSessionEvent::Done {
                response: AgentResponse {
                    content: "最终答案".to_string(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: true,
                    error: None,
                },
            })
            .await;

        assert_eq!(
            rx.recv().await,
            Some((
                "assistant_delta".to_string(),
                json!({ "content": "最终答案" })
            ))
        );
        assert_eq!(
            rx.recv().await,
            Some(("run_finished".to_string(), json!({ "success": true })))
        );
        assert!(
            rx.try_recv().is_err(),
            "unexpected duplicate or reset event"
        );
        assert_eq!(*sent_segments.lock().await, 1);
    }

    #[tokio::test]
    async fn failed_done_emits_one_run_finished() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        let listener = SseSessionListener {
            tx,
            user_id: "u1".to_string(),
            sent_segments: Arc::new(tokio::sync::Mutex::new(0)),
            active_run: None,
            terminal_sent: Arc::new(AtomicBool::new(false)),
        };

        listener
            .on_event(AgentSessionEvent::Done {
                response: AgentResponse {
                    content: "失败兜底文案不得作为第二轮正文补发".to_string(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: false,
                    error: Some("runner failed".to_string()),
                },
            })
            .await;

        assert_eq!(
            rx.recv().await,
            Some(("run_finished".to_string(), json!({ "success": false })))
        );
        assert!(
            rx.try_recv().is_err(),
            "unexpected duplicate terminal event"
        );
    }

    #[tokio::test]
    async fn terminal_event_finishes_recovery_state_and_blocks_late_frames() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-1".to_string())
            .expect("active run");
        let terminal_sent = Arc::new(AtomicBool::new(false));
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);
        let listener = SseSessionListener {
            tx,
            user_id: "u1".to_string(),
            // Pretend the answer was already streamed so Done only emits the
            // authoritative terminal frame.
            sent_segments: Arc::new(tokio::sync::Mutex::new(1)),
            active_run: Some(guard.handle()),
            terminal_sent: terminal_sent.clone(),
        };

        listener
            .on_event(AgentSessionEvent::Done {
                response: AgentResponse {
                    content: "最终答案".to_string(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: true,
                    error: None,
                },
            })
            .await;

        assert_eq!(registry.count(), 0, "refresh must not see a stale run");
        assert!(terminal_sent.load(Ordering::Acquire));
        assert_eq!(
            rx.recv().await,
            Some(("run_finished".to_string(), json!({ "success": true })))
        );

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::Progress {
                stage: "agent.run.progress",
                detail: None,
            }))
            .await;
        listener
            .on_event(AgentSessionEvent::Done {
                response: AgentResponse {
                    content: "重复答案".to_string(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: true,
                    error: None,
                },
            })
            .await;
        assert!(rx.try_recv().is_err(), "late frames must be suppressed");

        // The still-live guard can now drop without resurrecting/removing a
        // different run.
        drop(guard);
    }

    #[tokio::test]
    async fn web_heartbeat_preserves_server_start_and_stops_after_terminal() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-1".to_string())
            .expect("active run");
        let initial = guard.run().expect("initial run");
        let specific = guard
            .handle()
            .update("running", "正在识别当前问题中的公司或证券实体")
            .expect("specific progress stage");
        let terminal_sent = AtomicBool::new(false);
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);

        emit_web_progress_heartbeat(&tx, &guard.handle(), &terminal_sent, Duration::ZERO);
        let (event, payload) = rx.recv().await.expect("heartbeat event");
        assert_eq!(event, "run_progress");
        assert_eq!(payload["run_id"], initial.run_id);
        assert_eq!(payload["started_at_ms"], initial.started_at_ms);
        assert_eq!(payload["status_text"], "正在识别当前问题中的公司或证券实体");
        assert!(payload["updated_at_ms"].as_i64().unwrap() >= specific.updated_at_ms);

        terminal_sent.store(true, Ordering::Release);
        emit_web_progress_heartbeat(&tx, &guard.handle(), &terminal_sent, Duration::ZERO);
        assert!(rx.try_recv().is_err(), "terminal run must not heartbeat");
    }

    #[tokio::test]
    async fn preflight_progress_uses_safe_labels_and_full_active_run_snapshot() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-preflight".to_string())
            .expect("active run");
        let initial = guard.run().expect("initial run");
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);
        let listener = SseSessionListener {
            tx,
            user_id: "u1".to_string(),
            sent_segments: Arc::new(tokio::sync::Mutex::new(0)),
            active_run: Some(guard.handle()),
            terminal_sent: Arc::new(AtomicBool::new(false)),
        };

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::Progress {
                stage: "entity_resolution.preflight",
                detail: Some("raw provider payload must never be exposed".to_string()),
            }))
            .await;
        let (event, payload) = rx.recv().await.expect("preflight progress");
        assert_eq!(event, "run_progress");
        assert_eq!(payload["run_id"], initial.run_id);
        assert_eq!(payload["started_at_ms"], initial.started_at_ms);
        assert_eq!(payload["phase"], "running");
        assert_eq!(payload["status_text"], "正在识别当前问题中的公司或证券实体");
        assert!(payload["updated_at_ms"].as_i64().is_some());
        assert!(!payload.to_string().contains("raw provider payload"));

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::Progress {
                stage: "entity_resolution.preflight.done",
                detail: None,
            }))
            .await;
        let (_, payload) = rx.recv().await.expect("entity resolution completion");
        assert_eq!(payload["status_text"], "实体范围已确认，正在整理回答");

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::Progress {
                stage: "market_data.preflight.done",
                detail: None,
            }))
            .await;
        let (_, payload) = rx.recv().await.expect("preflight completion");
        assert_eq!(payload["status_text"], "实体与行情已核验，正在生成完整分析");

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::Progress {
                stage: "entity_resolution.preflight.failed",
                detail: Some("internal provider details".to_string()),
            }))
            .await;
        let (_, payload) = rx.recv().await.expect("preflight failure");
        assert_eq!(
            payload["status_text"],
            "实体或数据核验未完成，正在安全结束本轮"
        );
        assert!(!payload.to_string().contains("internal provider details"));
    }

    #[tokio::test]
    async fn progress_and_tool_events_emit_only_user_safe_status_fields() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);
        let listener = SseSessionListener {
            tx,
            user_id: "u1".to_string(),
            sent_segments: Arc::new(tokio::sync::Mutex::new(0)),
            active_run: None,
            terminal_sent: Arc::new(AtomicBool::new(false)),
        };

        listener
            .on_event(AgentSessionEvent::Run(RunEvent::Progress {
                stage: "agent.run.progress",
                detail: Some("internal runner detail must not be exposed".to_string()),
            }))
            .await;
        listener
            .on_event(AgentSessionEvent::Run(RunEvent::ToolStatus {
                tool: "data_fetch".to_string(),
                status: "running".to_string(),
                message: Some("raw provider detail".to_string()),
                reasoning: Some("internal reasoning".to_string()),
            }))
            .await;

        assert_eq!(
            rx.recv().await,
            Some((
                "run_progress".to_string(),
                json!({
                    "phase": "running",
                    "status_text": "仍在处理中，正在完成核验与分析",
                })
            ))
        );
        let (event, payload) = rx.recv().await.expect("tool status event");
        assert_eq!(event, "tool_call");
        assert_eq!(
            payload
                .get("public_status_text")
                .and_then(|value| value.as_str()),
            Some("正在查询并核验所需数据")
        );
    }
}
