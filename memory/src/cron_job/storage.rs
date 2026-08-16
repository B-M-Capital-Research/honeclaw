//! CronJobStorage PostgreSQL 存储层：按 actor 的定时任务 CRUD + 触发判定。

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Timelike, Utc};
use hone_core::cloud_runtime::CloudCronJobRecord;
use hone_core::{ActorIdentity, HoneError};
use tracing::warn;
use uuid::Uuid;

use super::CronJobStorage;
use super::run_cloud_cron;
use super::schedule::{
    DUE_WINDOW_MINUTES, is_holiday, is_trading_day, is_workday, job_existed_before_slot_in,
    normalize_schedule_date, normalized_repeat, normalized_tags, prompt_schedule_conflict,
    validate_schedule, validate_schedule_date,
};
use super::types::{
    ChannelTargetRecord, CronJob, CronJobData, CronJobUpdate, CronSchedule,
    MAX_ENABLED_JOBS_PER_ACTOR, cron_enabled_limit_error,
};

fn push_unique(values: &mut Vec<String>, value: &str) {
    let trimmed = value.trim();
    if !trimmed.is_empty() && !values.iter().any(|existing| existing == trimmed) {
        values.push(trimmed.to_string());
    }
}

fn newer_optional_string(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let left_instant = DateTime::parse_from_rfc3339(&left).ok();
            let right_instant = DateTime::parse_from_rfc3339(&right).ok();
            match (left_instant, right_instant) {
                (Some(left_instant), Some(right_instant)) => {
                    Some(if left_instant >= right_instant {
                        left
                    } else {
                        right
                    })
                }
                _ => Some(left.max(right)),
            }
        }
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn job_channel_allowed(job: &CronJob, channels: &[&str]) -> bool {
    channels.is_empty() || channels.contains(&job.channel.as_str())
}

/// 齐射削峰:把同一个半点槽到期的 **heartbeat** 按 `job_id` 确定性地摊到
/// `[0, JITTER_SPREAD_MINUTES)` 分钟内。
///
/// 背景:生产上 25 个 heartbeat 每个半点整齐射(每天 48 轮,约 1200 次执行),
/// 峰值单分钟 86 次;这些分钟的失败率明显高于基线(20:01 为 35%、08:31 为 28%,
/// 基线约 13%),失败集中在建连阶段的上游传输错误。
///
/// **只对 heartbeat 生效**:heartbeat 压根不看 `schedule.hour/minute`(每半点自动
/// 触发),推迟几分钟对用户不可见。用户显式设定时刻的定时任务(如 20:00 日报)不加
/// 抖动 —— 那是产品语义,改触发时刻要单独决策;它们的削峰靠投递侧并发闸完成,
/// 那条路径不改变任何触发时刻。
///
/// **必须严格小于 [`DUE_WINDOW_MINUTES`]** —— 被推迟的任务要靠后续 tick 重新进入
/// 同一个到期窗口才能跑到,偏移量若够到窗口边界,该轮就会被整个丢掉。
pub(super) const JITTER_SPREAD_MINUTES: i32 = 4;
const _: () = assert!(
    JITTER_SPREAD_MINUTES < DUE_WINDOW_MINUTES,
    "抖动偏移必须留在到期容错窗口内,否则会丢任务"
);

/// 确定性偏移:同一个 job 每天落在同一分钟,便于复现和排查。
///
/// 用手写 FNV-1a 而不是 `DefaultHasher`——后者的实现不保证跨 Rust 版本稳定,
/// 升级工具链就会让所有任务的触发分钟集体漂移。
pub(super) fn dispatch_jitter_minutes(job_id: &str) -> i32 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in job_id.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (hash % JITTER_SPREAD_MINUTES as u64) as i32
}

fn heartbeat_due_in_current_window(job: &CronJob, current_total: i32) -> bool {
    let slot_minute = (current_total / 30) * 30;
    let earliest = slot_minute + dispatch_jitter_minutes(&job.id);
    earliest <= current_total && current_total <= slot_minute + DUE_WINDOW_MINUTES
}

fn scheduled_job_due_in_current_window(
    job: &CronJob,
    current_total: i32,
    current_day: NaiveDate,
    timezone: &hone_core::RuntimeTimezone,
) -> bool {
    let job_total = (job.schedule.hour as i32) * 60 + (job.schedule.minute as i32);
    let due_in_window =
        current_total - DUE_WINDOW_MINUTES <= job_total && job_total <= current_total;
    let due_by_catch_up =
        current_total > job_total && job_existed_before_slot_in(job, current_day, timezone);
    due_in_window || due_by_catch_up
}

fn job_due_in_current_window(
    job: &CronJob,
    current_total: i32,
    current_day: NaiveDate,
    timezone: &hone_core::RuntimeTimezone,
) -> bool {
    if job.is_heartbeat() {
        heartbeat_due_in_current_window(job, current_total)
    } else {
        scheduled_job_due_in_current_window(job, current_total, current_day, timezone)
    }
}

fn once_job_matches_current_day(job: &CronJob, current_day: NaiveDate) -> bool {
    let Some(date) = job.schedule.date.as_deref() else {
        return true;
    };
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|scheduled_day| scheduled_day == current_day)
        .unwrap_or(false)
}

fn repeat_matches_current_day(
    job: &CronJob,
    repeat_kind: &str,
    current_day: NaiveDate,
    current_weekday: u32,
) -> bool {
    if repeat_kind == "once" && !once_job_matches_current_day(job, current_day) {
        return false;
    }

    match repeat_kind {
        "weekly" => job.schedule.weekday == Some(current_weekday),
        "workday" => is_workday(current_day),
        "trading_day" => is_trading_day(current_day),
        "holiday" => is_holiday(current_day),
        _ => true,
    }
}

fn already_ran_in_current_period(
    job: &CronJob,
    repeat_kind: &str,
    now: chrono::DateTime<FixedOffset>,
    current_total: i32,
    timezone: &hone_core::RuntimeTimezone,
) -> bool {
    let Some(last_run) = job.last_run_at.as_deref() else {
        return false;
    };
    let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(last_run) else {
        return false;
    };

    let last_local = timezone.at_utc(last_dt.with_timezone(&Utc));
    match repeat_kind {
        "heartbeat" => {
            let current_slot_start_minute = (current_total / 30) * 30;
            let current_slot_hour = current_slot_start_minute / 60;
            let current_slot_minute = current_slot_start_minute % 60;
            last_local.date_naive() == now.date_naive()
                && last_local.hour() as i32 == current_slot_hour
                && (last_local.minute() as i32 / 30) == (current_slot_minute / 30)
        }
        "weekly" => last_local.iso_week() == now.iso_week() && last_local.year() == now.year(),
        "once" => true,
        _ => last_local.date_naive() == now.date_naive(),
    }
}

fn due_job_dedup_key(job: &CronJob) -> String {
    format!("{}:{}:{}", job.channel, job.id, job.channel_target)
}

impl CronJobStorage {
    pub fn list_all_jobs(&self) -> Vec<(ActorIdentity, CronJob)> {
        let postgres = self.postgres.clone();
        return match run_cloud_cron(async move { postgres.list_cron_job_records().await }) {
            Ok(records) => records
                .into_iter()
                .filter_map(cron_pair_from_cloud_record)
                .collect(),
            Err(error) => {
                warn!("failed to list cloud cron jobs: {error}");
                Vec::new()
            }
        };
    }

    /// Fallibly load one actor's scheduled-task data.
    ///
    /// Mutation paths must use this method so a cloud or local read failure
    /// cannot be mistaken for an empty task list and reported as success.
    pub fn try_load_jobs(&self, actor: &ActorIdentity) -> hone_core::HoneResult<CronJobData> {
        let postgres = self.postgres.clone();
        let actor_key = actor.storage_key();
        let records =
            run_cloud_cron(
                async move { postgres.list_cron_job_records_for_actor(&actor_key).await },
            )?;
        return Ok(CronJobData {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            jobs: records
                .into_iter()
                .map(|record| {
                    cron_pair_from_cloud_record(record)
                        .map(|(_, job)| job)
                        .ok_or_else(|| {
                            HoneError::Serialization(
                                "PostgreSQL cron record 无法反序列化".to_string(),
                            )
                        })
                })
                .collect::<hone_core::HoneResult<Vec<_>>>()?,
            pending_updates: Vec::new(),
        });
    }

    /// 加载 actor 的定时任务数据
    pub fn load_jobs(&self, actor: &ActorIdentity) -> CronJobData {
        match self.try_load_jobs(actor) {
            Ok(data) => data,
            Err(error) => {
                warn!(
                    actor = %actor.storage_key(),
                    "failed to load cron jobs: {error}"
                );
                CronJobData {
                    actor: Some(actor.clone()),
                    user_id: actor.user_id.clone(),
                    jobs: Vec::new(),
                    pending_updates: Vec::new(),
                }
            }
        }
    }

    /// 保存 actor 的定时任务数据
    pub fn save_jobs(
        &self,
        actor: &ActorIdentity,
        data: &CronJobData,
    ) -> hone_core::HoneResult<()> {
        let postgres = self.postgres.clone();
        let actor_key = actor.storage_key();
        let actor_value = serde_json::to_value(actor)
            .map_err(|err| hone_core::HoneError::Serialization(err.to_string()))?;
        let records = data
            .jobs
            .iter()
            .map(|job| {
                Ok(CloudCronJobRecord {
                    actor_storage_key: actor_key.clone(),
                    job_id: job.id.clone(),
                    actor: actor_value.clone(),
                    job: serde_json::to_value(job)
                        .map_err(|err| HoneError::Serialization(err.to_string()))?,
                })
            })
            .collect::<hone_core::HoneResult<Vec<_>>>()?;
        return run_cloud_cron(async move {
            let existing = postgres.list_cron_job_records_for_actor(&actor_key).await?;
            let wanted = records
                .iter()
                .map(|record| record.job_id.clone())
                .collect::<HashSet<_>>();
            for record in existing {
                if !wanted.contains(&record.job_id) {
                    postgres
                        .delete_cron_job_record(&actor_key, &record.job_id)
                        .await?;
                }
            }
            for record in records {
                postgres
                    .upsert_cron_job_record(
                        &record.actor_storage_key,
                        &record.job_id,
                        record.actor,
                        record.job,
                    )
                    .await?;
            }
            Ok(())
        });
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

    pub fn list_channel_targets(&self) -> Vec<ChannelTargetRecord> {
        let mut records: BTreeMap<(String, Option<String>, String), ChannelTargetRecord> =
            BTreeMap::new();

        for (actor, job) in self.list_all_jobs() {
            let target = job.channel_target.trim();
            if target.is_empty() {
                continue;
            }
            let channel = if job.channel.trim().is_empty() {
                actor.channel.clone()
            } else {
                job.channel.trim().to_string()
            };
            let channel_scope = job
                .channel_scope
                .clone()
                .or_else(|| actor.channel_scope.clone())
                .filter(|scope| !scope.trim().is_empty());
            let key = (channel.clone(), channel_scope.clone(), target.to_string());
            let record = records.entry(key).or_insert_with(|| ChannelTargetRecord {
                channel,
                channel_scope,
                target: target.to_string(),
                actor_user_ids: Vec::new(),
                sources: Vec::new(),
                scheduled_jobs: 0,
                enabled_jobs: 0,
                last_seen_at: None,
            });
            push_unique(&mut record.actor_user_ids, &actor.user_id);
            push_unique(&mut record.sources, "cron_job");
            record.scheduled_jobs += 1;
            if job.enabled {
                record.enabled_jobs += 1;
            }
            record.last_seen_at =
                newer_optional_string(record.last_seen_at.clone(), job.created_at);
        }

        let executions = self
            .list_recent_executions(&super::ExecutionFilter {
                limit: 1000,
                ..super::ExecutionFilter::default()
            })
            .unwrap_or_default();
        for execution in executions {
            let target = execution.channel_target.trim();
            if target.is_empty() {
                continue;
            }
            let channel = execution.channel.trim().to_string();
            if channel.is_empty() {
                continue;
            }
            let channel_scope = execution
                .channel_scope
                .clone()
                .filter(|scope| !scope.trim().is_empty());
            let key = (channel.clone(), channel_scope.clone(), target.to_string());
            let record = records.entry(key).or_insert_with(|| ChannelTargetRecord {
                channel,
                channel_scope,
                target: target.to_string(),
                actor_user_ids: Vec::new(),
                sources: Vec::new(),
                scheduled_jobs: 0,
                enabled_jobs: 0,
                last_seen_at: None,
            });
            push_unique(&mut record.actor_user_ids, &execution.user_id);
            push_unique(&mut record.sources, "cron_execution");
            record.last_seen_at =
                newer_optional_string(record.last_seen_at.clone(), Some(execution.executed_at));
        }

        records.into_values().collect()
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
        date: Option<String>,
        push: Option<serde_json::Value>,
        enabled: bool,
        tags: Option<Vec<String>>,
        bypass_limits: bool,
    ) -> serde_json::Value {
        let mut data = self.load_jobs(actor);
        let channel_target = channel_target.trim();
        if channel_target.is_empty() {
            return serde_json::json!({
                "success": false,
                "error": "channel_target 不能为空；定时任务必须保存创建它的来源渠道目标"
            });
        }

        let enabled_count = data.jobs.iter().filter(|j| j.enabled).count();
        if enabled && !bypass_limits && enabled_count >= MAX_ENABLED_JOBS_PER_ACTOR {
            return serde_json::json!({
                "success": false,
                "error": cron_enabled_limit_error()
            });
        }

        let tags = normalized_tags(tags.unwrap_or_default(), repeat);
        let is_heartbeat = super::schedule::is_heartbeat_repeat_or_tags(repeat, &tags);
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
        let date = normalize_schedule_date(date);
        if let Err(error) = validate_schedule_date(repeat, date.as_deref()) {
            return serde_json::json!({"success": false, "error": error});
        }

        let job_id = format!("j_{}", &Uuid::new_v4().to_string()[..8]);
        let now = hone_core::local_now_rfc3339();

        let job = CronJob {
            id: job_id,
            name: name.to_string(),
            schedule: CronSchedule {
                hour,
                minute,
                repeat: repeat.to_string(),
                weekday,
                date,
            },
            task_prompt: task_prompt.to_string(),
            push: push.unwrap_or_else(|| serde_json::json!({"type": "analysis"})),
            enabled,
            channel: actor.channel.clone(),
            channel_scope: actor.channel_scope.clone(),
            channel_target: channel_target.to_string(),
            tags,
            created_at: Some(now),
            last_run_at: None,
            bypass_quiet_hours: false,
        };
        if let Some((declared_hour, declared_minute)) = prompt_schedule_conflict(&job) {
            return serde_json::json!({
                "success": false,
                "error": format!(
                    "task_prompt 声明的触发时间 {:02}:{:02} 与结构化 schedule {:02}:{:02} 不一致",
                    declared_hour,
                    declared_minute,
                    job.schedule.hour,
                    job.schedule.minute
                )
            });
        }

        let job_value = serde_json::to_value(&job).unwrap_or_default();
        data.jobs.push(job);
        if let Err(error) = self.save_jobs(actor, &data) {
            return serde_json::json!({
                "success": false,
                "error": format!("保存定时任务失败: {error}")
            });
        }

        serde_json::json!({"success": true, "job": job_value})
    }

    /// 删除定时任务
    pub fn remove_job(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
    ) -> hone_core::HoneResult<serde_json::Value> {
        let mut data = self.try_load_jobs(actor)?;
        let original_len = data.jobs.len();
        data.jobs.retain(|j| j.id != job_id);
        if data.jobs.len() == original_len {
            return Ok(
                serde_json::json!({"success": false, "error": format!("未找到任务 {job_id}")}),
            );
        }
        data.pending_updates
            .retain(|pending| pending.job_id != job_id);
        self.save_jobs(actor, &data)?;
        Ok(serde_json::json!({"success": true, "removed_job_id": job_id}))
    }

    /// Atomically remove every scheduled and heartbeat task owned by one actor.
    ///
    /// This is intentionally actor-scoped and idempotent. Persistence errors
    /// are propagated so callers never tell a user that cancellation succeeded
    /// while durable jobs are still present.
    pub fn remove_all_jobs(&self, actor: &ActorIdentity) -> hone_core::HoneResult<Vec<CronJob>> {
        let mut data = self.try_load_jobs(actor)?;
        let removed = std::mem::take(&mut data.jobs);
        data.pending_updates.clear();
        self.save_jobs(actor, &data)?;
        Ok(removed)
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
            if let Some(mut schedule) = updates.schedule.clone() {
                schedule.date = normalize_schedule_date(schedule.date);
                validate_schedule(
                    Some(schedule.hour),
                    Some(schedule.minute),
                    &schedule.repeat,
                    schedule.weekday,
                )
                .map_err(hone_core::HoneError::Tool)?;
                validate_schedule_date(&schedule.repeat, schedule.date.as_deref())
                    .map_err(hone_core::HoneError::Tool)?;
                job.schedule = schedule;
                job.tags = normalized_tags(job.tags.clone(), &job.schedule.repeat);
            }
            if let Some(task_prompt) = updates.task_prompt.clone() {
                job.task_prompt = task_prompt;
            }
            if let Some((declared_hour, declared_minute)) = prompt_schedule_conflict(job) {
                return Err(hone_core::HoneError::Tool(format!(
                    "task_prompt 声明的触发时间 {:02}:{:02} 与结构化 schedule {:02}:{:02} 不一致",
                    declared_hour, declared_minute, job.schedule.hour, job.schedule.minute
                )));
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
            if let Some(bypass) = updates.bypass_quiet_hours {
                job.bypass_quiet_hours = bypass;
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

        for actor in self.list_unique_cron_actors() {
            if let Some(removed) = self.delete_job_for_actor(job_id, &actor)? {
                return Ok(Some(removed));
            }
        }

        Ok(None)
    }

    /// 标记任务已执行
    pub fn mark_job_run(&self, actor: &ActorIdentity, job_id: &str) {
        let mut data = self.load_jobs(actor);
        let now = hone_core::local_now_rfc3339();
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

    /// 扫描所有 actor 的 cron 文件，返回当前时刻应触发的任务列表。
    ///
    /// 触发判定要同时满足多个维度：
    /// 1. `enabled = true`
    /// 2. `channels` 非空时要求 `job.channel` 命中（避免多渠道进程共享目录时相互误触发）
    /// 3. 时间窗口命中（heartbeat 走 30 分钟半点槽；普通任务走 `[job_total - DUE_WINDOW, job_total]`
    ///    的容错窗口，或在同日内的「错过后补跑」条件下命中）
    /// 4. 按 `repeat` 过滤星期/工作日/交易日/假日
    /// 5. `last_run_at` 未命中当前周期（heartbeat 以半点槽，weekly 以 ISO 周，once 只跑一次）
    /// 6. 跨文件去重（同一 `channel:job_id:target` 只返回一次）
    pub fn get_due_jobs_at(
        &self,
        now: DateTime<FixedOffset>,
        channels: &[&str],
    ) -> Vec<(ActorIdentity, CronJob)> {
        let mut due = Vec::new();
        let mut seen_due_keys = HashSet::new();
        let current_day = now.date_naive();
        let current_hour = now.hour() as i32;
        let current_minute = now.minute() as i32;
        let current_weekday = now.weekday().num_days_from_monday();
        let current_total = current_hour * 60 + current_minute;
        let timezone = hone_core::runtime_timezone();

        let postgres = self.postgres.clone();
        let owner_id = cron_owner_id();
        for (actor, mut job) in self.list_all_jobs() {
            if repair_prompt_schedule_mismatch(&mut job) {
                let mut data = self.load_jobs(&actor);
                if let Some(saved) = data.jobs.iter_mut().find(|saved| saved.id == job.id) {
                    *saved = job.clone();
                    if let Err(error) = self.save_jobs(&actor, &data) {
                        warn!(
                            actor = %actor.storage_key(),
                            job_id = %job.id,
                            "failed to persist repaired PostgreSQL cron schedule: {error}"
                        );
                    }
                }
            }
            if !job.enabled {
                continue;
            }

            if !job_channel_allowed(&job, channels) {
                continue;
            }

            if !job_due_in_current_window(&job, current_total, current_day, &timezone) {
                continue;
            }

            let repeat_kind = normalized_repeat(&job.schedule.repeat, &job.tags);
            if !repeat_matches_current_day(&job, repeat_kind, current_day, current_weekday) {
                continue;
            }

            if already_ran_in_current_period(&job, repeat_kind, now, current_total, &timezone) {
                continue;
            }

            let dedup_key = due_job_dedup_key(&job);
            if !seen_due_keys.insert(dedup_key) {
                warn!(
                    "skipping duplicate due cloud cron job actor={} job_id={} target={}",
                    actor.storage_key(),
                    job.id,
                    job.channel_target
                );
                continue;
            }

            let job_key = cron_claim_job_key(&actor, &job);
            let due_key = cron_claim_due_key(&job, repeat_kind, current_total, current_day);
            let claim = run_cloud_cron({
                let postgres = postgres.clone();
                let owner_id = owner_id.clone();
                async move {
                    postgres
                        .try_claim_cron_due_job(&job_key, &due_key, &owner_id)
                        .await
                }
            });
            match claim {
                Ok(true) => due.push((actor, job)),
                Ok(false) => {}
                Err(error) => warn!(
                    "failed to claim cloud cron due job actor={} job_id={}: {error}",
                    actor.storage_key(),
                    job.id
                ),
            }
        }
        return due;
    }

    #[cfg(test)]
    pub fn get_due_jobs(
        &self,
        current_hour: i32,
        current_minute: i32,
        current_weekday: u32,
        channels: &[&str],
    ) -> Vec<(ActorIdentity, CronJob)> {
        let now = hone_core::local_now()
            .with_hour(current_hour as u32)
            .and_then(|value| value.with_minute(current_minute as u32))
            .and_then(|value| value.with_second(0))
            .expect("valid test-local cron time");
        debug_assert_eq!(now.weekday().num_days_from_monday(), current_weekday);
        self.get_due_jobs_at(now, channels)
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

        for actor in self.list_unique_cron_actors() {
            if let Some(updated) =
                self.mutate_job_for_actor(job_id, &actor, bypass_limits, &mut mutator)?
            {
                return Ok(Some(updated));
            }
        }

        Ok(None)
    }

    fn list_unique_cron_actors(&self) -> Vec<ActorIdentity> {
        let mut actors = BTreeMap::new();
        for (actor, _) in self.list_all_jobs() {
            actors.entry(actor.storage_key()).or_insert(actor);
        }
        actors.into_values().collect()
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
        let mut data = self.try_load_jobs(actor)?;
        let Some(index) = data.jobs.iter().position(|job| job.id == job_id) else {
            return Ok(None);
        };
        let removed = data.jobs.remove(index);
        data.pending_updates
            .retain(|pending| pending.job_id != job_id);
        self.save_jobs(actor, &data)?;
        Ok(Some((actor.clone(), removed)))
    }
}

fn cron_pair_from_cloud_record(record: CloudCronJobRecord) -> Option<(ActorIdentity, CronJob)> {
    let actor = serde_json::from_value::<ActorIdentity>(record.actor).ok()?;
    let job = serde_json::from_value::<CronJob>(record.job).ok()?;
    Some((actor, job))
}

fn cron_owner_id() -> String {
    std::env::var("HONE_RUNTIME_OWNER_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "localhost".to_string())
        + &format!(":{}", std::process::id())
}

fn cron_claim_job_key(actor: &ActorIdentity, job: &CronJob) -> String {
    format!("{}::{}::{}", actor.storage_key(), job.channel, job.id)
}

fn cron_claim_due_key(
    job: &CronJob,
    repeat_kind: &str,
    current_total: i32,
    current_day: NaiveDate,
) -> String {
    if repeat_kind == "heartbeat" {
        let slot_minute = (current_total / 30) * 30;
        return format!("heartbeat:{}:{slot_minute}", current_day.format("%F"));
    }
    format!(
        "{}:{}:{:02}:{:02}",
        repeat_kind,
        current_day.format("%F"),
        job.schedule.hour,
        job.schedule.minute
    )
}

fn repair_prompt_schedule_mismatch(job: &mut CronJob) -> bool {
    let Some((declared_hour, declared_minute)) = prompt_schedule_conflict(job) else {
        return false;
    };
    warn!(
        "repairing legacy cron job schedule/prompt mismatch: job_id={} job={} schedule={:02}:{:02} prompt={:02}:{:02}",
        job.id, job.name, job.schedule.hour, job.schedule.minute, declared_hour, declared_minute
    );
    job.schedule.hour = declared_hour;
    job.schedule.minute = declared_minute;
    true
}

#[cfg(test)]
mod timezone_regression_tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn job(id: &str, hour: u32, minute: u32, repeat: &str) -> CronJob {
        CronJob {
            id: id.to_string(),
            name: "timezone regression".to_string(),
            schedule: CronSchedule {
                hour,
                minute,
                repeat: repeat.to_string(),
                weekday: None,
                date: None,
            },
            task_prompt: "test".to_string(),
            push: serde_json::Value::Null,
            enabled: true,
            channel: "test".to_string(),
            channel_scope: None,
            channel_target: "test".to_string(),
            tags: (repeat == "heartbeat")
                .then(|| vec!["heartbeat".to_string()])
                .unwrap_or_default(),
            created_at: Some("2026-01-15T02:00:00Z".to_string()),
            last_run_at: None,
            bypass_quiet_hours: false,
        }
    }

    #[test]
    fn non_eight_timezone_drives_cron_date_key_and_rendering() {
        let timezone = hone_core::RuntimeTimezone::parse_iana("America/New_York").unwrap();
        let instant = Utc.with_ymd_and_hms(2026, 1, 15, 4, 32, 0).unwrap();
        let local = timezone.at_utc(instant);

        assert_eq!(local.date_naive().to_string(), "2026-01-14");
        assert_eq!(local.format("%H:%M").to_string(), "23:32");
        assert!(local.to_rfc3339().ends_with("-05:00"));

        let current_total = local.hour() as i32 * 60 + local.minute() as i32;
        assert!(scheduled_job_due_in_current_window(
            &job("daily", 23, 30, "daily"),
            current_total,
            local.date_naive(),
            &timezone,
        ));

        let heartbeat = job("heartbeat", 0, 0, "heartbeat");
        let heartbeat_total = local.hour() as i32 * 60 + 33;
        assert!(heartbeat_due_in_current_window(&heartbeat, heartbeat_total));
        assert_eq!(
            cron_claim_due_key(&heartbeat, "heartbeat", heartbeat_total, local.date_naive(),),
            "heartbeat:2026-01-14:1410"
        );
    }
}
