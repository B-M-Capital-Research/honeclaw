use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use hone_memory::{AuditQueryFilter, LlmAuditStorage};

use crate::routes::json_error;
use crate::state::AppState;

/// GET /api/llm-audit
pub(crate) async fn handle_llm_audit_list(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<AuditQueryFilter>,
) -> impl IntoResponse {
    let db_path = &state.core.config.storage.llm_audit_db_path;
    let storage = match LlmAuditStorage::new_readonly(db_path) {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open audit db: {e}"),
            );
        }
    };

    match storage.list_audit_records(&filter) {
        Ok((records, total)) => Json(json!({ "records": records, "total": total })).into_response(),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query db: {e}"),
        ),
    }
}

/// GET /api/llm-audit/:id
pub(crate) async fn handle_llm_audit_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let db_path = &state.core.config.storage.llm_audit_db_path;
    let storage = match LlmAuditStorage::new_readonly(db_path) {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open audit db: {e}"),
            );
        }
    };

    match storage.get_audit_record(&id) {
        Ok(Some(record)) => Json(record).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Audit record not found"),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query db: {e}"),
        ),
    }
}
