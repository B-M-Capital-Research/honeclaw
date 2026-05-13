//! CronJobStorage JSON 存储层：按 actor 的定时任务 CRUD + 触发判定。

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use chrono::{Datelike, FixedOffset, NaiveDate, Timelike};
use hone_core::ActorIdentity;
use tracing::warn;
use uuid::Uuid;

use super::CronJobStorage;
use super::schedule::{
    DUE_WINDOW_MINUTES, is_holiday, is_trading_day, is_workday, job_existed_before_slot,
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
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn job_channel_allowed(job: &CronJob, channels: &[&str]) -> bool {
    channels.is_empty() || channels.contains(&job.channel.as_str())
}

fn heartbeat_due_in_current_window(current_total: i32) -> bool {
    let slot_minute = (current_total / 30) * 30;
    slot_minute <= current_total && current_total <= slot_minute + DUE_WINDOW_MINUTES
}

fn scheduled_job_due_in_current_window(
    job: &CronJob,
    current_total: i32,
    current_day: NaiveDate,
) -> bool {
    let job_total = (job.schedule.hour as i32) * 60 + (job.schedule.minute as i32);
    let due_in_window =
        current_total - DUE_WINDOW_MINUTES <= job_total && job_total <= current_total;
    let due_by_catch_up = current_total > job_total && job_existed_before_slot(job, current_day);
    due_in_window || due_by_catch_up
}

fn job_due_in_current_window(job: &CronJob, current_total: i32, current_day: NaiveDate) -> bool {
    if job.is_heartbeat() {
        heartbeat_due_in_current_window(current_total)
    } else {
        scheduled_job_due_in_current_window(job, current_total, current_day)
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
) -> bool {
    let Some(last_run) = job.last_run_at.as_deref() else {
        return false;
    };
    let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(last_run) else {
        return false;
    };

    match repeat_kind {
        "heartbeat" => {
            let current_slot_start_minute = (current_total / 30) * 30;
            let current_slot_hour = current_slot_start_minute / 60;
            let current_slot_minute = current_slot_start_minute % 60;
            last_dt.date_naive() == now.date_naive()
                && last_dt.hour() as i32 == current_slot_hour
                && (last_dt.minute() as i32 / 30) == (current_slot_minute / 30)
        }
        "weekly" => last_dt.iso_week() == now.iso_week() && last_dt.year() == now.year(),
        "once" => true,
        _ => last_dt.date_naive() == now.date_naive(),
    }
}

fn due_job_dedup_key(job: &CronJob) -> String {
    format!("{}:{}:{}", job.channel, job.id, job.channel_target)
}

impl CronJobStorage {
    pub(super) fn get_actor_file(&self, actor: &ActorIdentity) -> PathBuf {
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
        let now = hone_core::beijing_now_rfc3339();

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
        let _ = self.save_jobs(actor, &data);

        serde_json::json!({"success": true, "job": job_value})
    }

    /// 删除定时任务
    pub fn remove_job(&self, actor: &ActorIdentity, job_id: &str) -> serde_json::Value {
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

        // 只扫描 `cron_jobs_*.json` 文件；无法读取或解析的跳过，避免一个坏文件阻塞全部扫描。

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
            let mut data: CronJobData = match serde_json::from_str(&content) {
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
            let repaired_mismatches = repair_legacy_prompt_schedule_mismatches(&mut data);
            if repaired_mismatches > 0
                && let Err(err) = self.save_jobs(&actor, &data)
            {
                warn!(
                    "failed to persist repaired cron schedule/prompt mismatches actor={} repairs={} error={}",
                    actor.storage_key(),
                    repaired_mismatches,
                    err
                );
            }

            for job in &data.jobs {
                if !job.enabled {
                    continue;
                }

                if !job_channel_allowed(job, channels) {
                    continue;
                }

                if !job_due_in_current_window(job, current_total, current_day) {
                    continue;
                }

                let repeat_kind = normalized_repeat(&job.schedule.repeat, &job.tags);
                if !repeat_matches_current_day(job, repeat_kind, current_day, current_weekday) {
                    continue;
                }

                if already_ran_in_current_period(job, repeat_kind, now, current_total) {
                    continue;
                }

                let dedup_key = due_job_dedup_key(job);
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
        let mut actors = self
            .list_all_jobs()
            .into_iter()
            .map(|(actor, _)| actor)
            .collect::<Vec<_>>();
        actors.sort_by_key(|actor| actor.storage_key());
        actors.dedup_by_key(|actor| actor.storage_key());
        actors
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

fn repair_legacy_prompt_schedule_mismatches(data: &mut CronJobData) -> usize {
    let mut repaired = 0;
    for job in &mut data.jobs {
        let Some((declared_hour, declared_minute)) = prompt_schedule_conflict(job) else {
            continue;
        };
        warn!(
            "repairing legacy cron job schedule/prompt mismatch: job_id={} job={} schedule={:02}:{:02} prompt={:02}:{:02}",
            job.id,
            job.name,
            job.schedule.hour,
            job.schedule.minute,
            declared_hour,
            declared_minute
        );
        job.schedule.hour = declared_hour;
        job.schedule.minute = declared_minute;
        repaired += 1;
    }
    repaired
}
