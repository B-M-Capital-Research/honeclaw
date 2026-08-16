//! Cron 执行历史的 PostgreSQL 持久化。
//!
//! 定时任务定义、执行历史与 Web push 消息都使用同一个 PostgreSQL runtime。

use hone_core::cloud_runtime::{CloudCronExecutionFilter, CloudCronExecutionInput};
use hone_core::{ActorIdentity, HoneResult, truncate_chars_append};
use serde_json::Value;

use super::CronJobStorage;
use super::run_cloud_cron;
use super::types::{
    CronJobExecutionInput, CronJobExecutionRecord, WebPushMessage, WebPushMessageInput,
};

#[derive(Debug, Clone)]
struct CronTaskObservation {
    task: String,
    started_at: chrono::DateTime<chrono::Utc>,
    outcome: hone_core::TaskOutcome,
    error: Option<String>,
}

/// 跨任务列举执行记录的过滤条件。所有时间字段使用东八区 RFC3339 字符串
/// (与 `cron_job_runs.executed_at` 的写入格式一致),按字符串比较即可。
#[derive(Debug, Default, Clone)]
pub struct ExecutionFilter {
    pub since: Option<String>,
    pub until: Option<String>,
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub job_id: Option<String>,
    pub execution_status: Option<String>,
    pub message_send_status: Option<String>,
    pub heartbeat_only: Option<bool>,
    pub limit: usize,
}

impl CronJobStorage {
    pub fn mark_started_execution_failed_by_delivery_key(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
        channel_target: &str,
        heartbeat: bool,
        delivery_key: &str,
        recovered_by: &str,
        reason: &str,
    ) -> HoneResult<usize> {
        let delivery_key = delivery_key.trim();
        if delivery_key.is_empty() {
            return Ok(0);
        }

        let postgres = self.postgres.clone();
        let actor = actor.clone();
        let job_id = job_id.to_string();
        let channel_target = channel_target.to_string();
        let delivery_key = delivery_key.to_string();
        let recovered_by = recovered_by.to_string();
        let reason = truncate_chars_append(reason, 500, "...");
        return run_cloud_cron(async move {
            postgres
                .mark_cron_started_execution_failed_by_delivery_key(
                    &actor,
                    &job_id,
                    &channel_target,
                    heartbeat,
                    &delivery_key,
                    &recovered_by,
                    &reason,
                )
                .await
        });
    }

    pub fn recover_stale_started_executions(
        &self,
        channel: &str,
        stale_before_rfc3339: &str,
        recovered_by: &str,
        reason: &str,
    ) -> HoneResult<usize> {
        let postgres = self.postgres.clone();
        let channel = channel.to_string();
        let stale_before_rfc3339 = stale_before_rfc3339.to_string();
        let recovered_by = recovered_by.to_string();
        let reason = truncate_chars_append(reason, 500, "...");
        return run_cloud_cron(async move {
            postgres
                .recover_stale_cron_started_executions(
                    &channel,
                    &stale_before_rfc3339,
                    &recovered_by,
                    &reason,
                )
                .await
        });
    }

    pub fn record_execution_event(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
        job_name: &str,
        channel_target: &str,
        heartbeat: bool,
        input: CronJobExecutionInput,
    ) -> HoneResult<()> {
        let input = normalize_cron_execution_input_for_storage(actor, input);
        let observation = cron_task_observation(actor, job_name, heartbeat, &input);

        let postgres = self.postgres.clone();
        let actor = actor.clone();
        let job_id = job_id.to_string();
        let job_name = job_name.to_string();
        let channel_target = channel_target.to_string();
        let cloud_input = CloudCronExecutionInput {
            execution_status: input.execution_status,
            message_send_status: input.message_send_status,
            should_deliver: input.should_deliver,
            delivered: input.delivered,
            response_preview: input
                .response_preview
                .as_deref()
                .map(|text| truncate_chars_append(text, 500, "...")),
            error_message: input
                .error_message
                .as_deref()
                .map(|text| truncate_chars_append(text, 500, "...")),
            detail: input.detail,
        };
        let result = run_cloud_cron(async move {
            postgres
                .record_cron_execution_event(
                    &actor,
                    &job_id,
                    &job_name,
                    &channel_target,
                    heartbeat,
                    cloud_input,
                )
                .await
        });
        if result.is_ok() {
            self.record_cron_task_observation(observation.as_ref());
        }
        return result;
    }

    fn record_cron_task_observation(&self, observation: Option<&CronTaskObservation>) {
        let (Some(runtime_dir), Some(observation)) = (self.task_runs_dir.as_deref(), observation)
        else {
            return;
        };
        match observation.outcome {
            hone_core::TaskOutcome::Ok => hone_core::task_observer::record_ok(
                runtime_dir,
                &observation.task,
                observation.started_at,
                1,
            ),
            hone_core::TaskOutcome::Skipped => hone_core::task_observer::record_skipped(
                runtime_dir,
                &observation.task,
                observation.started_at,
            ),
            hone_core::TaskOutcome::Failed => hone_core::task_observer::record_failed(
                runtime_dir,
                &observation.task,
                observation.started_at,
                observation.error.as_deref().unwrap_or("cron task failed"),
            ),
        }
    }

    pub fn list_execution_records(
        &self,
        job_id: &str,
        limit: usize,
    ) -> HoneResult<Vec<CronJobExecutionRecord>> {
        let postgres = self.postgres.clone();
        let filter = CloudCronExecutionFilter {
            job_id: Some(job_id.to_string()),
            limit,
            ..CloudCronExecutionFilter::default()
        };
        return run_cloud_cron(async move { postgres.list_cron_execution_records(filter).await })
            .map(|records| records.into_iter().map(cron_execution_from_cloud).collect());
    }

    /// 跨任务查询执行记录,用于管理端"推送日志"页。filter 中的所有字段都是
    /// `AND` 连接;`limit` 必须 > 0,调用方负责裁剪到合理上限。
    pub fn list_recent_executions(
        &self,
        filter: &ExecutionFilter,
    ) -> HoneResult<Vec<CronJobExecutionRecord>> {
        let postgres = self.postgres.clone();
        let filter = CloudCronExecutionFilter {
            since: filter.since.clone(),
            until: filter.until.clone(),
            channel: filter.channel.clone(),
            user_id: filter.user_id.clone(),
            job_id: filter.job_id.clone(),
            execution_status: filter.execution_status.clone(),
            message_send_status: filter.message_send_status.clone(),
            heartbeat_only: filter.heartbeat_only,
            limit: filter.limit,
        };
        return run_cloud_cron(async move { postgres.list_cron_execution_records(filter).await })
            .map(|records| records.into_iter().map(cron_execution_from_cloud).collect());
    }

    pub fn upsert_web_push_message(
        &self,
        actor: &ActorIdentity,
        input: WebPushMessageInput,
    ) -> HoneResult<WebPushMessage> {
        let postgres = self.postgres.clone();
        let actor = actor.clone();
        return run_cloud_cron(async move {
            postgres
                .upsert_web_push_message(
                    &actor,
                    &input.push_id,
                    &input.job_id,
                    &input.job_name,
                    &input.summary,
                    &input.content,
                    &input.created_at,
                )
                .await
        })
        .map(web_push_from_cloud);
    }

    pub fn upsert_web_push_messages(
        &self,
        actor: &ActorIdentity,
        inputs: Vec<WebPushMessageInput>,
    ) -> HoneResult<usize> {
        if inputs.is_empty() {
            return Ok(0);
        }

        let postgres = self.postgres.clone();
        let actor = actor.clone();
        let actor_storage_key = actor.storage_key();
        let messages = inputs
            .into_iter()
            .map(|input| hone_core::cloud_runtime::CloudWebPushMessage {
                push_id: input.push_id,
                actor_storage_key: actor_storage_key.clone(),
                job_id: input.job_id,
                job_name: input.job_name,
                summary: input.summary,
                content: input.content,
                created_at: input.created_at,
                read_at: None,
            })
            .collect();
        return run_cloud_cron(
            async move { postgres.upsert_web_push_messages(&actor, messages).await },
        );
    }

    pub fn has_legacy_web_push_messages(&self, actor: &ActorIdentity) -> HoneResult<bool> {
        let postgres = self.postgres.clone();
        let actor = actor.clone();
        return run_cloud_cron(async move { postgres.has_legacy_web_push_messages(&actor).await });
    }

    pub fn list_web_push_messages(
        &self,
        actor: &ActorIdentity,
        before_push_id: Option<&str>,
        limit: usize,
    ) -> HoneResult<Vec<WebPushMessage>> {
        let postgres = self.postgres.clone();
        let actor = actor.clone();
        let before_push_id = before_push_id.map(str::to_string);
        return run_cloud_cron(async move {
            postgres
                .list_web_push_messages(&actor, before_push_id, limit)
                .await
        })
        .map(|records| records.into_iter().map(web_push_from_cloud).collect());
    }

    pub fn get_web_push_message(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
    ) -> HoneResult<Option<WebPushMessage>> {
        let postgres = self.postgres.clone();
        let actor = actor.clone();
        let push_id = push_id.to_string();
        return run_cloud_cron(
            async move { postgres.get_web_push_message(&actor, &push_id).await },
        )
        .map(|record| record.map(web_push_from_cloud));
    }

    pub fn count_unread_web_push_messages(&self, actor: &ActorIdentity) -> HoneResult<usize> {
        let postgres = self.postgres.clone();
        let actor = actor.clone();
        return run_cloud_cron(
            async move { postgres.count_unread_web_push_messages(&actor).await },
        );
    }

    pub fn mark_web_push_messages_read_through(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
    ) -> HoneResult<usize> {
        let read_at = hone_core::local_now_rfc3339();

        let postgres = self.postgres.clone();
        let actor = actor.clone();
        let push_id = push_id.to_string();
        return run_cloud_cron(async move {
            postgres
                .mark_web_push_messages_read_through(&actor, &push_id, &read_at)
                .await
        });
    }
}

fn normalize_cron_execution_input_for_storage(
    actor: &ActorIdentity,
    mut input: CronJobExecutionInput,
) -> CronJobExecutionInput {
    if input.message_send_status != "send_failed" || input.delivered {
        return input;
    }

    let sent_segments = input
        .detail
        .get("sent_segments")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let total_segments = input
        .detail
        .get("total_segments")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    if sent_segments != 0 || total_segments == 0 {
        return input;
    }

    let fallback_error = match actor.channel.as_str() {
        "discord" => "Discord 定时任务发送失败",
        _ => "定时任务发送失败",
    };
    if input
        .error_message
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        input.error_message = Some(fallback_error.to_string());
    }

    if let Value::Object(detail) = &mut input.detail
        && detail
            .get("failure_kind")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        let failure_kind = match actor.channel.as_str() {
            "discord" => "discord_send_failed",
            "feishu" => "feishu_send_failed",
            "telegram" => "telegram_send_failed",
            "web" => "web_send_failed",
            _ => "channel_send_failed",
        };
        detail.insert(
            "failure_kind".to_string(),
            Value::String(failure_kind.to_string()),
        );
    }

    input
}

fn cron_task_observation(
    actor: &ActorIdentity,
    job_name: &str,
    heartbeat: bool,
    input: &CronJobExecutionInput,
) -> Option<CronTaskObservation> {
    if input.execution_status == "running" && input.message_send_status == "pending" {
        return None;
    }

    let explicitly_skipped = matches!(
        input.message_send_status.as_str(),
        "skipped_noop" | "skipped_cancelled" | "duplicate_suppressed"
    ) || input.execution_status == "noop";
    let failed = input.execution_status == "execution_failed"
        || matches!(
            input.message_send_status.as_str(),
            "send_failed" | "skipped_error" | "target_missing" | "target_resolution_failed"
        );
    let outcome = if failed {
        hone_core::TaskOutcome::Failed
    } else if explicitly_skipped {
        hone_core::TaskOutcome::Skipped
    } else {
        hone_core::TaskOutcome::Ok
    };
    let error = (outcome == hone_core::TaskOutcome::Failed).then(|| {
        input
            .error_message
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "job={job_name} execution_status={} message_send_status={}",
                    input.execution_status, input.message_send_status
                )
            })
    });
    Some(CronTaskObservation {
        task: format!(
            "cron.{}.{}",
            actor.channel,
            if heartbeat { "heartbeat" } else { "scheduled" }
        ),
        // task_runs 的 `started_at` 在读取侧只当"这条记录发生的时刻"用
        // (`routes/task_runs.rs` 拿它做 24h 窗口过滤、排序、last_seen_at /
        // last_failure_at),不用来算时延。所以这里取终态落账时刻即可,不必为了
        // 补一个"真实开始时刻"再去查一次 started 行。
        // 单任务的真实耗时另有去处:`cron_job_runs.duration_ms`(由 SQL 用
        // started_at 与 executed_at 相减算出,未知时为 NULL)。
        started_at: chrono::Utc::now(),
        outcome,
        error,
    })
}

fn cron_execution_from_cloud(
    record: hone_core::cloud_runtime::CloudCronExecutionRecord,
) -> CronJobExecutionRecord {
    CronJobExecutionRecord {
        run_id: record.run_id,
        job_id: record.job_id,
        job_name: record.job_name,
        channel: record.channel,
        user_id: record.user_id,
        channel_scope: record.channel_scope,
        channel_target: record.channel_target,
        heartbeat: record.heartbeat,
        started_at: record.started_at,
        executed_at: record.executed_at,
        duration_ms: record.duration_ms,
        execution_status: record.execution_status,
        message_send_status: record.message_send_status,
        should_deliver: record.should_deliver,
        delivered: record.delivered,
        response_preview: record.response_preview,
        error_message: record.error_message,
        detail: record.detail,
    }
}

fn web_push_from_cloud(record: hone_core::cloud_runtime::CloudWebPushMessage) -> WebPushMessage {
    WebPushMessage {
        push_id: record.push_id,
        actor_storage_key: record.actor_storage_key,
        job_id: record.job_id,
        job_name: record.job_name,
        summary: record.summary,
        content: record.content,
        created_at: record.created_at,
        read_at: record.read_at,
    }
}

#[cfg(test)]
mod web_push_tests {
    use super::*;

    /// 进程内单调计数器。理由同 `cron_job::tests::make_temp_dir`:只靠
    /// `pid + nanos` 在 macOS 的时钟粒度下会让并行测试撞同一个目录,
    /// 随后互相 `remove_dir_all` 造成随机 `disk I/O error`。
    static TEMP_DIR_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn test_storage() -> (CronJobStorage, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "hone_web_push_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos(),
            TEMP_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).expect("mkdir");
        let storage = CronJobStorage::new(root.join("cron"));
        (storage, root)
    }

    fn input(push_id: &str, created_at: &str) -> WebPushMessageInput {
        WebPushMessageInput {
            push_id: push_id.to_string(),
            job_id: "job-1".to_string(),
            job_name: format!("Push {push_id}"),
            summary: format!("Summary {push_id}"),
            content: format!("Full content {push_id}"),
            created_at: created_at.to_string(),
        }
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn web_push_read_through_keeps_newer_pushes_unread() {
        let (storage, root) = test_storage();
        let actor = ActorIdentity::new("web", "web-user-1", None::<String>).expect("actor");
        let other = ActorIdentity::new("web", "web-user-2", None::<String>).expect("actor");
        storage
            .upsert_web_push_message(&actor, input("p1", "2026-07-10T09:00:00+08:00"))
            .expect("p1");
        storage
            .upsert_web_push_message(&actor, input("p2", "2026-07-10T10:00:00+08:00"))
            .expect("p2");
        storage
            .upsert_web_push_message(&actor, input("p3", "2026-07-10T11:00:00+08:00"))
            .expect("p3");
        storage
            .upsert_web_push_message(&other, input("p4", "2026-07-10T08:00:00+08:00"))
            .expect("p4");

        assert_eq!(storage.count_unread_web_push_messages(&actor).unwrap(), 3);
        assert_eq!(
            storage
                .mark_web_push_messages_read_through(&actor, "p2")
                .unwrap(),
            2
        );
        assert_eq!(storage.count_unread_web_push_messages(&actor).unwrap(), 1);
        assert_eq!(storage.count_unread_web_push_messages(&other).unwrap(), 1);
        assert!(
            storage
                .get_web_push_message(&actor, "p4")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            storage
                .mark_web_push_messages_read_through(&actor, "p4")
                .unwrap(),
            0
        );
        assert_eq!(storage.count_unread_web_push_messages(&other).unwrap(), 1);

        let listed = storage.list_web_push_messages(&actor, None, 10).unwrap();
        assert_eq!(
            listed
                .iter()
                .map(|item| item.push_id.as_str())
                .collect::<Vec<_>>(),
            vec!["p3", "p2", "p1"]
        );
        assert!(listed[0].read_at.is_none());
        assert!(listed[1].read_at.is_some());

        let page = storage
            .list_web_push_messages(&actor, Some("p2"), 10)
            .unwrap();
        assert_eq!(
            page.iter()
                .map(|item| item.push_id.as_str())
                .collect::<Vec<_>>(),
            vec!["p1"]
        );
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn legacy_web_push_batch_is_idempotent_and_preserves_read_state() {
        let (storage, root) = test_storage();
        let actor = ActorIdentity::new("web", "legacy-user", None::<String>).expect("actor");
        let inputs = vec![
            input("legacy:first", "2026-07-10T09:00:00+08:00"),
            input("legacy:second", "2026-07-10T10:00:00+08:00"),
        ];

        assert_eq!(
            storage
                .upsert_web_push_messages(&actor, inputs.clone())
                .expect("first import"),
            2
        );
        assert!(storage.has_legacy_web_push_messages(&actor).unwrap());
        storage
            .mark_web_push_messages_read_through(&actor, "legacy:first")
            .expect("mark read");
        assert_eq!(
            storage
                .upsert_web_push_messages(&actor, inputs)
                .expect("second import"),
            2
        );

        let listed = storage.list_web_push_messages(&actor, None, 10).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed[1].read_at.is_some());
        std::fs::remove_dir_all(root).ok();
    }

    /// 陈旧判定必须按**真实时刻**比较,不能用文本字典序。
    ///
    /// `executed_at` 是 TEXT 列，历史生产行使用 `+08:00`，新行使用配置的运行时时区；调用方
    /// (`crates/hone-scheduler/src/lib.rs:74`)给的 `stale_before` 是
    /// `Utc::now().to_rfc3339()`(`+00:00`)。字典序只比墙钟数字、完全无视偏移,
    /// 于是运行时时区的行会显得比 UTC 阈值"新"最多 8 小时,回收被推迟同样长的时间。
    ///
    /// 2026-08-16 生产实测:一条已 613 分钟未收口的行,文本比较判 false、时刻比较判 true;
    /// `recovered_stale_pending` 自 2026-08-11 起再未新增,同时积压 55 行僵尸 running。
    ///
    /// 构造方式:刚写入的行(运行时时区)配一个"1 小时之后"的 **UTC** 阈值。
    /// 真实时刻上该行必然早于阈值 ⇒ 应当回收;而字典序下 `2026-08-16T…+08:00`
    /// 往往大于 `2026-08-15T…+00:00`,旧实现会判成"不陈旧"从而漏掉。
    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn stale_recovery_compares_instants_not_text_across_timezones() {
        let (storage, root) = test_storage();
        let actor = ActorIdentity::new("feishu", "tz-stale-user", None::<String>).expect("actor");

        let started = |job: &str| CronJobExecutionInput {
            execution_status: "running".to_string(),
            message_send_status: "pending".to_string(),
            should_deliver: false,
            delivered: false,
            response_preview: None,
            error_message: None,
            detail: serde_json::json!({ "phase": "started", "job": job }),
        };

        storage
            .record_execution_event(
                &actor,
                "tz-stale-job",
                "跨时区陈旧回收",
                "tz-stale-user",
                true,
                started("tz-stale-job"),
            )
            .expect("started row");

        // 阈值取 UTC 的「一小时之后」:真实时刻上刚写入的行必然更早 ⇒ 必须回收。
        let stale_before_utc = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        assert_eq!(
            storage
                .recover_stale_started_executions(
                    "feishu",
                    &stale_before_utc,
                    "tz_regression",
                    "scheduler restarted",
                )
                .expect("recover"),
            1,
            "运行时时区写入的 started 行必须按真实时刻判为陈旧;字典序比较会漏掉它"
        );

        // 反向:阈值早于该行真实时刻时不得回收,否则会误杀在途任务。
        storage
            .record_execution_event(
                &actor,
                "tz-fresh-job",
                "在途任务不得误回收",
                "tz-stale-user",
                true,
                started("tz-fresh-job"),
            )
            .expect("fresh row");
        let too_early_utc = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        assert_eq!(
            storage
                .recover_stale_started_executions(
                    "feishu",
                    &too_early_utc,
                    "tz_regression",
                    "scheduler restarted",
                )
                .expect("recover fresh"),
            0,
            "阈值早于真实执行时刻时不得回收"
        );

        std::fs::remove_dir_all(root).ok();
    }
}
