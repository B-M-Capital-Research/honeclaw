use std::path::Path;
use std::sync::Mutex;

use hone_core::{HoneError, HoneResult, beijing_now, beijing_now_rfc3339};
use rusqlite::{Connection, OptionalExtension, params};

const SESSION_TTL_DAYS: i64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInviteUser {
    pub user_id: String,
    pub invite_code: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInviteSession {
    pub session_token: String,
    pub user_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub last_seen_at: String,
}

pub struct WebAuthStorage {
    conn: Mutex<Connection>,
}

impl WebAuthStorage {
    pub fn new(path: impl AsRef<Path>) -> HoneResult<Self> {
        let path = path.as_ref().to_path_buf();
        ensure_parent_dir(&path)?;

        let conn = Connection::open(&path)
            .map_err(|e| HoneError::Config(format!("打开 Web Auth SQLite 失败: {e}")))?;
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
            CREATE TABLE IF NOT EXISTS web_invite_users (
                user_id TEXT PRIMARY KEY,
                invite_code TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                last_login_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_web_invite_users_created_at
                ON web_invite_users(created_at DESC);

            CREATE TABLE IF NOT EXISTS web_auth_sessions (
                session_token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL,
                FOREIGN KEY(user_id) REFERENCES web_invite_users(user_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_web_auth_sessions_user_id
                ON web_auth_sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_web_auth_sessions_expires_at
                ON web_auth_sessions(expires_at);
            ",
        )
        .map_err(sql_err)?;
        Ok(())
    }

    pub fn create_invite_user(&self) -> HoneResult<WebInviteUser> {
        let user = WebInviteUser {
            user_id: generate_user_id(),
            invite_code: generate_invite_code(),
            created_at: beijing_now_rfc3339(),
            last_login_at: None,
        };

        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "
            INSERT INTO web_invite_users (user_id, invite_code, created_at, last_login_at)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                user.user_id,
                user.invite_code,
                user.created_at,
                user.last_login_at,
            ],
        )
        .map_err(sql_err)?;

        Ok(user)
    }

    pub fn list_invite_users(&self) -> HoneResult<Vec<WebInviteUser>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "
                SELECT user_id, invite_code, created_at, last_login_at
                FROM web_invite_users
                ORDER BY created_at DESC
                ",
            )
            .map_err(sql_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(WebInviteUser {
                    user_id: row.get(0)?,
                    invite_code: row.get(1)?,
                    created_at: row.get(2)?,
                    last_login_at: row.get(3)?,
                })
            })
            .map_err(sql_err)?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(sql_err)?);
        }
        Ok(out)
    }

    pub fn find_invite_user_by_code(&self, invite_code: &str) -> HoneResult<Option<WebInviteUser>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.query_row(
            "
            SELECT user_id, invite_code, created_at, last_login_at
            FROM web_invite_users
            WHERE invite_code = ?1
            ",
            params![invite_code],
            |row| {
                Ok(WebInviteUser {
                    user_id: row.get(0)?,
                    invite_code: row.get(1)?,
                    created_at: row.get(2)?,
                    last_login_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn find_invite_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.query_row(
            "
            SELECT user_id, invite_code, created_at, last_login_at
            FROM web_invite_users
            WHERE user_id = ?1
            ",
            params![user_id],
            |row| {
                Ok(WebInviteUser {
                    user_id: row.get(0)?,
                    invite_code: row.get(1)?,
                    created_at: row.get(2)?,
                    last_login_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn create_session_for_invite(&self, invite_code: &str) -> HoneResult<Option<WebInviteSession>> {
        let now = beijing_now();
        let created_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::days(SESSION_TTL_DAYS)).to_rfc3339();
        let token = uuid::Uuid::new_v4().to_string();
        let conn = self.conn.lock().map_err(lock_err)?;
        purge_expired_sessions_inner(&conn, &created_at)?;

        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let user = tx
            .query_row(
                "
                SELECT user_id, invite_code, created_at, last_login_at
                FROM web_invite_users
                WHERE invite_code = ?1
                ",
                params![invite_code],
                |row| {
                    Ok(WebInviteUser {
                        user_id: row.get(0)?,
                        invite_code: row.get(1)?,
                        created_at: row.get(2)?,
                        last_login_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(sql_err)?;
        let Some(user) = user else {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        };

        tx.execute(
            "
            UPDATE web_invite_users
            SET last_login_at = ?2
            WHERE user_id = ?1
            ",
            params![user.user_id, created_at],
        )
        .map_err(sql_err)?;
        tx.execute(
            "
            INSERT INTO web_auth_sessions (session_token, user_id, created_at, expires_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![token, user.user_id, created_at, expires_at, created_at],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)?;

        Ok(Some(WebInviteSession {
            session_token: token,
            user_id: user.user_id,
            created_at: created_at.clone(),
            expires_at,
            last_seen_at: created_at,
        }))
    }

    pub fn authenticate_session(&self, session_token: &str) -> HoneResult<Option<WebInviteUser>> {
        let now = beijing_now_rfc3339();
        let conn = self.conn.lock().map_err(lock_err)?;
        purge_expired_sessions_inner(&conn, &now)?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let user = tx
            .query_row(
                "
                SELECT u.user_id, u.invite_code, u.created_at, u.last_login_at
                FROM web_auth_sessions s
                JOIN web_invite_users u ON u.user_id = s.user_id
                WHERE s.session_token = ?1 AND s.expires_at > ?2
                ",
                params![session_token, now],
                |row| {
                    Ok(WebInviteUser {
                        user_id: row.get(0)?,
                        invite_code: row.get(1)?,
                        created_at: row.get(2)?,
                        last_login_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(sql_err)?;
        if user.is_some() {
            let next_expiry = (beijing_now() + chrono::Duration::days(SESSION_TTL_DAYS)).to_rfc3339();
            tx.execute(
                "
                UPDATE web_auth_sessions
                SET last_seen_at = ?2, expires_at = ?3
                WHERE session_token = ?1
                ",
                params![session_token, now, next_expiry],
            )
            .map_err(sql_err)?;
        }
        tx.commit().map_err(sql_err)?;
        Ok(user)
    }

    pub fn delete_session(&self, session_token: &str) -> HoneResult<()> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.execute(
            "DELETE FROM web_auth_sessions WHERE session_token = ?1",
            params![session_token],
        )
        .map_err(sql_err)?;
        Ok(())
    }
}

fn purge_expired_sessions_inner(conn: &Connection, now: &str) -> HoneResult<()> {
    conn.execute(
        "DELETE FROM web_auth_sessions WHERE expires_at <= ?1",
        params![now],
    )
    .map_err(sql_err)?;
    Ok(())
}

fn generate_user_id() -> String {
    let token = uuid::Uuid::new_v4().simple().to_string();
    format!("web-user-{}", &token[..12])
}

fn generate_invite_code() -> String {
    let token = uuid::Uuid::new_v4().simple().to_string().to_uppercase();
    format!("HONE-{}-{}", &token[..6], &token[6..12])
}

fn ensure_parent_dir(path: &Path) -> HoneResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| HoneError::Config("Web Auth SQLite 缺少父目录".to_string()))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| HoneError::Config(format!("创建 Web Auth SQLite 目录失败: {e}")))?;
    Ok(())
}

fn lock_err<E>(_: E) -> HoneError {
    HoneError::Storage("web auth storage lock poisoned".to_string())
}

fn sql_err(err: rusqlite::Error) -> HoneError {
    HoneError::Storage(format!("web auth sqlite error: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{WebAuthStorage, SESSION_TTL_DAYS};

    fn test_storage() -> WebAuthStorage {
        let root = std::env::temp_dir().join(format!("hone_web_auth_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        WebAuthStorage::new(root.join("sessions.sqlite3")).expect("storage")
    }

    #[test]
    fn create_and_list_invites_round_trip() {
        let storage = test_storage();
        let created = storage.create_invite_user().expect("create");
        let listed = storage.list_invite_users().expect("list");

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].user_id, created.user_id);
        assert_eq!(listed[0].invite_code, created.invite_code);
        assert_eq!(listed[0].last_login_at, None);
    }

    #[test]
    fn invite_login_creates_session_and_authenticates() {
        let storage = test_storage();
        let created = storage.create_invite_user().expect("create");
        let session = storage
            .create_session_for_invite(&created.invite_code)
            .expect("session")
            .expect("session exists");
        let authed = storage
            .authenticate_session(&session.session_token)
            .expect("auth")
            .expect("user");

        assert_eq!(authed.user_id, created.user_id);
        assert!(session.expires_at > session.created_at);
        assert_eq!(
            (chrono::DateTime::parse_from_rfc3339(&session.expires_at).expect("expiry")
                - chrono::DateTime::parse_from_rfc3339(&session.created_at).expect("created"))
            .num_days(),
            SESSION_TTL_DAYS
        );
    }

    #[test]
    fn deleting_session_invalidates_authentication() {
        let storage = test_storage();
        let created = storage.create_invite_user().expect("create");
        let session = storage
            .create_session_for_invite(&created.invite_code)
            .expect("session")
            .expect("session exists");
        storage
            .delete_session(&session.session_token)
            .expect("delete session");

        assert!(
            storage
                .authenticate_session(&session.session_token)
                .expect("auth")
                .is_none()
        );
    }
}
