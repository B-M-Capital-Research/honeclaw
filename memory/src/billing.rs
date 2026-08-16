use std::cmp::Ordering;

use chrono::DateTime;
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::cloud_sync::{ensure_cloud_schema_once, run_cloud_sync};
use hone_core::{HoneError, HoneResult, local_now_rfc3339};
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
    postgres: CloudPgRuntime,
    _test_postgres_lease: Option<std::sync::Arc<crate::test_postgres::TestPostgresLease>>,
}

impl BillingStorage {
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

    pub fn entitlement_id(provider: &str, provider_reference_id: &str) -> String {
        let digest = Sha256::digest(format!("{provider}:{provider_reference_id}").as_bytes());
        let suffix = digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!("ent_{suffix}")
    }

    pub fn list_user_entitlements(&self, user_id: &str) -> HoneResult<Vec<BillingEntitlement>> {
        let postgres = self.postgres.clone();
        let user_id = user_id.to_string();
        return run_cloud_billing(async move {
            postgres.list_billing_entitlement_records(&user_id).await
        })?
        .into_iter()
        .map(entitlement_from_value)
        .collect();
    }

    pub fn find_entitlement(
        &self,
        provider: &str,
        provider_reference_id: &str,
    ) -> HoneResult<Option<BillingEntitlement>> {
        let postgres = self.postgres.clone();
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

        let postgres = self.postgres.clone();
        let record = serde_json::to_value(&entitlement)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let changed =
            run_cloud_billing(
                async move { postgres.upsert_billing_entitlement_record(record).await },
            )?;
        if !changed {
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
        return Ok(if existing.is_some() {
            BillingEntitlementUpsertOutcome::Updated
        } else {
            BillingEntitlementUpsertOutcome::Created
        });
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

        let postgres = self.postgres.clone();
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

    pub fn webhook_event(
        &self,
        provider: &str,
        event_id: &str,
    ) -> HoneResult<Option<BillingWebhookEvent>> {
        let postgres = self.postgres.clone();
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

        let postgres = self.postgres.clone();
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

        let postgres = self.postgres.clone();
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
                event.processed_at = Some(local_now_rfc3339());
            }
            Err(error) => {
                event.processing_state = BILLING_EVENT_FAILED.to_string();
                event.last_error = Some(error.chars().take(1000).collect());
                event.processed_at = None;
            }
        }
        event.processing_started_at = None;

        let postgres = self.postgres.clone();
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

fn entitlement_from_value(value: serde_json::Value) -> HoneResult<BillingEntitlement> {
    serde_json::from_value(value).map_err(|err| HoneError::Serialization(err.to_string()))
}

fn webhook_from_value(value: serde_json::Value) -> HoneResult<BillingWebhookEvent> {
    serde_json::from_value(value).map_err(|err| HoneError::Serialization(err.to_string()))
}

fn run_cloud_billing<T, F>(future: F) -> HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = HoneResult<T>> + Send + 'static,
{
    run_cloud_sync(future, None, "cloud billing operation")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> BillingStorage {
        let root = std::env::temp_dir().join(format!("hone-billing-{}", uuid::Uuid::new_v4()));
        let storage = BillingStorage::new(&root).expect("billing");
        let postgres = storage.postgres.clone();
        run_cloud_billing(async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "INSERT INTO cloud_web_invite_users(user_id, phone_number, record) VALUES ('user_1', '', '{}'::jsonb) ON CONFLICT (user_id) DO NOTHING",
                    &[],
                )
                .await
                .map_err(|error| HoneError::Config(format!("{error:?}")))?;
            Ok(())
        })
        .expect("billing test user");
        storage
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_provider_constraints_reject_legacy_values() {
        let storage = test_storage();
        let postgres = storage.postgres.clone();
        let error = run_cloud_billing(async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    r#"
INSERT INTO billing_entitlements(
  entitlement_id, user_id, provider, entitlement_kind, provider_reference_id,
  raw_status, access_state, last_event_id, last_event_created_at, created_at, updated_at, record
) VALUES (
  'ent_legacy', 'user_1', 'legacy', 'recurring_subscription', 'legacy_1',
  'active', 'active', 'evt_legacy', '2026-08-04T00:00:00Z',
  '2026-08-04T00:00:00Z', '2026-08-04T00:00:00Z', '{}'::jsonb
)
"#,
                    &[],
                )
                .await
                .map_err(|error| HoneError::Config(format!("{error:?}")))?;
            Ok(())
        })
        .expect_err("legacy provider must violate the PostgreSQL CHECK constraint");
        assert!(
            error
                .to_string()
                .contains("billing_entitlements_provider_check")
        );
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
        let postgres = storage.postgres.clone();
        run_cloud_billing(async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "UPDATE billing_webhook_events SET processing_started_at = $1, record = jsonb_set(record, '{processing_started_at}', to_jsonb($1::text)) WHERE event_id = $2",
                    &[&"2020-08-03T01:00:00.000Z", &"evt_1"],
                )
                .await
                .map_err(|error| HoneError::Config(error.to_string()))?;
            Ok(())
        })
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
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
