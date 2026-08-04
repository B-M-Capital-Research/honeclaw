use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::routes::json_error;
use crate::state::AppState;
use crate::types::ImageQuery;

static LOGO_SVG: &str = include_str!("../../../../logo.svg");

/// GET /logo.svg — 返回 Hone Logo
pub(crate) async fn handle_logo() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/svg+xml")], LOGO_SVG)
}

/// GET /api/image?path=... — 代理读取本地图片（防路径穿越）
pub(crate) async fn handle_image(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ImageQuery>,
) -> impl IntoResponse {
    let Some(raw_path) = params.path else {
        return json_error(StatusCode::BAD_REQUEST, "缺少 path");
    };

    if let Some(response) = handle_oss_proxy(&state, &raw_path).await {
        return response;
    }

    let path = match resolve_file_proxy_path(&state, &raw_path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return json_error(StatusCode::NOT_FOUND, "图片不存在");
    };

    let content_type = if raw_path.ends_with(".png") {
        "image/png"
    } else if raw_path.ends_with(".jpg") || raw_path.ends_with(".jpeg") {
        "image/jpeg"
    } else if raw_path.ends_with(".gif") {
        "image/gif"
    } else if raw_path.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    };

    (
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                "private, max-age=31536000, immutable",
            ),
        ],
        bytes,
    )
        .into_response()
}

/// GET /api/file?path=... — 代理读取本地附件（防路径穿越）
pub(crate) async fn handle_file(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ImageQuery>,
) -> impl IntoResponse {
    let Some(raw_path) = params.path else {
        return json_error(StatusCode::BAD_REQUEST, "缺少 path");
    };

    if let Some(response) = handle_oss_proxy(&state, &raw_path).await {
        return response;
    }

    let path = match resolve_file_proxy_path(&state, &raw_path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return json_error(StatusCode::NOT_FOUND, "文件不存在");
    };

    let content_type = content_type_for_download(&path);
    let disposition = content_disposition_for_download(&path);
    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
            (header::CONTENT_DISPOSITION, disposition),
            (
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, max-age=31536000, immutable"),
            ),
        ],
        bytes,
    )
        .into_response()
}

fn content_type_for_download(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => "text/csv; charset=utf-8",
        "json" => "application/json",
        "md" | "txt" => "text/plain; charset=utf-8",
        "pdf" => "application/pdf",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

fn content_disposition_for_download(path: &Path) -> HeaderValue {
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("download.bin");
    let ascii_fallback = filename
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let encoded = filename
        .as_bytes()
        .iter()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'-' | b'_') {
                char::from(*byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect::<String>();
    HeaderValue::from_str(&format!(
        "attachment; filename=\"{ascii_fallback}\"; filename*=UTF-8''{encoded}"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static("attachment"))
}

async fn handle_oss_proxy(state: &AppState, raw_path: &str) -> Option<Response> {
    if !raw_path.trim().starts_with("oss://") {
        return None;
    }
    let Some(client) = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss)
    else {
        return Some(json_error(StatusCode::FORBIDDEN, "OSS 未配置"));
    };
    let Some(key) = client.parse_managed_uri(raw_path) else {
        return Some(json_error(StatusCode::FORBIDDEN, "OSS 路径不允许访问"));
    };
    match client.get_object(key).await {
        Ok(object) => Some(
            (
                [
                    (header::CONTENT_TYPE, object.content_type.as_str()),
                    (
                        header::CACHE_CONTROL,
                        "private, max-age=31536000, immutable",
                    ),
                ],
                object.bytes,
            )
                .into_response(),
        ),
        Err(error) => Some(json_error(StatusCode::BAD_GATEWAY, error)),
    }
}

fn file_proxy_roots(config: &hone_core::config::HoneConfig) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let sessions_dir = PathBuf::from(&config.storage.sessions_dir);
    if let Some(parent) = sessions_dir.parent() {
        roots.push(parent.to_path_buf());
    }

    let candidates = [
        &config.storage.sessions_dir,
        &config.storage.portfolio_dir,
        &config.storage.cron_jobs_dir,
        &config.storage.gen_images_dir,
    ];

    for dir in candidates {
        roots.push(PathBuf::from(dir));
    }

    roots.push(hone_channels::sandbox_base_dir());

    roots
}

fn resolve_file_proxy_path(state: &AppState, raw_path: &str) -> Result<PathBuf, Response> {
    let raw_path = raw_path.trim();
    if raw_path.is_empty() {
        return Err(json_error(StatusCode::BAD_REQUEST, "path 为空"));
    }

    let path = Path::new(raw_path.strip_prefix("file://").unwrap_or(raw_path));
    if let Some(path) = resolve_path_within_roots(path, &file_proxy_roots(&state.core.config)) {
        return Ok(path);
    }

    Err(json_error(StatusCode::FORBIDDEN, "路径不允许访问"))
}

fn resolve_path_within_roots(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    for root in roots {
        let canonical_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.clone());
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            canonical_root.join(path)
        };
        let Ok(canonical_candidate) = std::fs::canonicalize(candidate) else {
            continue;
        };
        if canonical_candidate.is_file() && canonical_candidate.starts_with(&canonical_root) {
            return Some(canonical_candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        content_disposition_for_download, content_type_for_download, resolve_path_within_roots,
    };

    #[test]
    fn generated_pdf_download_uses_pdf_mime_and_utf8_filename() {
        let path = Path::new("ANET-财报前瞻.pdf");

        assert_eq!(content_type_for_download(path), "application/pdf");
        let header = content_disposition_for_download(path);
        let disposition = header.to_str().expect("valid content disposition");
        assert!(disposition.starts_with("attachment;"));
        assert!(disposition.contains("filename*=UTF-8''ANET-"));
        assert!(disposition.ends_with(".pdf"));
    }

    #[test]
    fn unknown_generated_file_download_is_binary() {
        assert_eq!(
            content_type_for_download(Path::new("artifact.bin")),
            "application/octet-stream"
        );
    }

    #[test]
    fn file_proxy_canonicalizes_root_aliases_without_allowing_symlink_escape() {
        let temp = std::env::temp_dir().join(format!(
            "hone-file-proxy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        let allowed = temp.join("allowed");
        let outside = temp.join("outside");
        std::fs::create_dir_all(&allowed).expect("create allowed root");
        std::fs::create_dir_all(&outside).expect("create outside root");
        let own = allowed.join("report.pdf");
        let foreign = outside.join("secret.pdf");
        std::fs::write(&own, b"%PDF-own").expect("write own PDF");
        std::fs::write(&foreign, b"%PDF-foreign").expect("write foreign PDF");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let alias = temp.join("allowed-alias");
            symlink(&allowed, &alias).expect("link allowed root alias");
            assert_eq!(
                resolve_path_within_roots(&own, &[alias]),
                Some(std::fs::canonicalize(&own).expect("canonical own PDF"))
            );

            let escape = allowed.join("escape.pdf");
            symlink(&foreign, &escape).expect("link escaping file");
            assert_eq!(resolve_path_within_roots(&escape, &[allowed.clone()]), None);
        }

        assert_eq!(
            resolve_path_within_roots(Path::new("report.pdf"), &[allowed]),
            Some(std::fs::canonicalize(&own).expect("canonical relative PDF"))
        );
        let _ = std::fs::remove_dir_all(PathBuf::from(temp));
    }
}
