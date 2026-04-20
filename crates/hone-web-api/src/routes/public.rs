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

use crate::public_auth::PublicAuthLimitStatus;
use crate::routes::chat::build_chat_sse;
use crate::routes::history::history_from_messages;
use crate::state::{AppState, PushEvent};
use crate::types::{PublicAuthUserInfo, PublicChatRequest, PublicInviteLoginRequest};

const WEB_SESSION_COOKIE: &str = "hone_web_session";
const WEB_SESSION_MAX_AGE_SECS: i64 = 30 * 24 * 60 * 60;

pub(crate) async fn handle_invite_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicInviteLoginRequest>,
) -> Response {
    let invite_code = request
        .invite_code
        .as_deref()
        .map(normalize_invite_code)
        .filter(|value| !value.is_empty());
    let Some(invite_code) = invite_code else {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "缺少邀请码");
    };
    let phone_number = match crate::routes::require_phone_number(request.phone_number, "手机号")
    {
        Ok(phone_number) => phone_number,
        Err(response) => return response,
    };

    // Rate-limit by phone number (unforgeable) + IP (best-effort).
    // Even if an attacker spoofs X-Forwarded-For, the phone_number dimension
    // still prevents brute-forcing a single invite code.
    let ip_key = public_client_key(&headers);
    let phone_key = format!("phone:{phone_number}");
    for key in [&ip_key, &phone_key] {
        if let PublicAuthLimitStatus::Blocked { retry_after_secs } =
            state.public_auth_limiter.check(key)
        {
            return json_rate_limited(retry_after_secs);
        }
    }

    match state
        .web_auth
        .create_session_for_invite(&invite_code, &phone_number)
    {
        Ok(Some(session)) => {
            state.public_auth_limiter.record_success(&ip_key);
            state.public_auth_limiter.record_success(&phone_key);
            match state.web_auth.find_invite_user(&session.user_id) {
                Ok(Some(user)) => {
                    let user_id = user.user_id.clone();
                    let mut response = Json(json!({
                        "user": to_public_auth_user(&state, &user_id, user),
                    }))
                    .into_response();
                    response.headers_mut().append(
                        header::SET_COOKIE,
                        build_session_cookie(&session.session_token, &headers),
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
            }
        }
        Ok(None) => {
            let mut rate_limited_response = None;
            for key in [&ip_key, &phone_key] {
                if let Some(retry_after_secs) = state.public_auth_limiter.record_failure(key) {
                    rate_limited_response = Some(json_rate_limited(retry_after_secs));
                }
            }
            if let Some(response) = rate_limited_response {
                response
            } else {
                crate::routes::json_error(
                    StatusCode::UNAUTHORIZED,
                    "邀请码或手机号不正确，或邀请码已失效",
                )
            }
        }
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
        .append(header::SET_COOKIE, clear_session_cookie(&headers));
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

fn build_session_cookie(session_token: &str, headers: &HeaderMap) -> HeaderValue {
    let secure_attr = if request_is_secure(headers) {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{WEB_SESSION_COOKIE}={session_token}; Path=/; HttpOnly; SameSite=Strict{secure_attr}; Max-Age={WEB_SESSION_MAX_AGE_SECS}"
    ))
    .expect("valid session cookie")
}

fn clear_session_cookie(headers: &HeaderMap) -> HeaderValue {
    let secure_attr = if request_is_secure(headers) {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{WEB_SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Strict{secure_attr}; Max-Age=0"
    ))
    .expect("valid clear session cookie")
}

/// Determine whether the Secure flag should be set on session cookies.
///
/// Checks `HONE_PUBLIC_SECURE_COOKIE` env var first (accepts "true"/"1" to
/// force-enable, "false"/"0" to force-disable). Falls back to inspecting
/// request headers when the env var is absent or empty.
fn request_is_secure(headers: &HeaderMap) -> bool {
    if let Some(forced) = env_force_secure_cookie() {
        return forced;
    }
    header_is_https(headers, "x-forwarded-proto")
        || forwarded_proto_is_https(headers)
        || header_is_https_url(headers, header::ORIGIN.as_str())
        || header_is_https_url(headers, header::REFERER.as_str())
}

fn env_force_secure_cookie() -> Option<bool> {
    let value = std::env::var("HONE_PUBLIC_SECURE_COOKIE").ok()?;
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Some(true),
        "0" | "false" | "no" => Some(false),
        other => {
            tracing::warn!(
                "HONE_PUBLIC_SECURE_COOKIE has unrecognized value {:?}, defaulting to Secure=true",
                other
            );
            Some(true)
        }
    }
}

fn header_is_https(headers: &HeaderMap, name: &str) -> bool {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .any(|item| item.trim().eq_ignore_ascii_case("https"))
        })
        .unwrap_or(false)
}

fn header_is_https_url(headers: &HeaderMap, name: &str) -> bool {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim_start().starts_with("https://"))
}

fn forwarded_proto_is_https(headers: &HeaderMap) -> bool {
    headers
        .get("forwarded")
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value.split(';').any(|segment| {
                let lower = segment.trim().to_ascii_lowercase();
                lower == "proto=https" || lower.ends_with(", proto=https")
            })
        })
        .unwrap_or(false)
}

fn normalize_invite_code(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .trim()
        .to_uppercase()
}

fn public_client_key(headers: &HeaderMap) -> String {
    if let Some(forwarded_for) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').find(|item| !item.trim().is_empty()))
    {
        return format!("ip:{}", forwarded_for.trim());
    }
    if let Some(real_ip) = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("ip:{real_ip}");
    }

    "ip:unknown".to_string()
}

fn json_rate_limited(retry_after_secs: u64) -> Response {
    let mut response = crate::routes::json_error(
        StatusCode::TOO_MANY_REQUESTS,
        format!("邀请码尝试过于频繁，请在 {} 秒后重试", retry_after_secs),
    );
    if let Ok(value) = HeaderValue::from_str(&retry_after_secs.to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
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

#[cfg(test)]
mod tests {
    use super::{
        build_session_cookie, clear_session_cookie, normalize_invite_code, public_client_key,
    };
    use axum::http::{HeaderMap, HeaderValue, header};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn invite_code_normalization_removes_spaces_and_uppercases() {
        assert_eq!(normalize_invite_code(" hone-abc 123 \n"), "HONE-ABC123");
    }

    #[test]
    fn secure_cookie_is_enabled_for_https_origin() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://chat.example.com"),
        );

        let cookie_value = build_session_cookie("token", &headers);
        let cookie = cookie_value.to_str().expect("cookie");
        let cleared_value = clear_session_cookie(&headers);
        let cleared = cleared_value.to_str().expect("cookie");

        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Strict"));
        assert!(cleared.contains("Secure"));
    }

    #[test]
    fn client_key_prefers_forwarded_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.9, 10.0.0.2"),
        );
        assert_eq!(public_client_key(&headers), "ip:203.0.113.9");
    }

    #[test]
    fn env_force_secure_cookie_overrides_headers() {
        let _guard = env_lock().lock().unwrap();

        // When env is set to "true", Secure flag should be on even without https headers.
        unsafe { std::env::set_var("HONE_PUBLIC_SECURE_COOKIE", "true") };
        let headers = HeaderMap::new();
        let cookie = build_session_cookie("tok", &headers)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(cookie.contains("Secure"), "env=true should force Secure");

        // When env is set to "false", Secure flag should be off even with https origin.
        unsafe { std::env::set_var("HONE_PUBLIC_SECURE_COOKIE", "false") };
        let mut https_headers = HeaderMap::new();
        https_headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://chat.example.com"),
        );
        let cookie2 = build_session_cookie("tok", &https_headers)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(
            !cookie2.contains("Secure"),
            "env=false should suppress Secure"
        );

        unsafe { std::env::remove_var("HONE_PUBLIC_SECURE_COOKIE") };
    }
}
