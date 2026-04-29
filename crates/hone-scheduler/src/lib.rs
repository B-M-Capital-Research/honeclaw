//! Hone Scheduler — 定时任务调度器
//!
//! 使用 tokio interval 实现分钟级 cron 调度。

use chrono::{Datelike, FixedOffset, Timelike, Utc};
use hone_core::ActorIdentity;
use hone_memory::{CronJobStorage, cron_job::ExecutionFilter};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

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

/// Ensure cron execution history can finalize the pre-written `running/pending` row.
///
/// `CronJobStorage::record_execution_event` matches terminal records to started
/// records by a top-level `detail.delivery_key`. Scheduler execution metadata is
/// often a domain-specific object and may not carry that key, so channel
/// handlers should wrap every terminal detail through this helper before
/// recording it.
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
    object
        .entry("delivery_key".to_string())
        .or_insert_with(|| Value::String(delivery_key.to_string()));
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
        let beijing_tz = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = Utc::now().with_timezone(&beijing_tz);

        let hour = now.hour() as i32;
        let minute = now.minute() as i32;
        let weekday = now.weekday().num_days_from_monday();

        // 只查询本进程负责的渠道，防止多进程共享存储时跨渠道误标记
        let channel_refs: Vec<&str> = self.channels.iter().map(|s| s.as_str()).collect();
        let due_jobs = self
            .storage
            .get_due_jobs(hour, minute, weekday, &channel_refs);

        if due_jobs.is_empty() {
            return;
        }

        info!(
            "⏰ 发现 {} 个到期任务（渠道: {:?}）",
            due_jobs.len(),
            self.channels
        );

        for (actor, job) in due_jobs {
            let last_delivered_previews = if job.is_heartbeat() {
                load_heartbeat_delivery_history(&self.storage, &actor)
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
                delivery_key: scheduled_delivery_key(&job, &now),
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
                    self.storage.mark_job_run(&actor, &job.id);
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

fn load_heartbeat_delivery_history(
    storage: &CronJobStorage,
    actor: &ActorIdentity,
) -> Vec<(String, String)> {
    let filter = ExecutionFilter {
        channel: Some(actor.channel.clone()),
        user_id: Some(actor.user_id.clone()),
        heartbeat_only: Some(true),
        limit: 20,
        ..ExecutionFilter::default()
    };
    storage
        .list_recent_executions(&filter)
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.delivered && r.response_preview.is_some())
        .take(8)
        .map(|r| (r.executed_at, r.response_preview.unwrap_or_default()))
        .collect()
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
    use hone_memory::cron_job::CronJobExecutionInput;
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

    #[test]
    fn heartbeat_history_includes_actor_cross_job_deliveries() {
        let dir = make_temp_dir("hone_scheduler_cross_job_history");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
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
            .expect("record job b noop");

        let history = load_heartbeat_delivery_history(&storage, &actor);
        assert_eq!(history.len(), 1);
        assert!(history[0].1.contains("小米跌破 30 港元"));

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
}
