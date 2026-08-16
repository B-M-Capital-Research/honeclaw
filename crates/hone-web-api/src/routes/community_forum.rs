//! Authenticated local discussion forum.
//!
//! Forum content is an untrusted social layer. It is deliberately not exposed
//! to research retrieval, daily products, or the agent prompt path.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use axum::Json;
use axum::body::Body;
use axum::extract::{Multipart, Path as AxumPath, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::state::AppState;

const MAX_POSTS: usize = 2_000;
const MAX_VISIBLE_POSTS: usize = 50;
const MAX_COMMENTS_PER_POST: usize = 200;
const MAX_TITLE_CHARS: usize = 80;
const MAX_BODY_CHARS: usize = 5_000;
const MAX_COMMENT_CHARS: usize = 1_000;
const MAX_ATTACHMENT_BYTES: usize = 10 * 1024 * 1024;
const AUTO_HIDE_REPORTS: usize = 3;
const POST_RATE_WINDOW_SECS: i64 = 60 * 60;
const POST_RATE_LIMIT: usize = 3;
const COMMENT_RATE_WINDOW_SECS: i64 = 60 * 60;
const COMMENT_RATE_LIMIT: usize = 20;

static FORUM_WRITE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ForumStore {
    version: u8,
    posts: Vec<ForumPostRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForumPostRecord {
    id: String,
    author_key: String,
    author_label: String,
    title: String,
    body: String,
    tickers: Vec<String>,
    topics: Vec<String>,
    source_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    moderation_status: String,
    attachment: Option<ForumAttachmentRecord>,
    #[serde(default)]
    liked_by: Vec<String>,
    #[serde(default)]
    reports: Vec<ForumReportRecord>,
    #[serde(default)]
    comments: Vec<ForumCommentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForumAttachmentRecord {
    id: String,
    filename: String,
    stored_name: String,
    content_type: String,
    byte_size: usize,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForumCommentRecord {
    id: String,
    author_key: String,
    author_label: String,
    body: String,
    created_at: DateTime<Utc>,
    moderation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForumReportRecord {
    actor_key: String,
    reason: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct ForumPage {
    items: Vec<ForumPostProjection>,
    is_admin: bool,
    policy: ForumPolicyProjection,
}

#[derive(Debug, Serialize)]
struct ForumPolicyProjection {
    forum_content_is_research: bool,
    attachment_max_bytes: usize,
    auto_hide_report_count: usize,
}

#[derive(Debug, Serialize)]
struct ForumPostProjection {
    id: String,
    author_label: String,
    title: String,
    body: String,
    tickers: Vec<String>,
    topics: Vec<String>,
    source_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    moderation_status: String,
    attachment: Option<ForumAttachmentProjection>,
    like_count: usize,
    liked_by_me: bool,
    report_count: Option<usize>,
    can_delete: bool,
    comments: Vec<ForumCommentProjection>,
}

#[derive(Debug, Serialize)]
struct ForumAttachmentProjection {
    id: String,
    filename: String,
    content_type: String,
    byte_size: usize,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct ForumCommentProjection {
    id: String,
    author_label: String,
    body: String,
    created_at: DateTime<Utc>,
    moderation_status: String,
    can_delete: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCommentRequest {
    body: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReportPostRequest {
    reason: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModeratePostRequest {
    action: String,
}

#[derive(Default)]
struct CreatePostFields {
    title: String,
    body: String,
    tickers: String,
    topics: String,
    source_url: String,
    attachment_filename: String,
    attachment_content_type: String,
    attachment_bytes: Vec<u8>,
}

pub(crate) async fn handle_list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor_key = forum_actor_key(&user.user_id);
    let is_admin = state.web_auth.is_web_admin(&user.user_id).unwrap_or(false);
    match read_store(&state).await {
        Ok(store) => Json(ForumPage {
            items: visible_posts(&store, &actor_key, is_admin),
            is_admin,
            policy: ForumPolicyProjection {
                forum_content_is_research: false,
                attachment_max_bytes: MAX_ATTACHMENT_BYTES,
                auto_hide_report_count: AUTO_HIDE_REPORTS,
            },
        })
        .into_response(),
        Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

pub(crate) async fn handle_create_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let mut fields = CreatePostFields::default();
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                return crate::routes::json_error(
                    StatusCode::BAD_REQUEST,
                    format!("读取帖子失败: {error}"),
                );
            }
        };
        let name = field.name().unwrap_or_default().to_string();
        if name == "attachment" {
            fields.attachment_filename = field.file_name().unwrap_or("attachment").to_string();
            fields.attachment_content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            fields.attachment_bytes = match field.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(error) => {
                    return crate::routes::json_error(
                        StatusCode::BAD_REQUEST,
                        format!("读取附件失败: {error}"),
                    );
                }
            };
        } else {
            let value = field.text().await.unwrap_or_default();
            match name.as_str() {
                "title" => fields.title = value,
                "body" => fields.body = value,
                "tickers" => fields.tickers = value,
                "topics" => fields.topics = value,
                "source_url" => fields.source_url = value,
                _ => {}
            }
        }
    }

    let title = match bounded_required_text(&fields.title, 4, MAX_TITLE_CHARS, "标题") {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    let body = match bounded_required_text(&fields.body, 10, MAX_BODY_CHARS, "正文") {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    let source_url = match optional_http_url(&fields.source_url) {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    let attachment = match prepare_attachment(&fields) {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    let actor_key = forum_actor_key(&user.user_id);
    let now = Utc::now();
    let _guard = FORUM_WRITE_LOCK.lock().await;
    let mut store = match read_store(&state).await {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
    };
    if exceeds_rate_limit(
        store
            .posts
            .iter()
            .filter(|post| post.author_key == actor_key)
            .map(|post| post.created_at),
        now,
        POST_RATE_WINDOW_SECS,
        POST_RATE_LIMIT,
    ) {
        return crate::routes::json_error(StatusCode::TOO_MANY_REQUESTS, "每小时最多发布 3 篇帖子");
    }
    let attachment = match attachment {
        Some((record, bytes)) => {
            if let Err(error) = write_attachment(&state, &record, &bytes).await {
                return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error);
            }
            Some(record)
        }
        None => None,
    };
    let post = ForumPostRecord {
        id: Uuid::new_v4().to_string(),
        author_label: forum_author_label(&actor_key),
        author_key: actor_key,
        title,
        body,
        tickers: normalize_tickers(&fields.tickers),
        topics: normalize_topics(&fields.topics),
        source_url,
        created_at: now,
        updated_at: now,
        moderation_status: "visible".to_string(),
        attachment,
        liked_by: Vec::new(),
        reports: Vec::new(),
        comments: Vec::new(),
    };
    store.version = 1;
    store.posts.insert(0, post.clone());
    store.posts.truncate(MAX_POSTS);
    if let Err(error) = write_store(&state, &store).await {
        return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error);
    }
    Json(post_projection(&post, &post.author_key, false)).into_response()
}

pub(crate) async fn handle_toggle_like(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(post_id): AxumPath<String>,
) -> Response {
    mutate_post(&state, &headers, &post_id, |post, actor_key, _, _| {
        ensure_post_visible(post)?;
        if let Some(index) = post.liked_by.iter().position(|value| value == actor_key) {
            post.liked_by.remove(index);
        } else {
            post.liked_by.push(actor_key.to_string());
        }
        Ok(())
    })
    .await
}

pub(crate) async fn handle_comment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(post_id): AxumPath<String>,
    Json(request): Json<CreateCommentRequest>,
) -> Response {
    let body = match bounded_required_text(&request.body, 1, MAX_COMMENT_CHARS, "评论") {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    mutate_post(
        &state,
        &headers,
        &post_id,
        move |post, actor_key, _, now| {
            ensure_post_visible(post)?;
            if post.comments.len() >= MAX_COMMENTS_PER_POST {
                return Err((StatusCode::CONFLICT, "该帖评论已达上限".to_string()));
            }
            if exceeds_rate_limit(
                post.comments
                    .iter()
                    .filter(|comment| comment.author_key == actor_key)
                    .map(|comment| comment.created_at),
                now,
                COMMENT_RATE_WINDOW_SECS,
                COMMENT_RATE_LIMIT,
            ) {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    "每小时最多发布 20 条评论".to_string(),
                ));
            }
            post.comments.push(ForumCommentRecord {
                id: Uuid::new_v4().to_string(),
                author_key: actor_key.to_string(),
                author_label: forum_author_label(actor_key),
                body,
                created_at: now,
                moderation_status: "visible".to_string(),
            });
            post.updated_at = now;
            Ok(())
        },
    )
    .await
}

pub(crate) async fn handle_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(post_id): AxumPath<String>,
    Json(request): Json<ReportPostRequest>,
) -> Response {
    let reason = match bounded_required_text(&request.reason, 2, 200, "举报理由") {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    mutate_post(
        &state,
        &headers,
        &post_id,
        move |post, actor_key, _, now| {
            ensure_post_visible(post)?;
            record_report(post, actor_key, reason, now)
        },
    )
    .await
}

pub(crate) async fn handle_delete_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(post_id): AxumPath<String>,
) -> Response {
    mutate_post(
        &state,
        &headers,
        &post_id,
        |post, actor_key, is_admin, now| {
            if post.author_key != actor_key && !is_admin {
                return Err((StatusCode::FORBIDDEN, "只能删除自己的帖子".to_string()));
            }
            post.moderation_status = "deleted".to_string();
            post.updated_at = now;
            Ok(())
        },
    )
    .await
}

pub(crate) async fn handle_delete_comment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath((post_id, comment_id)): AxumPath<(String, String)>,
) -> Response {
    mutate_post(
        &state,
        &headers,
        &post_id,
        move |post, actor_key, is_admin, now| {
            let Some(comment) = post.comments.iter_mut().find(|item| item.id == comment_id) else {
                return Err((StatusCode::NOT_FOUND, "评论不存在".to_string()));
            };
            if comment.author_key != actor_key && !is_admin {
                return Err((StatusCode::FORBIDDEN, "只能删除自己的评论".to_string()));
            }
            comment.moderation_status = "deleted".to_string();
            post.updated_at = now;
            Ok(())
        },
    )
    .await
}

pub(crate) async fn handle_moderate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(post_id): AxumPath<String>,
    Json(request): Json<ModeratePostRequest>,
) -> Response {
    let action = request.action.trim().to_ascii_lowercase();
    mutate_post(&state, &headers, &post_id, move |post, _, is_admin, now| {
        if !is_admin {
            return Err((StatusCode::FORBIDDEN, "只有管理员可以审核帖子".to_string()));
        }
        post.moderation_status = match action.as_str() {
            "hide" => "hidden",
            "restore" => "visible",
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "审核动作只能是 hide 或 restore".to_string(),
                ));
            }
        }
        .to_string();
        post.updated_at = now;
        Ok(())
    })
    .await
}

pub(crate) async fn handle_attachment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath((post_id, attachment_id)): AxumPath<(String, String)>,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor_key = forum_actor_key(&user.user_id);
    let is_admin = state.web_auth.is_web_admin(&user.user_id).unwrap_or(false);
    let store = match read_store(&state).await {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
    };
    let Some(post) = store.posts.iter().find(|post| post.id == post_id) else {
        return crate::routes::json_error(StatusCode::NOT_FOUND, "帖子不存在");
    };
    if post.moderation_status != "visible" && post.author_key != actor_key && !is_admin {
        return crate::routes::json_error(StatusCode::NOT_FOUND, "附件不存在");
    }
    let Some(attachment) = post
        .attachment
        .as_ref()
        .filter(|attachment| attachment.id == attachment_id)
    else {
        return crate::routes::json_error(StatusCode::NOT_FOUND, "附件不存在");
    };
    let path = attachments_root(&state).join(&attachment.stored_name);
    let bytes = match tokio::fs::read(path).await {
        Ok(value) => value,
        Err(_) => return crate::routes::json_error(StatusCode::NOT_FOUND, "附件文件不存在"),
    };
    if hex_sha256(&bytes) != attachment.sha256 {
        return crate::routes::json_error(StatusCode::CONFLICT, "附件完整性校验失败");
    }
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    if let Ok(value) = HeaderValue::from_str(&attachment.content_type) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    let disposition = if attachment.content_type.starts_with("image/")
        || attachment.content_type == "application/pdf"
    {
        "inline"
    } else {
        "attachment"
    };
    let safe_name = attachment.filename.replace(['\r', '\n', '"'], "_");
    if let Ok(value) = HeaderValue::from_str(&format!("{disposition}; filename=\"{safe_name}\"")) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

async fn mutate_post<F>(state: &AppState, headers: &HeaderMap, post_id: &str, mutate: F) -> Response
where
    F: FnOnce(&mut ForumPostRecord, &str, bool, DateTime<Utc>) -> Result<(), (StatusCode, String)>,
{
    let user = match crate::routes::public::require_public_user(state, headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor_key = forum_actor_key(&user.user_id);
    let is_admin = state.web_auth.is_web_admin(&user.user_id).unwrap_or(false);
    let _guard = FORUM_WRITE_LOCK.lock().await;
    let mut store = match read_store(state).await {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
    };
    let Some(post) = store.posts.iter_mut().find(|post| post.id == post_id) else {
        return crate::routes::json_error(StatusCode::NOT_FOUND, "帖子不存在");
    };
    if let Err((status, error)) = mutate(post, &actor_key, is_admin, Utc::now()) {
        return crate::routes::json_error(status, error);
    }
    let projection = post_projection(post, &actor_key, is_admin);
    if let Err(error) = write_store(state, &store).await {
        return crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error);
    }
    Json(projection).into_response()
}

fn visible_posts(store: &ForumStore, actor_key: &str, is_admin: bool) -> Vec<ForumPostProjection> {
    store
        .posts
        .iter()
        .filter(|post| {
            post.moderation_status == "visible" || post.author_key == actor_key || is_admin
        })
        .take(MAX_VISIBLE_POSTS)
        .map(|post| post_projection(post, actor_key, is_admin))
        .collect()
}

fn post_projection(post: &ForumPostRecord, actor_key: &str, is_admin: bool) -> ForumPostProjection {
    ForumPostProjection {
        id: post.id.clone(),
        author_label: post.author_label.clone(),
        title: post.title.clone(),
        body: post.body.clone(),
        tickers: post.tickers.clone(),
        topics: post.topics.clone(),
        source_url: post.source_url.clone(),
        created_at: post.created_at,
        updated_at: post.updated_at,
        moderation_status: post.moderation_status.clone(),
        attachment: post
            .attachment
            .as_ref()
            .map(|attachment| ForumAttachmentProjection {
                id: attachment.id.clone(),
                filename: attachment.filename.clone(),
                content_type: attachment.content_type.clone(),
                byte_size: attachment.byte_size,
                sha256: attachment.sha256.clone(),
            }),
        like_count: post.liked_by.len(),
        liked_by_me: post.liked_by.iter().any(|value| value == actor_key),
        report_count: is_admin.then_some(post.reports.len()),
        can_delete: post.author_key == actor_key || is_admin,
        comments: post
            .comments
            .iter()
            .filter(|comment| {
                comment.moderation_status == "visible"
                    || comment.author_key == actor_key
                    || is_admin
            })
            .map(|comment| ForumCommentProjection {
                id: comment.id.clone(),
                author_label: comment.author_label.clone(),
                body: comment.body.clone(),
                created_at: comment.created_at,
                moderation_status: comment.moderation_status.clone(),
                can_delete: comment.author_key == actor_key || is_admin,
            })
            .collect(),
    }
}

fn ensure_post_visible(post: &ForumPostRecord) -> Result<(), (StatusCode, String)> {
    if post.moderation_status == "visible" {
        Ok(())
    } else {
        Err((StatusCode::CONFLICT, "该帖当前不可互动".to_string()))
    }
}

fn record_report(
    post: &mut ForumPostRecord,
    actor_key: &str,
    reason: String,
    now: DateTime<Utc>,
) -> Result<(), (StatusCode, String)> {
    if post.author_key == actor_key {
        return Err((StatusCode::BAD_REQUEST, "不能举报自己的帖子".to_string()));
    }
    if post
        .reports
        .iter()
        .any(|report| report.actor_key == actor_key)
    {
        return Err((StatusCode::CONFLICT, "你已经举报过这篇帖子".to_string()));
    }
    post.reports.push(ForumReportRecord {
        actor_key: actor_key.to_string(),
        reason,
        created_at: now,
    });
    if post.reports.len() >= AUTO_HIDE_REPORTS {
        post.moderation_status = "pending_review".to_string();
    }
    Ok(())
}

fn bounded_required_text(
    value: &str,
    min_chars: usize,
    max_chars: usize,
    field: &str,
) -> Result<String, String> {
    let normalized = value.trim().replace("\r\n", "\n");
    let count = normalized.chars().count();
    if count < min_chars {
        return Err(format!("{field}至少需要 {min_chars} 个字符"));
    }
    if count > max_chars {
        return Err(format!("{field}最多允许 {max_chars} 个字符"));
    }
    Ok(normalized)
}

fn optional_http_url(value: &str) -> Result<Option<String>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let parsed = url::Url::parse(value).map_err(|_| "来源链接格式无效".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("来源链接必须是 http 或 https 地址".to_string());
    }
    Ok(Some(parsed.to_string()))
}

fn normalize_tickers(value: &str) -> Vec<String> {
    normalize_tags(value, |token| {
        let token = token.trim_start_matches('$').to_ascii_uppercase();
        (token.len() <= 10
            && token
                .chars()
                .next()
                .is_some_and(|value| value.is_ascii_alphabetic())
            && token
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || matches!(value, '.' | '-')))
        .then_some(token)
    })
}

fn normalize_topics(value: &str) -> Vec<String> {
    normalize_tags(value, |token| {
        let token = token.trim().chars().take(30).collect::<String>();
        (!token.is_empty()).then_some(token)
    })
}

fn normalize_tags<F>(value: &str, mut normalize: F) -> Vec<String>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut seen = HashSet::new();
    value
        .split([',', '，', ';', '；', '\n'])
        .filter_map(|token| normalize(token.trim()))
        .filter(|token| seen.insert(token.clone()))
        .take(8)
        .collect()
}

fn prepare_attachment(
    fields: &CreatePostFields,
) -> Result<Option<(ForumAttachmentRecord, Vec<u8>)>, String> {
    if fields.attachment_bytes.is_empty() {
        return Ok(None);
    }
    if fields.attachment_bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err("论坛附件最大 10 MB".to_string());
    }
    let content_type = safe_attachment_type(
        &fields.attachment_content_type,
        &fields.attachment_filename,
        &fields.attachment_bytes,
    )
    .ok_or_else(|| "仅支持 PDF、Markdown、纯文本、PNG、JPEG 或 WebP".to_string())?;
    let id = Uuid::new_v4().to_string();
    let extension = safe_extension(content_type);
    let filename = sanitize_filename(&fields.attachment_filename, extension);
    let stored_name = format!("{id}.{extension}");
    Ok(Some((
        ForumAttachmentRecord {
            id,
            filename,
            stored_name,
            content_type: content_type.to_string(),
            byte_size: fields.attachment_bytes.len(),
            sha256: hex_sha256(&fields.attachment_bytes),
        },
        fields.attachment_bytes.clone(),
    )))
}

fn safe_attachment_type(raw: &str, filename: &str, bytes: &[u8]) -> Option<&'static str> {
    let raw = raw.split(';').next().unwrap_or_default().trim();
    let extension = Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match (raw, extension.as_str()) {
        ("application/pdf", "pdf") if bytes.starts_with(b"%PDF-") => Some("application/pdf"),
        ("image/png", "png") if bytes.starts_with(b"\x89PNG\r\n\x1a\n") => Some("image/png"),
        ("image/jpeg", "jpg" | "jpeg") if bytes.starts_with(&[0xff, 0xd8, 0xff]) => {
            Some("image/jpeg")
        }
        ("image/webp", "webp")
            if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" =>
        {
            Some("image/webp")
        }
        ("text/markdown" | "text/plain", "md" | "markdown" | "txt")
            if std::str::from_utf8(bytes).is_ok() =>
        {
            Some(if extension == "txt" {
                "text/plain"
            } else {
                "text/markdown"
            })
        }
        _ => None,
    }
}

fn safe_extension(content_type: &str) -> &'static str {
    match content_type {
        "application/pdf" => "pdf",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "text/markdown" => "md",
        _ => "txt",
    }
}

fn sanitize_filename(filename: &str, extension: &str) -> String {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment")
        .chars()
        .filter(|value| !value.is_control() && !matches!(value, '/' | '\\' | ':' | '"'))
        .take(80)
        .collect::<String>();
    let stem = if stem.trim().is_empty() {
        "attachment"
    } else {
        stem.trim()
    };
    format!("{stem}.{extension}")
}

fn exceeds_rate_limit<I>(timestamps: I, now: DateTime<Utc>, window_secs: i64, limit: usize) -> bool
where
    I: Iterator<Item = DateTime<Utc>>,
{
    let cutoff = now - chrono::Duration::seconds(window_secs);
    timestamps.filter(|timestamp| *timestamp >= cutoff).count() >= limit
}

fn forum_actor_key(user_id: &str) -> String {
    hex_sha256(format!("hone-forum-v1:{user_id}").as_bytes())
}

fn forum_author_label(actor_key: &str) -> String {
    let short = actor_key
        .chars()
        .take(4)
        .collect::<String>()
        .to_ascii_uppercase();
    format!("研究者 {short}")
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn forum_root(state: &AppState) -> PathBuf {
    state
        .core
        .config
        .storage
        .data_root()
        .join("community-forum")
}

fn attachments_root(state: &AppState) -> PathBuf {
    forum_root(state).join("attachments")
}

async fn read_store(state: &AppState) -> Result<ForumStore, String> {
    let path = forum_root(state).join("state.json");
    match tokio::fs::read(path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ForumStore {
            version: 1,
            posts: Vec::new(),
        }),
        Err(error) => Err(error.to_string()),
    }
}

async fn write_store(state: &AppState, store: &ForumStore) -> Result<(), String> {
    let root = forum_root(state);
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| error.to_string())?;
    let path = root.join("state.json");
    let temp = root.join(format!("state.tmp-{}", std::process::id()));
    let bytes = serde_json::to_vec_pretty(store).map_err(|error| error.to_string())?;
    tokio::fs::write(&temp, bytes)
        .await
        .map_err(|error| error.to_string())?;
    tokio::fs::rename(temp, path)
        .await
        .map_err(|error| error.to_string())
}

async fn write_attachment(
    state: &AppState,
    record: &ForumAttachmentRecord,
    bytes: &[u8],
) -> Result<(), String> {
    let root = attachments_root(state);
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| error.to_string())?;
    let path = root.join(&record.stored_name);
    tokio::fs::write(path, bytes)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(author_key: &str) -> ForumPostRecord {
        ForumPostRecord {
            id: "p1".into(),
            author_key: author_key.into(),
            author_label: forum_author_label(author_key),
            title: "HBM 供需讨论".into(),
            body: "这是一个等待官方来源确认的讨论。".into(),
            tickers: vec!["MU".into()],
            topics: vec!["HBM".into()],
            source_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            moderation_status: "visible".into(),
            attachment: None,
            liked_by: vec![],
            reports: vec![],
            comments: vec![],
        }
    }

    #[test]
    fn actor_identity_is_not_exposed_in_the_alias() {
        let user = "web:+8613800000000";
        let key = forum_actor_key(user);
        let label = forum_author_label(&key);
        assert_eq!(key.len(), 64);
        assert!(!label.contains(user));
        assert!(!label.contains("13800000000"));
    }

    #[test]
    fn tags_are_bounded_normalized_and_deduplicated() {
        assert_eq!(
            normalize_tickers("$mu, MU, sndk, 75, BAD/TICKER"),
            vec!["MU", "SNDK"]
        );
        assert_eq!(normalize_topics("HBM，存储; HBM"), vec!["HBM", "存储"]);
    }

    #[test]
    fn attachment_requires_matching_safe_type_extension_and_magic() {
        assert_eq!(
            safe_attachment_type("application/pdf", "memo.pdf", b"%PDF-1.7"),
            Some("application/pdf")
        );
        assert_eq!(
            safe_attachment_type("text/html", "memo.html", b"<script>x</script>"),
            None
        );
        assert_eq!(
            safe_attachment_type("application/pdf", "memo.pdf", b"not a pdf"),
            None
        );
    }

    #[test]
    fn hidden_posts_are_visible_only_to_owner_or_admin() {
        let mut record = post("owner");
        record.moderation_status = "pending_review".into();
        let store = ForumStore {
            version: 1,
            posts: vec![record],
        };
        assert!(visible_posts(&store, "reader", false).is_empty());
        assert_eq!(visible_posts(&store, "owner", false).len(), 1);
        assert_eq!(visible_posts(&store, "admin", true).len(), 1);
    }

    #[test]
    fn projection_deduplicates_like_state_and_hides_reports_from_members() {
        let mut record = post("owner");
        record.liked_by = vec!["reader".into()];
        record.reports = vec![ForumReportRecord {
            actor_key: "reporter".into(),
            reason: "spam".into(),
            created_at: Utc::now(),
        }];
        let member = post_projection(&record, "reader", false);
        assert_eq!(member.like_count, 1);
        assert!(member.liked_by_me);
        assert_eq!(member.report_count, None);
        let admin = post_projection(&record, "admin", true);
        assert_eq!(admin.report_count, Some(1));
    }

    #[test]
    fn report_threshold_is_three_unique_actors() {
        let mut record = post("owner");
        let now = Utc::now();
        record_report(&mut record, "a", "spam".into(), now).unwrap();
        assert!(record_report(&mut record, "a", "again".into(), now).is_err());
        record_report(&mut record, "b", "spam".into(), now).unwrap();
        assert_eq!(record.moderation_status, "visible");
        record_report(&mut record, "c", "spam".into(), now).unwrap();
        assert_eq!(record.moderation_status, "pending_review");
    }

    #[test]
    fn input_limits_and_rate_limit_are_fail_closed() {
        assert!(bounded_required_text("abc", 4, 80, "标题").is_err());
        assert!(bounded_required_text(&"a".repeat(81), 4, 80, "标题").is_err());
        let now = Utc::now();
        assert!(exceeds_rate_limit(
            [now, now, now].into_iter(),
            now,
            POST_RATE_WINDOW_SECS,
            POST_RATE_LIMIT
        ));
    }
}
