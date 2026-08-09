//! The signed-in user's own scheduled pushes.
//!
//! Managing a schedule already existed, but only behind the admin API. A user
//! who wanted a push to stop, or to fire an hour later, had no way to say so
//! and no way to see what they had subscribed to — which is why people did not
//! know how to turn these off. Everything here is scoped to the caller's own
//! actor: `user_id` is never read from the request.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use hone_core::ActorIdentity;
use hone_memory::cron_job::{CronJob, CronJobUpdate, CronSchedule};
use serde::Deserialize;
use serde_json::json;

use crate::routes::public::require_public_user;
use crate::state::AppState;

/// A schedule someone has to be able to read back and recognise. Anything not
/// here is deliberately withheld: channel targets and routing internals are
/// not the user's mental model of "a push I subscribed to".
fn serialize_subscription(job: &CronJob) -> serde_json::Value {
    json!({
        "job_id": job.id,
        "name": job.name,
        "task_prompt": job.task_prompt,
        "enabled": job.enabled,
        "channel": job.channel,
        "schedule": {
            "hour": job.schedule.hour,
            "minute": job.schedule.minute,
            "repeat": job.schedule.repeat,
            "weekday": job.schedule.weekday,
            "date": job.schedule.date,
        },
        "created_at": job.created_at,
        "last_run_at": job.last_run_at,
    })
}

fn public_web_actor(state: &AppState, headers: &HeaderMap) -> Result<ActorIdentity, Response> {
    let user = require_public_user(state, headers)?;
    ActorIdentity::new("web", user.user_id, None::<String>)
        .map_err(|error| crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()))
}

pub(crate) async fn handle_list_subscriptions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let actor = match public_web_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let mut jobs = state.core.cron_job_storage().list_jobs(&actor);
    // Active ones first: someone opening this page is usually looking for
    // something still firing, not something they already stopped.
    jobs.sort_by(|left, right| {
        right
            .enabled
            .cmp(&left.enabled)
            .then_with(|| left.schedule.hour.cmp(&right.schedule.hour))
            .then_with(|| left.schedule.minute.cmp(&right.schedule.minute))
    });
    Json(json!({
        "subscriptions": jobs.iter().map(serialize_subscription).collect::<Vec<_>>(),
    }))
    .into_response()
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionUpdateRequest {
    name: Option<String>,
    task_prompt: Option<String>,
    hour: Option<u32>,
    minute: Option<u32>,
    repeat: Option<String>,
    weekday: Option<u32>,
    date: Option<String>,
    enabled: Option<bool>,
}

pub(crate) async fn handle_update_subscription(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(request): Json<SubscriptionUpdateRequest>,
) -> Response {
    let actor = match public_web_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let storage = state.core.cron_job_storage();
    // Scoping the lookup to the caller is what stops one user editing
    // another's schedule by guessing a job id.
    let Some((_, existing)) = storage.get_job(&job_id, Some(&actor)) else {
        return crate::routes::json_error(StatusCode::NOT_FOUND, "未找到该订阅".to_string());
    };

    if let Some(prompt) = request.task_prompt.as_ref()
        && prompt.trim().is_empty()
    {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "推送内容不能为空".to_string());
    }

    // A schedule is edited as a whole: sending only `minute` and inheriting a
    // stale `hour` from a concurrent edit would move the push somewhere the
    // user never asked for.
    let schedule_touched = request.hour.is_some()
        || request.minute.is_some()
        || request.repeat.is_some()
        || request.weekday.is_some()
        || request.date.is_some();
    let schedule = schedule_touched.then(|| CronSchedule {
        hour: request.hour.unwrap_or(existing.schedule.hour),
        minute: request.minute.unwrap_or(existing.schedule.minute),
        repeat: request
            .repeat
            .clone()
            .unwrap_or_else(|| existing.schedule.repeat.clone()),
        weekday: request.weekday.or(existing.schedule.weekday),
        date: request
            .date
            .clone()
            .or_else(|| existing.schedule.date.clone()),
    });

    let updates = CronJobUpdate {
        name: request.name.clone().filter(|name| !name.trim().is_empty()),
        task_prompt: request.task_prompt.clone(),
        schedule,
        enabled: request.enabled,
        ..Default::default()
    };

    match storage.update_job(&job_id, Some(&actor), updates, false) {
        Ok(Some((_, job))) => {
            Json(json!({ "subscription": serialize_subscription(&job) })).into_response()
        }
        Ok(None) => crate::routes::json_error(StatusCode::NOT_FOUND, "未找到该订阅".to_string()),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error.to_string()),
    }
}

/// Unsubscribing disables rather than deletes, matching the link in delivered
/// pushes: the history stays readable and the person can turn it back on.
pub(crate) async fn handle_unsubscribe_subscription(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> Response {
    let actor = match public_web_actor(&state, &headers) {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let storage = state.core.cron_job_storage();
    let Some((_, existing)) = storage.get_job(&job_id, Some(&actor)) else {
        return crate::routes::json_error(StatusCode::NOT_FOUND, "未找到该订阅".to_string());
    };
    if !existing.enabled {
        // Idempotent: clicking twice is a normal thing to do.
        return Json(json!({
            "subscription": serialize_subscription(&existing),
            "already_unsubscribed": true,
        }))
        .into_response();
    }
    let updates = CronJobUpdate {
        enabled: Some(false),
        ..Default::default()
    };
    match storage.update_job(&job_id, Some(&actor), updates, true) {
        Ok(Some((_, job))) => Json(json!({
            "subscription": serialize_subscription(&job),
            "already_unsubscribed": false,
        }))
        .into_response(),
        Ok(None) => crate::routes::json_error(StatusCode::NOT_FOUND, "未找到该订阅".to_string()),
        Err(error) => {
            crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job() -> CronJob {
        CronJob {
            id: "job-1".to_string(),
            name: "盘前复盘".to_string(),
            schedule: CronSchedule {
                hour: 6,
                minute: 55,
                repeat: "daily".to_string(),
                weekday: None,
                date: None,
            },
            task_prompt: "帮我做盘前复盘".to_string(),
            push: serde_json::Value::Null,
            enabled: true,
            channel: "web".to_string(),
            channel_scope: None,
            channel_target: "internal-routing-detail".to_string(),
            tags: vec![],
            created_at: None,
            last_run_at: None,
            bypass_quiet_hours: false,
        }
    }

    #[test]
    fn a_subscription_reads_back_as_the_user_wrote_it() {
        let value = serialize_subscription(&job());
        assert_eq!(value["name"], "盘前复盘");
        assert_eq!(value["task_prompt"], "帮我做盘前复盘");
        assert_eq!(value["schedule"]["hour"], 6);
        assert_eq!(value["schedule"]["minute"], 55);
        assert_eq!(value["enabled"], true);
    }

    #[test]
    fn routing_internals_stay_out_of_the_user_facing_shape() {
        // channel_target is a delivery detail, not part of anyone's idea of
        // "a push I subscribed to", and it can identify a chat.
        let value = serialize_subscription(&job());
        assert!(value.get("channel_target").is_none());
        assert!(!value.to_string().contains("internal-routing-detail"));
    }
}
