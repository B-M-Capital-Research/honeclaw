//! Administrator-only company analysis, preserving the original full-US DAG.
mod store;
mod workflow;

use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use hone_core::{
    ActorIdentity,
    cloud_runtime::{CloudPgRuntime, OssObjectStore},
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use store::{Store, Task};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartRequest {
    company: String,
}

fn valid_company(input: &str) -> Result<&str, &'static str> {
    let value = input.trim();
    if value.is_empty() || value.chars().count() > 200 || value.chars().any(char::is_control) {
        Err("请输入公司名称或股票代码（不超过 200 字）")
    } else {
        Ok(value)
    }
}

async fn authorized(state: &AppState, headers: &HeaderMap) -> Result<(String, Store), Response> {
    let user = super::public::require_public_user(state, headers).await?;
    match state.web_auth.is_web_admin(&user.user_id).await {
        Ok(true) => {}
        Ok(false) => {
            return Err(super::json_error(
                StatusCode::FORBIDDEN,
                "该功能目前仅对管理员开放",
            ));
        }
        Err(e) => {
            tracing::error!(error=%e,"company analysis authorization failed");
            return Err(unavailable());
        }
    }
    let Some(pg) = CloudPgRuntime::from_cloud_config(&state.core.config.cloud) else {
        return Err(unavailable());
    };
    let store = Store(pg);
    if let Err(e) = store.initialize().await {
        tracing::error!(error=%e,"company analysis storage unavailable");
        return Err(unavailable());
    }
    Ok((user.user_id, store))
}

fn unavailable() -> Response {
    super::json_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "公司分析暂时不可用，请稍后重试",
    )
}

fn response(value: serde_json::Value) -> Response {
    ([(header::CACHE_CONTROL, "private, no-store")], Json(value)).into_response()
}

pub(crate) async fn start(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<StartRequest>,
) -> Response {
    let (actor, store) = match authorized(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let company = match valid_company(&request.company) {
        Ok(v) => v,
        Err(e) => return super::json_error(StatusCode::BAD_REQUEST, e),
    };
    if let Err(e) = workflow::validate_environment(&state) {
        tracing::error!(error=%e,"company analysis configuration unavailable");
        return unavailable();
    }
    match store
        .create(
            &actor,
            company,
            &state.core.config.agent.company_analysis_model,
        )
        .await
    {
        Ok(task) => {
            let result = response(json!(task));
            spawn(state, store, task);
            result
        }
        Err(e) if e.starts_with("已有公司分析任务") => {
            super::json_error(StatusCode::CONFLICT, e)
        }
        Err(e) => {
            tracing::error!(error=%e,"company analysis create failed");
            unavailable()
        }
    }
}

pub(crate) async fn list(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let (actor, store) = match authorized(&state, &headers).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    match store.list(&actor).await {
        Ok(tasks) => response(json!({"tasks":tasks})),
        Err(e) => {
            tracing::error!(error=%e,"company analysis list failed");
            unavailable()
        }
    }
}

async fn owned_task(
    state: &AppState,
    headers: &HeaderMap,
    id: &str,
) -> Result<(Store, Task), Response> {
    let (actor, store) = authorized(state, headers).await?;
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(super::json_error(StatusCode::NOT_FOUND, "任务不存在"));
    }
    match store.get(&actor, id).await {
        Ok(Some(task)) => Ok((store, task)),
        Ok(None) => Err(super::json_error(StatusCode::NOT_FOUND, "任务不存在")),
        Err(e) => {
            tracing::error!(error=%e,"company analysis read failed");
            Err(unavailable())
        }
    }
}

pub(crate) async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    match owned_task(&state, &headers, &id).await {
        Ok((_, task)) => response(json!(task)),
        Err(e) => e,
    }
}

pub(crate) async fn resume(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (store, task) = match owned_task(&state, &headers, &id).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if task.status == "completed" || task.status == "running" {
        return response(json!(task));
    }
    if let Err(e) = workflow::validate_environment(&state) {
        tracing::error!(error=%e,"company analysis resume unavailable");
        return unavailable();
    }
    match store.resume(&task.actor_user_id, &id).await {
        Ok(Some(task)) => {
            let result = response(json!(task));
            spawn(state, store, task);
            result
        }
        Ok(None) => super::json_error(StatusCode::CONFLICT, "任务状态已变化，请刷新查看"),
        Err(e) => {
            tracing::error!(error=%e,"company analysis resume failed");
            super::json_error(StatusCode::CONFLICT, "暂时无法继续，请检查是否已有任务运行")
        }
    }
}

pub(crate) async fn download(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (_, task) = match owned_task(&state, &headers, &id).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    if !task.pdf_ready {
        return super::json_error(StatusCode::CONFLICT, "PDF 尚未生成完成");
    }
    let Some(oss) = OssObjectStore::from_config(&state.core.config.cloud.oss) else {
        return unavailable();
    };
    let actor = match ActorIdentity::new("web", &task.actor_user_id, Option::<String>::None) {
        Ok(v) => v,
        Err(_) => return unavailable(),
    };
    let key = oss.actor_document_key(
        &actor,
        "company-analysis",
        &format!("{}-{}.pdf", task.task_id, task.lease_token),
    );
    if task.checkpoint.pdf_key.as_deref() != Some(key.as_str()) {
        return unavailable();
    }
    let object = match oss.get_object_limited(&key, 20 * 1024 * 1024).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error=%e,"company analysis PDF download failed");
            return unavailable();
        }
    };
    if task.checkpoint.pdf_sha256.as_deref()
        != Some(format!("{:x}", Sha256::digest(&object.bytes)).as_str())
    {
        tracing::error!(task_id=%id,"company analysis PDF checksum mismatch");
        return unavailable();
    }
    (
        [
            (header::CONTENT_TYPE, "application/pdf".to_owned()),
            (header::CACHE_CONTROL, "private, no-store".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"company-analysis-{id}.pdf\""),
            ),
        ],
        object.bytes,
    )
        .into_response()
}

fn spawn(state: Arc<AppState>, store: Store, task: Task) {
    tokio::spawn(async move {
        // Include these jobs in the existing production deploy/drain count.
        let _drain = state
            .active_chat_runs
            .try_begin(format!("company-analysis:{}", task.task_id));
        let heartbeat = async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                store.heartbeat(&task).await?;
            }
        };
        let result = tokio::select! {
            r=tokio::time::timeout(std::time::Duration::from_secs(3600),workflow::execute(&state,&store,&task))=>r.unwrap_or_else(|_|Err("company analysis overall timeout".to_owned())),
            r=heartbeat=>r,
        };
        if let Err(e) = result {
            tracing::error!(task_id=%task.task_id,error=%e,"company analysis failed; checkpoints retained");
            if let Ok(Some(latest)) = store.get(&task.actor_user_id, &task.task_id).await {
                let _ = store
                    .update(
                        &task,
                        latest.progress,
                        "任务中断，可继续已保存的分析",
                        "failed",
                        &latest.checkpoint,
                    )
                    .await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn company_input_is_bounded_and_cannot_inject_protocol_lines() {
        assert_eq!(valid_company(" TEM ").unwrap(), "TEM");
        assert!(valid_company("").is_err());
        assert!(valid_company("TEM\nmode:other").is_err());
        assert!(valid_company(&"美".repeat(201)).is_err());
        assert!(
            serde_json::from_value::<StartRequest>(json!({"company":"TEM","model":"evil"}))
                .is_err()
        );
    }
}
