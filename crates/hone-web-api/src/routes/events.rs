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
use hone_scheduler::SchedulerEvent;

use crate::routes::normalized_query_actor;
use crate::state::{AppState, PushEvent};
use crate::types::UserIdQuery;

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
                            "noop".to_string()
                        },
                        message_send_status: if result.error.is_some() {
                            "skipped_error".to_string()
                        } else {
                            "skipped_noop".to_string()
                        },
                        should_deliver: false,
                        delivered: false,
                        response_preview: None,
                        error_message: result.error.clone(),
                        detail: result.metadata.clone(),
                    },
                );
                return;
            }
            let response = if result.error.is_some() {
                format!("定时任务「{}」执行出错，请稍后重试。", event.job_name)
            } else {
                result.content.clone()
            };

            // 1. 推送到 Web 控制台 SSE（供控制台页面实时展示）
            let push_result = state_clone.push_tx.send(PushEvent {
                channel: event.actor.channel.clone(),
                user_id: event.actor.user_id.clone(),
                channel_scope: event.actor.channel_scope.clone(),
                event: "scheduled_message".into(),
                data: json!({
                    "text": response.clone(),
                    "job_name": event.job_name.clone(),
                    "job_id": event.job_id.clone(),
                }),
            });

            // 2. 若是 iMessage 渠道，把结果通过 hone-imessage 内置 HTTP 服务投递给用户
            let mut message_send_status = "sent".to_string();
            let mut delivered = true;
            let mut error_message = result.error.clone();
            let mut detail = json!({
                "scheduler": result.metadata,
                "console_event_sent": push_result.is_ok(),
                "delivery_channel": event.channel.clone(),
            });
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
                    "console_event_sent": push_result.is_ok(),
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
                    detail,
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
    let result =
        scheduler::execute_scheduler_event(state.core.clone(), event, prompt_options, run_options)
            .await;
    if !result.should_deliver {
        info!("⏰ [{}] 心跳任务未命中，跳过发送", actor.user_id);
    } else if let Some(err) = result.error.as_deref() {
        error!("⏰ [{}] 定时任务执行失败: {}", actor.user_id, err);
    } else {
        info!("⏰ [{}] 定时任务完成", actor.user_id);
    }
    result
}
