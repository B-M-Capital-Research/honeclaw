use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

use hone_core::ActorIdentity;

use crate::state::AppState;
use crate::types::WebInviteInfo;

pub(crate) async fn handle_list_invites(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let invites = state
        .web_auth
        .list_invite_users()
        .unwrap_or_default()
        .into_iter()
        .map(|invite| {
            let user_id = invite.user_id.clone();
            to_invite_info(&state, &user_id, invite)
        })
        .collect::<Vec<_>>();

    Json(json!({ "invites": invites }))
}

pub(crate) async fn handle_create_invite(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.web_auth.create_invite_user() {
        Ok(invite) => {
            let user_id = invite.user_id.clone();
            Json(json!({ "invite": to_invite_info(&state, &user_id, invite) })).into_response()
        }
        Err(error) => crate::routes::json_error(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("生成邀请码失败: {error}"),
        ),
    }
}

fn to_invite_info(
    state: &AppState,
    user_id: &str,
    invite: hone_memory::WebInviteUser,
) -> WebInviteInfo {
    let actor = ActorIdentity::new("web", user_id, Option::<String>::None).ok();
    let daily_limit = state.core.config.agent.daily_conversation_limit;
    let quota_date = hone_core::beijing_now().format("%F").to_string();
    let snapshot = actor.as_ref().and_then(|actor| {
        state
            .core
            .conversation_quota_storage
            .snapshot_for_date(actor, &quota_date)
            .ok()
            .flatten()
    });
    let success_count = snapshot
        .as_ref()
        .map(|value| value.success_count)
        .unwrap_or(0);
    let in_flight = snapshot.as_ref().map(|value| value.in_flight).unwrap_or(0);
    let remaining_today = if daily_limit == 0 {
        0
    } else {
        daily_limit.saturating_sub(success_count.saturating_add(in_flight))
    };

    WebInviteInfo {
        user_id: invite.user_id,
        invite_code: invite.invite_code,
        created_at: invite.created_at,
        last_login_at: invite.last_login_at,
        daily_limit,
        success_count,
        in_flight,
        remaining_today,
    }
}
