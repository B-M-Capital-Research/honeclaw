use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use hone_memory::session_message_text;

use crate::state::AppState;
use crate::types::UserInfo;

/// GET /api/users — 列出所有有会话记录的 session，按最后活跃时间降序
pub(crate) async fn handle_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut users: Vec<UserInfo> = Vec::new();
    let sessions = match state.core.session_storage.list_sessions() {
        Ok(sessions) => sessions,
        Err(_) => return Json(serde_json::json!([])),
    };

    for session in sessions {
        let session_id = session.id.clone();
        let session_identity = session.session_identity.clone().or_else(|| {
            session
                .actor
                .as_ref()
                .and_then(|actor| hone_core::SessionIdentity::from_actor(actor).ok())
        });
        let Some(identity) = session_identity else {
            continue;
        };
        let actor = session.actor.clone();

        // 取最后一条 user 或 assistant 消息作为预览
        let last_msg = session
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role.as_str(), "user" | "assistant"));

        let (last_message, last_role, last_time) = match last_msg {
            Some(m) => {
                let content = session_message_text(m);
                let preview: String = content.chars().take(60).collect();
                let preview = if content.chars().count() > 60 {
                    format!("{}…", preview)
                } else {
                    preview
                };
                (preview, m.role.clone(), m.timestamp.clone())
            }
            None => (
                "暂无消息".to_string(),
                "".to_string(),
                session.updated_at.clone(),
            ),
        };

        let message_count = session
            .messages
            .iter()
            .filter(|m| matches!(m.role.as_str(), "user" | "assistant"))
            .count();

        users.push(UserInfo {
            channel: identity.channel.clone(),
            user_id: actor
                .as_ref()
                .map(|value| value.user_id.clone())
                .or_else(|| identity.user_id.clone())
                .unwrap_or_else(|| "group".to_string()),
            channel_scope: identity.channel_scope.clone(),
            session_id,
            session_kind: if identity.is_group() {
                "group".to_string()
            } else {
                "direct".to_string()
            },
            session_label: if identity.is_group() {
                identity
                    .channel_scope
                    .clone()
                    .unwrap_or_else(|| "群聊".to_string())
            } else {
                actor
                    .as_ref()
                    .map(|value| value.user_id.clone())
                    .or_else(|| identity.user_id.clone())
                    .unwrap_or_else(|| "direct".to_string())
            },
            actor_user_id: actor.as_ref().map(|value| value.user_id.clone()),
            last_message,
            last_role,
            last_time,
            message_count,
        });
    }

    // 按最后时间降序
    users.sort_by(|a, b| b.last_time.cmp(&a.last_time));

    Json(serde_json::to_value(&users).unwrap_or(serde_json::json!([])))
}
