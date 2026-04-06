//! 定时任务存储 — JSON 文件 + SQLite 执行记录
//!
//! 管理按 actor（channel + user_id + channel_scope）隔离的定时任务持久化存储。

use chrono::{Datelike, NaiveDate, Timelike};
use hone_core::ActorIdentity;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::warn;
use uuid::Uuid;

/// 北京时间偏移（+8h）
const BEIJING_OFFSET: i32 = 8 * 3600;
/// 每个 actor 同时启用中的最大定时任务数
pub const MAX_ENABLED_JOBS_PER_ACTOR: usize = 12;
/// 容错窗口（分钟）— 向过去看 5 分钟，覆盖 LLM 处理时间导致的时间窗口错过
const DUE_WINDOW_MINUTES: i32 = 5;

pub fn cron_enabled_limit_error() -> String {
    format!(
        "已达到最大启用定时任务数量（{}个），请先停用或删除不需要的任务",
        MAX_ENABLED_JOBS_PER_ACTOR
    )
}

pub fn is_cron_enabled_limit_error(message: &str) -> bool {
    message.contains(&cron_enabled_limit_error())
}

/// 定时任务存储管理器
pub struct CronJobStorage {
    data_dir: PathBuf,
    sqlite_path: Option<PathBuf>,
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

impl CronJobStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).ok();
        Self {
            data_dir,
            sqlite_path: None,
        }
    }

    pub fn with_sqlite(data_dir: impl AsRef<Path>, sqlite_path: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).ok();
        let storage = Self {
            data_dir,
            sqlite_path: Some(sqlite_path.as_ref().to_path_buf()),
        };
        if let Err(err) = storage.init_execution_schema() {
            warn!("failed to initialize cron execution sqlite schema: {err}");
        }
        storage
    }

    fn get_actor_file(&self, actor: &ActorIdentity) -> PathBuf {
        self.data_dir
            .join(format!("cron_jobs_{}.json", actor.storage_key()))
    }

    pub fn list_all_jobs(&self) -> Vec<(ActorIdentity, CronJob)> {
        let mut jobs = Vec::new();
        let entries = match std::fs::read_dir(&self.data_dir) {
            Ok(entries) => entries,
            Err(_) => return jobs,
        };

        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.starts_with("cron_jobs_") || !fname.ends_with(".json") {
                continue;
            }

            let path = entry.path();
            let content = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue,
            };
            let data = match serde_json::from_str::<CronJobData>(&content) {
                Ok(data) => data,
                Err(_) => continue,
            };

            let Some(actor) = actor_from_cron_data(&data) else {
                continue;
            };
            if !cron_file_matches_actor(&path, &actor) {
                warn!(
                    "skipping mismatched cron file path={} actor={}",
                    path.display(),
                    actor.storage_key()
                );
                continue;
            }

            jobs.extend(data.jobs.into_iter().map(|job| (actor.clone(), job)));
        }

        jobs
    }

    /// 加载 actor 的定时任务数据
    pub fn load_jobs(&self, actor: &ActorIdentity) -> CronJobData {
        let path = self.get_actor_file(actor);
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(data) = serde_json::from_str::<CronJobData>(&content)
        {
            return data;
        }
        CronJobData {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            jobs: Vec::new(),
            pending_updates: Vec::new(),
        }
    }

    /// 保存 actor 的定时任务数据
    pub fn save_jobs(
        &self,
        actor: &ActorIdentity,
        data: &CronJobData,
    ) -> hone_core::HoneResult<()> {
        let path = self.get_actor_file(actor);
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| hone_core::HoneError::Storage(e.to_string()))?;
        std::fs::write(&path, content).map_err(|e| hone_core::HoneError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn get_job(
        &self,
        job_id: &str,
        actor: Option<&ActorIdentity>,
    ) -> Option<(ActorIdentity, CronJob)> {
        if let Some(actor) = actor {
            let data = self.load_jobs(actor);
            return data
                .jobs
                .into_iter()
                .find(|job| job.id == job_id)
                .map(|job| (actor.clone(), job));
        }

        self.list_all_jobs()
            .into_iter()
            .find(|(_, job)| job.id == job_id)
    }

    /// 添加定时任务
    pub fn add_job(
        &self,
        actor: &ActorIdentity,
        name: &str,
        hour: Option<u32>,
        minute: Option<u32>,
        repeat: &str,
        task_prompt: &str,
        channel_target: &str,
        weekday: Option<u32>,
        push: Option<Value>,
        enabled: bool,
        tags: Option<Vec<String>>,
        bypass_limits: bool,
    ) -> Value {
        let mut data = self.load_jobs(actor);

        let enabled_count = data.jobs.iter().filter(|j| j.enabled).count();
        if enabled && !bypass_limits && enabled_count >= MAX_ENABLED_JOBS_PER_ACTOR {
            return serde_json::json!({
                "success": false,
                "error": cron_enabled_limit_error()
            });
        }

        let tags = normalized_tags(tags.unwrap_or_default(), repeat);
        let is_heartbeat = is_heartbeat_repeat_or_tags(repeat, &tags);
        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);

        if let Err(error) = validate_schedule(
            if is_heartbeat { None } else { Some(hour) },
            if is_heartbeat { None } else { Some(minute) },
            repeat,
            weekday,
        ) {
            return serde_json::json!({"success": false, "error": error});
        }

        let job_id = format!("j_{}", &Uuid::new_v4().to_string()[..8]);
        let now = hone_core::beijing_now_rfc3339();

        let job = CronJob {
            id: job_id,
            name: name.to_string(),
            schedule: CronSchedule {
                hour,
                minute,
                repeat: repeat.to_string(),
                weekday,
            },
            task_prompt: task_prompt.to_string(),
            push: push.unwrap_or_else(|| serde_json::json!({"type": "analysis"})),
            enabled,
            channel: actor.channel.clone(),
            channel_scope: actor.channel_scope.clone(),
            channel_target: if channel_target.is_empty() {
                actor.user_id.clone()
            } else {
                channel_target.to_string()
            },
            tags,
            created_at: Some(now),
            last_run_at: None,
        };

        let job_value = serde_json::to_value(&job).unwrap_or_default();
        data.jobs.push(job);
        let _ = self.save_jobs(actor, &data);

        serde_json::json!({"success": true, "job": job_value})
    }

    /// 删除定时任务
    pub fn remove_job(&self, actor: &ActorIdentity, job_id: &str) -> Value {
        let mut data = self.load_jobs(actor);
        let original_len = data.jobs.len();
        data.jobs.retain(|j| j.id != job_id);
        if data.jobs.len() == original_len {
            return serde_json::json!({"success": false, "error": format!("未找到任务 {job_id}")});
        }
        let _ = self.save_jobs(actor, &data);
        serde_json::json!({"success": true, "removed_job_id": job_id})
    }

    /// 列出 actor 的所有定时任务
    pub fn list_jobs(&self, actor: &ActorIdentity) -> Vec<CronJob> {
        self.load_jobs(actor).jobs
    }

    pub fn update_job(
        &self,
        job_id: &str,
        actor: Option<&ActorIdentity>,
        updates: CronJobUpdate,
        bypass_limits: bool,
    ) -> hone_core::HoneResult<Option<(ActorIdentity, CronJob)>> {
        self.mutate_job(job_id, actor, bypass_limits, |job| {
            if let Some(name) = updates.name.clone() {
                job.name = name;
            }
            if let Some(schedule) = updates.schedule.clone() {
                validate_schedule(
                    Some(schedule.hour),
                    Some(schedule.minute),
                    &schedule.repeat,
                    schedule.weekday,
                )
                .map_err(hone_core::HoneError::Tool)?;
                job.schedule = schedule;
                job.tags = normalized_tags(job.tags.clone(), &job.schedule.repeat);
            }
            if let Some(task_prompt) = updates.task_prompt.clone() {
                job.task_prompt = task_prompt;
            }
            if let Some(push) = updates.push.clone() {
                job.push = push;
            }
            if let Some(enabled) = updates.enabled {
                job.enabled = enabled;
            }
            if let Some(channel_target) = updates.channel_target.clone() {
                job.channel_target = channel_target;
            }
            if let Some(tags) = updates.tags.clone() {
                job.tags = normalized_tags(tags, &job.schedule.repeat);
            }
            Ok(())
        })
    }

    pub fn toggle_job(
        &self,
        job_id: &str,
        actor: Option<&ActorIdentity>,
        bypass_limits: bool,
    ) -> hone_core::HoneResult<Option<(ActorIdentity, CronJob)>> {
        self.mutate_job(job_id, actor, bypass_limits, |job| {
            job.enabled = !job.enabled;
            Ok(())
        })
    }

    pub fn delete_job(
        &self,
        job_id: &str,
        actor: Option<&ActorIdentity>,
    ) -> hone_core::HoneResult<Option<(ActorIdentity, CronJob)>> {
        if let Some(actor) = actor {
            return self.delete_job_for_actor(job_id, actor);
        }

        let mut actors = self
            .list_all_jobs()
            .into_iter()
            .map(|(actor, _)| actor)
            .collect::<Vec<_>>();
        actors.sort_by(|left, right| left.storage_key().cmp(&right.storage_key()));
        actors.dedup_by(|left, right| left.storage_key() == right.storage_key());

        for actor in actors {
            if let Some(removed) = self.delete_job_for_actor(job_id, &actor)? {
                return Ok(Some(removed));
            }
        }

        Ok(None)
    }

    /// 标记任务已执行
    pub fn mark_job_run(&self, actor: &ActorIdentity, job_id: &str) {
        let mut data = self.load_jobs(actor);
        let now = hone_core::beijing_now_rfc3339();
        for job in &mut data.jobs {
            if job.id == job_id {
                job.last_run_at = Some(now.clone());
                if job.schedule.repeat == "once" {
                    job.enabled = false;
                }
                break;
            }
        }
        let _ = self.save_jobs(actor, &data);
    }

    /// 获取应触发的所有任务
    ///
    /// `channels`：若非空，只返回 `job.channel` 在列表中的任务；
    /// 若为空切片，则返回所有任务（向后兼容/测试用途）。
    pub fn get_due_jobs(
        &self,
        current_hour: i32,
        current_minute: i32,
        current_weekday: u32,
        channels: &[&str],
    ) -> Vec<(ActorIdentity, CronJob)> {
        let mut due = Vec::new();
        let mut seen_due_keys = HashSet::new();
        let now = hone_core::beijing_now();
        let current_day = now.date_naive();
        let current_total = current_hour * 60 + current_minute;

        let entries = match std::fs::read_dir(&self.data_dir) {
            Ok(e) => e,
            Err(_) => return due,
        };

        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.starts_with("cron_jobs_") || !fname.ends_with(".json") {
                continue;
            }

            let path = entry.path();
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let data: CronJobData = match serde_json::from_str(&content) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let Some(actor) = actor_from_cron_data(&data) else {
                continue;
            };
            if !cron_file_matches_actor(&path, &actor) {
                warn!(
                    "skipping mismatched cron file path={} actor={}",
                    path.display(),
                    actor.storage_key()
                );
                continue;
            }

            for job in &data.jobs {
                if !job.enabled {
                    continue;
                }

                // Channel 过滤：每个 scheduler 只处理属于自己渠道的任务，
                // 避免多进程共享存储时跨渠道误标记（cross-process mark race）。
                if !channels.is_empty() && !channels.contains(&job.channel.as_str()) {
                    continue;
                }

                let job_total = (job.schedule.hour as i32) * 60 + (job.schedule.minute as i32);
                let is_heartbeat = job.is_heartbeat();
                if is_heartbeat {
                    let slot_minute = (current_total / 30) * 30;
                    if !(slot_minute <= current_total
                        && current_total <= slot_minute + DUE_WINDOW_MINUTES)
                    {
                        continue;
                    }
                } else {
                    let due_in_window = current_total - DUE_WINDOW_MINUTES <= job_total
                        && job_total <= current_total;
                    let due_by_catch_up =
                        current_total > job_total && job_existed_before_slot(job, current_day);
                    if !(due_in_window || due_by_catch_up) {
                        continue;
                    }
                }

                let repeat_kind = normalized_repeat(&job.schedule.repeat, &job.tags);
                match repeat_kind {
                    "weekly" => {
                        if job.schedule.weekday != Some(current_weekday) {
                            continue;
                        }
                    }
                    "workday" => {
                        if !is_workday(current_day) {
                            continue;
                        }
                    }
                    "trading_day" => {
                        if !is_trading_day(current_day) {
                            continue;
                        }
                    }
                    "holiday" => {
                        if !is_holiday(current_day) {
                            continue;
                        }
                    }
                    _ => {}
                }

                if let Some(ref last_run) = job.last_run_at
                    && let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(last_run)
                {
                    let already_ran = match repeat_kind {
                        "heartbeat" => {
                            let current_slot_start_minute = (current_total / 30) * 30;
                            let current_slot_hour = current_slot_start_minute / 60;
                            let current_slot_minute = current_slot_start_minute % 60;
                            last_dt.date_naive() == now.date_naive()
                                && last_dt.hour() as i32 == current_slot_hour
                                && (last_dt.minute() as i32 / 30) == (current_slot_minute / 30)
                        }
                        "weekly" => {
                            last_dt.iso_week() == now.iso_week() && last_dt.year() == now.year()
                        }
                        "once" => true,
                        _ => last_dt.date_naive() == now.date_naive(),
                    };
                    if already_ran {
                        continue;
                    }
                }

                let dedup_key = format!("{}:{}:{}", job.channel, job.id, job.channel_target);
                if !seen_due_keys.insert(dedup_key) {
                    warn!(
                        "skipping duplicate due cron job actor={} job_id={} target={}",
                        actor.storage_key(),
                        job.id,
                        job.channel_target
                    );
                    continue;
                }

                due.push((actor.clone(), job.clone()));
            }
        }

        due
    }

    fn mutate_job<F>(
        &self,
        job_id: &str,
        actor: Option<&ActorIdentity>,
        bypass_limits: bool,
        mut mutator: F,
    ) -> hone_core::HoneResult<Option<(ActorIdentity, CronJob)>>
    where
        F: FnMut(&mut CronJob) -> hone_core::HoneResult<()>,
    {
        if let Some(actor) = actor {
            return self.mutate_job_for_actor(job_id, actor, bypass_limits, &mut mutator);
        }

        let mut actors = self
            .list_all_jobs()
            .into_iter()
            .map(|(actor, _)| actor)
            .collect::<Vec<_>>();
        actors.sort_by(|left, right| left.storage_key().cmp(&right.storage_key()));
        actors.dedup_by(|left, right| left.storage_key() == right.storage_key());

        for actor in actors {
            if let Some(updated) =
                self.mutate_job_for_actor(job_id, &actor, bypass_limits, &mut mutator)?
            {
                return Ok(Some(updated));
            }
        }

        Ok(None)
    }

    fn mutate_job_for_actor<F>(
        &self,
        job_id: &str,
        actor: &ActorIdentity,
        bypass_limits: bool,
        mutator: &mut F,
    ) -> hone_core::HoneResult<Option<(ActorIdentity, CronJob)>>
    where
        F: FnMut(&mut CronJob) -> hone_core::HoneResult<()>,
    {
        let mut data = self.load_jobs(actor);
        let Some(index) = data.jobs.iter().position(|job| job.id == job_id) else {
            return Ok(None);
        };
        let (is_enabling, updated) = {
            let job = &mut data.jobs[index];
            let was_enabled = job.enabled;
            mutator(job)?;
            (!was_enabled && job.enabled, job.clone())
        };
        if is_enabling
            && !bypass_limits
            && data.jobs.iter().filter(|job| job.enabled).count() > MAX_ENABLED_JOBS_PER_ACTOR
        {
            return Err(hone_core::HoneError::Tool(cron_enabled_limit_error()));
        }
        self.save_jobs(actor, &data)?;
        Ok(Some((actor.clone(), updated)))
    }

    fn delete_job_for_actor(
        &self,
        job_id: &str,
        actor: &ActorIdentity,
    ) -> hone_core::HoneResult<Option<(ActorIdentity, CronJob)>> {
        let mut data = self.load_jobs(actor);
        let Some(index) = data.jobs.iter().position(|job| job.id == job_id) else {
            return Ok(None);
        };
        let removed = data.jobs.remove(index);
        self.save_jobs(actor, &data)?;
        Ok(Some((actor.clone(), removed)))
    }

    pub fn record_execution_event(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
        job_name: &str,
        channel_target: &str,
        heartbeat: bool,
        input: CronJobExecutionInput,
    ) -> hone_core::HoneResult<()> {
        let Some(conn) = self.open_execution_conn()? else {
            return Ok(());
        };
        let executed_at = hone_core::beijing_now_rfc3339();
        let response_preview = input
            .response_preview
            .as_deref()
            .map(|text| truncate_chars(text, 500));
        let error_message = input
            .error_message
            .as_deref()
            .map(|text| truncate_chars(text, 500));
        let detail_json = serde_json::to_string(&input.detail)
            .map_err(|err| hone_core::HoneError::Serialization(err.to_string()))?;
        conn.execute(
            "
            INSERT INTO cron_job_runs (
                job_id, job_name,
                actor_channel, actor_user_id, actor_channel_scope,
                channel_target, heartbeat,
                executed_at, execution_status, message_send_status,
                should_deliver, delivered, response_preview, error_message, detail_json
            ) VALUES (
                ?1, ?2,
                ?3, ?4, ?5,
                ?6, ?7,
                ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15
            )
            ",
            params![
                job_id,
                job_name,
                actor.channel,
                actor.user_id,
                actor.channel_scope,
                channel_target,
                if heartbeat { 1 } else { 0 },
                executed_at,
                input.execution_status,
                input.message_send_status,
                if input.should_deliver { 1 } else { 0 },
                if input.delivered { 1 } else { 0 },
                response_preview,
                error_message,
                detail_json,
            ],
        )
        .map_err(sqlite_err)?;
        Ok(())
    }

    pub fn list_execution_records(
        &self,
        job_id: &str,
        limit: usize,
    ) -> hone_core::HoneResult<Vec<CronJobExecutionRecord>> {
        let Some(conn) = self.open_execution_conn()? else {
            return Ok(Vec::new());
        };
        let mut stmt = conn
            .prepare(
                "
                SELECT
                    run_id, job_id, job_name,
                    actor_channel, actor_user_id, actor_channel_scope,
                    channel_target, heartbeat,
                    executed_at, execution_status, message_send_status,
                    should_deliver, delivered, response_preview, error_message, detail_json
                FROM cron_job_runs
                WHERE job_id = ?1
                ORDER BY executed_at DESC, run_id DESC
                LIMIT ?2
                ",
            )
            .map_err(sqlite_err)?;
        let rows = stmt
            .query_map(params![job_id, limit as i64], |row| {
                let detail_raw: String = row.get(15)?;
                let detail = serde_json::from_str(&detail_raw).unwrap_or(Value::Null);
                Ok(CronJobExecutionRecord {
                    run_id: row.get(0)?,
                    job_id: row.get(1)?,
                    job_name: row.get(2)?,
                    channel: row.get(3)?,
                    user_id: row.get(4)?,
                    channel_scope: row.get(5)?,
                    channel_target: row.get(6)?,
                    heartbeat: row.get::<_, i64>(7)? != 0,
                    executed_at: row.get(8)?,
                    execution_status: row.get(9)?,
                    message_send_status: row.get(10)?,
                    should_deliver: row.get::<_, i64>(11)? != 0,
                    delivered: row.get::<_, i64>(12)? != 0,
                    response_preview: row.get(13)?,
                    error_message: row.get(14)?,
                    detail,
                })
            })
            .map_err(sqlite_err)?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(sqlite_err)?);
        }
        Ok(out)
    }

    fn open_execution_conn(&self) -> hone_core::HoneResult<Option<Connection>> {
        let Some(path) = &self.sqlite_path else {
            return Ok(None);
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| hone_core::HoneError::Storage(err.to_string()))?;
        }
        let conn = Connection::open(path).map_err(sqlite_err)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(sqlite_err)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(sqlite_err)?;
        conn.pragma_update(None, "busy_timeout", 5000)
            .map_err(sqlite_err)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(sqlite_err)?;
        self.init_execution_schema_with_conn(&conn)?;
        Ok(Some(conn))
    }

    fn init_execution_schema(&self) -> hone_core::HoneResult<()> {
        let Some(conn) = self.open_execution_conn()? else {
            return Ok(());
        };
        self.init_execution_schema_with_conn(&conn)
    }

    fn init_execution_schema_with_conn(&self, conn: &Connection) -> hone_core::HoneResult<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS cron_job_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                job_name TEXT NOT NULL,
                actor_channel TEXT NOT NULL,
                actor_user_id TEXT NOT NULL,
                actor_channel_scope TEXT,
                channel_target TEXT NOT NULL,
                heartbeat INTEGER NOT NULL DEFAULT 0,
                executed_at TEXT NOT NULL,
                execution_status TEXT NOT NULL,
                message_send_status TEXT NOT NULL,
                should_deliver INTEGER NOT NULL DEFAULT 0,
                delivered INTEGER NOT NULL DEFAULT 0,
                response_preview TEXT,
                error_message TEXT,
                detail_json TEXT NOT NULL DEFAULT '{}'
            );

            CREATE INDEX IF NOT EXISTS idx_cron_job_runs_job_time
                ON cron_job_runs(job_id, executed_at DESC, run_id DESC);
            CREATE INDEX IF NOT EXISTS idx_cron_job_runs_actor_time
                ON cron_job_runs(actor_channel, actor_user_id, executed_at DESC);
            ",
        )
        .map_err(sqlite_err)?;
        Ok(())
    }
}

impl CronJob {
    pub fn is_heartbeat(&self) -> bool {
        is_heartbeat_repeat_or_tags(&self.schedule.repeat, &self.tags)
    }
}

fn actor_from_cron_data(data: &CronJobData) -> Option<ActorIdentity> {
    match data.actor.clone() {
        Some(actor) => Some(actor),
        None => {
            if data.user_id.is_empty() {
                return None;
            }
            let channel = data
                .jobs
                .first()
                .map(|j| j.channel.clone())
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| "imessage".to_string());
            let scope = data.jobs.first().and_then(|j| j.channel_scope.clone());
            ActorIdentity::new(channel, data.user_id.clone(), scope).ok()
        }
    }
}

fn cron_file_matches_actor(path: &Path, actor: &ActorIdentity) -> bool {
    cron_filename_storage_key(path).is_none_or(|key| key == actor.storage_key())
}

fn cron_filename_storage_key(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    Some(
        file_name
            .strip_prefix("cron_jobs_")?
            .strip_suffix(".json")?
            .to_string(),
    )
}

fn validate_schedule(
    hour: Option<u32>,
    minute: Option<u32>,
    repeat: &str,
    weekday: Option<u32>,
) -> Result<(), String> {
    let normalized_repeat = normalized_repeat(repeat, &[]);
    if normalized_repeat != "heartbeat" {
        let Some(hour) = hour else {
            return Err("缺少 hour".to_string());
        };
        let Some(minute) = minute else {
            return Err("缺少 minute".to_string());
        };
        if hour > 23 {
            return Err(format!("小时须在 0-23 之间，收到 {hour}"));
        }
        if minute > 59 {
            return Err(format!("分钟须在 0-59 之间，收到 {minute}"));
        }
    }

    let valid_repeats = [
        "daily",
        "weekly",
        "once",
        "workday",
        "trading_day",
        "holiday",
        "heartbeat",
    ];
    if !valid_repeats.contains(&normalized_repeat) {
        return Err(format!(
            "repeat 须为 daily/weekly/once/workday/trading_day/holiday/heartbeat，收到 {repeat}"
        ));
    }
    if normalized_repeat == "weekly" && (weekday.is_none() || weekday.unwrap_or(7) > 6) {
        return Err("weekly 类型须指定 weekday (0-6)".to_string());
    }

    Ok(())
}

fn normalized_tags(tags: Vec<String>, repeat: &str) -> Vec<String> {
    let mut out = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_ascii_lowercase();
        if !tag.is_empty() && !out.contains(&tag) {
            out.push(tag);
        }
    }
    if repeat.trim().eq_ignore_ascii_case("heartbeat") && !out.iter().any(|t| t == "heartbeat") {
        out.push("heartbeat".to_string());
    }
    out
}

fn is_heartbeat_repeat_or_tags(repeat: &str, tags: &[String]) -> bool {
    repeat.trim().eq_ignore_ascii_case("heartbeat")
        || tags.iter().any(|tag| tag.eq_ignore_ascii_case("heartbeat"))
}

fn normalized_repeat<'a>(repeat: &'a str, tags: &[String]) -> &'a str {
    if is_heartbeat_repeat_or_tags(repeat, tags) {
        "heartbeat"
    } else {
        repeat
    }
}

fn is_workday(day: NaiveDate) -> bool {
    day.weekday().num_days_from_monday() < 5
}

fn is_market_holiday(day: NaiveDate) -> bool {
    us_market_holidays(day.year()).contains(&day)
}

fn is_trading_day(day: NaiveDate) -> bool {
    is_workday(day) && !is_market_holiday(day)
}

fn is_holiday(day: NaiveDate) -> bool {
    !is_workday(day) || is_market_holiday(day)
}

fn beijing_slot_time(
    day: NaiveDate,
    hour: u32,
    minute: u32,
) -> chrono::DateTime<chrono::FixedOffset> {
    day.and_hms_opt(hour, minute, 0)
        .expect("valid cron slot time")
        .and_local_timezone(chrono::FixedOffset::east_opt(BEIJING_OFFSET).expect("offset"))
        .single()
        .expect("fixed offset slot")
}

fn job_existed_before_slot(job: &CronJob, day: NaiveDate) -> bool {
    let Some(created_at) = job.created_at.as_deref() else {
        return true;
    };
    let Ok(created_dt) = chrono::DateTime::parse_from_rfc3339(created_at) else {
        return true;
    };
    created_dt <= beijing_slot_time(day, job.schedule.hour, job.schedule.minute)
}

fn observed_holiday(base: NaiveDate) -> NaiveDate {
    match base.weekday().num_days_from_monday() {
        5 => base - chrono::Duration::days(1), // Saturday → Friday
        6 => base + chrono::Duration::days(1), // Sunday → Monday
        _ => base,
    }
}

fn nth_weekday(year: i32, month: u32, weekday: u32, n: u32) -> NaiveDate {
    let mut current = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    while current.weekday().num_days_from_monday() != weekday {
        current += chrono::Duration::days(1);
    }
    current + chrono::Duration::days(((n - 1) * 7) as i64)
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

fn sqlite_err(err: rusqlite::Error) -> hone_core::HoneError {
    hone_core::HoneError::Config(format!("Cron 执行记录 SQLite 操作失败: {err}"))
}

fn last_weekday(year: i32, month: u32, weekday: u32) -> NaiveDate {
    let mut current = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    while current.weekday().num_days_from_monday() != weekday {
        current -= chrono::Duration::days(1);
    }
    current
}

fn easter_date(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap()
}

fn us_market_holidays(year: i32) -> Vec<NaiveDate> {
    vec![
        observed_holiday(NaiveDate::from_ymd_opt(year, 1, 1).unwrap()), // New Year
        nth_weekday(year, 1, 0, 3),                                     // MLK Day
        nth_weekday(year, 2, 0, 3),                                     // Presidents Day
        easter_date(year) - chrono::Duration::days(2),                  // Good Friday
        last_weekday(year, 5, 0),                                       // Memorial Day
        observed_holiday(NaiveDate::from_ymd_opt(year, 6, 19).unwrap()), // Juneteenth
        observed_holiday(NaiveDate::from_ymd_opt(year, 7, 4).unwrap()), // Independence Day
        nth_weekday(year, 9, 0, 1),                                     // Labor Day
        nth_weekday(year, 11, 3, 4),                                    // Thanksgiving
        observed_holiday(NaiveDate::from_ymd_opt(year, 12, 25).unwrap()), // Christmas
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;
    use chrono::Timelike;
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

    fn actor(channel: &str, user_id: &str, channel_scope: Option<&str>) -> ActorIdentity {
        ActorIdentity::new(channel, user_id, channel_scope).expect("actor")
    }

    fn add_enabled_job(storage: &CronJobStorage, actor: &ActorIdentity, name: &str) -> Value {
        storage.add_job(
            actor,
            name,
            Some(9),
            Some(0),
            "daily",
            "task",
            &actor.user_id,
            None,
            None,
            true,
            None,
            false,
        )
    }

    #[test]
    fn add_job_validates_params() {
        let dir = make_temp_dir("hone_cron_storage_validate");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("imessage", "u1", None);

        let bad_hour = storage.add_job(
            &actor,
            "bad hour",
            Some(24),
            Some(0),
            "daily",
            "task",
            "u1",
            None,
            None,
            true,
            None,
            false,
        );
        assert_eq!(bad_hour["success"], false);

        let bad_weekly = storage.add_job(
            &actor,
            "bad weekly",
            Some(9),
            Some(0),
            "weekly",
            "task",
            "u1",
            None,
            None,
            true,
            None,
            false,
        );
        assert_eq!(bad_weekly["success"], false);
    }

    #[test]
    fn due_job_and_mark_run_prevents_immediate_duplicate() {
        let dir = make_temp_dir("hone_cron_storage_due");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("imessage", "u1", None);

        let now_bj = chrono::Utc::now()
            .with_timezone(&FixedOffset::east_opt(BEIJING_OFFSET).expect("offset"));
        let hour = now_bj.hour() as u32;
        let minute = now_bj.minute() as u32;

        let add = storage.add_job(
            &actor,
            "daily report",
            Some(hour),
            Some(minute),
            "daily",
            "send report",
            "u1",
            None,
            None,
            true,
            None,
            false,
        );
        assert_eq!(add["success"], true);
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        let due_first = storage.get_due_jobs(
            hour as i32,
            minute as i32,
            now_bj.weekday().num_days_from_monday(),
            &["imessage"],
        );
        assert_eq!(due_first.len(), 1);
        assert_eq!(due_first[0].0, actor);
        assert_eq!(due_first[0].1.id, job_id);

        storage.mark_job_run(&due_first[0].0, &job_id);
        let due_second = storage.get_due_jobs(
            hour as i32,
            minute as i32,
            now_bj.weekday().num_days_from_monday(),
            &["imessage"],
        );
        assert!(due_second.is_empty());
    }

    #[test]
    fn due_jobs_skip_mismatched_cron_file_actor() {
        let dir = make_temp_dir("hone_cron_storage_mismatch");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_real", None);

        let now_bj = chrono::Utc::now()
            .with_timezone(&FixedOffset::east_opt(BEIJING_OFFSET).expect("offset"));
        let hour = now_bj.hour() as u32;
        let minute = now_bj.minute() as u32;

        let data = CronJobData {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            jobs: vec![CronJob {
                id: "j_dup".to_string(),
                name: "dup".to_string(),
                schedule: CronSchedule {
                    hour,
                    minute,
                    repeat: "daily".to_string(),
                    weekday: None,
                },
                task_prompt: "task".to_string(),
                push: serde_json::json!({"type": "analysis"}),
                enabled: true,
                channel: "feishu".to_string(),
                channel_scope: None,
                channel_target: "+86123".to_string(),
                tags: Vec::new(),
                created_at: None,
                last_run_at: None,
            }],
            pending_updates: Vec::new(),
        };

        let bad_path = dir.join("cron_jobs_feishu__direct__ou_wrong.json");
        std::fs::write(
            &bad_path,
            serde_json::to_string_pretty(&data).expect("encode"),
        )
        .expect("write");

        let due = storage.get_due_jobs(
            hour as i32,
            minute as i32,
            now_bj.weekday().num_days_from_monday(),
            &["feishu"],
        );
        assert!(due.is_empty());
    }

    #[test]
    fn due_jobs_dedup_same_job_id_across_files() {
        let dir = make_temp_dir("hone_cron_storage_dup_files");
        let storage = CronJobStorage::new(&dir);
        let primary_actor = actor("feishu", "ou_real", None);
        let other_actor = actor("feishu", "ou_other", None);

        let now_bj = chrono::Utc::now()
            .with_timezone(&FixedOffset::east_opt(BEIJING_OFFSET).expect("offset"));
        let hour = now_bj.hour() as u32;
        let minute = now_bj.minute() as u32;

        let add = storage.add_job(
            &primary_actor,
            "daily report",
            Some(hour),
            Some(minute),
            "daily",
            "send report",
            "+86123",
            None,
            None,
            true,
            None,
            false,
        );
        let job: CronJob = serde_json::from_value(add["job"].clone()).expect("job");
        let duplicate_data = CronJobData {
            actor: Some(other_actor.clone()),
            user_id: other_actor.user_id.clone(),
            jobs: vec![CronJob {
                channel_target: "+86123".to_string(),
                ..job
            }],
            pending_updates: Vec::new(),
        };
        let duplicate_path = dir.join(format!("cron_jobs_{}.json", other_actor.storage_key()));
        std::fs::write(
            &duplicate_path,
            serde_json::to_string_pretty(&duplicate_data).expect("encode"),
        )
        .expect("write");

        let due = storage.get_due_jobs(
            hour as i32,
            minute as i32,
            now_bj.weekday().num_days_from_monday(),
            &["feishu"],
        );
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn list_jobs_isolated_by_actor_scope() {
        let dir = make_temp_dir("hone_cron_storage_scope");
        let storage = CronJobStorage::new(&dir);
        let actor_one = actor("discord", "alice", Some("g:1:c:1"));
        let actor_two = actor("discord", "alice", Some("g:1:c:2"));

        assert_eq!(
            storage.add_job(
                &actor_one,
                "report one",
                Some(9),
                Some(0),
                "daily",
                "task one",
                "alice",
                None,
                None,
                true,
                None,
                false,
            )["success"],
            true
        );
        assert_eq!(
            storage.add_job(
                &actor_two,
                "report two",
                Some(9),
                Some(30),
                "daily",
                "task two",
                "alice",
                None,
                None,
                true,
                None,
                false,
            )["success"],
            true
        );

        let first = storage.list_jobs(&actor_one);
        let second = storage.list_jobs(&actor_two);
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(first[0].name, "report one");
        assert_eq!(second[0].name, "report two");
    }

    #[test]
    fn sixth_enabled_job_is_rejected_but_disabled_job_is_allowed() {
        let dir = make_temp_dir("hone_cron_storage_limit_add");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("discord", "alice", None);

        for index in 0..MAX_ENABLED_JOBS_PER_ACTOR {
            assert_eq!(
                add_enabled_job(&storage, &actor, &format!("job-{index}"))["success"],
                true
            );
        }

        let rejected = add_enabled_job(&storage, &actor, "job-6");
        assert_eq!(rejected["success"], false);
        assert_eq!(rejected["error"], cron_enabled_limit_error());

        let disabled = storage.add_job(
            &actor,
            "disabled",
            Some(9),
            Some(0),
            "daily",
            "task",
            "alice",
            None,
            None,
            false,
            None,
            false,
        );
        assert_eq!(disabled["success"], true);
        assert_eq!(
            storage.list_jobs(&actor).len(),
            MAX_ENABLED_JOBS_PER_ACTOR + 1
        );
    }

    #[test]
    fn enabling_sixth_job_via_toggle_or_update_is_rejected() {
        let dir = make_temp_dir("hone_cron_storage_limit_enable");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("discord", "alice", None);

        let mut job_ids = Vec::new();
        for index in 0..MAX_ENABLED_JOBS_PER_ACTOR {
            let result = add_enabled_job(&storage, &actor, &format!("job-{index}"));
            job_ids.push(result["job"]["id"].as_str().unwrap_or_default().to_string());
        }

        let disabled = storage.add_job(
            &actor,
            "disabled",
            Some(9),
            Some(0),
            "daily",
            "task",
            "alice",
            None,
            None,
            false,
            None,
            false,
        );
        let disabled_id = disabled["job"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let toggle_err = storage
            .toggle_job(&disabled_id, Some(&actor), false)
            .expect_err("toggle should hit limit");
        assert!(is_cron_enabled_limit_error(&toggle_err.to_string()));

        let update_err = storage
            .update_job(
                &disabled_id,
                Some(&actor),
                CronJobUpdate {
                    enabled: Some(true),
                    ..Default::default()
                },
                false,
            )
            .expect_err("update should hit limit");
        assert!(is_cron_enabled_limit_error(&update_err.to_string()));

        storage
            .toggle_job(&job_ids[0], Some(&actor), false)
            .expect("disable first job");

        let enabled = storage
            .toggle_job(&disabled_id, Some(&actor), false)
            .expect("toggle after freeing slot")
            .expect("job exists");
        assert!(enabled.1.enabled);
    }

    #[test]
    fn heartbeat_jobs_run_once_per_half_hour_slot() {
        let dir = make_temp_dir("hone_cron_storage_heartbeat");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_heartbeat", None);
        let add = storage.add_job(
            &actor,
            "price watch",
            None,
            None,
            "heartbeat",
            "当闪迪低于 520 提醒我",
            "ou_heartbeat",
            None,
            None,
            true,
            Some(vec!["heartbeat".to_string()]),
            false,
        );
        assert_eq!(add["success"], true);
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        let now_bj = chrono::Utc::now()
            .with_timezone(&FixedOffset::east_opt(BEIJING_OFFSET).expect("offset"));
        let due_first =
            storage.get_due_jobs(10, 30, now_bj.weekday().num_days_from_monday(), &["feishu"]);
        assert_eq!(due_first.len(), 1);
        assert_eq!(due_first[0].1.id, job_id);
        assert!(due_first[0].1.is_heartbeat());

        let mut data = storage.load_jobs(&actor);
        let slot_time = now_bj
            .with_hour(10)
            .and_then(|dt| dt.with_minute(30))
            .and_then(|dt| dt.with_second(0))
            .expect("slot time");
        let job = data
            .jobs
            .iter_mut()
            .find(|job| job.id == job_id)
            .expect("job exists");
        job.last_run_at = Some(slot_time.to_rfc3339());
        storage.save_jobs(&actor, &data).expect("save");
        let due_second =
            storage.get_due_jobs(10, 30, now_bj.weekday().num_days_from_monday(), &["feishu"]);
        assert!(due_second.is_empty());
    }

    #[test]
    fn daily_jobs_catch_up_after_missed_window_same_day() {
        let dir = make_temp_dir("hone_cron_storage_catch_up");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_catch_up", None);

        let add = storage.add_job(
            &actor,
            "daily report",
            Some(9),
            Some(30),
            "daily",
            "task",
            "ou_catch_up",
            None,
            None,
            true,
            None,
            false,
        );
        assert_eq!(add["success"], true);
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        let mut data = storage.load_jobs(&actor);
        let job = data
            .jobs
            .iter_mut()
            .find(|job| job.id == job_id)
            .expect("job exists");
        job.created_at = Some("2026-03-29T08:00:00+08:00".to_string());
        storage.save_jobs(&actor, &data).expect("save");

        let due = storage.get_due_jobs(12, 0, 6, &["feishu"]);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].1.id, job_id);
    }

    #[test]
    fn daily_jobs_created_after_slot_do_not_backfill_immediately() {
        let dir = make_temp_dir("hone_cron_storage_no_backfill_new_job");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_new_job", None);

        let add = storage.add_job(
            &actor,
            "late daily report",
            Some(9),
            Some(30),
            "daily",
            "task",
            "ou_new_job",
            None,
            None,
            true,
            None,
            false,
        );
        assert_eq!(add["success"], true);
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        let mut data = storage.load_jobs(&actor);
        let job = data
            .jobs
            .iter_mut()
            .find(|job| job.id == job_id)
            .expect("job exists");
        job.created_at = Some("2026-03-29T12:15:00+08:00".to_string());
        storage.save_jobs(&actor, &data).expect("save");

        let due = storage.get_due_jobs(12, 30, 6, &["feishu"]);
        assert!(due.is_empty());
    }

    #[test]
    fn execution_records_are_persisted_in_sqlite() {
        let dir = make_temp_dir("hone_cron_storage_exec_records");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_exec", None);

        let add = storage.add_job(
            &actor,
            "daily report",
            Some(9),
            Some(0),
            "daily",
            "task",
            "ou_exec",
            None,
            None,
            true,
            None,
            false,
        );
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        storage
            .record_execution_event(
                &actor,
                &job_id,
                "daily report",
                "ou_exec",
                false,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "sent".to_string(),
                    should_deliver: true,
                    delivered: true,
                    response_preview: Some("hello world".to_string()),
                    error_message: None,
                    detail: serde_json::json!({"sent_segments": 1}),
                },
            )
            .expect("record execution");

        let records = storage
            .list_execution_records(&job_id, 10)
            .expect("list execution records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].job_id, job_id);
        assert_eq!(records[0].execution_status, "completed");
        assert_eq!(records[0].message_send_status, "sent");
        assert!(records[0].delivered);
        assert_eq!(records[0].detail["sent_segments"], 1);
    }
}
