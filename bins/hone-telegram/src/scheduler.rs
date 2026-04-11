use std::sync::Arc;

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::outbound::split_segments;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_scheduler::SchedulerEvent;
use serde_json::json;
use teloxide::prelude::{Bot, ChatId};
use tracing::{error, info};

use crate::listener::send_segments;

pub(crate) async fn handle_scheduler_events(
    bot: Bot,
    core: Arc<hone_channels::HoneBotCore>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: telegram）");
    while let Some(event) = event_rx.recv().await {
        if event.channel != "telegram" {
            continue;
        }

        let bot_clone = bot.clone();
        let core_clone = core.clone();
        tokio::spawn(async move {
            let storage = core_clone.cron_job_storage();
            let result = run_scheduled_task(&core_clone, &event).await;
            if !result.should_deliver {
                info!(
                    "[Telegram] 心跳任务未命中，本轮不发送: job={} target={}",
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
            let chat_id: i64 = match event.channel_target.parse() {
                Ok(id) => id,
                Err(_) => {
                    error!(
                        "[Telegram] 定时任务目标解析失败: job={} target={} ",
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
                                "completed".to_string()
                            },
                            message_send_status: "target_resolution_failed".to_string(),
                            should_deliver: true,
                            delivered: false,
                            response_preview: Some(response.clone()),
                            error_message: Some("Telegram 定时任务目标解析失败".to_string()),
                            detail: result.metadata.clone(),
                        },
                    );
                    return;
                }
            };
            let segments = split_segments(
                &response,
                core_clone.config.telegram.max_message_length,
                3500,
            );
            let total_segments = segments.len();
            let sent = send_segments(&bot_clone, ChatId(chat_id), segments, None).await;
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
                    message_send_status: if sent > 0 {
                        "sent".to_string()
                    } else {
                        "send_failed".to_string()
                    },
                    should_deliver: true,
                    delivered: sent > 0,
                    response_preview: Some(response),
                    error_message: result.error.clone(),
                    detail: json!({
                        "sent_segments": sent,
                        "total_segments": total_segments,
                        "scheduler": result.metadata,
                    }),
                },
            );
        });
    }
}

async fn run_scheduled_task(
    core: &Arc<hone_channels::HoneBotCore>,
    event: &SchedulerEvent,
) -> scheduler::ScheduledTaskExecution {
    let prompt_options = PromptOptions::default();
    scheduler::execute_scheduler_event(
        core.clone(),
        event,
        prompt_options,
        AgentRunOptions::default(),
    )
    .await
}
