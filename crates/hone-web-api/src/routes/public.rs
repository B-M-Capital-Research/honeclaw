use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response, sse::Event, sse::KeepAlive, sse::Sse};
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use hone_core::ActorIdentity;

use crate::public_auth::PublicAuthLimitStatus;
use crate::routes::chat::build_chat_sse;
use crate::routes::history::history_from_messages;
use crate::state::{AppState, PushEvent};
use crate::types::{
    PublicAuthUserInfo, PublicChatRequest, PublicInviteLoginRequest, PublicUploadedAttachment,
};

/// Upper bounds enforced when users upload files through the public chat.
/// Kept conservative so a misbehaving client can't fill disk with a single request.
const PUBLIC_UPLOAD_MAX_FILES: usize = 4;
const PUBLIC_UPLOAD_MAX_BYTES: usize = 10 * 1024 * 1024;

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

    // Rate-limit by phone number + IP (best-effort). The phone number comes
    // from the request body, but it still helps throttle attempts that target
    // a known phone number even if the IP headers are spoofed.
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
    let attachments = request.attachments.unwrap_or_default();

    if message.is_empty() && attachments.is_empty() {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "消息不能为空");
    }

    let user_upload_root = public_upload_dir(&state, &user.user_id);
    let mut validated_paths = Vec::with_capacity(attachments.len());
    for attachment in &attachments {
        match validate_public_upload_path(&user_upload_root, &attachment.path) {
            Ok(path) => validated_paths.push(path),
            Err(response) => return response,
        }
    }

    let attachments_count = validated_paths.len();
    let combined_message = compose_message_with_attachments(&message, &validated_paths);

    build_chat_sse(state, Ok(actor), combined_message, attachments_count).into_response()
}

pub(crate) async fn handle_upload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };

    let upload_root = public_upload_dir(&state, &user.user_id);
    let day = hone_core::beijing_now().format("%Y-%m-%d").to_string();
    let target_dir = upload_root.join(&day);
    if let Err(error) = std::fs::create_dir_all(&target_dir) {
        return crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("创建上传目录失败: {error}"),
        );
    }

    let mut stored: Vec<PublicUploadedAttachment> = Vec::new();
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                return crate::routes::json_error(
                    StatusCode::BAD_REQUEST,
                    format!("读取 multipart 失败: {error}"),
                );
            }
        };
        // We accept either `file` or `files` as the form field name; other fields are ignored.
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name != "file" && field_name != "files" {
            continue;
        }
        let original_name = field
            .file_name()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "attachment".to_string());
        let bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(error) => {
                return crate::routes::json_error(
                    StatusCode::BAD_REQUEST,
                    format!("读取上传数据失败: {error}"),
                );
            }
        };
        if bytes.is_empty() {
            continue;
        }
        if bytes.len() > PUBLIC_UPLOAD_MAX_BYTES {
            return crate::routes::json_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "单个附件过大，最大 {} MB",
                    PUBLIC_UPLOAD_MAX_BYTES / 1024 / 1024
                ),
            );
        }
        if stored.len() >= PUBLIC_UPLOAD_MAX_FILES {
            return crate::routes::json_error(
                StatusCode::BAD_REQUEST,
                format!("单次最多上传 {PUBLIC_UPLOAD_MAX_FILES} 个附件"),
            );
        }

        let safe_name = sanitize_attachment_name(&original_name);
        let stored_name = format!("{}-{}", Uuid::new_v4().simple(), safe_name);
        let final_path = target_dir.join(&stored_name);
        if let Err(error) = std::fs::write(&final_path, &bytes) {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("写入附件失败: {error}"),
            );
        }

        stored.push(PublicUploadedAttachment {
            path: final_path.to_string_lossy().to_string(),
            name: safe_name,
            kind: classify_attachment_kind(&original_name),
            size: bytes.len() as u64,
        });
    }

    if stored.is_empty() {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "未收到文件");
    }

    Json(json!({ "attachments": stored })).into_response()
}

/// Per-user upload root. Lives under the configured sessions dir so the existing
/// `/api/image` and `/api/file` proxy whitelist already covers reads.
fn public_upload_dir(state: &AppState, user_id: &str) -> PathBuf {
    let base = PathBuf::from(&state.core.config.storage.sessions_dir);
    base.join("public-uploads").join(sanitize_user_id(user_id))
}

fn compose_message_with_attachments(message: &str, attachment_paths: &[PathBuf]) -> String {
    if attachment_paths.is_empty() {
        return message.to_string();
    }
    let att = attachment_paths
        .iter()
        .map(|path| format!("[附件: {}]", path.to_string_lossy()))
        .collect::<Vec<_>>()
        .join("\n");
    if message.is_empty() {
        att
    } else {
        format!("{message}\n{att}")
    }
}

/// Only accept attachment paths that sit inside this user's upload root, so the
/// chat endpoint can't be used to reference arbitrary files on disk.
fn validate_public_upload_path(upload_root: &Path, raw_path: &str) -> Result<PathBuf, Response> {
    let cleaned = raw_path.trim().strip_prefix("file://").unwrap_or(raw_path);
    if cleaned.is_empty() {
        return Err(crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "附件路径为空",
        ));
    }
    let path = PathBuf::from(cleaned);
    let canonical_root =
        std::fs::canonicalize(upload_root).unwrap_or_else(|_| upload_root.to_path_buf());
    let canonical_target = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    if !canonical_target.starts_with(&canonical_root) {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "附件路径不在允许范围内",
        ));
    }
    if !canonical_target.is_file() {
        return Err(crate::routes::json_error(
            StatusCode::NOT_FOUND,
            "附件不存在",
        ));
    }
    Ok(canonical_target)
}

fn sanitize_user_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            out.push(char::from(*byte));
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "anonymous".to_string()
    } else {
        trimmed
    }
}

fn sanitize_attachment_name(raw: &str) -> String {
    let stem = Path::new(raw)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "attachment".to_string());
    let mut out = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "attachment".to_string()
    } else {
        trimmed
    }
}

fn classify_attachment_kind(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
    {
        "image".to_string()
    } else if lower.ends_with(".pdf") {
        "pdf".to_string()
    } else {
        "file".to_string()
    }
}

pub(crate) async fn handle_public_image(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    query: axum::extract::Query<crate::types::ImageQuery>,
) -> Response {
    if let Err(response) = require_public_user(&state, &headers) {
        return response;
    }
    crate::routes::files::handle_image(state, query)
        .await
        .into_response()
}

pub(crate) async fn handle_public_file(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    query: axum::extract::Query<crate::types::ImageQuery>,
) -> Response {
    if let Err(response) = require_public_user(&state, &headers) {
        return response;
    }
    crate::routes::files::handle_file(state, query)
        .await
        .into_response()
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
/// Checks `HONE_PUBLIC_SECURE_COOKIE` env var first (accepts "true"/"1"/"yes"
/// to force-enable, "false"/"0"/"no" to force-disable). Falls back to
/// inspecting request headers when the env var is absent or empty.
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
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    match normalized.as_str() {
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
    const SECURE_COOKIE_ENV: &str = "HONE_PUBLIC_SECURE_COOKIE";

    struct EnvVarGuard {
        name: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let previous = std::env::var(name).ok();
            unsafe { std::env::set_var(name, value) };
            Self { name, previous }
        }

        fn unset(name: &'static str) -> Self {
            let previous = std::env::var(name).ok();
            unsafe { std::env::remove_var(name) };
            Self { name, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    #[test]
    fn invite_code_normalization_removes_spaces_and_uppercases() {
        assert_eq!(normalize_invite_code(" hone-abc 123 \n"), "HONE-ABC123");
    }

    #[test]
    fn secure_cookie_is_enabled_for_https_origin() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::unset(SECURE_COOKIE_ENV);
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
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::set(SECURE_COOKIE_ENV, "true");

        // When env is set to "true", Secure flag should be on even without https headers.
        let headers = HeaderMap::new();
        let cookie = build_session_cookie("tok", &headers)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(cookie.contains("Secure"), "env=true should force Secure");

        // When env is set to "false", Secure flag should be off even with https origin.
        unsafe { std::env::set_var(SECURE_COOKIE_ENV, "false") };
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
    }

    #[test]
    fn empty_env_secure_cookie_falls_back_to_headers() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::set(SECURE_COOKIE_ENV, "");

        let cookie_without_https = build_session_cookie("tok", &HeaderMap::new())
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(
            !cookie_without_https.contains("Secure"),
            "empty env should not force Secure"
        );

        let mut https_headers = HeaderMap::new();
        https_headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://chat.example.com"),
        );
        let cookie_with_https = build_session_cookie("tok", &https_headers)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(
            cookie_with_https.contains("Secure"),
            "empty env should fall back to https headers"
        );
    }
}
