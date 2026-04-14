use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;
use tracing::info;
use url::{Host, Url, form_urlencoded::byte_serialize};

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
    let url = match build_research_url(
        &state.core.config.web.research_api_base,
        "api/pdf/deep-research/start",
    ) {
        Ok(url) => url,
        Err(err) => return json_error(StatusCode::BAD_GATEWAY, err),
    };
    let resp = match state
        .http_client
        .post(url)
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
    let url = match build_research_url(
        &state.core.config.web.research_api_base,
        &format!(
            "api/pdf/deep-research/status/{}",
            encode_path_segment(&task_id)
        ),
    ) {
        Ok(url) => url,
        Err(err) => return json_error(StatusCode::BAD_GATEWAY, err),
    };
    let resp = match state.http_client.get(url).send().await {
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
    let url = match build_research_url(&state.core.config.web.research_api_base, "api/pdf/generate")
    {
        Ok(url) => url,
        Err(err) => return json_error(StatusCode::BAD_GATEWAY, err),
    };
    let resp = match state
        .http_client
        .post(url)
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
    let mut url = match build_research_url(&state.core.config.web.research_api_base, "api/pdf/get")
    {
        Ok(url) => url,
        Err(err) => return json_error(StatusCode::BAD_GATEWAY, err),
    };
    url.query_pairs_mut().append_pair("path", query.path.trim());
    let resp = match state.http_client.get(url).send().await {
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

fn build_research_url(raw_base: &str, relative_path: &str) -> Result<Url, String> {
    let mut base = validate_research_base_url(raw_base)?;
    let normalized_path = relative_path.trim_start_matches('/');
    if normalized_path.is_empty() {
        return Err("research_api_base 路径不能为空".to_string());
    }
    if !base.path().ends_with('/') {
        let path = format!("{}/", base.path().trim_end_matches('/'));
        base.set_path(&path);
    }
    base.join(normalized_path)
        .map_err(|err| format!("research_api_base 拼接失败: {err}"))
}

fn validate_research_base_url(raw_base: &str) -> Result<Url, String> {
    let trimmed = raw_base.trim();
    let url = Url::parse(trimmed).map_err(|err| format!("research_api_base 无效: {err}"))?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err("research_api_base 不能携带 query 或 fragment".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("research_api_base 不支持内嵌认证信息".to_string());
    }

    match (url.scheme(), url.host()) {
        ("https", Some(Host::Domain(host))) if !host.eq_ignore_ascii_case("localhost") => Ok(url),
        ("https", Some(Host::Ipv4(ip))) if !ip.is_loopback() => Ok(url),
        ("https", Some(Host::Ipv6(ip))) if !ip.is_loopback() => Ok(url),
        ("http", Some(Host::Domain(host))) if host.eq_ignore_ascii_case("localhost") => Ok(url),
        ("http", Some(Host::Ipv4(ip))) if ip.is_loopback() => Ok(url),
        ("http", Some(Host::Ipv6(ip))) if ip.is_loopback() => Ok(url),
        ("https" | "http", Some(_)) => {
            Err("research_api_base 仅允许 HTTPS 远端地址或 HTTP loopback 地址".to_string())
        }
        _ => Err("research_api_base 必须是有效的 HTTP(S) URL".to_string()),
    }
}

fn encode_path_segment(value: &str) -> String {
    byte_serialize(value.trim().as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::{build_research_url, validate_research_base_url};

    #[test]
    fn validate_research_base_allows_https_remote_and_loopback_http() {
        assert!(validate_research_base_url("https://research.example.com").is_ok());
        assert!(validate_research_base_url("http://127.0.0.1:3213").is_ok());
        assert!(validate_research_base_url("http://localhost:3213").is_ok());
    }

    #[test]
    fn validate_research_base_rejects_non_loopback_http_and_embedded_auth() {
        assert!(validate_research_base_url("http://10.0.0.5:3213").is_err());
        assert!(validate_research_base_url("https://user:pass@example.com").is_err());
        assert!(validate_research_base_url("ftp://research.example.com").is_err());
    }

    #[test]
    fn build_research_url_preserves_base_prefix_and_encodes_segments() {
        let url = build_research_url(
            "https://research.example.com/proxy",
            "api/pdf/deep-research/status/task%2Fwith%20space",
        )
        .expect("build url");
        assert_eq!(
            url.as_str(),
            "https://research.example.com/proxy/api/pdf/deep-research/status/task%2Fwith%20space"
        );
    }
}
