use hone_core::cloud_runtime::{
    CloudPgRuntime, CloudWebAdminCreateOutcome, CloudWebAdminDisableOutcome,
    CloudWebUserExternalStateRecord,
};
use hone_core::cloud_sync::{ensure_cloud_schema_once, run_cloud_sync};
use hone_core::{HoneError, HoneResult, beijing_now, beijing_now_rfc3339};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SESSION_TTL_DAYS_LONG: i64 = 30;
pub const SESSION_TTL_DAYS_SHORT: i64 = 1;
pub const WEB_IDENTITY_DOMESTIC_INVITE: &str = "domestic_invite";
pub const WEB_IDENTITY_INTERNATIONAL_EMAIL: &str = "international_email";
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAdminInviteSummary {
    pub user_id: String,
    pub phone_number: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
    pub revoked_at: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebUserExternalProfile {
    pub email_address: Option<String>,
    pub email_verified_at: Option<String>,
    #[serde(
        default = "default_identity_kind",
        deserialize_with = "deserialize_identity_kind"
    )]
    pub identity_kind: String,
}

impl Default for WebUserExternalProfile {
    fn default() -> Self {
        Self {
            email_address: None,
            email_verified_at: None,
            identity_kind: WEB_IDENTITY_DOMESTIC_INVITE.to_string(),
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
    postgres: CloudPgRuntime,
    _test_postgres_lease: Option<std::sync::Arc<crate::test_postgres::TestPostgresLease>>,
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
    /// PostgreSQL-backed test constructor. The path is only an isolation namespace.
    #[doc(hidden)]
    pub fn new(path: impl AsRef<std::path::Path>) -> HoneResult<Self> {
        let (postgres, lease) = crate::test_postgres::isolated_postgres(path)?;
        let mut storage = Self::new_cloud(postgres)?;
        storage._test_postgres_lease = Some(lease);
        Ok(storage)
    }

    pub fn new_cloud(postgres: CloudPgRuntime) -> HoneResult<Self> {
        ensure_cloud_schema_once(postgres.clone(), None)?;
        Ok(Self {
            postgres,
            _test_postgres_lease: None,
        })
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
        let postgres = self.postgres.clone();
        let record = CloudWebInviteRecord {
            user: user.clone(),
            api_key_hash,
            external_state: WebUserExternalState::default(),
        };
        let value = serde_json::to_value(&record)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let user_id = user.user_id.clone();
        let phone_number = user.phone_number.clone();
        let external_state = cloud_external_state_record(&user_id, &external_state)?;
        run_cloud_web_auth(async move {
            postgres
                .upsert_web_invite_user_record_with_external_state(
                    &user_id,
                    &phone_number,
                    value,
                    &external_state,
                )
                .await
        })
    }

    fn cloud_record_to_user(
        value: serde_json::Value,
    ) -> HoneResult<(WebInviteUser, Option<String>)> {
        let record = cloud_record_from_value(value)?;
        Ok((record.user, record.api_key_hash))
    }

    fn cloud_find_record_by(
        &self,
        field: &str,
        value: &str,
    ) -> HoneResult<Option<CloudWebInviteRecord>> {
        let postgres = self.postgres.clone();
        let field = field.to_string();
        let value = value.to_string();
        run_cloud_web_auth(
            async move { postgres.find_web_invite_user_record(&field, &value).await },
        )?
        .map(cloud_record_from_value)
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

    fn load_external_state(&self, user_id: &str) -> HoneResult<WebUserExternalState> {
        let postgres = self.postgres.clone();
        let user_id = user_id.to_string();
        return run_cloud_web_auth(async move {
            postgres.find_web_user_external_state_record(&user_id).await
        })?
        .map(cloud_external_state_from_record)
        .transpose()
        .map(|state| state.unwrap_or_default());
    }

    fn save_external_state(
        &self,
        user: &WebInviteUser,
        state: WebUserExternalState,
    ) -> HoneResult<()> {
        let postgres = self.postgres.clone();
        let external_state = cloud_external_state_record(&user.user_id, &state)?;
        return run_cloud_web_auth(async move {
            postgres
                .upsert_web_user_external_state_record(&external_state)
                .await
        });
    }

    fn find_external_user_by_email(
        &self,
        email_address: &str,
    ) -> HoneResult<Option<(WebInviteUser, WebUserExternalState)>> {
        let email = normalize_email_address(email_address)?;

        let postgres = self.postgres.clone();
        return run_cloud_web_auth(async move {
            postgres.find_web_invite_user_record_by_email(&email).await
        })?
        .map(|(record, external_state)| {
            Ok((
                cloud_record_from_value(record)?.user,
                cloud_external_state_from_record(external_state)?,
            ))
        })
        .transpose();
    }

    fn cloud_upsert_session(&self, session: &CloudWebAuthSessionRecord) -> HoneResult<()> {
        let postgres = self.postgres.clone();
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
        let postgres = self.postgres.clone();
        let now = now.to_string();
        run_cloud_web_auth(async move {
            postgres.purge_expired_web_auth_sessions(&now).await?;
            Ok(())
        })
    }

    pub fn is_web_admin(&self, user_id: &str) -> HoneResult<bool> {
        let postgres = self.postgres.clone();
        let user_id = user_id.to_string();
        return run_cloud_web_auth(
            async move { postgres.web_invite_user_is_admin(&user_id).await },
        );
    }

    pub fn set_web_admin_by_phone(
        &self,
        phone_number: &str,
        is_admin: bool,
    ) -> HoneResult<Option<String>> {
        let phone_number = validate_phone_number(phone_number)?;

        let postgres = self.postgres.clone();
        return run_cloud_web_auth(async move {
            postgres
                .set_web_invite_user_admin_by_phone(&phone_number, is_admin)
                .await
        });
    }

    pub fn web_admin_create_count_today(&self, admin_user_id: &str) -> HoneResult<u32> {
        let beijing_date = beijing_now().format("%F").to_string();

        let postgres = self.postgres.clone();
        let admin_user_id = admin_user_id.to_string();
        return run_cloud_web_auth(async move {
            postgres
                .web_admin_create_count_for_date(&admin_user_id, &beijing_date)
                .await
        });
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

        let postgres = self.postgres.clone();
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

    pub fn disable_invite_user_by_admin(
        &self,
        admin_user_id: &str,
        target_user_id: &str,
    ) -> HoneResult<WebAdminInviteDisableOutcome> {
        let now = beijing_now();
        let beijing_date = now.format("%F").to_string();
        let now = now.to_rfc3339();

        let postgres = self.postgres.clone();
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

    pub fn create_invite_user(&self, phone_number: &str) -> HoneResult<WebInviteUser> {
        let created_at = beijing_now_rfc3339();
        let user_id = generate_user_id();
        let phone_number = validate_phone_number(phone_number)?;

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

    pub fn external_profile(&self, user_id: &str) -> HoneResult<WebUserExternalProfile> {
        Ok(self.load_external_state(user_id)?.profile)
    }

    pub fn find_user_by_email(&self, email_address: &str) -> HoneResult<Option<WebInviteUser>> {
        Ok(self
            .find_external_user_by_email(email_address)?
            .map(|(user, _)| user))
    }

    pub fn ensure_international_email_user(
        &self,
        email_address: &str,
    ) -> HoneResult<WebInviteUser> {
        let email_address = normalize_email_address(email_address)?;
        if let Some((user, mut state)) = self.find_external_user_by_email(&email_address)? {
            if state.profile.email_verified_at.is_some()
                && state.profile.email_address.as_deref() != Some(email_address.as_str())
            {
                return Err(HoneError::Storage(
                    "已验证邮箱不允许被外部付款事件覆盖".to_string(),
                ));
            }
            state.profile.email_address = Some(email_address);
            state.profile.identity_kind = WEB_IDENTITY_INTERNATIONAL_EMAIL.to_string();
            self.save_external_state(&user, state)?;
            return Ok(user);
        }
        self.create_international_email_user(email_address)
    }

    pub fn begin_email_verification(
        &self,
        email_address: &str,
        ttl_minutes: i64,
    ) -> HoneResult<Option<String>> {
        let Some((user, mut state)) = self.find_external_user_by_email(email_address)? else {
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
        self.save_external_state(&user, state)?;
        Ok(Some(code))
    }

    pub fn verify_email_code(
        &self,
        email_address: &str,
        code: &str,
    ) -> HoneResult<EmailVerificationResult> {
        let Some((user, mut state)) = self.find_external_user_by_email(email_address)? else {
            return Ok(EmailVerificationResult::Missing);
        };
        let Some(mut challenge) = state.email_challenge.take() else {
            return Ok(EmailVerificationResult::Missing);
        };
        if challenge.attempts >= 5 {
            state.email_challenge = Some(challenge);
            self.save_external_state(&user, state)?;
            return Ok(EmailVerificationResult::AttemptsExceeded);
        }
        if challenge.expires_at <= beijing_now_rfc3339() {
            self.save_external_state(&user, state)?;
            return Ok(EmailVerificationResult::Expired);
        }
        let normalized_code = normalize_email_verification_code(code);
        if hash_email_verification_code(&user.user_id, &normalized_code) != challenge.code_hash {
            challenge.attempts = challenge.attempts.saturating_add(1);
            let exhausted = challenge.attempts >= 5;
            state.email_challenge = Some(challenge);
            self.save_external_state(&user, state)?;
            return Ok(if exhausted {
                EmailVerificationResult::AttemptsExceeded
            } else {
                EmailVerificationResult::Invalid
            });
        }
        state.profile.email_verified_at = Some(beijing_now_rfc3339());
        self.save_external_state(&user, state)?;
        Ok(EmailVerificationResult::Verified {
            user_id: user.user_id,
        })
    }

    fn create_international_email_user(&self, email_address: String) -> HoneResult<WebInviteUser> {
        let created_at = beijing_now_rfc3339();
        let user_id = generate_user_id();
        let external_state = WebUserExternalState {
            profile: WebUserExternalProfile {
                email_address: Some(email_address),
                email_verified_at: None,
                identity_kind: WEB_IDENTITY_INTERNATIONAL_EMAIL.to_string(),
            },
            email_challenge: None,
        };

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

    pub fn list_invite_users(&self) -> HoneResult<Vec<WebInviteUser>> {
        let postgres = self.postgres.clone();
        let records =
            run_cloud_web_auth(async move { postgres.list_web_invite_user_records().await })?;
        return records
            .into_iter()
            .map(Self::cloud_record_to_user)
            .map(|result| result.map(|(user, _)| user))
            .collect();
    }

    pub fn list_web_admin_invite_summaries(&self) -> HoneResult<Vec<WebAdminInviteSummary>> {
        let postgres = self.postgres.clone();
        let records =
            run_cloud_web_auth(async move { postgres.list_web_admin_invite_summaries().await })?;
        return Ok(records
            .into_iter()
            .map(|record| WebAdminInviteSummary {
                user_id: record.user_id,
                phone_number: record.phone_number,
                created_at: record.created_at,
                last_login_at: record.last_login_at,
                revoked_at: record.revoked_at,
                is_admin: record.is_admin,
            })
            .collect());
    }

    pub fn find_invite_user_by_code(&self, invite_code: &str) -> HoneResult<Option<WebInviteUser>> {
        let invite_code = normalize_invite_code(invite_code);

        return Ok(self
            .cloud_find_invite_by("invite_code", &invite_code)?
            .map(|(user, _)| user));
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

        let user = self
            .cloud_find_invite_by("phone_number", &phone)?
            .map(|(user, _)| user)
            .filter(|user| user.revoked_at.is_none());
        return Ok(user);
    }

    pub fn find_invite_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        return Ok(self
            .cloud_find_invite_by("user_id", user_id)?
            .map(|(user, _)| user));
    }

    pub fn find_invite_user_by_api_key(&self, api_key: &str) -> HoneResult<Option<WebInviteUser>> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Ok(None);
        }
        let now = beijing_now_rfc3339();
        let api_key_hash = hash_api_key(api_key);

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

    pub fn ensure_api_key_for_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        let now = beijing_now_rfc3339();

        let Some((mut existing, _existing_hash)) = self.cloud_find_invite_by("user_id", user_id)?
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

    pub fn reset_api_key_for_user(&self, user_id: &str) -> HoneResult<Option<WebInviteUser>> {
        let now = beijing_now_rfc3339();

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

        let postgres = self.postgres.clone();
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

    pub fn delete_session(&self, session_token: &str) -> HoneResult<()> {
        let token_hash = hash_session_token(session_token);

        let postgres = self.postgres.clone();
        let session_token = session_token.to_string();
        return run_cloud_web_auth(async move {
            postgres
                .delete_web_auth_session(&token_hash, &session_token)
                .await
        });
    }

    pub fn count_active_sessions_for_user(&self, user_id: &str) -> HoneResult<u32> {
        let now = beijing_now_rfc3339();

        let postgres = self.postgres.clone();
        self.cloud_purge_expired_sessions(&now)?;
        let user_id = user_id.to_string();
        return run_cloud_web_auth(async move {
            postgres
                .count_active_web_auth_sessions(&user_id, &now)
                .await
        });
    }

    pub fn set_invite_revoked(
        &self,
        user_id: &str,
        revoked: bool,
    ) -> HoneResult<Option<WebInviteMutation>> {
        let now = beijing_now_rfc3339();

        self.cloud_purge_expired_sessions(&now)?;
        let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)? else {
            return Ok(None);
        };
        let cleared_session_count = if revoked {
            let postgres = self.postgres.clone();
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

    pub fn reset_invite_code(&self, user_id: &str) -> HoneResult<Option<WebInviteMutation>> {
        let now = beijing_now_rfc3339();

        self.cloud_purge_expired_sessions(&now)?;
        let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)? else {
            return Ok(None);
        };
        let invite_code = generate_unique_invite_code_cloud(self)?;
        let postgres = self.postgres.clone();
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

    /// 查询已设置密码、未吊销的用户,用于手机号+密码登录校验。
    pub fn find_by_phone_password_ready(
        &self,
        phone_number: &str,
    ) -> HoneResult<Option<WebInviteUser>> {
        let phone = normalize_phone_number(phone_number);
        if phone.is_empty() {
            return Ok(None);
        }

        return Ok(self
            .cloud_find_invite_by("phone_number", &phone)?
            .map(|(user, _)| user)
            .filter(|user| user.revoked_at.is_none() && user.password_hash.is_some()));
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

        let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)? else {
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

    /// 已设置密码后用于修改密码(/me 页)。不动 tos_accepted_at / tos_version。
    pub fn change_password(&self, user_id: &str, password_hash: &str) -> HoneResult<bool> {
        let now = beijing_now_rfc3339();

        let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)? else {
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

    pub fn record_tos_acceptance(&self, user_id: &str, tos_version: &str) -> HoneResult<bool> {
        let now = beijing_now_rfc3339();

        let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)? else {
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

        self.cloud_purge_expired_sessions(&created_at)?;
        let Some((mut user, api_key_hash)) = self.cloud_find_invite_by("user_id", user_id)? else {
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
}

fn run_cloud_web_auth<T, F>(future: F) -> HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = HoneResult<T>> + Send + 'static,
{
    run_cloud_sync(future, None, "cloud web auth operation")
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

fn external_state_is_default(state: &WebUserExternalState) -> bool {
    state == &WebUserExternalState::default()
}

fn cloud_external_state_record(
    user_id: &str,
    state: &WebUserExternalState,
) -> HoneResult<CloudWebUserExternalStateRecord> {
    let email_challenge_json = state
        .email_challenge
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|err| HoneError::Serialization(err.to_string()))?;
    Ok(CloudWebUserExternalStateRecord {
        user_id: user_id.to_string(),
        email_address: state.profile.email_address.clone(),
        email_verified_at: state.profile.email_verified_at.clone(),
        identity_kind: Some(state.profile.identity_kind.clone()),
        email_challenge_json,
    })
}

fn cloud_external_state_from_record(
    record: CloudWebUserExternalStateRecord,
) -> HoneResult<WebUserExternalState> {
    let identity_kind = record
        .identity_kind
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            canonical_identity_kind(&value)
                .map(str::to_string)
                .ok_or_else(|| HoneError::Serialization(format!("未知 Web 身份类型: {value}")))
        })
        .transpose()?
        .unwrap_or_else(default_identity_kind);
    let email_challenge = record
        .email_challenge_json
        .map(|value| serde_json::from_str(&value))
        .transpose()
        .map_err(|err| HoneError::Serialization(err.to_string()))?;
    Ok(WebUserExternalState {
        profile: WebUserExternalProfile {
            email_address: record.email_address,
            email_verified_at: record.email_verified_at,
            identity_kind,
        },
        email_challenge,
    })
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

fn default_identity_kind() -> String {
    WEB_IDENTITY_DOMESTIC_INVITE.to_string()
}

fn canonical_identity_kind(value: &str) -> Option<&'static str> {
    match value.trim() {
        WEB_IDENTITY_DOMESTIC_INVITE => Some(WEB_IDENTITY_DOMESTIC_INVITE),
        WEB_IDENTITY_INTERNATIONAL_EMAIL => Some(WEB_IDENTITY_INTERNATIONAL_EMAIL),
        _ => None,
    }
}

fn deserialize_identity_kind<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    canonical_identity_kind(&value)
        .map(str::to_string)
        .ok_or_else(|| serde::de::Error::custom(format!("未知 Web 身份类型: {value}")))
}

fn cloud_record_from_value(value: serde_json::Value) -> HoneResult<CloudWebInviteRecord> {
    if let Some(external_state) = value
        .get("external_state")
        .and_then(serde_json::Value::as_object)
    {
        if !external_state.contains_key("identity_kind") {
            return Err(HoneError::Serialization(
                "Web 外部状态缺少 identity_kind".to_string(),
            ));
        }
    }
    serde_json::from_value(value).map_err(|err| HoneError::Serialization(err.to_string()))
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

/// Add a column to an existing table if it does not already exist.
///
/// # Safety (SQL injection)
///
/// `table`, `column`, and `definition` are interpolated directly into DDL.
/// **All arguments MUST be hard-coded string literals** — never pass values
/// derived from user input or external configuration.

#[cfg(test)]
mod tests {
    use super::{
        CloudWebInviteRecord, EmailVerificationResult, SESSION_TTL_DAYS_LONG,
        SESSION_TTL_DAYS_SHORT, WEB_IDENTITY_INTERNATIONAL_EMAIL, WebAdminInviteCreateOutcome,
        WebAdminInviteDisableOutcome, WebAuthStorage, WebSessionAuthResult, generate_api_key,
        generate_invite_code, generate_session_token, hash_session_token, run_cloud_web_auth,
    };
    use hone_core::cloud_runtime::CloudPgRuntime;
    use hone_core::config::{CloudConfig, PostgresConfig};
    use hone_core::{HoneError, HoneResult, beijing_now};
    use tokio_postgres::NoTls;

    fn test_storage() -> WebAuthStorage {
        let namespace =
            std::env::temp_dir().join(format!("hone_web_auth_{}", uuid::Uuid::new_v4()));
        WebAuthStorage::new(namespace).expect("storage")
    }

    struct CloudWebAuthTestUser {
        database_url: String,
        user_id: String,
    }

    impl Drop for CloudWebAuthTestUser {
        fn drop(&mut self) {
            let database_url = self.database_url.clone();
            let user_id = self.user_id.clone();
            let _ = run_cloud_web_auth(async move {
                let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
                    .await
                    .map_err(|err| {
                        HoneError::Config(format!("Postgres test cleanup 连接失败: {err}"))
                    })?;
                tokio::spawn(async move {
                    let _ = connection.await;
                });
                client
                    .execute(
                        "DELETE FROM cloud_web_invite_users WHERE user_id = $1",
                        &[&user_id],
                    )
                    .await
                    .map_err(|err| {
                        HoneError::Config(format!("Postgres test cleanup 删除失败: {err}"))
                    })?;
                Ok(())
            });
        }
    }

    fn inspect_cloud_external_state(
        database_url: String,
        user_id: String,
    ) -> HoneResult<(Option<String>, Option<String>, Option<String>, bool)> {
        run_cloud_web_auth(async move {
            let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
                .await
                .map_err(|err| {
                    HoneError::Config(format!("Postgres test inspect 连接失败: {err}"))
                })?;
            tokio::spawn(async move {
                let _ = connection.await;
            });
            let row = client
                .query_one(
                    r#"
SELECT s.email_address, s.email_verified_at, s.email_challenge_json,
       u.record ? 'external_state'
FROM cloud_web_user_external_state s
JOIN cloud_web_invite_users u ON u.user_id = s.user_id
WHERE s.user_id = $1
"#,
                    &[&user_id],
                )
                .await
                .map_err(|err| {
                    HoneError::Config(format!("Postgres test inspect 查询失败: {err}"))
                })?;
            Ok((row.get(0), row.get(1), row.get(2), row.get(3)))
        })
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn web_admin_summary_list_uses_minimal_fields_and_excludes_non_phone_accounts() {
        let storage = test_storage();
        let admin = storage.create_invite_user("13871396421").expect("admin");
        let member = storage.create_invite_user("13900000000").expect("member");
        let international = storage
            .create_invite_user("13700000000")
            .expect("international");
        storage
            .set_web_admin_by_phone("13871396421", true)
            .expect("grant admin");
        storage
            .set_invite_revoked(&member.user_id, true)
            .expect("disable member");
        let postgres = storage.postgres.clone();
        let international_user_id = international.user_id.clone();
        run_cloud_web_auth(async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "UPDATE cloud_web_invite_users SET phone_number = '', record = jsonb_set(record, '{phone_number}', '\"\"'::jsonb) WHERE user_id = $1",
                    &[&international_user_id],
                )
                .await
                .map_err(|error| HoneError::Config(error.to_string()))?;
            Ok(())
        })
        .expect("clear international placeholder phone");

        let summaries = storage
            .list_web_admin_invite_summaries()
            .expect("list summaries");

        assert_eq!(summaries.len(), 2);
        let admin_summary = summaries
            .iter()
            .find(|summary| summary.user_id == admin.user_id)
            .expect("admin summary");
        assert!(admin_summary.is_admin);
        assert_eq!(admin_summary.phone_number, "13871396421");
        let member_summary = summaries
            .iter()
            .find(|summary| summary.user_id == member.user_id)
            .expect("member summary");
        assert!(!member_summary.is_admin);
        assert!(member_summary.revoked_at.is_some());
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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

        let postgres = storage.postgres.clone();
        let admin_user_id = admin.user_id.clone();
        let audit_count = run_cloud_web_auth(async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    "SELECT count(*) FROM cloud_web_admin_actions WHERE admin_user_id = $1 AND action = 'disable'",
                    &[&admin_user_id],
                )
                .await
                .map_err(|error| HoneError::Config(error.to_string()))?;
            Ok(row.get::<_, i64>(0))
        })
        .expect("audit");
        assert_eq!(audit_count, 1);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn existing_user_can_generate_api_key_once_without_plaintext_replay() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let postgres = storage.postgres.clone();
        let user_id = created.user_id.clone();
        run_cloud_web_auth(async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "UPDATE cloud_web_invite_users SET record = jsonb_set(jsonb_set(jsonb_set(jsonb_set(record, '{api_key_hash}', 'null'::jsonb), '{api_key_prefix}', 'null'::jsonb), '{api_key_created_at}', 'null'::jsonb), '{api_key_last_used_at}', 'null'::jsonb) WHERE user_id = $1",
                    &[&user_id],
                )
                .await
                .map_err(|error| HoneError::Config(error.to_string()))?;
            Ok(())
        })
        .expect("clear key");
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
        let postgres = storage.postgres.clone();
        let user_id = created.user_id.clone();
        let stored_token: String = run_cloud_web_auth(async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    "SELECT session_hash FROM cloud_web_auth_sessions WHERE user_id = $1",
                    &[&user_id],
                )
                .await
                .map_err(|error| HoneError::Config(error.to_string()))?;
            Ok(row.get(0))
        })
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn legacy_plaintext_session_tokens_remain_accepted_during_migration() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let now = beijing_now();
        let created_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::days(SESSION_TTL_DAYS_LONG)).to_rfc3339();
        let legacy_token = "legacy-plaintext-session-token";
        let record = super::CloudWebAuthSessionRecord {
            session_hash: legacy_token.to_string(),
            user_id: created.user_id.clone(),
            created_at: created_at.clone(),
            expires_at: expires_at.clone(),
            last_seen_at: created_at.clone(),
        };
        let postgres = storage.postgres.clone();
        let record_value = serde_json::to_value(&record).expect("record");
        run_cloud_web_auth(async move {
            postgres
                .upsert_web_auth_session_record(
                    legacy_token,
                    &record.user_id,
                    record_value,
                    Some(&record.expires_at),
                )
                .await
        })
        .expect("insert legacy session");

        let authed = storage
            .authenticate_session(legacy_token)
            .expect("auth")
            .expect("user");

        assert_eq!(authed.user_id, created.user_id);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn detailed_auth_reports_expired_and_missing_sessions() {
        let storage = test_storage();
        let created = storage.create_invite_user("13800138000").expect("create");
        let now = beijing_now();
        let created_at = (now - chrono::Duration::days(2)).to_rfc3339();
        let expires_at = (now - chrono::Duration::days(1)).to_rfc3339();
        let raw_token = "expired-session-token";
        let token_hash = hash_session_token(raw_token);
        let record = super::CloudWebAuthSessionRecord {
            session_hash: token_hash.clone(),
            user_id: created.user_id.clone(),
            created_at: created_at.clone(),
            expires_at: expires_at.clone(),
            last_seen_at: created_at.clone(),
        };
        let postgres = storage.postgres.clone();
        let record_value = serde_json::to_value(&record).expect("record");
        run_cloud_web_auth(async move {
            postgres
                .upsert_web_auth_session_record(
                    &token_hash,
                    &record.user_id,
                    record_value,
                    Some(&record.expires_at),
                )
                .await
        })
        .expect("insert expired session");

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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn invalid_phone_number_is_rejected_when_creating_invite() {
        let storage = test_storage();
        let error = storage
            .create_invite_user("abc")
            .expect_err("invalid phone");
        assert!(error.to_string().contains("手机号格式不合法"));
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_schema_keeps_invite_columns_and_defaults() {
        let root =
            std::env::temp_dir().join(format!("hone_web_auth_migrate_{}", uuid::Uuid::new_v4()));
        let storage = WebAuthStorage::new(&root).expect("postgres storage");
        let postgres = storage.postgres.clone();
        let columns = run_cloud_web_auth(async move {
            let client = postgres.connect_cached_client().await?;
            let rows = client
                .query(
                    "SELECT column_name FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = 'cloud_web_invite_users'",
                    &[],
                )
                .await
                .map_err(|error| HoneError::Config(error.to_string()))?;
            Ok(rows.into_iter().map(|row| row.get::<_, String>(0)).collect::<Vec<_>>())
        })
        .expect("columns");
        assert!(columns.iter().any(|column| column == "phone_number"));
        assert!(columns.iter().any(|column| column == "is_admin"));
        assert!(columns.iter().any(|column| column == "record"));
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn international_email_identity_is_provider_neutral_and_verifiable() {
        let storage = test_storage();
        let created = storage
            .ensure_international_email_user("Buyer@Example.com")
            .expect("create identity");
        let same = storage
            .ensure_international_email_user("buyer@example.com")
            .expect("idempotent identity");
        assert_eq!(same.user_id, created.user_id);
        assert!(created.phone_number.is_empty());
        let profile = storage.external_profile(&created.user_id).expect("profile");
        assert_eq!(profile.email_address.as_deref(), Some("buyer@example.com"));
        assert_eq!(profile.identity_kind, WEB_IDENTITY_INTERNATIONAL_EMAIL);

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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn cloud_web_user_external_state_round_trip() {
        let postgres_config = PostgresConfig::default();
        let database_url = postgres_config.resolved_database_url();
        assert!(
            !database_url.is_empty(),
            "HONE_POSTGRES_HOST/PORT/USER/PASSWORD/DATABASE must be configured"
        );
        let cloud_config = CloudConfig {
            postgres: postgres_config,
            ..CloudConfig::default()
        };
        let postgres =
            CloudPgRuntime::from_cloud_config(&cloud_config).expect("configured postgres runtime");
        let storage = WebAuthStorage::new_cloud(postgres.clone()).expect("cloud web auth storage");
        let email = format!("pg-web-auth-{}@example.com", uuid::Uuid::new_v4().simple());

        let created = storage
            .ensure_international_email_user(&email)
            .expect("create international user");
        let _cleanup = CloudWebAuthTestUser {
            database_url: database_url.clone(),
            user_id: created.user_id.clone(),
        };
        let same = storage
            .ensure_international_email_user(&email.to_ascii_uppercase())
            .expect("lookup international user by indexed email");
        assert_eq!(same.user_id, created.user_id);
        assert_eq!(
            storage
                .external_profile(&created.user_id)
                .expect("load external profile")
                .email_address
                .as_deref(),
            Some(email.as_str())
        );

        let code = storage
            .begin_email_verification(&email, 10)
            .expect("write email challenge")
            .expect("known email");
        assert_eq!(
            storage
                .verify_email_code(&email, "00000000")
                .expect("persist invalid attempt"),
            EmailVerificationResult::Invalid
        );
        assert_eq!(
            storage
                .verify_email_code(&email, &code)
                .expect("verify email"),
            EmailVerificationResult::Verified {
                user_id: created.user_id.clone()
            }
        );

        let (stored_email, verified_at, challenge_json, invite_record_has_external_state) =
            inspect_cloud_external_state(database_url, created.user_id.clone())
                .expect("inspect external state row");
        assert_eq!(stored_email.as_deref(), Some(email.as_str()));
        assert!(verified_at.is_some());
        assert_eq!(challenge_json, None);
        assert!(
            !invite_record_has_external_state,
            "new Cloud writes must keep external state in its dedicated table"
        );
    }
}
