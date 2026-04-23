use std::sync::Arc;

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_scheduler::SchedulerEvent;
use serde_json::json;
use tracing::{error, info, warn};

use crate::handler::{
    resolve_receive_id, scheduler_receive_id_for_target, validate_scheduler_receive_id,
};
use crate::outbound::{scheduled_send_idempotency, send_rendered_messages};
use crate::types::AppState;

pub(crate) async fn handle_scheduler_events(
    state: Arc<AppState>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: feishu）");
    while let Some(event) = event_rx.recv().await {
        if event.channel != "feishu" {
            continue;
        }

        let state_clone = state.clone();
        tokio::spawn(async move {
            let storage = state_clone.core.cron_job_storage();
            let result = run_scheduled_task(&state_clone, &event).await;
            if !result.should_deliver {
                info!(
                    "[Feishu] 心跳任务未命中，本轮不发送: job={} target={}",
                    event.job_name, event.channel_target
                );
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
            let response = result
                .error
                .clone()
                .unwrap_or_else(|| result.content.clone());
            let receive_id = if let Some(overridden) =
                scheduler_receive_id_for_target(&event.actor, &event.channel_target)
            {
                overridden
            } else {
                match resolve_receive_id(&state_clone.facade, &event.channel_target).await {
                    Ok(id) => id,
                    Err(err) => {
                        error!(
                            "[Feishu] 定时任务目标解析失败: job={} target={} err={}",
                            event.job_name, event.channel_target, err
                        );
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
                                message_send_status: "target_resolution_failed".to_string(),
                                should_deliver: true,
                                delivered: false,
                                response_preview: Some(response.clone()),
                                error_message: Some(err.to_string()),
                                detail: result.metadata.clone(),
                            },
                        );
                        return;
                    }
                }
            };
            if let Err(err) =
                validate_scheduler_receive_id(&event.actor, &event.channel_target, &receive_id)
            {
                error!(
                    "[Feishu] 定时任务目标校验失败: job={} target={} receive_id={} err={}",
                    event.job_name, event.channel_target, receive_id, err
                );
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
                        message_send_status: "target_resolution_failed".to_string(),
                        should_deliver: true,
                        delivered: false,
                        response_preview: Some(response.clone()),
                        error_message: Some(err.to_string()),
                        detail: result.metadata.clone(),
                    },
                );
                return;
            }
            let idempotency = scheduled_send_idempotency(&event, &receive_id, &response, "open_id");
            if state_clone
                .scheduled_dedup
                .is_duplicate(&idempotency.dedup_key)
            {
                warn!(
                    "[Feishu] 已拦截重复定时任务投递: job={} delivery_key={} target={}",
                    event.job_name, event.delivery_key, receive_id
                );
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
                        message_send_status: "duplicate_suppressed".to_string(),
                        should_deliver: true,
                        delivered: false,
                        response_preview: Some(response.clone()),
                        error_message: result.error.clone(),
                        detail: json!({
                            "receive_id": receive_id,
                            "delivery_key": event.delivery_key,
                            "scheduler": result.metadata,
                        }),
                    },
                );
                return;
            }

            if let Err(err) = send_rendered_messages(
                &state_clone.facade,
                &receive_id,
                "open_id",
                &response,
                state_clone.core.config.feishu.max_message_length,
                None,
                Some(&idempotency.uuid_seed),
            )
            .await
            {
                error!(
                    "[Feishu] 定时任务投递失败: job={} target={} err={}",
                    event.job_name, event.channel_target, err
                );
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
                        message_send_status: "send_failed".to_string(),
                        should_deliver: true,
                        delivered: false,
                        response_preview: Some(response.clone()),
                        error_message: Some(err.to_string()),
                        detail: json!({
                            "receive_id": receive_id,
                            "delivery_key": event.delivery_key,
                            "scheduler": result.metadata,
                        }),
                    },
                );
            } else {
                if result.error.is_none() {
                    scheduler::persist_feed_push_to_session(
                        &state_clone.core,
                        &event.actor,
                        &result.content,
                        &event.job_id,
                        &event.job_name,
                    );
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
                        message_send_status: "sent".to_string(),
                        should_deliver: true,
                        delivered: true,
                        response_preview: Some(response),
                        error_message: result.error.clone(),
                        detail: json!({
                            "receive_id": receive_id,
                            "delivery_key": event.delivery_key,
                            "scheduler": result.metadata,
                        }),
                    },
                );
            }
        });
    }
}

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
    scheduler::execute_scheduler_event(state.core.clone(), event, prompt_options, run_options).await
}
