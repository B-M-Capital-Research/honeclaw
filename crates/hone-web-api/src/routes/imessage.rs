use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use tracing::info;

use crate::state::{AppState, IMessageEventRequest, PushEvent};

/// POST /api/imessage-event — 接收 iMessage Bot 推送的实时事件并广播到 SSE
///
/// iMessage Bot 在以下时机调用此接口：
/// - 收到用户消息（event_type = "imessage_user_message"）
/// - 开始处理（event_type = "imessage_processing_start"）
/// - 处理完成有回复（event_type = "imessage_assistant_message"）
/// - 处理失败（event_type = "imessage_processing_error"）
pub(crate) async fn handle_imessage_event(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IMessageEventRequest>,
) -> impl IntoResponse {
    // 记录到日志系统（会出现在控制台日志面板）
    info!(
        "[iMessage→Console] user={} event={} data={}",
        req.user_id,
        req.event_type,
        serde_json::to_string(&req.data).unwrap_or_default()
    );

    // 广播到 SSE 订阅者（只有正在查看该会话的用户会收到）
    let _ = state.push_tx.send(PushEvent {
        channel: req.channel,
        user_id: req.user_id,
        channel_scope: req.channel_scope,
        event: req.event_type,
        data: req.data,
    });

    (StatusCode::OK, Json(json!({ "ok": true })))
}
