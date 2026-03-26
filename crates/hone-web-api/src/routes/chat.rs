use std::convert::Infallible;
use std::sync::Arc;
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
};
use hone_channels::prompt::PromptOptions;
use hone_channels::runtime::{clean_msg_markers, should_skip_buffer};

use crate::state::AppState;
use crate::types::ChatRequest;

struct SseSessionListener {
    tx: tokio::sync::mpsc::Sender<(String, Value)>,
    user_id: String,
    sent_segments: Arc<tokio::sync::Mutex<usize>>,
}

#[async_trait]
impl AgentSessionListener for SseSessionListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        match event {
            AgentSessionEvent::Segment { text } => {
                let _ = self
                    .tx
                    .send(("assistant_delta".into(), json!({ "content": text })))
                    .await;
                let mut guard = self.sent_segments.lock().await;
                *guard += 1;
            }
            AgentSessionEvent::StreamDelta { content } => {
                let _ = self
                    .tx
                    .send(("assistant_delta".into(), json!({ "content": content })))
                    .await;
                let mut guard = self.sent_segments.lock().await;
                *guard += 1;
            }
            AgentSessionEvent::ToolStatus {
                reasoning, message, ..
            } => {
                let payload = json!({
                    "tool": message,
                    "status": "update",
                    "text": message,
                    "reasoning": reasoning,
                });
                let _ = self.tx.send(("tool_call".into(), payload)).await;
            }
            AgentSessionEvent::Error { error } => {
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
                let sent = *self.sent_segments.lock().await;
                // ── 安全刷新：如果推送的增量很少（可能是握手或元数据），则强制刷新全量内容 ──
                // 这解决了流式开启瞬间可能存在的第一个字丢失的问题
                if sent < 3 {
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

    // mpsc channel 连接 spawn task ↔ SSE stream
    let (tx, rx) = tokio::sync::mpsc::channel::<(String, Value)>(64);

    let arc = state.clone();
    let msg = message.clone();
    let att_count = attachments_count;

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
        if arc
            .core
            .try_intercept_admin_registration(&actor_clone, &msg)
        {
            let _ = tx
                .send((
                    "assistant_delta".into(),
                    json!({ "content": hone_channels::core::REGISTER_ADMIN_INTERCEPT_ACK }),
                ))
                .await;
            let _ = tx.send(("done".into(), json!({}))).await;
            return;
        }
        // 立即发送 ack (使用极简文本，避免与 AI 回复的第一句冲突)
        let _ = tx
            .send((
                "run_started".into(),
                json!({
                    "runner": arc.core.config.agent.runner,
                    "text": ""
                }),
            ))
            .await;

        let _session_id = actor_clone.session_id();
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
        .with_prompt_options(prompt_options)
        .with_recv_extra(Some(recv_extra));

        let sent_segments = Arc::new(tokio::sync::Mutex::new(0usize));
        session.add_listener(Arc::new(SseSessionListener {
            tx: tx.clone(),
            user_id: actor_clone.user_id.clone(),
            sent_segments: sent_segments.clone(),
        }));

        info!("[Console] [{}] 收到: {}", actor_clone.user_id, msg);
        eprintln!("[Console] [{}] 收到消息，开始处理...", actor_clone.user_id);

        let run_options = AgentRunOptions {
            timeout: Some(Duration::from_secs(300)),
            segmenter: None,
            quota_mode: AgentRunQuotaMode::UserConversation,
            model_override: None,
        };
        let result = session.run(&msg, run_options).await;
        if !result.response.success {
            let _ = tx
                .send(("run_finished".into(), json!({ "success": false })))
                .await;
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(|(event, data)| {
        let data_str = serde_json::to_string(&data).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().event(event).data(data_str))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
