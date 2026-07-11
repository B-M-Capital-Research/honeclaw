//! Cron 执行历史的 SQLite 持久化。
//!
//! 与 `storage.rs` 的 JSON 定义文件互补：
//! - JSON：定时任务的「定义」（hour / minute / repeat / enabled 等）
//! - SQLite：每一次实际触发的「执行事件」（执行时间、投递状态、错误、响应预览）
//!
//! 只在构造 `CronJobStorage` 时传了 sqlite_path 才会启用,这样测试和轻量
//! 场景下仍然可以只用 JSON 即可工作（`open_execution_conn` 会返回 `None`）。
//! schema 用 `CREATE TABLE IF NOT EXISTS`,第一次连接时自动建表。

use hone_core::cloud_runtime::{CloudCronExecutionFilter, CloudCronExecutionInput};
use hone_core::{ActorIdentity, HoneError, HoneResult, truncate_chars_append};
use rusqlite::{Connection, params};
use serde_json::Value;

use super::CronJobStorage;
use super::run_cloud_cron;
use super::types::{
    CronJobExecutionInput, CronJobExecutionRecord, WebPushMessage, WebPushMessageInput,
};

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
        if let Some(postgres) = self.cloud_postgres() {
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

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(0);
        };
        let recovered_at = hone_core::beijing_now_rfc3339();
        let error_message = truncate_chars_append(reason, 500, "...");
        let updated = conn
            .execute(
                "
                UPDATE cron_job_runs
                SET
                    executed_at = ?1,
                    execution_status = 'execution_failed',
                    message_send_status = 'skipped_error',
                    should_deliver = 0,
                    delivered = 0,
                    response_preview = NULL,
                    error_message = ?2,
                    detail_json = json_object(
                        'phase', 'scheduler_handler_watchdog_timeout',
                        'recovered_at', ?1,
                        'recovered_by', ?3,
                        'delivery_key', json_extract(detail_json, '$.delivery_key'),
                        'previous_phase', json_extract(detail_json, '$.phase')
                    )
                WHERE job_id = ?4
                  AND actor_channel = ?5
                  AND actor_user_id = ?6
                  AND COALESCE(actor_channel_scope, '') = COALESCE(?7, '')
                  AND channel_target = ?8
                  AND heartbeat = ?9
                  AND execution_status = 'running'
                  AND message_send_status = 'pending'
                  AND json_extract(detail_json, '$.phase') = 'started'
                  AND json_extract(detail_json, '$.delivery_key') = ?10
                ",
                params![
                    recovered_at,
                    error_message,
                    recovered_by,
                    job_id,
                    actor.channel,
                    actor.user_id,
                    actor.channel_scope,
                    channel_target,
                    if heartbeat { 1 } else { 0 },
                    delivery_key,
                ],
            )
            .map_err(sqlite_err)?;
        Ok(updated)
    }

    pub fn recover_stale_started_executions(
        &self,
        channel: &str,
        stale_before_rfc3339: &str,
        recovered_by: &str,
        reason: &str,
    ) -> HoneResult<usize> {
        if let Some(postgres) = self.cloud_postgres() {
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

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(0);
        };
        let interrupted_at = hone_core::beijing_now_rfc3339();
        let error_message = truncate_chars_append(reason, 500, "...");
        let updated = conn
            .execute(
                "
                UPDATE cron_job_runs
                SET
                    executed_at = ?1,
                    execution_status = 'execution_failed',
                    message_send_status = 'send_failed',
                    should_deliver = 0,
                    delivered = 0,
                    response_preview = NULL,
                    error_message = ?2,
                    detail_json = json_object(
                        'phase', 'recovered_stale_pending',
                        'recovered_at', ?1,
                        'recovered_by', ?3,
                        'delivery_key', json_extract(detail_json, '$.delivery_key'),
                        'previous_phase', json_extract(detail_json, '$.phase')
                    )
                WHERE actor_channel = ?4
                  AND execution_status = 'running'
                  AND message_send_status = 'pending'
                  AND json_extract(detail_json, '$.phase') = 'started'
                  AND executed_at < ?5
                ",
                params![
                    interrupted_at,
                    error_message,
                    recovered_by,
                    channel,
                    stale_before_rfc3339,
                ],
            )
            .map_err(sqlite_err)?;
        Ok(updated)
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
        if let Some(postgres) = self.cloud_postgres() {
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
            return run_cloud_cron(async move {
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
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(());
        };
        let executed_at = hone_core::beijing_now_rfc3339();
        let response_preview = input
            .response_preview
            .as_deref()
            .map(|text| truncate_chars_append(text, 500, "..."));
        let error_message = input
            .error_message
            .as_deref()
            .map(|text| truncate_chars_append(text, 500, "..."));
        let detail_json = serde_json::to_string(&input.detail)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        if input.execution_status != "running" && input.message_send_status != "pending" {
            if let Some(delivery_key) = input.detail.get("delivery_key").and_then(|v| v.as_str())
                && !delivery_key.trim().is_empty()
            {
                let updated = conn
                    .execute(
                        "
                        UPDATE cron_job_runs
                        SET
                            executed_at = ?1,
                            execution_status = ?2,
                            message_send_status = ?3,
                            should_deliver = ?4,
                            delivered = ?5,
                            response_preview = ?6,
                            error_message = ?7,
                            detail_json = ?8
                        WHERE run_id = (
                            SELECT run_id
                            FROM cron_job_runs
                            WHERE job_id = ?9
                              AND actor_channel = ?10
                              AND actor_user_id = ?11
                              AND COALESCE(actor_channel_scope, '') = COALESCE(?12, '')
                              AND channel_target = ?13
                              AND heartbeat = ?14
                              AND execution_status = 'running'
                              AND message_send_status = 'pending'
                              AND json_extract(detail_json, '$.delivery_key') = ?15
                            ORDER BY executed_at DESC, run_id DESC
                            LIMIT 1
                        )
                        ",
                        params![
                            executed_at,
                            input.execution_status,
                            input.message_send_status,
                            if input.should_deliver { 1 } else { 0 },
                            if input.delivered { 1 } else { 0 },
                            response_preview.as_deref(),
                            error_message.as_deref(),
                            detail_json.as_str(),
                            job_id,
                            actor.channel,
                            actor.user_id,
                            actor.channel_scope,
                            channel_target,
                            if heartbeat { 1 } else { 0 },
                            delivery_key.trim(),
                        ],
                    )
                    .map_err(sqlite_err)?;
                if updated > 0 {
                    return Ok(());
                }
            }

            let updated = conn
                .execute(
                    "
                    UPDATE cron_job_runs
                    SET
                        executed_at = ?1,
                        execution_status = ?2,
                        message_send_status = ?3,
                        should_deliver = ?4,
                        delivered = ?5,
                        response_preview = ?6,
                        error_message = ?7,
                        detail_json = ?8
                    WHERE run_id = (
                        SELECT run_id
                        FROM cron_job_runs
                        WHERE job_id = ?9
                          AND actor_channel = ?10
                          AND actor_user_id = ?11
                          AND COALESCE(actor_channel_scope, '') = COALESCE(?12, '')
                          AND channel_target = ?13
                          AND heartbeat = ?14
                          AND execution_status = 'running'
                          AND message_send_status = 'pending'
                          AND json_extract(detail_json, '$.phase') = 'started'
                          AND datetime(executed_at) >= datetime(?15, '-2 hours')
                        ORDER BY executed_at DESC, run_id DESC
                        LIMIT 1
                    )
                    ",
                    params![
                        executed_at,
                        input.execution_status,
                        input.message_send_status,
                        if input.should_deliver { 1 } else { 0 },
                        if input.delivered { 1 } else { 0 },
                        response_preview.as_deref(),
                        error_message.as_deref(),
                        detail_json.as_str(),
                        job_id,
                        actor.channel,
                        actor.user_id,
                        actor.channel_scope,
                        channel_target,
                        if heartbeat { 1 } else { 0 },
                        executed_at,
                    ],
                )
                .map_err(sqlite_err)?;
            if updated > 0 {
                return Ok(());
            }
        }
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
    ) -> HoneResult<Vec<CronJobExecutionRecord>> {
        if let Some(postgres) = self.cloud_postgres() {
            let filter = CloudCronExecutionFilter {
                job_id: Some(job_id.to_string()),
                limit,
                ..CloudCronExecutionFilter::default()
            };
            return run_cloud_cron(
                async move { postgres.list_cron_execution_records(filter).await },
            )
            .map(|records| records.into_iter().map(cron_execution_from_cloud).collect());
        }

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

    /// 跨任务查询执行记录,用于管理端"推送日志"页。filter 中的所有字段都是
    /// `AND` 连接;`limit` 必须 > 0,调用方负责裁剪到合理上限。
    pub fn list_recent_executions(
        &self,
        filter: &ExecutionFilter,
    ) -> HoneResult<Vec<CronJobExecutionRecord>> {
        if let Some(postgres) = self.cloud_postgres() {
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
            return run_cloud_cron(
                async move { postgres.list_cron_execution_records(filter).await },
            )
            .map(|records| records.into_iter().map(cron_execution_from_cloud).collect());
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(Vec::new());
        };
        let mut sql = String::from(
            "
            SELECT
                run_id, job_id, job_name,
                actor_channel, actor_user_id, actor_channel_scope,
                channel_target, heartbeat,
                executed_at, execution_status, message_send_status,
                should_deliver, delivered, response_preview, error_message, detail_json
            FROM cron_job_runs
            WHERE 1=1
            ",
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(since) = filter.since.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND executed_at >= ?");
            params.push(Box::new(since.to_string()));
        }
        if let Some(until) = filter.until.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND executed_at <= ?");
            params.push(Box::new(until.to_string()));
        }
        if let Some(channel) = filter.channel.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND actor_channel = ?");
            params.push(Box::new(channel.to_string()));
        }
        if let Some(user_id) = filter.user_id.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND actor_user_id = ?");
            params.push(Box::new(user_id.to_string()));
        }
        if let Some(job_id) = filter.job_id.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND job_id = ?");
            params.push(Box::new(job_id.to_string()));
        }
        if let Some(status) = filter.execution_status.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND execution_status = ?");
            params.push(Box::new(status.to_string()));
        }
        if let Some(status) = filter
            .message_send_status
            .as_deref()
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND message_send_status = ?");
            params.push(Box::new(status.to_string()));
        }
        if let Some(true) = filter.heartbeat_only {
            sql.push_str(" AND heartbeat = 1");
        }
        sql.push_str(" ORDER BY executed_at DESC, run_id DESC LIMIT ?");
        let limit = filter.limit.max(1);
        params.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&sql).map_err(sqlite_err)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let rows = stmt
            .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
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

    pub fn upsert_web_push_message(
        &self,
        actor: &ActorIdentity,
        input: WebPushMessageInput,
    ) -> HoneResult<WebPushMessage> {
        if let Some(postgres) = self.cloud_postgres() {
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

        let actor_storage_key = actor.storage_key();
        let Some(conn) = self.open_execution_conn()? else {
            return Err(HoneError::Storage(
                "Web 推送存储未配置 SQLite 或 Postgres".to_string(),
            ));
        };
        conn.execute(
            "
            INSERT INTO web_push_messages (
                actor_storage_key, push_id, job_id, job_name,
                summary, content, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(actor_storage_key, push_id) DO UPDATE SET
                job_id = excluded.job_id,
                job_name = excluded.job_name,
                summary = excluded.summary,
                content = excluded.content,
                created_at = excluded.created_at
            ",
            params![
                actor_storage_key,
                input.push_id,
                input.job_id,
                input.job_name,
                input.summary,
                input.content,
                input.created_at,
            ],
        )
        .map_err(sqlite_err)?;
        self.get_web_push_message(actor, &input.push_id)?
            .ok_or_else(|| HoneError::Storage("Web 推送写入后无法读取".to_string()))
    }

    pub fn upsert_web_push_messages(
        &self,
        actor: &ActorIdentity,
        inputs: Vec<WebPushMessageInput>,
    ) -> HoneResult<usize> {
        if inputs.is_empty() {
            return Ok(0);
        }
        if let Some(postgres) = self.cloud_postgres() {
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
            return run_cloud_cron(async move {
                postgres.upsert_web_push_messages(&actor, messages).await
            });
        }

        let actor_storage_key = actor.storage_key();
        let Some(mut conn) = self.open_execution_conn()? else {
            return Err(HoneError::Storage(
                "Web 推送存储未配置 SQLite 或 Postgres".to_string(),
            ));
        };
        let transaction = conn.transaction().map_err(sqlite_err)?;
        for input in &inputs {
            transaction
                .execute(
                    "
                    INSERT INTO web_push_messages (
                        actor_storage_key, push_id, job_id, job_name,
                        summary, content, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    ON CONFLICT(actor_storage_key, push_id) DO UPDATE SET
                        job_id = excluded.job_id,
                        job_name = excluded.job_name,
                        summary = excluded.summary,
                        content = excluded.content,
                        created_at = excluded.created_at
                    ",
                    params![
                        actor_storage_key,
                        input.push_id,
                        input.job_id,
                        input.job_name,
                        input.summary,
                        input.content,
                        input.created_at,
                    ],
                )
                .map_err(sqlite_err)?;
        }
        transaction.commit().map_err(sqlite_err)?;
        Ok(inputs.len())
    }

    pub fn has_legacy_web_push_messages(&self, actor: &ActorIdentity) -> HoneResult<bool> {
        if let Some(postgres) = self.cloud_postgres() {
            let actor = actor.clone();
            return run_cloud_cron(
                async move { postgres.has_legacy_web_push_messages(&actor).await },
            );
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(false);
        };
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM web_push_messages WHERE actor_storage_key = ?1 AND push_id LIKE 'legacy:%')",
            params![actor.storage_key()],
            |row| row.get(0),
        )
        .map_err(sqlite_err)
    }

    pub fn list_web_push_messages(
        &self,
        actor: &ActorIdentity,
        before_push_id: Option<&str>,
        limit: usize,
    ) -> HoneResult<Vec<WebPushMessage>> {
        if let Some(postgres) = self.cloud_postgres() {
            let actor = actor.clone();
            let before_push_id = before_push_id.map(str::to_string);
            return run_cloud_cron(async move {
                postgres
                    .list_web_push_messages(&actor, before_push_id, limit)
                    .await
            })
            .map(|records| records.into_iter().map(web_push_from_cloud).collect());
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(Vec::new());
        };
        let actor_storage_key = actor.storage_key();
        let mut stmt = conn
            .prepare(
                "
                SELECT push_id, actor_storage_key, job_id, job_name,
                       summary, content, created_at, read_at
                FROM web_push_messages
                WHERE actor_storage_key = ?1
                  AND (
                    ?2 IS NULL
                    OR created_at < (
                        SELECT created_at FROM web_push_messages
                        WHERE actor_storage_key = ?1 AND push_id = ?2
                    )
                    OR (
                        created_at = (
                            SELECT created_at FROM web_push_messages
                            WHERE actor_storage_key = ?1 AND push_id = ?2
                        )
                        AND push_id < ?2
                    )
                  )
                ORDER BY created_at DESC, push_id DESC
                LIMIT ?3
                ",
            )
            .map_err(sqlite_err)?;
        let rows = stmt
            .query_map(
                params![actor_storage_key, before_push_id, limit.max(1) as i64],
                web_push_from_sqlite_row,
            )
            .map_err(sqlite_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sqlite_err)
    }

    pub fn get_web_push_message(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
    ) -> HoneResult<Option<WebPushMessage>> {
        if let Some(postgres) = self.cloud_postgres() {
            let actor = actor.clone();
            let push_id = push_id.to_string();
            return run_cloud_cron(
                async move { postgres.get_web_push_message(&actor, &push_id).await },
            )
            .map(|record| record.map(web_push_from_cloud));
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(None);
        };
        let actor_storage_key = actor.storage_key();
        conn.query_row(
            "
            SELECT push_id, actor_storage_key, job_id, job_name,
                   summary, content, created_at, read_at
            FROM web_push_messages
            WHERE actor_storage_key = ?1 AND push_id = ?2
            ",
            params![actor_storage_key, push_id],
            web_push_from_sqlite_row,
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(sqlite_err(other)),
        })
    }

    pub fn count_unread_web_push_messages(&self, actor: &ActorIdentity) -> HoneResult<usize> {
        if let Some(postgres) = self.cloud_postgres() {
            let actor = actor.clone();
            return run_cloud_cron(
                async move { postgres.count_unread_web_push_messages(&actor).await },
            );
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(0);
        };
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM web_push_messages WHERE actor_storage_key = ?1 AND read_at IS NULL",
                params![actor.storage_key()],
                |row| row.get(0),
            )
            .map_err(sqlite_err)?;
        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    pub fn mark_web_push_messages_read_through(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
    ) -> HoneResult<usize> {
        let read_at = hone_core::beijing_now_rfc3339();
        if let Some(postgres) = self.cloud_postgres() {
            let actor = actor.clone();
            let push_id = push_id.to_string();
            return run_cloud_cron(async move {
                postgres
                    .mark_web_push_messages_read_through(&actor, &push_id, &read_at)
                    .await
            });
        }

        let Some(conn) = self.open_execution_conn()? else {
            return Ok(0);
        };
        conn.execute(
            "
            UPDATE web_push_messages
            SET read_at = ?3
            WHERE actor_storage_key = ?1
              AND read_at IS NULL
              AND created_at <= (
                  SELECT created_at FROM web_push_messages
                  WHERE actor_storage_key = ?1 AND push_id = ?2
              )
            ",
            params![actor.storage_key(), push_id, read_at],
        )
        .map_err(sqlite_err)
    }

    pub(super) fn open_execution_conn(&self) -> HoneResult<Option<Connection>> {
        let Some(path) = &self.sqlite_path else {
            return Ok(None);
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| HoneError::Storage(err.to_string()))?;
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

    pub(super) fn init_execution_schema(&self) -> HoneResult<()> {
        let Some(conn) = self.open_execution_conn()? else {
            return Ok(());
        };
        self.init_execution_schema_with_conn(&conn)
    }

    fn init_execution_schema_with_conn(&self, conn: &Connection) -> HoneResult<()> {
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

            CREATE TABLE IF NOT EXISTS web_push_messages (
                actor_storage_key TEXT NOT NULL,
                push_id TEXT NOT NULL,
                job_id TEXT NOT NULL,
                job_name TEXT NOT NULL,
                summary TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                read_at TEXT,
                PRIMARY KEY (actor_storage_key, push_id)
            );

            CREATE INDEX IF NOT EXISTS idx_web_push_actor_time
                ON web_push_messages(actor_storage_key, created_at DESC, push_id DESC);
            ",
        )
        .map_err(sqlite_err)?;
        Ok(())
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
        executed_at: record.executed_at,
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

fn web_push_from_sqlite_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WebPushMessage> {
    Ok(WebPushMessage {
        push_id: row.get(0)?,
        actor_storage_key: row.get(1)?,
        job_id: row.get(2)?,
        job_name: row.get(3)?,
        summary: row.get(4)?,
        content: row.get(5)?,
        created_at: row.get(6)?,
        read_at: row.get(7)?,
    })
}

fn sqlite_err(err: rusqlite::Error) -> HoneError {
    HoneError::Config(format!("Cron 执行记录 SQLite 操作失败: {err}"))
}

#[cfg(test)]
mod web_push_tests {
    use super::*;

    fn test_storage() -> (CronJobStorage, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "hone_web_push_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("mkdir");
        let storage = CronJobStorage::with_sqlite(root.join("cron"), root.join("cron.sqlite3"));
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
}
