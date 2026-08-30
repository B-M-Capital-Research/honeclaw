//! Capability minting for the public media edge.
//!
//! Chat image bytes do not belong on this origin. A pasted screenshot used to
//! cross the Pacific twice to be stored and twice more to be rendered, because
//! both directions were proxied through this process in us-central1. The edge
//! Worker at `hone-claw.com/_media/v1/*` moves the bytes to the nearest
//! Cloudflare PoP and talks to R2 directly; all this module does is hand out
//! narrow, short-lived, signed permission slips.
//!
//! Two capability shapes, deliberately different:
//!
//! * A **read session** is an `HttpOnly; Secure; SameSite=Strict` cookie scoped
//!   to `/_media/v1/`, carrying the caller's own upload prefix. It never appears
//!   in a URL, so it cannot leak through `Referer`, browser history, or an
//!   intermediary's access log the way a signed query string would.
//! * An **upload grant** is a single-use token for one exact object key, bound
//!   to one content type and one byte ceiling. The client never chooses a key,
//!   so it cannot overwrite its own history or aim at another tenant, and the
//!   edge refuses to replace an object that already exists.
//!
//! The signing secret is shared with the Worker and lives only in the
//! environment. Nothing here trusts a client-supplied path, filename extension,
//! or content type beyond using it to pick from a fixed allowlist.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use hone_core::cloud_runtime::sanitize_key_component;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

use crate::routes::json_error;
use crate::routes::public::{
    PUBLIC_UPLOAD_MAX_BYTES, PUBLIC_UPLOAD_MAX_FILES, require_public_user, sanitize_attachment_name,
};
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

pub(crate) const MEDIA_EDGE_COOKIE: &str = "hone_media_edge";
pub(crate) const MEDIA_EDGE_COOKIE_PATH: &str = "/_media/v1/";
const MEDIA_EDGE_BASE_PATH: &str = "/_media/v1";
const MEDIA_EDGE_AUDIENCE: &str = "hone-media-edge-v1";
const MEDIA_EDGE_TOKEN_VERSION: u8 = 1;
const MEDIA_EDGE_SECRET_MIN_BYTES: usize = 32;
const MEDIA_EDGE_SECRET_MAX_BYTES: usize = 1024;

/// How many upload grants one account may request per window. A grant is cheap
/// to mint but authorizes a write, so the ceiling exists to keep a stolen
/// session from enumerating storage rather than to shape normal use: pasting
/// four images at once costs one request.
const GRANT_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const GRANT_RATE_LIMIT_MAX_REQUESTS: usize = 30;
const GRANT_RATE_LIMIT_MAX_TRACKED_USERS: usize = 10_000;

/// The only content types the edge will store. `image/svg+xml` is absent and
/// must stay absent: an SVG served from `hone-claw.com` is same-origin script.
const ALLOWED_IMAGE_TYPES: &[(&str, &str)] = &[
    ("image/png", "png"),
    ("image/jpeg", "jpg"),
    ("image/webp", "webp"),
    ("image/gif", "gif"),
];

#[derive(Debug, Serialize)]
struct MediaSessionProjection<'a> {
    enabled: bool,
    mode: &'a str,
    base_path: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MediaUploadGrantRequest {
    #[serde(default)]
    files: Vec<MediaUploadGrantFile>,
}

#[derive(Debug, Deserialize)]
struct MediaUploadGrantFile {
    #[serde(default)]
    name: Option<String>,
    content_type: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct MediaUploadGrant {
    /// Same-origin path the browser PUTs to.
    upload_path: String,
    /// Capability for exactly that object, sent in `X-Hone-Media-Token`.
    token: String,
    /// `oss://` URI to hand back to `/api/public/chat` once the PUT succeeds.
    path: String,
    name: String,
    kind: &'static str,
    content_type: &'static str,
    size: u64,
}

#[derive(Debug, Serialize)]
struct MediaUploadGrantProjection<'a> {
    enabled: bool,
    mode: &'a str,
    base_path: &'static str,
    expires_at: i64,
    uploads: Vec<MediaUploadGrant>,
}

#[derive(Debug, Serialize)]
struct MediaReadClaims<'a> {
    v: u8,
    aud: &'a str,
    op: &'a str,
    sub: &'a str,
    pfx: &'a str,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Serialize)]
struct MediaWriteClaims<'a> {
    v: u8,
    aud: &'a str,
    op: &'a str,
    sub: &'a str,
    pfx: &'a str,
    key: &'a str,
    ct: &'a str,
    max: u64,
    iat: i64,
    exp: i64,
}

fn sign_claims<T: Serialize>(claims: &T, secret: &[u8]) -> String {
    let payload = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(claims).expect("media edge claims always serialize as JSON"));
    let mut signer =
        HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts secrets of any length");
    signer.update(payload.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(signer.finalize().into_bytes());
    format!("{payload}.{signature}")
}

fn encode_media_read_session(
    user_id: &str,
    owner_prefix: &str,
    issued_at: i64,
    ttl_secs: u64,
    secret: &[u8],
) -> (String, i64) {
    let expires_at = issued_at.saturating_add(ttl_secs as i64);
    let token = sign_claims(
        &MediaReadClaims {
            v: MEDIA_EDGE_TOKEN_VERSION,
            aud: MEDIA_EDGE_AUDIENCE,
            op: "get",
            sub: user_id,
            pfx: owner_prefix,
            iat: issued_at,
            exp: expires_at,
        },
        secret,
    );
    (token, expires_at)
}

fn encode_media_write_grant(
    user_id: &str,
    owner_prefix: &str,
    key: &str,
    content_type: &str,
    max_bytes: u64,
    issued_at: i64,
    ttl_secs: u64,
    secret: &[u8],
) -> (String, i64) {
    let expires_at = issued_at.saturating_add(ttl_secs as i64);
    let token = sign_claims(
        &MediaWriteClaims {
            v: MEDIA_EDGE_TOKEN_VERSION,
            aud: MEDIA_EDGE_AUDIENCE,
            op: "put",
            sub: user_id,
            pfx: owner_prefix,
            key,
            ct: content_type,
            max: max_bytes,
            iat: issued_at,
            exp: expires_at,
        },
        secret,
    );
    (token, expires_at)
}

fn valid_media_edge_secret(secret: &str) -> bool {
    (MEDIA_EDGE_SECRET_MIN_BYTES..=MEDIA_EDGE_SECRET_MAX_BYTES)
        .contains(&secret.trim().as_bytes().len())
}

pub(crate) fn build_media_edge_cookie(token: &str, max_age_secs: u64) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{MEDIA_EDGE_COOKIE}={token}; Path={MEDIA_EDGE_COOKIE_PATH}; HttpOnly; Secure; SameSite=Strict; Max-Age={max_age_secs}"
    ))
    .expect("base64url media session always forms a valid cookie")
}

pub(crate) fn clear_media_edge_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "hone_media_edge=; Path=/_media/v1/; HttpOnly; Secure; SameSite=Strict; Max-Age=0",
    )
}

fn personal_json_response<T: Serialize>(value: T) -> Response {
    let mut response = Json(value).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Cookie"));
    response
}

fn session_response(
    enabled: bool,
    mode: &str,
    expires_at: Option<i64>,
    cookie: HeaderValue,
) -> Response {
    let mut response = personal_json_response(MediaSessionProjection {
        enabled,
        mode,
        base_path: MEDIA_EDGE_BASE_PATH,
        expires_at,
    });
    response.headers_mut().append(header::SET_COOKIE, cookie);
    response
}

/// A user's R2 upload root, `public-uploads/<sanitized-user-id>/`.
///
/// Built from the same `sanitize_key_component` the object store uses, so the
/// prefix in a token always matches the keys the store actually produces. The
/// Worker re-checks the shape of whatever arrives, which is what keeps a bug
/// here from becoming cross-tenant access on its own.
fn owner_prefix(oss: &hone_core::cloud_runtime::OssObjectStore, user_id: &str) -> String {
    let sample = oss.public_upload_key(user_id, "d", "n");
    let mut segments = sample.split('/');
    let root = segments.next().unwrap_or("public-uploads");
    format!("{root}/{}/", sanitize_key_component(user_id))
}

fn canonical_image_type(raw: &str) -> Option<(&'static str, &'static str)> {
    let normalized = raw
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let normalized = if normalized == "image/jpg" {
        "image/jpeg".to_string()
    } else {
        normalized
    };
    ALLOWED_IMAGE_TYPES
        .iter()
        .find(|(mime, _)| *mime == normalized)
        .copied()
}

/// Stored name for one upload: `<uuid>-<sanitized stem>.<extension of the
/// validated content type>`.
///
/// The extension comes from the content type rather than from the client's
/// filename on purpose. Downstream, both `classify_attachment_kind` and
/// `content_type_for_attachment` read the extension off this name, so letting a
/// client pair `report.txt` with `image/png` would leave the stored object and
/// the chat pipeline disagreeing about what the bytes are.
fn stored_upload_name(raw_name: Option<&str>, extension: &str) -> String {
    let sanitized = sanitize_attachment_name(raw_name.unwrap_or("image").trim());
    let stem = sanitized
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .filter(|stem| !stem.is_empty())
        .unwrap_or(sanitized.as_str());
    let stem: String = stem.chars().take(64).collect();
    let stem = stem.trim_matches('_');
    let stem = if stem.is_empty() { "image" } else { stem };
    format!("{stem}.{extension}")
}

fn grant_rate_limiter() -> &'static Mutex<HashMap<String, Vec<Instant>>> {
    static LIMITER: OnceLock<Mutex<HashMap<String, Vec<Instant>>>> = OnceLock::new();
    LIMITER.get_or_init(|| Mutex::new(HashMap::new()))
}

fn exceeds_grant_rate_limit(user_id: &str, now: Instant) -> bool {
    let Ok(mut limiter) = grant_rate_limiter().lock() else {
        // A poisoned lock must not become a way to bypass the ceiling.
        return true;
    };
    limiter.retain(|_, hits| {
        hits.retain(|hit| now.duration_since(*hit) < GRANT_RATE_LIMIT_WINDOW);
        !hits.is_empty()
    });
    let hits = limiter.entry(user_id.to_string()).or_default();
    if hits.len() >= GRANT_RATE_LIMIT_MAX_REQUESTS {
        return true;
    }
    if limiter.len() > GRANT_RATE_LIMIT_MAX_TRACKED_USERS {
        // Bounded memory beats perfect accounting; the window is short enough
        // that dropping the table only relaxes the ceiling for one window.
        limiter.clear();
        return false;
    }
    limiter.entry(user_id.to_string()).or_default().push(now);
    false
}

#[cfg(test)]
pub(crate) fn reset_grant_rate_limiter_for_tests() {
    if let Ok(mut limiter) = grant_rate_limiter().lock() {
        limiter.clear();
    }
}

/// POST /api/public/media/session
///
/// Issues (or clears) the read cookie the edge Worker checks on `GET`.
pub(crate) async fn handle_media_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let delivery = &state.core.config.cloud.media_delivery;
    let mode = delivery.effective_mode();
    let secret = delivery.resolved_secret();
    let oss = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss);

    if !mode.issues_capabilities() || !valid_media_edge_secret(&secret) || oss.is_none() {
        return session_response(false, mode.as_str(), None, clear_media_edge_cookie());
    }
    let oss = oss.expect("checked above");

    let user = match require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => {
            let mut response = response;
            response
                .headers_mut()
                .append(header::SET_COOKIE, clear_media_edge_cookie());
            return response;
        }
    };

    let ttl_secs = delivery.effective_read_ttl_secs();
    let (token, expires_at) = encode_media_read_session(
        &user.user_id,
        &owner_prefix(&oss, &user.user_id),
        chrono::Utc::now().timestamp(),
        ttl_secs,
        secret.as_bytes(),
    );
    session_response(
        true,
        mode.as_str(),
        Some(expires_at),
        build_media_edge_cookie(&token, ttl_secs),
    )
}

/// POST /api/public/media/upload-grant
///
/// Mints one single-use upload capability per file and refreshes the read
/// cookie in the same response, so a paste costs one origin round trip before
/// the bytes go straight to the edge.
pub(crate) async fn handle_media_upload_grant(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<MediaUploadGrantRequest>,
) -> Response {
    let delivery = &state.core.config.cloud.media_delivery;
    let mode = delivery.effective_mode();
    let secret = delivery.resolved_secret();
    let oss = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss);

    if !mode.issues_capabilities() || !valid_media_edge_secret(&secret) || oss.is_none() {
        return personal_json_response(MediaUploadGrantProjection {
            enabled: false,
            mode: mode.as_str(),
            base_path: MEDIA_EDGE_BASE_PATH,
            expires_at: 0,
            uploads: Vec::new(),
        });
    }
    let oss = oss.expect("checked above");

    let user = match require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if request.files.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "未指定要上传的文件");
    }
    if request.files.len() > PUBLIC_UPLOAD_MAX_FILES {
        return json_error(
            StatusCode::BAD_REQUEST,
            format!("单次最多上传 {PUBLIC_UPLOAD_MAX_FILES} 个附件"),
        );
    }
    if exceeds_grant_rate_limit(&user.user_id, Instant::now()) {
        return json_error(
            StatusCode::TOO_MANY_REQUESTS,
            "上传请求过于频繁，请稍后再试",
        );
    }

    let issued_at = chrono::Utc::now().timestamp();
    let write_ttl = delivery.effective_write_ttl_secs();
    let read_ttl = delivery.effective_read_ttl_secs();
    let day = hone_core::local_now().format("%Y-%m-%d").to_string();
    let prefix = owner_prefix(&oss, &user.user_id);

    let mut uploads = Vec::with_capacity(request.files.len());
    for file in &request.files {
        let Some((content_type, extension)) = canonical_image_type(&file.content_type) else {
            return json_error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "边缘直传仅支持 PNG / JPEG / WebP / GIF 图片",
            );
        };
        if file.size == 0 {
            return json_error(StatusCode::BAD_REQUEST, "附件内容为空");
        }
        if file.size > PUBLIC_UPLOAD_MAX_BYTES as u64 {
            return json_error(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "单个附件过大，最大 {} MB",
                    PUBLIC_UPLOAD_MAX_BYTES / 1024 / 1024
                ),
            );
        }

        let display_name = stored_upload_name(file.name.as_deref(), extension);
        let stored_name = format!("{}-{}", Uuid::new_v4().simple(), display_name);
        let key = oss.public_upload_key(&user.user_id, &day, &stored_name);
        // A key that does not sit under the caller's own prefix means the store
        // and this module disagree about identity; refuse rather than sign it.
        if !key.starts_with(&prefix) {
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "上传路径构造失败");
        }

        let (token, _) = encode_media_write_grant(
            &user.user_id,
            &prefix,
            &key,
            content_type,
            file.size,
            issued_at,
            write_ttl,
            secret.as_bytes(),
        );
        uploads.push(MediaUploadGrant {
            upload_path: format!("{MEDIA_EDGE_BASE_PATH}/o/{key}"),
            token,
            path: oss.object_uri(&key),
            name: display_name,
            kind: "image",
            content_type,
            size: file.size,
        });
    }

    let (read_token, read_expires_at) = encode_media_read_session(
        &user.user_id,
        &prefix,
        issued_at,
        read_ttl,
        secret.as_bytes(),
    );
    let mut response = personal_json_response(MediaUploadGrantProjection {
        enabled: true,
        mode: mode.as_str(),
        base_path: MEDIA_EDGE_BASE_PATH,
        expires_at: read_expires_at,
        uploads,
    });
    response.headers_mut().append(
        header::SET_COOKIE,
        build_media_edge_cookie(&read_token, read_ttl),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const SECRET: &[u8] = b"test-only-media-edge-secret-32b!!";

    fn decode_payload(token: &str) -> Value {
        let payload = token.split('.').next().expect("payload segment");
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).expect("base64url")).expect("json")
    }

    /// The Worker verifies HMAC-SHA256 over the base64url payload segment. If
    /// this vector ever changes, every issued cookie and grant stops verifying
    /// at the edge, so pin it here the way the community edge does.
    #[test]
    fn read_session_matches_cross_language_golden_vector() {
        let (token, expires_at) = encode_media_read_session(
            "web-user-1",
            "public-uploads/web-user-1/",
            1_700_000_000,
            900,
            SECRET,
        );
        assert_eq!(expires_at, 1_700_000_900);
        assert_eq!(
            token,
            "eyJ2IjoxLCJhdWQiOiJob25lLW1lZGlhLWVkZ2UtdjEiLCJvcCI6ImdldCIsInN1YiI6IndlYi11c2VyLTEiLCJwZngiOiJwdWJsaWMtdXBsb2Fkcy93ZWItdXNlci0xLyIsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjoxNzAwMDAwOTAwfQ.cVELwXFuzF90eMPYKsVnPYAIXvx2Gj53NnuMyhS6VLg"
        );
    }

    #[test]
    fn write_grant_binds_key_content_type_and_ceiling() {
        let (token, expires_at) = encode_media_write_grant(
            "web-user-1",
            "public-uploads/web-user-1/",
            "public-uploads/web-user-1/2026-08-30/abc-image.png",
            "image/png",
            4096,
            1_700_000_000,
            120,
            SECRET,
        );
        assert_eq!(expires_at, 1_700_000_120);
        // Pinned in workers/public-media-edge/test/index.test.ts as well, so a
        // change to either side's claim order or encoding fails a test rather
        // than silently breaking uploads at the edge.
        assert_eq!(
            token,
            "eyJ2IjoxLCJhdWQiOiJob25lLW1lZGlhLWVkZ2UtdjEiLCJvcCI6InB1dCIsInN1YiI6IndlYi11c2VyLTEiLCJwZngiOiJwdWJsaWMtdXBsb2Fkcy93ZWItdXNlci0xLyIsImtleSI6InB1YmxpYy11cGxvYWRzL3dlYi11c2VyLTEvMjAyNi0wOC0zMC9hYmMtaW1hZ2UucG5nIiwiY3QiOiJpbWFnZS9wbmciLCJtYXgiOjQwOTYsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjoxNzAwMDAwMTIwfQ.7vEJxs8VnJ4ENcOzGkKpcfScPUDGD4NJJI8NCgxNfjk"
        );
        let payload = decode_payload(&token);
        assert_eq!(payload["op"], "put");
        assert_eq!(payload["aud"], "hone-media-edge-v1");
        assert_eq!(payload["pfx"], "public-uploads/web-user-1/");
        assert_eq!(
            payload["key"],
            "public-uploads/web-user-1/2026-08-30/abc-image.png"
        );
        assert_eq!(payload["ct"], "image/png");
        assert_eq!(payload["max"], 4096);
    }

    #[test]
    fn write_grant_lifetime_stays_inside_the_edge_ceiling() {
        // The Worker rejects a lifetime over MAX_WRITE_TOKEN_LIFETIME_SECONDS,
        // so the clamp here is what keeps a misconfigured TTL from silently
        // producing tokens the edge will never accept.
        let config = hone_core::config::MediaDeliveryConfig {
            write_ttl_secs: 86_400,
            read_ttl_secs: 86_400,
            ..Default::default()
        };
        assert_eq!(
            config.effective_write_ttl_secs(),
            hone_core::config::MediaDeliveryConfig::MAX_WRITE_TTL_SECS
        );
        assert_eq!(
            config.effective_read_ttl_secs(),
            hone_core::config::MediaDeliveryConfig::MAX_READ_TTL_SECS
        );
    }

    #[test]
    fn only_the_image_allowlist_is_accepted() {
        assert_eq!(
            canonical_image_type("image/png"),
            Some(("image/png", "png"))
        );
        assert_eq!(
            canonical_image_type("IMAGE/JPG; charset=binary"),
            Some(("image/jpeg", "jpg"))
        );
        assert_eq!(
            canonical_image_type("image/webp"),
            Some(("image/webp", "webp"))
        );
        for rejected in [
            "image/svg+xml",
            "text/html",
            "application/pdf",
            "image/bmp",
            "",
            "image/png/../../svg",
        ] {
            assert_eq!(canonical_image_type(rejected), None, "{rejected}");
        }
    }

    #[test]
    fn stored_name_takes_its_extension_from_the_content_type() {
        assert_eq!(stored_upload_name(Some("report.txt"), "png"), "report.png");
        assert_eq!(
            stored_upload_name(Some("../../etc/passwd"), "png"),
            "passwd.png"
        );
        assert_eq!(stored_upload_name(Some("  "), "webp"), "attachment.webp");
        assert_eq!(stored_upload_name(None, "gif"), "image.gif");
        // A fully non-ASCII name sanitizes down to the shared fallback, which is
        // the same thing the existing origin upload path does with it.
        assert_eq!(
            stored_upload_name(Some("屏幕截图.png"), "png"),
            "attachment.png"
        );
        assert_eq!(stored_upload_name(Some(".hidden"), "png"), "attachment.png");
    }

    #[test]
    fn stored_names_stay_inside_one_key_segment() {
        for raw in [
            "../../secret.png",
            "a/b/c.png",
            "x\\y.png",
            "name with spaces.png",
            &"n".repeat(500),
        ] {
            let name = stored_upload_name(Some(raw), "png");
            assert!(!name.contains('/'), "{name}");
            assert!(!name.contains('\\'), "{name}");
            assert_ne!(name, "..");
            assert!(
                name.chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.')),
                "{name}"
            );
        }
    }

    #[test]
    fn secret_length_is_checked_on_utf8_byte_boundaries() {
        assert!(!valid_media_edge_secret(&"x".repeat(31)));
        assert!(valid_media_edge_secret(&"x".repeat(32)));
        assert!(valid_media_edge_secret(&"x".repeat(1024)));
        assert!(!valid_media_edge_secret(&"x".repeat(1025)));
        // 11 three-byte characters are 33 bytes even though they are 11 chars.
        assert!(valid_media_edge_secret(&"密".repeat(11)));
        assert!(!valid_media_edge_secret(&"密".repeat(10)));
    }

    #[test]
    fn cookies_are_httponly_secure_strict_and_path_scoped() {
        let cookie = build_media_edge_cookie("payload.signature", 900);
        let cookie = cookie.to_str().expect("ascii cookie");
        assert!(cookie.starts_with("hone_media_edge=payload.signature;"));
        assert!(cookie.contains("Path=/_media/v1/"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Strict"));
        assert!(cookie.contains("Max-Age=900"));

        let cleared = clear_media_edge_cookie();
        let cleared = cleared.to_str().expect("ascii cookie");
        assert!(cleared.contains("Max-Age=0"));
        assert!(cleared.contains("Path=/_media/v1/"));
    }

    #[test]
    fn grant_rate_limit_closes_after_the_ceiling_and_reopens_after_the_window() {
        reset_grant_rate_limiter_for_tests();
        let now = Instant::now();
        for attempt in 0..GRANT_RATE_LIMIT_MAX_REQUESTS {
            assert!(
                !exceeds_grant_rate_limit("web-user-rate", now),
                "attempt {attempt} should be allowed"
            );
        }
        assert!(exceeds_grant_rate_limit("web-user-rate", now));
        // A different account is unaffected.
        assert!(!exceeds_grant_rate_limit("web-user-other", now));
        assert!(!exceeds_grant_rate_limit(
            "web-user-rate",
            now + GRANT_RATE_LIMIT_WINDOW + Duration::from_secs(1)
        ));
        reset_grant_rate_limiter_for_tests();
    }
}
