//! Cron 执行历史的 SQLite 持久化。

use hone_core::{ActorIdentity, HoneError, HoneResult, truncate_chars_append};
use rusqlite::{Connection, params};
use serde_json::Value;

use super::CronJobStorage;
use super::types::{CronJobExecutionInput, CronJobExecutionRecord};

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
