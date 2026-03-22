use std::sync::Arc;

use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use serde_json::json;
use tracing::warn;

use hone_channels::run_kb_analysis;
use hone_memory::{KbEntry, StockRow};

use crate::routes::json_error;
use crate::state::AppState;

/// GET /api/kb — 列出所有知识库条目（按上传时间倒序）
pub(crate) async fn handle_kb_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct KbListResponse {
        entries: Vec<KbEntry>,
    }
    let entries = state.core.kb_storage.list_entries().await;
    Json(KbListResponse { entries }).into_response()
}

/// GET /api/kb/:id — 获取单条知识库条目及其完整解析文本
pub(crate) async fn handle_kb_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    #[derive(Serialize)]
    struct KbDetailResponse {
        entry: KbEntry,
        parsed_text: Option<String>,
    }
    match state.core.kb_storage.get_entry(&id).await {
        Some(entry) => {
            let parsed_text = state.core.kb_storage.get_parsed_text(&id).await;
            Json(KbDetailResponse { entry, parsed_text }).into_response()
        }
        None => json_error(StatusCode::NOT_FOUND, "KB entry not found"),
    }
}

/// GET /api/kb-stock-table — 获取全局股票信息表
pub(crate) async fn handle_kb_stock_table(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct StockTableResponse {
        rows: Vec<StockRow>,
    }
    let rows = state.core.stock_table.list().await;
    Json(StockTableResponse { rows }).into_response()
}

/// PUT /api/kb-stock-table/knowledge — 更新某个标的的重点知识列表
///
/// Body: { company_name, stock_code, key_knowledge: string[] }
/// 用于前端直接编辑；`key_knowledge` 整体替换（不追加）。
pub(crate) async fn handle_update_stock_knowledge(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let company_name = body
        .get("company_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let stock_code = body
        .get("stock_code")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let key_knowledge: Vec<String> = body
        .get("key_knowledge")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    if company_name.is_empty() && stock_code.is_empty() {
        return json_error(
            axum::http::StatusCode::BAD_REQUEST,
            "company_name 和 stock_code 至少填一个",
        );
    }

    match state
        .core
        .stock_table
        .update_key_knowledge(&company_name, &stock_code, key_knowledge)
        .await
    {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

/// POST /api/kb/:id/analyze — 对指定 KB 条目同步运行股票信息分析
///
/// 使用当前配置的 agent provider（gemini_cli / function_calling 等）提取公司/股票信息，
/// upsert 到 stock_table.json。同步执行，完成后返回结果。
pub(crate) async fn handle_kb_analyze(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(entry) = state.core.kb_storage.get_entry(&id).await else {
        return json_error(StatusCode::NOT_FOUND, "KB entry not found");
    };
    let parsed_text = state
        .core
        .kb_storage
        .get_parsed_text(&id)
        .await
        .unwrap_or_default();
    if parsed_text.trim().is_empty() {
        return json_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "No parsed text available for this entry",
        );
    }
    let analyzed =
        run_kb_analysis(&state.core, &entry, &parsed_text, &state.core.stock_table).await;
    if analyzed {
        if let Err(e) = state.core.kb_storage.mark_analyzed(&id).await {
            warn!("[KB/Analyze] mark_analyzed 失败: {e}");
        }
    }
    Json(json!({ "ok": analyzed })).into_response()
}

/// DELETE /api/kb/:id — 删除指定知识库条目
pub(crate) async fn handle_kb_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.core.kb_storage.delete_entry(&id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) if e.contains("不存在") => json_error(StatusCode::NOT_FOUND, &e),
        Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

/// POST /api/kb/upload — 上传文件并解析，保存到知识库
///
/// 接受 multipart/form-data，字段 `file` 为文件内容。
/// 支持 PDF（自动提取全文）及其他类型（跳过解析）。
pub(crate) async fn handle_kb_upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    use hone_channels::attachments::{
        AttachmentKind, extract_full_pdf_text, infer_attachment_kind,
    };
    use hone_memory::KbSaveRequest;

    // 从 multipart 中取出文件字段
    let field = loop {
        match multipart.next_field().await {
            Ok(Some(f)) => {
                // 接受名为 "file" 的字段，或第一个有文件名的字段
                let name = f.name().unwrap_or("").to_string();
                let fname = f.file_name().unwrap_or("").to_string();
                if name == "file" || !fname.is_empty() {
                    break f;
                }
            }
            Ok(None) => {
                return json_error(StatusCode::BAD_REQUEST, "multipart 中未找到文件字段");
            }
            Err(e) => {
                return json_error(
                    StatusCode::BAD_REQUEST,
                    &format!("读取 multipart 失败: {e}"),
                );
            }
        }
    };

    let filename = field.file_name().unwrap_or("upload.bin").to_string();
    let content_type = field.content_type().map(|s| s.to_string());
    let kind = infer_attachment_kind(content_type.as_deref(), &filename);

    // 读取文件内容
    let bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return json_error(StatusCode::BAD_REQUEST, &format!("读取文件内容失败: {e}")),
    };
    if bytes.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "文件内容为空");
    }
    let size = bytes.len() as u32;

    // 写到临时目录
    let tmp_dir = std::env::temp_dir().join(format!("hone-kb-upload-{}", uuid::Uuid::new_v4()));
    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("创建临时目录失败: {e}"),
        );
    }
    let tmp_path = tmp_dir.join(&filename);
    if let Err(e) = tokio::fs::write(&tmp_path, &bytes).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("写临时文件失败: {e}"),
        );
    }

    // 提取文本（仅 PDF）
    let (parsed_text, parse_error) = if kind == AttachmentKind::Pdf {
        match extract_full_pdf_text(&tmp_path).await {
            Ok(text) => (Some(text), None),
            Err(e) => (None, Some(e)),
        }
    } else {
        (None, None)
    };

    let req = KbSaveRequest {
        filename: filename.clone(),
        kind: format!("{kind:?}"),
        size,
        content_type,
        channel: "console".to_string(),
        user_id: "console-user".to_string(),
        session_id: "console-upload".to_string(),
        source_path: tmp_path.clone(),
        parsed_text,
        parse_error,
    };

    let result = state.core.kb_storage.save_attachment(req).await;

    // 清理临时目录（忽略错误）
    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;

    match result {
        Ok(entry) => Json(json!({ "ok": true, "entry": entry })).into_response(),
        Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}
