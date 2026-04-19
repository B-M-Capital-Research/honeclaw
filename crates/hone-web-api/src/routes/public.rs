use std::convert::Infallible;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response, sse::Event, sse::KeepAlive, sse::Sse};
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use hone_core::ActorIdentity;

use crate::routes::chat::build_chat_sse;
use crate::routes::history::history_from_messages;
use crate::state::{AppState, PushEvent};
use crate::types::{PublicAuthUserInfo, PublicChatRequest, PublicInviteLoginRequest};

const WEB_SESSION_COOKIE: &str = "hone_web_session";
const WEB_SESSION_MAX_AGE_SECS: i64 = 30 * 24 * 60 * 60;

pub(crate) async fn handle_invite_login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PublicInviteLoginRequest>,
) -> Response {
    let invite_code = request
        .invite_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(invite_code) = invite_code else {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "缺少邀请码");
    };

    match state.web_auth.create_session_for_invite(invite_code) {
        Ok(Some(session)) => match state.web_auth.find_invite_user(&session.user_id) {
            Ok(Some(user)) => {
                let user_id = user.user_id.clone();
                let mut response = Json(json!({
                    "user": to_public_auth_user(&state, &user_id, user),
                }))
                .into_response();
                response.headers_mut().append(
                    header::SET_COOKIE,
                    build_session_cookie(&session.session_token),
                );
                response
            }
            Ok(None) => {
                crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, "邀请码用户不存在")
            }
            Err(error) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取邀请码用户失败: {error}"),
            ),
        },
        Ok(None) => crate::routes::json_error(StatusCode::UNAUTHORIZED, "邀请码无效"),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("邀请码登录失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(token) = read_session_token(&headers) {
        let _ = state.web_auth.delete_session(&token);
    }

    let mut response = Json(json!({ "ok": true })).into_response();
    response
        .headers_mut()
        .append(header::SET_COOKIE, clear_session_cookie());
    response
}

pub(crate) async fn handle_me(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    match require_public_user(&state, &headers) {
        Ok(user) => {
            let user_id = user.user_id.clone();
            Json(json!({
                "user": to_public_auth_user(&state, &user_id, user),
            }))
        }
        .into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn handle_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()),
    };
    let messages = state
        .core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .unwrap_or_default();

    Json(json!({
        "messages": history_from_messages(&messages),
    }))
    .into_response()
}

pub(crate) async fn handle_chat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicChatRequest>,
) -> Response {
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()),
    };
    let message = request.message.unwrap_or_default().trim().to_string();
    if message.is_empty() {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "消息不能为空");
    }

    build_chat_sse(state, Ok(actor), message, 0).into_response()
}

pub(crate) async fn handle_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()),
    };
    let rx = state.push_tx.subscribe();
    let stream =
        BroadcastStream::new(rx).filter_map(move |msg| filter_public_push(actor.clone(), msg));
    let init = tokio_stream::iter(vec![Ok::<_, Infallible>(
        Event::default().event("connected").data("{}"),
    )]);

    Sse::new(init.chain(stream))
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn filter_public_push(
    actor: ActorIdentity,
    msg: Result<PushEvent, tokio_stream::wrappers::errors::BroadcastStreamRecvError>,
) -> Option<Result<Event, Infallible>> {
    match msg {
        Ok(event)
            if event.channel == actor.channel
                && event.user_id == actor.user_id
                && event.channel_scope == actor.channel_scope =>
        {
            let data = serde_json::to_string(&event.data).unwrap_or_else(|_| "{}".to_string());
            Some(Ok(Event::default().event(event.event).data(data)))
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
            Some(Ok(Event::default()
                .event("events_lagged")
                .data(json!({ "skipped": n }).to_string())))
        }
        _ => None,
    }
}

fn require_public_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<hone_memory::WebInviteUser, Response> {
    let token = read_session_token(headers)
        .ok_or_else(|| crate::routes::json_error(StatusCode::UNAUTHORIZED, "未登录"))?;
    match state.web_auth.authenticate_session(&token) {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(crate::routes::json_error(
            StatusCode::UNAUTHORIZED,
            "登录已过期，请重新输入邀请码",
        )),
        Err(error) => Err(crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("验证登录态失败: {error}"),
        )),
    }
}

fn read_session_token(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    cookies.split(';').find_map(|item| {
        let trimmed = item.trim();
        let (name, value) = trimmed.split_once('=')?;
        if name == WEB_SESSION_COOKIE {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn build_session_cookie(session_token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{WEB_SESSION_COOKIE}={session_token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={WEB_SESSION_MAX_AGE_SECS}"
    ))
    .expect("valid session cookie")
}

fn clear_session_cookie() -> HeaderValue {
    HeaderValue::from_static("hone_web_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

fn to_public_auth_user(
    state: &AppState,
    user_id: &str,
    user: hone_memory::WebInviteUser,
) -> PublicAuthUserInfo {
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

    PublicAuthUserInfo {
        user_id: user.user_id,
        created_at: user.created_at,
        last_login_at: user.last_login_at,
        daily_limit,
        success_count,
        in_flight,
        remaining_today,
    }
}
