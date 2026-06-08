use std::sync::Arc;

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler as channel_scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_scheduler::{SchedulerEvent, execution_detail_with_delivery_key};
use serde_json::{Value, json};
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
                        detail: execution_detail_with_delivery_key(
                            result.metadata.clone(),
                            &event.delivery_key,
                        ),
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
                            detail: execution_detail_with_delivery_key(
                                json!({
                                    "failure_kind": "discord_target_resolution_failed",
                                    "scheduler": result.metadata,
                                }),
                                &event.delivery_key,
                            ),
                        },
                    );
                    return;
                }
            };

            let segments =
                split_into_segments(&response, core_clone.config.discord.max_message_length);
            let send_result = send_or_edit_segments(
                http_clone.as_ref(),
                ChannelId::new(channel_id),
                None,
                segments,
            )
            .await;
            let message_send_status = if send_result.sent > 0 {
                "sent".to_string()
            } else {
                "send_failed".to_string()
            };
            if send_result.sent == 0 && send_result.total > 0 {
                warn!(
                    "[Discord] 定时任务投递失败: job={} target={} sent=0 error={}",
                    event.job_name,
                    event.channel_target,
                    send_result.error.as_deref().unwrap_or("unknown")
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
                    delivered: send_result.sent > 0,
                    response_preview: Some(response),
                    error_message: scheduler_error_message(
                        result.error.clone(),
                        send_result.error.clone(),
                        send_result.sent,
                        send_result.total,
                    ),
                    detail: scheduler_delivery_detail(
                        result.metadata,
                        &event.delivery_key,
                        send_result.sent,
                        send_result.total,
                        send_result.error.as_deref(),
                    ),
                },
            );
        });
    }
}

fn scheduler_error_message(
    run_error: Option<String>,
    send_error: Option<String>,
    sent_segments: usize,
    total_segments: usize,
) -> Option<String> {
    let run_error = run_error.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    });
    if run_error.is_some() {
        return run_error;
    }
    let send_error = send_error.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    });
    if send_error.is_some() {
        return send_error;
    }
    if sent_segments == 0 && total_segments > 0 {
        return Some("Discord 定时任务发送失败".to_string());
    }
    None
}

fn scheduler_delivery_detail(
    metadata: Value,
    delivery_key: &str,
    sent_segments: usize,
    total_segments: usize,
    send_error: Option<&str>,
) -> Value {
    let mut detail = json!({
        "sent_segments": sent_segments,
        "total_segments": total_segments,
        "scheduler": metadata,
    });

    if sent_segments == 0 && total_segments > 0 {
        detail["failure_kind"] = Value::String("discord_send_failed".to_string());
    }
    if let Some(error) = send_error.map(str::trim).filter(|value| !value.is_empty()) {
        detail["send_error"] = Value::String(error.to_string());
    }

    execution_detail_with_delivery_key(detail, delivery_key)
}

#[cfg(test)]
mod tests {
    use super::{scheduler_delivery_detail, scheduler_error_message};
    use serde_json::json;

    #[test]
    fn scheduler_error_message_prefers_run_error() {
        let error = scheduler_error_message(
            Some("runner failed".to_string()),
            Some("send failed".to_string()),
            0,
            3,
        );
        assert_eq!(error.as_deref(), Some("runner failed"));
    }

    #[test]
    fn scheduler_error_message_uses_send_error_when_delivery_fails() {
        let error = scheduler_error_message(
            None,
            Some("发送 Discord 消息失败: missing access".to_string()),
            0,
            3,
        );
        assert_eq!(
            error.as_deref(),
            Some("发送 Discord 消息失败: missing access")
        );
    }

    #[test]
    fn scheduler_error_message_falls_back_to_generic_send_failure() {
        let error = scheduler_error_message(None, None, 0, 3);
        assert_eq!(error.as_deref(), Some("Discord 定时任务发送失败"));
    }

    #[test]
    fn scheduler_delivery_detail_keeps_delivery_key_and_send_failure() {
        let detail = scheduler_delivery_detail(
            json!({"runner": "ok"}),
            "delivery-123",
            0,
            3,
            Some("missing channel access"),
        );

        assert_eq!(detail["delivery_key"], "delivery-123");
        assert_eq!(detail["sent_segments"], 0);
        assert_eq!(detail["total_segments"], 3);
        assert_eq!(detail["failure_kind"], "discord_send_failed");
        assert_eq!(detail["send_error"], "missing channel access");
        assert_eq!(detail["scheduler"]["runner"], "ok");
    }
}
