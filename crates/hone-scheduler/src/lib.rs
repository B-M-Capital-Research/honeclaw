//! Hone Scheduler — 定时任务调度器
//!
//! 使用 tokio interval 实现分钟级 cron 调度。

use chrono::{Timelike, Utc};
use hone_core::ActorIdentity;
use hone_core::config::HoneConfig;
use hone_memory::{
    CronJobStorage,
    cron_job::{CronJobExecutionInput, ExecutionFilter},
};
use serde_json::{Value, json};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::mpsc;
use tracing::{info, warn};

/// 同一时刻允许并行执行的定时任务数。
///
/// 生产实证(2026-08-15):同分钟到期的任务此前是各渠道裸 `tokio::spawn`、完全无
/// 上限。北京 22:09 的 8 个任务齐射把上游 LLM 的**建连**阶段打挂(16 次调用全部
/// `error sending request`),而同一时刻手工单发流式请求 2.2 秒即成功;峰值单分钟
/// 曾达 86 次执行,这些分钟的失败率明显高于基线。
///
/// 闸设在 job 层而不是 LLM provider 内部,原因有二:
/// 1. provider 实例并不共享——`main` profile 是 `HoneBotCore` 上的单例,但
///    event-engine 另建了约 10 个独立 Provider(各自独立 `reqwest::Client` 与
///    连接池),闸放实例内部限不住全局并发;
/// 2. `chat_with_tools_stream` 返回 `BoxStream`,permit 必须随 stream 一起 move
///    才能覆盖读流阶段,否则只闸住建连、形同虚设。
///
/// 放在 job 层则天然覆盖整条下游链路。每个渠道进程各持一份(渠道本就是独立进程)。
const DEFAULT_JOB_CONCURRENCY: usize = 4;

fn configured_job_concurrency() -> usize {
    std::env::var("HONE_SCHEDULER_JOB_CONCURRENCY")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_JOB_CONCURRENCY)
}

static JOB_SLOTS: std::sync::LazyLock<Arc<tokio::sync::Semaphore>> =
    std::sync::LazyLock::new(|| {
        Arc::new(tokio::sync::Semaphore::new(configured_job_concurrency()))
    });

/// 取一个定时任务执行位;permit 在返回值 drop 时归还。
///
/// 调用点应放在**已 spawn 的任务内部**,而不是收事件之前:在收事件前等 permit
/// 会让一个卡住的任务连带堵死整个消费循环;放在任务内部则事件照收,超额任务在此
/// 排队,排队深度天然被单轮到期任务数限住。
pub async fn acquire_job_slot() -> Option<tokio::sync::OwnedSemaphorePermit> {
    JOB_SLOTS.clone().acquire_owned().await.ok()
}

/// 回收上一进程遗留的 `running/pending` 执行记录。
///
/// `mark_job_run` 在任务**送进队列时**就落 `last_run_at`(而不是执行成功后),
/// 所以进程中途崩溃的任务当天不会重跑,并在执行历史里留下永久 `running/pending`
/// 行。生产实测:今日 15 条,历史累计约 40 条,且每次重启新增。
///
/// 这段回收此前**只有飞书渠道实现**,web / telegram / discord 全都没有。
/// 提到这里由各渠道启动时统一调用。
pub async fn recover_stale_started_rows(
    storage: &CronJobStorage,
    channel: &str,
    recovery_window: std::time::Duration,
    reason: &str,
) {
    let Ok(recovery_delta) = chrono::TimeDelta::from_std(recovery_window) else {
        warn!("[{channel}] scheduler 启动恢复：非法恢复窗口");
        return;
    };
    let stale_before = (Utc::now() - recovery_delta).to_rfc3339();
    match storage
        .recover_stale_started_executions(
            channel,
            &stale_before,
            reason,
            "Scheduler runtime restarted before this run reached a terminal status",
        )
        .await
    {
        Ok(0) => {}
        Ok(count) => {
            warn!("[{channel}] 已回收上一进程遗留的 stale pending 定时任务: count={count}")
        }
        Err(err) => warn!("[{channel}] 回收上一进程 stale pending 定时任务失败: err={err}"),
    }
}

/// 定时任务触发事件
#[derive(Debug, Clone)]
pub struct SchedulerEvent {
    pub actor: ActorIdentity,
    pub job_id: String,
    pub job_name: String,
    pub task_prompt: String,
    pub channel: String,
    pub channel_scope: Option<String>,
    pub channel_target: String,
    pub delivery_key: String,
    pub push: serde_json::Value,
    pub tags: Vec<String>,
    pub heartbeat: bool,
    pub schedule_hour: u32,
    pub schedule_minute: u32,
    pub schedule_repeat: String,
    pub schedule_date: Option<String>,
    /// 最近几轮已送达的提醒摘要，仅 heartbeat 任务填充，用于去重判断
    pub last_delivered_previews: Vec<(String, String)>,
    /// 是否绕过用户的 quiet_hours 静音。来源是 `CronJob.bypass_quiet_hours`，
    /// 默认 false（cron 任务遵守用户的勿扰时段）。
    pub bypass_quiet_hours: bool,
}

/// Return whether a queued scheduler event still belongs to a durable task.
///
/// Cancellation deletes the actor-owned job, so queued or in-flight events
/// must re-check this immediately before model work and delivery. One-shot
/// jobs are disabled when dispatched, but remain registered with `last_run_at`
/// set; they are still allowed to finish that single claimed run.
pub async fn scheduler_event_is_active(storage: &CronJobStorage, event: &SchedulerEvent) -> bool {
    storage
        .get_job(&event.job_id, Some(&event.actor))
        .await
        .is_some_and(|(_, job)| {
            job.enabled
                || (event.schedule_repeat.eq_ignore_ascii_case("once")
                    && job.schedule.repeat.eq_ignore_ascii_case("once")
                    && job.last_run_at.is_some())
        })
}

/// Ensure cron execution history can finalize the pre-written `running/pending` row.
///
/// `CronJobStorage::record_execution_event` matches terminal records to started
/// records by a top-level `detail.delivery_key`. Scheduler execution metadata is
/// often a domain-specific object and may not carry that key, so channel
/// handlers should wrap every terminal detail through this helper before
/// recording it.
/// Appends the login-free unsubscribe footer to a push about to be delivered.
///
/// Every channel builds its outgoing text the same way, so this lives here
/// rather than in each binary: a channel that forgets the footer is a channel
/// whose recipients have no way to stop a push, and that is exactly the kind
/// of omission nobody notices until someone complains.
///
/// Returns the text unchanged when no signing secret is configured. A link
/// that cannot work is worse than no link — it reads as an offer the product
/// then fails to honour.
pub fn with_unsubscribe_footer(response: String, config: &HoneConfig, job_id: &str) -> String {
    let secret = config.email.resolved_unsubscribe_secret();
    let Some(token) = hone_core::unsubscribe_token::issue_unsubscribe_token(&secret, job_id) else {
        return response;
    };
    let link =
        hone_core::unsubscribe_token::unsubscribe_link(&config.agent.hone_cloud.base_url, &token);
    // A failing job is precisely when someone wants out, so the footer is
    // attached to error notices too.
    format!("{}\n\n———\n不想再收到这条推送？{link}", response.trim_end())
}

pub fn execution_detail_with_delivery_key(detail: Value, delivery_key: &str) -> Value {
    let mut object = match detail {
        Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            if !other.is_null() {
                map.insert("scheduler".to_string(), other);
            }
            map
        }
    };
    let existing_key = object
        .get("delivery_key")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim();
    if existing_key.is_empty() {
        object.insert(
            "delivery_key".to_string(),
            Value::String(delivery_key.to_string()),
        );
    }
    Value::Object(object)
}

/// 定时任务调度器
pub struct HoneScheduler {
    storage: Arc<CronJobStorage>,
    event_tx: mpsc::Sender<SchedulerEvent>,
    /// 本调度器负责的渠道列表，只触发 job.channel 在列表中的任务。
    /// 空列表表示处理所有渠道（通常不使用）。
    channels: Vec<String>,
}

impl HoneScheduler {
    pub fn new(
        storage: Arc<CronJobStorage>,
        event_tx: mpsc::Sender<SchedulerEvent>,
        channels: Vec<String>,
    ) -> Self {
        Self {
            storage,
            event_tx,
            channels,
        }
    }

    /// 启动调度循环（每分钟检查一次）
    pub async fn start(&self) {
        info!("⏰ 定时任务调度器启动");

        // 对齐到下一分钟的 0 秒
        let now = Utc::now();
        let secs_into_minute = now.second();
        if secs_into_minute > 0 {
            let wait_secs = 60 - secs_into_minute;
            tokio::time::sleep(std::time::Duration::from_secs(wait_secs as u64)).await;
        }

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        loop {
            interval.tick().await;
            self.check_due_jobs().await;
        }
    }

    /// 检查到期任务
    async fn check_due_jobs(&self) {
        let now = hone_core::local_now();

        // 只查询本进程负责的渠道，防止多进程共享存储时跨渠道误标记
        let channel_refs: Vec<&str> = self.channels.iter().map(|s| s.as_str()).collect();
        let due_jobs = self.storage.get_due_jobs_at(now, &channel_refs).await;

        if due_jobs.is_empty() {
            return;
        }

        info!(
            "⏰ 发现 {} 个到期任务（渠道: {:?}）",
            due_jobs.len(),
            self.channels
        );

        for (actor, job) in due_jobs {
            let delivery_key = scheduled_delivery_key(&job, &now);
            if job.channel_target.trim().is_empty() {
                warn!(
                    "⏰ 定时任务缺少 channel_target，跳过投递并记录失败: actor={} job={}",
                    actor.storage_key(),
                    job.id
                );
                let _ = self
                    .storage
                    .record_execution_event(
                        &actor,
                        &job.id,
                        &job.name,
                        "",
                        job.is_heartbeat(),
                        CronJobExecutionInput {
                            execution_status: "execution_failed".to_string(),
                            message_send_status: "target_missing".to_string(),
                            should_deliver: true,
                            delivered: false,
                            response_preview: None,
                            error_message: Some(
                                "定时任务缺少 channel_target，无法确认来源渠道投递目标".to_string(),
                            ),
                            detail: json!({
                                "phase": "target_missing",
                                "delivery_key": delivery_key,
                            }),
                        },
                    )
                    .await;
                self.storage.mark_job_run(&actor, &job.id).await;
                continue;
            }
            let last_delivered_previews = if job.is_heartbeat() {
                load_heartbeat_delivery_history(&self.storage, &actor, &job.id).await
            } else {
                Vec::new()
            };
            let event = SchedulerEvent {
                actor: actor.clone(),
                job_id: job.id.clone(),
                job_name: job.name.clone(),
                task_prompt: job.task_prompt.clone(),
                channel: job.channel.clone(),
                channel_scope: job.channel_scope.clone(),
                channel_target: job.channel_target.clone(),
                delivery_key,
                push: job.push.clone(),
                tags: job.tags.clone(),
                heartbeat: job.is_heartbeat(),
                schedule_hour: job.schedule.hour,
                schedule_minute: job.schedule.minute,
                schedule_repeat: job.schedule.repeat.clone(),
                schedule_date: job.schedule.date.clone(),
                last_delivered_previews,
                bypass_quiet_hours: job.bypass_quiet_hours,
            };

            // 先发送事件，成功后再标记已执行；
            // 若 channel 已关闭，任务不标记，留待下轮（DUE_WINDOW 内）重试，
            // 避免"已标记但未处理"导致任务永久丢失。
            match self.event_tx.send(event).await {
                Ok(_) => {
                    self.storage.mark_job_run(&actor, &job.id).await;
                }
                Err(e) => {
                    warn!(
                        "⏰ 调度事件发送失败，任务将在窗口期内重试: job={} err={e}",
                        job.id
                    );
                }
            }
        }
    }
}

async fn load_heartbeat_delivery_history(
    storage: &CronJobStorage,
    actor: &ActorIdentity,
    job_id: &str,
) -> Vec<(String, String)> {
    async fn delivered_previews(
        storage: &CronJobStorage,
        filter: ExecutionFilter,
        limit: usize,
    ) -> Vec<(String, String)> {
        storage
            .list_recent_executions(&filter)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.delivered && r.response_preview.is_some())
            .take(limit)
            .map(|r| (r.executed_at, r.response_preview.unwrap_or_default()))
            .collect()
    }

    let same_job_filter = ExecutionFilter {
        channel: Some(actor.channel.clone()),
        user_id: Some(actor.user_id.clone()),
        job_id: Some(job_id.to_string()),
        heartbeat_only: Some(true),
        limit: 60,
        ..ExecutionFilter::default()
    };
    let actor_filter = ExecutionFilter {
        channel: Some(actor.channel.clone()),
        user_id: Some(actor.user_id.clone()),
        heartbeat_only: Some(true),
        limit: 40,
        ..ExecutionFilter::default()
    };

    let mut seen = HashSet::new();
    let mut history = Vec::new();
    let same_job = delivered_previews(storage, same_job_filter, 8).await;
    let actor_history = delivered_previews(storage, actor_filter, 8).await;
    for item in same_job.into_iter().chain(actor_history) {
        if seen.insert(item.clone()) {
            history.push(item);
        }
        if history.len() >= 12 {
            break;
        }
    }
    history
}

fn scheduled_delivery_key(
    job: &hone_memory::cron_job::CronJob,
    now: &chrono::DateTime<chrono::FixedOffset>,
) -> String {
    if job.is_heartbeat() {
        let total_minutes = (now.hour() as i32) * 60 + now.minute() as i32;
        let slot_total = (total_minutes / 30) * 30;
        let slot_hour = slot_total / 60;
        let slot_minute = slot_total % 60;
        return format!(
            "{}:{}:{:02}:{:02}:heartbeat",
            job.id,
            now.date_naive().format("%Y-%m-%d"),
            slot_hour,
            slot_minute
        );
    }
    format!(
        "{}:{}:{:02}:{:02}",
        job.id,
        now.date_naive().format("%Y-%m-%d"),
        job.schedule.hour,
        job.schedule.minute
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_memory::cron_job::{CronJob, CronJobData, CronSchedule};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[tokio::test]
    async fn heartbeat_history_includes_actor_cross_job_deliveries() {
        let dir = make_temp_dir("hone_scheduler_cross_job_history");
        let storage = CronJobStorage::new(&dir).await;
        let actor = ActorIdentity::new("feishu", "ou_cross_job", None::<String>).expect("actor");

        storage
            .record_execution_event(
                &actor,
                "job-a",
                "小米破位预警",
                "ou_cross_job",
                true,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "sent".to_string(),
                    should_deliver: true,
                    delivered: true,
                    response_preview: Some("小米跌破 30 港元，当前 29.8 港元".to_string()),
                    error_message: None,
                    detail: serde_json::json!({"delivery_key": "a-1"}),
                },
            )
            .await
            .expect("record job a");
        storage
            .record_execution_event(
                &actor,
                "job-b",
                "持仓重大事件心跳检测",
                "ou_cross_job",
                true,
                CronJobExecutionInput {
                    execution_status: "noop".to_string(),
                    message_send_status: "skipped_noop".to_string(),
                    should_deliver: false,
                    delivered: false,
                    response_preview: Some("未命中".to_string()),
                    error_message: None,
                    detail: serde_json::json!({"delivery_key": "b-1"}),
                },
            )
            .await
            .expect("record job b noop");

        let history = load_heartbeat_delivery_history(&storage, &actor, "job-b").await;
        assert_eq!(history.len(), 1);
        assert!(history[0].1.contains("小米跌破 30 港元"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn heartbeat_history_keeps_same_job_delivery_when_actor_history_is_busy() {
        let dir = make_temp_dir("hone_scheduler_same_job_history");
        let storage = CronJobStorage::new(&dir).await;
        let actor = ActorIdentity::new("feishu", "ou_same_job", None::<String>).expect("actor");

        storage
            .record_execution_event(
                &actor,
                "job-rklb",
                "RKLB异动监控",
                "ou_same_job",
                true,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "sent".to_string(),
                    should_deliver: true,
                    delivered: true,
                    response_preview: Some(
                        "RKLB 6月12日下跌 -10.79%，SpaceX IPO 资金虹吸已提醒".to_string(),
                    ),
                    error_message: None,
                    detail: serde_json::json!({"delivery_key": "rklb-old"}),
                },
            )
            .await
            .expect("record old rklb");

        for idx in 0..14 {
            storage
                .record_execution_event(
                    &actor,
                    &format!("job-other-{idx}"),
                    "其它心跳",
                    "ou_same_job",
                    true,
                    CronJobExecutionInput {
                        execution_status: "completed".to_string(),
                        message_send_status: "sent".to_string(),
                        should_deliver: true,
                        delivered: true,
                        response_preview: Some(format!("其它标的新事件 {idx}")),
                        error_message: None,
                        detail: serde_json::json!({"delivery_key": format!("other-{idx}")}),
                    },
                )
                .await
                .expect("record other");
        }

        let history = load_heartbeat_delivery_history(&storage, &actor, "job-rklb").await;
        assert!(
            history
                .iter()
                .any(|(_, preview)| preview.contains("RKLB 6月12日下跌 -10.79%")),
            "same-job delivery should not be evicted by other heartbeat deliveries: {history:?}"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn scheduler_records_missing_channel_target_without_dispatching() {
        let dir = make_temp_dir("hone_scheduler_missing_target");
        let storage = Arc::new(CronJobStorage::new(&dir).await);
        let actor = ActorIdentity::new("telegram", "user_missing", None::<String>).expect("actor");
        let now = hone_core::local_now();
        let job = CronJob {
            id: "j_missing_target".to_string(),
            name: "missing target".to_string(),
            schedule: CronSchedule {
                hour: now.hour(),
                minute: now.minute(),
                repeat: "daily".to_string(),
                weekday: None,
                date: None,
            },
            task_prompt: "task".to_string(),
            push: json!({"type": "analysis"}),
            enabled: true,
            channel: "telegram".to_string(),
            channel_scope: None,
            channel_target: String::new(),
            tags: Vec::new(),
            created_at: None,
            last_run_at: None,
            bypass_quiet_hours: false,
        };
        storage
            .save_jobs(
                &actor,
                &CronJobData {
                    actor: Some(actor.clone()),
                    user_id: actor.user_id.clone(),
                    jobs: vec![job],
                    pending_updates: Vec::new(),
                },
            )
            .await
            .expect("save invalid legacy job");

        let (tx, mut rx) = mpsc::channel(4);
        let scheduler = HoneScheduler::new(storage.clone(), tx, vec!["telegram".to_string()]);
        scheduler.check_due_jobs().await;

        assert!(rx.try_recv().is_err());
        let records = storage
            .list_recent_executions(&ExecutionFilter {
                job_id: Some("j_missing_target".to_string()),
                limit: 10,
                ..ExecutionFilter::default()
            })
            .await
            .expect("list executions");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].execution_status, "execution_failed");
        assert_eq!(records[0].message_send_status, "target_missing");
        assert!(
            records[0]
                .error_message
                .as_deref()
                .unwrap_or("")
                .contains("channel_target")
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn execution_detail_with_delivery_key_preserves_metadata() {
        let detail = execution_detail_with_delivery_key(
            serde_json::json!({
                "parse_kind": "JsonNoop",
                "heartbeat_model": "model-a",
            }),
            "delivery-123",
        );

        assert_eq!(detail["delivery_key"], "delivery-123");
        assert_eq!(detail["parse_kind"], "JsonNoop");
        assert_eq!(detail["heartbeat_model"], "model-a");
    }

    #[test]
    fn execution_detail_with_delivery_key_wraps_non_object_metadata() {
        let detail = execution_detail_with_delivery_key(Value::Null, "delivery-123");
        assert_eq!(detail["delivery_key"], "delivery-123");
        assert!(detail.get("scheduler").is_none());

        let detail = execution_detail_with_delivery_key(
            serde_json::json!(["metadata", "array"]),
            "delivery-456",
        );
        assert_eq!(detail["delivery_key"], "delivery-456");
        assert_eq!(detail["scheduler"][0], "metadata");
    }

    #[test]
    fn execution_detail_with_delivery_key_overwrites_unusable_key() {
        let detail = execution_detail_with_delivery_key(
            serde_json::json!({
                "delivery_key": null,
                "parse_kind": "JsonNoop",
            }),
            "delivery-789",
        );
        assert_eq!(detail["delivery_key"], "delivery-789");
        assert_eq!(detail["parse_kind"], "JsonNoop");

        let detail = execution_detail_with_delivery_key(
            serde_json::json!({
                "delivery_key": "",
                "parse_kind": "JsonTriggered",
            }),
            "delivery-abc",
        );
        assert_eq!(detail["delivery_key"], "delivery-abc");
        assert_eq!(detail["parse_kind"], "JsonTriggered");
    }

    #[tokio::test]
    async fn scheduler_event_activity_fails_closed_after_actor_scoped_cancellation() {
        let dir = make_temp_dir("hone_scheduler_event_activity");
        let storage = CronJobStorage::new(&dir).await;
        let actor = ActorIdentity::new("feishu", "ou_cancelled", None::<String>).expect("actor");
        let added = storage
            .add_job(
                &actor,
                "daily report",
                Some(9),
                Some(0),
                "daily",
                "task",
                "ou_cancelled",
                None,
                None,
                None,
                true,
                None,
                false,
            )
            .await;
        let job: CronJob = serde_json::from_value(added["job"].clone()).expect("job");
        let event = SchedulerEvent {
            actor: actor.clone(),
            job_id: job.id.clone(),
            job_name: job.name.clone(),
            task_prompt: job.task_prompt.clone(),
            channel: job.channel.clone(),
            channel_scope: job.channel_scope.clone(),
            channel_target: job.channel_target.clone(),
            delivery_key: "cancel-test".to_string(),
            push: job.push.clone(),
            tags: job.tags.clone(),
            heartbeat: false,
            schedule_hour: job.schedule.hour,
            schedule_minute: job.schedule.minute,
            schedule_repeat: job.schedule.repeat.clone(),
            schedule_date: job.schedule.date.clone(),
            last_delivered_previews: Vec::new(),
            bypass_quiet_hours: false,
        };

        assert!(scheduler_event_is_active(&storage, &event).await);
        storage
            .remove_all_jobs(&actor)
            .await
            .expect("cancel all actor jobs");
        assert!(!scheduler_event_is_active(&storage, &event).await);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn dispatched_once_event_remains_active_for_its_claimed_run() {
        let dir = make_temp_dir("hone_scheduler_once_event_activity");
        let storage = CronJobStorage::new(&dir).await;
        let actor = ActorIdentity::new("telegram", "once-user", None::<String>).expect("actor");
        let date = hone_core::local_now().date_naive().format("%F").to_string();
        let added = storage
            .add_job(
                &actor,
                "one-time report",
                Some(9),
                Some(0),
                "once",
                "task",
                "123",
                None,
                Some(date),
                None,
                true,
                None,
                false,
            )
            .await;
        let job: CronJob = serde_json::from_value(added["job"].clone()).expect("job");
        let event = SchedulerEvent {
            actor: actor.clone(),
            job_id: job.id.clone(),
            job_name: job.name.clone(),
            task_prompt: job.task_prompt.clone(),
            channel: job.channel.clone(),
            channel_scope: job.channel_scope.clone(),
            channel_target: job.channel_target.clone(),
            delivery_key: "once-test".to_string(),
            push: job.push.clone(),
            tags: job.tags.clone(),
            heartbeat: false,
            schedule_hour: job.schedule.hour,
            schedule_minute: job.schedule.minute,
            schedule_repeat: job.schedule.repeat.clone(),
            schedule_date: job.schedule.date.clone(),
            last_delivered_previews: Vec::new(),
            bypass_quiet_hours: false,
        };

        storage.mark_job_run(&actor, &job.id).await;
        assert!(scheduler_event_is_active(&storage, &event).await);

        let _ = std::fs::remove_dir_all(dir);
    }
}

#[cfg(test)]
mod unsubscribe_footer_tests {
    use super::with_unsubscribe_footer;
    use hone_core::config::HoneConfig;

    fn config_with_secret(env_name: &str) -> HoneConfig {
        let mut config = HoneConfig::default();
        config.email.unsubscribe_secret_env = env_name.to_string();
        config.agent.hone_cloud.base_url = "https://hone-claw.com".to_string();
        config
    }

    #[test]
    fn a_delivered_push_carries_its_own_way_out() {
        unsafe { std::env::set_var("HONE_TEST_FOOTER_SECRET", "s3cret") };
        let config = config_with_secret("HONE_TEST_FOOTER_SECRET");
        let out = with_unsubscribe_footer("今日复盘。".to_string(), &config, "job-7");

        assert!(out.starts_with("今日复盘。"));
        assert!(out.contains("不想再收到这条推送"));
        assert!(out.contains("https://hone-claw.com/api/public/unsubscribe/job-7."));
        unsafe { std::env::remove_var("HONE_TEST_FOOTER_SECRET") };
    }

    #[test]
    fn no_secret_means_no_link_rather_than_a_broken_one() {
        // Offering a link the product then cannot honour is worse than not
        // offering one.
        let config = config_with_secret("HONE_TEST_FOOTER_ABSENT");
        let out = with_unsubscribe_footer("今日复盘。".to_string(), &config, "job-7");
        assert_eq!(out, "今日复盘。");
    }

    #[test]
    fn a_failing_job_is_exactly_when_someone_wants_out() {
        unsafe { std::env::set_var("HONE_TEST_FOOTER_SECRET2", "s3cret") };
        let config = config_with_secret("HONE_TEST_FOOTER_SECRET2");
        let out = with_unsubscribe_footer("执行失败：上游超时".to_string(), &config, "job-9");
        assert!(out.contains("/api/public/unsubscribe/job-9."));
        unsafe { std::env::remove_var("HONE_TEST_FOOTER_SECRET2") };
    }

    #[test]
    fn trailing_whitespace_does_not_stack_up_blank_lines() {
        unsafe { std::env::set_var("HONE_TEST_FOOTER_SECRET3", "s3cret") };
        let config = config_with_secret("HONE_TEST_FOOTER_SECRET3");
        let out = with_unsubscribe_footer("正文\n\n\n".to_string(), &config, "job-1");
        assert!(out.starts_with("正文\n\n———"));
        unsafe { std::env::remove_var("HONE_TEST_FOOTER_SECRET3") };
    }
}
