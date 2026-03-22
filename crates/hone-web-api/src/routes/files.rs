use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::routes::json_error;
use crate::runtime::web_index_path;
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

    ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
}

/// GET /api/file?path=... — 代理读取本地附件（防路径穿越）
pub(crate) async fn handle_file(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ImageQuery>,
) -> impl IntoResponse {
    let Some(raw_path) = params.path else {
        return json_error(StatusCode::BAD_REQUEST, "缺少 path");
    };

    let path = match resolve_file_proxy_path(&state, &raw_path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return json_error(StatusCode::NOT_FOUND, "文件不存在");
    };

    ([(header::CONTENT_TYPE, "application/octet-stream")], bytes).into_response()
}

pub(crate) async fn handle_spa_index() -> Response {
    let index_path = web_index_path();
    match std::fs::read_to_string(&index_path) {
        Ok(index) => axum::response::Html(index).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "Hone Web assets not found at {}. Run `bun run build:web` first.",
                index_path.display()
            ),
        )
            .into_response(),
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
        &config.storage.reports_dir,
        &config.storage.x_drafts_dir,
        &config.storage.gen_images_dir,
        &config.storage.kb_dir,
    ];

    for dir in candidates {
        roots.push(PathBuf::from(dir));
    }

    roots
}

fn resolve_file_proxy_path(state: &AppState, raw_path: &str) -> Result<PathBuf, Response> {
    let raw_path = raw_path.trim();
    if raw_path.is_empty() {
        return Err(json_error(StatusCode::BAD_REQUEST, "path 为空"));
    }

    let path = raw_path.strip_prefix("file://").unwrap_or(raw_path);
    let path = Path::new(path);

    if path.is_absolute() {
        for root in file_proxy_roots(&state.core.config) {
            if let Ok(clean) = path.strip_prefix(&root) {
                let final_path = root.join(clean);
                if final_path.exists() {
                    return Ok(final_path);
                }
            }
        }
    } else {
        for root in file_proxy_roots(&state.core.config) {
            let final_path = root.join(path);
            if final_path.exists() {
                return Ok(final_path);
            }
        }
    }

    Err(json_error(StatusCode::FORBIDDEN, "路径不允许访问"))
}
