use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use hone_core::cloud_runtime::CloudPgRuntime;
use hone_memory::{AuditQueryFilter, LlmAuditStorage};

use crate::routes::json_error;
use crate::state::AppState;

/// GET /api/llm-audit
pub(crate) async fn handle_llm_audit_list(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<AuditQueryFilter>,
) -> impl IntoResponse {
    let storage_result = match CloudPgRuntime::from_cloud_config(&state.core.config.cloud) {
        Some(postgres) => {
            LlmAuditStorage::new_cloud(postgres, state.core.config.storage.llm_audit_retention_days)
                .await
                .map_err(|error| error.to_string())
        }
        None => Err("PostgreSQL is not configured".to_string()),
    };
    let storage = match storage_result {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open audit db: {e}"),
            );
        }
    };

    match storage.list_audit_records(&filter).await {
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
    let storage_result = match CloudPgRuntime::from_cloud_config(&state.core.config.cloud) {
        Some(postgres) => {
            LlmAuditStorage::new_cloud(postgres, state.core.config.storage.llm_audit_retention_days)
                .await
                .map_err(|error| error.to_string())
        }
        None => Err("PostgreSQL is not configured".to_string()),
    };
    let storage = match storage_result {
        Ok(s) => s,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open audit db: {e}"),
            );
        }
    };

    match storage.get_audit_record(&id).await {
        Ok(Some(record)) => Json(record).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Audit record not found"),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query db: {e}"),
        ),
    }
}
