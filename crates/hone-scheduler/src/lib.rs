//! Hone Scheduler — 定时任务调度器
//!
//! 使用 tokio interval 实现分钟级 cron 调度。

use chrono::{Datelike, FixedOffset, Timelike, Utc};
use hone_core::ActorIdentity;
use hone_memory::CronJobStorage;
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
    pub push: serde_json::Value,
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
            let event = SchedulerEvent {
                actor: actor.clone(),
                job_id: job.id.clone(),
                job_name: job.name.clone(),
                task_prompt: job.task_prompt.clone(),
                channel: job.channel.clone(),
                channel_scope: job.channel_scope.clone(),
                channel_target: job.channel_target.clone(),
                push: job.push.clone(),
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
