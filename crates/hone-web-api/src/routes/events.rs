use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use serde_json::json;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{error, info, warn};

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_memory::session_message_text;
use hone_scheduler::{SchedulerEvent, execution_detail_with_delivery_key};

use crate::routes::normalized_query_actor;
use crate::routes::public_pushes::{
    StoredWebPush, build_web_push_summary, store_web_scheduler_push,
};
use crate::state::{AppState, PushEvent};
use crate::types::UserIdQuery;

const SCHEDULER_EXECUTION_GRACE_SECS: u64 = 30;

/// GET /api/events?user_id=... — 长连接 SSE 推送通道（调度器消息用）
pub(crate) async fn handle_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let actor = normalized_query_actor(&params).ok().flatten();

    let rx = state.push_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |msg| {
        let actor = actor.clone();
        match msg {
            Ok(ev)
                if actor.as_ref().is_some_and(|actor| {
                    ev.channel == actor.channel
                        && ev.user_id == actor.user_id
                        && ev.channel_scope == actor.channel_scope
                }) =>
            {
                let data = serde_json::to_string(&ev.data).unwrap_or_default();
                Some(Ok(Event::default().event(ev.event).data(data)))
            }
            Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
                warn!("SSE /api/events: 客户端消费过慢，跳过了 {n} 条事件");
                let data = serde_json::json!({ "skipped": n }).to_string();
                Some(Ok(Event::default().event("events_lagged").data(data)))
            }
            _ => None,
        }
    });

    // 首条 connected 确认事件
    let init = tokio_stream::iter(vec![Ok::<_, Infallible>(
        Event::default().event("connected").data("{}"),
    )]);

    Sse::new(init.chain(stream)).keep_alive(KeepAlive::default())
}

fn web_scheduler_delivery_status(_console_event_sent: bool) -> (String, bool) {
    // Web scheduled results are already persisted to the conversation by this point.
    // The SSE event only controls whether an online console sees the update in real time.
    ("sent".to_string(), true)
}

fn web_scheduler_delivery_detail(
    scheduler_metadata: serde_json::Value,
    console_event_sent: bool,
    channel: &str,
) -> serde_json::Value {
    json!({
        "scheduler": scheduler_metadata,
        "console_event_sent": console_event_sent,
        "system_push_supported": false,
        "system_push_sent": false,
        "delivery_channel": channel,
    })
}

fn build_web_scheduler_push_event(
    event: &SchedulerEvent,
    stored: &StoredWebPush,
) -> Option<PushEvent> {
    if event.channel != "web" {
        return None;
    }

    Some(PushEvent {
        channel: event.actor.channel.clone(),
        user_id: event.actor.user_id.clone(),
        channel_scope: event.actor.channel_scope.clone(),
        event: "scheduled_message".into(),
        data: json!({
            "push_id": stored.message.push_id,
            "job_name": event.job_name.clone(),
            "job_id": event.job_id.clone(),
            "summary": stored.message.summary,
            "created_at": stored.message.created_at,
            "unread_count": stored.unread_count,
        }),
    })
}

fn emit_web_scheduler_push(
    push_tx: &tokio::sync::broadcast::Sender<PushEvent>,
    event: &SchedulerEvent,
    stored: &StoredWebPush,
) -> bool {
    build_web_scheduler_push_event(event, stored)
        .is_some_and(|push_event| push_tx.send(push_event).is_ok())
}

fn persist_and_emit_web_scheduler_push(
    state: &AppState,
    event: &SchedulerEvent,
    response: &str,
) -> bool {
    if event.channel != "web" || response.trim().is_empty() {
        return false;
    }
    match store_web_scheduler_push(state, event, response) {
        Ok(stored) => emit_web_scheduler_push(&state.push_tx, event, &stored),
        Err(error) => {
            warn!(
                "⏰ [Web] 定时任务推送卡片落库失败: user={} job={} err={}",
                event.actor.user_id, event.job_name, error
            );
            state
                .push_tx
                .send(PushEvent {
                    channel: event.actor.channel.clone(),
                    user_id: event.actor.user_id.clone(),
                    channel_scope: event.actor.channel_scope.clone(),
                    event: "scheduled_message".into(),
                    data: json!({
                        "push_id": event.delivery_key.clone(),
                        "job_name": event.job_name.clone(),
                        "job_id": event.job_id.clone(),
                        "summary": build_web_push_summary(&event.job_name, response),
                        "created_at": hone_core::beijing_now_rfc3339(),
                        "content": response.trim(),
                    }),
                })
                .is_ok()
        }
    }
}

fn scheduler_execution_timeout_for(overall_timeout: Duration) -> Duration {
    overall_timeout.saturating_add(Duration::from_secs(SCHEDULER_EXECUTION_GRACE_SECS))
}

fn scheduler_handler_timeout_execution(
    event: &SchedulerEvent,
    timeout: Duration,
) -> scheduler::ScheduledTaskExecution {
    scheduler::ScheduledTaskExecution {
        should_deliver: false,
        content: String::new(),
        error: Some(format!(
            "web_scheduler_handler_timeout:{}s",
            timeout.as_secs()
        )),
        metadata: json!({
            "failure_kind": "web_scheduler_handler_timeout",
            "timeout_secs": timeout.as_secs(),
        }),
        session_id: Some(event.actor.session_id()),
    }
}

/// 接收调度器事件，为每个触发的任务启动独立处理协程
pub(crate) async fn handle_scheduler_events(
    state: Arc<AppState>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: imessage）");
    while let Some(event) = event_rx.recv().await {
        if event.channel == "imessage" && !state.core.config.imessage.enabled {
            warn!(
                "⏰ 已禁用 iMessage 渠道，跳过调度任务: user={} job={}",
                event.actor.user_id, event.job_name
            );
            continue;
        }
        // 仅处理本渠道（imessage / web）
        if !matches!(event.channel.as_str(), "imessage" | "web") {
            warn!(
                "⏰ 跳过不属于本渠道的调度任务: user={} channel={}",
                event.actor.user_id, event.channel
            );
            continue;
        }

        info!(
            "⏰ 触发定时任务: user={} job={} prompt={}",
            event.actor.user_id, event.job_name, event.task_prompt
        );

        let state_clone = state.clone();
        tokio::spawn(async move {
            let storage = state_clone.core.cron_job_storage();
            let _ = storage.record_execution_event(
                &event.actor,
                &event.job_id,
                &event.job_name,
                &event.channel_target,
                event.heartbeat,
                CronJobExecutionInput {
                    execution_status: "running".to_string(),
                    message_send_status: "pending".to_string(),
                    should_deliver: true,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: json!({
                        "delivery_key": event.delivery_key,
                        "phase": "started",
                    }),
                },
            );
            let result = run_scheduled_task(&state_clone, &event).await;
            if !result.should_deliver {
                let failure_trace = scheduler_failure_trace_required(&result);
                let response = if failure_trace {
                    Some(format!(
                        "定时任务「{}」执行出错，请稍后重试。",
                        event.job_name
                    ))
                } else {
                    None
                };
                if let Some(response) = response.as_deref() {
                    persist_web_scheduler_failure(&state_clone, &event, &result, response);
                }
                let console_event_sent = response
                    .as_deref()
                    .map(|response| {
                        persist_and_emit_web_scheduler_push(&state_clone, &event, response)
                    })
                    .unwrap_or(false);
                let _ = storage.record_execution_event(
                    &event.actor,
                    &event.job_id,
                    &event.job_name,
                    &event.channel_target,
                    event.heartbeat,
                    CronJobExecutionInput {
                        execution_status: if failure_trace {
                            "execution_failed".to_string()
                        } else {
                            "noop".to_string()
                        },
                        message_send_status: if failure_trace {
                            "skipped_error".to_string()
                        } else {
                            "skipped_noop".to_string()
                        },
                        should_deliver: false,
                        delivered: false,
                        response_preview: response,
                        error_message: result.error.clone().or_else(|| {
                            failure_trace
                                .then(|| "内部错误已抑制，已写入用户可见失败提示".to_string())
                        }),
                        detail: execution_detail_with_delivery_key(
                            if failure_trace && event.channel == "web" {
                                web_scheduler_delivery_detail(
                                    result.metadata.clone(),
                                    console_event_sent,
                                    &event.channel,
                                )
                            } else {
                                result.metadata.clone()
                            },
                            &event.delivery_key,
                        ),
                    },
                );
                return;
            }
            let response = if result.error.is_some() {
                format!("定时任务「{}」执行出错，请稍后重试。", event.job_name)
            } else {
                result.content.clone()
            };
            if result.error.is_some() {
                persist_web_scheduler_failure(&state_clone, &event, &result, &response);
            }

            // 1. 推送到 Web 控制台 SSE（供控制台页面实时展示）
            let console_event_sent =
                persist_and_emit_web_scheduler_push(&state_clone, &event, &response);

            // 2. 若是 iMessage 渠道，把结果通过 hone-imessage 内置 HTTP 服务投递给用户
            let (mut message_send_status, mut delivered) =
                web_scheduler_delivery_status(console_event_sent);
            let mut error_message = result.error.clone();
            let mut detail = web_scheduler_delivery_detail(
                result.metadata.clone(),
                console_event_sent,
                &event.channel,
            );
            if event.channel == "imessage" {
                let url = format!(
                    "http://{}/api/send",
                    state_clone.core.config.imessage.listen_addr
                );
                let handle = event.channel_target.clone();
                let job_name = event.job_name.clone();
                let text = response.clone();
                let payload = serde_json::json!({
                    "handle": handle,
                    "text": text,
                    "job_name": job_name,
                });

                // 复用 AppState 中的 http_client，最多重试一次
                delivered = false;
                for attempt in 1u8..=2 {
                    match state_clone
                        .http_client
                        .post(&url)
                        .json(&payload)
                        .timeout(std::time::Duration::from_secs(10))
                        .send()
                        .await
                    {
                        Ok(resp) if resp.status().is_success() => {
                            info!(
                                "⏰ [iMessage] 定时任务结果已投递: handle={} job={} attempt={attempt}",
                                handle, job_name
                            );
                            delivered = true;
                            break;
                        }
                        Ok(resp) => {
                            warn!(
                                "⏰ [iMessage] 定时任务投递失败: handle={} job={} status={} attempt={attempt}",
                                handle,
                                job_name,
                                resp.status()
                            );
                        }
                        Err(e) => {
                            warn!(
                                "⏰ [iMessage] 定时任务投递请求错误: handle={} job={} err={e} attempt={attempt}\n  \
                                 → 请确认 hone-imessage 进程正在运行且 imessage.listen_addr 配置正确",
                                handle, job_name
                            );
                        }
                    }
                    if attempt < 2 {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
                if !delivered {
                    error!(
                        "⏰ [iMessage] 定时任务 2 次尝试均失败，消息未送达: handle={} job={}",
                        handle, job_name
                    );
                    message_send_status = "send_failed".to_string();
                    if error_message.is_none() {
                        error_message = Some("iMessage 定时任务消息未送达".to_string());
                    }
                } else {
                    message_send_status = "sent".to_string();
                }
                detail = json!({
                    "scheduler": result.metadata,
                    "console_event_sent": console_event_sent,
                    "imessage_http_delivery": delivered,
                    "delivery_channel": event.channel.clone(),
                });
            }
            let _ = storage.record_execution_event(
                &event.actor,
                &event.job_id,
                &event.job_name,
                &event.channel_target,
                event.heartbeat,
                CronJobExecutionInput {
                    execution_status: if result.error.is_some() {
                        "execution_failed".to_string()
                    } else {
                        "completed".to_string()
                    },
                    message_send_status,
                    should_deliver: true,
                    delivered,
                    response_preview: Some(response),
                    error_message,
                    detail: execution_detail_with_delivery_key(detail, &event.delivery_key),
                },
            );
        });
    }
}

/// 以调度任务的 task_prompt 运行 Agent，返回完整响应文本
async fn run_scheduled_task(
    state: &Arc<AppState>,
    event: &SchedulerEvent,
) -> scheduler::ScheduledTaskExecution {
    let actor = &event.actor;
    let is_admin = state.core.is_admin_actor(actor);
    let prompt_options = PromptOptions {
        is_admin,
        ..PromptOptions::default()
    };
    let run_options = AgentRunOptions {
        timeout: Some(state.core.config.agent.overall_timeout()),
        segmenter: None,
        quota_mode: hone_channels::agent_session::AgentRunQuotaMode::ScheduledTask,
        model_override: None,
    };
    let timeout = scheduler_execution_timeout_for(state.core.config.agent.overall_timeout());
    let result = match tokio::time::timeout(
        timeout,
        scheduler::execute_scheduler_event(state.core.clone(), event, prompt_options, run_options),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            warn!(
                "⏰ [{}] Web/iMessage scheduler 执行超时: job={} target={} timeout_secs={}",
                actor.user_id,
                event.job_name,
                event.channel_target,
                timeout.as_secs()
            );
            scheduler_handler_timeout_execution(event, timeout)
        }
    };
    if !result.should_deliver {
        if let Some(err) = result.error.as_deref() {
            error!(
                "⏰ [{}] 定时任务执行失败，跳过发送: failure_kind={} err={}",
                actor.user_id,
                scheduler::scheduled_task_failure_kind(&result).unwrap_or("execution_failed"),
                err.replace('\n', "\\n")
            );
        } else {
            info!("⏰ [{}] 心跳任务未命中，跳过发送", actor.user_id);
        }
    } else if let Some(err) = result.error.as_deref() {
        error!("⏰ [{}] 定时任务执行失败: {}", actor.user_id, err);
    } else {
        info!("⏰ [{}] 定时任务完成", actor.user_id);
    }
    result
}

fn scheduler_failure_trace_required(result: &scheduler::ScheduledTaskExecution) -> bool {
    result.error.is_some()
        || result
            .metadata
            .get("failure_kind")
            .and_then(|value| value.as_str())
            == Some("internal_error_suppressed")
}

fn persist_web_scheduler_failure(
    state: &AppState,
    event: &SchedulerEvent,
    result: &scheduler::ScheduledTaskExecution,
    response: &str,
) {
    if event.channel != "web" {
        return;
    }
    let Some(session_id) = result.session_id.as_deref() else {
        return;
    };
    match state.core.session_storage.get_messages(session_id, Some(1)) {
        Ok(messages) => {
            if let Some(last) = messages.last() {
                if last.role == "assistant" && session_message_text(last).trim() == response.trim()
                {
                    return;
                }
            }
        }
        Err(err) => {
            warn!(
                "⏰ [Web] 定时任务失败提示落库前读取会话失败: session={} job={} err={}",
                session_id, event.job_name, err
            );
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("channel".to_string(), json!("web"));
    metadata.insert("source".to_string(), json!("scheduler"));
    metadata.insert("scheduler_failure".to_string(), json!(true));
    metadata.insert("job_id".to_string(), json!(event.job_id.clone()));
    metadata.insert("job_name".to_string(), json!(event.job_name.clone()));
    metadata.insert("web_push_id".to_string(), json!(event.delivery_key.clone()));
    metadata.insert(
        "delivery_key".to_string(),
        json!(event.delivery_key.clone()),
    );
    if let Some(error) = result.error.as_deref() {
        metadata.insert("error".to_string(), json!(error));
    }
    if let Err(err) =
        state
            .core
            .session_storage
            .add_message(session_id, "assistant", response, Some(metadata))
    {
        warn!(
            "⏰ [Web] 定时任务失败提示落库失败: session={} job={} err={}",
            session_id, event.job_name, err
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::ActorIdentity;
    use serde_json::Value;

    fn scheduled_result(error: Option<&str>, metadata: Value) -> scheduler::ScheduledTaskExecution {
        scheduler::ScheduledTaskExecution {
            should_deliver: false,
            content: String::new(),
            error: error.map(str::to_string),
            metadata,
            session_id: Some("session-1".to_string()),
        }
    }

    fn sample_scheduler_event(channel: &str) -> SchedulerEvent {
        SchedulerEvent {
            actor: ActorIdentity::new(channel, "web-user-1", None::<String>).expect("actor"),
            job_id: "job-1".to_string(),
            job_name: "收盘复盘".to_string(),
            task_prompt: "总结今天市场".to_string(),
            channel: channel.to_string(),
            channel_scope: None,
            channel_target: "web-user-1".to_string(),
            delivery_key: "delivery-1".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: false,
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_repeat: "daily".to_string(),
            schedule_date: None,
            last_delivered_previews: vec![],
            bypass_quiet_hours: false,
        }
    }

    fn assert_detail_bool(detail: &Value, key: &str, expected: bool) {
        assert_eq!(
            detail[key].as_bool(),
            Some(expected),
            "expected scheduler detail {key:?} to be {expected}: {detail}"
        );
    }

    #[test]
    fn scheduler_failure_trace_required_keeps_internal_suppressed_web_failures() {
        let result = scheduled_result(
            None,
            json!({
                "failure_kind": "internal_error_suppressed",
            }),
        );
        assert!(scheduler_failure_trace_required(&result));
    }

    #[test]
    fn scheduler_handler_timeout_execution_is_failure_trace() {
        let actor =
            hone_core::ActorIdentity::new("web", "web-user-timeout", None::<String>).unwrap();
        let event = SchedulerEvent {
            actor: actor.clone(),
            channel: "web".to_string(),
            channel_target: "web-user-timeout".to_string(),
            job_id: "job-timeout".to_string(),
            job_name: "timeout job".to_string(),
            task_prompt: "run".to_string(),
            channel_scope: None,
            push: json!({}),
            tags: Vec::new(),
            schedule_repeat: "daily".to_string(),
            schedule_hour: 20,
            schedule_minute: 0,
            schedule_date: None,
            last_delivered_previews: Vec::new(),
            heartbeat: false,
            bypass_quiet_hours: false,
            delivery_key: "delivery-timeout".to_string(),
        };
        let result = scheduler_handler_timeout_execution(&event, Duration::from_secs(630));

        assert!(!result.should_deliver);
        assert_eq!(
            result.session_id.as_deref(),
            Some(actor.session_id().as_str())
        );
        assert_eq!(
            result.metadata["failure_kind"].as_str(),
            Some("web_scheduler_handler_timeout")
        );
        assert!(result.error.as_deref().unwrap().contains("630s"));
        assert!(scheduler_failure_trace_required(&result));
    }

    #[test]
    fn scheduler_execution_timeout_includes_grace_period() {
        assert_eq!(
            scheduler_execution_timeout_for(Duration::from_secs(600)),
            Duration::from_secs(630)
        );
    }

    #[test]
    fn scheduler_failure_trace_required_ignores_clean_noop() {
        let result = scheduled_result(None, json!({ "parse_kind": "JsonNoop" }));
        assert!(!scheduler_failure_trace_required(&result));
    }

    #[test]
    fn web_scheduler_offline_console_still_counts_as_sent() {
        let (message_send_status, delivered) = web_scheduler_delivery_status(false);

        assert_eq!(message_send_status, "sent");
        assert!(delivered);
    }

    #[test]
    fn web_scheduler_detail_distinguishes_session_delivery_from_system_push() {
        let detail = web_scheduler_delivery_detail(json!({"status": "triggered"}), false, "web");

        assert_eq!(detail["delivery_channel"], "web");
        assert_detail_bool(&detail, "console_event_sent", false);
        assert_detail_bool(&detail, "system_push_supported", false);
        assert_detail_bool(&detail, "system_push_sent", false);
    }

    #[test]
    fn build_web_scheduler_push_event_uses_scheduled_message_payload() {
        let event = sample_scheduler_event("web");
        let stored = StoredWebPush {
            message: hone_memory::cron_job::WebPushMessage {
                push_id: "delivery-1".to_string(),
                actor_storage_key: event.actor.storage_key(),
                job_id: event.job_id.clone(),
                job_name: event.job_name.clone(),
                summary: "执行出错，请稍后重试。".to_string(),
                content: "定时任务执行出错，请稍后重试。".to_string(),
                created_at: "2026-07-10T20:00:00+08:00".to_string(),
                read_at: None,
            },
            unread_count: 3,
        };
        let push_event = build_web_scheduler_push_event(&event, &stored).expect("push event");

        assert_eq!(push_event.channel, "web");
        assert_eq!(push_event.user_id, "web-user-1");
        assert_eq!(push_event.event, "scheduled_message");
        assert_eq!(push_event.data["push_id"], "delivery-1");
        assert_eq!(push_event.data["job_id"], "job-1");
        assert_eq!(push_event.data["job_name"], "收盘复盘");
        assert_eq!(push_event.data["summary"], "执行出错，请稍后重试。");
        assert_eq!(push_event.data["unread_count"], 3);
        assert!(push_event.data.get("text").is_none());
    }

    #[tokio::test]
    async fn emit_web_scheduler_push_broadcasts_failure_prompt() {
        let event = sample_scheduler_event("web");
        let (tx, mut rx) = tokio::sync::broadcast::channel(1);
        let stored = StoredWebPush {
            message: hone_memory::cron_job::WebPushMessage {
                push_id: "delivery-1".to_string(),
                actor_storage_key: event.actor.storage_key(),
                job_id: event.job_id.clone(),
                job_name: event.job_name.clone(),
                summary: "执行出错，请稍后重试。".to_string(),
                content: "定时任务执行出错，请稍后重试。".to_string(),
                created_at: "2026-07-10T20:00:00+08:00".to_string(),
                read_at: None,
            },
            unread_count: 1,
        };

        assert!(emit_web_scheduler_push(&tx, &event, &stored));

        let push_event = rx.recv().await.expect("recv push event");
        assert_eq!(push_event.event, "scheduled_message");
        assert_eq!(push_event.data["job_id"], "job-1");
        assert_eq!(push_event.data["summary"], "执行出错，请稍后重试。");
    }
}
