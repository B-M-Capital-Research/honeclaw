use std::path::{Path, PathBuf};
use std::sync::Mutex;

use hone_core::{HoneError, HoneResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::session::{Session, session_message_text};

pub struct SqliteSessionMirror {
    conn: Mutex<Connection>,
}

impl SqliteSessionMirror {
    pub fn new(path: impl AsRef<Path>) -> HoneResult<Self> {
        let path = path.as_ref().to_path_buf();
        ensure_parent_dir(&path)?;

        let conn = Connection::open(&path)
            .map_err(|e| HoneError::Config(format!("打开 Session SQLite 失败: {e}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "busy_timeout", 5000)
            .map_err(sql_err)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(sql_err)?;

        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> HoneResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS migration_runs (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                mode TEXT NOT NULL,
                sessions_scanned INTEGER NOT NULL DEFAULT 0,
                sessions_imported INTEGER NOT NULL DEFAULT 0,
                sessions_skipped INTEGER NOT NULL DEFAULT 0,
                sessions_failed INTEGER NOT NULL DEFAULT 0,
                notes TEXT
            );

            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                source_path TEXT NOT NULL UNIQUE,
                version INTEGER,
                actor_channel TEXT,
                actor_user_id TEXT,
                actor_channel_scope TEXT,
                session_channel TEXT,
                session_kind TEXT,
                session_user_id TEXT,
                session_channel_scope TEXT,
                created_at TEXT,
                updated_at TEXT,
                frozen_time_beijing TEXT,
                message_count INTEGER NOT NULL DEFAULT 0,
                last_message_at TEXT,
                last_message_role TEXT,
                last_message_preview TEXT,
                actor_json TEXT,
                session_identity_json TEXT,
                runtime_json TEXT,
                summary_json TEXT,
                metadata_json TEXT,
                source_json TEXT NOT NULL,
                normalized_json TEXT NOT NULL,
                source_mtime_ns INTEGER NOT NULL,
                source_size INTEGER NOT NULL,
                content_sha256 TEXT NOT NULL,
                imported_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_updated_at
                ON sessions(updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_sessions_actor
                ON sessions(actor_channel, actor_user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_scope
                ON sessions(session_channel, session_kind, session_channel_scope);
            CREATE INDEX IF NOT EXISTS idx_sessions_sha
                ON sessions(content_sha256);

            CREATE TABLE IF NOT EXISTS session_metadata (
                session_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value_json TEXT NOT NULL,
                imported_at TEXT NOT NULL,
                PRIMARY KEY(session_id, key),
                FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS session_messages (
                session_id TEXT NOT NULL,
                ordinal INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT,
                metadata_json TEXT,
                message_id TEXT,
                tool_name TEXT,
                tool_call_id TEXT,
                channel TEXT,
                open_id TEXT,
                mobile TEXT,
                chat_id TEXT,
                chat_type TEXT,
                message_type TEXT,
                content_sha256 TEXT NOT NULL,
                imported_at TEXT NOT NULL,
                PRIMARY KEY(session_id, ordinal),
                FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_session_messages_session_ts
                ON session_messages(session_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_session_messages_tool
                ON session_messages(tool_name, tool_call_id);
            CREATE INDEX IF NOT EXISTS idx_session_messages_message_id
                ON session_messages(message_id);
            ",
        )
        .map_err(sql_err)?;
        Ok(())
    }

    pub fn upsert_session(&self, source_path: &Path, session: &Session) -> HoneResult<()> {
        let source_json = serde_json::to_vec_pretty(session)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let normalized_json = serde_json::to_string(session)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let metadata_json = serde_json::to_string(&session.metadata)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let actor_json = session
            .actor
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let session_identity_json = session
            .session_identity
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let runtime_json = serde_json::to_string(&session.runtime)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let summary_json = session
            .summary
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| HoneError::Serialization(err.to_string()))?;

        let source_meta = std::fs::metadata(source_path)?;
        let source_path = source_path
            .canonicalize()
            .unwrap_or_else(|_| source_path.to_path_buf());
        let source_path_str = source_path.to_string_lossy().to_string();
        let imported_at = hone_core::beijing_now_rfc3339();
        let content_sha256 = sha256_hex(&source_json);
        let last_message = session.messages.last();

        let conn = self.conn.lock().map_err(lock_err)?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        tx.execute(
            "DELETE FROM session_metadata WHERE session_id = ?1",
            params![session.id],
        )
        .map_err(sql_err)?;
        tx.execute(
            "DELETE FROM session_messages WHERE session_id = ?1",
            params![session.id],
        )
        .map_err(sql_err)?;
        tx.execute(
            "DELETE FROM sessions WHERE session_id = ?1",
            params![session.id],
        )
        .map_err(sql_err)?;

        tx.execute(
            "
            INSERT INTO sessions (
                session_id, source_path, version,
                actor_channel, actor_user_id, actor_channel_scope,
                session_channel, session_kind, session_user_id, session_channel_scope,
                created_at, updated_at, frozen_time_beijing,
                message_count, last_message_at, last_message_role, last_message_preview,
                actor_json, session_identity_json, runtime_json, summary_json, metadata_json,
                source_json, normalized_json, source_mtime_ns, source_size, content_sha256, imported_at
            ) VALUES (
                ?1, ?2, ?3,
                ?4, ?5, ?6,
                ?7, ?8, ?9, ?10,
                ?11, ?12, ?13,
                ?14, ?15, ?16, ?17,
                ?18, ?19, ?20, ?21, ?22,
                ?23, ?24, ?25, ?26, ?27, ?28
            )
            ",
            params![
                session.id,
                source_path_str,
                session.version,
                session.actor.as_ref().map(|actor| actor.channel.as_str()),
                session.actor.as_ref().map(|actor| actor.user_id.as_str()),
                session
                    .actor
                    .as_ref()
                    .and_then(|actor| actor.channel_scope.as_deref()),
                session
                    .session_identity
                    .as_ref()
                    .map(|identity| identity.channel.as_str()),
                session
                    .session_identity
                    .as_ref()
                    .map(|identity| match identity.kind {
                        hone_core::SessionKind::Direct => "direct",
                        hone_core::SessionKind::Group => "group",
                    }),
                session
                    .session_identity
                    .as_ref()
                    .and_then(|identity| identity.user_id.as_deref()),
                session
                    .session_identity
                    .as_ref()
                    .and_then(|identity| identity.channel_scope.as_deref()),
                session.created_at,
                session.updated_at,
                session.runtime.prompt.frozen_time_beijing,
                session.messages.len() as i64,
                last_message.map(|message| message.timestamp.as_str()),
                last_message.map(|message| message.role.as_str()),
                last_message.and_then(|message| preview_text(&session_message_text(message))),
                actor_json,
                session_identity_json,
                runtime_json,
                summary_json,
                metadata_json,
                String::from_utf8_lossy(&source_json).to_string(),
                normalized_json,
                source_meta.modified()?.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() as i64,
                source_meta.len() as i64,
                content_sha256,
                imported_at,
            ],
        )
        .map_err(sql_err)?;

        for (key, value) in session.metadata.iter() {
            tx.execute(
                "
                INSERT INTO session_metadata (session_id, key, value_json, imported_at)
                VALUES (?1, ?2, ?3, ?4)
                ",
                params![
                    session.id,
                    key,
                    serde_json::to_string(value)
                        .map_err(|err| HoneError::Serialization(err.to_string()))?,
                    imported_at,
                ],
            )
            .map_err(sql_err)?;
        }

        for (ordinal, message) in session.messages.iter().enumerate() {
            let metadata = message.metadata.as_ref();
            let content_json = serde_json::to_string(&message.content)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            tx.execute(
                "
                INSERT INTO session_messages (
                    session_id, ordinal, role, content, timestamp, metadata_json,
                    message_id, tool_name, tool_call_id, channel, open_id, mobile,
                    chat_id, chat_type, message_type, content_sha256, imported_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14, ?15, ?16, ?17
                )
                ",
                params![
                    session.id,
                    ordinal as i64,
                    message.role,
                    content_json,
                    message.timestamp,
                    metadata
                        .map(serde_json::to_string)
                        .transpose()
                        .map_err(|err| HoneError::Serialization(err.to_string()))?,
                    metadata_value(metadata, "message_id"),
                    metadata_value(metadata, "tool_name"),
                    metadata_value(metadata, "tool_call_id"),
                    metadata_value(metadata, "channel"),
                    metadata_value(metadata, "open_id"),
                    metadata_value(metadata, "mobile"),
                    metadata_value(metadata, "chat_id"),
                    metadata_value(metadata, "chat_type"),
                    metadata_value(metadata, "message_type"),
                    sha256_hex(session_message_text(message).as_bytes()),
                    imported_at,
                ],
            )
            .map_err(sql_err)?;
        }

        tx.commit().map_err(sql_err)?;
        Ok(())
    }

    pub fn load_session(&self, session_id: &str) -> HoneResult<Option<Session>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let json: Option<String> = conn
            .query_row(
                "SELECT normalized_json FROM sessions WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(sql_err)?;

        json.map(|raw| {
            serde_json::from_str(&raw).map_err(|err| HoneError::Serialization(err.to_string()))
        })
        .transpose()
    }

    pub fn list_sessions(&self) -> HoneResult<Vec<Session>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare("SELECT normalized_json FROM sessions ORDER BY updated_at DESC")
            .map_err(sql_err)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(sql_err)?;

        let mut sessions = Vec::new();
        for row in rows {
            let raw = row.map_err(sql_err)?;
            match serde_json::from_str(&raw) {
                Ok(session) => sessions.push(session),
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        "skipping unreadable session row while listing sqlite sessions"
                    );
                }
            }
        }
        Ok(sessions)
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

fn metadata_value<'a>(
    metadata: Option<&'a std::collections::HashMap<String, Value>>,
    key: &str,
) -> Option<&'a str> {
    metadata
        .and_then(|metadata| metadata.get(key))
        .and_then(|value| value.as_str())
}

fn preview_text(content: &str) -> Option<String> {
    let collapsed = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() <= 160 {
        return Some(trimmed.to_string());
    }

    let mut out = String::new();
    for ch in trimmed.chars().take(159) {
        out.push(ch);
    }
    out.push('…');
    Some(out)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn sql_err(err: rusqlite::Error) -> HoneError {
    HoneError::Config(format!("Session SQLite 操作失败: {err}"))
}

fn lock_err<T>(_: std::sync::PoisonError<T>) -> HoneError {
    HoneError::Config("Session SQLite 锁已污染".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{
        Session, SessionRuntimeState, SessionSummary, build_tool_message_metadata_parts,
        session_message_from_text,
    };
    use hone_core::{ActorIdentity, SessionIdentity};
    use std::collections::HashMap;

    fn make_session() -> Session {
        Session {
            version: 4,
            id: "Actor_feishu__direct__alice".to_string(),
            actor: Some(ActorIdentity::new("feishu", "alice", None::<String>).expect("actor")),
            session_identity: Some(SessionIdentity::direct("feishu", "alice").expect("identity")),
            created_at: "2026-03-26T09:00:00+08:00".to_string(),
            updated_at: "2026-03-26T09:05:00+08:00".to_string(),
            messages: vec![
                session_message_from_text("user", "hello", "2026-03-26T09:00:00+08:00", None),
                session_message_from_text("assistant", "world", "2026-03-26T09:05:00+08:00", None),
            ],
            metadata: HashMap::from([("channel".to_string(), Value::String("feishu".to_string()))]),
            runtime: SessionRuntimeState::default(),
            summary: Some(SessionSummary {
                content: "summary".to_string(),
                updated_at: "2026-03-26T09:05:00+08:00".to_string(),
            }),
        }
    }

    #[test]
    fn upsert_session_persists_rows() {
        let root =
            std::env::temp_dir().join(format!("hone_session_sqlite_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        let db_path = root.join("sessions.sqlite3");
        let source_path = root
            .join("sessions")
            .join("Actor_feishu__direct__alice.json");
        std::fs::create_dir_all(source_path.parent().expect("parent")).expect("sessions dir");
        let session = make_session();
        std::fs::write(
            &source_path,
            serde_json::to_string_pretty(&session).expect("json"),
        )
        .expect("write source");

        let mirror = SqliteSessionMirror::new(&db_path).expect("mirror");
        mirror
            .upsert_session(&source_path, &session)
            .expect("upsert");

        let conn = sqlite3_connect(&db_path);
        let session_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .expect("session count");
        let message_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM session_messages", [], |row| {
                row.get(0)
            })
            .expect("message count");
        assert_eq!(session_count, 1);
        assert_eq!(message_count, 2);

        let loaded = mirror
            .load_session("Actor_feishu__direct__alice")
            .expect("load")
            .expect("loaded");
        assert_eq!(loaded.messages.len(), 2);
    }

    #[test]
    fn upsert_session_replaces_old_rows_and_stores_message_metadata_columns() {
        let root = std::env::temp_dir().join(format!(
            "hone_session_sqlite_replace_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("root");
        let db_path = root.join("sessions.sqlite3");
        let source_path = root
            .join("sessions")
            .join("Actor_feishu__direct__alice.json");
        std::fs::create_dir_all(source_path.parent().expect("parent")).expect("sessions dir");

        let mirror = SqliteSessionMirror::new(&db_path).expect("mirror");

        let mut first = make_session();
        std::fs::write(
            &source_path,
            serde_json::to_string_pretty(&first).expect("json"),
        )
        .expect("write source");
        mirror
            .upsert_session(&source_path, &first)
            .expect("first upsert");

        let long_reply = "市场回顾 ".repeat(80);
        first.updated_at = "2026-03-26T09:20:00+08:00".to_string();
        first.summary = Some(SessionSummary {
            content: "updated summary".to_string(),
            updated_at: "2026-03-26T09:20:00+08:00".to_string(),
        });
        first.metadata.insert(
            "channel_scope".to_string(),
            Value::String("chat-1".to_string()),
        );
        first.messages = vec![
            session_message_from_text(
                "tool",
                "{\"ok\":true}",
                "2026-03-26T09:10:00+08:00",
                Some({
                    let mut metadata =
                        build_tool_message_metadata_parts("web_search", Some("call-1"), None);
                    metadata.insert("message_id".to_string(), Value::String("msg-1".to_string()));
                    metadata
                }),
            ),
            session_message_from_text(
                "assistant",
                &long_reply,
                "2026-03-26T09:20:00+08:00",
                Some(HashMap::from([(
                    "message_type".to_string(),
                    Value::String("text".to_string()),
                )])),
            ),
        ];

        std::fs::write(
            &source_path,
            serde_json::to_string_pretty(&first).expect("json"),
        )
        .expect("rewrite source");
        mirror
            .upsert_session(&source_path, &first)
            .expect("second upsert");

        let conn = sqlite3_connect(&db_path);
        let message_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_messages WHERE session_id = ?1",
                params![first.id.clone()],
                |row| row.get(0),
            )
            .expect("message count");
        let metadata_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_metadata WHERE session_id = ?1",
                params![first.id.clone()],
                |row| row.get(0),
            )
            .expect("metadata count");
        let stored_summary: String = conn
            .query_row(
                "SELECT summary_json FROM sessions WHERE session_id = ?1",
                params![first.id.clone()],
                |row| row.get(0),
            )
            .expect("summary json");
        let (tool_name, tool_call_id, message_id): (Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT tool_name, tool_call_id, message_id FROM session_messages WHERE session_id = ?1 AND ordinal = 0",
                params![first.id.clone()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("tool metadata row");
        let last_preview: String = conn
            .query_row(
                "SELECT last_message_preview FROM sessions WHERE session_id = ?1",
                params![first.id.clone()],
                |row| row.get(0),
            )
            .expect("last preview");

        assert_eq!(message_count, 2);
        assert_eq!(metadata_count, 2);
        assert!(stored_summary.contains("updated summary"));
        assert_eq!(tool_name.as_deref(), Some("web_search"));
        assert_eq!(tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(message_id.as_deref(), Some("msg-1"));
        assert!(last_preview.ends_with('…'));
        assert!(last_preview.chars().count() <= 160);

        let loaded = mirror
            .load_session("Actor_feishu__direct__alice")
            .expect("load")
            .expect("loaded");
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(
            loaded.summary.as_ref().map(|v| v.content.as_str()),
            Some("updated summary")
        );
    }

    #[test]
    fn list_sessions_orders_by_updated_at_desc() {
        let root =
            std::env::temp_dir().join(format!("hone_session_sqlite_list_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        let db_path = root.join("sessions.sqlite3");
        let sessions_dir = root.join("sessions");
        std::fs::create_dir_all(&sessions_dir).expect("sessions dir");
        let mirror = SqliteSessionMirror::new(&db_path).expect("mirror");

        let mut older = make_session();
        older.id = "Actor_feishu__direct__older".to_string();
        older.updated_at = "2026-03-26T09:00:00+08:00".to_string();
        let older_path = sessions_dir.join("older.json");
        std::fs::write(
            &older_path,
            serde_json::to_string_pretty(&older).expect("json"),
        )
        .expect("write older");
        mirror
            .upsert_session(&older_path, &older)
            .expect("upsert older");

        let mut newer = make_session();
        newer.id = "Actor_feishu__direct__newer".to_string();
        newer.updated_at = "2026-03-26T10:00:00+08:00".to_string();
        let newer_path = sessions_dir.join("newer.json");
        std::fs::write(
            &newer_path,
            serde_json::to_string_pretty(&newer).expect("json"),
        )
        .expect("write newer");
        mirror
            .upsert_session(&newer_path, &newer)
            .expect("upsert newer");

        let sessions = mirror.list_sessions().expect("list sessions");
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, "Actor_feishu__direct__newer");
        assert_eq!(sessions[1].id, "Actor_feishu__direct__older");
    }

    #[test]
    fn list_sessions_skips_unreadable_rows() {
        let root = std::env::temp_dir().join(format!(
            "hone_session_sqlite_skip_invalid_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("root");
        let db_path = root.join("sessions.sqlite3");
        let sessions_dir = root.join("sessions");
        std::fs::create_dir_all(&sessions_dir).expect("sessions dir");
        let mirror = SqliteSessionMirror::new(&db_path).expect("mirror");

        let valid = make_session();
        let valid_path = sessions_dir.join("valid.json");
        std::fs::write(
            &valid_path,
            serde_json::to_string_pretty(&valid).expect("json"),
        )
        .expect("write valid");
        mirror
            .upsert_session(&valid_path, &valid)
            .expect("upsert valid");

        let conn = sqlite3_connect(&db_path);
        conn.execute(
            "INSERT INTO sessions (
                session_id, source_path, version, actor_channel, actor_user_id, actor_channel_scope,
                session_channel, session_kind, session_user_id, session_channel_scope, created_at,
                updated_at, frozen_time_beijing, message_count, last_message_at, last_message_role,
                last_message_preview, actor_json, session_identity_json, runtime_json, summary_json,
                metadata_json, source_json, normalized_json, source_mtime_ns, source_size,
                content_sha256, imported_at
            ) VALUES (
                ?1, ?2, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, ?3, ?4, NULL, 0, NULL,
                NULL, NULL, NULL, NULL, NULL, NULL, NULL, '{}', '{broken json', 0, 0, 'broken',
                ?5
            )",
            params![
                "Broken_session",
                sessions_dir.join("broken.json").display().to_string(),
                "2026-04-16T09:00:00+08:00",
                "2026-04-16T09:01:00+08:00",
                "2026-04-16T09:01:00+08:00",
            ],
        )
        .expect("insert broken row");

        let sessions = mirror.list_sessions().expect("list sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, valid.id);
    }

    fn sqlite3_connect(path: &Path) -> Connection {
        Connection::open(path).expect("sqlite")
    }
}
