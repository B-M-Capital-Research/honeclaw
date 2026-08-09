use std::sync::Arc;

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::outbound::PlatformMessageSplitter;
use hone_channels::prompt::PromptOptions;
use hone_channels::runtime::sanitize_user_visible_output;
use hone_channels::scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_scheduler::{
    SchedulerEvent, execution_detail_with_delivery_key, scheduler_event_is_active,
    with_unsubscribe_footer,
};
use serde_json::json;
use teloxide::prelude::{Bot, ChatId};
use tracing::{error, info};

use crate::listener::{TelegramSplitter, send_segments};
use crate::markdown_v2::sanitize_telegram_html_public;

fn scheduler_public_response_text(text: &str) -> String {
    let sanitized = sanitize_user_visible_output(text).content;
    let filtered = sanitized
        .lines()
        .filter(|line| line.trim() != "{}")
        .collect::<Vec<_>>()
        .join("\n");
    sanitize_telegram_html_public(&filtered)
}

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
            let result = run_scheduled_task(&core_clone, &event, &storage).await;
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
            // Every delivered push carries its own way out. Doing this in the
            // shared helper rather than per channel is what keeps a new
            // channel from silently shipping pushes nobody can stop.
            let response = with_unsubscribe_footer(response, &core_clone.config, &event.job_id);
            let response = scheduler_public_response_text(&response);
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
            let segments = TelegramSplitter
                .split_html(&response, core_clone.config.telegram.max_message_length);
            let context_segments = segments.clone();
            let total_segments = segments.len();
            if !scheduler_event_is_active(&storage, &event) {
                info!(
                    "[Telegram] 定时任务已取消，抑制发送: job={} target={}",
                    event.job_name, event.channel_target
                );
                let _ = storage.record_execution_event(
                    &event.actor,
                    &event.job_id,
                    &event.job_name,
                    &event.channel_target,
                    event.heartbeat,
                    CronJobExecutionInput {
                        execution_status: "noop".to_string(),
                        message_send_status: "skipped_cancelled".to_string(),
                        should_deliver: false,
                        delivered: false,
                        response_preview: None,
                        error_message: None,
                        detail: execution_detail_with_delivery_key(
                            json!({"skipped": "job_cancelled"}),
                            &event.delivery_key,
                        ),
                    },
                );
                return;
            }
            let sent = send_segments(&bot_clone, ChatId(chat_id), segments, None).await;
            if sent > 0 {
                let delivered_context = context_segments
                    .iter()
                    .take(sent)
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join("\n");
                scheduler::record_confirmed_scheduled_delivery(
                    &core_clone,
                    &event,
                    &result,
                    &delivered_context,
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
    storage: &hone_memory::CronJobStorage,
) -> scheduler::ScheduledTaskExecution {
    let prompt_options = PromptOptions::default();
    scheduler::execute_scheduler_event_with_storage(
        core.clone(),
        event,
        prompt_options,
        AgentRunOptions::default(),
        storage,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::scheduler_public_response_text;

    #[test]
    fn scheduler_public_response_text_hides_internal_output_and_normalizes_html() {
        let raw = "<think>先想想</think>\n**结论**<tool_call>{}</tool_call>";
        let sanitized = scheduler_public_response_text(raw);
        assert_eq!(sanitized, "<b>结论</b>");
    }
}
