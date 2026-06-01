use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use hone_channels::agent_session::AgentRunOptions;
use hone_channels::prompt::PromptOptions;
use hone_channels::scheduler;
use hone_memory::cron_job::CronJobExecutionInput;
use hone_memory::session_message_text;
use hone_scheduler::{SchedulerEvent, execution_detail_with_delivery_key};
use serde_json::json;
use tracing::{error, info, warn};

use crate::handler::{
    resolve_scheduler_receive_id, scheduler_receive_id_for_target, validate_scheduler_receive_id,
};
use crate::outbound::{scheduled_send_idempotency, send_rendered_messages};
use crate::types::AppState;

const SCHEDULER_EXECUTION_GRACE_SECS: u64 = 30;
const SCHEDULER_STALE_RECOVERY_GRACE_SECS: u64 = 30;
const SCHEDULER_HANDLER_WATCHDOG_GRACE_SECS: u64 = 5;
const SCHEDULER_TIMEOUT_FAILURE_TRANSCRIPT_MESSAGE: &str =
    "本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。";

pub(crate) async fn handle_scheduler_events(
    state: Arc<AppState>,
    mut event_rx: tokio::sync::mpsc::Receiver<SchedulerEvent>,
) {
    info!("⏰ 调度事件处理器已启动（渠道: feishu）");
    recover_stale_started_rows(&state);
    while let Some(event) = event_rx.recv().await {
        if event.channel != "feishu" {
            continue;
        }

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
            let handler_completed = Arc::new(AtomicBool::new(false));
            let watchdog_recovered = Arc::new(AtomicBool::new(false));
            let watchdog = spawn_scheduler_timeout_watchdog(
                state_clone.clone(),
                event.clone(),
                handler_completed.clone(),
                watchdog_recovered.clone(),
            );
            let result = run_scheduled_task(&state_clone, &event).await;
            handler_completed.store(true, Ordering::SeqCst);
            if watchdog_recovered.load(Ordering::SeqCst) {
                warn!(
                    "[Feishu] 定时任务已由独立 watchdog 收口，跳过迟到结果: job={} target={} delivery_key={}",
                    event.job_name, event.channel_target, event.delivery_key
                );
                return;
            }
            watchdog.abort();
            if !result.should_deliver {
                if let Some(err) = result.error.as_deref() {
                    error!(
                        "[Feishu] 定时任务执行失败，本轮不发送: job={} target={} failure_kind={} err={}",
                        event.job_name,
                        event.channel_target,
                        scheduler::scheduled_task_failure_kind(&result)
                            .unwrap_or("execution_failed"),
                        err.replace('\n', "\\n")
                    );
                } else {
                    info!(
                        "[Feishu] 心跳任务未命中，本轮不发送: job={} target={}",
                        event.job_name, event.channel_target
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
            let receive_id = if let Some(overridden) =
                scheduler_receive_id_for_target(&event.actor, &event.channel_target)
            {
                overridden
            } else {
                match resolve_scheduler_receive_id(
                    &state_clone.facade,
                    &event.channel_target,
                    &state_clone.core.config.feishu.allow_emails,
                    &state_clone.core.config.feishu.allow_mobiles,
                )
                .await
                {
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
                                detail: execution_detail_with_delivery_key(
                                    result.metadata.clone(),
                                    &event.delivery_key,
                                ),
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
                        detail: execution_detail_with_delivery_key(
                            result.metadata.clone(),
                            &event.delivery_key,
                        ),
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
                        detail: execution_detail_with_delivery_key(
                            json!({
                                "receive_id": receive_id,
                                "scheduler": result.metadata,
                            }),
                            &event.delivery_key,
                        ),
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
                        detail: execution_detail_with_delivery_key(
                            json!({
                                "receive_id": receive_id,
                                "scheduler": result.metadata,
                            }),
                            &event.delivery_key,
                        ),
                    },
                );
            } else {
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
                        detail: execution_detail_with_delivery_key(
                            json!({
                                "receive_id": receive_id,
                                "scheduler": result.metadata,
                            }),
                            &event.delivery_key,
                        ),
                    },
                );
            }
        });
    }
}

fn scheduler_execution_timeout(state: &AppState) -> Duration {
    state
        .core
        .config
        .agent
        .overall_timeout()
        .saturating_add(Duration::from_secs(SCHEDULER_EXECUTION_GRACE_SECS))
}

fn scheduler_handler_watchdog_timeout(state: &AppState) -> Duration {
    scheduler_execution_timeout(state)
        .saturating_add(Duration::from_secs(SCHEDULER_HANDLER_WATCHDOG_GRACE_SECS))
}

fn spawn_scheduler_timeout_watchdog(
    state: Arc<AppState>,
    event: SchedulerEvent,
    handler_completed: Arc<AtomicBool>,
    watchdog_recovered: Arc<AtomicBool>,
) -> tokio::task::JoinHandle<()> {
    let timeout = scheduler_handler_watchdog_timeout(&state);
    tokio::spawn(async move {
        tokio::time::sleep(timeout).await;
        if handler_completed.load(Ordering::SeqCst) {
            return;
        }
        if mark_scheduler_handler_watchdog_timeout(&state, &event, timeout) {
            watchdog_recovered.store(true, Ordering::SeqCst);
            warn!(
                "[Feishu] scheduler watchdog 已将超时定时任务收口为失败: job={} target={} timeout_secs={} delivery_key={}",
                event.job_name,
                event.channel_target,
                timeout.as_secs(),
                event.delivery_key
            );
        }
    })
}

fn mark_scheduler_handler_watchdog_timeout(
    state: &AppState,
    event: &SchedulerEvent,
    timeout: Duration,
) -> bool {
    let reason = format!("scheduler_handler_watchdog_timeout:{}s", timeout.as_secs());
    match state
        .core
        .cron_job_storage()
        .mark_started_execution_failed_by_delivery_key(
            &event.actor,
            &event.job_id,
            &event.channel_target,
            event.heartbeat,
            &event.delivery_key,
            "feishu_scheduler_handler_watchdog",
            &reason,
        ) {
        Ok(0) => false,
        Ok(_) => {
            persist_scheduler_timeout_failure_turn(
                &state.core.session_storage,
                &event.actor.session_id(),
                "scheduler_handler_watchdog_timeout",
            );
            true
        }
        Err(err) => {
            warn!(
                "[Feishu] scheduler watchdog 超时收口失败: job={} target={} err={}",
                event.job_name, event.channel_target, err
            );
            false
        }
    }
}

fn recover_stale_started_rows(state: &AppState) {
    let recovery_window = scheduler_execution_timeout(state)
        .saturating_add(Duration::from_secs(SCHEDULER_STALE_RECOVERY_GRACE_SECS));
    let Ok(recovery_delta) = chrono::TimeDelta::from_std(recovery_window) else {
        warn!("[Feishu] scheduler 启动恢复：非法恢复窗口");
        return;
    };
    let stale_before = (chrono::Utc::now() - recovery_delta).to_rfc3339();
    match state
        .core
        .cron_job_storage()
        .recover_stale_started_executions(
            "feishu",
            &stale_before,
            "feishu_scheduler_startup",
            "Feishu scheduler runtime restarted before this run reached a terminal status",
        ) {
        Ok(0) => {}
        Ok(count) => warn!("[Feishu] 已回收上一进程遗留的 stale pending 定时任务: count={count}"),
        Err(err) => warn!(
            "[Feishu] 回收上一进程 stale pending 定时任务失败: err={}",
            err
        ),
    }
}

fn persist_scheduler_timeout_failure_turn(
    storage: &hone_memory::SessionStorage,
    session_id: &str,
    failure_kind: &str,
) {
    if session_id.is_empty() {
        return;
    }
    match storage.get_messages(session_id, Some(1)) {
        Ok(messages) => {
            if messages.last().is_some_and(|message| {
                message.role == "assistant"
                    && session_message_text(message) == SCHEDULER_TIMEOUT_FAILURE_TRANSCRIPT_MESSAGE
            }) {
                return;
            }
        }
        Err(err) => {
            warn!(
                "[Feishu] scheduler timeout: failed to inspect session tail session_id={} err={}",
                session_id, err
            );
            return;
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert(
        "scheduler_failure".to_string(),
        serde_json::Value::Bool(true),
    );
    metadata.insert(
        "failure_kind".to_string(),
        serde_json::Value::String(failure_kind.to_string()),
    );
    if let Err(err) = storage.add_message(
        session_id,
        "assistant",
        SCHEDULER_TIMEOUT_FAILURE_TRANSCRIPT_MESSAGE,
        Some(metadata),
    ) {
        warn!(
            "[Feishu] scheduler timeout: failed to persist failure transcript session_id={} err={}",
            session_id, err
        );
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
    let timeout = scheduler_execution_timeout(state);
    match tokio::time::timeout(
        timeout,
        scheduler::execute_scheduler_event(state.core.clone(), event, prompt_options, run_options),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            let session_id = actor.session_id();
            warn!(
                "[Feishu] scheduler 执行超时: job={} target={} timeout_secs={}",
                event.job_name,
                event.channel_target,
                timeout.as_secs()
            );
            persist_scheduler_timeout_failure_turn(
                &state.core.session_storage,
                &session_id,
                "scheduler_handler_timeout",
            );
            scheduler::ScheduledTaskExecution {
                should_deliver: false,
                content: String::new(),
                error: Some(format!("scheduler_handler_timeout:{}s", timeout.as_secs())),
                metadata: json!({
                    "failure_kind": "scheduler_handler_timeout",
                    "timeout_secs": timeout.as_secs(),
                }),
                session_id: Some(session_id),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::persist_scheduler_timeout_failure_turn;

    #[test]
    fn persist_scheduler_timeout_failure_turn_is_idempotent() {
        let root = std::env::temp_dir().join(format!(
            "hone_feishu_scheduler_timeout_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create temp dir");
        let storage = hone_memory::SessionStorage::new(root.join("sessions"));
        let actor =
            hone_core::ActorIdentity::new("feishu", "ou_timeout", None::<String>).expect("actor");
        let session_id = actor.session_id();

        storage
            .create_session(Some(&session_id), Some(actor.clone()), None)
            .expect("create session");
        storage
            .add_message(&session_id, "user", "[定时任务触发] test", None)
            .expect("add user");

        persist_scheduler_timeout_failure_turn(&storage, &session_id, "scheduler_handler_timeout");
        persist_scheduler_timeout_failure_turn(&storage, &session_id, "scheduler_handler_timeout");

        let messages = storage.get_messages(&session_id, None).expect("messages");
        let assistant_messages = messages
            .iter()
            .filter(|message| message.role == "assistant")
            .count();
        assert_eq!(assistant_messages, 1);

        let metadata = messages
            .last()
            .and_then(|message| message.metadata.as_ref())
            .expect("assistant metadata");
        assert_eq!(
            metadata
                .get("failure_kind")
                .and_then(|value| value.as_str()),
            Some("scheduler_handler_timeout")
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
