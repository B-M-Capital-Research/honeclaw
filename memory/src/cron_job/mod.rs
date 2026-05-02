//! 定时任务存储 — JSON 文件 + SQLite 执行记录
//!
//! 管理按 actor（channel + user_id + channel_scope）隔离的定时任务持久化存储。
//!
//! 子模块布局：
//! - [`types`]  —— 纯数据结构、错误与常量
//! - [`schedule`] —— 触发时间 / 日历 / 节假日计算
//! - [`storage`] —— `CronJobStorage` 的 JSON CRUD 与 `get_due_jobs`
//! - [`history`] —— `CronJobStorage` 的 SQLite 执行历史读写

use std::path::{Path, PathBuf};

use tracing::warn;

pub mod history;
pub mod schedule;
pub mod storage;
pub mod types;

pub use history::ExecutionFilter;
pub use types::{
    CronJob, CronJobData, CronJobExecutionInput, CronJobExecutionRecord, CronJobUpdate,
    CronSchedule, MAX_ENABLED_JOBS_PER_ACTOR, PendingUpdate, cron_enabled_limit_error,
    is_cron_enabled_limit_error,
};

/// 定时任务存储管理器
pub struct CronJobStorage {
    pub(super) data_dir: PathBuf,
    pub(super) sqlite_path: Option<PathBuf>,
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
}

#[cfg(test)]
mod tests {
    use super::schedule::beijing_slot_time;
    use super::*;
    use chrono::{Datelike, Timelike};
    use hone_core::{ActorIdentity, beijing_offset};
    use serde_json::Value;
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

        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
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

        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
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
                    date: None,
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
                bypass_quiet_hours: false,
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

        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
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
            None,
            true,
            Some(vec!["heartbeat".to_string()]),
            false,
        );
        assert_eq!(add["success"], true);
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
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
        let today = hone_core::beijing_now().date_naive();
        job.created_at = Some(beijing_slot_time(today, 8, 0).to_rfc3339());
        storage.save_jobs(&actor, &data).expect("save");

        let due = storage.get_due_jobs(
            12,
            0,
            hone_core::beijing_now().weekday().num_days_from_monday(),
            &["feishu"],
        );
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
        let today = hone_core::beijing_now().date_naive();
        job.created_at = Some(beijing_slot_time(today, 12, 15).to_rfc3339());
        storage.save_jobs(&actor, &data).expect("save");

        let due = storage.get_due_jobs(
            12,
            30,
            hone_core::beijing_now().weekday().num_days_from_monday(),
            &["feishu"],
        );
        assert!(due.is_empty());
    }

    #[test]
    fn add_job_rejects_prompt_schedule_time_mismatch() {
        let dir = make_temp_dir("hone_cron_storage_prompt_mismatch_add");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_real", None);

        let result = storage.add_job(
            &actor,
            "美股盘后AI及高景气产业链推演",
            Some(8),
            Some(30),
            "trading_day",
            "【触发时间】每个交易日 20:45（交易日）\n请执行复盘。",
            "ou_real",
            None,
            None,
            None,
            true,
            None,
            false,
        );

        assert_eq!(result["success"], false);
        assert!(
            result["error"]
                .as_str()
                .unwrap_or_default()
                .contains("与结构化 schedule 08:30 不一致")
        );
    }

    #[test]
    fn due_jobs_skip_existing_prompt_schedule_time_mismatch() {
        let dir = make_temp_dir("hone_cron_storage_prompt_mismatch_due");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_real", None);
        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
        let hour = now_bj.hour() as u32;
        let minute = now_bj.minute() as u32;

        let data = CronJobData {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            jobs: vec![CronJob {
                id: "j_mismatch".to_string(),
                name: "错配任务".to_string(),
                schedule: CronSchedule {
                    hour,
                    minute,
                    repeat: "daily".to_string(),
                    weekday: None,
                    date: None,
                },
                task_prompt: "【触发时间】每天 20:45\n执行任务".to_string(),
                push: serde_json::json!({"type": "analysis"}),
                enabled: true,
                channel: "feishu".to_string(),
                channel_scope: None,
                channel_target: actor.user_id.clone(),
                tags: Vec::new(),
                created_at: None,
                last_run_at: None,
                bypass_quiet_hours: false,
            }],
            pending_updates: Vec::new(),
        };
        storage.save_jobs(&actor, &data).expect("save");

        let due = storage.get_due_jobs(
            hour as i32,
            minute as i32,
            now_bj.weekday().num_days_from_monday(),
            &["feishu"],
        );
        assert!(due.is_empty());
    }

    #[test]
    fn once_jobs_with_future_date_do_not_run_today() {
        let dir = make_temp_dir("hone_cron_storage_once_date");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_once", None);
        let today = hone_core::beijing_now().date_naive();
        let tomorrow = today + chrono::Duration::days(1);

        let add = storage.add_job(
            &actor,
            "future once",
            Some(8),
            Some(30),
            "once",
            "task",
            "ou_once",
            None,
            Some(tomorrow.format("%Y-%m-%d").to_string()),
            None,
            true,
            None,
            false,
        );
        assert_eq!(add["success"], true);

        let due = storage.get_due_jobs(12, 0, today.weekday().num_days_from_monday(), &["feishu"]);
        assert!(
            due.is_empty(),
            "future one-shot job must not be catch-up executed today"
        );
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

    #[test]
    fn execution_terminal_event_updates_matching_pending_row() {
        let dir = make_temp_dir("hone_cron_storage_exec_update_pending");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_exec_update", None);

        let add = storage.add_job(
            &actor,
            "daily report",
            Some(9),
            Some(0),
            "daily",
            "task",
            "ou_exec_update",
            None,
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
                "ou_exec_update",
                false,
                CronJobExecutionInput {
                    execution_status: "running".to_string(),
                    message_send_status: "pending".to_string(),
                    should_deliver: false,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({"phase": "started", "delivery_key": "k-1"}),
                },
            )
            .expect("record started");

        storage
            .record_execution_event(
                &actor,
                &job_id,
                "daily report",
                "ou_exec_update",
                false,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "sent".to_string(),
                    should_deliver: true,
                    delivered: true,
                    response_preview: Some("final report".to_string()),
                    error_message: None,
                    detail: serde_json::json!({"phase": "terminal", "delivery_key": "k-1"}),
                },
            )
            .expect("record terminal");

        let records = storage
            .list_execution_records(&job_id, 10)
            .expect("list execution records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].execution_status, "completed");
        assert_eq!(records[0].message_send_status, "sent");
        assert!(records[0].delivered);
        assert_eq!(records[0].response_preview.as_deref(), Some("final report"));
        assert_eq!(records[0].detail["phase"], "terminal");
    }

    /// Reproduce production sequence: 12 heartbeat jobs, two consecutive 30-min
    /// windows each. Every (job, window) pair writes a started row then a noop
    /// terminal — production observes started rows persisting as
    /// `running + pending` across windows, so verify no orphan started rows
    /// remain after both windows finish.
    #[test]
    fn heartbeat_started_rows_finalize_across_two_windows() {
        let dir = make_temp_dir("hone_cron_storage_heartbeat_two_windows");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_heartbeat", None);

        let job_names = [
            "持仓重大事件心跳检测",
            "TEM破位预警",
            "CAI破位预警",
            "ORCL 大事件监控",
            "ASTS 重大异动心跳监控",
            "Monitor_Watchlist_11",
            "RKLB异动监控",
            "全天原油价格3小时播报",
            "小米30港元破位预警",
            "Cerebras IPO与业务进展心跳监控",
            "TEM大事件心跳监控",
            "小米破位预警",
        ];
        let windows = ["2026-04-28:15:30:heartbeat", "2026-04-28:16:00:heartbeat"];

        for window in &windows {
            for (idx, job_name) in job_names.iter().enumerate() {
                let job_id = format!("j_{idx:08x}");
                let delivery_key = format!("{job_id}:{window}");
                storage
                    .record_execution_event(
                        &actor,
                        &job_id,
                        job_name,
                        &actor.user_id,
                        true,
                        CronJobExecutionInput {
                            execution_status: "running".to_string(),
                            message_send_status: "pending".to_string(),
                            should_deliver: true,
                            delivered: false,
                            response_preview: None,
                            error_message: None,
                            detail: serde_json::json!({
                                "delivery_key": delivery_key,
                                "phase": "started",
                            }),
                        },
                    )
                    .expect("record started");

                storage
                    .record_execution_event(
                        &actor,
                        &job_id,
                        job_name,
                        &actor.user_id,
                        true,
                        CronJobExecutionInput {
                            execution_status: "noop".to_string(),
                            message_send_status: "skipped_noop".to_string(),
                            should_deliver: false,
                            delivered: false,
                            response_preview: None,
                            error_message: None,
                            detail: serde_json::json!({
                                "delivery_key": delivery_key,
                                "heartbeat_model": "model-x",
                                "parse_kind": "Empty",
                            }),
                        },
                    )
                    .expect("record terminal");
            }
        }

        let conn = rusqlite::Connection::open(&sqlite_path).expect("open conn");
        let stuck: i64 = conn
            .query_row(
                "SELECT count(*) FROM cron_job_runs WHERE execution_status='running' AND message_send_status='pending'",
                [],
                |row| row.get(0),
            )
            .expect("count stuck");
        assert_eq!(
            stuck, 0,
            "no started row should remain running+pending after terminal noop"
        );

        let total: i64 = conn
            .query_row("SELECT count(*) FROM cron_job_runs", [], |row| row.get(0))
            .expect("count total");
        let expected = (job_names.len() * windows.len()) as i64;
        assert_eq!(
            total, expected,
            "exactly one row per (job, window) should remain"
        );
    }

    /// Reproduce a Feishu-style terminal where `result.metadata` is wrapped via
    /// `execution_detail_with_delivery_key`, producing a detail object with
    /// `delivery_key` at top level, plus a `scheduler` sub-object — matches the
    /// real production payload exactly.
    #[test]
    fn heartbeat_started_rows_finalize_with_scheduler_metadata_wrapper() {
        let dir = make_temp_dir("hone_cron_storage_heartbeat_scheduler_wrap");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_heartbeat_wrap", None);

        let job_id = "j_db12f27f";
        let delivery_key = "j_db12f27f:2026-04-30:13:00:heartbeat";

        storage
            .record_execution_event(
                &actor,
                job_id,
                "RKLB异动监控",
                &actor.user_id,
                true,
                CronJobExecutionInput {
                    execution_status: "running".to_string(),
                    message_send_status: "pending".to_string(),
                    should_deliver: true,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({
                        "delivery_key": delivery_key,
                        "phase": "started",
                    }),
                },
            )
            .expect("record started");

        let terminal_detail = serde_json::json!({
            "delivery_key": delivery_key,
            "receive_id": "ou_heartbeat_wrap",
            "scheduler": {
                "heartbeat_model": "model-x",
                "parse_kind": "JsonTriggered",
                "raw_chars": 312,
                "starts_with_json": true,
                "raw_preview": "{\"status\":\"triggered\"}",
                "deliver_preview": "RKLB 触发提醒",
            },
        });
        storage
            .record_execution_event(
                &actor,
                job_id,
                "RKLB异动监控",
                &actor.user_id,
                true,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "sent".to_string(),
                    should_deliver: true,
                    delivered: true,
                    response_preview: Some("RKLB 触发提醒".to_string()),
                    error_message: None,
                    detail: terminal_detail,
                },
            )
            .expect("record terminal");

        let records = storage.list_execution_records(job_id, 10).expect("list");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].execution_status, "completed");
        assert_eq!(records[0].message_send_status, "sent");
        assert!(records[0].delivered);
    }

    #[test]
    fn execution_terminal_event_falls_back_to_recent_started_row() {
        let dir = make_temp_dir("hone_cron_storage_exec_update_recent_started");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_exec_update_fallback", None);

        let add = storage.add_job(
            &actor,
            "heartbeat",
            Some(9),
            Some(0),
            "heartbeat",
            "task",
            "ou_exec_update_fallback",
            None,
            None,
            None,
            true,
            None,
            true,
        );
        let job_id = add["job"]["id"].as_str().unwrap_or_default().to_string();

        storage
            .record_execution_event(
                &actor,
                &job_id,
                "heartbeat",
                "ou_exec_update_fallback",
                true,
                CronJobExecutionInput {
                    execution_status: "running".to_string(),
                    message_send_status: "pending".to_string(),
                    should_deliver: true,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({"phase": "started", "delivery_key": "k-recent"}),
                },
            )
            .expect("record started");

        storage
            .record_execution_event(
                &actor,
                &job_id,
                "heartbeat",
                "ou_exec_update_fallback",
                true,
                CronJobExecutionInput {
                    execution_status: "noop".to_string(),
                    message_send_status: "skipped_noop".to_string(),
                    should_deliver: false,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({"phase": "terminal", "delivery_key": null}),
                },
            )
            .expect("record terminal");

        let records = storage
            .list_execution_records(&job_id, 10)
            .expect("list execution records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].execution_status, "noop");
        assert_eq!(records[0].message_send_status, "skipped_noop");
        assert_eq!(records[0].detail["phase"], "terminal");
    }

    /// Reproduce the v0.5.0 terminal write that handed raw heartbeat
    /// diagnostics to storage without wrapping a top-level delivery_key.
    #[test]
    fn pre_fix_v0_5_0_terminal_without_delivery_key_finalizes_recent_started_row() {
        let dir = make_temp_dir("hone_cron_storage_pre_fix_terminal");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_pre_fix", None);

        let job_id = "j_654aef9b";
        let delivery_key = "j_654aef9b:2026-04-28:15:30:heartbeat";

        storage
            .record_execution_event(
                &actor,
                job_id,
                "小米30港元破位预警",
                &actor.user_id,
                true,
                CronJobExecutionInput {
                    execution_status: "running".to_string(),
                    message_send_status: "pending".to_string(),
                    should_deliver: true,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({
                        "delivery_key": delivery_key,
                        "phase": "started",
                    }),
                },
            )
            .expect("record started");

        storage
            .record_execution_event(
                &actor,
                job_id,
                "小米30港元破位预警",
                &actor.user_id,
                true,
                CronJobExecutionInput {
                    execution_status: "noop".to_string(),
                    message_send_status: "skipped_noop".to_string(),
                    should_deliver: false,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({
                        "heartbeat_model": "model-x",
                        "parse_kind": "JsonNoop",
                        "raw_chars": 18,
                        "starts_with_json": true,
                        "raw_preview": "{\"status\":\"noop\"}",
                    }),
                },
            )
            .expect("record terminal");

        let conn = rusqlite::Connection::open(&sqlite_path).expect("open conn");
        let stuck: i64 = conn
            .query_row(
                "SELECT count(*) FROM cron_job_runs WHERE execution_status='running' AND message_send_status='pending'",
                [],
                |row| row.get(0),
            )
            .expect("count stuck");
        let total: i64 = conn
            .query_row("SELECT count(*) FROM cron_job_runs", [], |row| row.get(0))
            .expect("count total");

        assert_eq!(stuck, 0);
        assert_eq!(total, 1);
    }

    /// Reproduce a legacy started row written without delivery_key in detail.
    #[test]
    fn heartbeat_started_row_without_delivery_key_is_finalized_by_recent_started_fallback() {
        let dir = make_temp_dir("hone_cron_storage_legacy_started");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("feishu", "ou_legacy", None);

        let job_id = "j_legacy";
        storage
            .record_execution_event(
                &actor,
                job_id,
                "legacy heartbeat",
                &actor.user_id,
                true,
                CronJobExecutionInput {
                    execution_status: "running".to_string(),
                    message_send_status: "pending".to_string(),
                    should_deliver: true,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({"phase": "started"}),
                },
            )
            .expect("record legacy started");

        storage
            .record_execution_event(
                &actor,
                job_id,
                "legacy heartbeat",
                &actor.user_id,
                true,
                CronJobExecutionInput {
                    execution_status: "noop".to_string(),
                    message_send_status: "skipped_noop".to_string(),
                    should_deliver: false,
                    delivered: false,
                    response_preview: None,
                    error_message: None,
                    detail: serde_json::json!({
                        "delivery_key": "j_legacy:2026-04-30:13:00:heartbeat",
                        "heartbeat_model": "model-x",
                    }),
                },
            )
            .expect("record terminal");

        let conn = rusqlite::Connection::open(&sqlite_path).expect("open conn");
        let stuck: i64 = conn
            .query_row(
                "SELECT count(*) FROM cron_job_runs WHERE execution_status='running' AND message_send_status='pending'",
                [],
                |row| row.get(0),
            )
            .expect("count stuck");
        let total: i64 = conn
            .query_row("SELECT count(*) FROM cron_job_runs", [], |row| row.get(0))
            .expect("count total");

        assert_eq!(stuck, 0);
        assert_eq!(total, 1);
    }
}
