use hone_core::{HoneError, HoneResult, LlmAuditRecord, LlmAuditSink};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Deserialize, Default)]
pub struct AuditQueryFilter {
    pub actor_channel: Option<String>,
    pub actor_user_id: Option<String>,
    pub actor_scope: Option<String>,
    pub session_id: Option<String>,
    pub success: Option<bool>,
    pub source: Option<String>,
    pub provider: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AuditRecordSummary {
    pub id: String,
    pub created_at: String,
    pub session_id: String,
    pub actor_channel: Option<String>,
    pub actor_user_id: Option<String>,
    pub actor_scope: Option<String>,
    pub source: String,
    pub operation: String,
    pub provider: String,
    pub model: Option<String>,
    pub success: bool,
    pub latency_ms: Option<u128>,
}

pub struct LlmAuditStorage {
    conn: Mutex<Connection>,
    retention_days: u32,
    write_count: AtomicU64,
}

impl LlmAuditStorage {
    pub fn new(path: impl AsRef<Path>, retention_days: u32) -> HoneResult<Self> {
        let path = path.as_ref().to_path_buf();
        ensure_parent_dir(&path)?;

        let conn = Connection::open(&path)
            .map_err(|e| HoneError::Config(format!("打开 LLM 审计 SQLite 失败: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "busy_timeout", 5000)
            .map_err(sql_err)?;

        let storage = Self {
            conn: Mutex::new(conn),
            retention_days: retention_days.max(1),
            write_count: AtomicU64::new(0),
        };
        storage.init_schema()?;
        storage.prune_expired()?;
        Ok(storage)
    }

    pub fn new_readonly(path: impl AsRef<Path>) -> HoneResult<Self> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|e| HoneError::Config(format!("以只读模式打开 LLM 审计 SQLite 失败: {e}")))?;

        Ok(Self {
            conn: Mutex::new(conn),
            retention_days: 0,
            write_count: AtomicU64::new(0),
        })
    }

    fn init_schema(&self) -> HoneResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS llm_audit_records (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                session_id TEXT NOT NULL,
                actor_channel TEXT,
                actor_user_id TEXT,
                actor_scope TEXT,
                source TEXT NOT NULL,
                operation TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT,
                success INTEGER NOT NULL,
                latency_ms INTEGER,
                request_json TEXT NOT NULL,
                response_json TEXT,
                error_text TEXT,
                metadata_json TEXT,
                prompt_tokens INTEGER,
                completion_tokens INTEGER,
                total_tokens INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_llm_audit_created_at
                ON llm_audit_records(created_at);
            CREATE INDEX IF NOT EXISTS idx_llm_audit_actor
                ON llm_audit_records(actor_channel, actor_user_id, actor_scope, created_at);
            CREATE INDEX IF NOT EXISTS idx_llm_audit_session
                ON llm_audit_records(session_id, created_at);
            ",
        )
        .map_err(sql_err)?;
        Ok(())
    }

    pub fn prune_expired(&self) -> HoneResult<()> {
        let cutoff = hone_core::beijing_now() - chrono::Duration::days(self.retention_days as i64);
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "DELETE FROM llm_audit_records WHERE created_at < ?1",
            params![cutoff.to_rfc3339()],
        )
        .map_err(sql_err)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn count_records(&self) -> HoneResult<i64> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let count = conn
            .query_row("SELECT COUNT(*) FROM llm_audit_records", [], |row| {
                row.get(0)
            })
            .map_err(sql_err)?;
        Ok(count)
    }

    fn maybe_prune_after_write(&self) -> HoneResult<()> {
        let count = self.write_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 100 == 0 {
            self.prune_expired()?;
        }
        Ok(())
    }

    pub fn list_audit_records(
        &self,
        filter: &AuditQueryFilter,
    ) -> HoneResult<(Vec<AuditRecordSummary>, i64)> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut query = "SELECT id, created_at, session_id, actor_channel, actor_user_id, actor_scope, source, operation, provider, model, success, latency_ms FROM llm_audit_records WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM llm_audit_records WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ch) = &filter.actor_channel {
            query.push_str(&format!(" AND actor_channel = ?{param_idx}"));
            count_query.push_str(&format!(" AND actor_channel = ?{param_idx}"));
            params.push(Box::new(ch.clone()));
            param_idx += 1;
        }
        if let Some(uid) = &filter.actor_user_id {
            query.push_str(&format!(" AND actor_user_id = ?{param_idx}"));
            count_query.push_str(&format!(" AND actor_user_id = ?{param_idx}"));
            params.push(Box::new(uid.clone()));
            param_idx += 1;
        }
        if let Some(scp) = &filter.actor_scope {
            query.push_str(&format!(" AND actor_scope = ?{param_idx}"));
            count_query.push_str(&format!(" AND actor_scope = ?{param_idx}"));
            params.push(Box::new(scp.clone()));
            param_idx += 1;
        }
        if let Some(sid) = &filter.session_id {
            query.push_str(&format!(" AND session_id = ?{param_idx}"));
            count_query.push_str(&format!(" AND session_id = ?{param_idx}"));
            params.push(Box::new(sid.clone()));
            param_idx += 1;
        }
        if let Some(suc) = filter.success {
            query.push_str(&format!(" AND success = ?{param_idx}"));
            count_query.push_str(&format!(" AND success = ?{param_idx}"));
            params.push(Box::new(if suc { 1 } else { 0 }));
            param_idx += 1;
        }
        if let Some(src) = &filter.source {
            query.push_str(&format!(" AND source = ?{param_idx}"));
            count_query.push_str(&format!(" AND source = ?{param_idx}"));
            params.push(Box::new(src.clone()));
            param_idx += 1;
        }
        if let Some(prov) = &filter.provider {
            query.push_str(&format!(" AND provider = ?{param_idx}"));
            count_query.push_str(&format!(" AND provider = ?{param_idx}"));
            params.push(Box::new(prov.clone()));
            param_idx += 1;
        }
        if let Some(df) = &filter.date_from {
            query.push_str(&format!(" AND created_at >= ?{param_idx}"));
            count_query.push_str(&format!(" AND created_at >= ?{param_idx}"));
            params.push(Box::new(df.clone()));
            param_idx += 1;
        }
        if let Some(dt) = &filter.date_to {
            query.push_str(&format!(" AND created_at <= ?{param_idx}"));
            count_query.push_str(&format!(" AND created_at <= ?{param_idx}"));
            params.push(Box::new(dt.clone()));
        }

        query.push_str(" ORDER BY created_at DESC");

        let page = filter.page.unwrap_or(1).max(1);
        let page_size = filter.page_size.unwrap_or(50).max(1).min(100);
        query.push_str(&format!(
            " LIMIT {} OFFSET {}",
            page_size,
            (page - 1) * page_size
        ));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        let count: i64 = conn
            .query_row(&count_query, param_refs.as_slice(), |row| row.get(0))
            .map_err(sql_err)?;

        let mut stmt = conn.prepare(&query).map_err(sql_err)?;
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let latency_i64: Option<i64> = row.get(11)?;
                Ok(AuditRecordSummary {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    session_id: row.get(2)?,
                    actor_channel: row.get(3)?,
                    actor_user_id: row.get(4)?,
                    actor_scope: row.get(5)?,
                    source: row.get(6)?,
                    operation: row.get(7)?,
                    provider: row.get(8)?,
                    model: row.get(9)?,
                    success: row.get::<_, i32>(10)? != 0,
                    latency_ms: latency_i64.map(|v| v as u128),
                })
            })
            .map_err(sql_err)?;

        let mut vec = Vec::new();
        for r in rows {
            vec.push(r.map_err(sql_err)?);
        }

        Ok((vec, count))
    }

    pub fn get_audit_record(&self, id: &str) -> HoneResult<Option<LlmAuditRecord>> {
        use hone_core::ActorIdentity;
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, created_at, session_id, actor_channel, actor_user_id, actor_scope, source, operation, provider, model, success, latency_ms, request_json, response_json, error_text, metadata_json, prompt_tokens, completion_tokens, total_tokens
                 FROM llm_audit_records WHERE id = ?1",
            )
            .map_err(sql_err)?;

        let mut rows = stmt
            .query_map(rusqlite::params![id], |row| {
                let actor_channel: Option<String> = row.get(3)?;
                let actor_user_id: Option<String> = row.get(4)?;
                let actor_scope: Option<String> = row.get(5)?;
                let actor = match (actor_channel, actor_user_id) {
                    (Some(c), Some(u)) => ActorIdentity::new(&c, &u, actor_scope.as_deref()).ok(),
                    _ => None,
                };

                let success: i32 = row.get(10)?;
                let latency_i64: Option<i64> = row.get(11)?;
                let request_json: String = row.get(12)?;
                let response_json: Option<String> = row.get(13)?;
                let metadata_json: String = row.get(15)?;

                let request =
                    serde_json::from_str(&request_json).unwrap_or(serde_json::Value::Null);
                let response = response_json.and_then(|j| serde_json::from_str(&j).ok());
                let metadata =
                    serde_json::from_str(&metadata_json).unwrap_or(serde_json::Value::Null);

                let prompt_tokens: Option<u32> = row.get(16)?;
                let completion_tokens: Option<u32> = row.get(17)?;
                let total_tokens: Option<u32> = row.get(18)?;

                Ok(LlmAuditRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    session_id: row.get(2)?,
                    actor,
                    source: row.get(6)?,
                    operation: row.get(7)?,
                    provider: row.get(8)?,
                    model: row.get(9)?,
                    success: success != 0,
                    latency_ms: latency_i64.map(|v| v as u128),
                    request,
                    response,
                    error: row.get(14)?,
                    metadata,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                })
            })
            .map_err(sql_err)?;

        if let Some(r) = rows.next() {
            Ok(Some(r.map_err(sql_err)?))
        } else {
            Ok(None)
        }
    }
}

impl LlmAuditSink for LlmAuditStorage {
    fn record(&self, record: LlmAuditRecord) -> HoneResult<()> {
        let actor_channel = record.actor.as_ref().map(|actor| actor.channel.as_str());
        let actor_user_id = record.actor.as_ref().map(|actor| actor.user_id.as_str());
        let actor_scope = record
            .actor
            .as_ref()
            .and_then(|actor| actor.channel_scope.as_deref());

        let request_json = serde_json::to_string(&record.request)
            .map_err(|e| HoneError::Serialization(e.to_string()))?;
        let response_json = record
            .response
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| HoneError::Serialization(e.to_string()))?;
        let metadata_json = serde_json::to_string(&record.metadata)
            .map_err(|e| HoneError::Serialization(e.to_string()))?;

        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "INSERT INTO llm_audit_records (
                id, created_at, session_id,
                actor_channel, actor_user_id, actor_scope,
                source, operation, provider, model,
                success, latency_ms,
                request_json, response_json, error_text, metadata_json,
                prompt_tokens, completion_tokens, total_tokens
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                record.id,
                record.created_at,
                record.session_id,
                actor_channel,
                actor_user_id,
                actor_scope,
                record.source,
                record.operation,
                record.provider,
                record.model,
                if record.success { 1 } else { 0 },
                record.latency_ms.map(|v| v as i64),
                request_json,
                response_json,
                record.error,
                metadata_json,
                record.prompt_tokens,
                record.completion_tokens,
                record.total_tokens,
            ],
        )
        .map_err(sql_err)?;
        drop(conn);
        self.maybe_prune_after_write()?;
        Ok(())
    }
}

fn ensure_parent_dir(path: &Path) -> HoneResult<()> {
    let parent: PathBuf = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(parent)?;
    Ok(())
}

fn sql_err(err: rusqlite::Error) -> HoneError {
    HoneError::Config(format!("LLM 审计 SQLite 操作失败: {err}"))
}

fn lock_err<T>(_: std::sync::PoisonError<T>) -> HoneError {
    HoneError::Config("LLM 审计 SQLite 锁已污染".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::{ActorIdentity, LlmAuditRecord};
    use serde_json::{Value, json};

    #[test]
    fn record_and_prune_expired_rows() {
        let root = std::env::temp_dir().join(format!("hone_llm_audit_{}", uuid::Uuid::new_v4()));
        let db_path = root.join("audit.sqlite3");
        let storage = LlmAuditStorage::new(&db_path, 30).expect("storage");

        let mut fresh = LlmAuditRecord::new(
            "Actor_feishu__direct__alice",
            Some(ActorIdentity::new("feishu", "alice", None::<String>).expect("actor")),
            "agent.function_calling",
            "chat_with_tools",
            "openrouter",
            Some("moonshotai/kimi-k2.5".to_string()),
            json!({"messages":[{"role":"user","content":"hi"}]}),
        );
        fresh.success = true;
        fresh.response = Some(json!({"content":"hello"}));
        storage.record(fresh).expect("record fresh");

        let stale = LlmAuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: (hone_core::beijing_now() - chrono::Duration::days(31)).to_rfc3339(),
            session_id: "old".to_string(),
            actor: None,
            source: "agent.function_calling".to_string(),
            operation: "chat".to_string(),
            provider: "openrouter".to_string(),
            model: Some("test".to_string()),
            success: true,
            latency_ms: Some(12),
            request: json!({"messages":[]}),
            response: Some(json!({"content":"stale"})),
            error: None,
            metadata: Value::Null,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        };
        storage.record(stale).expect("record stale");

        assert_eq!(storage.count_records().expect("count"), 2);
        storage.prune_expired().expect("prune");
        assert_eq!(storage.count_records().expect("count after"), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn query_audit_records_with_filters() {
        let root = std::env::temp_dir().join(format!("hone_llm_audit_{}", uuid::Uuid::new_v4()));
        let db_path = root.join("audit.sqlite3");
        let storage = LlmAuditStorage::new(&db_path, 30).expect("storage");

        let mut r1 = LlmAuditRecord::new(
            "sess1",
            Some(ActorIdentity::new("wx", "bob", None::<String>).expect("actor")),
            "agent",
            "chat",
            "openai",
            Some("gpt-4".to_string()),
            json!({"q": 1}),
        );
        r1.success = true;
        storage.record(r1.clone()).unwrap();

        let mut r2 = LlmAuditRecord::new(
            "sess2",
            Some(ActorIdentity::new("feishu", "alice", None::<String>).expect("actor")),
            "tool",
            "search",
            "bing",
            None,
            json!({"q": 2}),
        );
        r2.success = false;
        r2.latency_ms = Some(150);
        storage.record(r2.clone()).unwrap();

        // 1. 无条件过滤
        let res_all = storage
            .list_audit_records(&AuditQueryFilter::default())
            .unwrap();
        assert_eq!(res_all.1, 2);

        // 2. Test filtering by actor_channel
        let res = storage
            .list_audit_records(&AuditQueryFilter {
                actor_channel: Some("feishu".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.1, 1);
        assert_eq!(res.0[0].actor_channel.as_deref(), Some("feishu"));
        assert_eq!(res.0[0].latency_ms, Some(150));

        // 3. Test success boolean filtering
        let res_success = storage
            .list_audit_records(&AuditQueryFilter {
                success: Some(true),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res_success.1, 1);
        assert_eq!(res_success.0[0].session_id, "sess1");

        // 4. Test pagination
        let res2 = storage
            .list_audit_records(&AuditQueryFilter {
                page: Some(1),
                page_size: Some(1),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res2.1, 2); // Total count is 2
        assert_eq!(res2.0.len(), 1);

        // 5. Test detail query
        let detail = storage.get_audit_record(&r1.id).unwrap().unwrap();
        assert_eq!(detail.request, json!({"q": 1}));
        assert_eq!(detail.session_id, "sess1");

        let missing = storage.get_audit_record("not-exist").unwrap();
        assert!(missing.is_none());

        let _ = std::fs::remove_dir_all(root);
    }
}
