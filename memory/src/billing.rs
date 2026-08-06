use std::cmp::Ordering;
use std::future::Future;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use chrono::DateTime;
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::{HoneError, HoneResult, beijing_now_rfc3339};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const BILLING_PROVIDER_STRIPE: &str = "stripe";
pub const BILLING_PROVIDER_DOMESTIC_INVITE: &str = "domestic_invite";

pub const BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION: &str = "recurring_subscription";
pub const BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE: &str = "fixed_term_purchase";
pub const BILLING_ENTITLEMENT_DOMESTIC_INVITE: &str = "domestic_invite";

pub const BILLING_ACCESS_PENDING: &str = "pending";
pub const BILLING_ACCESS_ACTIVE: &str = "active";
pub const BILLING_ACCESS_GRACE: &str = "grace";
pub const BILLING_ACCESS_INACTIVE: &str = "inactive";

pub const BILLING_EVENT_RECEIVED: &str = "received";
pub const BILLING_EVENT_PROCESSING: &str = "processing";
pub const BILLING_EVENT_PROCESSED: &str = "processed";
pub const BILLING_EVENT_FAILED: &str = "failed";

const BILLING_WEBHOOK_MAX_ATTEMPTS: u32 = 10;
const BILLING_WEBHOOK_LEASE_MINUTES: i64 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingEntitlement {
    pub entitlement_id: String,
    pub user_id: String,
    pub provider: String,
    pub entitlement_kind: String,
    pub provider_customer_id: Option<String>,
    pub provider_reference_id: String,
    pub provider_product_id: Option<String>,
    pub provider_price_id: Option<String>,
    pub purchase_email_normalized: Option<String>,
    pub raw_status: String,
    pub access_state: String,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub manage_url: Option<String>,
    pub grace_expires_at: Option<String>,
    pub last_event_id: String,
    pub last_event_created_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl BillingEntitlement {
    pub fn grants_paid_access(&self) -> bool {
        match self.access_state.as_str() {
            BILLING_ACCESS_ACTIVE => {
                self.entitlement_kind != BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
                    || self
                        .current_period_end
                        .as_deref()
                        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                        .is_some_and(|deadline| deadline >= chrono::Utc::now())
            }
            BILLING_ACCESS_GRACE => self
                .grace_expires_at
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .is_some_and(|deadline| deadline >= chrono::Utc::now()),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingEntitlementUpsertOutcome {
    Created,
    Updated,
    Duplicate,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingWebhookEvent {
    pub provider: String,
    pub event_id: String,
    pub event_type: String,
    pub object_id: Option<String>,
    pub payload_sha256: String,
    pub provider_created_at: String,
    pub processing_state: String,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub received_at: String,
    #[serde(default)]
    pub processing_started_at: Option<String>,
    pub processed_at: Option<String>,
    pub normalized_payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingWebhookRecordOutcome {
    Inserted,
    Duplicate,
}

pub struct BillingStorage {
    backend: BillingBackend,
}

enum BillingBackend {
    Sqlite { conn: Mutex<Connection> },
    Cloud { postgres: CloudPgRuntime },
}

impl BillingStorage {
    pub fn new(path: impl AsRef<Path>) -> HoneResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| HoneError::Config(format!("创建 Billing 目录失败: {err}")))?;
        }
        let conn = Connection::open(path)
            .map_err(|err| HoneError::Config(format!("打开 Billing SQLite 失败: {err}")))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(sql_err)?;
        conn.pragma_update(None, "busy_timeout", 5000)
            .map_err(sql_err)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(sql_err)?;
        let storage = Self {
            backend: BillingBackend::Sqlite {
                conn: Mutex::new(conn),
            },
        };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn new_cloud(postgres: CloudPgRuntime) -> HoneResult<Self> {
        let schema_postgres = postgres.clone();
        run_cloud_billing(async move { schema_postgres.ensure_schema().await })?;
        Ok(Self {
            backend: BillingBackend::Cloud { postgres },
        })
    }

    fn init_schema(&self) -> HoneResult<()> {
        let BillingBackend::Sqlite { conn } = &self.backend else {
            return Ok(());
        };
        let conn = conn.lock().map_err(lock_err)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS billing_entitlements (
                entitlement_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL CHECK (provider IN ('stripe', 'domestic_invite')),
                entitlement_kind TEXT NOT NULL CHECK (entitlement_kind IN ('recurring_subscription', 'fixed_term_purchase', 'domestic_invite')),
                provider_customer_id TEXT,
                provider_reference_id TEXT NOT NULL,
                provider_product_id TEXT,
                provider_price_id TEXT,
                purchase_email_normalized TEXT,
                raw_status TEXT NOT NULL,
                access_state TEXT NOT NULL CHECK (access_state IN ('pending', 'active', 'grace', 'inactive')),
                current_period_start TEXT,
                current_period_end TEXT,
                cancel_at_period_end INTEGER NOT NULL DEFAULT 0 CHECK (cancel_at_period_end IN (0, 1)),
                manage_url TEXT,
                grace_expires_at TEXT,
                last_event_id TEXT NOT NULL,
                last_event_created_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(provider, provider_reference_id),
                FOREIGN KEY(user_id) REFERENCES web_invite_users(user_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_billing_entitlements_user_access
                ON billing_entitlements(user_id, access_state);
            CREATE INDEX IF NOT EXISTS idx_billing_entitlements_purchase_email
                ON billing_entitlements(purchase_email_normalized)
                WHERE purchase_email_normalized IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_billing_entitlements_customer
                ON billing_entitlements(provider, provider_customer_id)
                WHERE provider_customer_id IS NOT NULL;

            CREATE TABLE IF NOT EXISTS billing_webhook_events (
                provider TEXT NOT NULL CHECK (provider = 'stripe'),
                event_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                object_id TEXT,
                payload_sha256 TEXT NOT NULL,
                provider_created_at TEXT NOT NULL,
                processing_state TEXT NOT NULL CHECK (processing_state IN ('received', 'processing', 'processed', 'failed')),
                attempt_count INTEGER NOT NULL DEFAULT 0,
                last_error TEXT,
                received_at TEXT NOT NULL,
                processing_started_at TEXT,
                processed_at TEXT,
                normalized_payload TEXT NOT NULL DEFAULT '{}',
                PRIMARY KEY(provider, event_id)
            );

            CREATE INDEX IF NOT EXISTS idx_billing_webhook_events_processing
                ON billing_webhook_events(processing_state, received_at);
            ",
        )
        .map_err(sql_err)?;
        ensure_billing_column(&conn, "billing_entitlements", "manage_url", "TEXT")?;
        ensure_billing_column(&conn, "billing_entitlements", "grace_expires_at", "TEXT")?;
        ensure_billing_column(
            &conn,
            "billing_webhook_events",
            "normalized_payload",
            "TEXT NOT NULL DEFAULT '{}'",
        )?;
        ensure_billing_column(
            &conn,
            "billing_webhook_events",
            "processing_started_at",
            "TEXT",
        )?;
        migrate_sqlite_billing_to_typed_entitlements(&conn)?;
        conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_billing_webhook_events_retry
                ON billing_webhook_events(
                    provider, processing_state, processing_started_at, received_at
                )
            ",
            [],
        )
        .map_err(sql_err)?;
        Ok(())
    }

    fn sqlite_conn(&self) -> HoneResult<MutexGuard<'_, Connection>> {
        match &self.backend {
            BillingBackend::Sqlite { conn } => conn.lock().map_err(lock_err),
            BillingBackend::Cloud { .. } => Err(HoneError::Storage(
                "billing sqlite connection requested in cloud mode".to_string(),
            )),
        }
    }

    fn cloud_postgres(&self) -> Option<CloudPgRuntime> {
        match &self.backend {
            BillingBackend::Cloud { postgres } => Some(postgres.clone()),
            BillingBackend::Sqlite { .. } => None,
        }
    }

    pub fn entitlement_id(provider: &str, provider_reference_id: &str) -> String {
        let digest = Sha256::digest(format!("{provider}:{provider_reference_id}").as_bytes());
        let suffix = digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!("ent_{suffix}")
    }

    pub fn list_user_entitlements(&self, user_id: &str) -> HoneResult<Vec<BillingEntitlement>> {
        if let Some(postgres) = self.cloud_postgres() {
            let user_id = user_id.to_string();
            return run_cloud_billing(async move {
                postgres.list_billing_entitlement_records(&user_id).await
            })?
            .into_iter()
            .map(entitlement_from_value)
            .collect();
        }
        let conn = self.sqlite_conn()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT entitlement_id, user_id, provider, entitlement_kind,
                       provider_customer_id, provider_reference_id,
                       provider_product_id, provider_price_id,
                       purchase_email_normalized, raw_status, access_state,
                       current_period_start, current_period_end, cancel_at_period_end,
                       manage_url, grace_expires_at, last_event_id, last_event_created_at,
                       created_at, updated_at
                FROM billing_entitlements
                WHERE user_id = ?1
                ORDER BY updated_at DESC, entitlement_id DESC
                ",
            )
            .map_err(sql_err)?;
        let rows = stmt
            .query_map(params![user_id], map_entitlement)
            .map_err(sql_err)?;
        rows.map(|row| row.map_err(sql_err)).collect()
    }

    pub fn find_entitlement(
        &self,
        provider: &str,
        provider_reference_id: &str,
    ) -> HoneResult<Option<BillingEntitlement>> {
        if let Some(postgres) = self.cloud_postgres() {
            let provider = provider.to_string();
            let provider_reference_id = provider_reference_id.to_string();
            return run_cloud_billing(async move {
                postgres
                    .find_billing_entitlement_record(&provider, &provider_reference_id)
                    .await
            })?
            .map(entitlement_from_value)
            .transpose();
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT entitlement_id, user_id, provider, entitlement_kind,
                   provider_customer_id, provider_reference_id,
                   provider_product_id, provider_price_id,
                   purchase_email_normalized, raw_status, access_state,
                   current_period_start, current_period_end, cancel_at_period_end,
                   manage_url, grace_expires_at, last_event_id, last_event_created_at,
                   created_at, updated_at
            FROM billing_entitlements
            WHERE provider = ?1 AND provider_reference_id = ?2
            ",
            params![provider, provider_reference_id],
            map_entitlement,
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn user_has_paid_access(&self, user_id: &str) -> HoneResult<bool> {
        Ok(self
            .list_user_entitlements(user_id)?
            .iter()
            .any(BillingEntitlement::grants_paid_access))
    }

    pub fn upsert_entitlement(
        &self,
        mut entitlement: BillingEntitlement,
    ) -> HoneResult<BillingEntitlementUpsertOutcome> {
        normalize_entitlement_timestamps(&mut entitlement)?;
        validate_entitlement(&entitlement)?;
        let existing =
            self.find_entitlement(&entitlement.provider, &entitlement.provider_reference_id)?;
        if let Some(current) = existing.as_ref() {
            match compare_event_order(
                &entitlement.last_event_created_at,
                &entitlement.last_event_id,
                &current.last_event_created_at,
                &current.last_event_id,
            )? {
                Ordering::Less => return Ok(BillingEntitlementUpsertOutcome::Stale),
                Ordering::Equal => return Ok(BillingEntitlementUpsertOutcome::Duplicate),
                Ordering::Greater => {}
            }
        }

        if let Some(postgres) = self.cloud_postgres() {
            let record = serde_json::to_value(&entitlement)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            let changed = run_cloud_billing(async move {
                postgres.upsert_billing_entitlement_record(record).await
            })?;
            if !changed {
                let current = self
                    .find_entitlement(&entitlement.provider, &entitlement.provider_reference_id)?;
                return Ok(
                    if current
                        .as_ref()
                        .is_some_and(|value| value.last_event_id == entitlement.last_event_id)
                    {
                        BillingEntitlementUpsertOutcome::Duplicate
                    } else {
                        BillingEntitlementUpsertOutcome::Stale
                    },
                );
            }
            return Ok(if existing.is_some() {
                BillingEntitlementUpsertOutcome::Updated
            } else {
                BillingEntitlementUpsertOutcome::Created
            });
        }

        let conn = self.sqlite_conn()?;
        let changed = conn
            .execute(
                "
                INSERT INTO billing_entitlements(
                    entitlement_id, user_id, provider, entitlement_kind,
                    provider_customer_id, provider_reference_id,
                    provider_product_id, provider_price_id,
                    purchase_email_normalized, raw_status, access_state,
                    current_period_start, current_period_end, cancel_at_period_end,
                    manage_url, grace_expires_at, last_event_id, last_event_created_at,
                    created_at, updated_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
                ON CONFLICT(provider, provider_reference_id)
                DO UPDATE SET
                    user_id = excluded.user_id,
                    entitlement_kind = excluded.entitlement_kind,
                    provider_customer_id = excluded.provider_customer_id,
                    provider_product_id = excluded.provider_product_id,
                    provider_price_id = excluded.provider_price_id,
                    purchase_email_normalized = excluded.purchase_email_normalized,
                    raw_status = excluded.raw_status,
                    access_state = excluded.access_state,
                    current_period_start = excluded.current_period_start,
                    current_period_end = excluded.current_period_end,
                    cancel_at_period_end = excluded.cancel_at_period_end,
                    manage_url = excluded.manage_url,
                    grace_expires_at = excluded.grace_expires_at,
                    last_event_id = excluded.last_event_id,
                    last_event_created_at = excluded.last_event_created_at,
                    updated_at = excluded.updated_at
                WHERE excluded.last_event_created_at > billing_entitlements.last_event_created_at
                   OR (
                     excluded.last_event_created_at = billing_entitlements.last_event_created_at
                     AND excluded.last_event_id > billing_entitlements.last_event_id
                   )
                ",
                entitlement_params(&entitlement),
            )
            .map_err(sql_err)?;
        if changed == 0 {
            let current =
                self.find_entitlement(&entitlement.provider, &entitlement.provider_reference_id)?;
            return Ok(
                if current
                    .as_ref()
                    .is_some_and(|value| value.last_event_id == entitlement.last_event_id)
                {
                    BillingEntitlementUpsertOutcome::Duplicate
                } else {
                    BillingEntitlementUpsertOutcome::Stale
                },
            );
        }
        Ok(if existing.is_some() {
            BillingEntitlementUpsertOutcome::Updated
        } else {
            BillingEntitlementUpsertOutcome::Created
        })
    }

    pub fn record_webhook_event(
        &self,
        mut event: BillingWebhookEvent,
    ) -> HoneResult<BillingWebhookRecordOutcome> {
        normalize_webhook_timestamps(&mut event)?;
        validate_webhook_event(&event)?;
        if let Some(existing) = self.webhook_event(&event.provider, &event.event_id)? {
            if existing.payload_sha256 != event.payload_sha256 {
                return Err(HoneError::Storage(
                    "同一 billing webhook event_id 对应不同载荷摘要".to_string(),
                ));
            }
            return Ok(BillingWebhookRecordOutcome::Duplicate);
        }
        if let Some(postgres) = self.cloud_postgres() {
            let record = serde_json::to_value(&event)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            let inserted = run_cloud_billing(async move {
                postgres.insert_billing_webhook_event_record(record).await
            })?;
            if inserted {
                return Ok(BillingWebhookRecordOutcome::Inserted);
            }
            let existing = self.webhook_event(&event.provider, &event.event_id)?;
            if existing
                .as_ref()
                .is_some_and(|value| value.payload_sha256 == event.payload_sha256)
            {
                return Ok(BillingWebhookRecordOutcome::Duplicate);
            }
            return Err(HoneError::Storage(
                "同一 billing webhook event_id 对应不同载荷摘要".to_string(),
            ));
        }
        let payload = serde_json::to_string(&event.normalized_payload)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let conn = self.sqlite_conn()?;
        let inserted = conn
            .execute(
                "
            INSERT OR IGNORE INTO billing_webhook_events(
                provider, event_id, event_type, object_id, payload_sha256,
                provider_created_at, processing_state, attempt_count,
                last_error, received_at, processing_started_at, processed_at,
                normalized_payload
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ",
                params![
                    &event.provider,
                    &event.event_id,
                    &event.event_type,
                    &event.object_id,
                    &event.payload_sha256,
                    &event.provider_created_at,
                    &event.processing_state,
                    event.attempt_count,
                    &event.last_error,
                    &event.received_at,
                    &event.processing_started_at,
                    &event.processed_at,
                    payload,
                ],
            )
            .map_err(sql_err)?;
        if inserted > 0 {
            return Ok(BillingWebhookRecordOutcome::Inserted);
        }
        let existing = self.webhook_event(&event.provider, &event.event_id)?;
        if existing
            .as_ref()
            .is_some_and(|value| value.payload_sha256 == event.payload_sha256)
        {
            Ok(BillingWebhookRecordOutcome::Duplicate)
        } else {
            Err(HoneError::Storage(
                "同一 billing webhook event_id 对应不同载荷摘要".to_string(),
            ))
        }
    }

    pub fn webhook_event(
        &self,
        provider: &str,
        event_id: &str,
    ) -> HoneResult<Option<BillingWebhookEvent>> {
        if let Some(postgres) = self.cloud_postgres() {
            let provider = provider.to_string();
            let event_id = event_id.to_string();
            return run_cloud_billing(async move {
                postgres
                    .billing_webhook_event_record(&provider, &event_id)
                    .await
            })?
            .map(webhook_from_value)
            .transpose();
        }
        let conn = self.sqlite_conn()?;
        conn.query_row(
            "
            SELECT provider, event_id, event_type, object_id, payload_sha256,
                   provider_created_at, processing_state, attempt_count,
                   last_error, received_at, processing_started_at, processed_at,
                   normalized_payload
            FROM billing_webhook_events
            WHERE provider = ?1 AND event_id = ?2
            ",
            params![provider, event_id],
            map_webhook_event,
        )
        .optional()
        .map_err(sql_err)
    }

    pub fn claim_webhook_event(
        &self,
        provider: &str,
        event_id: &str,
    ) -> HoneResult<Option<BillingWebhookEvent>> {
        let Some(mut event) = self.webhook_event(provider, event_id)? else {
            return Ok(None);
        };
        let now = chrono::Utc::now();
        let stale_before = now - chrono::Duration::minutes(BILLING_WEBHOOK_LEASE_MINUTES);
        let lease_expired = event.processing_state == BILLING_EVENT_PROCESSING
            && event
                .processing_started_at
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .is_none_or(|started| started <= stale_before);
        if event.attempt_count >= BILLING_WEBHOOK_MAX_ATTEMPTS
            || (!matches!(
                event.processing_state.as_str(),
                BILLING_EVENT_RECEIVED | BILLING_EVENT_FAILED
            ) && !lease_expired)
        {
            return Ok(None);
        }
        event.processing_state = BILLING_EVENT_PROCESSING.to_string();
        event.attempt_count = event.attempt_count.saturating_add(1);
        event.last_error = None;
        event.processing_started_at =
            Some(now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
        if let Some(postgres) = self.cloud_postgres() {
            let provider = provider.to_string();
            let event_id = event_id.to_string();
            let record = serde_json::to_value(&event)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            let claimed = run_cloud_billing(async move {
                postgres
                    .claim_billing_webhook_event(
                        &provider,
                        &event_id,
                        &stale_before.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                        BILLING_WEBHOOK_MAX_ATTEMPTS,
                        record,
                    )
                    .await
            })?;
            return Ok(claimed.then_some(event));
        }
        let conn = self.sqlite_conn()?;
        let changed = conn
            .execute(
                "
                UPDATE billing_webhook_events
                SET processing_state = 'processing',
                    attempt_count = attempt_count + 1,
                    last_error = NULL,
                    processing_started_at = ?5
                WHERE provider = ?1
                  AND event_id = ?2
                  AND attempt_count < ?3
                  AND (
                    processing_state IN ('received', 'failed')
                    OR (
                      processing_state = 'processing'
                      AND (processing_started_at IS NULL OR processing_started_at <= ?4)
                    )
                  )
                ",
                params![
                    provider,
                    event_id,
                    BILLING_WEBHOOK_MAX_ATTEMPTS,
                    stale_before.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                    &event.processing_started_at,
                ],
            )
            .map_err(sql_err)?;
        Ok((changed > 0).then_some(event))
    }

    pub fn claimable_webhook_event_ids(
        &self,
        provider: &str,
        limit: usize,
    ) -> HoneResult<Vec<String>> {
        if provider != BILLING_PROVIDER_STRIPE {
            return Err(HoneError::Config(
                "billing webhook provider 不合法".to_string(),
            ));
        }
        let stale_before = (chrono::Utc::now()
            - chrono::Duration::minutes(BILLING_WEBHOOK_LEASE_MINUTES))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let limit = limit.clamp(1, 1000);
        if let Some(postgres) = self.cloud_postgres() {
            let provider = provider.to_string();
            return run_cloud_billing(async move {
                postgres
                    .list_claimable_billing_webhook_event_ids(
                        &provider,
                        &stale_before,
                        BILLING_WEBHOOK_MAX_ATTEMPTS,
                        limit,
                    )
                    .await
            });
        }
        let conn = self.sqlite_conn()?;
        let mut statement = conn
            .prepare(
                "
                SELECT event_id
                FROM billing_webhook_events
                WHERE provider = ?1
                  AND attempt_count < ?2
                  AND (
                    processing_state IN ('received', 'failed')
                    OR (
                      processing_state = 'processing'
                      AND (processing_started_at IS NULL OR processing_started_at <= ?3)
                    )
                  )
                ORDER BY received_at, event_id
                LIMIT ?4
                ",
            )
            .map_err(sql_err)?;
        statement
            .query_map(
                params![
                    provider,
                    BILLING_WEBHOOK_MAX_ATTEMPTS,
                    stale_before,
                    i64::try_from(limit).unwrap_or(1000),
                ],
                |row| row.get::<_, String>(0),
            )
            .map_err(sql_err)?
            .map(|row| row.map_err(sql_err))
            .collect()
    }

    pub fn finish_webhook_event(
        &self,
        provider: &str,
        event_id: &str,
        expected_attempt_count: u32,
        result: Result<(), &str>,
    ) -> HoneResult<bool> {
        let Some(mut event) = self.webhook_event(provider, event_id)? else {
            return Err(HoneError::Storage(
                "billing webhook event 不存在".to_string(),
            ));
        };
        if event.processing_state != BILLING_EVENT_PROCESSING
            || event.attempt_count != expected_attempt_count
        {
            return Ok(false);
        }
        match result {
            Ok(()) => {
                event.processing_state = BILLING_EVENT_PROCESSED.to_string();
                event.last_error = None;
                event.processed_at = Some(beijing_now_rfc3339());
            }
            Err(error) => {
                event.processing_state = BILLING_EVENT_FAILED.to_string();
                event.last_error = Some(error.chars().take(1000).collect());
                event.processed_at = None;
            }
        }
        event.processing_started_at = None;
        if let Some(postgres) = self.cloud_postgres() {
            let provider = provider.to_string();
            let event_id = event_id.to_string();
            let state = event.processing_state.clone();
            let last_error = event.last_error.clone();
            let processed_at = event.processed_at.clone();
            let record = serde_json::to_value(&event)
                .map_err(|err| HoneError::Serialization(err.to_string()))?;
            return run_cloud_billing(async move {
                postgres
                    .finish_billing_webhook_event(
                        &provider,
                        &event_id,
                        expected_attempt_count,
                        &state,
                        last_error.as_deref(),
                        processed_at.as_deref(),
                        record,
                    )
                    .await
            });
        }
        let conn = self.sqlite_conn()?;
        let changed = conn
            .execute(
                "
            UPDATE billing_webhook_events
            SET processing_state = ?3,
                last_error = ?4,
                processed_at = ?5,
                processing_started_at = NULL
            WHERE provider = ?1
              AND event_id = ?2
              AND processing_state = 'processing'
              AND attempt_count = ?6
            ",
                params![
                    provider,
                    event_id,
                    &event.processing_state,
                    &event.last_error,
                    &event.processed_at,
                    expected_attempt_count,
                ],
            )
            .map_err(sql_err)?;
        Ok(changed > 0)
    }
}

fn validate_entitlement(value: &BillingEntitlement) -> HoneResult<()> {
    for (name, field) in [
        ("entitlement_id", value.entitlement_id.as_str()),
        ("user_id", value.user_id.as_str()),
        (
            "provider_reference_id",
            value.provider_reference_id.as_str(),
        ),
        ("raw_status", value.raw_status.as_str()),
        ("last_event_id", value.last_event_id.as_str()),
    ] {
        if field.trim().is_empty() {
            return Err(HoneError::Config(format!("billing {name} 不能为空")));
        }
    }
    if !matches!(
        value.provider.as_str(),
        BILLING_PROVIDER_STRIPE | BILLING_PROVIDER_DOMESTIC_INVITE
    ) {
        return Err(HoneError::Config("billing provider 不合法".to_string()));
    }
    let kind_matches_provider = match value.provider.as_str() {
        BILLING_PROVIDER_STRIPE => matches!(
            value.entitlement_kind.as_str(),
            BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION | BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
        ),
        BILLING_PROVIDER_DOMESTIC_INVITE => {
            value.entitlement_kind == BILLING_ENTITLEMENT_DOMESTIC_INVITE
        }
        _ => false,
    };
    if !kind_matches_provider {
        return Err(HoneError::Config(
            "billing entitlement_kind 与 provider 不匹配".to_string(),
        ));
    }
    if !matches!(
        value.access_state.as_str(),
        BILLING_ACCESS_PENDING
            | BILLING_ACCESS_ACTIVE
            | BILLING_ACCESS_GRACE
            | BILLING_ACCESS_INACTIVE
    ) {
        return Err(HoneError::Config("billing access_state 不合法".to_string()));
    }
    parse_timestamp(&value.last_event_created_at, "last_event_created_at")?;
    parse_timestamp(&value.created_at, "created_at")?;
    parse_timestamp(&value.updated_at, "updated_at")?;
    if let Some(value) = value.grace_expires_at.as_deref() {
        parse_timestamp(value, "grace_expires_at")?;
    }
    if value.entitlement_kind == BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE {
        if value.cancel_at_period_end {
            return Err(HoneError::Config(
                "fixed_term_purchase 不能设置 cancel_at_period_end".to_string(),
            ));
        }
        if value.access_state == BILLING_ACCESS_ACTIVE && value.current_period_end.is_none() {
            return Err(HoneError::Config(
                "active fixed_term_purchase 必须设置 current_period_end".to_string(),
            ));
        }
    }
    Ok(())
}

fn normalize_entitlement_timestamps(value: &mut BillingEntitlement) -> HoneResult<()> {
    normalize_timestamp(&mut value.last_event_created_at, "last_event_created_at")?;
    normalize_timestamp(&mut value.created_at, "created_at")?;
    normalize_timestamp(&mut value.updated_at, "updated_at")?;
    normalize_optional_timestamp(&mut value.current_period_start, "current_period_start")?;
    normalize_optional_timestamp(&mut value.current_period_end, "current_period_end")?;
    normalize_optional_timestamp(&mut value.grace_expires_at, "grace_expires_at")
}

fn normalize_webhook_timestamps(value: &mut BillingWebhookEvent) -> HoneResult<()> {
    normalize_timestamp(&mut value.provider_created_at, "provider_created_at")?;
    normalize_timestamp(&mut value.received_at, "received_at")?;
    normalize_optional_timestamp(&mut value.processing_started_at, "processing_started_at")?;
    normalize_optional_timestamp(&mut value.processed_at, "processed_at")
}

fn normalize_optional_timestamp(value: &mut Option<String>, name: &str) -> HoneResult<()> {
    if let Some(value) = value {
        normalize_timestamp(value, name)?;
    }
    Ok(())
}

fn normalize_timestamp(value: &mut String, name: &str) -> HoneResult<()> {
    let timestamp = parse_timestamp(value, name)?;
    *value = timestamp
        .with_timezone(&chrono::Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    Ok(())
}

fn validate_webhook_event(value: &BillingWebhookEvent) -> HoneResult<()> {
    if value.provider != BILLING_PROVIDER_STRIPE {
        return Err(HoneError::Config(
            "billing webhook provider 不合法".to_string(),
        ));
    }
    if value.event_id.trim().is_empty()
        || value.event_type.trim().is_empty()
        || value.payload_sha256.len() != 64
    {
        return Err(HoneError::Config(
            "billing webhook 事件字段不合法".to_string(),
        ));
    }
    if !matches!(
        value.processing_state.as_str(),
        BILLING_EVENT_RECEIVED
            | BILLING_EVENT_PROCESSING
            | BILLING_EVENT_PROCESSED
            | BILLING_EVENT_FAILED
    ) {
        return Err(HoneError::Config(
            "billing webhook processing_state 不合法".to_string(),
        ));
    }
    parse_timestamp(&value.provider_created_at, "provider_created_at")?;
    parse_timestamp(&value.received_at, "received_at")?;
    Ok(())
}

fn compare_event_order(
    left_at: &str,
    left_id: &str,
    right_at: &str,
    right_id: &str,
) -> HoneResult<Ordering> {
    let left = parse_timestamp(left_at, "last_event_created_at")?;
    let right = parse_timestamp(right_at, "last_event_created_at")?;
    Ok(left.cmp(&right).then_with(|| left_id.cmp(right_id)))
}

fn parse_timestamp(value: &str, name: &str) -> HoneResult<DateTime<chrono::FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| HoneError::Config(format!("billing {name} 时间格式不合法")))
}

fn entitlement_params(
    value: &BillingEntitlement,
) -> rusqlite::ParamsFromIter<Vec<rusqlite::types::Value>> {
    use rusqlite::types::Value;
    let values = vec![
        Value::Text(value.entitlement_id.clone()),
        Value::Text(value.user_id.clone()),
        Value::Text(value.provider.clone()),
        Value::Text(value.entitlement_kind.clone()),
        option_text(&value.provider_customer_id),
        Value::Text(value.provider_reference_id.clone()),
        option_text(&value.provider_product_id),
        option_text(&value.provider_price_id),
        option_text(&value.purchase_email_normalized),
        Value::Text(value.raw_status.clone()),
        Value::Text(value.access_state.clone()),
        option_text(&value.current_period_start),
        option_text(&value.current_period_end),
        Value::Integer(i64::from(value.cancel_at_period_end)),
        option_text(&value.manage_url),
        option_text(&value.grace_expires_at),
        Value::Text(value.last_event_id.clone()),
        Value::Text(value.last_event_created_at.clone()),
        Value::Text(value.created_at.clone()),
        Value::Text(value.updated_at.clone()),
    ];
    rusqlite::params_from_iter(values)
}

fn option_text(value: &Option<String>) -> rusqlite::types::Value {
    value
        .clone()
        .map(rusqlite::types::Value::Text)
        .unwrap_or(rusqlite::types::Value::Null)
}

fn map_entitlement(row: &Row<'_>) -> rusqlite::Result<BillingEntitlement> {
    Ok(BillingEntitlement {
        entitlement_id: row.get(0)?,
        user_id: row.get(1)?,
        provider: row.get(2)?,
        entitlement_kind: row.get(3)?,
        provider_customer_id: row.get(4)?,
        provider_reference_id: row.get(5)?,
        provider_product_id: row.get(6)?,
        provider_price_id: row.get(7)?,
        purchase_email_normalized: row.get(8)?,
        raw_status: row.get(9)?,
        access_state: row.get(10)?,
        current_period_start: row.get(11)?,
        current_period_end: row.get(12)?,
        cancel_at_period_end: row.get::<_, i64>(13)? != 0,
        manage_url: row.get(14)?,
        grace_expires_at: row.get(15)?,
        last_event_id: row.get(16)?,
        last_event_created_at: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
    })
}

fn map_webhook_event(row: &Row<'_>) -> rusqlite::Result<BillingWebhookEvent> {
    Ok(BillingWebhookEvent {
        provider: row.get(0)?,
        event_id: row.get(1)?,
        event_type: row.get(2)?,
        object_id: row.get(3)?,
        payload_sha256: row.get(4)?,
        provider_created_at: row.get(5)?,
        processing_state: row.get(6)?,
        attempt_count: row.get(7)?,
        last_error: row.get(8)?,
        received_at: row.get(9)?,
        processing_started_at: row.get(10)?,
        processed_at: row.get(11)?,
        normalized_payload: serde_json::from_str(&row.get::<_, String>(12)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
    })
}

fn entitlement_from_value(value: serde_json::Value) -> HoneResult<BillingEntitlement> {
    serde_json::from_value(value).map_err(|err| HoneError::Serialization(err.to_string()))
}

fn webhook_from_value(value: serde_json::Value) -> HoneResult<BillingWebhookEvent> {
    serde_json::from_value(value).map_err(|err| HoneError::Serialization(err.to_string()))
}

fn run_cloud_billing<T, F>(future: F) -> HoneResult<T>
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
        .map_err(|_| HoneError::Storage("cloud billing worker panicked".to_string()))?;
    }
    let runtime =
        tokio::runtime::Runtime::new().map_err(|err| HoneError::Config(err.to_string()))?;
    runtime.block_on(future)
}

fn lock_err<T>(error: std::sync::PoisonError<T>) -> HoneError {
    HoneError::Storage(format!("billing sqlite lock poisoned: {error}"))
}

fn sql_err(error: rusqlite::Error) -> HoneError {
    HoneError::Storage(format!("billing sqlite error: {error}"))
}

fn ensure_billing_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> HoneResult<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(sql_err)?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(sql_err)?;
    for value in columns {
        if value.map_err(sql_err)? == column {
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

fn migrate_sqlite_billing_to_typed_entitlements(conn: &Connection) -> HoneResult<()> {
    let entitlement_sql = sqlite_table_sql(conn, "billing_entitlements")?;
    let webhook_sql = sqlite_table_sql(conn, "billing_webhook_events")?;
    if entitlement_sql.contains("provider IN ('stripe', 'domestic_invite')")
        && entitlement_sql.contains("entitlement_kind")
        && entitlement_sql.contains("provider_reference_id")
        && webhook_sql.contains("provider = 'stripe'")
    {
        return Ok(());
    }

    let reference_column = if entitlement_sql.contains("provider_reference_id") {
        "provider_reference_id"
    } else {
        "provider_subscription_id"
    };
    let kind_expression = if entitlement_sql.contains("entitlement_kind") {
        "entitlement_kind"
    } else {
        "CASE WHEN provider = 'stripe' THEN 'recurring_subscription' ELSE 'domestic_invite' END"
    };

    conn.execute_batch(
        &format!(
            "
        BEGIN IMMEDIATE;

        ALTER TABLE billing_entitlements RENAME TO billing_entitlements_before_stripe_only;
        CREATE TABLE billing_entitlements (
            entitlement_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL CHECK (provider IN ('stripe', 'domestic_invite')),
            entitlement_kind TEXT NOT NULL CHECK (entitlement_kind IN ('recurring_subscription', 'fixed_term_purchase', 'domestic_invite')),
            provider_customer_id TEXT,
            provider_reference_id TEXT NOT NULL,
            provider_product_id TEXT,
            provider_price_id TEXT,
            purchase_email_normalized TEXT,
            raw_status TEXT NOT NULL,
            access_state TEXT NOT NULL CHECK (access_state IN ('pending', 'active', 'grace', 'inactive')),
            current_period_start TEXT,
            current_period_end TEXT,
            cancel_at_period_end INTEGER NOT NULL DEFAULT 0 CHECK (cancel_at_period_end IN (0, 1)),
            manage_url TEXT,
            grace_expires_at TEXT,
            last_event_id TEXT NOT NULL,
            last_event_created_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(provider, provider_reference_id),
            FOREIGN KEY(user_id) REFERENCES web_invite_users(user_id) ON DELETE CASCADE
        );
        INSERT INTO billing_entitlements (
            entitlement_id, user_id, provider, entitlement_kind,
            provider_customer_id, provider_reference_id,
            provider_product_id, provider_price_id,
            purchase_email_normalized, raw_status, access_state,
            current_period_start, current_period_end, cancel_at_period_end,
            manage_url, grace_expires_at, last_event_id, last_event_created_at,
            created_at, updated_at
        )
        SELECT
            entitlement_id, user_id, provider, {kind_expression},
            provider_customer_id, {reference_column},
            provider_product_id, provider_price_id,
            purchase_email_normalized, raw_status, access_state,
            current_period_start, current_period_end, cancel_at_period_end,
            manage_url, grace_expires_at, last_event_id, last_event_created_at,
            created_at, updated_at
        FROM billing_entitlements_before_stripe_only
        WHERE provider IN ('stripe', 'domestic_invite');
        DROP TABLE billing_entitlements_before_stripe_only;

        ALTER TABLE billing_webhook_events RENAME TO billing_webhook_events_before_stripe_only;
        CREATE TABLE billing_webhook_events (
            provider TEXT NOT NULL CHECK (provider = 'stripe'),
            event_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            object_id TEXT,
            payload_sha256 TEXT NOT NULL,
            provider_created_at TEXT NOT NULL,
            processing_state TEXT NOT NULL CHECK (processing_state IN ('received', 'processing', 'processed', 'failed')),
            attempt_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            received_at TEXT NOT NULL,
            processing_started_at TEXT,
            processed_at TEXT,
            normalized_payload TEXT NOT NULL DEFAULT '{{}}',
            PRIMARY KEY(provider, event_id)
        );
        INSERT INTO billing_webhook_events (
            provider, event_id, event_type, object_id, payload_sha256,
            provider_created_at, processing_state, attempt_count, last_error,
            received_at, processing_started_at, processed_at, normalized_payload
        )
        SELECT
            provider, event_id, event_type, object_id, payload_sha256,
            provider_created_at, processing_state, attempt_count, last_error,
            received_at, processing_started_at, processed_at, normalized_payload
        FROM billing_webhook_events_before_stripe_only
        WHERE provider = 'stripe';
        DROP TABLE billing_webhook_events_before_stripe_only;

        CREATE INDEX idx_billing_entitlements_user_access
            ON billing_entitlements(user_id, access_state);
        CREATE INDEX idx_billing_entitlements_purchase_email
            ON billing_entitlements(purchase_email_normalized)
            WHERE purchase_email_normalized IS NOT NULL;
        CREATE INDEX idx_billing_entitlements_customer
            ON billing_entitlements(provider, provider_customer_id)
            WHERE provider_customer_id IS NOT NULL;
        CREATE INDEX idx_billing_webhook_events_processing
            ON billing_webhook_events(processing_state, received_at);
        CREATE INDEX idx_billing_webhook_events_retry
            ON billing_webhook_events(
                provider, processing_state, processing_started_at, received_at
            );

        COMMIT;
        "
        ),
    )
    .map_err(sql_err)
}

fn sqlite_table_sql(conn: &Connection, table: &str) -> HoneResult<String> {
    conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [table],
        |row| row.get(0),
    )
    .optional()
    .map_err(sql_err)?
    .ok_or_else(|| HoneError::Storage(format!("billing sqlite table {table} 不存在")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> BillingStorage {
        let root = std::env::temp_dir().join(format!("hone-billing-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        let auth = Connection::open(root.join("sessions.sqlite3")).expect("auth db");
        auth.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE web_invite_users(user_id TEXT PRIMARY KEY);
            INSERT INTO web_invite_users(user_id) VALUES ('user_1');
            ",
        )
        .expect("auth schema");
        BillingStorage::new(root.join("sessions.sqlite3")).expect("billing")
    }

    fn entitlement_for(
        provider: &str,
        reference_id: &str,
        event_id: &str,
        event_at: &str,
        state: &str,
    ) -> BillingEntitlement {
        BillingEntitlement {
            entitlement_id: BillingStorage::entitlement_id(provider, reference_id),
            user_id: "user_1".to_string(),
            provider: provider.to_string(),
            entitlement_kind: if provider == BILLING_PROVIDER_STRIPE {
                BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION.to_string()
            } else {
                BILLING_ENTITLEMENT_DOMESTIC_INVITE.to_string()
            },
            provider_customer_id: Some("cus_1".to_string()),
            provider_reference_id: reference_id.to_string(),
            provider_product_id: Some("prod_1".to_string()),
            provider_price_id: Some("price_1".to_string()),
            purchase_email_normalized: Some("buyer@example.com".to_string()),
            raw_status: state.to_string(),
            access_state: state.to_string(),
            current_period_start: None,
            current_period_end: None,
            cancel_at_period_end: false,
            manage_url: None,
            grace_expires_at: None,
            last_event_id: event_id.to_string(),
            last_event_created_at: event_at.to_string(),
            created_at: "2026-08-03T00:00:00+00:00".to_string(),
            updated_at: "2026-08-03T00:00:00+00:00".to_string(),
        }
    }

    fn entitlement(event_id: &str, event_at: &str, state: &str) -> BillingEntitlement {
        entitlement_for(BILLING_PROVIDER_STRIPE, "sub_1", event_id, event_at, state)
    }

    #[test]
    fn active_stripe_entitlement_grants_access_and_stale_events_cannot_revoke() {
        let storage = test_storage();
        assert_eq!(
            storage
                .upsert_entitlement(entitlement(
                    "evt_2",
                    "2026-08-03T10:00:00+08:00",
                    BILLING_ACCESS_ACTIVE,
                ))
                .expect("insert"),
            BillingEntitlementUpsertOutcome::Created
        );
        assert!(storage.user_has_paid_access("user_1").expect("access"));
        assert_eq!(
            storage
                .find_entitlement(BILLING_PROVIDER_STRIPE, "sub_1")
                .expect("read")
                .expect("entitlement")
                .last_event_created_at,
            "2026-08-03T02:00:00.000Z"
        );
        assert_eq!(
            storage
                .upsert_entitlement(entitlement(
                    "evt_1",
                    "2026-08-03T09:00:00+08:00",
                    BILLING_ACCESS_INACTIVE,
                ))
                .expect("stale"),
            BillingEntitlementUpsertOutcome::Stale
        );
        assert!(storage.user_has_paid_access("user_1").expect("access"));
    }

    #[test]
    fn legacy_provider_tables_are_rebuilt_as_stripe_only() {
        let root =
            std::env::temp_dir().join(format!("hone-billing-stripe-only-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("root");
        let database_path = root.join("sessions.sqlite3");
        {
            let conn = Connection::open(&database_path).expect("legacy db");
            conn.execute_batch(
                "
                PRAGMA foreign_keys = ON;
                CREATE TABLE web_invite_users(user_id TEXT PRIMARY KEY);
                INSERT INTO web_invite_users(user_id) VALUES ('user_1');
                CREATE TABLE billing_entitlements (
                    entitlement_id TEXT PRIMARY KEY,
                    user_id TEXT NOT NULL,
                    provider TEXT NOT NULL CHECK (provider IN ('legacy', 'stripe', 'domestic_invite')),
                    provider_customer_id TEXT,
                    provider_subscription_id TEXT NOT NULL,
                    provider_product_id TEXT,
                    provider_price_id TEXT,
                    purchase_email_normalized TEXT,
                    raw_status TEXT NOT NULL,
                    access_state TEXT NOT NULL CHECK (access_state IN ('pending', 'active', 'grace', 'inactive')),
                    current_period_start TEXT,
                    current_period_end TEXT,
                    cancel_at_period_end INTEGER NOT NULL DEFAULT 0 CHECK (cancel_at_period_end IN (0, 1)),
                    manage_url TEXT,
                    grace_expires_at TEXT,
                    last_event_id TEXT NOT NULL,
                    last_event_created_at TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    UNIQUE(provider, provider_subscription_id),
                    FOREIGN KEY(user_id) REFERENCES web_invite_users(user_id) ON DELETE CASCADE
                );
                INSERT INTO billing_entitlements (
                    entitlement_id, user_id, provider, provider_subscription_id,
                    raw_status, access_state, last_event_id, last_event_created_at,
                    created_at, updated_at
                ) VALUES
                    ('ent_stripe', 'user_1', 'stripe', 'sub_1', 'active', 'active',
                     'evt_stripe', '2026-08-04T00:00:00Z', '2026-08-04T00:00:00Z', '2026-08-04T00:00:00Z'),
                    ('ent_legacy', 'user_1', 'legacy', 'mem_1', 'active', 'active',
                     'evt_legacy', '2026-08-04T00:00:00Z', '2026-08-04T00:00:00Z', '2026-08-04T00:00:00Z');
                CREATE TABLE billing_webhook_events (
                    provider TEXT NOT NULL CHECK (provider IN ('legacy', 'stripe')),
                    event_id TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    object_id TEXT,
                    payload_sha256 TEXT NOT NULL,
                    provider_created_at TEXT NOT NULL,
                    processing_state TEXT NOT NULL CHECK (processing_state IN ('received', 'processing', 'processed', 'failed')),
                    attempt_count INTEGER NOT NULL DEFAULT 0,
                    last_error TEXT,
                    received_at TEXT NOT NULL,
                    processing_started_at TEXT,
                    processed_at TEXT,
                    normalized_payload TEXT NOT NULL DEFAULT '{}',
                    PRIMARY KEY(provider, event_id)
                );
                INSERT INTO billing_webhook_events (
                    provider, event_id, event_type, payload_sha256,
                    provider_created_at, processing_state, received_at
                ) VALUES
                    ('stripe', 'evt_stripe', 'invoice.paid',
                     'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                     '2026-08-04T00:00:00Z', 'processed', '2026-08-04T00:00:00Z'),
                    ('legacy', 'evt_legacy', 'membership.activated',
                     'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                     '2026-08-04T00:00:00Z', 'processed', '2026-08-04T00:00:00Z');
                ",
            )
            .expect("legacy schema");
        }

        let storage = BillingStorage::new(&database_path).expect("migrate");
        let conn = storage.sqlite_conn().expect("connection");
        let entitlement_providers: Vec<String> = conn
            .prepare("SELECT provider FROM billing_entitlements ORDER BY provider")
            .expect("prepare entitlements")
            .query_map([], |row| row.get(0))
            .expect("query entitlements")
            .collect::<Result<_, _>>()
            .expect("entitlement providers");
        let event_providers: Vec<String> = conn
            .prepare("SELECT provider FROM billing_webhook_events ORDER BY provider")
            .expect("prepare events")
            .query_map([], |row| row.get(0))
            .expect("query events")
            .collect::<Result<_, _>>()
            .expect("event providers");
        assert_eq!(entitlement_providers, vec!["stripe"]);
        assert_eq!(event_providers, vec!["stripe"]);
        assert!(
            sqlite_table_sql(&conn, "billing_entitlements")
                .expect("entitlement schema")
                .contains("provider IN ('stripe', 'domestic_invite')")
        );
        assert!(
            sqlite_table_sql(&conn, "billing_entitlements")
                .expect("entitlement schema")
                .contains("provider_reference_id")
        );
        let migrated: (String, String) = conn
            .query_row(
                "SELECT entitlement_kind, provider_reference_id FROM billing_entitlements WHERE entitlement_id = 'ent_stripe'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("migrated entitlement");
        assert_eq!(
            migrated,
            (
                BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION.to_string(),
                "sub_1".to_string()
            )
        );
        assert!(
            sqlite_table_sql(&conn, "billing_webhook_events")
                .expect("event schema")
                .contains("provider = 'stripe'")
        );
    }

    #[test]
    fn old_subscription_events_cannot_revoke_a_repurchase_and_all_inactive_denies_access() {
        let storage = test_storage();
        storage
            .upsert_entitlement(entitlement_for(
                BILLING_PROVIDER_STRIPE,
                "sub_old",
                "evt_old_paid",
                "2026-08-03T01:00:00+00:00",
                BILLING_ACCESS_ACTIVE,
            ))
            .expect("old subscription");
        storage
            .upsert_entitlement(entitlement_for(
                BILLING_PROVIDER_STRIPE,
                "sub_repurchase",
                "evt_repurchase_paid",
                "2026-08-03T02:00:00+00:00",
                BILLING_ACCESS_ACTIVE,
            ))
            .expect("repurchased subscription");

        storage
            .upsert_entitlement(entitlement_for(
                BILLING_PROVIDER_STRIPE,
                "sub_old",
                "evt_old_canceled_late",
                "2026-08-03T03:00:00+00:00",
                BILLING_ACCESS_INACTIVE,
            ))
            .expect("late event for old subscription");

        assert!(
            storage.user_has_paid_access("user_1").expect("access"),
            "an event for the old subscription must not revoke the repurchase"
        );
        assert_eq!(
            storage
                .find_entitlement(BILLING_PROVIDER_STRIPE, "sub_repurchase")
                .expect("read repurchase")
                .expect("repurchase entitlement")
                .access_state,
            BILLING_ACCESS_ACTIVE
        );

        storage
            .upsert_entitlement(entitlement_for(
                BILLING_PROVIDER_STRIPE,
                "sub_repurchase",
                "evt_repurchase_canceled",
                "2026-08-03T04:00:00+00:00",
                BILLING_ACCESS_INACTIVE,
            ))
            .expect("cancel repurchase");
        assert!(
            !storage.user_has_paid_access("user_1").expect("access"),
            "access must be denied only after every entitlement is inactive"
        );
    }

    #[test]
    fn webhook_inbox_rejects_event_id_payload_conflicts_and_tracks_processing() {
        let storage = test_storage();
        let event = BillingWebhookEvent {
            provider: BILLING_PROVIDER_STRIPE.to_string(),
            event_id: "evt_1".to_string(),
            event_type: "invoice.paid".to_string(),
            object_id: Some("in_1".to_string()),
            payload_sha256: "a".repeat(64),
            provider_created_at: "2026-08-03T01:00:00+00:00".to_string(),
            processing_state: BILLING_EVENT_RECEIVED.to_string(),
            attempt_count: 0,
            last_error: None,
            received_at: "2026-08-03T01:00:01+00:00".to_string(),
            processing_started_at: None,
            processed_at: None,
            normalized_payload: serde_json::json!({"subscription_id": "sub_1"}),
        };
        assert_eq!(
            storage.record_webhook_event(event.clone()).expect("insert"),
            BillingWebhookRecordOutcome::Inserted
        );
        assert_eq!(
            storage
                .record_webhook_event(event.clone())
                .expect("duplicate"),
            BillingWebhookRecordOutcome::Duplicate
        );
        let mut conflict = event;
        conflict.payload_sha256 = "b".repeat(64);
        assert!(storage.record_webhook_event(conflict).is_err());

        assert_eq!(
            storage
                .claimable_webhook_event_ids(BILLING_PROVIDER_STRIPE, 10)
                .expect("claimable"),
            vec!["evt_1".to_string()]
        );

        let claimed = storage
            .claim_webhook_event(BILLING_PROVIDER_STRIPE, "evt_1")
            .expect("claim")
            .expect("claimed");
        assert_eq!(claimed.attempt_count, 1);
        assert!(claimed.processing_started_at.is_some());
        assert!(
            storage
                .claimable_webhook_event_ids(BILLING_PROVIDER_STRIPE, 10)
                .expect("leased")
                .is_empty()
        );
        assert!(
            storage
                .claim_webhook_event(BILLING_PROVIDER_STRIPE, "evt_1")
                .expect("second claim")
                .is_none()
        );
        storage
            .sqlite_conn()
            .expect("connection")
            .execute(
                "UPDATE billing_webhook_events SET processing_started_at = ?1 WHERE event_id = ?2",
                params!["2020-08-03T01:00:00.000Z", "evt_1"],
            )
            .expect("expire lease");
        let reclaimed = storage
            .claim_webhook_event(BILLING_PROVIDER_STRIPE, "evt_1")
            .expect("reclaim")
            .expect("reclaimed");
        assert_eq!(reclaimed.attempt_count, 2);
        assert!(
            !storage
                .finish_webhook_event(BILLING_PROVIDER_STRIPE, "evt_1", 1, Ok(()))
                .expect("reject stale completion")
        );
        assert_eq!(
            storage
                .webhook_event(BILLING_PROVIDER_STRIPE, "evt_1")
                .expect("event after stale completion")
                .expect("present after stale completion")
                .processing_state,
            BILLING_EVENT_PROCESSING
        );
        assert!(
            storage
                .finish_webhook_event(
                    BILLING_PROVIDER_STRIPE,
                    "evt_1",
                    reclaimed.attempt_count,
                    Ok(()),
                )
                .expect("finish")
        );
        assert_eq!(
            storage
                .webhook_event(BILLING_PROVIDER_STRIPE, "evt_1")
                .expect("event")
                .expect("present")
                .processing_state,
            BILLING_EVENT_PROCESSED
        );
    }

    #[test]
    fn grace_access_requires_an_unexpired_deadline() {
        let mut value = entitlement(
            "evt_grace",
            "2026-08-03T03:00:00+00:00",
            BILLING_ACCESS_GRACE,
        );
        value.grace_expires_at = Some("2099-08-10T03:00:00+00:00".to_string());
        assert!(value.grants_paid_access());
        value.grace_expires_at = Some("2020-08-10T03:00:00+00:00".to_string());
        assert!(!value.grants_paid_access());
        value.grace_expires_at = None;
        assert!(!value.grants_paid_access());
    }

    #[test]
    fn fixed_term_access_requires_an_unexpired_period_end() {
        let mut value = entitlement_for(
            BILLING_PROVIDER_STRIPE,
            "pi_fixed",
            "evt_fixed",
            "2026-08-03T03:00:00+00:00",
            BILLING_ACCESS_ACTIVE,
        );
        value.entitlement_kind = BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE.to_string();
        value.current_period_end = Some("2099-08-03T03:00:00+00:00".to_string());
        assert!(value.grants_paid_access());
        value.current_period_end = Some("2020-08-03T03:00:00+00:00".to_string());
        assert!(!value.grants_paid_access());
        value.current_period_end = None;
        assert!(!value.grants_paid_access());
    }
}
