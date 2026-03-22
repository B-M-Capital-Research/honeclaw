use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;
use tracing::info;
use url::form_urlencoded::byte_serialize;

use crate::routes::json_error;
use crate::state::AppState;

/// 请求体：启动深度研究
#[derive(Deserialize)]
pub(crate) struct ResearchStartRequest {
    #[serde(rename = "companyName")]
    company_name: String,
}

/// 请求体：生成 PDF
#[derive(Deserialize)]
pub(crate) struct ResearchGeneratePdfRequest {
    #[serde(rename = "taskId")]
    task_id: String,
}

/// 查询参数：下载 PDF
#[derive(Deserialize)]
pub(crate) struct ResearchDownloadQuery {
    path: String,
}

/// POST /api/research/start
/// 代理到外部 API：POST /api/pdf/deep-research/start
pub(crate) async fn handle_research_start(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResearchStartRequest>,
) -> impl IntoResponse {
    let url = format!(
        "{}/api/pdf/deep-research/start",
        state.core.config.web.research_api_base
    );
    let resp = match state
        .http_client
        .post(&url)
        .json(&json!({ "company_name": req.company_name }))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("转发请求失败: {e}") })),
            )
                .into_response();
        }
    };

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());
    (status, body).into_response()
}

/// GET /api/research/status/:task_id
/// 代理到外部 API：GET /api/pdf/deep-research/status/:task_id
pub(crate) async fn handle_research_status(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let url = format!(
        "{}/api/pdf/deep-research/status/{}",
        state.core.config.web.research_api_base, task_id
    );
    let resp = match state.http_client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("转发请求失败: {e}") })),
            )
                .into_response();
        }
    };

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());
    (status, body).into_response()
}

/// POST /api/research/generate-pdf
/// 代理到外部 API：POST /api/pdf/generate
pub(crate) async fn handle_research_generate_pdf(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResearchGeneratePdfRequest>,
) -> impl IntoResponse {
    let url = format!(
        "{}/api/pdf/generate",
        state.core.config.web.research_api_base
    );
    let resp = match state
        .http_client
        .post(&url)
        .json(&json!({ "task_id": req.task_id }))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("转发请求失败: {e}") })),
            )
                .into_response();
        }
    };

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());
    (status, body).into_response()
}

/// GET /api/research/download-pdf?path=<encoded_path>
/// 代理到外部 API：GET /api/pdf/get?path=...，将 PDF 二进制流返回给前端
pub(crate) async fn handle_research_download_pdf(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ResearchDownloadQuery>,
) -> impl IntoResponse {
    let url = format!(
        "{}/api/pdf/get?path={}",
        state.core.config.web.research_api_base,
        byte_serialize(query.path.as_bytes()).collect::<String>()
    );
    let resp = match state.http_client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("转发请求失败: {e}") })),
            )
                .into_response();
        }
    };

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());
        return (status, body).into_response();
    }

    let bytes = match resp.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            return json_error(StatusCode::BAD_GATEWAY, format!("读取响应内容失败: {e}"));
        }
    };

    info!("研究报告 PDF 下载: {} bytes", bytes.len());
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        bytes,
    )
        .into_response()
}
