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

/// Markdown of the assembled report. It exists before the PDF and survives a
/// failed render, so a finished analysis is never withheld by layout.
pub(crate) async fn markdown(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let (_, task) = match owned_task(&state, &headers, &id).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let Some(report) = task.checkpoint.outputs.get(workflow::MARKDOWN_KEY) else {
        return super::json_error(StatusCode::CONFLICT, "报告正文尚未生成完成");
    };
    (
        [
            (
                header::CONTENT_TYPE,
                "text/markdown; charset=utf-8".to_owned(),
            ),
            (header::CACHE_CONTROL, "private, no-store".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"company-analysis-{id}.md\""),
            ),
        ],
        report.clone(),
    )
        .into_response()
}

/// One automatic continuation from checkpoints after a non-lease failure.
const AUTOMATIC_RESUMES: usize = 1;
const AUTOMATIC_RESUME_DELAY: std::time::Duration = std::time::Duration::from_secs(30);
/// Consecutive heartbeat errors tolerated; 3 x 20 s stays inside the 2 min lease.
const HEARTBEAT_FAILURES_TOLERATED: usize = 3;

fn failure_info(checkpoint: &store::Checkpoint) -> &'static str {
    if checkpoint.outputs.contains_key(workflow::MARKDOWN_KEY) {
        "报告正文已完成，PDF 生成未完成；可先下载 Markdown，或继续重新生成 PDF"
    } else {
        "任务中断，可继续已保存的分析"
    }
}

fn spawn(state: Arc<AppState>, store: Store, task: Task) {
    tokio::spawn(async move {
        // Include these jobs in the existing production deploy/drain count.
        let _drain = state
            .active_chat_runs
            .try_begin(format!("company-analysis:{}", task.task_id));
        let heartbeat = async {
            let mut failures = 0;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                match store.heartbeat(&task).await {
                    Ok(()) => failures = 0,
                    Err(e) if e == store::LEASE_LOST => return Err(e),
                    Err(e) => {
                        failures += 1;
                        tracing::warn!(task_id=%task.task_id,failures,error=%e,"company analysis heartbeat failed");
                        if failures > HEARTBEAT_FAILURES_TOLERATED {
                            return Err(e);
                        }
                    }
                }
            }
        };
        let attempts = async {
            let mut current = task.clone();
            let mut resumes = 0;
            loop {
                match workflow::execute(&state, &store, &current).await {
                    Ok(()) => return Ok(()),
                    Err(e) if resumes < AUTOMATIC_RESUMES && !e.contains(store::LEASE_LOST) => {
                        resumes += 1;
                        tracing::warn!(task_id=%task.task_id,error=%e,"company analysis failed; continuing from checkpoints");
                        tokio::time::sleep(AUTOMATIC_RESUME_DELAY).await;
                        // Continue from the persisted checkpoint, never from the
                        // stale snapshot this attempt started with.
                        match store.get(&task.actor_user_id, &task.task_id).await {
                            Ok(Some(latest)) if latest.lease_token == task.lease_token => {
                                current = latest
                            }
                            Ok(_) => return Err(store::LEASE_LOST.to_owned()),
                            Err(read) => return Err(format!("{e}; checkpoint reload: {read}")),
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        };
        let result = tokio::select! {
            r=tokio::time::timeout(std::time::Duration::from_secs(3600),attempts)=>r.unwrap_or_else(|_|Err("company analysis overall timeout".to_owned())),
            r=heartbeat=>r,
        };
        if let Err(e) = result {
            tracing::error!(task_id=%task.task_id,error=%e,"company analysis failed; checkpoints retained");
            for delay in [0, 2, 5] {
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                match store.get(&task.actor_user_id, &task.task_id).await {
                    Ok(Some(latest)) => {
                        let info = failure_info(&latest.checkpoint);
                        match store
                            .update(&task, latest.progress, info, "failed", &latest.checkpoint)
                            .await
                        {
                            // Success, or another attempt now owns the task.
                            Ok(()) => break,
                            Err(write) if write == store::LEASE_LOST => break,
                            Err(write) => {
                                tracing::warn!(task_id=%task.task_id,error=%write,"company analysis failure state write retry")
                            }
                        }
                    }
                    Ok(None) => break,
                    Err(read) => {
                        tracing::warn!(task_id=%task.task_id,error=%read,"company analysis failure state read retry")
                    }
                }
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

    #[test]
    fn failure_message_points_to_the_markdown_once_the_report_exists() {
        let mut checkpoint = store::Checkpoint::default();
        assert_eq!(failure_info(&checkpoint), "任务中断，可继续已保存的分析");
        checkpoint
            .outputs
            .insert(workflow::MARKDOWN_KEY.into(), "# 报告".into());
        assert!(failure_info(&checkpoint).contains("下载 Markdown"));
    }
}
