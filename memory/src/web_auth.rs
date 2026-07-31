use std::future::Future;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use hone_core::cloud_runtime::{
    CloudPgRuntime, CloudWebAdminCreateOutcome, CloudWebAdminDisableOutcome,
};
use hone_core::{HoneError, HoneResult, beijing_now, beijing_now_rfc3339};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SESSION_TTL_DAYS_LONG: i64 = 30;
pub const SESSION_TTL_DAYS_SHORT: i64 = 1;
pub const REGISTRATION_POLICY_CN_DOMESTIC: &str = "cn_domestic";
pub const REGISTRATION_POLICY_WHOP_INTERNATIONAL: &str = "whop_international";
pub const WEB_ADMIN_DAILY_INVITE_LIMIT: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebInviteUser {
    pub user_id: String,
    pub invite_code: String,
    pub phone_number: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
    pub revoked_at: Option<String>,
    pub password_hash: Option<String>,
    pub password_set_at: Option<String>,
    pub tos_accepted_at: Option<String>,
    pub tos_version: Option<String>,
    pub api_key_prefix: Option<String>,
    pub api_key_created_at: Option<String>,
    pub api_key_last_used_at: Option<String>,
    /// One-time plaintext key returned only by create/generate/reset flows.
    #[serde(default, skip_serializing, skip_deserializing)]
    pub api_key_plaintext: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhopMembershipRecord {
    pub membership_id: String,
    pub whop_user_id: String,
    pub company_id: String,
    pub product_id: String,
    pub plan_id: String,
    pub status: String,
    pub manage_url: Option<String>,
    pub renewal_period_start: Option<String>,
    pub renewal_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub last_event_id: String,
    pub last_event_at: String,
    pub updated_at: String,
}

impl WhopMembershipRecord {
    pub fn grants_paid_access(&self) -> bool {
        matches!(
            self.status.as_str(),
            "active" | "trialing" | "past_due" | "canceling"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhopMembershipEvent {
    pub membership_id: String,
    pub whop_user_id: String,
    pub email_address: String,
    pub company_id: String,
    pub product_id: String,
    pub plan_id: String,
    pub status: String,
    pub manage_url: Option<String>,
    pub renewal_period_start: Option<String>,
    pub renewal_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub event_id: String,
    pub event_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhopMembershipUpsertOutcome {
    Created,
    Updated,
    Duplicate,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebUserExternalProfile {
    pub email_address: Option<String>,
    pub email_verified_at: Option<String>,
    pub registration_policy: String,
    pub whop_membership: Option<WhopMembershipRecord>,
}

impl Default for WebUserExternalProfile {
    fn default() -> Self {
        Self {
            email_address: None,
            email_verified_at: None,
            registration_policy: REGISTRATION_POLICY_CN_DOMESTIC.to_string(),
            whop_membership: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailVerificationResult {
    Verified { user_id: String },
    Missing,
    Expired,
    Invalid,
    AttemptsExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EmailVerificationChallenge {
    code_hash: String,
    requested_at: String,
    expires_at: String,
    attempts: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct WebUserExternalState {
    #[serde(flatten)]
    profile: WebUserExternalProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    email_challenge: Option<EmailVerificationChallenge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebAdminInviteCreateOutcome {
    Created {
        invite: WebInviteUser,
        used_today: u32,
    },
    NotAdmin,
    LimitReached {
        used_today: u32,
    },
    DuplicatePhone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebAdminInviteDisableOutcome {
    Disabled(WebInviteMutation),
    AlreadyDisabled(WebInviteUser),
    NotAdmin,
    NotFound,
    ProtectedAdmin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSessionAuthResult {
    Authenticated(WebInviteUser),
    Missing,
    Expired { user_id: String },
    UserRevoked { user_id: String },
    UserMissing { user_id: String },
}

pub struct WebAuthStorage {
    backend: WebAuthBackend,
}

enum WebAuthBackend {
    Sqlite { conn: Mutex<Connection> },
    Cloud { postgres: CloudPgRuntime },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudWebInviteRecord {
    #[serde(flatten)]
    user: WebInviteUser,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "external_state_is_default")]
    external_state: WebUserExternalState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudWebAuthSessionRecord {
    session_hash: String,
    user_id: String,
    created_at: String,
    expires_at: String,
    last_seen_at: String,
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
            backend: WebAuthBackend::Sqlite {
                conn: Mutex::new(conn),
            },
        };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn new_cloud(postgres: CloudPgRuntime) -> HoneResult<Self> {
        let schema_postgres = postgres.clone();
        run_cloud_web_auth(async move { schema_postgres.ensure_schema().await })?;
        Ok(Self {
            backend: WebAuthBackend::Cloud { postgres },
        })
    }

    fn init_schema(&self) -> HoneResult<()> {
        let WebAuthBackend::Sqlite { conn } = &self.backend else {
            return Ok(());
        };
        let conn = conn.lock().map_err(lock_err)?;
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

            CREATE TABLE IF NOT EXISTS web_user_external_state (
                user_id TEXT PRIMARY KEY,
                email_address TEXT,
                email_verified_at TEXT,
                registration_policy TEXT NOT NULL DEFAULT 'cn_domestic',
                whop_membership_json TEXT,
                email_challenge_json TEXT,
                FOREIGN KEY(user_id) REFERENCES web_invite_users(user_id) ON DELETE CASCADE
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_web_user_external_email
                ON web_user_external_state(email_address)
                WHERE email_address IS NOT NULL;

            CREATE TABLE IF NOT EXISTS web_admin_actions (
                action_id INTEGER PRIMARY KEY AUTOINCREMENT,
                admin_user_id TEXT NOT NULL,
                target_user_id TEXT NOT NULL,
                action TEXT NOT NULL CHECK (action IN ('create', 'disable')),
                beijing_date TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(admin_user_id) REFERENCES web_invite_users(user_id),
                FOREIGN KEY(target_user_id) REFERENCES web_invite_users(user_id)
            );

            CREATE INDEX IF NOT EXISTS idx_web_admin_actions_daily
                ON web_admin_actions(admin_user_id, beijing_date, action);
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
        ensure_column(&conn, "web_invite_users", "password_hash", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "password_set_at", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "tos_accepted_at", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "tos_version", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "api_key_hash", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "api_key_prefix", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "api_key_created_at", "TEXT")?;
        ensure_column(&conn, "web_invite_users", "api_key_last_used_at", "TEXT")?;
        ensure_column(
            &conn,
            "web_invite_users",
            "is_admin",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        conn.execute(
            "
            CREATE UNIQUE INDEX IF NOT EXISTS idx_web_invite_users_api_key_hash
                ON web_invite_users(api_key_hash)
                WHERE api_key_hash IS NOT NULL
            ",
            [],
        )
        .map_err(sql_err)?;
        Ok(())
    }

    fn sqlite_conn(&self) -> HoneResult<MutexGuard<'_, Connection>> {
        match &self.backend {
            WebAuthBackend::Sqlite { conn } => conn.lock().map_err(lock_err),
            WebAuthBackend::Cloud { .. } => Err(HoneError::Storage(
                "web auth sqlite connection requested in cloud mode".to_string(),
            )),
        }
    }

    fn cloud_postgres(&self) -> Option<CloudPgRuntime> {
        match &self.backend {
            WebAuthBackend::Cloud { postgres } => Some(postgres.clone()),
            WebAuthBackend::Sqlite { .. } => None,
        }
    }

    fn cloud_upsert_invite(
        &self,
        user: &WebInviteUser,
        api_key_hash: Option<String>,
    ) -> HoneResult<()> {
        let external_state = self
            .cloud_find_record_by("user_id", &user.user_id)?
            .map(|record| record.external_state)
            .unwrap_or_default();
        self.cloud_upsert_invite_with_state(user, api_key_hash, external_state)
    }

    fn cloud_upsert_invite_with_state(
        &self,
        user: &WebInviteUser,
        api_key_hash: Option<String>,
        external_state: WebUserExternalState,
    ) -> HoneResult<()> {
        let Some(postgres) = self.cloud_postgres() else {
            return Ok(());
        };
        let record = CloudWebInviteRecord {
            user: user.clone(),
            api_key_hash,
            external_state,
        };
        let value = serde_json::to_value(&record)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let user_id = user.user_id.clone();
        let phone_number = user.phone_number.clone();
        run_cloud_web_auth(async move {
            postgres
                .upsert_web_invite_user_record(&user_id, &phone_number, value)
                .await
        })
    }

    fn cloud_record_to_user(
        value: serde_json::Value,
    ) -> HoneResult<(WebInviteUser, Option<String>)> {
        let record: CloudWebInviteRecord = serde_json::from_value(value)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        Ok((record.user, record.api_key_hash))
    }

    fn cloud_find_record_by(
        &self,
        field: &str,
        value: &str,
    ) -> HoneResult<Option<CloudWebInviteRecord>> {
        let Some(postgres) = self.cloud_postgres() else {
            return Ok(None);
        };
        let field = field.to_string();
        let value = value.to_string();
        run_cloud_web_auth(
            async move { postgres.find_web_invite_user_record(&field, &value).await },
        )?
        .map(|value| {
            serde_json::from_value(value).map_err(|err| HoneError::Serialization(err.to_string()))
        })
        .transpose()
    }

    fn cloud_find_invite_by(
        &self,
        field: &str,
        value: &str,
    ) -> HoneResult<Option<(WebInviteUser, Option<String>)>> {
        Ok(self
            .cloud_find_record_by(field, value)?
            .map(|record| (record.user, record.api_key_hash)))
    }

    fn cloud_list_records(&self) -> HoneResult<Vec<CloudWebInviteRecord>> {
        let Some(postgres) = self.cloud_postgres() else {
            return Ok(Vec::new());
        };
        run_cloud_web_auth(async move { postgres.list_web_invite_user_records().await })?
            .into_iter()
            .map(|value| {
                serde_json::from_value(value)
                    .map_err(|err| HoneError::Serialization(err.to_string()))
            })
            .collect()
    }

    fn load_external_state(&self, user_id: &str) -> HoneResult<WebUserExternalState> {
        if self.cloud_postgres().is_some() {
            return Ok(self
                .cloud_find_record_by("user_id", user_id)?
                .map(|record| record.external_state)
                .unwrap_or_default());
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT email_address, email_verified_at, registration_policy,
                   whop_membership_json, email_challenge_json
            FROM web_user_external_state
            WHERE user_id = ?1
            ",
            params![user_id],
            external_state_from_row,
        )
        .optional()
        .map(|state| state.unwrap_or_default())
        .map_err(sql_err)
    }

    fn save_external_state(
        &self,
        user: &WebInviteUser,
        api_key_hash: Option<String>,
        state: WebUserExternalState,
    ) -> HoneResult<()> {
        if self.cloud_postgres().is_some() {
            return self.cloud_upsert_invite_with_state(user, api_key_hash, state);
        }
        let membership_json = state
            .profile
            .whop_membership
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let challenge_json = state
            .email_challenge
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let conn = self.sqlite_conn()?;
        conn.execute(
            "
            INSERT INTO web_user_external_state(
                user_id, email_address, email_verified_at, registration_policy,
                whop_membership_json, email_challenge_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(user_id)
            DO UPDATE SET
                email_address = excluded.email_address,
                email_verified_at = excluded.email_verified_at,
                registration_policy = excluded.registration_policy,
                whop_membership_json = excluded.whop_membership_json,
                email_challenge_json = excluded.email_challenge_json
            ",
            params![
                &user.user_id,
                &state.profile.email_address,
                &state.profile.email_verified_at,
                &state.profile.registration_policy,
                membership_json,
                challenge_json,
            ],
        )
        .map_err(sql_err)?;
        Ok(())
    }

    fn find_external_user_by_email(
        &self,
        email_address: &str,
    ) -> HoneResult<Option<(WebInviteUser, Option<String>, WebUserExternalState)>> {
        let email = normalize_email_address(email_address)?;
        if self.cloud_postgres().is_some() {
            return Ok(self.cloud_list_records()?.into_iter().find_map(|record| {
                (record.external_state.profile.email_address.as_deref() == Some(email.as_str()))
                    .then_some((record.user, record.api_key_hash, record.external_state))
            }));
        }
        let user_id = {
            let conn = self.sqlite_conn()?;
            conn.query_row(
                "SELECT user_id FROM web_user_external_state WHERE email_address = ?1",
                params![email],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(sql_err)?
        };
        let Some(user_id) = user_id else {
            return Ok(None);
        };
        let Some(user) = self.find_invite_user(&user_id)? else {
            return Ok(None);
        };
        let state = self.load_external_state(&user_id)?;
        Ok(Some((user, None, state)))
    }

    fn find_external_user_by_membership(
        &self,
        membership_id: &str,
    ) -> HoneResult<Option<(WebInviteUser, Option<String>, WebUserExternalState)>> {
        if self.cloud_postgres().is_some() {
            return Ok(self.cloud_list_records()?.into_iter().find_map(|record| {
                (record
                    .external_state
                    .profile
                    .whop_membership
                    .as_ref()
                    .is_some_and(|membership| membership.membership_id == membership_id))
                .then_some((record.user, record.api_key_hash, record.external_state))
            }));
        }
        let user_id = {
            let conn = self.sqlite_conn()?;
            let mut stmt = conn
                .prepare(
                    "
                    SELECT user_id, whop_membership_json
                    FROM web_user_external_state
                    WHERE whop_membership_json IS NOT NULL
                    ",
                )
                .map_err(sql_err)?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(sql_err)?;
            let mut matched = None;
            for row in rows {
                let (user_id, raw) = row.map_err(sql_err)?;
                let membership: WhopMembershipRecord = serde_json::from_str(&raw)
                    .map_err(|err| HoneError::Serialization(err.to_string()))?;
                if membership.membership_id == membership_id {
                    matched = Some(user_id);
                    break;
                }
            }
            matched
        };
        let Some(user_id) = user_id else {
            return Ok(None);
        };
        let Some(user) = self.find_invite_user(&user_id)? else {
            return Ok(None);
        };
        let state = self.load_external_state(&user_id)?;
        Ok(Some((user, None, state)))
    }

    fn cloud_upsert_session(&self, session: &CloudWebAuthSessionRecord) -> HoneResult<()> {
        let Some(postgres) = self.cloud_postgres() else {
            return Ok(());
        };
        let record = serde_json::to_value(session)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let session_hash = session.session_hash.clone();
        let user_id = session.user_id.clone();
        let expires_at = session.expires_at.clone();
        run_cloud_web_auth(async move {
            postgres
                .upsert_web_auth_session_record(&session_hash, &user_id, record, Some(&expires_at))
                .await
        })
    }

    fn cloud_purge_expired_sessions(&self, now: &str) -> HoneResult<()> {
        let Some(postgres) = self.cloud_postgres() else {
            return Ok(());
        };
        let now = now.to_string();
        run_cloud_web_auth(async move {
            postgres.purge_expired_web_auth_sessions(&now).await?;
            Ok(())
        })
    }

    pub fn is_web_admin(&self, user_id: &str) -> HoneResult<bool> {
        if let Some(postgres) = self.cloud_postgres() {
            let user_id = user_id.to_string();
            return run_cloud_web_auth(
                async move { postgres.web_invite_user_is_admin(&user_id).await },
            );
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "SELECT is_admin FROM web_invite_users WHERE user_id = ?1",
            params![user_id],
            |row| row.get::<_, bool>(0),
        )
        .optional()
        .map(|value| value.unwrap_or(false))
        .map_err(sql_err)
    }

    pub fn set_web_admin_by_phone(
        &self,
        phone_number: &str,
        is_admin: bool,
    ) -> HoneResult<Option<String>> {
        let phone_number = validate_phone_number(phone_number)?;
        if let Some(postgres) = self.cloud_postgres() {
            return run_cloud_web_auth(async move {
                postgres
                    .set_web_invite_user_admin_by_phone(&phone_number, is_admin)
                    .await
            });
        }
        let conn = self.sqlite_conn()?;
        let user_ids = {
            let mut statement = conn
                .prepare("SELECT user_id FROM web_invite_users WHERE phone_number = ?1")
                .map_err(sql_err)?;
            statement
                .query_map(params![&phone_number], |row| row.get::<_, String>(0))
                .map_err(sql_err)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_err)?
        };
        match user_ids.as_slice() {
            [] => Ok(None),
            [user_id] => {
                conn.execute(
                    "UPDATE web_invite_users SET is_admin = ?2 WHERE user_id = ?1",
                    params![user_id, is_admin],
                )
                .map_err(sql_err)?;
                Ok(Some(user_id.clone()))
            }
            _ => Err(HoneError::Storage(format!(
                "手机号 {phone_number} 对应多个 Web 用户，拒绝设置管理员"
            ))),
        }
    }

    pub fn web_admin_create_count_today(&self, admin_user_id: &str) -> HoneResult<u32> {
        let beijing_date = beijing_now().format("%F").to_string();
        if let Some(postgres) = self.cloud_postgres() {
            let admin_user_id = admin_user_id.to_string();
            return run_cloud_web_auth(async move {
                postgres
                    .web_admin_create_count_for_date(&admin_user_id, &beijing_date)
                    .await
            });
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            r#"
SELECT count(*)
FROM web_admin_actions
WHERE admin_user_id = ?1
  AND beijing_date = ?2
  AND action = 'create'
"#,
            params![admin_user_id, beijing_date],
            |row| row.get::<_, u32>(0),
        )
        .map_err(sql_err)
    }

    pub fn create_invite_user_by_admin(
        &self,
        admin_user_id: &str,
        phone_number: &str,
    ) -> HoneResult<WebAdminInviteCreateOutcome> {
        let now = beijing_now();
        let created_at = now.to_rfc3339();
        let beijing_date = now.format("%F").to_string();
        let user_id = generate_user_id();
        let phone_number = validate_phone_number(phone_number)?;

        if let Some(postgres) = self.cloud_postgres() {
            let invite_code = generate_unique_invite_code_cloud(self)?;
            let api_key = generate_unique_api_key_cloud(self)?;
            let api_key_hash = hash_api_key(&api_key);
            let api_key_prefix = api_key_prefix(&api_key);
            let user = WebInviteUser {
                user_id: user_id.clone(),
                invite_code,
                phone_number: phone_number.clone(),
                created_at: created_at.clone(),
                last_login_at: None,
                revoked_at: None,
                password_hash: None,
                password_set_at: None,
                tos_accepted_at: None,
                tos_version: None,
                api_key_prefix: Some(api_key_prefix),
                api_key_created_at: Some(created_at),
                api_key_last_used_at: None,
                api_key_plaintext: Some(api_key),
            };
            let record = CloudWebInviteRecord {
                user: user.clone(),
                api_key_hash: Some(api_key_hash),
                external_state: WebUserExternalState::default(),
            };
            let record = serde_json::to_value(record)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            let admin_user_id = admin_user_id.to_string();
            let outcome = run_cloud_web_auth(async move {
                postgres
                    .create_web_invite_user_record_by_admin(
                        &admin_user_id,
                        &user_id,
                        &phone_number,
                        record,
                        &beijing_date,
                        WEB_ADMIN_DAILY_INVITE_LIMIT,
                    )
                    .await
            })?;
            return Ok(match outcome {
                CloudWebAdminCreateOutcome::Created { used_today } => {
                    WebAdminInviteCreateOutcome::Created {
                        invite: user,
                        used_today,
                    }
                }
                CloudWebAdminCreateOutcome::NotAdmin => WebAdminInviteCreateOutcome::NotAdmin,
                CloudWebAdminCreateOutcome::LimitReached { used_today } => {
                    WebAdminInviteCreateOutcome::LimitReached { used_today }
                }
                CloudWebAdminCreateOutcome::DuplicatePhone => {
                    WebAdminInviteCreateOutcome::DuplicatePhone
                }
            });
        }

        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let admin_allowed = tx
            .query_row(
                r#"
SELECT 1
FROM web_invite_users
WHERE user_id = ?1
  AND is_admin = 1
  AND revoked_at IS NULL
"#,
                params![admin_user_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(sql_err)?
            .is_some();
        if !admin_allowed {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteCreateOutcome::NotAdmin);
        }
        let duplicate = tx
            .query_row(
                "SELECT 1 FROM web_invite_users WHERE phone_number = ?1 LIMIT 1",
                params![&phone_number],
                |_| Ok(()),
            )
            .optional()
            .map_err(sql_err)?
            .is_some();
        if duplicate {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteCreateOutcome::DuplicatePhone);
        }
        let used_today = tx
            .query_row(
                r#"
SELECT count(*)
FROM web_admin_actions
WHERE admin_user_id = ?1
  AND beijing_date = ?2
  AND action = 'create'
"#,
                params![admin_user_id, &beijing_date],
                |row| row.get::<_, u32>(0),
            )
            .map_err(sql_err)?;
        if used_today >= WEB_ADMIN_DAILY_INVITE_LIMIT {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteCreateOutcome::LimitReached { used_today });
        }

        let invite_code = generate_unique_invite_code(&tx)?;
        let api_key = generate_unique_api_key(&tx)?;
        let api_key_hash = hash_api_key(&api_key);
        let api_key_prefix = api_key_prefix(&api_key);
        tx.execute(
            r#"
INSERT INTO web_invite_users (
  user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
  api_key_hash, api_key_prefix, api_key_created_at, api_key_last_used_at,
  is_admin
)
VALUES (?1, ?2, ?3, ?4, NULL, NULL, ?5, ?6, ?4, NULL, 0)
"#,
            params![
                &user_id,
                &invite_code,
                &phone_number,
                &created_at,
                &api_key_hash,
                &api_key_prefix,
            ],
        )
        .map_err(sql_err)?;
        tx.execute(
            r#"
INSERT INTO web_admin_actions(
  admin_user_id, target_user_id, action, beijing_date, created_at
)
VALUES (?1, ?2, 'create', ?3, ?4)
"#,
            params![admin_user_id, &user_id, &beijing_date, &created_at],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)?;

        Ok(WebAdminInviteCreateOutcome::Created {
            used_today: used_today.saturating_add(1),
            invite: WebInviteUser {
                user_id,
                invite_code,
                phone_number,
                created_at: created_at.clone(),
                last_login_at: None,
                revoked_at: None,
                password_hash: None,
                password_set_at: None,
                tos_accepted_at: None,
                tos_version: None,
                api_key_prefix: Some(api_key_prefix),
                api_key_created_at: Some(created_at),
                api_key_last_used_at: None,
                api_key_plaintext: Some(api_key),
            },
        })
    }

    pub fn disable_invite_user_by_admin(
        &self,
        admin_user_id: &str,
        target_user_id: &str,
    ) -> HoneResult<WebAdminInviteDisableOutcome> {
        let now = beijing_now();
        let beijing_date = now.format("%F").to_string();
        let now = now.to_rfc3339();
        if let Some(postgres) = self.cloud_postgres() {
            let admin_user_id = admin_user_id.to_string();
            let target_user_id = target_user_id.to_string();
            let outcome = run_cloud_web_auth(async move {
                postgres
                    .disable_web_invite_user_by_admin(
                        &admin_user_id,
                        &target_user_id,
                        &now,
                        &beijing_date,
                    )
                    .await
            })?;
            return Ok(match outcome {
                CloudWebAdminDisableOutcome::Disabled {
                    record,
                    cleared_session_count,
                } => {
                    let (invite, _) = Self::cloud_record_to_user(record)?;
                    WebAdminInviteDisableOutcome::Disabled(WebInviteMutation {
                        invite,
                        cleared_session_count,
                    })
                }
                CloudWebAdminDisableOutcome::AlreadyDisabled { record } => {
                    let (invite, _) = Self::cloud_record_to_user(record)?;
                    WebAdminInviteDisableOutcome::AlreadyDisabled(invite)
                }
                CloudWebAdminDisableOutcome::NotAdmin => WebAdminInviteDisableOutcome::NotAdmin,
                CloudWebAdminDisableOutcome::NotFound => WebAdminInviteDisableOutcome::NotFound,
                CloudWebAdminDisableOutcome::ProtectedAdmin => {
                    WebAdminInviteDisableOutcome::ProtectedAdmin
                }
            });
        }

        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let admin_allowed = tx
            .query_row(
                r#"
SELECT 1
FROM web_invite_users
WHERE user_id = ?1
  AND is_admin = 1
  AND revoked_at IS NULL
"#,
                params![admin_user_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(sql_err)?
            .is_some();
        if !admin_allowed {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteDisableOutcome::NotAdmin);
        }
        if admin_user_id == target_user_id {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteDisableOutcome::ProtectedAdmin);
        }
        let target = tx
            .query_row(
                r#"
SELECT is_admin, revoked_at
FROM web_invite_users
WHERE user_id = ?1
"#,
                params![target_user_id],
                |row| Ok((row.get::<_, bool>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(sql_err)?;
        let Some((target_is_admin, target_revoked_at)) = target else {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteDisableOutcome::NotFound);
        };
        if target_is_admin {
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteDisableOutcome::ProtectedAdmin);
        }
        if target_revoked_at.is_some() {
            let invite = find_invite_user_tx(&tx, target_user_id)?.ok_or_else(|| {
                HoneError::Storage("web invite disappeared during admin disable".to_string())
            })?;
            tx.rollback().map_err(sql_err)?;
            return Ok(WebAdminInviteDisableOutcome::AlreadyDisabled(invite));
        }

        let cleared_session_count = delete_sessions_for_user_tx(&tx, target_user_id)? as u32;
        tx.execute(
            "UPDATE web_invite_users SET revoked_at = ?2 WHERE user_id = ?1",
            params![target_user_id, &now],
        )
        .map_err(sql_err)?;
        tx.execute(
            r#"
INSERT INTO web_admin_actions(
  admin_user_id, target_user_id, action, beijing_date, created_at
)
VALUES (?1, ?2, 'disable', ?3, ?4)
"#,
            params![admin_user_id, target_user_id, &beijing_date, &now],
        )
        .map_err(sql_err)?;
        let invite = find_invite_user_tx(&tx, target_user_id)?.ok_or_else(|| {
            HoneError::Storage("web invite disappeared during admin disable".to_string())
        })?;
        tx.commit().map_err(sql_err)?;
        Ok(WebAdminInviteDisableOutcome::Disabled(WebInviteMutation {
            invite,
            cleared_session_count,
        }))
    }

    pub fn create_invite_user(&self, phone_number: &str) -> HoneResult<WebInviteUser> {
        let created_at = beijing_now_rfc3339();
        let user_id = generate_user_id();
        let phone_number = validate_phone_number(phone_number)?;
        if self.cloud_postgres().is_some() {
            let invite_code = generate_unique_invite_code_cloud(self)?;
            let api_key = generate_unique_api_key_cloud(self)?;
            let api_key_hash = hash_api_key(&api_key);
            let api_key_prefix = api_key_prefix(&api_key);
            let user = WebInviteUser {
                user_id,
                invite_code,
                phone_number,
                created_at: created_at.clone(),
                last_login_at: None,
                revoked_at: None,
                password_hash: None,
                password_set_at: None,
                tos_accepted_at: None,
                tos_version: None,
                api_key_prefix: Some(api_key_prefix),
                api_key_created_at: Some(created_at),
                api_key_last_used_at: None,
                api_key_plaintext: Some(api_key),
            };
            self.cloud_upsert_invite(&user, Some(api_key_hash))?;
            return Ok(user);
        }
        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let invite_code = generate_unique_invite_code(&tx)?;
        let api_key = generate_unique_api_key(&tx)?;
        let api_key_hash = hash_api_key(&api_key);
        let api_key_prefix = api_key_prefix(&api_key);
        tx.execute(
            "
            INSERT INTO web_invite_users (
                user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                api_key_hash, api_key_prefix, api_key_created_at, api_key_last_used_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ",
            params![
                &user_id,
                &invite_code,
                &phone_number,
                &created_at,
                None::<String>,
                None::<String>,
                &api_key_hash,
                &api_key_prefix,
                &created_at,
                None::<String>
            ],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)?;

        Ok(WebInviteUser {
            user_id,
            invite_code,
            phone_number,
            created_at: created_at.clone(),
            last_login_at: None,
            revoked_at: None,
            password_hash: None,
            password_set_at: None,
            tos_accepted_at: None,
            tos_version: None,
            api_key_prefix: Some(api_key_prefix),
            api_key_created_at: Some(created_at),
            api_key_last_used_at: None,
            api_key_plaintext: Some(api_key),
        })
    }

    pub fn external_profile(&self, user_id: &str) -> HoneResult<WebUserExternalProfile> {
        Ok(self.load_external_state(user_id)?.profile)
    }

    pub fn find_user_by_email(&self, email_address: &str) -> HoneResult<Option<WebInviteUser>> {
        Ok(self
            .find_external_user_by_email(email_address)?
            .map(|(user, _, _)| user))
    }

    pub fn user_has_paid_access(&self, user_id: &str) -> HoneResult<bool> {
        let state = self.load_external_state(user_id)?;
        if state.profile.registration_policy != REGISTRATION_POLICY_WHOP_INTERNATIONAL {
            return Ok(true);
        }
        Ok(state
            .profile
            .whop_membership
            .as_ref()
            .is_some_and(WhopMembershipRecord::grants_paid_access))
    }

    pub fn upsert_whop_membership(
        &self,
        event: WhopMembershipEvent,
    ) -> HoneResult<(WebInviteUser, WhopMembershipUpsertOutcome)> {
        validate_whop_membership_event(&event)?;
        let email_address = normalize_email_address(&event.email_address)?;
        let existing_by_membership = self.find_external_user_by_membership(&event.membership_id)?;
        let existing_by_email = self.find_external_user_by_email(&email_address)?;
        let existing = match (existing_by_membership, existing_by_email) {
            (Some(by_membership), Some(by_email))
                if by_membership.0.user_id != by_email.0.user_id =>
            {
                return Err(HoneError::Storage(
                    "Whop membership 与付款邮箱已绑定到不同 HONE 用户".to_string(),
                ));
            }
            (Some(record), _) | (_, Some(record)) => Some(record),
            (None, None) => None,
        };

        let membership = WhopMembershipRecord {
            membership_id: event.membership_id,
            whop_user_id: event.whop_user_id,
            company_id: event.company_id,
            product_id: event.product_id,
            plan_id: event.plan_id,
            status: event.status,
            manage_url: event.manage_url,
            renewal_period_start: event.renewal_period_start,
            renewal_period_end: event.renewal_period_end,
            cancel_at_period_end: event.cancel_at_period_end,
            last_event_id: event.event_id,
            last_event_at: event.event_at,
            updated_at: beijing_now_rfc3339(),
        };

        if let Some((user, api_key_hash, mut state)) = existing {
            if let Some(current) = state.profile.whop_membership.as_ref() {
                if current.whop_user_id != membership.whop_user_id {
                    return Err(HoneError::Storage(
                        "付款邮箱已绑定到另一个 Whop 用户".to_string(),
                    ));
                }
                if current.membership_id != membership.membership_id {
                    // A repurchase can create a fresh membership ID for the same Whop
                    // user. Only an access-granting, newer membership may replace the
                    // current record. Late deactivation events from the old membership
                    // must not revoke the newly purchased access.
                    if !membership.grants_paid_access()
                        || !event_is_older(&current.last_event_at, &membership.last_event_at)?
                    {
                        return Ok((user, WhopMembershipUpsertOutcome::Stale));
                    }
                }
                if current.last_event_id == membership.last_event_id {
                    return Ok((user, WhopMembershipUpsertOutcome::Duplicate));
                }
                if current.membership_id == membership.membership_id
                    && event_is_older(&membership.last_event_at, &current.last_event_at)?
                {
                    return Ok((user, WhopMembershipUpsertOutcome::Stale));
                }
            }
            if state.profile.email_verified_at.is_some()
                && state.profile.email_address.as_deref() != Some(email_address.as_str())
            {
                return Err(HoneError::Storage(
                    "已验证邮箱与 Whop 付款邮箱不一致，拒绝自动覆盖".to_string(),
                ));
            }
            state.profile.email_address = Some(email_address);
            state.profile.registration_policy = REGISTRATION_POLICY_WHOP_INTERNATIONAL.to_string();
            state.profile.whop_membership = Some(membership);
            self.save_external_state(&user, api_key_hash, state)?;
            return Ok((user, WhopMembershipUpsertOutcome::Updated));
        }

        let user = self.create_whop_user(email_address, membership)?;
        Ok((user, WhopMembershipUpsertOutcome::Created))
    }

    pub fn begin_email_verification(
        &self,
        email_address: &str,
        ttl_minutes: i64,
    ) -> HoneResult<Option<String>> {
        let Some((user, api_key_hash, mut state)) =
            self.find_external_user_by_email(email_address)?
        else {
            return Ok(None);
        };
        let now = beijing_now();
        let code = generate_email_verification_code();
        state.email_challenge = Some(EmailVerificationChallenge {
            code_hash: hash_email_verification_code(&user.user_id, &code),
            requested_at: now.to_rfc3339(),
            expires_at: (now + chrono::Duration::minutes(ttl_minutes.max(1))).to_rfc3339(),
            attempts: 0,
        });
        self.save_external_state(&user, api_key_hash, state)?;
        Ok(Some(code))
    }

    pub fn verify_email_code(
        &self,
        email_address: &str,
        code: &str,
    ) -> HoneResult<EmailVerificationResult> {
        let Some((user, api_key_hash, mut state)) =
            self.find_external_user_by_email(email_address)?
        else {
            return Ok(EmailVerificationResult::Missing);
        };
        let Some(mut challenge) = state.email_challenge.take() else {
            return Ok(EmailVerificationResult::Missing);
        };
        if challenge.attempts >= 5 {
            state.email_challenge = Some(challenge);
            self.save_external_state(&user, api_key_hash, state)?;
            return Ok(EmailVerificationResult::AttemptsExceeded);
        }
        if challenge.expires_at <= beijing_now_rfc3339() {
            self.save_external_state(&user, api_key_hash, state)?;
            return Ok(EmailVerificationResult::Expired);
        }
        let normalized_code = normalize_email_verification_code(code);
        if hash_email_verification_code(&user.user_id, &normalized_code) != challenge.code_hash {
            challenge.attempts = challenge.attempts.saturating_add(1);
            let exhausted = challenge.attempts >= 5;
            state.email_challenge = Some(challenge);
            self.save_external_state(&user, api_key_hash, state)?;
            return Ok(if exhausted {
                EmailVerificationResult::AttemptsExceeded
            } else {
                EmailVerificationResult::Invalid
            });
        }
        state.profile.email_verified_at = Some(beijing_now_rfc3339());
        self.save_external_state(&user, api_key_hash, state)?;
        Ok(EmailVerificationResult::Verified {
            user_id: user.user_id,
        })
    }

    fn create_whop_user(
        &self,
        email_address: String,
        membership: WhopMembershipRecord,
    ) -> HoneResult<WebInviteUser> {
        let created_at = beijing_now_rfc3339();
        let user_id = generate_user_id();
        let external_state = WebUserExternalState {
            profile: WebUserExternalProfile {
                email_address: Some(email_address),
                email_verified_at: None,
                registration_policy: REGISTRATION_POLICY_WHOP_INTERNATIONAL.to_string(),
                whop_membership: Some(membership),
            },
            email_challenge: None,
        };
        if self.cloud_postgres().is_some() {
            let user = WebInviteUser {
                user_id,
                invite_code: generate_unique_invite_code_cloud(self)?,
                phone_number: String::new(),
                created_at,
                last_login_at: None,
                revoked_at: None,
                password_hash: None,
                password_set_at: None,
                tos_accepted_at: None,
                tos_version: None,
                api_key_prefix: None,
                api_key_created_at: None,
                api_key_last_used_at: None,
                api_key_plaintext: None,
            };
            self.cloud_upsert_invite_with_state(&user, None, external_state)?;
            return Ok(user);
        }

        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let invite_code = generate_unique_invite_code(&tx)?;
        tx.execute(
            "
            INSERT INTO web_invite_users(
                user_id, invite_code, phone_number, created_at, last_login_at, revoked_at
            )
            VALUES (?1, ?2, '', ?3, NULL, NULL)
            ",
            params![&user_id, &invite_code, &created_at],
        )
        .map_err(sql_err)?;
        let membership_json = serde_json::to_string(
            external_state
                .profile
                .whop_membership
                .as_ref()
                .expect("membership"),
        )
        .map_err(|err| HoneError::Serialization(err.to_string()))?;
        tx.execute(
            "
            INSERT INTO web_user_external_state(
                user_id, email_address, email_verified_at, registration_policy,
                whop_membership_json, email_challenge_json
            )
            VALUES (?1, ?2, NULL, ?3, ?4, NULL)
            ",
            params![
                &user_id,
                &external_state.profile.email_address,
                REGISTRATION_POLICY_WHOP_INTERNATIONAL,
                membership_json,
            ],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)?;
        Ok(WebInviteUser {
            user_id,
            invite_code,
            phone_number: String::new(),
            created_at,
            last_login_at: None,
            revoked_at: None,
            password_hash: None,
            password_set_at: None,
            tos_accepted_at: None,
            tos_version: None,
            api_key_prefix: None,
            api_key_created_at: None,
            api_key_last_used_at: None,
            api_key_plaintext: None,
        })
    }

    pub fn list_invite_users(&self) -> HoneResult<Vec<WebInviteUser>> {
        if let Some(postgres) = self.cloud_postgres() {
            let records =
                run_cloud_web_auth(async move { postgres.list_web_invite_user_records().await })?;
            return records
                .into_iter()
                .map(Self::cloud_record_to_user)
                .map(|result| result.map(|(user, _)| user))
                .collect();
        }
        let conn = self.sqlite_conn()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                       password_hash, password_set_at, tos_accepted_at, tos_version,
                       api_key_prefix, api_key_created_at, api_key_last_used_at
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
        if self.cloud_postgres().is_some() {
            return Ok(self
                .cloud_find_invite_by("invite_code", &invite_code)?
                .map(|(user, _)| user));
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                       password_hash, password_set_at, tos_accepted_at, tos_version,
                       api_key_prefix, api_key_created_at, api_key_last_used_at
            FROM web_invite_users
            WHERE invite_code = ?1
            ",
            params![invite_code],
            map_invite_user,
        )
        .optional()
        .map_err(sql_err)
    }

    /// Public SMS login whitelist lookup. Admin-created invite users are the
    /// current whitelist source; revoked users cannot receive or verify codes.
    pub fn find_active_invite_user_by_phone(
        &self,
        phone_number: &str,
    ) -> HoneResult<Option<WebInviteUser>> {
        let phone = normalize_phone_number(phone_number);
        if phone.is_empty() {
            return Ok(None);
        }
        if self.cloud_postgres().is_some() {
            let user = self
                .cloud_find_invite_by("phone_number", &phone)?
                .map(|(user, _)| user)
                .filter(|user| user.revoked_at.is_none());
            return Ok(user);
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                   password_hash, password_set_at, tos_accepted_at, tos_version,
                   api_key_prefix, api_key_created_at, api_key_last_used_at
            FROM web_invite_users
            WHERE phone_number = ?1 AND revoked_at IS NULL
            ",
            params![phone],
            map_invite_user,
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn find_invite_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        if self.cloud_postgres().is_some() {
            return Ok(self
                .cloud_find_invite_by("user_id", user_id)?
                .map(|(user, _)| user));
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                       password_hash, password_set_at, tos_accepted_at, tos_version,
                       api_key_prefix, api_key_created_at, api_key_last_used_at
            FROM web_invite_users
            WHERE user_id = ?1
            ",
            params![user_id],
            map_invite_user,
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn find_invite_user_by_api_key(&self, api_key: &str) -> HoneResult<Option<WebInviteUser>> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Ok(None);
        }
        let now = beijing_now_rfc3339();
        let api_key_hash = hash_api_key(api_key);
        if self.cloud_postgres().is_some() {
            let Some((mut user, stored_hash)) =
                self.cloud_find_invite_by("api_key_hash", &api_key_hash)?
            else {
                return Ok(None);
            };
            if user.revoked_at.is_some() || stored_hash.as_deref() != Some(api_key_hash.as_str()) {
                return Ok(None);
            }
            user.api_key_last_used_at = Some(now);
            user.api_key_plaintext = None;
            self.cloud_upsert_invite(&user, stored_hash)?;
            return Ok(Some(user));
        }
        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let user = tx
            .query_row(
                "
                SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                       password_hash, password_set_at, tos_accepted_at, tos_version,
                       api_key_prefix, api_key_created_at, api_key_last_used_at
                FROM web_invite_users
                WHERE api_key_hash = ?1 AND revoked_at IS NULL
                ",
                params![&api_key_hash],
                map_invite_user,
            )
            .optional()
            .map_err(sql_err)?;
        if let Some(user) = user {
            tx.execute(
                "
                UPDATE web_invite_users
                SET api_key_last_used_at = ?2
                WHERE user_id = ?1
                ",
                params![&user.user_id, &now],
            )
            .map_err(sql_err)?;
            let refreshed = find_invite_user_tx(&tx, &user.user_id)?.ok_or_else(|| {
                HoneError::Storage("web invite disappeared during api key lookup".to_string())
            })?;
            tx.commit().map_err(sql_err)?;
            Ok(Some(refreshed))
        } else {
            tx.commit().map_err(sql_err)?;
            Ok(None)
        }
    }

    pub fn ensure_api_key_for_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        let now = beijing_now_rfc3339();
        if self.cloud_postgres().is_some() {
            let Some((mut existing, _existing_hash)) =
                self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(None);
            };
            if existing.api_key_prefix.is_some() {
                existing.api_key_plaintext = None;
                return Ok(Some(existing));
            }
            let api_key = generate_unique_api_key_cloud(self)?;
            let api_key_hash = hash_api_key(&api_key);
            existing.api_key_prefix = Some(api_key_prefix(&api_key));
            existing.api_key_created_at = Some(now);
            existing.api_key_last_used_at = None;
            existing.api_key_plaintext = Some(api_key);
            self.cloud_upsert_invite(&existing, Some(api_key_hash))?;
            return Ok(Some(existing));
        }
        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let Some(mut existing) = find_invite_user_tx(&tx, user_id)? else {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        };
        if existing.api_key_prefix.is_some() {
            tx.commit().map_err(sql_err)?;
            existing.api_key_plaintext = None;
            return Ok(Some(existing));
        }

        let api_key = generate_unique_api_key(&tx)?;
        let api_key_hash = hash_api_key(&api_key);
        let prefix = api_key_prefix(&api_key);
        tx.execute(
            "
            UPDATE web_invite_users
            SET api_key_hash = ?2,
                api_key_prefix = ?3,
                api_key_created_at = ?4,
                api_key_last_used_at = NULL
            WHERE user_id = ?1
            ",
            params![user_id, &api_key_hash, &prefix, &now],
        )
        .map_err(sql_err)?;
        let mut invite = find_invite_user_tx(&tx, user_id)?.ok_or_else(|| {
            HoneError::Storage("web invite disappeared during api key generate".to_string())
        })?;
        tx.commit().map_err(sql_err)?;
        invite.api_key_plaintext = Some(api_key);
        Ok(Some(invite))
    }

    pub fn reset_api_key_for_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        let now = beijing_now_rfc3339();
        if self.cloud_postgres().is_some() {
            let Some((mut invite, _)) = self.cloud_find_invite_by("user_id", user_id)? else {
                return Ok(None);
            };
            let api_key = generate_unique_api_key_cloud(self)?;
            let api_key_hash = hash_api_key(&api_key);
            invite.api_key_prefix = Some(api_key_prefix(&api_key));
            invite.api_key_created_at = Some(now);
            invite.api_key_last_used_at = None;
            invite.api_key_plaintext = Some(api_key);
            self.cloud_upsert_invite(&invite, Some(api_key_hash))?;
            return Ok(Some(invite));
        }
        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let Some(_) = find_invite_user_tx(&tx, user_id)? else {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        };
        let api_key = generate_unique_api_key(&tx)?;
        let api_key_hash = hash_api_key(&api_key);
        let prefix = api_key_prefix(&api_key);
        tx.execute(
            "
            UPDATE web_invite_users
            SET api_key_hash = ?2,
                api_key_prefix = ?3,
                api_key_created_at = ?4,
                api_key_last_used_at = NULL
            WHERE user_id = ?1
            ",
            params![user_id, &api_key_hash, &prefix, &now],
        )
        .map_err(sql_err)?;
        let mut invite = find_invite_user_tx(&tx, user_id)?.ok_or_else(|| {
            HoneError::Storage("web invite disappeared during api key reset".to_string())
        })?;
        tx.commit().map_err(sql_err)?;
        invite.api_key_plaintext = Some(api_key);
        Ok(Some(invite))
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
        let expires_at = (now + chrono::Duration::days(SESSION_TTL_DAYS_LONG)).to_rfc3339();
        let token = generate_session_token();
        let token_hash = hash_session_token(&token);
        if self.cloud_postgres().is_some() {
            self.cloud_purge_expired_sessions(&created_at)?;
            let Some((mut user, api_key_hash)) =
                self.cloud_find_invite_by("invite_code", &invite_code)?
            else {
                return Ok(None);
            };
            if user.phone_number != phone_number || user.revoked_at.is_some() {
                return Ok(None);
            }
            user.last_login_at = Some(created_at.clone());
            self.cloud_upsert_invite(&user, api_key_hash)?;
            let session = CloudWebAuthSessionRecord {
                session_hash: token_hash,
                user_id: user.user_id.clone(),
                created_at: created_at.clone(),
                expires_at: expires_at.clone(),
                last_seen_at: created_at.clone(),
            };
            self.cloud_upsert_session(&session)?;
            return Ok(Some(WebInviteSession {
                session_token: token,
                user_id: user.user_id,
                created_at: created_at.clone(),
                expires_at,
                last_seen_at: created_at,
            }));
        }
        let conn = self.sqlite_conn()?;
        purge_expired_sessions_inner(&conn, &created_at)?;

        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let user = tx
            .query_row(
                "
                SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                       password_hash, password_set_at, tos_accepted_at, tos_version,
                       api_key_prefix, api_key_created_at, api_key_last_used_at
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
        tx.execute(
            "
            INSERT INTO web_auth_sessions (session_token, user_id, created_at, expires_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                &token_hash,
                &user.user_id,
                &created_at,
                &expires_at,
                &created_at
            ],
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
        match self.authenticate_session_detailed(session_token)? {
            WebSessionAuthResult::Authenticated(user) => Ok(Some(user)),
            WebSessionAuthResult::Missing
            | WebSessionAuthResult::Expired { .. }
            | WebSessionAuthResult::UserRevoked { .. }
            | WebSessionAuthResult::UserMissing { .. } => Ok(None),
        }
    }

    pub fn authenticate_session_detailed(
        &self,
        session_token: &str,
    ) -> HoneResult<WebSessionAuthResult> {
        let now = beijing_now_rfc3339();
        let token_hash = hash_session_token(session_token);
        if let Some(postgres) = self.cloud_postgres() {
            let value = run_cloud_web_auth({
                let token_hash = token_hash.clone();
                let session_token = session_token.to_string();
                async move {
                    postgres
                        .find_web_auth_session_record(&token_hash, &session_token)
                        .await
                }
            })?;
            let Some(value) = value else {
                return Ok(WebSessionAuthResult::Missing);
            };
            let mut session: CloudWebAuthSessionRecord = serde_json::from_value(value)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            if session.expires_at <= now {
                self.delete_session(session_token)?;
                return Ok(WebSessionAuthResult::Expired {
                    user_id: session.user_id,
                });
            }
            let Some((user, _)) = self.cloud_find_invite_by("user_id", &session.user_id)? else {
                return Ok(WebSessionAuthResult::UserMissing {
                    user_id: session.user_id,
                });
            };
            if user.revoked_at.is_some() {
                return Ok(WebSessionAuthResult::UserRevoked {
                    user_id: user.user_id,
                });
            }
            session.last_seen_at = now;
            self.cloud_upsert_session(&session)?;
            return Ok(WebSessionAuthResult::Authenticated(user));
        }
        let conn = self.sqlite_conn()?;
        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let session = tx
            .query_row(
                "
                SELECT session_token, user_id, expires_at
                FROM web_auth_sessions
                WHERE session_token = ?1 OR session_token = ?2
                ",
                params![&token_hash, session_token],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(sql_err)?;
        let Some((stored_token, user_id, expires_at)) = session else {
            tx.commit().map_err(sql_err)?;
            return Ok(WebSessionAuthResult::Missing);
        };
        if expires_at <= now {
            tx.execute(
                "DELETE FROM web_auth_sessions WHERE session_token = ?1",
                params![&stored_token],
            )
            .map_err(sql_err)?;
            tx.commit().map_err(sql_err)?;
            return Ok(WebSessionAuthResult::Expired { user_id });
        }

        let user = tx
            .query_row(
                "
                SELECT u.user_id, u.invite_code, u.phone_number, u.created_at, u.last_login_at, u.revoked_at,
                       u.password_hash, u.password_set_at, u.tos_accepted_at, u.tos_version,
                       u.api_key_prefix, u.api_key_created_at, u.api_key_last_used_at
                FROM web_invite_users u
                WHERE u.user_id = ?1
                ",
                params![&user_id],
                map_invite_user,
            )
            .optional()
            .map_err(sql_err)?;
        let Some(user) = user else {
            tx.commit().map_err(sql_err)?;
            return Ok(WebSessionAuthResult::UserMissing { user_id });
        };
        if user.revoked_at.is_some() {
            tx.commit().map_err(sql_err)?;
            return Ok(WebSessionAuthResult::UserRevoked { user_id });
        }

        // 不做 sliding expiry:`expires_at` 由 session 创建时选择的 TTL
        // (1 天 / 30 天) 决定,访问只更新 `last_seen_at`。否则"不勾选保持登录"
        // 的短 TTL 会被每次访问延到长 TTL,违背用户意图。
        tx.execute(
            "
            UPDATE web_auth_sessions
            SET last_seen_at = ?2
            WHERE session_token = ?1
            ",
            params![&stored_token, now],
        )
        .map_err(sql_err)?;
        tx.commit().map_err(sql_err)?;
        Ok(WebSessionAuthResult::Authenticated(user))
    }

    pub fn delete_session(&self, session_token: &str) -> HoneResult<()> {
        let token_hash = hash_session_token(session_token);
        if let Some(postgres) = self.cloud_postgres() {
            let session_token = session_token.to_string();
            return run_cloud_web_auth(async move {
                postgres
                    .delete_web_auth_session(&token_hash, &session_token)
                    .await
            });
        }
        let conn = self.sqlite_conn()?;
        conn.execute(
            "DELETE FROM web_auth_sessions WHERE session_token = ?1 OR session_token = ?2",
            params![&token_hash, session_token],
        )
        .map_err(sql_err)?;
        Ok(())
    }

    pub fn count_active_sessions_for_user(&self, user_id: &str) -> HoneResult<u32> {
        let now = beijing_now_rfc3339();
        if let Some(postgres) = self.cloud_postgres() {
            self.cloud_purge_expired_sessions(&now)?;
            let user_id = user_id.to_string();
            return run_cloud_web_auth(async move {
                postgres
                    .count_active_web_auth_sessions(&user_id, &now)
                    .await
            });
        }
        let conn = self.sqlite_conn()?;
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
        if self.cloud_postgres().is_some() {
            self.cloud_purge_expired_sessions(&now)?;
            let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(None);
            };
            let cleared_session_count = if revoked {
                let postgres = self.cloud_postgres().expect("cloud postgres");
                let user_id_owned = user_id.to_string();
                run_cloud_web_auth(async move {
                    postgres
                        .delete_web_auth_sessions_for_user(&user_id_owned)
                        .await
                })? as u32
            } else {
                0
            };
            user.revoked_at = if revoked { Some(now) } else { None };
            self.cloud_upsert_invite(&user, api_key_hash)?;
            return Ok(Some(WebInviteMutation {
                invite: user,
                cleared_session_count,
            }));
        }
        let conn = self.sqlite_conn()?;
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
        if self.cloud_postgres().is_some() {
            self.cloud_purge_expired_sessions(&now)?;
            let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(None);
            };
            let invite_code = generate_unique_invite_code_cloud(self)?;
            let postgres = self.cloud_postgres().expect("cloud postgres");
            let user_id_owned = user_id.to_string();
            let cleared_session_count = run_cloud_web_auth(async move {
                postgres
                    .delete_web_auth_sessions_for_user(&user_id_owned)
                    .await
            })? as u32;
            user.invite_code = invite_code;
            user.revoked_at = None;
            self.cloud_upsert_invite(&user, api_key_hash)?;
            return Ok(Some(WebInviteMutation {
                invite: user,
                cleared_session_count,
            }));
        }
        let conn = self.sqlite_conn()?;
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

    /// 查询已设置密码、未吊销的用户,用于手机号+密码登录校验。
    pub fn find_by_phone_password_ready(
        &self,
        phone_number: &str,
    ) -> HoneResult<Option<WebInviteUser>> {
        let phone = normalize_phone_number(phone_number);
        if phone.is_empty() {
            return Ok(None);
        }
        if self.cloud_postgres().is_some() {
            return Ok(self
                .cloud_find_invite_by("phone_number", &phone)?
                .map(|(user, _)| user)
                .filter(|user| user.revoked_at.is_none() && user.password_hash.is_some()));
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                   password_hash, password_set_at, tos_accepted_at, tos_version,
                   api_key_prefix, api_key_created_at, api_key_last_used_at
            FROM web_invite_users
            WHERE phone_number = ?1 AND revoked_at IS NULL AND password_hash IS NOT NULL
            ",
            params![phone],
            map_invite_user,
        )
        .optional()
        .map_err(sql_err)
    }

    /// 首次设置密码 / 同时记录协议接受。用于"强制设密码" guard。
    ///
    /// 返回 Ok(true) 表示成功写入,Ok(false) 表示用户已经有密码(调用方应走
    /// change_password 路径避免覆写)或用户不存在。
    pub fn set_password(
        &self,
        user_id: &str,
        password_hash: &str,
        tos_version: &str,
    ) -> HoneResult<bool> {
        let now = beijing_now_rfc3339();
        if self.cloud_postgres().is_some() {
            let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(false);
            };
            if user.password_hash.is_some() {
                return Ok(false);
            }
            user.password_hash = Some(password_hash.to_string());
            user.password_set_at = Some(now.clone());
            user.tos_accepted_at = Some(now);
            user.tos_version = Some(tos_version.to_string());
            self.cloud_upsert_invite(&user, api_key_hash)?;
            return Ok(true);
        }
        let conn = self.sqlite_conn()?;
        let updated = conn
            .execute(
                "
                UPDATE web_invite_users
                SET password_hash = ?2,
                    password_set_at = ?3,
                    tos_accepted_at = ?3,
                    tos_version = ?4
                WHERE user_id = ?1 AND password_hash IS NULL
                ",
                params![user_id, password_hash, now, tos_version],
            )
            .map_err(sql_err)?;
        Ok(updated > 0)
    }

    /// 已设置密码后用于修改密码(/me 页)。不动 tos_accepted_at / tos_version。
    pub fn change_password(&self, user_id: &str, password_hash: &str) -> HoneResult<bool> {
        let now = beijing_now_rfc3339();
        if self.cloud_postgres().is_some() {
            let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(false);
            };
            if user.password_hash.is_none() {
                return Ok(false);
            }
            user.password_hash = Some(password_hash.to_string());
            user.password_set_at = Some(now);
            self.cloud_upsert_invite(&user, api_key_hash)?;
            return Ok(true);
        }
        let conn = self.sqlite_conn()?;
        let updated = conn
            .execute(
                "
                UPDATE web_invite_users
                SET password_hash = ?2, password_set_at = ?3
                WHERE user_id = ?1 AND password_hash IS NOT NULL
                ",
                params![user_id, password_hash, now],
            )
            .map_err(sql_err)?;
        Ok(updated > 0)
    }

    pub fn record_tos_acceptance(&self, user_id: &str, tos_version: &str) -> HoneResult<bool> {
        let now = beijing_now_rfc3339();
        if self.cloud_postgres().is_some() {
            let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(false);
            };
            if user.revoked_at.is_some() {
                return Ok(false);
            }
            user.tos_accepted_at = Some(now);
            user.tos_version = Some(tos_version.to_string());
            self.cloud_upsert_invite(&user, api_key_hash)?;
            return Ok(true);
        }
        let conn = self.sqlite_conn()?;
        let updated = conn
            .execute(
                "
                UPDATE web_invite_users
                SET tos_accepted_at = ?2,
                    tos_version = ?3
                WHERE user_id = ?1 AND revoked_at IS NULL
                ",
                params![user_id, now, tos_version],
            )
            .map_err(sql_err)?;
        Ok(updated > 0)
    }

    /// 按 user_id 创建 session,TTL 由调用方指定(密码登录根据"保持登录"勾选
    /// 选择 long / short)。普通登录不清理该用户的其它活跃 session,避免
    /// 用户浏览器、自动化健康检查和多设备登录互相踢掉登录态。
    pub fn create_session_for_user(
        &self,
        user_id: &str,
        ttl_days: i64,
    ) -> HoneResult<Option<WebInviteSession>> {
        let now = beijing_now();
        let created_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::days(ttl_days)).to_rfc3339();
        let token = generate_session_token();
        let token_hash = hash_session_token(&token);

        if self.cloud_postgres().is_some() {
            self.cloud_purge_expired_sessions(&created_at)?;
            let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)?
            else {
                return Ok(None);
            };
            if user.revoked_at.is_some() {
                return Ok(None);
            }
            user.last_login_at = Some(created_at.clone());
            self.cloud_upsert_invite(&user, api_key_hash)?;
            let session = CloudWebAuthSessionRecord {
                session_hash: token_hash,
                user_id: user.user_id.clone(),
                created_at: created_at.clone(),
                expires_at: expires_at.clone(),
                last_seen_at: created_at.clone(),
            };
            self.cloud_upsert_session(&session)?;
            return Ok(Some(WebInviteSession {
                session_token: token,
                user_id: user.user_id,
                created_at: created_at.clone(),
                expires_at,
                last_seen_at: created_at,
            }));
        }

        let conn = self.sqlite_conn()?;
        purge_expired_sessions_inner(&conn, &created_at)?;

        let tx = conn.unchecked_transaction().map_err(sql_err)?;
        let Some(user) = find_invite_user_tx(&tx, user_id)? else {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        };
        if user.revoked_at.is_some() {
            tx.rollback().map_err(sql_err)?;
            return Ok(None);
        }

        tx.execute(
            "
            UPDATE web_invite_users
            SET last_login_at = ?2
            WHERE user_id = ?1
            ",
            params![&user.user_id, &created_at],
        )
        .map_err(sql_err)?;
        tx.execute(
            "
            INSERT INTO web_auth_sessions (session_token, user_id, created_at, expires_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                &token_hash,
                &user.user_id,
                &created_at,
                &expires_at,
                &created_at
            ],
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

    pub fn export_cloud_records(
        &self,
    ) -> HoneResult<(
        Vec<hone_core::cloud_runtime::CloudWebInviteUserRecord>,
        Vec<hone_core::cloud_runtime::CloudWebAuthSessionRecord>,
    )> {
        let conn = self.sqlite_conn()?;
        let mut users_stmt = conn
            .prepare(
                "
                SELECT u.user_id, u.invite_code, u.phone_number, u.created_at,
                       u.last_login_at, u.revoked_at, u.password_hash, u.password_set_at,
                       u.tos_accepted_at, u.tos_version, u.api_key_prefix,
                       u.api_key_created_at, u.api_key_last_used_at, u.api_key_hash,
                       e.email_address, e.email_verified_at, e.registration_policy,
                       e.whop_membership_json, e.email_challenge_json
                FROM web_invite_users u
                LEFT JOIN web_user_external_state e ON e.user_id = u.user_id
                ORDER BY u.created_at DESC
                ",
            )
            .map_err(sql_err)?;
        let user_rows = users_stmt
            .query_map([], |row| {
                let user = WebInviteUser {
                    user_id: row.get(0)?,
                    invite_code: row.get(1)?,
                    phone_number: row.get(2)?,
                    created_at: row.get(3)?,
                    last_login_at: row.get(4)?,
                    revoked_at: row.get(5)?,
                    password_hash: row.get(6)?,
                    password_set_at: row.get(7)?,
                    tos_accepted_at: row.get(8)?,
                    tos_version: row.get(9)?,
                    api_key_prefix: row.get(10)?,
                    api_key_created_at: row.get(11)?,
                    api_key_last_used_at: row.get(12)?,
                    api_key_plaintext: None,
                };
                let api_key_hash: Option<String> = row.get(13)?;
                let external_state = external_state_from_values(
                    row.get(14)?,
                    row.get(15)?,
                    row.get(16)?,
                    row.get(17)?,
                    row.get(18)?,
                )?;
                Ok((user, api_key_hash, external_state))
            })
            .map_err(sql_err)?;
        let mut users = Vec::new();
        for row in user_rows {
            let (user, api_key_hash, external_state) = row.map_err(sql_err)?;
            let phone_number = user.phone_number.clone();
            let user_id = user.user_id.clone();
            let record = serde_json::to_value(CloudWebInviteRecord {
                user,
                api_key_hash,
                external_state,
            })
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
            users.push(hone_core::cloud_runtime::CloudWebInviteUserRecord {
                user_id,
                phone_number,
                record,
            });
        }

        let mut sessions_stmt = conn
            .prepare(
                "
                SELECT session_token, user_id, created_at, expires_at, last_seen_at
                FROM web_auth_sessions
                ORDER BY created_at DESC
                ",
            )
            .map_err(sql_err)?;
        let session_rows = sessions_stmt
            .query_map([], |row| {
                Ok(CloudWebAuthSessionRecord {
                    session_hash: row.get(0)?,
                    user_id: row.get(1)?,
                    created_at: row.get(2)?,
                    expires_at: row.get(3)?,
                    last_seen_at: row.get(4)?,
                })
            })
            .map_err(sql_err)?;
        let mut sessions = Vec::new();
        for row in session_rows {
            let session = row.map_err(sql_err)?;
            let record = serde_json::to_value(&session)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            sessions.push(hone_core::cloud_runtime::CloudWebAuthSessionRecord {
                session_hash: session.session_hash,
                user_id: session.user_id,
                expires_at: Some(session.expires_at),
                record,
            });
        }
        Ok((users, sessions))
    }
}

fn run_cloud_web_auth<T, F>(future: F) -> HoneResult<T>
where
    T: Send + 'static,
    F: Future<Output = HoneResult<T>> + Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return std::thread::spawn(move || {
            let runtime =
                tokio::runtime::Runtime::new().map_err(|err| HoneError::Config(err.to_string()))?;
            runtime.block_on(future)
        })
        .join()
        .map_err(|_| HoneError::Storage("cloud web auth worker panicked".to_string()))?;
    }
    let runtime =
        tokio::runtime::Runtime::new().map_err(|err| HoneError::Config(err.to_string()))?;
    runtime.block_on(future)
}

fn generate_unique_invite_code_cloud(storage: &WebAuthStorage) -> HoneResult<String> {
    for _ in 0..16 {
        let invite_code = generate_invite_code();
        if storage
            .cloud_find_invite_by("invite_code", &invite_code)?
            .is_none()
        {
            return Ok(invite_code);
        }
    }
    Err(HoneError::Storage(
        "failed to generate unique cloud web invite code".to_string(),
    ))
}

fn generate_unique_api_key_cloud(storage: &WebAuthStorage) -> HoneResult<String> {
    for _ in 0..16 {
        let api_key = generate_api_key();
        let api_key_hash = hash_api_key(&api_key);
        if storage
            .cloud_find_invite_by("api_key_hash", &api_key_hash)?
            .is_none()
        {
            return Ok(api_key);
        }
    }
    Err(HoneError::Storage(
        "failed to generate unique cloud web api key".to_string(),
    ))
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
    // A single UUID v4 provides more than enough random hex material.
    // We take 20 hex chars (80 bits entropy) for brute-force resistance.
    let token = uuid::Uuid::new_v4().simple().to_string().to_uppercase();
    format!(
        "HONE-{}-{}-{}-{}",
        &token[..5],
        &token[5..10],
        &token[10..15],
        &token[15..20]
    )
}

fn generate_session_token() -> String {
    // 2 x UUID v4 gives 256 bits of CSPRNG-backed entropy and stays
    // cookie-safe without extra encoding.
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn generate_api_key() -> String {
    // 2 x UUID v4 gives 256 bits of random hex material while keeping the key
    // easy to copy into Authorization: Bearer headers.
    format!(
        "hck_{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn hash_session_token(session_token: &str) -> String {
    let digest = Sha256::digest(session_token.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn hash_api_key(api_key: &str) -> String {
    hash_session_token(api_key.trim())
}

fn api_key_prefix(api_key: &str) -> String {
    api_key.chars().take(12).collect()
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
        if ch.is_ascii_digit() || (ch == '+' && normalized.is_empty()) {
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

fn generate_unique_api_key(tx: &Transaction<'_>) -> HoneResult<String> {
    for _ in 0..8 {
        let api_key = generate_api_key();
        let api_key_hash = hash_api_key(&api_key);
        let existing = tx
            .query_row(
                "SELECT api_key_hash FROM web_invite_users WHERE api_key_hash = ?1",
                params![&api_key_hash],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(sql_err)?;
        if existing.is_none() {
            return Ok(api_key);
        }
    }

    Err(HoneError::Storage(
        "failed to generate unique web api key".to_string(),
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

fn external_state_is_default(state: &WebUserExternalState) -> bool {
    state == &WebUserExternalState::default()
}

fn normalize_email_address(email_address: &str) -> HoneResult<String> {
    let normalized = email_address.trim().to_ascii_lowercase();
    let Some((local, domain)) = normalized.split_once('@') else {
        return Err(HoneError::Config("邮箱格式不合法".to_string()));
    };
    if normalized.len() > 254
        || local.len() > 64
        || local.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || !domain.contains('.')
        || local.starts_with('.')
        || local.ends_with('.')
        || domain.starts_with('.')
        || domain.starts_with('-')
        || domain.ends_with('.')
        || domain.ends_with('-')
        || normalized.contains("..")
        || normalized.chars().any(char::is_whitespace)
    {
        return Err(HoneError::Config("邮箱格式不合法".to_string()));
    }
    Ok(normalized)
}

fn validate_whop_membership_event(event: &WhopMembershipEvent) -> HoneResult<()> {
    for (label, value, prefix) in [
        ("membership_id", event.membership_id.as_str(), "mem_"),
        ("whop_user_id", event.whop_user_id.as_str(), "user_"),
        ("company_id", event.company_id.as_str(), "biz_"),
        ("product_id", event.product_id.as_str(), "prod_"),
        ("plan_id", event.plan_id.as_str(), "plan_"),
        ("event_id", event.event_id.as_str(), "msg_"),
    ] {
        if !value.starts_with(prefix) || value.len() <= prefix.len() {
            return Err(HoneError::Config(format!("Whop {label} 格式不合法")));
        }
    }
    if !matches!(
        event.status.as_str(),
        "trialing"
            | "active"
            | "past_due"
            | "completed"
            | "canceled"
            | "expired"
            | "unresolved"
            | "drafted"
            | "canceling"
    ) {
        return Err(HoneError::Config("Whop membership 状态不合法".to_string()));
    }
    chrono::DateTime::parse_from_rfc3339(&event.event_at)
        .map_err(|_| HoneError::Config("Whop event_at 格式不合法".to_string()))?;
    Ok(())
}

fn event_is_older(candidate: &str, current: &str) -> HoneResult<bool> {
    let candidate = chrono::DateTime::parse_from_rfc3339(candidate)
        .map_err(|_| HoneError::Config("Whop candidate event time 格式不合法".to_string()))?;
    let current = chrono::DateTime::parse_from_rfc3339(current)
        .map_err(|_| HoneError::Config("Whop current event time 格式不合法".to_string()))?;
    Ok(candidate < current)
}

fn generate_email_verification_code() -> String {
    let bytes = uuid::Uuid::new_v4();
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&bytes.as_bytes()[..8]);
    format!("{:08}", u64::from_be_bytes(prefix) % 100_000_000)
}

fn normalize_email_verification_code(code: &str) -> String {
    code.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn hash_email_verification_code(user_id: &str, code: &str) -> String {
    hash_session_token(&format!(
        "{user_id}:{}",
        normalize_email_verification_code(code)
    ))
}

fn lock_err<E>(_: E) -> HoneError {
    HoneError::Storage("web auth storage lock poisoned".to_string())
}

fn sql_err(err: rusqlite::Error) -> HoneError {
    HoneError::Storage(format!("web auth sqlite error: {err}"))
}

fn external_state_from_row(row: &Row<'_>) -> rusqlite::Result<WebUserExternalState> {
    external_state_from_values(
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    )
}

fn external_state_from_values(
    email_address: Option<String>,
    email_verified_at: Option<String>,
    registration_policy: Option<String>,
    membership_json: Option<String>,
    challenge_json: Option<String>,
) -> rusqlite::Result<WebUserExternalState> {
    let whop_membership = parse_json_column(membership_json, 3)?;
    let email_challenge = parse_json_column(challenge_json, 4)?;
    Ok(WebUserExternalState {
        profile: WebUserExternalProfile {
            email_address,
            email_verified_at,
            registration_policy: registration_policy
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| REGISTRATION_POLICY_CN_DOMESTIC.to_string()),
            whop_membership,
        },
        email_challenge,
    })
}

fn parse_json_column<T: for<'de> Deserialize<'de>>(
    raw: Option<String>,
    column: usize,
) -> rusqlite::Result<Option<T>> {
    raw.map(|value| {
        serde_json::from_str(&value).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                column,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
    })
    .transpose()
}

fn map_invite_user(row: &Row<'_>) -> rusqlite::Result<WebInviteUser> {
    Ok(WebInviteUser {
        user_id: row.get(0)?,
        invite_code: row.get(1)?,
        phone_number: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        created_at: row.get(3)?,
        last_login_at: row.get(4)?,
        revoked_at: row.get(5)?,
        password_hash: row.get(6)?,
        password_set_at: row.get(7)?,
        tos_accepted_at: row.get(8)?,
        tos_version: row.get(9)?,
        api_key_prefix: row.get(10)?,
        api_key_created_at: row.get(11)?,
        api_key_last_used_at: row.get(12)?,
        api_key_plaintext: None,
    })
}

fn find_invite_user_tx(tx: &Transaction<'_>, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
    tx.query_row(
        "
        SELECT user_id, invite_code, phone_number, created_at, last_login_at, revoked_at,
                       password_hash, password_set_at, tos_accepted_at, tos_version,
                       api_key_prefix, api_key_created_at, api_key_last_used_at
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

/// Add a column to an existing table if it does not already exist.
///
/// # Safety (SQL injection)
///
/// `table`, `column`, and `definition` are interpolated directly into DDL.
/// **All arguments MUST be hard-coded string literals** — never pass values
/// derived from user input or external configuration.
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
    use super::{
        CloudWebInviteRecord, EmailVerificationResult, REGISTRATION_POLICY_WHOP_INTERNATIONAL,
        SESSION_TTL_DAYS_LONG, SESSION_TTL_DAYS_SHORT, WebAdminInviteCreateOutcome,
        WebAdminInviteDisableOutcome, WebAuthStorage, WebSessionAuthResult, WhopMembershipEvent,
        WhopMembershipUpsertOutcome, generate_api_key, generate_invite_code,
        generate_session_token, hash_session_token,
    };
    use hone_core::beijing_now;
    use rusqlite::{Connection, params};

    fn test_storage() -> WebAuthStorage {
        let root = std::env::temp_dir().join(format!("hone_web_auth_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        WebAuthStorage::new(root.join("sessions.sqlite3")).expect("storage")
    }

    fn whop_event(event_id: &str, event_at: &str, status: &str) -> WhopMembershipEvent {
        WhopMembershipEvent {
            membership_id: "mem_test123".to_string(),
            whop_user_id: "user_test123".to_string(),
            email_address: "Buyer@Example.com".to_string(),
            company_id: "biz_test123".to_string(),
            product_id: "prod_test123".to_string(),
            plan_id: "plan_test123".to_string(),
            status: status.to_string(),
            manage_url: Some("https://whop.com/billing/manage/mem_test123".to_string()),
            renewal_period_start: Some("2026-07-26T00:00:00Z".to_string()),
            renewal_period_end: Some("2027-07-26T00:00:00Z".to_string()),
            cancel_at_period_end: status == "canceling",
            event_id: event_id.to_string(),
            event_at: event_at.to_string(),
        }
    }

    #[test]
    fn invite_code_has_sufficient_entropy() {
        let code = generate_invite_code();
        assert!(code.starts_with("HONE-"), "prefix: {code}");
        // HONE- + 4 groups of 5 hex chars separated by dashes = 28 chars total
        assert_eq!(code.len(), 28, "length: {code}");
        // 20 hex characters after the "HONE-" prefix = 80 bits of entropy.
        // Count only the random part (skip the "HONE-" prefix and dashes).
        let random_part = &code["HONE-".len()..];
        let hex_chars: String = random_part
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();
        assert_eq!(hex_chars.len(), 20, "hex chars in random part: {code}");
    }

    #[test]
    fn session_token_has_256_bits_of_hex_entropy() {
        let token = generate_session_token();

        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn api_key_has_hone_cloud_prefix_and_entropy() {
        let key = generate_api_key();
        assert!(key.starts_with("hck_"));
        assert_eq!(key.len(), 68);
    }

    #[test]
    fn cloud_invite_records_never_serialize_or_restore_plaintext_api_keys() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        assert!(created.api_key_plaintext.is_some());

        let record = CloudWebInviteRecord {
            user: created,
            api_key_hash: Some("hashed-key".to_string()),
            external_state: Default::default(),
        };
        let mut value = serde_json::to_value(&record).expect("serialize cloud invite");
        assert!(value.get("api_key_plaintext").is_none());

        value.as_object_mut().expect("object").insert(
            "api_key_plaintext".to_string(),
            serde_json::json!("legacy-secret"),
        );
        let restored: CloudWebInviteRecord =
            serde_json::from_value(value).expect("deserialize legacy cloud invite");
        assert!(restored.user.api_key_plaintext.is_none());
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
        assert!(
            created
                .api_key_plaintext
                .as_deref()
                .is_some_and(|key| key.starts_with("hck_"))
        );
        assert!(listed[0].api_key_plaintext.is_none());
        assert_eq!(listed[0].api_key_prefix, created.api_key_prefix);
    }

    #[test]
    fn active_invite_user_by_phone_is_sms_login_whitelist() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");

        let found = storage
            .find_active_invite_user_by_phone("138-0013-8000")
            .expect("lookup")
            .expect("user");
        assert_eq!(found.user_id, created.user_id);

        storage
            .set_invite_revoked(&created.user_id, true)
            .expect("revoke");
        assert!(
            storage
                .find_active_invite_user_by_phone("13800138000")
                .expect("lookup revoked")
                .is_none()
        );
    }

    #[test]
    fn web_admin_role_is_storage_authoritative() {
        let storage = test_storage();
        let user = storage.create_invite_user("13871396421").expect("create");

        assert!(!storage.is_web_admin(&user.user_id).expect("role"));
        assert_eq!(
            storage
                .set_web_admin_by_phone("138-7139-6421", true)
                .expect("grant")
                .as_deref(),
            Some(user.user_id.as_str())
        );
        assert!(storage.is_web_admin(&user.user_id).expect("role"));
        assert_eq!(
            storage
                .set_web_admin_by_phone("13871396421", false)
                .expect("revoke")
                .as_deref(),
            Some(user.user_id.as_str())
        );
        assert!(!storage.is_web_admin(&user.user_id).expect("role"));
    }

    #[test]
    fn public_admin_create_limit_counts_only_successful_creates() {
        let storage = test_storage();
        let admin = storage.create_invite_user("13871396421").expect("admin");
        storage
            .set_web_admin_by_phone("13871396421", true)
            .expect("grant");

        let first = storage
            .create_invite_user_by_admin(&admin.user_id, "13900000000")
            .expect("first");
        assert!(matches!(
            first,
            WebAdminInviteCreateOutcome::Created { used_today: 1, .. }
        ));
        assert_eq!(
            storage
                .web_admin_create_count_today(&admin.user_id)
                .unwrap(),
            1
        );

        assert_eq!(
            storage
                .create_invite_user_by_admin(&admin.user_id, "13900000000")
                .expect("duplicate"),
            WebAdminInviteCreateOutcome::DuplicatePhone
        );
        assert_eq!(
            storage
                .web_admin_create_count_today(&admin.user_id)
                .unwrap(),
            1,
            "duplicate attempts must not consume the daily allowance"
        );

        for (index, phone) in ["13900000001", "13900000002", "13900000003", "13900000004"]
            .into_iter()
            .enumerate()
        {
            assert!(matches!(
                storage
                    .create_invite_user_by_admin(&admin.user_id, phone)
                    .expect("create"),
                WebAdminInviteCreateOutcome::Created { used_today, .. }
                    if used_today == index as u32 + 2
            ));
        }
        assert_eq!(
            storage
                .create_invite_user_by_admin(&admin.user_id, "13900000005")
                .expect("limit"),
            WebAdminInviteCreateOutcome::LimitReached { used_today: 5 }
        );
        assert_eq!(
            storage
                .web_admin_create_count_today(&admin.user_id)
                .unwrap(),
            5
        );
    }

    #[test]
    fn non_admin_cannot_create_or_disable_whitelist_users() {
        let storage = test_storage();
        let ordinary = storage.create_invite_user("13800138000").expect("ordinary");
        let target = storage.create_invite_user("13900139000").expect("target");

        assert_eq!(
            storage
                .create_invite_user_by_admin(&ordinary.user_id, "13700137000")
                .expect("create"),
            WebAdminInviteCreateOutcome::NotAdmin
        );
        assert_eq!(
            storage
                .disable_invite_user_by_admin(&ordinary.user_id, &target.user_id)
                .expect("disable"),
            WebAdminInviteDisableOutcome::NotAdmin
        );
        assert!(
            storage
                .find_active_invite_user_by_phone("13900139000")
                .expect("target")
                .is_some()
        );
    }

    #[test]
    fn admin_disable_is_audited_clears_sessions_and_protects_admins() {
        let storage = test_storage();
        let admin = storage.create_invite_user("13871396421").expect("admin");
        storage
            .set_web_admin_by_phone("13871396421", true)
            .expect("grant");
        let target = storage.create_invite_user("13900139000").expect("target");
        let session = storage
            .create_session_for_user(&target.user_id, SESSION_TTL_DAYS_LONG)
            .expect("session")
            .expect("session");

        assert_eq!(
            storage
                .disable_invite_user_by_admin(&admin.user_id, &admin.user_id)
                .expect("self"),
            WebAdminInviteDisableOutcome::ProtectedAdmin
        );
        let disabled = storage
            .disable_invite_user_by_admin(&admin.user_id, &target.user_id)
            .expect("disable");
        assert!(matches!(
            disabled,
            WebAdminInviteDisableOutcome::Disabled(super::WebInviteMutation {
                cleared_session_count: 1,
                ..
            })
        ));
        assert!(
            storage
                .authenticate_session(&session.session_token)
                .expect("auth")
                .is_none()
        );
        assert!(matches!(
            storage
                .disable_invite_user_by_admin(&admin.user_id, &target.user_id)
                .expect("idempotent"),
            WebAdminInviteDisableOutcome::AlreadyDisabled(_)
        ));

        let conn = storage.sqlite_conn().expect("conn");
        let audit_count: u32 = conn
            .query_row(
                "SELECT count(*) FROM web_admin_actions WHERE admin_user_id = ?1 AND action = 'disable'",
                params![&admin.user_id],
                |row| row.get(0),
            )
            .expect("audit");
        assert_eq!(audit_count, 1);
    }

    #[test]
    fn record_tos_acceptance_updates_public_login_terms() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");

        assert!(
            storage
                .record_tos_acceptance(&created.user_id, "2.0")
                .expect("record")
        );
        let refreshed = storage
            .find_invite_user(&created.user_id)
            .expect("lookup")
            .expect("user");
        assert_eq!(refreshed.tos_version.as_deref(), Some("2.0"));
        assert!(refreshed.tos_accepted_at.is_some());
    }

    #[test]
    fn api_key_lookup_updates_last_used_and_reset_invalidates_old_key() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let first_key = created.api_key_plaintext.clone().expect("api key");
        let authed = storage
            .find_invite_user_by_api_key(&first_key)
            .expect("lookup")
            .expect("user");
        assert_eq!(authed.user_id, created.user_id);
        assert!(authed.api_key_last_used_at.is_some());

        let reset = storage
            .reset_api_key_for_user(&created.user_id)
            .expect("reset")
            .expect("user");
        let next_key = reset.api_key_plaintext.expect("new api key");
        assert_ne!(first_key, next_key);
        assert!(
            storage
                .find_invite_user_by_api_key(&first_key)
                .expect("old lookup")
                .is_none()
        );
        assert!(
            storage
                .find_invite_user_by_api_key(&next_key)
                .expect("new lookup")
                .is_some()
        );
    }

    #[test]
    fn existing_user_can_generate_api_key_once_without_plaintext_replay() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        {
            let conn = storage.sqlite_conn().expect("conn");
            conn.execute(
                "UPDATE web_invite_users SET api_key_hash = NULL, api_key_prefix = NULL, api_key_created_at = NULL WHERE user_id = ?1",
                params![&created.user_id],
            )
            .expect("clear key");
        }
        let generated = storage
            .ensure_api_key_for_user(&created.user_id)
            .expect("generate")
            .expect("user");
        assert!(generated.api_key_plaintext.is_some());
        let replay = storage
            .ensure_api_key_for_user(&created.user_id)
            .expect("replay")
            .expect("user");
        assert!(replay.api_key_plaintext.is_none());
        assert_eq!(replay.api_key_prefix, generated.api_key_prefix);
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
        let conn = storage.sqlite_conn().expect("conn");
        let stored_token: String = conn
            .query_row(
                "SELECT session_token FROM web_auth_sessions WHERE user_id = ?1",
                params![&created.user_id],
                |row| row.get(0),
            )
            .expect("stored token");
        assert_eq!(stored_token, hash_session_token(&session.session_token));
        assert_ne!(stored_token, session.session_token);
        assert!(session.expires_at > session.created_at);
        assert_eq!(
            (chrono::DateTime::parse_from_rfc3339(&session.expires_at).expect("expiry")
                - chrono::DateTime::parse_from_rfc3339(&session.created_at).expect("created"))
            .num_days(),
            SESSION_TTL_DAYS_LONG
        );
    }

    #[test]
    fn legacy_plaintext_session_tokens_remain_accepted_during_migration() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let now = beijing_now();
        let created_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::days(SESSION_TTL_DAYS_LONG)).to_rfc3339();
        let legacy_token = "legacy-plaintext-session-token";
        {
            let conn = storage.sqlite_conn().expect("conn");
            conn.execute(
                "
                INSERT INTO web_auth_sessions (session_token, user_id, created_at, expires_at, last_seen_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![
                    legacy_token,
                    &created.user_id,
                    &created_at,
                    &expires_at,
                    &created_at
                ],
            )
            .expect("insert legacy session");
        }

        let authed = storage
            .authenticate_session(legacy_token)
            .expect("auth")
            .expect("user");

        assert_eq!(authed.user_id, created.user_id);
    }

    #[test]
    fn detailed_auth_reports_expired_and_missing_sessions() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let now = beijing_now();
        let created_at = (now - chrono::Duration::days(2)).to_rfc3339();
        let expires_at = (now - chrono::Duration::days(1)).to_rfc3339();
        let raw_token = "expired-session-token";
        let token_hash = hash_session_token(raw_token);
        {
            let conn = storage.sqlite_conn().expect("conn");
            conn.execute(
                "
                INSERT INTO web_auth_sessions (session_token, user_id, created_at, expires_at, last_seen_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ",
                params![
                    &token_hash,
                    &created.user_id,
                    &created_at,
                    &expires_at,
                    &created_at
                ],
            )
            .expect("insert expired session");
        }

        assert_eq!(
            storage
                .authenticate_session_detailed(raw_token)
                .expect("auth"),
            WebSessionAuthResult::Expired {
                user_id: created.user_id
            }
        );
        assert_eq!(
            storage
                .authenticate_session_detailed("not-a-real-token")
                .expect("auth"),
            WebSessionAuthResult::Missing
        );
    }

    #[test]
    fn repeated_invite_logins_keep_existing_sessions() {
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
                .is_some()
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
            2
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
        assert_eq!(listed[0].password_hash, None);
        assert_eq!(listed[0].password_set_at, None);
        assert_eq!(listed[0].tos_accepted_at, None);
        assert_eq!(listed[0].tos_version, None);
    }

    #[test]
    fn set_password_roundtrip_and_find_by_phone() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        assert_eq!(created.password_hash, None);
        assert!(
            storage
                .find_by_phone_password_ready("13800138000")
                .expect("find")
                .is_none(),
            "未设密码的账号不应被 password-ready 查询命中"
        );

        let ok = storage
            .set_password(&created.user_id, "argon2-hash-v1", "1.0")
            .expect("set password");
        assert!(ok);

        let found = storage
            .find_by_phone_password_ready("13800138000")
            .expect("find")
            .expect("user exists");
        assert_eq!(found.user_id, created.user_id);
        assert_eq!(found.password_hash.as_deref(), Some("argon2-hash-v1"));
        assert_eq!(found.tos_version.as_deref(), Some("1.0"));
        assert!(found.password_set_at.is_some());
        assert!(found.tos_accepted_at.is_some());

        // set_password 对已有密码的用户应为幂等禁止(返回 false)。
        let second = storage
            .set_password(&created.user_id, "another-hash", "1.0")
            .expect("second set");
        assert!(!second, "已有密码的账号不能再 set_password");
        let still = storage
            .find_by_phone_password_ready("13800138000")
            .expect("find")
            .expect("user");
        assert_eq!(still.password_hash.as_deref(), Some("argon2-hash-v1"));

        // change_password 更新但保留 tos。
        let changed = storage
            .change_password(&created.user_id, "argon2-hash-v2")
            .expect("change password");
        assert!(changed);
        let after = storage
            .find_by_phone_password_ready("13800138000")
            .expect("find")
            .expect("user");
        assert_eq!(after.password_hash.as_deref(), Some("argon2-hash-v2"));
        assert_eq!(after.tos_version.as_deref(), Some("1.0"));
    }

    #[test]
    fn create_session_for_user_respects_ttl_parameter() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");

        let short = storage
            .create_session_for_user(&created.user_id, SESSION_TTL_DAYS_SHORT)
            .expect("short")
            .expect("session");
        let span = (chrono::DateTime::parse_from_rfc3339(&short.expires_at).unwrap()
            - chrono::DateTime::parse_from_rfc3339(&short.created_at).unwrap())
        .num_hours();
        assert!((23..=25).contains(&span), "short TTL ≈ 24h, got {span}h");

        let long = storage
            .create_session_for_user(&created.user_id, SESSION_TTL_DAYS_LONG)
            .expect("long")
            .expect("session");
        let span_days = (chrono::DateTime::parse_from_rfc3339(&long.expires_at).unwrap()
            - chrono::DateTime::parse_from_rfc3339(&long.created_at).unwrap())
        .num_days();
        assert_eq!(span_days, SESSION_TTL_DAYS_LONG);

        assert!(
            storage
                .authenticate_session(&short.session_token)
                .expect("auth")
                .is_some()
        );
        assert!(
            storage
                .authenticate_session(&long.session_token)
                .expect("auth")
                .is_some()
        );
        assert_eq!(
            storage
                .count_active_sessions_for_user(&created.user_id)
                .expect("count"),
            2
        );
    }

    #[test]
    fn create_session_for_user_rejects_revoked() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        storage
            .set_invite_revoked(&created.user_id, true)
            .expect("revoke")
            .expect("invite");

        let attempt = storage
            .create_session_for_user(&created.user_id, SESSION_TTL_DAYS_LONG)
            .expect("attempt");
        assert!(attempt.is_none(), "revoked 用户不能创建 session");
    }

    #[test]
    fn whop_membership_creates_an_international_email_user_idempotently() {
        let storage = test_storage();
        let (created, outcome) = storage
            .upsert_whop_membership(whop_event("msg_test001", "2026-07-26T00:00:00Z", "active"))
            .expect("activate");
        assert_eq!(outcome, WhopMembershipUpsertOutcome::Created);
        assert!(created.phone_number.is_empty());
        assert!(
            storage
                .user_has_paid_access(&created.user_id)
                .expect("access")
        );

        let profile = storage.external_profile(&created.user_id).expect("profile");
        assert_eq!(profile.email_address.as_deref(), Some("buyer@example.com"));
        assert_eq!(
            profile.registration_policy,
            REGISTRATION_POLICY_WHOP_INTERNATIONAL
        );
        assert!(profile.email_verified_at.is_none());
        assert_eq!(
            profile
                .whop_membership
                .as_ref()
                .map(|membership| membership.status.as_str()),
            Some("active")
        );

        let (same, duplicate) = storage
            .upsert_whop_membership(whop_event("msg_test001", "2026-07-26T00:00:00Z", "active"))
            .expect("duplicate");
        assert_eq!(same.user_id, created.user_id);
        assert_eq!(duplicate, WhopMembershipUpsertOutcome::Duplicate);
        assert_eq!(storage.list_invite_users().expect("users").len(), 1);
    }

    #[test]
    fn email_challenge_verifies_the_webhook_created_user() {
        let storage = test_storage();
        let (created, _) = storage
            .upsert_whop_membership(whop_event("msg_test002", "2026-07-26T00:00:00Z", "active"))
            .expect("activate");
        let code = storage
            .begin_email_verification("BUYER@example.com", 10)
            .expect("challenge")
            .expect("known email");
        assert_eq!(code.len(), 8);
        assert_eq!(
            storage
                .verify_email_code("buyer@example.com", "00000000")
                .expect("invalid"),
            EmailVerificationResult::Invalid
        );
        assert_eq!(
            storage
                .verify_email_code("buyer@example.com", &code)
                .expect("verify"),
            EmailVerificationResult::Verified {
                user_id: created.user_id.clone()
            }
        );
        assert!(
            storage
                .external_profile(&created.user_id)
                .expect("profile")
                .email_verified_at
                .is_some()
        );
        assert!(
            storage
                .create_session_for_user(&created.user_id, SESSION_TTL_DAYS_LONG)
                .expect("session")
                .is_some()
        );
    }

    #[test]
    fn newer_deactivation_revokes_paid_access_and_older_events_stay_stale() {
        let storage = test_storage();
        let (created, _) = storage
            .upsert_whop_membership(whop_event("msg_test003", "2026-07-26T00:00:00Z", "active"))
            .expect("activate");
        let (_, updated) = storage
            .upsert_whop_membership(whop_event("msg_test004", "2026-07-27T00:00:00Z", "expired"))
            .expect("deactivate");
        assert_eq!(updated, WhopMembershipUpsertOutcome::Updated);
        assert!(
            !storage
                .user_has_paid_access(&created.user_id)
                .expect("access")
        );

        let (_, stale) = storage
            .upsert_whop_membership(whop_event("msg_test005", "2026-07-26T12:00:00Z", "active"))
            .expect("stale");
        assert_eq!(stale, WhopMembershipUpsertOutcome::Stale);
        assert!(
            !storage
                .user_has_paid_access(&created.user_id)
                .expect("access")
        );
    }

    #[test]
    fn newer_repurchase_replaces_membership_and_old_deactivation_cannot_revoke_it() {
        let storage = test_storage();
        let mut first = whop_event("msg_test006", "2026-07-26T00:00:00Z", "expired");
        first.membership_id = "mem_first".to_string();
        let (created, _) = storage
            .upsert_whop_membership(first)
            .expect("first membership");

        let mut repurchase = whop_event("msg_test007", "2026-07-27T00:00:00Z", "active");
        repurchase.membership_id = "mem_second".to_string();
        let (_, replaced) = storage
            .upsert_whop_membership(repurchase)
            .expect("repurchase");
        assert_eq!(replaced, WhopMembershipUpsertOutcome::Updated);
        assert!(
            storage
                .user_has_paid_access(&created.user_id)
                .expect("access")
        );

        let mut old_deactivation = whop_event("msg_test008", "2026-07-28T00:00:00Z", "canceled");
        old_deactivation.membership_id = "mem_first".to_string();
        let (_, stale) = storage
            .upsert_whop_membership(old_deactivation)
            .expect("late old event");
        assert_eq!(stale, WhopMembershipUpsertOutcome::Stale);
        assert!(
            storage
                .user_has_paid_access(&created.user_id)
                .expect("access")
        );
        assert_eq!(
            storage
                .external_profile(&created.user_id)
                .expect("profile")
                .whop_membership
                .expect("membership")
                .membership_id,
            "mem_second"
        );
    }
}
