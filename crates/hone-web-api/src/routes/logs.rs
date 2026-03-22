use std::convert::Infallible;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::state::AppState;

/// GET /api/logs — 返回最近的日志条目（最多 500 条）
pub(crate) async fn handle_logs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let log_buffer = state.log_buffer.clone();
    let logs = {
        let buf = log_buffer.buffer.lock().unwrap();
        let len = buf.len();
        let start = len.saturating_sub(500);
        buf.iter().skip(start).cloned().collect::<Vec<_>>()
    };
    Json(serde_json::json!({ "logs": logs }))
}

/// GET /api/logs/stream — SSE 实时日志流
pub(crate) async fn handle_logs_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let log_buffer = state.log_buffer.clone();
    let rx = log_buffer.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(entry) => {
            let data = serde_json::to_string(&entry).unwrap_or_default();
            Some(Ok(Event::default().event("log").data(data)))
        }
        Err(_) => None,
    });

    // 先发送 connected 确认事件
    let init = tokio_stream::iter(vec![Ok::<_, Infallible>(
        Event::default().event("connected").data("{}"),
    )]);

    Sse::new(init.chain(stream)).keep_alive(KeepAlive::default())
}
