//! 定时任务存储 — 本地 JSON/SQLite 或 cloud PG 执行记录
//!
//! 管理按 actor（channel + user_id + channel_scope）隔离的定时任务持久化存储。
//!
//! 子模块布局：
//! - [`types`]  —— 纯数据结构、错误与常量
//! - [`schedule`] —— 触发时间 / 日历 / 节假日计算
//! - [`storage`] —— `CronJobStorage` 的 JSON CRUD 与 `get_due_jobs`
//! - [`history`] —— `CronJobStorage` 的 SQLite 执行历史读写

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::cloud_sync::{ensure_cloud_schema_once, run_cloud_sync};
use tracing::warn;

pub mod history;
pub mod schedule;
pub mod storage;
pub mod types;

pub use history::ExecutionFilter;
pub use types::{
    ChannelTargetRecord, CronJob, CronJobData, CronJobExecutionInput, CronJobExecutionRecord,
    CronJobUpdate, CronSchedule, MAX_ENABLED_JOBS_PER_ACTOR, PendingUpdate, WebPushMessage,
    WebPushMessageInput, cron_enabled_limit_error, is_cron_enabled_limit_error,
};

/// 定时任务存储管理器
pub struct CronJobStorage {
    pub(super) data_dir: PathBuf,
    pub(super) sqlite_path: Option<PathBuf>,
    pub(super) postgres: Option<CloudPgRuntime>,
    pub(super) task_runs_dir: Option<PathBuf>,
}

const DEFAULT_CLOUD_CRON_TIMEOUT_SECS: u64 = 15;

impl CronJobStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).ok();
        Self {
            data_dir,
            sqlite_path: None,
            postgres: None,
            task_runs_dir: None,
        }
    }

    pub fn with_sqlite(data_dir: impl AsRef<Path>, sqlite_path: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).ok();
        let storage = Self {
            data_dir,
            sqlite_path: Some(sqlite_path.as_ref().to_path_buf()),
            postgres: None,
            task_runs_dir: None,
        };
        if let Err(err) = storage.init_execution_schema() {
            warn!("failed to initialize cron execution sqlite schema: {err}");
        }
        storage
    }

    pub fn new_cloud(postgres: CloudPgRuntime) -> hone_core::HoneResult<Self> {
        // `ensure_schema` 是 ~430 行 DDL(26 个 CREATE TABLE + 17 个 CREATE INDEX
        // + 若干 ALTER 迁移)。而 `cron_job_storage()` 是**每个调度事件**都会调一次的
        // ——此前等于每条定时任务都把全库 schema 重跑一遍。
        //
        // schema 在一个进程生命周期内不会变(换版本 = 换进程 = 会重跑),所以只需
        // 首次成功后就跳过。失败不置位,下次仍会重试。竞态下重复跑一次无害:DDL 幂等。
        ensure_cloud_schema_once(postgres.clone(), Some(cloud_cron_operation_timeout()))?;
        Ok(Self {
            data_dir: PathBuf::new(),
            sqlite_path: None,
            postgres: Some(postgres),
            task_runs_dir: None,
        })
    }

    pub(super) fn cloud_postgres(&self) -> Option<CloudPgRuntime> {
        self.postgres.clone()
    }

    pub fn with_task_runs_dir(mut self, task_runs_dir: Option<PathBuf>) -> Self {
        self.task_runs_dir = task_runs_dir;
        self
    }
}

pub(super) fn run_cloud_cron<T, F>(future: F) -> hone_core::HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = hone_core::HoneResult<T>> + Send + 'static,
{
    run_cloud_cron_with_timeout(future, cloud_cron_operation_timeout())
}

fn cloud_cron_operation_timeout() -> Duration {
    std::env::var("HONE_CLOUD_CRON_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_CLOUD_CRON_TIMEOUT_SECS))
}

fn run_cloud_cron_with_timeout<T, F>(
    future: F,
    operation_timeout: Duration,
) -> hone_core::HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = hone_core::HoneResult<T>> + Send + 'static,
{
    run_cloud_sync(future, Some(operation_timeout), "cloud cron operation")
}

#[cfg(test)]
mod tests {
    use super::schedule::beijing_slot_time;
    use super::*;
    use chrono::{Datelike, Timelike};
    use hone_core::{ActorIdentity, HoneError, beijing_offset};
    use serde_json::Value;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    /// 进程内单调计数器,保证并行测试各自拿到独立目录。
    ///
    /// 只用 `pid + nanos` 是不够的:macOS 上 `SystemTime` 的实际粒度远粗于纳秒,
    /// 两个同时启动的测试会拿到**同一个**时间戳 → 同一个目录,随后各自
    /// `remove_dir_all` 把对方的 sqlite 删掉,表现为随机的
    /// `disk I/O error`。实测基线 8 次里偶发失败 4 次。
    static TEMP_DIR_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let seq = TEMP_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{ts}_{seq}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn cloud_cron_timeout_returns_storage_error_instead_of_blocking() {
        let started = Instant::now();
        let err = run_cloud_cron_with_timeout(
            async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok::<(), HoneError>(())
            },
            Duration::from_millis(20),
        )
        .expect_err("cloud cron bridge should time out");

        assert!(
            started.elapsed() < Duration::from_secs(1),
            "timeout should bound a stuck cloud cron operation"
        );
        let message = err.to_string();
        assert!(
            message.contains("cloud cron operation timed out"),
            "unexpected error: {message}"
        );
    }

    /// 同步桥在 **tokio 上下文内**也必须正常返回结果与超时。
    ///
    /// 这条分支此前(`std::thread::spawn(...).join()` + 每次新建 Runtime)一行
    /// 测试都没有。现在它改成"交给长驻 runtime + 阻塞等结果",更需要被钉住:
    /// 一旦误用 `Runtime::block_on`,tokio 会 panic
    /// "Cannot start a runtime from within a runtime"。
    #[tokio::test]
    async fn cloud_cron_bridge_works_from_inside_a_tokio_context() {
        // 借 blocking 线程调用同步桥——这正是生产里的形态:
        // 同步的 `get_due_jobs` / `record_execution_event` 跑在 tokio worker 上。
        let ok = tokio::task::spawn_blocking(|| {
            run_cloud_cron_with_timeout(
                async { Ok::<u32, HoneError>(7) },
                Duration::from_millis(500),
            )
        })
        .await
        .expect("join");
        assert_eq!(ok.expect("bridge should return the value"), 7);

        let timed_out = tokio::task::spawn_blocking(|| {
            run_cloud_cron_with_timeout(
                async {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    Ok::<(), HoneError>(())
                },
                Duration::from_millis(20),
            )
        })
        .await
        .expect("join")
        .expect_err("should time out");
        assert!(
            timed_out
                .to_string()
                .contains("cloud cron operation timed out"),
            "unexpected error: {timed_out}"
        );

        // 反复调用不应再有 runtime/线程 churn:连续跑一批,全部成功即说明长驻
        // runtime 被复用(此前每次都会新建并销毁 1+N 个 OS 线程)。
        let batch = tokio::task::spawn_blocking(|| {
            (0..32)
                .map(|i| {
                    run_cloud_cron_with_timeout(
                        async move { Ok::<u32, HoneError>(i) },
                        Duration::from_millis(500),
                    )
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .await
        .expect("join")
        .expect("all bridge calls should succeed");
        assert_eq!(batch.len(), 32);
        assert_eq!(batch[31], 31);
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

    fn assert_job_result_success(result: &Value) {
        assert_eq!(
            result.get("success").and_then(Value::as_bool),
            Some(true),
            "expected cron job result to succeed: {result}"
        );
    }

    fn assert_job_result_failure(result: &Value) {
        assert_eq!(
            result.get("success").and_then(Value::as_bool),
            Some(false),
            "expected cron job result to fail: {result}"
        );
    }

    fn job_id_from_add_result(result: &Value) -> String {
        assert_job_result_success(result);
        result["job"]["id"]
            .as_str()
            .expect("successful add_job result should include job.id")
            .to_string()
    }

    fn assert_job_result_error_contains(result: &Value, expected: &str) {
        let error = result["error"].as_str().unwrap_or_default();
        assert!(
            error.contains(expected),
            "expected cron job error containing {expected:?}, got result: {result}"
        );
    }

    fn assert_job_result_error_eq(result: &Value, expected: &str) {
        assert_eq!(
            result["error"].as_str(),
            Some(expected),
            "unexpected cron job error in result: {result}"
        );
    }

    fn assert_enabled_limit_error(error: &impl std::fmt::Display) {
        assert!(
            is_cron_enabled_limit_error(&error.to_string()),
            "expected enabled-job limit error, got: {error}"
        );
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
        assert_job_result_failure(&bad_hour);

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
        assert_job_result_failure(&bad_weekly);
    }

    #[test]
    fn add_job_rejects_empty_channel_target() {
        let dir = make_temp_dir("hone_cron_storage_empty_target");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("telegram", "user_1", None);

        let result = storage.add_job(
            &actor,
            "missing target",
            Some(9),
            Some(0),
            "daily",
            "task",
            "   ",
            None,
            None,
            None,
            true,
            None,
            false,
        );

        assert_job_result_failure(&result);
        assert_job_result_error_contains(&result, "channel_target 不能为空");
        assert!(storage.list_jobs(&actor).is_empty());
    }

    #[test]
    fn channel_target_directory_aggregates_jobs_and_execution_history() {
        let dir = make_temp_dir("hone_cron_storage_target_directory");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("telegram", "user_1", Some("g:1:c:2"));

        let add = storage.add_job(
            &actor,
            "group heartbeat",
            Some(9),
            Some(0),
            "heartbeat",
            "task",
            "-100123",
            None,
            None,
            None,
            true,
            None,
            false,
        );
        let job_id = job_id_from_add_result(&add);

        storage
            .record_execution_event(
                &actor,
                &job_id,
                "group heartbeat",
                "-100123",
                true,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "sent".to_string(),
                    should_deliver: true,
                    delivered: true,
                    response_preview: Some("ok".to_string()),
                    error_message: None,
                    detail: serde_json::json!({"delivery_key": "target-directory-test"}),
                },
            )
            .expect("record execution");

        let targets = storage.list_channel_targets();
        let target = targets
            .iter()
            .find(|target| target.target == "-100123")
            .expect("target should be discoverable");
        assert_eq!(target.channel, "telegram");
        assert_eq!(target.channel_scope.as_deref(), Some("g:1:c:2"));
        assert_eq!(target.scheduled_jobs, 1);
        assert_eq!(target.enabled_jobs, 1);
        assert!(target.sources.iter().any(|source| source == "cron_job"));
        assert!(
            target
                .sources
                .iter()
                .any(|source| source == "cron_execution")
        );
        assert!(target.actor_user_ids.iter().any(|user| user == "user_1"));
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
        let job_id = job_id_from_add_result(&add);

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

        let first_add = storage.add_job(
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
        );
        assert_job_result_success(&first_add);
        let second_add = storage.add_job(
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
        );
        assert_job_result_success(&second_add);

        let first = storage.list_jobs(&actor_one);
        let second = storage.list_jobs(&actor_two);
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(first[0].name, "report one");
        assert_eq!(second[0].name, "report two");
    }

    #[test]
    fn remove_all_jobs_is_actor_scoped_clears_pending_updates_and_is_idempotent() {
        let dir = make_temp_dir("hone_cron_storage_remove_all");
        let storage = CronJobStorage::new(&dir);
        let actor_one = actor("feishu", "u1", None);
        let actor_two = actor("feishu", "u2", None);

        let first_job_id =
            job_id_from_add_result(&add_enabled_job(&storage, &actor_one, "daily report"));
        assert_job_result_success(&add_enabled_job(&storage, &actor_one, "price heartbeat"));
        assert_job_result_success(&add_enabled_job(&storage, &actor_two, "other actor report"));

        let mut actor_one_data = storage
            .try_load_jobs(&actor_one)
            .expect("load actor one jobs");
        actor_one_data.pending_updates.push(PendingUpdate {
            token: "pending-1".to_string(),
            job_id: first_job_id,
            updates: serde_json::json!({"hour": 10}),
            created_at: hone_core::beijing_now_rfc3339(),
        });
        storage
            .save_jobs(&actor_one, &actor_one_data)
            .expect("save pending update");

        let removed = storage
            .remove_all_jobs(&actor_one)
            .expect("remove all actor one jobs");
        assert_eq!(removed.len(), 2);
        let remaining_actor_one = storage.try_load_jobs(&actor_one).expect("reload actor one");
        assert!(remaining_actor_one.jobs.is_empty());
        assert!(remaining_actor_one.pending_updates.is_empty());
        assert_eq!(storage.list_jobs(&actor_two).len(), 1);

        let repeated = storage
            .remove_all_jobs(&actor_one)
            .expect("remove all should be idempotent");
        assert!(repeated.is_empty());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn remove_all_jobs_does_not_report_success_when_durable_data_is_unreadable() {
        let dir = make_temp_dir("hone_cron_storage_remove_all_corrupt");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "u1", None);
        std::fs::write(storage.get_actor_file(&actor), "{not valid json")
            .expect("write corrupt cron data");

        let error = storage
            .remove_all_jobs(&actor)
            .expect_err("corrupt durable data must fail cancellation");
        assert!(
            error.to_string().contains("expected")
                || error.to_string().contains("Serialization")
                || error.to_string().contains("序列化错误"),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn sixth_enabled_job_is_rejected_but_disabled_job_is_allowed() {
        let dir = make_temp_dir("hone_cron_storage_limit_add");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("discord", "alice", None);

        for index in 0..MAX_ENABLED_JOBS_PER_ACTOR {
            assert_job_result_success(&add_enabled_job(&storage, &actor, &format!("job-{index}")));
        }

        let rejected = add_enabled_job(&storage, &actor, "job-6");
        assert_job_result_failure(&rejected);
        assert_job_result_error_eq(&rejected, &cron_enabled_limit_error());

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
        assert_job_result_success(&disabled);
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
            job_ids.push(job_id_from_add_result(&result));
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
        let disabled_id = job_id_from_add_result(&disabled);

        let toggle_err = storage
            .toggle_job(&disabled_id, Some(&actor), false)
            .expect_err("toggle should hit limit");
        assert_enabled_limit_error(&toggle_err);

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
        assert_enabled_limit_error(&update_err);

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
        let job_id = job_id_from_add_result(&add);

        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
        // 查询分钟取到最大抖动偏移之后:heartbeat 现在按 job_id 在半点后的
        // [0, JITTER_SPREAD_MINUTES) 分钟内错峰,槽内任一分钟都算同一槽。
        let probe_minute = 30 + super::storage::JITTER_SPREAD_MINUTES - 1;
        let due_first = storage.get_due_jobs(
            10,
            probe_minute,
            now_bj.weekday().num_days_from_monday(),
            &["feishu"],
        );
        assert_eq!(due_first.len(), 1);
        assert_eq!(due_first[0].1.id, job_id);
        assert!(due_first[0].1.is_heartbeat());

        let mut data = storage.load_jobs(&actor);
        let slot_time = now_bj
            .with_hour(10)
            .and_then(|dt| dt.with_minute(probe_minute as u32))
            .and_then(|dt| dt.with_second(0))
            .expect("slot time");
        let job = data
            .jobs
            .iter_mut()
            .find(|job| job.id == job_id)
            .expect("job exists");
        job.last_run_at = Some(slot_time.to_rfc3339());
        storage.save_jobs(&actor, &data).expect("save");
        let due_second = storage.get_due_jobs(
            10,
            probe_minute,
            now_bj.weekday().num_days_from_monday(),
            &["feishu"],
        );
        assert!(due_second.is_empty());
    }

    /// heartbeat 齐射削峰:偏移必须确定性、落在 `[0, JITTER_SPREAD_MINUTES)`,
    /// 且始终留在到期容错窗口内(否则该轮任务会被整个丢掉)。
    #[test]
    fn heartbeat_dispatch_jitter_is_deterministic_and_stays_inside_due_window() {
        use super::storage::{JITTER_SPREAD_MINUTES, dispatch_jitter_minutes};

        assert!(
            JITTER_SPREAD_MINUTES < super::schedule::DUE_WINDOW_MINUTES,
            "抖动越过容错窗口会导致任务被丢弃"
        );

        let mut seen_offsets = std::collections::HashSet::new();
        for index in 0..200 {
            let job_id = format!("j_{index:06x}");
            let offset = dispatch_jitter_minutes(&job_id);
            assert!(
                (0..JITTER_SPREAD_MINUTES).contains(&offset),
                "offset {offset} out of range for {job_id}"
            );
            // 确定性:同一 job_id 必须永远得到同一偏移,否则无法复现排查。
            assert_eq!(offset, dispatch_jitter_minutes(&job_id));
            seen_offsets.insert(offset);
        }
        // 真的摊开了,而不是全挤在同一分钟。
        assert_eq!(
            seen_offsets.len(),
            JITTER_SPREAD_MINUTES as usize,
            "jitter should spread jobs across every offset minute"
        );
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
        let job_id = job_id_from_add_result(&add);

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
        let job_id = job_id_from_add_result(&add);

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

        assert_job_result_failure(&result);
        assert_job_result_error_contains(&result, "与结构化 schedule 08:30 不一致");
    }

    #[test]
    fn due_jobs_repair_existing_prompt_schedule_time_mismatch() {
        let dir = make_temp_dir("hone_cron_storage_prompt_mismatch_due");
        let storage = CronJobStorage::new(&dir);
        let actor = actor("feishu", "ou_real", None);
        let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
        let hour = now_bj.hour() as u32;
        let minute = now_bj.minute() as u32;
        let stale_hour = (hour + 1) % 24;

        let data = CronJobData {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            jobs: vec![CronJob {
                id: "j_mismatch".to_string(),
                name: "错配任务".to_string(),
                schedule: CronSchedule {
                    hour: stale_hour,
                    minute,
                    repeat: "daily".to_string(),
                    weekday: None,
                    date: None,
                },
                task_prompt: format!("【触发时间】每天 {hour:02}:{minute:02}\n执行任务"),
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
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].1.id, "j_mismatch");
        assert_eq!(
            (due[0].1.schedule.hour, due[0].1.schedule.minute),
            (hour, minute)
        );

        let saved = storage.load_jobs(&actor);
        let repaired = saved
            .jobs
            .into_iter()
            .find(|job| job.id == "j_mismatch")
            .expect("repaired job");
        assert_eq!(
            (repaired.schedule.hour, repaired.schedule.minute),
            (hour, minute)
        );
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
        assert_job_result_success(&add);

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
        let job_id = job_id_from_add_result(&add);

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
    fn discord_send_failed_without_error_is_classified_by_storage_backstop() {
        let dir = make_temp_dir("hone_cron_storage_discord_send_failed_backstop");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("discord", "g_exec", Some("channel-1"));

        storage
            .record_execution_event(
                &actor,
                "j_discord",
                "daily report",
                "channel-1",
                false,
                CronJobExecutionInput {
                    execution_status: "completed".to_string(),
                    message_send_status: "send_failed".to_string(),
                    should_deliver: true,
                    delivered: false,
                    response_preview: Some("final report".to_string()),
                    error_message: None,
                    detail: serde_json::json!({
                        "scheduler": null,
                        "sent_segments": 0,
                        "total_segments": 2,
                    }),
                },
            )
            .expect("record execution");

        let records = storage
            .list_execution_records("j_discord", 10)
            .expect("list execution records");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].error_message.as_deref(),
            Some("Discord 定时任务发送失败")
        );
        assert_eq!(records[0].detail["failure_kind"], "discord_send_failed");
        assert_eq!(records[0].detail["sent_segments"], 0);
        assert_eq!(records[0].detail["total_segments"], 2);
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
        let job_id = job_id_from_add_result(&add);

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
        // 配对 started 行存在 ⇒ 开始时刻已知,时延可算。
        assert!(
            records[0].started_at.is_some(),
            "paired started row should keep its start timestamp"
        );
        assert!(
            records[0].duration_ms.is_some(),
            "duration must be derivable when the start timestamp is known"
        );
    }

    /// 没有配对 started 行的终态记录:开始时刻未知,必须留 NULL,
    /// 不能回填成 executed_at —— 那会谎报 0 毫秒并污染时延统计。
    #[test]
    fn terminal_only_execution_leaves_start_and_duration_unknown() {
        let dir = make_temp_dir("hone_cron_storage_exec_terminal_only");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let actor = actor("web", "web-user-terminal-only", None);

        storage
            .record_execution_event(
                &actor,
                "j_terminal_only",
                "target missing job",
                "web-user-terminal-only",
                false,
                CronJobExecutionInput {
                    execution_status: "execution_failed".to_string(),
                    message_send_status: "skipped_error".to_string(),
                    should_deliver: false,
                    delivered: false,
                    response_preview: None,
                    error_message: Some("channel target missing".to_string()),
                    detail: serde_json::json!({"phase": "terminal"}),
                },
            )
            .expect("record terminal without a started row");

        let records = storage
            .list_execution_records("j_terminal_only", 10)
            .expect("list execution records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].execution_status, "execution_failed");
        assert_eq!(
            records[0].started_at, None,
            "unpaired terminal row must not claim a start timestamp"
        );
        assert_eq!(
            records[0].duration_ms, None,
            "unknown start must yield unknown duration, not 0"
        );
    }

    #[test]
    fn started_execution_can_be_failed_by_exact_delivery_key_watchdog() {
        let dir = make_temp_dir("hone_cron_storage_watchdog_pending");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let target_actor = actor("feishu", "ou_watchdog", None);
        let other_actor = actor("feishu", "ou_watchdog_other", None);

        for (actor, job_id, delivery_key) in [
            (&target_actor, "j_watchdog", "delivery-watchdog"),
            (&target_actor, "j_other_key", "delivery-other"),
            (&other_actor, "j_watchdog", "delivery-watchdog"),
        ] {
            storage
                .record_execution_event(
                    actor,
                    job_id,
                    "pending job",
                    &actor.user_id,
                    false,
                    CronJobExecutionInput {
                        execution_status: "running".to_string(),
                        message_send_status: "pending".to_string(),
                        should_deliver: true,
                        delivered: false,
                        response_preview: None,
                        error_message: None,
                        detail: serde_json::json!({
                            "phase": "started",
                            "delivery_key": delivery_key,
                        }),
                    },
                )
                .expect("record started");
        }

        let updated = storage
            .mark_started_execution_failed_by_delivery_key(
                &target_actor,
                "j_watchdog",
                &target_actor.user_id,
                false,
                "delivery-watchdog",
                "feishu_scheduler_handler_watchdog",
                "scheduler_handler_watchdog_timeout:1235s",
            )
            .expect("watchdog finalize");
        assert_eq!(updated, 1);

        let second = storage
            .mark_started_execution_failed_by_delivery_key(
                &target_actor,
                "j_watchdog",
                &target_actor.user_id,
                false,
                "delivery-watchdog",
                "feishu_scheduler_handler_watchdog",
                "scheduler_handler_watchdog_timeout:1235s",
            )
            .expect("watchdog finalize is idempotent");
        assert_eq!(second, 0);

        let finalized = storage
            .list_execution_records("j_watchdog", 10)
            .expect("list finalized");
        assert_eq!(finalized.len(), 2);
        let target = finalized
            .iter()
            .find(|record| record.user_id == target_actor.user_id)
            .expect("target record");
        assert_eq!(target.execution_status, "execution_failed");
        assert_eq!(target.message_send_status, "skipped_error");
        assert!(!target.should_deliver);
        assert!(!target.delivered);
        assert_eq!(
            target.error_message.as_deref(),
            Some("scheduler_handler_watchdog_timeout:1235s")
        );
        assert_eq!(target.detail["phase"], "scheduler_handler_watchdog_timeout");
        assert_eq!(
            target.detail["recovered_by"],
            "feishu_scheduler_handler_watchdog"
        );

        let other_key = storage
            .list_execution_records("j_other_key", 10)
            .expect("list other key");
        assert_eq!(other_key[0].execution_status, "running");
        assert_eq!(other_key[0].message_send_status, "pending");

        let other_actor_records = finalized
            .iter()
            .find(|record| record.user_id == other_actor.user_id)
            .expect("other actor record");
        assert_eq!(other_actor_records.execution_status, "running");
        assert_eq!(other_actor_records.message_send_status, "pending");
    }

    #[test]
    fn stale_started_rows_can_be_recovered_as_failed() {
        let dir = make_temp_dir("hone_cron_storage_interrupted_pending");
        let sqlite_path = dir.join("sessions.sqlite3");
        let storage = CronJobStorage::with_sqlite(&dir, &sqlite_path);
        let feishu_actor = actor("feishu", "ou_interrupted", None);
        let discord_actor = actor("discord", "du_interrupted", None);

        for (actor, job_id, channel_target, phase) in [
            (
                &feishu_actor,
                "j_feishu_started",
                "ou_interrupted",
                "started",
            ),
            (
                &feishu_actor,
                "j_feishu_other",
                "ou_interrupted",
                "progress",
            ),
            (
                &discord_actor,
                "j_discord_started",
                "du_interrupted",
                "started",
            ),
        ] {
            storage
                .record_execution_event(
                    actor,
                    job_id,
                    "pending job",
                    channel_target,
                    false,
                    CronJobExecutionInput {
                        execution_status: "running".to_string(),
                        message_send_status: "pending".to_string(),
                        should_deliver: true,
                        delivered: false,
                        response_preview: None,
                        error_message: None,
                        detail: serde_json::json!({
                            "phase": phase,
                            "delivery_key": format!("{job_id}:2026-05-08:08:00"),
                        }),
                    },
                )
                .expect("record pending");
        }

        let conn = rusqlite::Connection::open(&sqlite_path).expect("open conn");
        conn.execute(
            "UPDATE cron_job_runs SET executed_at = ?1 WHERE job_id = ?2",
            rusqlite::params!["2026-05-07T20:30:00+08:00", "j_feishu_started"],
        )
        .expect("make started row stale");

        let updated = storage
            .recover_stale_started_executions(
                "feishu",
                "2026-05-07T20:45:00+08:00",
                "feishu_scheduler_startup",
                "Feishu scheduler runtime restarted before this run reached a terminal status",
            )
            .expect("finalize pending");
        assert_eq!(updated, 1);

        let finalized = storage
            .list_execution_records("j_feishu_started", 10)
            .expect("list finalized");
        assert_eq!(finalized.len(), 1);
        assert_eq!(finalized[0].execution_status, "execution_failed");
        assert_eq!(finalized[0].message_send_status, "send_failed");
        assert!(!finalized[0].should_deliver);
        assert!(!finalized[0].delivered);
        assert_eq!(finalized[0].detail["phase"], "recovered_stale_pending");
        assert_eq!(
            finalized[0].detail["delivery_key"],
            "j_feishu_started:2026-05-08:08:00"
        );
        assert_eq!(
            finalized[0].detail["recovered_by"],
            "feishu_scheduler_startup"
        );
        assert!(
            finalized[0]
                .error_message
                .as_deref()
                .unwrap_or_default()
                .contains("runtime restarted")
        );

        let feishu_other = storage
            .list_execution_records("j_feishu_other", 10)
            .expect("list feishu other");
        assert_eq!(feishu_other[0].execution_status, "running");
        assert_eq!(feishu_other[0].message_send_status, "pending");

        let discord = storage
            .list_execution_records("j_discord_started", 10)
            .expect("list discord");
        assert_eq!(discord[0].execution_status, "running");
        assert_eq!(discord[0].message_send_status, "pending");
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
        let job_id = job_id_from_add_result(&add);

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
