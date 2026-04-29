//! Cron 执行历史的 SQLite 持久化。
//!
//! 与 `storage.rs` 的 JSON 定义文件互补：
//! - JSON：定时任务的「定义」（hour / minute / repeat / enabled 等）
//! - SQLite：每一次实际触发的「执行事件」（执行时间、投递状态、错误、响应预览）
//!
//! 只在构造 `CronJobStorage` 时传了 sqlite_path 才会启用,这样测试和轻量
//! 场景下仍然可以只用 JSON 即可工作（`open_execution_conn` 会返回 `None`）。
//! schema 用 `CREATE TABLE IF NOT EXISTS`,第一次连接时自动建表。

use hone_core::{ActorIdentity, HoneError, HoneResult, truncate_chars_append};
use rusqlite::{Connection, params};
use serde_json::Value;

use super::CronJobStorage;
use super::types::{CronJobExecutionInput, CronJobExecutionRecord};

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
    pub fn record_execution_event(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
        job_name: &str,
        channel_target: &str,
        heartbeat: bool,
        input: CronJobExecutionInput,
    ) -> HoneResult<()> {
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
        if input.execution_status != "running"
            && input.message_send_status != "pending"
            && let Some(delivery_key) = input.detail.get("delivery_key").and_then(|v| v.as_str())
            && !delivery_key.is_empty()
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
                        delivery_key,
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
            ",
        )
        .map_err(sqlite_err)?;
        Ok(())
    }
}

fn sqlite_err(err: rusqlite::Error) -> HoneError {
    HoneError::Config(format!("Cron 执行记录 SQLite 操作失败: {err}"))
}
