//! Authenticated, read-only projection of the user-authorized community archive.
//!
//! Source-protected files never leave this route: only resources already stored
//! in Hone object storage can be streamed for inline preview.

use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use hone_core::ActorIdentity;
use hone_core::cloud_runtime::{
    CloudCommunityContentRecord, CloudCommunityResourceRecord, CloudPgRuntime, OssObjectStore,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::routes::json_error;
use crate::state::AppState;

const COMMUNITY_SOURCE: &str = "zsxq";
const COMMUNITY_EXTERNAL_ID: &str = "51115212285814";
const COMMUNITY_PUBLIC_AUTHOR: &str = "HONE 官方";
const COMMUNITY_PREVIEW_MAX_BYTES: usize = 25 * 1024 * 1024;

#[derive(Debug, Serialize)]
struct PublicCommunityContent {
    content_id: i64,
    author_name: &'static str,
    published_at: Option<String>,
    published_at_raw: Option<String>,
    content_type: String,
    body_text: String,
    body_blocks: serde_json::Value,
    crawl_status: String,
    resources: Vec<PublicCommunityResource>,
}

#[derive(Debug, Serialize)]
struct PublicCommunityResource {
    resource_id: i64,
    ordinal: i32,
    resource_kind: String,
    display_name: Option<String>,
    content_type: Option<String>,
    byte_size: Option<i64>,
    access_state: String,
}

impl From<CloudCommunityResourceRecord> for PublicCommunityResource {
    fn from(value: CloudCommunityResourceRecord) -> Self {
        Self {
            resource_id: value.resource_id,
            ordinal: value.ordinal,
            resource_kind: value.resource_kind,
            display_name: value.display_name,
            content_type: value.content_type,
            byte_size: value.byte_size,
            access_state: value.access_state,
        }
    }
}

impl From<CloudCommunityContentRecord> for PublicCommunityContent {
    fn from(value: CloudCommunityContentRecord) -> Self {
        Self {
            content_id: value.content_id,
            author_name: COMMUNITY_PUBLIC_AUTHOR,
            published_at: value.published_at,
            published_at_raw: value.published_at_raw,
            content_type: value.content_type,
            body_text: value.body_text,
            body_blocks: value.body_blocks,
            crawl_status: value.crawl_status,
            resources: value.resources.into_iter().map(Into::into).collect(),
        }
    }
}

fn safe_inline_content_type(raw: &str) -> Option<&'static str> {
    match raw
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/png" => Some("image/png"),
        "image/webp" => Some("image/webp"),
        "image/gif" => Some("image/gif"),
        "image/avif" => Some("image/avif"),
        "application/pdf" => Some("application/pdf"),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommunityQuery {
    before: Option<i64>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommunitySeenRequest {
    content_id: i64,
}

fn community_runtime(state: &AppState) -> Result<CloudPgRuntime, Response> {
    CloudPgRuntime::from_cloud_config(&state.core.config.cloud)
        .ok_or_else(|| json_error(StatusCode::SERVICE_UNAVAILABLE, "社区归档服务暂不可用"))
}

fn public_actor_storage_key(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<String, Response> {
    let user = crate::routes::public::require_public_user(state, headers)?;
    ActorIdentity::new("web", user.user_id, Option::<String>::None)
        .map(|actor| actor.storage_key())
        .map_err(|error| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("构造社区用户身份失败: {error}"),
            )
        })
}

/// GET /api/public/community?before=<content_id>&limit=20
pub(crate) async fn handle_list_community(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<CommunityQuery>,
) -> Response {
    let actor_storage_key = match public_actor_storage_key(&state, &headers) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let runtime = match community_runtime(&state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    let items = match runtime
        .list_community_contents(COMMUNITY_SOURCE, COMMUNITY_EXTERNAL_ID, query.before, limit)
        .await
    {
        Ok(value) => value,
        Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    };
    let unread = match runtime
        .community_unread_state(COMMUNITY_SOURCE, COMMUNITY_EXTERNAL_ID, &actor_storage_key)
        .await
    {
        Ok(value) => value,
        Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    };
    let next_before = (items.len() == limit)
        .then(|| items.last().map(|item| item.content_id))
        .flatten();
    let items = items
        .into_iter()
        .map(PublicCommunityContent::from)
        .collect::<Vec<_>>();
    Json(json!({
        "community": { "id": COMMUNITY_EXTERNAL_ID, "name": "HONE 官方社区" },
        "items": items,
        "next_before": next_before,
        "unread": unread.unread,
        "latest_content_id": unread.latest_content_id,
    }))
    .into_response()
}

/// POST /api/public/community/seen
pub(crate) async fn handle_mark_community_seen(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CommunitySeenRequest>,
) -> Response {
    if request.content_id <= 0 {
        return json_error(StatusCode::BAD_REQUEST, "无效的社区内容标识");
    }
    let actor_storage_key = match public_actor_storage_key(&state, &headers) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let runtime = match community_runtime(&state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match runtime
        .mark_community_seen(
            COMMUNITY_SOURCE,
            COMMUNITY_EXTERNAL_ID,
            &actor_storage_key,
            request.content_id,
        )
        .await
    {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

/// GET /api/public/community/resources/:resource_id
pub(crate) async fn handle_community_resource_preview(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(resource_id): Path<i64>,
) -> Response {
    if let Err(response) = public_actor_storage_key(&state, &headers) {
        return response;
    }
    let runtime = match community_runtime(&state) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let resource = match runtime
        .get_community_resource(COMMUNITY_SOURCE, COMMUNITY_EXTERNAL_ID, resource_id)
        .await
    {
        Ok(Some(value)) => value,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "社区资源不存在"),
        Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    };
    if resource.access_state != "stored" {
        return json_error(StatusCode::CONFLICT, "该资源受来源保护，仅保留了元数据");
    }
    let Some(uri) = resource.oss_uri.as_deref() else {
        return json_error(StatusCode::NOT_FOUND, "该资源尚未归档到对象存储");
    };
    let Some(store) = OssObjectStore::from_config(&state.core.config.cloud.oss) else {
        return json_error(StatusCode::SERVICE_UNAVAILABLE, "社区对象存储暂不可用");
    };
    let Some(key) = store.parse_managed_uri(uri) else {
        return json_error(StatusCode::INTERNAL_SERVER_ERROR, "社区资源地址无效");
    };
    if resource
        .byte_size
        .is_some_and(|size| size > COMMUNITY_PREVIEW_MAX_BYTES as i64)
    {
        return json_error(StatusCode::PAYLOAD_TOO_LARGE, "该资源超出在线预览大小上限");
    }
    let object = match store
        .get_object_limited(key, COMMUNITY_PREVIEW_MAX_BYTES)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            let status = if error.contains("大小超过允许上限") {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_GATEWAY
            };
            return json_error(status, format!("读取社区资源失败: {error}"));
        }
    };
    let content_type = resource.content_type.unwrap_or(object.content_type);
    let inline_content_type = safe_inline_content_type(&content_type);
    let mut response = Response::new(Body::from(object.bytes));
    *response.status_mut() = StatusCode::OK;
    let response_headers = response.headers_mut();
    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(inline_content_type.unwrap_or("application/octet-stream")),
    );
    response_headers.insert(
        header::CONTENT_DISPOSITION,
        if inline_content_type.is_some() {
            HeaderValue::from_static("inline")
        } else {
            HeaderValue::from_str(&format!(
                "attachment; filename=\"community-resource-{resource_id}\""
            ))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment"))
        },
    );
    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=86400"),
    );
    response_headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    response_headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "sandbox; default-src 'none'; img-src 'self' data:; style-src 'unsafe-inline'",
        ),
    );
    response_headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
    response_headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_preview_only_allows_passive_image_types_and_pdf() {
        assert_eq!(safe_inline_content_type("image/jpeg"), Some("image/jpeg"));
        assert_eq!(
            safe_inline_content_type("Application/PDF; charset=binary"),
            Some("application/pdf")
        );
        assert_eq!(safe_inline_content_type("image/svg+xml"), None);
        assert_eq!(safe_inline_content_type("text/html"), None);
        assert_eq!(safe_inline_content_type("application/javascript"), None);
    }

    #[test]
    fn public_projection_hides_source_author_and_object_storage_uri() {
        let projected = PublicCommunityContent::from(CloudCommunityContentRecord {
            content_id: 7,
            author_name: Some("来源作者".to_string()),
            published_at: None,
            published_at_raw: None,
            content_type: "post".to_string(),
            body_text: "hello".to_string(),
            body_blocks: json!([]),
            crawl_status: "complete".to_string(),
            resources: vec![CloudCommunityResourceRecord {
                resource_id: 9,
                ordinal: 0,
                resource_kind: "image".to_string(),
                display_name: Some("preview.jpg".to_string()),
                content_type: Some("image/jpeg".to_string()),
                byte_size: Some(12),
                oss_uri: Some("oss://private/internal-key".to_string()),
                access_state: "stored".to_string(),
            }],
        });

        let encoded = serde_json::to_value(projected).expect("public projection serializes");
        assert_eq!(encoded["author_name"], COMMUNITY_PUBLIC_AUTHOR);
        assert!(encoded.to_string().contains("HONE 官方"));
        assert!(!encoded.to_string().contains("来源作者"));
        assert!(!encoded.to_string().contains("oss://"));
    }
}
