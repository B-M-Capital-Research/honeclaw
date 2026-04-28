//! Cron job 纯数据类型、常量与错误 helper。

use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 每个 actor 同时启用中的最大定时任务数
pub const MAX_ENABLED_JOBS_PER_ACTOR: usize = 12;

pub fn cron_enabled_limit_error() -> String {
    format!(
        "已达到最大启用定时任务数量（{}个），请先停用或删除不需要的任务",
        MAX_ENABLED_JOBS_PER_ACTOR
    )
}

pub fn is_cron_enabled_limit_error(message: &str) -> bool {
    message.contains(&cron_enabled_limit_error())
}

#[derive(Debug, Clone, Default)]
pub struct CronJobUpdate {
    pub name: Option<String>,
    pub schedule: Option<CronSchedule>,
    pub task_prompt: Option<String>,
    pub push: Option<Value>,
    pub enabled: Option<bool>,
    pub channel_target: Option<String>,
    pub tags: Option<Vec<String>>,
    pub bypass_quiet_hours: Option<bool>,
}

/// Cron 任务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<ActorIdentity>,
    #[serde(default)]
    pub user_id: String,
    pub jobs: Vec<CronJob>,
    #[serde(default)]
    pub pending_updates: Vec<PendingUpdate>,
}

/// 单个定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: CronSchedule,
    pub task_prompt: String,
    #[serde(default)]
    pub push: Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_scope: Option<String>,
    #[serde(default)]
    pub channel_target: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_run_at: Option<String>,
    /// 是否绕过 quiet_hours 静音。默认 false（cron 任务遵守用户的勿扰时段）；
    /// 用户可对个别需要严守时间的任务（如 06:55 盘前复盘）显式打开。
    #[serde(default)]
    pub bypass_quiet_hours: bool,
}

fn default_true() -> bool {
    true
}

/// 调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    pub hour: u32,
    pub minute: u32,
    pub repeat: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekday: Option<u32>,
    /// Absolute Beijing date for one-shot jobs, formatted as YYYY-MM-DD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// 待确认更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUpdate {
    pub token: String,
    pub job_id: String,
    pub updates: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobExecutionRecord {
    pub run_id: i64,
    pub job_id: String,
    pub job_name: String,
    pub channel: String,
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_scope: Option<String>,
    pub channel_target: String,
    pub heartbeat: bool,
    pub executed_at: String,
    pub execution_status: String,
    pub message_send_status: String,
    pub should_deliver: bool,
    pub delivered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default)]
    pub detail: Value,
}

#[derive(Debug, Clone, Default)]
pub struct CronJobExecutionInput {
    pub execution_status: String,
    pub message_send_status: String,
    pub should_deliver: bool,
    pub delivered: bool,
    pub response_preview: Option<String>,
    pub error_message: Option<String>,
    pub detail: Value,
}
