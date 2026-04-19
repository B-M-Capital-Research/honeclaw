use std::path::Path;
use std::sync::Mutex;

use hone_core::{HoneError, HoneResult, beijing_now, beijing_now_rfc3339};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};

const SESSION_TTL_DAYS: i64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInviteUser {
    pub user_id: String,
    pub invite_code: String,
    pub phone_number: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInviteSession {
    pub session_token: String,
    pub user_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInviteMutation {
    pub invite: WebInviteUser,
    pub cleared_session_count: u32,
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
                phone_number TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_login_at TEXT,
                revoked_at TEXT
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
        ensure_column(&conn, "web_invite_users", "phone_number", "TEXT")?;
        conn.execute(
            "UPDATE web_invite_users SET phone_number = '' WHERE phone_number IS NULL",
            [],
        )
        .map_err(sql_err)?;
        ensure_column(&conn, "web_invite_users", "revoked_at", "TEXT")?;
        Ok(())
    }

    pub fn create_invite_user(&self, phone_number: &str) -> HoneResult<WebInviteUser> {
        let created_at = beijing_now_rfc3339();
        let user_id = generate_user_id();
        let phone_number = validate_phone_number(phone_number)?;
        let conn = self.conn.lock().map_err(lock_err)?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let invite_code = generate_unique_invite_code(&tx)?;
        tx.execute(
            "
            INSERT INTO web_invite_users (user_id, invite_code, phone_number, created_at, last_login_at, revoked_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                &user_id,
                &invite_code,
                &phone_number,
                &created_at,
                None::<String>,
                None::<String>
            ],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)?;

        Ok(WebInviteUser {
            user_id,
            invite_code,
            phone_number,
            created_at,
            last_login_at: None,
            revoked_at: None,
        })
    }

    pub fn list_invite_users(&self) -> HoneResult<Vec<WebInviteUser>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        let mut stmt = conn
            .prepare(
                "
                SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at
                FROM web_invite_users
                ORDER BY created_at DESC
                ",
            )
            .map_err(sql_err)?;
        let rows = stmt.query_map([], map_invite_user).map_err(sql_err)?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(sql_err)?);
        }
        Ok(out)
    }

    pub fn find_invite_user_by_code(&self, invite_code: &str) -> HoneResult<Option<WebInviteUser>> {
        let invite_code = normalize_invite_code(invite_code);
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.query_row(
            "
            SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at
            FROM web_invite_users
            WHERE invite_code = ?1
            ",
            params![invite_code],
            map_invite_user,
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn find_invite_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        let conn = self.conn.lock().map_err(lock_err)?;
        conn.query_row(
            "
            SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at
            FROM web_invite_users
            WHERE user_id = ?1
            ",
            params![user_id],
            map_invite_user,
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn create_session_for_invite(
        &self,
        invite_code: &str,
        phone_number: &str,
    ) -> HoneResult<Option<WebInviteSession>> {
        let invite_code = normalize_invite_code(invite_code);
        let phone_number = normalize_phone_number(phone_number);
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
                SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at
                FROM web_invite_users
                WHERE invite_code = ?1 AND phone_number = ?2 AND revoked_at IS NULL
                ",
                params![invite_code, phone_number],
                map_invite_user,
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
            params![&user.user_id, &created_at],
        )
        .map_err(sql_err)?;
        delete_sessions_for_user_tx(&tx, &user.user_id)?;
        tx.execute(
            "
            INSERT INTO web_auth_sessions (session_token, user_id, created_at, expires_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![&token, &user.user_id, &created_at, &expires_at, &created_at],
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
                SELECT u.user_id, u.invite_code, u.phone_number, u.created_at, u.last_login_at, u.revoked_at
                FROM web_auth_sessions s
                JOIN web_invite_users u ON u.user_id = s.user_id
                WHERE s.session_token = ?1 AND s.expires_at > ?2 AND u.revoked_at IS NULL
                ",
                params![session_token, now],
                map_invite_user,
            )
            .optional()
            .map_err(sql_err)?;
        if user.is_some() {
            let next_expiry =
                (beijing_now() + chrono::Duration::days(SESSION_TTL_DAYS)).to_rfc3339();
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

    pub fn count_active_sessions_for_user(&self, user_id: &str) -> HoneResult<u32> {
        let now = beijing_now_rfc3339();
        let conn = self.conn.lock().map_err(lock_err)?;
        purge_expired_sessions_inner(&conn, &now)?;
        let count = conn
            .query_row(
                "SELECT COUNT(*) FROM web_auth_sessions WHERE user_id = ?1 AND expires_at > ?2",
                params![user_id, now],
                |row| row.get::<_, i64>(0),
            )
            .map_err(sql_err)?;
        Ok(count.max(0) as u32)
    }

    pub fn set_invite_revoked(
        &self,
        user_id: &str,
        revoked: bool,
    ) -> HoneResult<Option<WebInviteMutation>> {
        let now = beijing_now_rfc3339();
        let conn = self.conn.lock().map_err(lock_err)?;
        purge_expired_sessions_inner(&conn, &now)?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let Some(_) = find_invite_user_tx(&tx, user_id)? else {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        };

        let cleared_session_count = if revoked {
            delete_sessions_for_user_tx(&tx, user_id)? as u32
        } else {
            0
        };
        let revoked_at = if revoked { Some(now.as_str()) } else { None };
        tx.execute(
            "
            UPDATE web_invite_users
            SET revoked_at = ?2
            WHERE user_id = ?1
            ",
            params![user_id, revoked_at],
        )
        .map_err(sql_err)?;
        let invite = find_invite_user_tx(&tx, user_id)?.ok_or_else(|| {
            HoneError::Storage("web invite disappeared during update".to_string())
        })?;
        tx.commit().map_err(sql_err)?;
        Ok(Some(WebInviteMutation {
            invite,
            cleared_session_count,
        }))
    }

    pub fn reset_invite_code(&self, user_id: &str) -> HoneResult<Option<WebInviteMutation>> {
        let now = beijing_now_rfc3339();
        let conn = self.conn.lock().map_err(lock_err)?;
        purge_expired_sessions_inner(&conn, &now)?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let Some(_) = find_invite_user_tx(&tx, user_id)? else {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        };

        let invite_code = generate_unique_invite_code(&tx)?;
        let cleared_session_count = delete_sessions_for_user_tx(&tx, user_id)? as u32;
        tx.execute(
            "
            UPDATE web_invite_users
            SET invite_code = ?2, revoked_at = NULL
            WHERE user_id = ?1
            ",
            params![user_id, &invite_code],
        )
        .map_err(sql_err)?;
        let invite = find_invite_user_tx(&tx, user_id)?
            .ok_or_else(|| HoneError::Storage("web invite disappeared during reset".to_string()))?;
        tx.commit().map_err(sql_err)?;
        Ok(Some(WebInviteMutation {
            invite,
            cleared_session_count,
        }))
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

fn normalize_invite_code(invite_code: &str) -> String {
    invite_code
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .trim()
        .to_uppercase()
}

fn normalize_phone_number(phone_number: &str) -> String {
    let mut normalized = String::new();
    for ch in phone_number.trim().chars() {
        if ch.is_ascii_digit() {
            normalized.push(ch);
        } else if ch == '+' && normalized.is_empty() {
            normalized.push(ch);
        }
    }
    normalized
}

fn validate_phone_number(phone_number: &str) -> HoneResult<String> {
    let normalized = normalize_phone_number(phone_number);
    let digit_count = normalized.chars().filter(|ch| ch.is_ascii_digit()).count();
    if (6..=20).contains(&digit_count) {
        Ok(normalized)
    } else {
        Err(HoneError::Config("手机号格式不合法".to_string()))
    }
}

fn generate_unique_invite_code(tx: &Transaction<'_>) -> HoneResult<String> {
    for _ in 0..8 {
        let invite_code = generate_invite_code();
        let existing = tx
            .query_row(
                "SELECT invite_code FROM web_invite_users WHERE invite_code = ?1",
                params![&invite_code],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(sql_err)?;
        if existing.is_none() {
            return Ok(invite_code);
        }
    }

    Err(HoneError::Storage(
        "failed to generate unique web invite code".to_string(),
    ))
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

fn map_invite_user(row: &Row<'_>) -> rusqlite::Result<WebInviteUser> {
    Ok(WebInviteUser {
        user_id: row.get(0)?,
        invite_code: row.get(1)?,
        phone_number: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        created_at: row.get(3)?,
        last_login_at: row.get(4)?,
        revoked_at: row.get(5)?,
    })
}

fn find_invite_user_tx(tx: &Transaction<'_>, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
    tx.query_row(
        "
        SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at
        FROM web_invite_users
        WHERE user_id = ?1
        ",
        params![user_id],
        map_invite_user,
    )
    .optional()
    .map_err(sql_err)
}

fn delete_sessions_for_user_tx(tx: &Transaction<'_>, user_id: &str) -> HoneResult<usize> {
    tx.execute(
        "DELETE FROM web_auth_sessions WHERE user_id = ?1",
        params![user_id],
    )
    .map_err(sql_err)
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> HoneResult<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(sql_err)?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(sql_err)?;
    for item in columns {
        if item.map_err(sql_err)? == column {
            return Ok(());
        }
    }

    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
        [],
    )
    .map_err(sql_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SESSION_TTL_DAYS, WebAuthStorage};
    use rusqlite::Connection;

    fn test_storage() -> WebAuthStorage {
        let root = std::env::temp_dir().join(format!("hone_web_auth_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        WebAuthStorage::new(root.join("sessions.sqlite3")).expect("storage")
    }

    #[test]
    fn create_and_list_invites_round_trip() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let listed = storage.list_invite_users().expect("list");

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].user_id, created.user_id);
        assert_eq!(listed[0].invite_code, created.invite_code);
        assert_eq!(listed[0].phone_number, "13800138000");
        assert_eq!(listed[0].last_login_at, None);
        assert_eq!(listed[0].revoked_at, None);
    }

    #[test]
    fn invite_login_creates_session_and_authenticates() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let session = storage
            .create_session_for_invite(&created.invite_code, "13800138000")
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
    fn second_login_replaces_previous_session() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let first = storage
            .create_session_for_invite(&created.invite_code, "13800138000")
            .expect("first")
            .expect("session exists");
        let second = storage
            .create_session_for_invite(&created.invite_code, "13800138000")
            .expect("second")
            .expect("session exists");

        assert!(
            storage
                .authenticate_session(&first.session_token)
                .expect("auth first")
                .is_none()
        );
        assert!(
            storage
                .authenticate_session(&second.session_token)
                .expect("auth second")
                .is_some()
        );
        assert_eq!(
            storage
                .count_active_sessions_for_user(&created.user_id)
                .expect("count"),
            1
        );
    }

    #[test]
    fn deleting_session_invalidates_authentication() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let session = storage
            .create_session_for_invite(&created.invite_code, "13800138000")
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

    #[test]
    fn revoking_invite_invalidates_existing_session_and_blocks_future_login() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let session = storage
            .create_session_for_invite(&created.invite_code, "13800138000")
            .expect("session")
            .expect("session exists");

        let revoked = storage
            .set_invite_revoked(&created.user_id, true)
            .expect("revoke")
            .expect("invite exists");

        assert_eq!(revoked.cleared_session_count, 1);
        assert!(revoked.invite.revoked_at.is_some());
        assert!(
            storage
                .authenticate_session(&session.session_token)
                .expect("auth")
                .is_none()
        );
        assert!(
            storage
                .create_session_for_invite(&created.invite_code, "13800138000")
                .expect("login")
                .is_none()
        );
    }

    #[test]
    fn reactivating_invite_allows_login_again() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        storage
            .set_invite_revoked(&created.user_id, true)
            .expect("revoke")
            .expect("invite exists");
        let restored = storage
            .set_invite_revoked(&created.user_id, false)
            .expect("restore")
            .expect("invite exists");

        assert_eq!(restored.cleared_session_count, 0);
        assert_eq!(restored.invite.revoked_at, None);
        assert!(
            storage
                .create_session_for_invite(&created.invite_code, "13800138000")
                .expect("login")
                .is_some()
        );
    }

    #[test]
    fn resetting_invite_rotates_code_and_invalidates_existing_session() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let session = storage
            .create_session_for_invite(&created.invite_code, "13800138000")
            .expect("session")
            .expect("session exists");
        let reset = storage
            .reset_invite_code(&created.user_id)
            .expect("reset")
            .expect("invite exists");

        assert_eq!(reset.cleared_session_count, 1);
        assert_ne!(reset.invite.invite_code, created.invite_code);
        assert_eq!(reset.invite.revoked_at, None);
        assert!(
            storage
                .create_session_for_invite(&created.invite_code, "13800138000")
                .expect("old code")
                .is_none()
        );
        assert!(
            storage
                .authenticate_session(&session.session_token)
                .expect("auth")
                .is_none()
        );
        assert!(
            storage
                .create_session_for_invite(&reset.invite.invite_code, "13800138000")
                .expect("new code")
                .is_some()
        );
    }

    #[test]
    fn invite_login_requires_matching_phone_number() {
        let storage = test_storage();
        let created = storage
            .create_invite_user("+86 138-0013-8000")
            .expect("create");

        assert!(
            storage
                .create_session_for_invite(&created.invite_code, "13900139000")
                .expect("login mismatch")
                .is_none()
        );
        assert!(
            storage
                .create_session_for_invite(&created.invite_code, "+86 138 0013 8000")
                .expect("login match")
                .is_some()
        );
    }

    #[test]
    fn invalid_phone_number_is_rejected_when_creating_invite() {
        let storage = test_storage();
        let error = storage
            .create_invite_user("abc")
            .expect_err("invalid phone");
        assert!(error.to_string().contains("手机号格式不合法"));
    }

    #[test]
    fn new_storage_adds_phone_and_revoked_columns_for_existing_database() {
        let root =
            std::env::temp_dir().join(format!("hone_web_auth_migrate_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        let path = root.join("sessions.sqlite3");
        let conn = Connection::open(&path).expect("open");
        conn.execute_batch(
            "
            CREATE TABLE web_invite_users (
                user_id TEXT PRIMARY KEY,
                invite_code TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                last_login_at TEXT
            );
            CREATE TABLE web_auth_sessions (
                session_token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL
            );
            ",
        )
        .expect("legacy schema");
        drop(conn);

        let storage = WebAuthStorage::new(&path).expect("migrate");
        let created = storage.create_invite_user("13800138000").expect("create");
        let listed = storage.list_invite_users().expect("list");

        assert_eq!(listed[0].user_id, created.user_id);
        assert_eq!(listed[0].phone_number, "13800138000");
        assert_eq!(listed[0].revoked_at, None);
    }
}
