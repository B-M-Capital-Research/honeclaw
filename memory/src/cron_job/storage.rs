//! CronJobStorage JSON 存储层：按 actor 的定时任务 CRUD + 触发判定。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
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
    CronJob, CronJobData, CronJobUpdate, CronSchedule, MAX_ENABLED_JOBS_PER_ACTOR,
    cron_enabled_limit_error,
};

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
            channel_target: if channel_target.is_empty() {
                actor.user_id.clone()
            } else {
                channel_target.to_string()
            },
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
                if let Some((declared_hour, declared_minute)) = prompt_schedule_conflict(job) {
                    warn!(
                        "skipping cron job with schedule/prompt mismatch: job_id={} job={} schedule={:02}:{:02} prompt={:02}:{:02}",
                        job.id,
                        job.name,
                        job.schedule.hour,
                        job.schedule.minute,
                        declared_hour,
                        declared_minute
                    );
                    continue;
                }

                let job_total = (job.schedule.hour as i32) * 60 + (job.schedule.minute as i32);
                let is_heartbeat = job.is_heartbeat();
                if is_heartbeat {
                    // Heartbeat 任务按每半小时的整点槽触发；只在槽起点及其后的 DUE_WINDOW 分钟
                    // 内视为「当前槽内」,其它时刻一律跳过。
                    let slot_minute = (current_total / 30) * 30;
                    if !(slot_minute <= current_total
                        && current_total <= slot_minute + DUE_WINDOW_MINUTES)
                    {
                        continue;
                    }
                } else {
                    // 普通任务：
                    // - `due_in_window`: 处在计划时刻前 DUE_WINDOW 分钟到计划时刻之间
                    // - `due_by_catch_up`: 已经过了计划时刻,但任务在当天的计划时刻之前就存在,
                    //   说明是当天错过的那次调用,允许同日内补跑一次
                    let due_in_window = current_total - DUE_WINDOW_MINUTES <= job_total
                        && job_total <= current_total;
                    let due_by_catch_up =
                        current_total > job_total && job_existed_before_slot(job, current_day);
                    if !(due_in_window || due_by_catch_up) {
                        continue;
                    }
                }

                let repeat_kind = normalized_repeat(&job.schedule.repeat, &job.tags);
                if repeat_kind == "once"
                    && let Some(date) = job.schedule.date.as_deref()
                    && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .map(|scheduled_day| scheduled_day != current_day)
                        .unwrap_or(true)
                {
                    continue;
                }
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

                // 去抖：已经在当前周期内跑过一次就跳过。各 repeat 类型用不同的粒度比较：
                // heartbeat 以「当日 + 同一 30 分钟槽」；weekly 以 ISO 周编号;
                // once 跑完就永久视为已跑；其它(daily/workday/trading_day/holiday)以自然日。
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
