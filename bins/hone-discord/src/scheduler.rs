use std::sync::Arc;
use std::time::Duration;

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler as channel_scheduler;
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
            let prompt_options = PromptOptions {
                is_admin: core_clone.is_admin_actor(&event.actor),
                ..PromptOptions::default()
            };
            let timeout_secs = core_clone.config.llm.openrouter.timeout.max(180);
            let run_options = AgentRunOptions {
                timeout: Some(Duration::from_secs(timeout_secs)),
                segmenter: None,
                quota_mode: hone_channels::agent_session::AgentRunQuotaMode::ScheduledTask,
            };
            let result = channel_scheduler::run_scheduled_task(
                core_clone.clone(),
                &event,
                prompt_options,
                run_options,
            )
            .await;
            let response = if result.response.success {
                result.response.content
            } else {
                result
                    .response
                    .error
                    .unwrap_or_else(|| "定时任务执行失败".to_string())
            };

            let channel_id = match parse_channel_id_from_target(&event.channel_target) {
                Some(id) => id,
                None => {
                    error!(
                        "[Discord] 定时任务目标解析失败: job={} target={}",
                        event.job_name, event.channel_target
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
            if sent == 0 && total > 0 {
                warn!(
                    "[Discord] 定时任务投递失败: job={} target={} sent=0",
                    event.job_name, event.channel_target
                );
            }
        });
    }
}
