use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use axum::Json;
use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response, sse::Event, sse::KeepAlive, sse::Sse};
use serde::Deserialize;
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;
use uuid::Uuid;

use hone_channels::agent_session::{
    AgentRunOptions, AgentRunQuotaMode, AgentSession, AgentSessionEvent, AgentSessionListener,
};
use hone_channels::attachments::{
    AttachmentIngestRequest, AttachmentPersistRequest, RawAttachment, build_attachment_ack_message,
    build_user_input, ingest_raw_attachments, spawn_attachment_persist_pipeline,
};
use hone_channels::prompt::PromptOptions;
use hone_channels::run_event::RunEvent;
use hone_core::{ActorIdentity, HoneError};
use hone_memory::WebSessionAuthResult;

use crate::public_auth::PublicAuthLimitStatus;
use crate::routes::chat::build_chat_sse;
use crate::routes::history::public_history_from_messages;
use crate::state::{AppState, PushEvent};
use crate::types::{
    PublicAuthUserInfo, PublicChatAttachmentInput, PublicChatRequest, PublicSmsLoginRequest,
    PublicSmsSendRequest, PublicUploadedAttachment,
};

/// Upper bounds enforced when users upload files through the public chat.
/// Kept conservative so a misbehaving client can't fill disk with a single request.
const PUBLIC_UPLOAD_MAX_FILES: usize = 4;
const PUBLIC_UPLOAD_MAX_BYTES: usize = 10 * 1024 * 1024;

const WEB_SESSION_COOKIE: &str = "hone_web_session";
const WEB_SESSION_MAX_AGE_LONG_SECS: i64 = 30 * 24 * 60 * 60;
const WEB_SESSION_MAX_AGE_SHORT_SECS: i64 = 24 * 60 * 60;

/// 与 memory::web_auth 中的 TTL 常量保持一致。
const SESSION_TTL_DAYS_LONG: i64 = hone_memory::SESSION_TTL_DAYS_LONG;
const SESSION_TTL_DAYS_SHORT: i64 = hone_memory::SESSION_TTL_DAYS_SHORT;

/// 当前生效的协议版本。改动 /terms /privacy 文本时手动 bump,
/// 并让已登录用户重新勾选接受(可后续增强)。
pub(crate) const TOS_VERSION: &str = "2.1";

pub(crate) async fn handle_captcha_config() -> Response {
    Json(crate::aliyun_captcha::AliyunCaptchaConfig::public_config_from_env()).into_response()
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAiChatCompletionRequest {
    pub model: Option<String>,
    #[serde(default)]
    pub messages: Vec<OpenAiChatMessage>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAiChatMessage {
    pub role: String,
    pub content: serde_json::Value,
}

struct OpenAiStreamListener {
    tx: tokio::sync::mpsc::Sender<String>,
    id: String,
    model: String,
    created: i64,
}

#[async_trait]
impl AgentSessionListener for OpenAiStreamListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        let content = match event {
            AgentSessionEvent::Segment { text } => Some(text),
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content),
            AgentSessionEvent::Done { response } if !response.success => Some(
                response
                    .error
                    .unwrap_or_else(|| "Hone Cloud run failed".to_string()),
            ),
            _ => None,
        };
        if let Some(content) = content.filter(|value| !value.is_empty()) {
            let _ = self
                .tx
                .send(openai_stream_chunk(
                    &self.id,
                    &self.model,
                    self.created,
                    Some(&content),
                    None,
                ))
                .await;
        }
    }
}

pub(crate) async fn handle_sms_send_code(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicSmsSendRequest>,
) -> Response {
    let phone_number = match crate::routes::require_phone_number(request.phone_number, "手机号")
    {
        Ok(value) => value,
        Err(response) => return response,
    };
    let sms_phone_number = aliyun_sms_phone_number(&phone_number);

    let ip_key = public_client_key(&headers);
    let phone_key = format!("sms-send:{sms_phone_number}");
    for key in [&ip_key, &phone_key] {
        if let PublicAuthLimitStatus::Blocked { retry_after_secs } =
            state.public_auth_limiter.check(key)
        {
            return json_rate_limited(retry_after_secs);
        }
    }

    if crate::aliyun_captcha::AliyunCaptchaConfig::public_config_from_env().enabled {
        let captcha_verify_param = request.captcha_verify_param.unwrap_or_default();
        match crate::aliyun_captcha::verify_captcha(&state.http_client, &captcha_verify_param).await
        {
            Ok(true) => {}
            Ok(false) => {
                let _ = state.public_auth_limiter.record_failure(&ip_key);
                return crate::routes::json_error(StatusCode::FORBIDDEN, "请先完成图形验证");
            }
            Err(error) => {
                warn!("aliyun captcha verification failed: {error}");
                return captcha_provider_error_response("图形验证服务暂不可用", error);
            }
        }
    }

    let user = match find_active_invite_user_by_sms_phone(&state, &phone_number) {
        Ok(value) => value,
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("查询邀请资格失败: {error}"),
            );
        }
    };
    if user.is_none() {
        let _ = state.public_auth_limiter.record_failure(&phone_key);
        return crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "目前是邀请制，请联系 bm@hone-claw.com 加入邀请名单",
        );
    }

    match crate::aliyun_sms::send_verify_code(&state.http_client, &sms_phone_number).await {
        Ok(()) => {
            state.public_auth_limiter.record_success(&phone_key);
            Json(json!({ "ok": true })).into_response()
        }
        Err(error) => sms_provider_error_response("发送验证码失败", error),
    }
}

pub(crate) async fn handle_sms_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicSmsLoginRequest>,
) -> Response {
    let phone_number = match crate::routes::require_phone_number(request.phone_number, "手机号")
    {
        Ok(value) => value,
        Err(response) => return response,
    };
    let sms_phone_number = aliyun_sms_phone_number(&phone_number);
    let verify_code = request
        .verify_code
        .unwrap_or_default()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if !(4..=8).contains(&verify_code.len()) {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "验证码格式不正确");
    }
    let tos_version = request.tos_version.unwrap_or_default();
    if tos_version.trim().is_empty() {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "需同意用户协议与隐私政策");
    }
    if tos_version.trim() != TOS_VERSION {
        return crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "协议版本已更新，请刷新页面后重新确认",
        );
    }

    let ip_key = public_client_key(&headers);
    let phone_key = format!("sms-login:{sms_phone_number}");
    for key in [&ip_key, &phone_key] {
        if let PublicAuthLimitStatus::Blocked { retry_after_secs } =
            state.public_auth_limiter.check(key)
        {
            return json_rate_limited(retry_after_secs);
        }
    }

    let user = match find_active_invite_user_by_sms_phone(&state, &phone_number) {
        Ok(Some(user)) => user,
        Ok(None) => {
            let _ = state.public_auth_limiter.record_failure(&phone_key);
            return crate::routes::json_error(
                StatusCode::FORBIDDEN,
                "目前是邀请制，请联系 bm@hone-claw.com 加入邀请名单",
            );
        }
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("查询邀请资格失败: {error}"),
            );
        }
    };

    let verified = match crate::aliyun_sms::check_verify_code(
        &state.http_client,
        &sms_phone_number,
        &verify_code,
    )
    .await
    {
        Ok(value) => value,
        Err(error) => return sms_provider_error_response("核验验证码失败", error),
    };
    if !verified {
        return sms_login_failed(&state, &ip_key, &phone_key);
    }

    if let Err(error) = state
        .web_auth
        .record_tos_acceptance(&user.user_id, TOS_VERSION)
    {
        return crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("记录协议接受失败: {error}"),
        );
    }

    let ttl_days = if request.remember {
        SESSION_TTL_DAYS_LONG
    } else {
        SESSION_TTL_DAYS_SHORT
    };
    let max_age_secs = if request.remember {
        WEB_SESSION_MAX_AGE_LONG_SECS
    } else {
        WEB_SESSION_MAX_AGE_SHORT_SECS
    };
    let session = match state
        .web_auth
        .create_session_for_user(&user.user_id, ttl_days)
    {
        Ok(Some(session)) => session,
        Ok(None) => {
            return crate::routes::json_error(StatusCode::UNAUTHORIZED, "账号不可用，请联系管理员");
        }
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("创建登录态失败: {error}"),
            );
        }
    };

    state.public_auth_limiter.record_success(&ip_key);
    state.public_auth_limiter.record_success(&phone_key);
    let refreshed = match state.web_auth.find_invite_user(&session.user_id) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, "用户已丢失");
        }
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取用户失败: {error}"),
            );
        }
    };
    let user_id = refreshed.user_id.clone();
    let mut response = Json(json!({
        "user": to_public_auth_user(&state, &user_id, refreshed),
    }))
    .into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        build_session_cookie(&session.session_token, &headers, max_age_secs),
    );
    response
}

fn aliyun_sms_phone_number(phone_number: &str) -> String {
    strip_china_country_code(phone_number).unwrap_or_else(|| phone_number.to_string())
}

fn public_sms_phone_candidates(phone_number: &str) -> Vec<String> {
    let mut candidates = Vec::with_capacity(2);
    if let Some(local_number) = strip_china_country_code(phone_number) {
        candidates.push(local_number);
    }
    if !candidates.iter().any(|candidate| candidate == phone_number) {
        candidates.push(phone_number.to_string());
    }
    candidates
}

fn strip_china_country_code(phone_number: &str) -> Option<String> {
    phone_number
        .strip_prefix("+86")
        .filter(|local_number| local_number.chars().all(|ch| ch.is_ascii_digit()))
        .filter(|local_number| (6..=20).contains(&local_number.len()))
        .map(ToString::to_string)
}

fn find_active_invite_user_by_sms_phone(
    state: &AppState,
    phone_number: &str,
) -> Result<Option<hone_memory::WebInviteUser>, HoneError> {
    for candidate in public_sms_phone_candidates(phone_number) {
        if let Some(user) = state
            .web_auth
            .find_active_invite_user_by_phone(&candidate)?
        {
            return Ok(Some(user));
        }
    }
    Ok(None)
}

fn sms_login_failed(state: &AppState, ip_key: &str, phone_key: &str) -> Response {
    let mut rate_limited = None;
    for key in [ip_key, phone_key] {
        if let Some(retry_after_secs) = state.public_auth_limiter.record_failure(key) {
            rate_limited = Some(json_rate_limited(retry_after_secs));
        }
    }
    rate_limited.unwrap_or_else(|| {
        crate::routes::json_error(StatusCode::UNAUTHORIZED, "验证码不正确或已过期")
    })
}

fn sms_provider_error_response(prefix: &str, error: crate::aliyun_sms::AliyunSmsError) -> Response {
    match error.kind {
        crate::aliyun_sms::AliyunSmsErrorKind::Config => crate::routes::json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{prefix}: 短信服务未配置"),
        ),
        crate::aliyun_sms::AliyunSmsErrorKind::Transport => {
            crate::routes::json_error(StatusCode::BAD_GATEWAY, format!("{prefix}: {error}"))
        }
        crate::aliyun_sms::AliyunSmsErrorKind::Provider => {
            crate::routes::json_error(StatusCode::BAD_GATEWAY, format!("{prefix}: {error}"))
        }
    }
}

fn captcha_provider_error_response(
    prefix: &str,
    error: crate::aliyun_captcha::AliyunCaptchaError,
) -> Response {
    match error.kind {
        crate::aliyun_captcha::AliyunCaptchaErrorKind::Config => crate::routes::json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{prefix}: 配置缺失"),
        ),
        crate::aliyun_captcha::AliyunCaptchaErrorKind::Transport
        | crate::aliyun_captcha::AliyunCaptchaErrorKind::Provider => {
            crate::routes::json_error(StatusCode::BAD_GATEWAY, prefix)
        }
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

pub(crate) async fn handle_bootstrap(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let user_id = user.user_id.clone();
    let actor = match ActorIdentity::new("web", &user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()),
    };
    let messages = state
        .core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .unwrap_or_default();

    Json(json!({
        "user": to_public_auth_user(&state, &user_id, user),
        "messages": public_history_from_messages(&messages),
    }))
    .into_response()
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
        "messages": public_history_from_messages(&messages),
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

    let (combined_message, attachments_count) =
        match build_public_chat_input(&state, &actor, &user.user_id, &message, attachments).await {
            Ok(value) => value,
            Err(response) => return response,
        };

    build_chat_sse(state, Ok(actor), combined_message, attachments_count).into_response()
}

pub(crate) async fn handle_openai_chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<OpenAiChatCompletionRequest>,
) -> Response {
    let user = match require_public_api_key_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()),
    };
    let message = match last_openai_user_message(&request.messages) {
        Some(message) if !message.trim().is_empty() => message.trim().to_string(),
        _ => return crate::routes::json_error(StatusCode::BAD_REQUEST, "messages 缺少 user 内容"),
    };
    let model = request
        .model
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("hone-cloud")
        .trim()
        .to_string();

    if request.stream {
        build_openai_chat_sse(state, actor, message, model).into_response()
    } else {
        let content = match run_public_api_chat_once(state, actor, message).await {
            Ok(content) => content,
            Err(response) => return response,
        };
        let id = format!("chatcmpl-{}", Uuid::new_v4().simple());
        Json(json!({
            "id": id,
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": content,
                },
                "finish_reason": "stop",
            }],
        }))
        .into_response()
    }
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
    let oss = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss);
    let target_dir = upload_root.join(&day);
    if oss.is_none()
        && let Err(error) = std::fs::create_dir_all(&target_dir)
    {
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
        let stored_path = if let Some(oss) = oss.as_ref() {
            let key = oss.public_upload_key(&user.user_id, &day, &stored_name);
            if let Err(error) = oss
                .put_object(
                    &key,
                    bytes.to_vec(),
                    content_type_for_attachment(&original_name),
                )
                .await
            {
                return crate::routes::json_error(StatusCode::BAD_GATEWAY, error);
            }
            oss.object_uri(&key)
        } else {
            let final_path = target_dir.join(&stored_name);
            if let Err(error) = std::fs::write(&final_path, &bytes) {
                return crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("写入附件失败: {error}"),
                );
            }
            final_path.to_string_lossy().to_string()
        };

        stored.push(PublicUploadedAttachment {
            path: stored_path,
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

/// Per-user upload root. Lives under the configured sessions dir so the shared
/// file proxy roots cover `/api/image`, `/api/file`, and their `/api/public/*`
/// wrappers.
pub(crate) fn public_upload_dir(state: &AppState, user_id: &str) -> PathBuf {
    let base = PathBuf::from(&state.core.config.storage.sessions_dir);
    base.join("public-uploads").join(sanitize_user_id(user_id))
}

async fn build_public_chat_input(
    state: &Arc<AppState>,
    actor: &ActorIdentity,
    user_id: &str,
    message: &str,
    attachments: Vec<PublicChatAttachmentInput>,
) -> Result<(String, usize), Response> {
    if attachments.is_empty() {
        return Ok((message.to_string(), 0));
    }

    let upload_root = public_upload_dir(state, user_id);
    let oss = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss);
    let mut raw_attachments = Vec::with_capacity(attachments.len());
    for attachment in attachments {
        raw_attachments.push(
            public_chat_raw_attachment(&upload_root, oss.as_ref(), user_id, attachment).await?,
        );
    }

    let session_id = actor.session_id();
    let received = ingest_raw_attachments(
        state.core.as_ref(),
        AttachmentIngestRequest {
            channel: "web".to_string(),
            actor: actor.clone(),
            session_id: session_id.clone(),
            attachments: raw_attachments,
        },
    )
    .await;
    if !received.is_empty() {
        spawn_attachment_persist_pipeline(
            state.core.clone(),
            AttachmentPersistRequest {
                channel: "web".to_string(),
                actor: actor.clone(),
                user_id: user_id.to_string(),
                session_id,
                attachments: received.clone(),
            },
        );
    }

    build_public_chat_user_input(message, &received).map(|input| (input, received.len()))
}

async fn public_chat_raw_attachment(
    upload_root: &Path,
    oss: Option<&crate::cloud_oss::OssClient>,
    user_id: &str,
    attachment: PublicChatAttachmentInput,
) -> Result<RawAttachment, Response> {
    let validated_path = validate_public_upload_path(upload_root, oss, user_id, &attachment.path)?;
    let filename = public_attachment_filename(&attachment, &validated_path);

    if let Some(oss) = oss
        && oss.is_public_upload_uri_for_user(&validated_path, user_id)
    {
        let Some(key) = oss.parse_managed_uri(&validated_path) else {
            return Err(crate::routes::json_error(
                StatusCode::BAD_REQUEST,
                "附件路径不在允许范围内",
            ));
        };
        let object = oss.get_object(key).await.map_err(|error| {
            warn!(
                user_id = %user_id,
                attachment = %filename,
                "读取 public OSS 附件失败: {error}"
            );
            crate::routes::json_error(
                StatusCode::BAD_GATEWAY,
                "附件读取失败，请重新上传后重试，或直接粘贴图片中的文字。",
            )
        })?;
        let size = u32::try_from(object.bytes.len()).unwrap_or(u32::MAX);
        let content_type = if object.content_type.trim().is_empty() {
            content_type_for_attachment(&filename).to_string()
        } else {
            object.content_type
        };
        return Ok(RawAttachment {
            filename,
            content_type: Some(content_type),
            size: Some(size),
            url: validated_path,
            local_path: None,
            data: Some(object.bytes),
            error: None,
        });
    }

    let local_path = PathBuf::from(&validated_path);
    let metadata = std::fs::metadata(&local_path).map_err(|_| {
        crate::routes::json_error(
            StatusCode::NOT_FOUND,
            "附件不存在，请重新上传后重试，或直接粘贴附件中的文字。",
        )
    })?;
    let size = u32::try_from(metadata.len()).unwrap_or(u32::MAX);
    let content_type = content_type_for_attachment(&filename).to_string();
    Ok(RawAttachment {
        filename,
        content_type: Some(content_type),
        size: Some(size),
        url: format!("file://{}", local_path.display()),
        local_path: Some(local_path),
        data: None,
        error: None,
    })
}

fn public_attachment_filename(
    attachment: &PublicChatAttachmentInput,
    validated_path: &str,
) -> String {
    attachment
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_attachment_name)
        .or_else(|| {
            Path::new(validated_path)
                .file_name()
                .and_then(|value| value.to_str())
                .map(sanitize_attachment_name)
        })
        .unwrap_or_else(|| "attachment".to_string())
}

fn build_public_chat_user_input(
    message: &str,
    attachments: &[hone_channels::attachments::ReceivedAttachment],
) -> Result<String, Response> {
    if attachments.is_empty() {
        return Ok(message.to_string());
    }

    if attachments
        .iter()
        .all(|attachment| attachment.error.is_some())
    {
        return Err(crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            build_attachment_ack_message(attachments),
        ));
    }

    let input = build_user_input(message, attachments);
    if input.trim().is_empty() {
        return Err(crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "附件暂时无法读取，请重新上传后重试，或直接粘贴附件中的文字。",
        ));
    }
    Ok(input)
}

/// Only accept attachment paths that sit inside this user's upload root, so the
/// chat endpoint can't be used to reference arbitrary files on disk.
pub(crate) fn validate_public_upload_path(
    upload_root: &Path,
    oss: Option<&crate::cloud_oss::OssClient>,
    user_id: &str,
    raw_path: &str,
) -> Result<String, Response> {
    if let Some(oss) = oss
        && oss.is_public_upload_uri_for_user(raw_path, user_id)
    {
        return Ok(raw_path.trim().to_string());
    }

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
    Ok(canonical_target.to_string_lossy().to_string())
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
    } else if trimmed.starts_with('.') {
        format!("attachment{trimmed}")
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

fn content_type_for_attachment(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".pdf") {
        "application/pdf"
    } else {
        "application/octet-stream"
    }
}

pub(crate) async fn handle_public_image(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    query: axum::extract::Query<crate::types::ImageQuery>,
) -> Response {
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = validate_public_proxy_path(&state, &user.user_id, &query.path) {
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
    let user = match require_public_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = validate_public_proxy_path(&state, &user.user_id, &query.path) {
        return response;
    }
    crate::routes::files::handle_file(state, query)
        .await
        .into_response()
}

fn validate_public_proxy_path(
    state: &AppState,
    user_id: &str,
    raw_path: &Option<String>,
) -> Result<(), Response> {
    let Some(raw_path) = raw_path.as_deref() else {
        return Err(crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "缺少 path",
        ));
    };
    let user_upload_root = public_upload_dir(state, user_id);
    let oss = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss);
    validate_public_upload_path(&user_upload_root, oss.as_ref(), user_id, raw_path).map(|_| ())
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

pub(crate) fn require_public_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<hone_memory::WebInviteUser, Response> {
    let Some(token) = read_session_token(headers) else {
        warn!("public auth rejected: missing session cookie");
        return Err(crate::routes::json_error(
            StatusCode::UNAUTHORIZED,
            "未登录",
        ));
    };
    match state.web_auth.authenticate_session_detailed(&token) {
        Ok(WebSessionAuthResult::Authenticated(user)) => Ok(user),
        Ok(WebSessionAuthResult::Missing) => {
            warn!("public auth rejected: session token not found");
            Err(crate::routes::json_error(
                StatusCode::UNAUTHORIZED,
                "登录已过期，请重新输入邀请码",
            ))
        }
        Ok(WebSessionAuthResult::Expired { user_id }) => {
            warn!(%user_id, "public auth rejected: session expired");
            Err(crate::routes::json_error(
                StatusCode::UNAUTHORIZED,
                "登录已过期，请重新输入邀请码",
            ))
        }
        Ok(WebSessionAuthResult::UserRevoked { user_id }) => {
            warn!(%user_id, "public auth rejected: user revoked");
            Err(crate::routes::json_error(
                StatusCode::UNAUTHORIZED,
                "登录已过期，请重新输入邀请码",
            ))
        }
        Ok(WebSessionAuthResult::UserMissing { user_id }) => {
            warn!(%user_id, "public auth rejected: session user missing");
            Err(crate::routes::json_error(
                StatusCode::UNAUTHORIZED,
                "登录已过期，请重新输入邀请码",
            ))
        }
        Err(error) => Err(crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("验证登录态失败: {error}"),
        )),
    }
}

fn require_public_api_key_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<hone_memory::WebInviteUser, Response> {
    let Some(api_key) = read_bearer_token(headers) else {
        return Err(crate::routes::json_error(
            StatusCode::UNAUTHORIZED,
            "缺少 Authorization: Bearer API Key",
        ));
    };
    match state.web_auth.find_invite_user_by_api_key(&api_key) {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "API Key 无效或已停用",
        )),
        Err(error) => Err(crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("验证 API Key 失败: {error}"),
        )),
    }
}

fn read_bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?.trim();
    raw.strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
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

fn last_openai_user_message(messages: &[OpenAiChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| openai_content_to_text(&message.content))
}

fn openai_content_to_text(content: &serde_json::Value) -> String {
    if let Some(text) = content.as_str() {
        return text.to_string();
    }
    if let Some(parts) = content.as_array() {
        return parts
            .iter()
            .filter_map(|part| {
                part.get("text")
                    .and_then(|value| value.as_str())
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    String::new()
}

async fn run_public_api_chat_once(
    state: Arc<AppState>,
    actor: ActorIdentity,
    message: String,
) -> Result<String, Response> {
    if let Some(reply) = state
        .core
        .try_handle_intercept_command(&actor, &message)
        .await
    {
        return Ok(reply);
    }
    let prompt_options = PromptOptions {
        is_admin: state.core.is_admin_actor(&actor),
        ..PromptOptions::default()
    };
    let session = AgentSession::new(state.core.clone(), actor.clone(), actor.user_id.clone())
        .with_restore_max_messages(None)
        .with_prompt_options(prompt_options)
        .with_recv_extra(Some("openai_compatible_api=true".to_string()));
    let run_options = AgentRunOptions {
        timeout: Some(state.core.config.agent.overall_timeout()),
        segmenter: None,
        quota_mode: AgentRunQuotaMode::UserConversation,
        model_override: None,
    };
    let result = session.run(&message, run_options).await;
    if result.response.success {
        Ok(result.response.content)
    } else {
        Err(crate::routes::json_error(
            StatusCode::BAD_GATEWAY,
            result
                .response
                .error
                .unwrap_or_else(|| "Hone Cloud run failed".to_string()),
        ))
    }
}

fn build_openai_chat_sse(
    state: Arc<AppState>,
    actor: ActorIdentity,
    message: String,
    model: String,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(64);
    let id = format!("chatcmpl-{}", Uuid::new_v4().simple());
    let created = chrono::Utc::now().timestamp();
    tokio::spawn(async move {
        if let Some(reply) = state
            .core
            .try_handle_intercept_command(&actor, &message)
            .await
        {
            let _ = tx
                .send(openai_stream_chunk(
                    &id,
                    &model,
                    created,
                    Some(&reply),
                    None,
                ))
                .await;
            let _ = tx
                .send(openai_stream_chunk(
                    &id,
                    &model,
                    created,
                    None,
                    Some("stop"),
                ))
                .await;
            let _ = tx.send("[DONE]".to_string()).await;
            return;
        }
        let prompt_options = PromptOptions {
            is_admin: state.core.is_admin_actor(&actor),
            ..PromptOptions::default()
        };
        let mut session =
            AgentSession::new(state.core.clone(), actor.clone(), actor.user_id.clone())
                .with_restore_max_messages(None)
                .with_prompt_options(prompt_options)
                .with_recv_extra(Some("openai_compatible_api=true".to_string()));
        session.add_listener(Arc::new(OpenAiStreamListener {
            tx: tx.clone(),
            id: id.clone(),
            model: model.clone(),
            created,
        }));
        let run_options = AgentRunOptions {
            timeout: Some(state.core.config.agent.overall_timeout()),
            segmenter: None,
            quota_mode: AgentRunQuotaMode::UserConversation,
            model_override: None,
        };
        let result = session.run(&message, run_options).await;
        let finish = if result.response.success {
            "stop"
        } else {
            "error"
        };
        let _ = tx
            .send(openai_stream_chunk(
                &id,
                &model,
                created,
                None,
                Some(finish),
            ))
            .await;
        let _ = tx.send("[DONE]".to_string()).await;
    });
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|data| Ok::<_, Infallible>(Event::default().data(data)));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

fn openai_stream_chunk(
    id: &str,
    model: &str,
    created: i64,
    content: Option<&str>,
    finish_reason: Option<&str>,
) -> String {
    let delta = content
        .map(|text| json!({ "content": text }))
        .unwrap_or_else(|| json!({}));
    json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [{
            "index": 0,
            "delta": delta,
            "finish_reason": finish_reason,
        }],
    })
    .to_string()
}

fn build_session_cookie(
    session_token: &str,
    headers: &HeaderMap,
    max_age_secs: i64,
) -> HeaderValue {
    let secure_attr = if request_is_secure(headers) {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{WEB_SESSION_COOKIE}={session_token}; Path=/; HttpOnly; SameSite=Strict{secure_attr}; Max-Age={max_age_secs}"
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
        format!("登录尝试过于频繁，请在 {} 秒后重试", retry_after_secs),
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
        has_password: user.password_hash.is_some(),
        tos_accepted_at: user.tos_accepted_at,
        tos_version: user.tos_version,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WEB_SESSION_MAX_AGE_LONG_SECS, WEB_SESSION_MAX_AGE_SHORT_SECS, aliyun_sms_phone_number,
        build_public_chat_user_input, build_session_cookie, clear_session_cookie,
        public_attachment_filename, public_client_key, public_sms_phone_candidates,
        validate_public_upload_path,
    };
    use axum::http::{HeaderMap, HeaderValue, header};
    use hone_channels::attachments::{AttachmentKind, ReceivedAttachment};
    use std::fs;
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
    fn secure_cookie_is_enabled_for_https_origin() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::unset(SECURE_COOKIE_ENV);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://chat.example.com"),
        );

        let cookie_value = build_session_cookie("token", &headers, WEB_SESSION_MAX_AGE_LONG_SECS);
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
    fn sms_phone_candidates_accept_plus_86_and_local_numbers() {
        assert_eq!(
            public_sms_phone_candidates("+8613871396421"),
            vec!["13871396421".to_string(), "+8613871396421".to_string()]
        );
        assert_eq!(
            public_sms_phone_candidates("13871396421"),
            vec!["13871396421".to_string()]
        );
        assert_eq!(aliyun_sms_phone_number("+8613871396421"), "13871396421");
        assert_eq!(aliyun_sms_phone_number("13871396421"), "13871396421");
    }

    #[test]
    fn env_force_secure_cookie_overrides_headers() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::set(SECURE_COOKIE_ENV, "true");

        // When env is set to "true", Secure flag should be on even without https headers.
        let headers = HeaderMap::new();
        let cookie = build_session_cookie("tok", &headers, WEB_SESSION_MAX_AGE_LONG_SECS)
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
        let cookie2 = build_session_cookie("tok", &https_headers, WEB_SESSION_MAX_AGE_LONG_SECS)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(
            !cookie2.contains("Secure"),
            "env=false should suppress Secure"
        );
    }

    #[test]
    fn env_force_secure_cookie_accepts_aliases_and_fails_closed() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::set(SECURE_COOKIE_ENV, "yes");
        let headers = HeaderMap::new();

        let yes_cookie = build_session_cookie("tok", &headers, WEB_SESSION_MAX_AGE_LONG_SECS)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(yes_cookie.contains("Secure"));

        unsafe { std::env::set_var(SECURE_COOKIE_ENV, "0") };
        let mut https_headers = HeaderMap::new();
        https_headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://chat.example.com"),
        );
        let zero_cookie =
            build_session_cookie("tok", &https_headers, WEB_SESSION_MAX_AGE_LONG_SECS)
                .to_str()
                .expect("cookie")
                .to_string();
        assert!(!zero_cookie.contains("Secure"));

        unsafe { std::env::set_var(SECURE_COOKIE_ENV, "maybe") };
        let invalid_cookie = build_session_cookie("tok", &headers, WEB_SESSION_MAX_AGE_LONG_SECS)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(invalid_cookie.contains("Secure"));
    }

    #[test]
    fn cookie_max_age_reflects_remember_choice() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::unset(SECURE_COOKIE_ENV);
        let headers = HeaderMap::new();

        let long_cookie = build_session_cookie("tok", &headers, WEB_SESSION_MAX_AGE_LONG_SECS)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(long_cookie.contains(&format!("Max-Age={WEB_SESSION_MAX_AGE_LONG_SECS}")));

        let short_cookie = build_session_cookie("tok", &headers, WEB_SESSION_MAX_AGE_SHORT_SECS)
            .to_str()
            .expect("cookie")
            .to_string();
        assert!(short_cookie.contains(&format!("Max-Age={WEB_SESSION_MAX_AGE_SHORT_SECS}")));
        assert_eq!(WEB_SESSION_MAX_AGE_SHORT_SECS, 86_400);
        assert_eq!(WEB_SESSION_MAX_AGE_LONG_SECS, 30 * 86_400);
    }

    #[test]
    fn empty_env_secure_cookie_falls_back_to_headers() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let _env = EnvVarGuard::set(SECURE_COOKIE_ENV, "");

        let cookie_without_https =
            build_session_cookie("tok", &HeaderMap::new(), WEB_SESSION_MAX_AGE_LONG_SECS)
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
        let cookie_with_https =
            build_session_cookie("tok", &https_headers, WEB_SESSION_MAX_AGE_LONG_SECS)
                .to_str()
                .expect("cookie")
                .to_string();
        assert!(
            cookie_with_https.contains("Secure"),
            "empty env should fall back to https headers"
        );
    }

    #[test]
    fn public_upload_path_rejects_traversal_outside_user_root() {
        let root = std::env::temp_dir().join(format!(
            "hone-public-upload-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let upload_root = root.join("public-uploads").join("web-user-a");
        fs::create_dir_all(&upload_root).expect("upload root");
        let allowed = upload_root.join("ok.txt");
        fs::write(&allowed, "ok").expect("allowed file");
        let sibling = root.join("config.yaml");
        fs::write(&sibling, "secret").expect("sibling file");

        assert!(
            validate_public_upload_path(
                &upload_root,
                None,
                "web-user-a",
                &allowed.to_string_lossy()
            )
            .is_ok()
        );
        assert!(
            validate_public_upload_path(&upload_root, None, "web-user-a", "../config.yaml")
                .is_err()
        );
        assert!(
            validate_public_upload_path(
                &upload_root,
                None,
                "web-user-a",
                &sibling.to_string_lossy()
            )
            .is_err()
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn public_chat_user_input_uses_shared_attachment_context() {
        let attachments = vec![ReceivedAttachment {
            filename: "portfolio.png".to_string(),
            content_type: Some("image/png".to_string()),
            size: 1024,
            url: "oss://bucket/public/web-user-a/portfolio.png".to_string(),
            kind: AttachmentKind::Image,
            local_path: Some(
                "/tmp/hone-agent-sandboxes/web/direct/uploads/portfolio.png".to_string(),
            ),
            error: None,
            extracted_files: vec![],
            extraction_error: None,
            pdf_text_preview: None,
            pdf_extract_error: None,
        }];

        let input = build_public_chat_user_input("帮我看持仓截图", &attachments)
            .expect("public chat input");

        assert!(input.contains("用户上传了附件"));
        assert!(input.contains("文件名=portfolio.png"));
        assert!(input.contains("本地路径="));
        assert!(input.contains("优先基于附件行里的本地可读路径理解截图/图表"));
        assert!(!input.contains("[附件:"));
        assert!(!input.contains("当前工具链"));
        assert!(!input.contains("会话数据库"));
    }

    #[test]
    fn public_chat_user_input_rejects_all_rejected_attachments() {
        let attachments = vec![ReceivedAttachment {
            filename: "large.png".to_string(),
            content_type: Some("image/png".to_string()),
            size: 5 * 1024 * 1024,
            url: "oss://bucket/public/web-user-a/large.png".to_string(),
            kind: AttachmentKind::Image,
            local_path: None,
            error: Some("附件未通过准入限制：图片大小 5.0MB 超过 3MB 上限".to_string()),
            extracted_files: vec![],
            extraction_error: None,
            pdf_text_preview: None,
            pdf_extract_error: None,
        }];

        assert!(build_public_chat_user_input("", &attachments).is_err());
    }

    #[test]
    fn public_attachment_filename_prefers_client_name_for_oss_uri() {
        let attachment = crate::types::PublicChatAttachmentInput {
            path: "oss://bucket/public/web-user-a/2026-06-08/uuid.bin".to_string(),
            name: Some("截图 组合.png".to_string()),
        };

        assert_eq!(
            public_attachment_filename(&attachment, &attachment.path),
            "attachment.png"
        );
    }
}
