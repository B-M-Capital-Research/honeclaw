use std::sync::Arc;

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler as channel_scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_scheduler::SchedulerEvent;
use serenity::all::ChannelId;
use serenity::http::Http;
use tracing::{error, info, warn};

use crate::utils::{parse_channel_id_from_target, send_or_edit_segments, split_into_segments};

pub(crate) async fn handle_scheduler_events(
    http: Arc<Http>,
    core: Arc<hone_channels::HoneBotCore>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: discord）");
    while let Some(event) = event_rx.recv().await {
        if event.channel != "discord" {
            continue;
        }

        let http_clone = http.clone();
        let core_clone = core.clone();
        tokio::spawn(async move {
            let storage = core_clone.cron_job_storage();
            let prompt_options = PromptOptions {
                is_admin: core_clone.is_admin_actor(&event.actor),
                ..PromptOptions::default()
            };
            let run_options = AgentRunOptions {
                timeout: Some(core_clone.config.agent.overall_timeout()),
                segmenter: None,
                quota_mode: hone_channels::agent_session::AgentRunQuotaMode::ScheduledTask,
                model_override: None,
            };
            let result = channel_scheduler::execute_scheduler_event(
                core_clone.clone(),
                &event,
                prompt_options,
                run_options,
            )
            .await;
            if !result.should_deliver {
                info!(
                    "[Discord] 心跳任务未命中，本轮不发送: job={} target={}",
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

            let channel_id = match parse_channel_id_from_target(&event.channel_target) {
                Some(id) => id,
                None => {
                    error!(
                        "[Discord] 定时任务目标解析失败: job={} target={}",
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
                            error_message: Some("Discord 定时任务目标解析失败".to_string()),
                            detail: result.metadata.clone(),
                        },
                    );
                    return;
                }
            };

            let segments =
                split_into_segments(&response, core_clone.config.discord.max_message_length);
            let (sent, total) = send_or_edit_segments(
                http_clone.as_ref(),
                ChannelId::new(channel_id),
                None,
                segments,
            )
            .await;
            let message_send_status = if sent > 0 {
                "sent".to_string()
            } else {
                "send_failed".to_string()
            };
            if sent == 0 && total > 0 {
                warn!(
                    "[Discord] 定时任务投递失败: job={} target={} sent=0",
                    event.job_name, event.channel_target
                );
            }
            if sent > 0 && result.error.is_none() {
                channel_scheduler::persist_feed_push_to_session(
                    &core_clone,
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
                    message_send_status,
                    should_deliver: true,
                    delivered: sent > 0,
                    response_preview: Some(response),
                    error_message: result
                        .error
                        .clone()
                        .filter(|value| !value.trim().is_empty()),
                    detail: serde_json::json!({
                        "sent_segments": sent,
                        "total_segments": total,
                        "scheduler": result.metadata,
                    }),
                },
            );
        });
    }
}
