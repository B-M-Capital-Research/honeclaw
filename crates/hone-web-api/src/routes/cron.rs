use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use hone_core::ActorIdentity;
use hone_memory::CronJobStorage;
use hone_memory::cron_job::{CronJob, CronJobUpdate, CronSchedule, is_cron_enabled_limit_error};

use crate::routes::{
    json_error, normalize_optional_string, normalized_actor, normalized_query_actor, require_actor,
    require_string,
};
use crate::state::AppState;
use crate::types::{CronJobDetailRecord, CronJobRecord, CronJobUpsertRequest, UserIdQuery};

/// GET /api/cron-jobs — 列出某个用户或全局的定时任务
pub(crate) async fn handle_cron_jobs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> Response {
    let actor = match normalized_query_actor(&params) {
        Ok(actor) => actor,
        Err(error) => return error,
    };

    let storage = cron_job_storage(&state);
    let records: Vec<CronJobRecord> = if let Some(actor) = actor {
        storage
            .list_jobs(&actor)
            .into_iter()
            .map(|job| serialize_cron_job(actor.clone(), job))
            .collect()
    } else {
        storage
            .list_all_jobs()
            .into_iter()
            .map(|(actor, job)| serialize_cron_job(actor, job))
            .collect()
    };

    Json(serde_json::json!({ "jobs": records })).into_response()
}

/// GET /api/cron-jobs/{id} — 查看单个定时任务
pub(crate) async fn handle_cron_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let storage = cron_job_storage(&state);
    let actor = match normalized_query_actor(&params) {
        Ok(actor) => actor,
        Err(error) => return error,
    };

    match storage.get_job(&job_id, actor.as_ref()) {
        Some((actor, job)) => Json(json!({
            "job": CronJobDetailRecord {
                job: serialize_cron_job(actor, job.clone()),
                executions: storage
                    .list_execution_records(&job.id, 50)
                    .unwrap_or_default(),
            }
        }))
        .into_response(),
        None => json_error(StatusCode::NOT_FOUND, format!("未找到任务 {job_id}")),
    }
}

/// POST /api/cron-jobs — 创建定时任务
pub(crate) async fn handle_create_cron_job(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CronJobUpsertRequest>,
) -> axum::response::Response {
    let actor = match require_actor(
        req.channel.clone(),
        req.user_id.clone(),
        req.channel_scope.clone(),
    ) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let name = match require_string(req.name, "name") {
        Ok(name) => name,
        Err(error) => return error,
    };
    let task_prompt = match require_string(req.task_prompt, "task_prompt") {
        Ok(task_prompt) => task_prompt,
        Err(error) => return error,
    };
    let repeat = req.repeat.unwrap_or_else(|| "daily".to_string());
    let channel_target = req.channel_target.unwrap_or_else(|| actor.user_id.clone());
    let enabled = req.enabled.unwrap_or(true);
    let admin_bypass = state.core.is_admin_actor(&actor);

    let storage = cron_job_storage(&state);
    let result = storage.add_job(
        &actor,
        &name,
        req.hour,
        req.minute,
        &repeat,
        &task_prompt,
        &channel_target,
        req.weekday,
        req.push,
        enabled,
        req.tags,
        admin_bypass,
    );
    if result["success"] != true {
        let message = result["error"]
            .as_str()
            .unwrap_or("创建定时任务失败")
            .to_string();
        let status = if is_cron_enabled_limit_error(&message) {
            StatusCode::TOO_MANY_REQUESTS
        } else {
            StatusCode::BAD_REQUEST
        };
        return json_error(status, message);
    }

    let final_job = match serde_json::from_value::<CronJob>(result["job"].clone()) {
        Ok(job) => job,
        Err(error) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("任务创建成功但响应解析失败: {error}"),
            );
        }
    };

    (
        StatusCode::CREATED,
        Json(json!({
            "job": serialize_cron_job(actor, final_job)
        })),
    )
        .into_response()
}

/// PUT /api/cron-jobs/{id} — 更新定时任务
pub(crate) async fn handle_update_cron_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    Query(params): Query<UserIdQuery>,
    Json(req): Json<CronJobUpsertRequest>,
) -> axum::response::Response {
    let lookup_actor = match normalized_actor(
        normalize_optional_string(req.channel.clone()),
        normalize_optional_string(req.user_id.clone()),
        normalize_optional_string(req.channel_scope.clone()),
    ) {
        Ok(Some(actor)) => Some(actor),
        Ok(None) => match normalized_query_actor(&params) {
            Ok(actor) => actor,
            Err(error) => return error,
        },
        Err(error) => return error,
    };
    let storage = cron_job_storage(&state);
    let Some((resolved_actor, existing)) = storage.get_job(&job_id, lookup_actor.as_ref()) else {
        return json_error(StatusCode::NOT_FOUND, format!("未找到任务 {job_id}"));
    };

    if matches!(req.name.as_deref(), Some(name) if name.trim().is_empty()) {
        return json_error(StatusCode::BAD_REQUEST, "name 不能为空");
    }
    if matches!(req.task_prompt.as_deref(), Some(prompt) if prompt.trim().is_empty()) {
        return json_error(StatusCode::BAD_REQUEST, "task_prompt 不能为空");
    }

    let schedule = if req.hour.is_some()
        || req.minute.is_some()
        || req.repeat.is_some()
        || req.weekday.is_some()
    {
        let repeat = req
            .repeat
            .clone()
            .unwrap_or_else(|| existing.schedule.repeat.clone());
        Some(CronSchedule {
            hour: req.hour.unwrap_or(existing.schedule.hour),
            minute: req.minute.unwrap_or(existing.schedule.minute),
            weekday: if repeat == "weekly" {
                req.weekday.or(existing.schedule.weekday)
            } else {
                None
            },
            repeat,
        })
    } else {
        None
    };

    let updates = CronJobUpdate {
        name: normalize_optional_string(req.name),
        schedule,
        task_prompt: normalize_optional_string(req.task_prompt),
        push: req.push,
        enabled: req.enabled,
        channel_target: normalize_optional_string(req.channel_target),
        tags: req.tags,
    };

    let admin_bypass = state.core.is_admin_actor(&resolved_actor);
    match storage.update_job(&job_id, Some(&resolved_actor), updates, admin_bypass) {
        Ok(Some((actor, job))) => Json(json!({
            "job": serialize_cron_job(actor, job)
        }))
        .into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, format!("未找到任务 {job_id}")),
        Err(error) => {
            let message = error.to_string();
            let status = if is_cron_enabled_limit_error(&message) {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::BAD_REQUEST
            };
            json_error(status, message)
        }
    }
}

/// POST /api/cron-jobs/{id}/toggle — 切换任务启用状态
pub(crate) async fn handle_toggle_cron_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> axum::response::Response {
    let storage = cron_job_storage(&state);
    let actor = match normalized_query_actor(&params) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let Some((resolved_actor, _)) = storage.get_job(&job_id, actor.as_ref()) else {
        return json_error(StatusCode::NOT_FOUND, format!("未找到任务 {job_id}"));
    };
    let admin_bypass = state.core.is_admin_actor(&resolved_actor);
    match storage.toggle_job(&job_id, Some(&resolved_actor), admin_bypass) {
        Ok(Some((actor, job))) => Json(json!({
            "job": serialize_cron_job(actor, job)
        }))
        .into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, format!("未找到任务 {job_id}")),
        Err(error) => {
            let message = error.to_string();
            let status = if is_cron_enabled_limit_error(&message) {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            json_error(status, message)
        }
    }
}

/// DELETE /api/cron-jobs/{id} — 删除定时任务
pub(crate) async fn handle_delete_cron_job(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> axum::response::Response {
    let storage = cron_job_storage(&state);
    let actor = match normalized_query_actor(&params) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    match storage.delete_job(&job_id, actor.as_ref()) {
        Ok(Some((actor, job))) => Json(json!({
            "removed_job": serialize_cron_job(actor, job)
        }))
        .into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, format!("未找到任务 {job_id}")),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

fn cron_job_storage(state: &AppState) -> CronJobStorage {
    state.core.cron_job_storage()
}

fn serialize_cron_job(actor: ActorIdentity, job: CronJob) -> CronJobRecord {
    CronJobRecord {
        channel: actor.channel,
        user_id: actor.user_id,
        channel_scope: actor.channel_scope,
        job,
    }
}
