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
        let actor = session
            .actor
            .clone()
            .or_else(|| hone_core::ActorIdentity::from_session_id(&session_id));
        let session_identity = session
            .session_identity
            .clone()
            .or_else(|| {
                actor
                    .as_ref()
                    .and_then(|value| hone_core::SessionIdentity::from_actor(value).ok())
            })
            .or_else(|| hone_core::SessionIdentity::from_session_id(&session_id));
        let Some(identity) = session_identity else {
            continue;
        };

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use hone_memory::session::Session;

    fn empty_session(id: &str) -> Session {
        Session {
            version: 4,
            id: id.to_string(),
            actor: None,
            session_identity: None,
            created_at: "2026-04-16T09:00:00+08:00".to_string(),
            updated_at: "2026-04-16T09:05:00+08:00".to_string(),
            messages: Vec::new(),
            metadata: HashMap::new(),
            runtime: Default::default(),
            summary: None,
        }
    }

    #[test]
    fn actor_session_id_is_enough_for_listing_identity() {
        let session = empty_session("Actor_feishu__direct__ou_5f123");
        let actor = session
            .actor
            .clone()
            .or_else(|| hone_core::ActorIdentity::from_session_id(&session.id))
            .expect("actor");
        let identity = session
            .session_identity
            .clone()
            .or_else(|| hone_core::SessionIdentity::from_actor(&actor).ok())
            .or_else(|| hone_core::SessionIdentity::from_session_id(&session.id))
            .expect("identity");

        assert_eq!(actor.channel, "feishu");
        assert_eq!(actor.user_id, "ou_123");
        assert_eq!(identity.channel, "feishu");
        assert_eq!(identity.user_id.as_deref(), Some("ou_123"));
        assert_eq!(identity.channel_scope, None);
    }

    #[test]
    fn shared_group_session_id_is_enough_for_listing_identity() {
        let session = empty_session("Session_discord__group__g_3a1_3ac_3a2");
        let identity = session
            .session_identity
            .clone()
            .or_else(|| hone_core::SessionIdentity::from_session_id(&session.id))
            .expect("identity");

        assert_eq!(identity.channel, "discord");
        assert_eq!(identity.channel_scope.as_deref(), Some("g:1:c:2"));
        assert!(identity.is_group());
    }
}
