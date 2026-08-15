//! EventStore — PostgreSQL 持久化与去重，附 JSONL 镜像 + append-only 推送审计。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::cloud_sync::{ensure_cloud_schema_once, run_cloud_sync};
use hone_core::config::CloudConfig;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use tokio_postgres::Row;

use crate::earnings_document::{
    EARNINGS_DOCUMENT_KEY, EARNINGS_RESEARCH_OBJECT_KEY, canonical_earnings_document_key,
    earnings_research_material_kind, earnings_research_object_key_for_event,
};
use crate::event::{EventKind, MarketEvent};

const EARNINGS_RESEARCH_LINK_WINDOW_SECS: i64 = 45 * 24 * 60 * 60;
const EARNINGS_CONTINUITY_LEASE_SECS: i64 = 15 * 60;
const DEFAULT_EVENT_STORE_TIMEOUT_SECS: u64 = 15;

pub struct EventStore {
    postgres: CloudPgRuntime,
    jsonl_path: Option<PathBuf>,
    _test_connection_lease: Option<Arc<TestConnectionLease>>,
}

struct TestConnectionLease {
    namespace: String,
    postgres: CloudPgRuntime,
}

impl Drop for TestConnectionLease {
    fn drop(&mut self) {
        let namespace = self.namespace.clone();
        let postgres = self.postgres.clone();
        std::thread::spawn(move || {
            // Sequential "restart" tests reopen the same path immediately after
            // dropping the first store. Leave a very small reuse window, then
            // evict only if no replacement lease appeared.
            std::thread::sleep(Duration::from_millis(10));
            let mut leases = TEST_CONNECTION_LEASES
                .lock()
                .expect("event store test connection lease cleanup lock");
            let still_used = leases.get(&namespace).and_then(Weak::upgrade).is_some();
            if !still_used {
                leases.remove(&namespace);
                postgres.evict_cached_test_client();
            }
        });
    }
}

static TEST_CONNECTION_LEASES: LazyLock<
    Mutex<std::collections::HashMap<String, Weak<TestConnectionLease>>>,
> = LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

#[derive(Debug, Clone, Default)]
pub struct DeliveryLogFilter {
    pub since_ts: Option<i64>,
    pub until_ts: Option<i64>,
    pub actor: Option<String>,
    pub actor_channel: Option<String>,
    pub actor_user_id: Option<String>,
    pub event_id: Option<String>,
    pub status: Option<String>,
    pub delivery_channel: Option<String>,
    pub top_level_only: bool,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub struct DeliveryLogRecord {
    pub id: i64,
    pub event_id: String,
    pub actor: String,
    pub channel: String,
    pub severity: String,
    pub sent_at_ts: i64,
    pub status: String,
    pub body: Option<String>,
    pub event_title: Option<String>,
    pub event_summary: Option<String>,
    pub event_kind: Option<String>,
    pub event_source: Option<String>,
    pub event_url: Option<String>,
    pub event_symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveredPushContextRecord {
    pub delivery_log_id: i64,
    pub source_id: String,
    pub delivered_at_ms: i64,
    pub body: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeliveredPushContextClaim {
    pub records: Vec<DeliveredPushContextRecord>,
    pub remaining_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct EarningsContinuityJob {
    pub job_key: String,
    pub actor: ActorIdentity,
    pub event: MarketEvent,
    pub attempts: u32,
}

impl EventStore {
    /// 打开生产 PostgreSQL event store。schema 在进程内只确保一次。
    pub fn new(postgres: CloudPgRuntime) -> anyhow::Result<Self> {
        ensure_cloud_schema_once(postgres.clone(), Some(event_store_operation_timeout()))?;
        let store = Self {
            postgres,
            jsonl_path: None,
            _test_connection_lease: None,
        };
        store.ensure_baseline(Utc::now())?;
        Ok(store)
    }

    /// 真实 PostgreSQL 测试构造器。
    ///
    /// 路径只作为缓存连接的隔离 namespace；表建在该连接的 `pg_temp` schema，
    /// 不会读写 SQLite，也不会污染生产 schema。同一路径的两个句柄复用连接，
    /// 用于覆盖跨 `EventStore` 句柄可见性。
    #[doc(hidden)]
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let cloud = CloudConfig::default();
        let postgres = CloudPgRuntime::from_cloud_config(&cloud).ok_or_else(|| {
            anyhow::anyhow!(
                "EventStore PostgreSQL 测试需要 HONE_POSTGRES_* 或 HONE_POSTGRES_DATABASE_URL"
            )
        })?;
        let namespace = path.as_ref().to_string_lossy().to_string();
        let postgres = postgres.with_isolated_test_connection(namespace.clone())?;
        let test_connection_lease = acquire_test_connection_lease(&namespace, postgres.clone());
        let schema_postgres = postgres.clone();
        run_cloud_sync(
            async move { schema_postgres.ensure_event_store_schema().await },
            Some(event_store_operation_timeout()),
            "event store schema operation",
        )?;
        let store = Self {
            postgres,
            jsonl_path: None,
            _test_connection_lease: Some(test_connection_lease),
        };
        store.ensure_baseline(Utc::now())?;
        Ok(store)
    }

    pub fn with_jsonl_path(mut self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        self.jsonl_path = Some(path);
        self
    }

    fn run<T, F, Fut>(&self, operation: F) -> anyhow::Result<T>
    where
        T: Send + 'static,
        F: FnOnce(CloudPgRuntime) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HoneResult<T>> + Send + 'static,
    {
        let postgres = self.postgres.clone();
        run_cloud_sync(
            async move { operation(postgres).await },
            Some(event_store_operation_timeout()),
            "event store operation",
        )
        .map_err(anyhow::Error::from)
    }

    fn ensure_baseline(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let now = now.timestamp().to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "INSERT INTO engine_meta(key, value) VALUES ('baseline_at_ts', $1) ON CONFLICT (key) DO NOTHING",
                    &[&now],
                )
                .await
                .map_err(|error| pg_store_error("initialize baseline", error))?;
            Ok(())
        })
    }

    pub fn baseline_at(&self) -> anyhow::Result<DateTime<Utc>> {
        let baseline_ts = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_opt(
                    "SELECT value::bigint FROM engine_meta WHERE key='baseline_at_ts'",
                    &[],
                )
                .await
                .map_err(|error| pg_store_error("load baseline", error))?;
            Ok(row.map(|row| row.get::<_, i64>(0)))
        })?;
        let baseline_ts = baseline_ts.ok_or_else(|| anyhow::anyhow!("baseline 未初始化"))?;
        Utc.timestamp_opt(baseline_ts, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("baseline 时间戳无效: {baseline_ts}"))
    }

    /// 同 id 只写入一次；冲突时返回 `false`。
    pub fn insert_event(&self, event: &MarketEvent) -> anyhow::Result<bool> {
        let id = event.id.clone();
        let kind_json = serde_json::to_string(&event.kind)?;
        let severity = severity_tag(&event.severity).to_string();
        let symbols_json = serde_json::to_string(&event.symbols)?;
        let occurred_at_ts = event.occurred_at.timestamp();
        let title = event.title.clone();
        let summary = event.summary.clone();
        let url = event.url.clone();
        let source = event.source.clone();
        let payload_json = serde_json::to_string(&event.payload)?;
        let created_at_ts = Utc::now().timestamp();
        let is_new = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let affected = client
                .execute(
                    r#"
INSERT INTO events (
  id, kind_json, severity, symbols_json, occurred_at_ts,
  title, summary, url, source, payload_json, created_at_ts
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
ON CONFLICT (id) DO NOTHING
"#,
                    &[
                        &id,
                        &kind_json,
                        &severity,
                        &symbols_json,
                        &occurred_at_ts,
                        &title,
                        &summary,
                        &url,
                        &source,
                        &payload_json,
                        &created_at_ts,
                    ],
                )
                .await
                .map_err(|error| pg_store_error("insert event", error))?;
            Ok(affected > 0)
        })?;
        if is_new
            && earnings_research_material_kind(event) == Some("earnings_release")
            && let Some(research_object_key) = earnings_research_object_key_for_event(event)
            && let Err(error) =
                self.backfill_earnings_research_materials(event, &research_object_key)
        {
            tracing::warn!(
                event_id = %event.id,
                research_object_key = %research_object_key,
                "earnings research material backfill failed: {error:#}"
            );
        }
        if is_new && let Err(error) = self.append_jsonl_mirror(event) {
            tracing::warn!(
                event_id = %event.id,
                source = %event.source,
                symbols = ?event.symbols,
                "events jsonl mirror append failed: {error:#}"
            );
        }
        Ok(is_new)
    }

    pub(crate) fn link_earnings_research_object(
        &self,
        event: &mut MarketEvent,
    ) -> anyhow::Result<Option<String>> {
        let Some(material_kind) = earnings_research_material_kind(event) else {
            return Ok(None);
        };
        if let Some(existing) = event
            .payload
            .get(EARNINGS_RESEARCH_OBJECT_KEY)
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(Some(existing.to_string()));
        }
        let research_object_key = if material_kind == "earnings_release" {
            earnings_research_object_key_for_event(event)
        } else {
            let Some(symbol) = event
                .symbols
                .first()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
            else {
                return Ok(None);
            };
            self.nearest_earnings_research_object_key(symbol, event.occurred_at.timestamp())?
        };
        if let Some(key) = research_object_key.as_deref() {
            ensure_payload_object(&mut event.payload).insert(
                EARNINGS_RESEARCH_OBJECT_KEY.to_string(),
                serde_json::Value::String(key.to_string()),
            );
        }
        Ok(research_object_key)
    }

    pub(crate) fn enqueue_earnings_continuity_job(
        &self,
        actor: &ActorIdentity,
        event: &MarketEvent,
    ) -> anyhow::Result<Option<String>> {
        let research_object_key = match earnings_research_object_key_for_event(event) {
            Some(key) => key,
            None if earnings_research_material_kind(event) == Some("earnings_release") => {
                event.id.clone()
            }
            None => return Ok(None),
        };
        let job_key = format!(
            "{}::{}{}",
            delivery_actor_key(actor),
            research_object_key.trim(),
            if matches!(event.kind, EventKind::EarningsCallTranscript) {
                "::earnings_transcript"
            } else {
                ""
            }
        );
        let actor_json = serde_json::to_string(actor)?;
        let event_json = serde_json::to_string(event)?;
        let now = Utc::now().timestamp();
        let returned_key = job_key.clone();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    r#"
INSERT INTO earnings_continuity_jobs (
  job_key, actor_json, event_json, status, attempts,
  next_attempt_ts, lease_until_ts, last_error, created_at_ts, updated_at_ts
) VALUES ($1, $2, $3, 'pending', 0, $4, NULL, NULL, $4, $4)
ON CONFLICT (job_key) DO NOTHING
"#,
                    &[&job_key, &actor_json, &event_json, &now],
                )
                .await
                .map_err(|error| pg_store_error("enqueue earnings continuity job", error))?;
            Ok(())
        })?;
        Ok(Some(returned_key))
    }

    pub(crate) fn claim_due_earnings_continuity_jobs(
        &self,
        now: DateTime<Utc>,
        limit: usize,
    ) -> anyhow::Result<Vec<EarningsContinuityJob>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let now_ts = now.timestamp();
        let lease_until_ts = now_ts + EARNINGS_CONTINUITY_LEASE_SECS;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
WITH due AS (
  SELECT job_key
  FROM earnings_continuity_jobs
  WHERE (
    status IN ('pending', 'retry') AND next_attempt_ts <= $1
  ) OR (
    status = 'running' AND (lease_until_ts IS NULL OR lease_until_ts <= $1)
  )
  ORDER BY next_attempt_ts ASC, created_at_ts ASC
  LIMIT $2
  FOR UPDATE
), claimed AS (
  UPDATE earnings_continuity_jobs AS jobs
  SET status = 'running',
      attempts = jobs.attempts + 1,
      lease_until_ts = $3,
      updated_at_ts = $1,
      last_error = NULL
  FROM due
  WHERE jobs.job_key = due.job_key
  RETURNING jobs.job_key, jobs.actor_json, jobs.event_json, jobs.attempts
)
SELECT job_key, actor_json, event_json, attempts
FROM claimed
ORDER BY job_key
"#,
                    &[&now_ts, &limit, &lease_until_ts],
                )
                .await
                .map_err(|error| pg_store_error("claim earnings continuity jobs", error))
        })?;
        let mut jobs = Vec::with_capacity(rows.len());
        for row in rows {
            let job_key: String = row.get(0);
            let actor_json: String = row.get(1);
            let event_json: String = row.get(2);
            let attempts_i32: i32 = row.get(3);
            let actor = match serde_json::from_str::<ActorIdentity>(&actor_json) {
                Ok(actor) => actor,
                Err(error) => {
                    self.mark_earnings_continuity_job_dead(
                        &job_key,
                        &format!("invalid actor_json: {error}"),
                        now_ts,
                    )?;
                    continue;
                }
            };
            let event = match serde_json::from_str::<MarketEvent>(&event_json) {
                Ok(event) => event,
                Err(error) => {
                    self.mark_earnings_continuity_job_dead(
                        &job_key,
                        &format!("invalid event_json: {error}"),
                        now_ts,
                    )?;
                    continue;
                }
            };
            jobs.push(EarningsContinuityJob {
                job_key,
                actor,
                event,
                attempts: u32::try_from(attempts_i32).unwrap_or(u32::MAX),
            });
        }
        Ok(jobs)
    }

    fn mark_earnings_continuity_job_dead(
        &self,
        job_key: &str,
        error: &str,
        now_ts: i64,
    ) -> anyhow::Result<()> {
        let job_key = job_key.to_string();
        let error = truncate_store_error(error);
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "UPDATE earnings_continuity_jobs SET status='dead', last_error=$2, lease_until_ts=NULL, updated_at_ts=$3 WHERE job_key=$1",
                    &[&job_key, &error, &now_ts],
                )
                .await
                .map_err(|error| pg_store_error("mark earnings continuity job dead", error))?;
            Ok(())
        })
    }

    pub(crate) fn complete_earnings_continuity_job(
        &self,
        job_key: &str,
        attempts: u32,
    ) -> anyhow::Result<bool> {
        let job_key = job_key.to_string();
        let attempts = i32::try_from(attempts).unwrap_or(i32::MAX);
        let now = Utc::now().timestamp();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let affected = client
                .execute(
                    r#"
UPDATE earnings_continuity_jobs
SET status='completed', lease_until_ts=NULL, last_error=NULL, updated_at_ts=$2
WHERE job_key=$1 AND status='running' AND attempts=$3
"#,
                    &[&job_key, &now, &attempts],
                )
                .await
                .map_err(|error| pg_store_error("complete earnings continuity job", error))?;
            Ok(affected > 0)
        })
    }

    pub(crate) fn retry_earnings_continuity_job(
        &self,
        job_key: &str,
        attempts: u32,
        error: &str,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let exponent = attempts.saturating_sub(1).min(8);
        let delay_secs = 60_i64.saturating_mul(1_i64 << exponent).min(6 * 60 * 60);
        let job_key = job_key.to_string();
        let attempts = i32::try_from(attempts).unwrap_or(i32::MAX);
        let error = truncate_store_error(error);
        let now_ts = now.timestamp();
        let next_attempt_ts = now_ts + delay_secs;
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let affected = client
                .execute(
                    r#"
UPDATE earnings_continuity_jobs
SET status='retry', next_attempt_ts=$2, lease_until_ts=NULL,
    last_error=$3, updated_at_ts=$4
WHERE job_key=$1 AND status='running' AND attempts=$5
"#,
                    &[&job_key, &next_attempt_ts, &error, &now_ts, &attempts],
                )
                .await
                .map_err(|error| pg_store_error("retry earnings continuity job", error))?;
            Ok(affected > 0)
        })
    }

    #[cfg(test)]
    fn earnings_continuity_job_status(&self, job_key: &str) -> anyhow::Result<Option<String>> {
        let job_key = job_key.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_opt(
                    "SELECT status FROM earnings_continuity_jobs WHERE job_key=$1",
                    &[&job_key],
                )
                .await
                .map_err(|error| pg_store_error("load earnings continuity job status", error))?;
            Ok(row.map(|row| row.get(0)))
        })
    }

    fn nearest_earnings_research_object_key(
        &self,
        symbol: &str,
        occurred_at_ts: i64,
    ) -> anyhow::Result<Option<String>> {
        let symbol = symbol.to_string();
        let query_symbol = symbol.clone();
        let start = occurred_at_ts - EARNINGS_RESEARCH_LINK_WINDOW_SECS;
        let end = occurred_at_ts + EARNINGS_RESEARCH_LINK_WINDOW_SECS;
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT kind_json, symbols_json, payload_json
FROM events
WHERE occurred_at_ts BETWEEN $1 AND $2
  AND EXISTS (
    SELECT 1 FROM jsonb_array_elements_text(symbols_json::jsonb) AS item(value)
    WHERE lower(item.value) = lower($4)
  )
ORDER BY abs(occurred_at_ts - $3) ASC
LIMIT 200
"#,
                    &[&start, &end, &occurred_at_ts, &query_symbol],
                )
                .await
                .map_err(|error| pg_store_error("find nearest earnings research object", error))
        })?;
        for row in rows {
            let kind_json: String = row.get(0);
            let symbols_json: String = row.get(1);
            let payload_json: String = row.get(2);
            let Ok(EventKind::EarningsReleased) = serde_json::from_str(&kind_json) else {
                continue;
            };
            let symbols: Vec<String> = serde_json::from_str(&symbols_json).unwrap_or_default();
            if !symbols
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&symbol))
            {
                continue;
            }
            let payload: serde_json::Value =
                serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);
            if payload
                .get("earnings_quality_review_applied")
                .and_then(|value| value.as_bool())
                != Some(true)
            {
                continue;
            }
            if let Some(key) = payload
                .get(EARNINGS_RESEARCH_OBJECT_KEY)
                .or_else(|| payload.get(EARNINGS_DOCUMENT_KEY))
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return Ok(Some(key.to_string()));
            }
        }
        Ok(None)
    }

    fn backfill_earnings_research_materials(
        &self,
        release: &MarketEvent,
        research_object_key: &str,
    ) -> anyhow::Result<usize> {
        let Some(symbol) = release
            .symbols
            .first()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            return Ok(0);
        };
        let symbol = symbol.to_string();
        let release_id = release.id.clone();
        let occurred_at_ts = release.occurred_at.timestamp();
        let start = occurred_at_ts - EARNINGS_RESEARCH_LINK_WINDOW_SECS;
        let end = occurred_at_ts + EARNINGS_RESEARCH_LINK_WINDOW_SECS;
        let research_object_key = research_object_key.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let updated = client
                .execute(
                    r#"
UPDATE events
SET payload_json = jsonb_set(
      CASE
        WHEN jsonb_typeof(payload_json::jsonb) = 'object' THEN payload_json::jsonb
        ELSE '{}'::jsonb
      END,
      ARRAY['hone_earnings_research_object_key'],
      to_jsonb($6::text),
      true
    )::text
WHERE occurred_at_ts BETWEEN $1 AND $2
  AND id <> $3
  AND EXISTS (
    SELECT 1 FROM jsonb_array_elements_text(symbols_json::jsonb) AS item(value)
    WHERE lower(item.value) = lower($5)
  )
  AND abs(occurred_at_ts - $4) <= $7
  AND NOT (payload_json::jsonb ? 'hone_earnings_research_object_key')
  AND (
    kind_json LIKE '%"earnings_call_transcript"%'
    OR (
      kind_json LIKE '%"sec_filing"%'
      AND (
        lower(payload_json::jsonb ->> 'form') IN ('10-q', '10-k')
        OR lower(kind_json) LIKE '%10-q%'
        OR lower(kind_json) LIKE '%10-k%'
      )
    )
  )
"#,
                    &[
                        &start,
                        &end,
                        &release_id,
                        &occurred_at_ts,
                        &symbol,
                        &research_object_key,
                        &EARNINGS_RESEARCH_LINK_WINDOW_SECS,
                    ],
                )
                .await
                .map_err(|error| pg_store_error("backfill earnings research materials", error))?;
            Ok(usize::try_from(updated).unwrap_or(usize::MAX))
        })
    }

    pub(crate) fn list_earnings_research_materials(
        &self,
        research_object_key: &str,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let research_object_key = research_object_key.to_string();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT id, kind_json, severity, symbols_json, occurred_at_ts,
       title, summary, url, source, payload_json
FROM events
WHERE payload_json::jsonb ->> 'hone_earnings_research_object_key' = $1
ORDER BY occurred_at_ts ASC, id ASC
"#,
                    &[&research_object_key],
                )
                .await
                .map_err(|error| pg_store_error("list earnings research materials", error))
        })?;
        let mut materials = rows
            .iter()
            .map(decode_market_event)
            .collect::<HoneResult<Vec<_>>>()?;
        materials.retain(|event| {
            matches!(
                earnings_research_material_kind(event),
                Some("earnings_call_transcript" | "formal_filing")
            )
        });
        Ok(materials)
    }

    #[cfg(test)]
    fn event_research_object_key(&self, event_id: &str) -> anyhow::Result<Option<String>> {
        let event_id = event_id.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_opt("SELECT payload_json FROM events WHERE id=$1", &[&event_id])
                .await
                .map_err(|error| pg_store_error("load event research object key", error))?;
            Ok(row
                .and_then(|row| {
                    serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(0)).ok()
                })
                .and_then(|payload| {
                    payload
                        .get(EARNINGS_RESEARCH_OBJECT_KEY)
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                }))
        })
    }

    #[cfg(test)]
    fn test_delivery_attempt_summary(
        &self,
        event_id: &str,
        actor: &str,
    ) -> anyhow::Result<(i64, String)> {
        let event_id = event_id.to_string();
        let actor = actor.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT count(*)::bigint,
       (array_agg(status ORDER BY sent_at_ts DESC, id DESC))[1]
FROM delivery_log WHERE event_id=$1 AND actor=$2
"#,
                    &[&event_id, &actor],
                )
                .await
                .map_err(|error| pg_store_error("summarize delivery attempts", error))?;
            Ok((row.get(0), row.get(1)))
        })
    }

    #[cfg(test)]
    fn test_set_delivered_at_ms(&self, source_id: &str, value: i64) -> anyhow::Result<()> {
        let source_id = source_id.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "UPDATE delivered_push_context SET delivered_at_ms=$1 WHERE source_id=$2",
                    &[&value, &source_id],
                )
                .await
                .map_err(|error| pg_store_error("set delivered push timestamp", error))?;
            Ok(())
        })
    }

    #[cfg(test)]
    fn test_insert_historical_delivery(&self) -> anyhow::Result<()> {
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    r#"
INSERT INTO delivery_log(event_id, actor, channel, severity, sent_at_ts, status, body)
VALUES ('historical', 'discord::::u1', 'sink', 'high', 1, 'sent', 'OLD PUSH')
"#,
                    &[],
                )
                .await
                .map_err(|error| pg_store_error("insert historical delivery", error))?;
            Ok(())
        })
    }

    #[cfg(test)]
    fn test_set_event_created_at(&self, event_id: &str, value: i64) -> anyhow::Result<()> {
        let event_id = event_id.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    "UPDATE events SET created_at_ts=$1 WHERE id=$2",
                    &[&value, &event_id],
                )
                .await
                .map_err(|error| pg_store_error("set event created_at", error))?;
            Ok(())
        })
    }

    fn append_jsonl_mirror(&self, event: &MarketEvent) -> anyhow::Result<()> {
        let Some(path) = self.jsonl_path.as_ref() else {
            return Ok(());
        };
        use std::io::Write;
        let line = serde_json::to_string(event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn purge_events_older_than(&self, cutoff_days: i64) -> anyhow::Result<usize> {
        let cutoff = Utc::now().timestamp() - cutoff_days * 86_400;
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let count = client
                .execute("DELETE FROM events WHERE created_at_ts < $1", &[&cutoff])
                .await
                .map_err(|error| pg_store_error("purge events", error))?;
            Ok(usize::try_from(count).unwrap_or(usize::MAX))
        })
    }

    pub fn purge_delivery_log_older_than(&self, cutoff_days: i64) -> anyhow::Result<usize> {
        let cutoff = Utc::now().timestamp() - cutoff_days * 86_400;
        let cutoff_ms = cutoff.saturating_mul(1000);
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let rows = client
                .query(
                    r#"
WITH removed_context AS (
  DELETE FROM delivered_push_context WHERE delivered_at_ms < $1
)
DELETE FROM delivery_log WHERE sent_at_ts < $2
RETURNING id
"#,
                    &[&cutoff_ms, &cutoff],
                )
                .await;
            match rows {
                Ok(rows) => Ok(rows.len()),
                Err(error) => Err(pg_store_error("purge delivery log", error)),
            }
        })
    }

    pub fn count_events(&self) -> anyhow::Result<i64> {
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one("SELECT count(*)::bigint FROM events", &[])
                .await
                .map_err(|error| pg_store_error("count events", error))?;
            Ok(row.get(0))
        })
    }

    pub fn contains_event(&self, event_id: &str) -> anyhow::Result<bool> {
        let event_id = event_id.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM events WHERE id=$1)",
                    &[&event_id],
                )
                .await
                .map_err(|error| pg_store_error("contains event", error))?;
            Ok(row.get(0))
        })
    }

    pub(crate) fn actor_has_delivered_earnings_for_document(
        &self,
        actor: &str,
        document_url: &str,
    ) -> anyhow::Result<bool> {
        let Some(document_key) = canonical_earnings_document_key(document_url) else {
            return Ok(false);
        };
        let actor = actor.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT EXISTS(
  SELECT 1
  FROM delivery_log d
  JOIN events e ON e.id = d.event_id
  WHERE d.actor = $1
    AND d.status IN ('sent', 'dryrun')
    AND d.channel IN ('sink', 'digest_item')
    AND e.kind_json LIKE '%"earnings_released"%'
    AND (e.payload_json::jsonb ->> 'earnings_quality_review_applied')::boolean IS TRUE
    AND (
      lower(rtrim(split_part(e.url, '?', 1), '/')) = $2
      OR lower(e.payload_json::jsonb ->> $3) = $2
    )
)
"#,
                    &[&actor, &document_key, &EARNINGS_DOCUMENT_KEY],
                )
                .await
                .map_err(|error| pg_store_error("check delivered earnings document", error))?;
            Ok(row.get(0))
        })
    }

    pub fn symbol_signal_kinds_in_window(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let start = start.timestamp();
        let end = end.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    "SELECT kind_json FROM events WHERE occurred_at_ts >= $1 AND occurred_at_ts <= $2 AND symbols_json ILIKE $3",
                    &[&start, &end, &needle],
                )
                .await
                .map_err(|error| pg_store_error("list symbol signal kinds", error))
        })?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                serde_json::from_str::<serde_json::Value>(&row.get::<_, String>(0))
                    .ok()
                    .and_then(|value| value.get("type")?.as_str().map(str::to_string))
            })
            .collect())
    }

    pub fn list_analyst_grade_payloads_in_window(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let start = start.timestamp();
        let end = end.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT payload_json FROM events
WHERE occurred_at_ts >= $1 AND occurred_at_ts <= $2
  AND symbols_json ILIKE $3
  AND kind_json LIKE '%analyst_grade%'
"#,
                    &[&start, &end, &needle],
                )
                .await
                .map_err(|error| pg_store_error("list analyst grade payloads", error))
        })?;
        Ok(rows
            .into_iter()
            .filter_map(|row| serde_json::from_str(&row.get::<_, String>(0)).ok())
            .collect())
    }

    pub fn today_signal_kinds(
        &self,
        symbol: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        self.symbol_signal_kinds_in_window(symbol, since, Utc::now())
    }

    pub fn list_upcoming_earnings(
        &self,
        now: DateTime<Utc>,
        within_days: i64,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let start = now.timestamp();
        let end = (now + chrono::Duration::days(within_days)).timestamp();
        self.query_events(
            r#"
SELECT id, kind_json, severity, symbols_json, occurred_at_ts,
       title, summary, url, source, payload_json
FROM events
WHERE occurred_at_ts >= $1 AND occurred_at_ts <= $2
  AND kind_json LIKE '%"earnings_upcoming"%'
"#,
            start,
            end,
            "list upcoming earnings",
        )
    }

    pub fn next_upcoming_earnings_for_symbol(
        &self,
        symbol: &str,
        now: DateTime<Utc>,
        within_days: i64,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let start = now.timestamp();
        let end = (now + chrono::Duration::days(within_days)).timestamp();
        let timestamp = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_opt(
                    r#"
SELECT occurred_at_ts FROM events
WHERE occurred_at_ts >= $1 AND occurred_at_ts <= $2
  AND kind_json LIKE '%"earnings_upcoming"%'
  AND symbols_json ILIKE $3
ORDER BY occurred_at_ts ASC LIMIT 1
"#,
                    &[&start, &end, &needle],
                )
                .await
                .map_err(|error| pg_store_error("find next upcoming earnings", error))?;
            Ok(row.map(|row| row.get::<_, i64>(0)))
        })?;
        Ok(timestamp.and_then(|value| DateTime::<Utc>::from_timestamp(value, 0)))
    }

    pub fn count_event_ids_in_window(
        &self,
        id_prefix: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<i64> {
        let escaped = id_prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("{escaped}%");
        let start = start.timestamp();
        let end = end.timestamp();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT count(*)::bigint FROM events
WHERE occurred_at_ts >= $1 AND occurred_at_ts <= $2
  AND id LIKE $3 ESCAPE '\'
"#,
                    &[&start, &end, &pattern],
                )
                .await
                .map_err(|error| pg_store_error("count event ids in window", error))?;
            Ok(row.get(0))
        })
    }

    pub fn count_high_sent_since(&self, actor: &str, since: DateTime<Utc>) -> anyhow::Result<i64> {
        self.count_high_sent_since_for_category(actor, since, "all")
    }

    pub fn count_high_sent_since_for_category(
        &self,
        actor: &str,
        since: DateTime<Utc>,
        category: &str,
    ) -> anyhow::Result<i64> {
        if category == "all" {
            return self.count_high_sent_since_all(actor, since);
        }
        let Some(tags) = category_kind_tags(category) else {
            return self.count_high_sent_since_all(actor, since);
        };
        let actor = actor.to_string();
        let since = since.timestamp();
        let patterns = tags
            .iter()
            .map(|tag| format!("%\"{tag}\"%"))
            .collect::<Vec<_>>();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT count(*)::bigint FROM delivery_log d
JOIN events e ON d.event_id = e.id
WHERE d.actor = $1
  AND d.severity = 'high'
  AND d.status = 'sent'
  AND d.channel = 'sink'
  AND d.sent_at_ts >= $2
  AND e.kind_json LIKE ANY($3::text[])
"#,
                    &[&actor, &since, &patterns],
                )
                .await
                .map_err(|error| pg_store_error("count high sends by category", error))?;
            Ok(row.get(0))
        })
    }

    fn count_high_sent_since_all(&self, actor: &str, since: DateTime<Utc>) -> anyhow::Result<i64> {
        let actor = actor.to_string();
        let since = since.timestamp();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT count(*)::bigint FROM delivery_log
WHERE actor=$1 AND severity='high' AND status='sent'
  AND channel='sink' AND sent_at_ts >= $2
"#,
                    &[&actor, &since],
                )
                .await
                .map_err(|error| pg_store_error("count high sends", error))?;
            Ok(row.get(0))
        })
    }

    pub fn last_high_sink_send_for_symbol(
        &self,
        actor: &str,
        symbol: &str,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        self.last_high_sink_send_for_symbol_category(actor, symbol, "all", None)
    }

    pub fn last_high_sink_send_for_symbol_category(
        &self,
        actor: &str,
        symbol: &str,
        category: &str,
        firm: Option<&str>,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        if category == "all" {
            return self.last_high_sink_send_for_symbol_all(actor, symbol);
        }
        let Some(tags) = category_kind_tags(category) else {
            return self.last_high_sink_send_for_symbol_all(actor, symbol);
        };
        let actor = actor.to_string();
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let patterns = tags
            .iter()
            .map(|tag| format!("%\"{tag}\"%"))
            .collect::<Vec<_>>();
        let firm = firm.map(str::to_string);
        let timestamp = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT max(d.sent_at_ts) FROM delivery_log d
JOIN events e ON d.event_id = e.id
WHERE d.actor=$1 AND d.severity='high' AND d.status='sent' AND d.channel='sink'
  AND e.symbols_json ILIKE $2
  AND e.kind_json LIKE ANY($3::text[])
  AND ($4::text IS NULL OR e.payload_json::jsonb ->> 'gradingCompany' = $4)
"#,
                    &[&actor, &needle, &patterns, &firm],
                )
                .await
                .map_err(|error| pg_store_error("load last high send by category", error))?;
            Ok(row.get::<_, Option<i64>>(0))
        })?;
        Ok(timestamp.and_then(|value| DateTime::<Utc>::from_timestamp(value, 0)))
    }

    fn last_high_sink_send_for_symbol_all(
        &self,
        actor: &str,
        symbol: &str,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let actor = actor.to_string();
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let timestamp = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT max(d.sent_at_ts) FROM delivery_log d
JOIN events e ON d.event_id = e.id
WHERE d.actor=$1 AND d.severity='high' AND d.status='sent'
  AND d.channel='sink' AND e.symbols_json ILIKE $2
"#,
                    &[&actor, &needle],
                )
                .await
                .map_err(|error| pg_store_error("load last high send", error))?;
            Ok(row.get::<_, Option<i64>>(0))
        })?;
        Ok(timestamp.and_then(|value| DateTime::<Utc>::from_timestamp(value, 0)))
    }

    pub fn last_high_sink_send_for_analyst_news_url(
        &self,
        actor: &str,
        symbol: &str,
        news_url: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let news_url = news_url.trim();
        if news_url.is_empty() {
            return Ok(None);
        }
        let actor = actor.to_string();
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let news_url = news_url.to_string();
        let since = since.timestamp();
        let timestamp = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
SELECT max(d.sent_at_ts) FROM delivery_log d
JOIN events e ON d.event_id = e.id
WHERE d.actor=$1 AND d.severity='high' AND d.status='sent' AND d.channel='sink'
  AND d.sent_at_ts >= $2 AND e.symbols_json ILIKE $3
  AND e.kind_json LIKE '%"analyst_grade"%'
  AND (e.payload_json::jsonb ->> 'newsURL' = $4 OR e.url = $4)
"#,
                    &[&actor, &since, &needle, &news_url],
                )
                .await
                .map_err(|error| pg_store_error("load analyst article send", error))?;
            Ok(row.get::<_, Option<i64>>(0))
        })?;
        Ok(timestamp.and_then(|value| DateTime::<Utc>::from_timestamp(value, 0)))
    }

    pub fn last_price_band_max_bps_for_symbol_direction(
        &self,
        actor: &str,
        symbol: &str,
        direction: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Option<i64>> {
        let Some(pattern) = price_band_id_pattern(symbol, direction) else {
            return Ok(None);
        };
        let actor = actor.to_string();
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let since = since.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT e.id FROM delivery_log d
JOIN events e ON d.event_id=e.id
WHERE d.actor=$1 AND d.severity='high' AND d.status='sent' AND d.channel='sink'
  AND d.sent_at_ts >= $2 AND e.symbols_json ILIKE $3 AND e.id LIKE $4
"#,
                    &[&actor, &since, &needle, &pattern],
                )
                .await
                .map_err(|error| pg_store_error("list delivered price bands", error))
        })?;
        Ok(rows
            .iter()
            .filter_map(|row| parse_bps_from_band_id(&row.get::<_, String>(0)))
            .max())
    }

    pub fn last_digest_success_at(&self, actor: &str) -> anyhow::Result<Option<DateTime<Utc>>> {
        let actor = actor.to_string();
        let timestamp = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    "SELECT max(sent_at_ts) FROM delivery_log WHERE actor=$1 AND channel='digest' AND status IN ('sent','dryrun')",
                    &[&actor],
                )
                .await
                .map_err(|error| pg_store_error("load last digest success", error))?;
            Ok(row.get::<_, Option<i64>>(0))
        })?;
        Ok(timestamp.and_then(|value| DateTime::<Utc>::from_timestamp(value, 0)))
    }

    pub fn list_missed_digest_items_since(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<(MarketEvent, String)>> {
        let actor = actor.to_string();
        let since = since.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT e.id, e.kind_json, e.severity, e.symbols_json, e.occurred_at_ts,
       e.title, e.summary, e.url, e.source, e.payload_json, d.status
FROM delivery_log d JOIN events e ON d.event_id=e.id
WHERE d.actor=$1 AND d.channel IN ('digest_item','prefs')
  AND d.status NOT IN ('sent','dryrun','queued') AND d.sent_at_ts >= $2
ORDER BY d.sent_at_ts DESC
"#,
                    &[&actor, &since],
                )
                .await
                .map_err(|error| pg_store_error("list missed digest items", error))
        })?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                decode_market_event(row)
                    .ok()
                    .map(|event| (event, row.get(10)))
            })
            .collect())
    }

    pub fn delivered_event_ids_since(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<HashSet<String>> {
        self.event_ids_since(
            "SELECT DISTINCT event_id FROM delivery_log WHERE actor=$1 AND status IN ('sent','dryrun') AND sent_at_ts >= $2",
            actor,
            since,
            "list delivered event ids",
        )
    }

    pub fn list_actors_with_quiet_held_since(
        &self,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        let since = since.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    "SELECT DISTINCT actor FROM delivery_log WHERE channel='sink' AND status='quiet_held' AND sent_at_ts >= $1",
                    &[&since],
                )
                .await
                .map_err(|error| pg_store_error("list quiet-held actors", error))
        })?;
        Ok(rows.into_iter().map(|row| row.get(0)).collect())
    }

    pub fn list_quiet_held_since(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<(MarketEvent, i64)>> {
        let actor = actor.to_string();
        let since = since.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT e.id, e.kind_json, e.severity, e.symbols_json, e.occurred_at_ts,
       e.title, e.summary, e.url, e.source, e.payload_json, d.sent_at_ts
FROM delivery_log d JOIN events e ON d.event_id=e.id
WHERE d.actor=$1 AND d.channel='sink' AND d.status='quiet_held' AND d.sent_at_ts >= $2
ORDER BY d.sent_at_ts ASC
"#,
                    &[&actor, &since],
                )
                .await
                .map_err(|error| pg_store_error("list quiet-held events", error))
        })?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                decode_market_event(row)
                    .ok()
                    .map(|event| (event, row.get(10)))
            })
            .collect())
    }

    pub fn list_recent_digest_item_events(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let actor = actor.to_string();
        let since = since.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT e.id, e.kind_json, e.severity, e.symbols_json, e.occurred_at_ts,
       e.title, e.summary, e.url, e.source, e.payload_json
FROM delivery_log d JOIN events e ON d.event_id=e.id
WHERE d.actor=$1 AND d.channel='digest_item'
  AND d.status IN ('sent','dryrun') AND d.sent_at_ts >= $2
"#,
                    &[&actor, &since],
                )
                .await
                .map_err(|error| pg_store_error("list recent digest item events", error))
        })?;
        Ok(rows
            .iter()
            .filter_map(|row| decode_market_event(row).ok())
            .collect())
    }

    pub fn list_global_digest_news_candidates(
        &self,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let since = since.timestamp();
        let until = until.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(
                    r#"
SELECT id, kind_json, severity, symbols_json, occurred_at_ts,
       title, summary, url, source, payload_json
FROM events
WHERE occurred_at_ts >= $1 AND occurred_at_ts < $2
  AND kind_json LIKE '%news_critical%'
  AND (
    source LIKE 'rss:%'
    OR (source LIKE 'fmp.stock_news:%' AND severity IN ('high','medium'))
    OR (source LIKE 'fmp.stock_news:%' AND payload_json::jsonb ->> 'source_class' = 'trusted')
  )
ORDER BY occurred_at_ts DESC
"#,
                    &[&since, &until],
                )
                .await
                .map_err(|error| pg_store_error("list global digest news candidates", error))
        })?;
        Ok(rows
            .iter()
            .filter_map(|row| decode_market_event(row).ok())
            .collect())
    }

    pub fn broadcasted_event_ids_since(
        &self,
        channel: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<HashSet<String>> {
        self.event_ids_since(
            "SELECT DISTINCT event_id FROM delivery_log WHERE channel=$1 AND status IN ('sent','dryrun') AND sent_at_ts >= $2",
            channel,
            since,
            "list broadcast event ids",
        )
    }

    pub fn log_delivery(
        &self,
        event_id: &str,
        actor: &str,
        channel: &str,
        severity: crate::event::Severity,
        status: &str,
        body: Option<&str>,
    ) -> anyhow::Result<()> {
        let event_id = event_id.to_string();
        let actor = actor.to_string();
        let channel = channel.to_string();
        let severity = severity_tag(&severity).to_string();
        let status = status.to_string();
        let body = body.map(str::to_string);
        let sent_at_ts = Utc::now().timestamp();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    r#"
INSERT INTO delivery_log(event_id, actor, channel, severity, sent_at_ts, status, body)
VALUES ($1, $2, $3, $4, $5, $6, $7)
"#,
                    &[
                        &event_id,
                        &actor,
                        &channel,
                        &severity,
                        &sent_at_ts,
                        &status,
                        &body,
                    ],
                )
                .await
                .map_err(|error| pg_store_error("append delivery log", error))?;
            Ok(())
        })
    }

    pub fn log_confirmed_delivery(
        &self,
        event_id: &str,
        actor: &ActorIdentity,
        channel: &str,
        severity: crate::event::Severity,
        body: &str,
        observed_native_session_id: Option<&str>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(!body.trim().is_empty(), "确认送达的正文不能为空");
        let event_id = event_id.to_string();
        let actor = delivery_actor_key(actor);
        let channel = channel.to_string();
        let severity = severity_tag(&severity).to_string();
        let body = body.to_string();
        let observed_native_session_id = observed_native_session_id.map(str::to_string);
        let now = Utc::now();
        let sent_at_ts = now.timestamp();
        let delivered_at_ms = now.timestamp_millis();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .execute(
                    r#"
WITH logged AS (
  INSERT INTO delivery_log(event_id, actor, channel, severity, sent_at_ts, status, body)
  VALUES ($1, $2, $3, $4, $5, 'sent', $6)
  RETURNING id
)
INSERT INTO delivered_push_context(
  delivery_log_id, actor, source_id, delivered_at_ms, body,
  observed_native_session_id
)
SELECT id, $2, $1, $7, $6, $8 FROM logged
ON CONFLICT DO NOTHING
"#,
                    &[
                        &event_id,
                        &actor,
                        &channel,
                        &severity,
                        &sent_at_ts,
                        &body,
                        &delivered_at_ms,
                        &observed_native_session_id,
                    ],
                )
                .await
                .map_err(|error| pg_store_error("record confirmed delivery", error))?;
            Ok(())
        })
    }

    pub fn claim_delivered_push_context(
        &self,
        actor: &ActorIdentity,
        turn_id: &str,
        delivered_before_ms: i64,
        max_records: usize,
        max_body_chars: usize,
        lease_ms: i64,
    ) -> anyhow::Result<DeliveredPushContextClaim> {
        self.claim_delivered_push_context_with_native_observation(
            actor,
            turn_id,
            delivered_before_ms,
            max_records,
            max_body_chars,
            lease_ms,
            None,
        )
    }

    pub fn claim_delivered_push_context_with_native_observation(
        &self,
        actor: &ActorIdentity,
        turn_id: &str,
        delivered_before_ms: i64,
        max_records: usize,
        max_body_chars: usize,
        lease_ms: i64,
        consumer_native_session_id: Option<&str>,
    ) -> anyhow::Result<DeliveredPushContextClaim> {
        let actor = delivery_actor_key(actor);
        let turn_id = turn_id.to_string();
        let max_records = i64::try_from(max_records.clamp(1, 100)).unwrap_or(100);
        let max_body_chars = i64::try_from(max_body_chars.max(1)).unwrap_or(i64::MAX);
        let now_ms = Utc::now().timestamp_millis();
        let claim_expires_at_ms = now_ms.saturating_add(lease_ms.max(1));
        let consumer_native_session_id = consumer_native_session_id.map(str::to_string);
        let (records_json, remaining_count) = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let row = client
                .query_one(
                    r#"
WITH existing AS MATERIALIZED (
  SELECT delivery_log_id, source_id, delivered_at_ms, body
  FROM delivered_push_context
  WHERE actor=$1 AND claimed_turn_id=$2 AND consumed_at_ms IS NULL
), native_consumed AS (
  UPDATE delivered_push_context AS context
  SET consumed_turn_id=$2,
      consumed_at_ms=$6,
      claimed_turn_id=NULL,
      claim_expires_at_ms=NULL
  WHERE context.actor=$1
    AND NOT EXISTS (SELECT 1 FROM existing)
    AND $8::text IS NOT NULL
    AND context.observed_native_session_id=$8
    AND context.delivered_at_ms <= $3
    AND context.consumed_at_ms IS NULL
    AND (
      context.claimed_turn_id IS NULL
      OR context.claim_expires_at_ms <= $6
    )
  RETURNING context.delivery_log_id
), ranked AS MATERIALIZED (
  SELECT
    context.delivery_log_id,
    context.source_id,
    context.delivered_at_ms,
    context.body,
    row_number() OVER (
      ORDER BY context.delivered_at_ms, context.delivery_log_id
    ) AS row_number,
    sum(char_length(context.body)) OVER (
      ORDER BY context.delivered_at_ms, context.delivery_log_id
    ) AS cumulative_chars
  FROM delivered_push_context AS context
  WHERE context.actor=$1
    AND NOT EXISTS (SELECT 1 FROM existing)
    AND context.consumed_at_ms IS NULL
    AND context.delivered_at_ms <= $3
    AND (
      context.claimed_turn_id IS NULL
      OR context.claim_expires_at_ms <= $6
    )
    AND ($8::text IS NULL OR context.observed_native_session_id IS DISTINCT FROM $8)
), candidates AS MATERIALIZED (
  SELECT delivery_log_id
  FROM ranked
  WHERE row_number=1 OR cumulative_chars <= $5
  ORDER BY delivered_at_ms, delivery_log_id
  LIMIT $4
), claimed AS (
  UPDATE delivered_push_context AS context
  SET claimed_turn_id=$2,
      claim_expires_at_ms=$7
  FROM candidates
  WHERE context.delivery_log_id=candidates.delivery_log_id
    AND context.consumed_at_ms IS NULL
    AND (
      context.claimed_turn_id IS NULL
      OR context.claim_expires_at_ms <= $6
    )
  RETURNING context.delivery_log_id, context.source_id,
            context.delivered_at_ms, context.body
), selected AS MATERIALIZED (
  SELECT * FROM existing
  UNION ALL
  SELECT * FROM claimed
)
SELECT
  COALESCE(
    (
      SELECT jsonb_agg(
        jsonb_build_object(
          'delivery_log_id', delivery_log_id,
          'source_id', source_id,
          'delivered_at_ms', delivered_at_ms,
          'body', body
        ) ORDER BY delivered_at_ms, delivery_log_id
      )
      FROM selected
    ),
    '[]'::jsonb
  ),
  (
    SELECT count(*)::bigint
    FROM delivered_push_context AS context
    WHERE context.actor=$1
      AND context.consumed_at_ms IS NULL
      AND context.delivered_at_ms <= $3
      AND (
        context.claimed_turn_id IS NULL
        OR context.claim_expires_at_ms <= $6
      )
      AND NOT EXISTS (
        SELECT 1 FROM selected WHERE selected.delivery_log_id=context.delivery_log_id
      )
      AND NOT EXISTS (
        SELECT 1 FROM native_consumed
        WHERE native_consumed.delivery_log_id=context.delivery_log_id
      )
  )
"#,
                    &[
                        &actor,
                        &turn_id,
                        &delivered_before_ms,
                        &max_records,
                        &max_body_chars,
                        &now_ms,
                        &claim_expires_at_ms,
                        &consumer_native_session_id,
                    ],
                )
                .await
                .map_err(|error| pg_store_error("claim delivered push context", error))?;
            Ok((row.get::<_, serde_json::Value>(0), row.get::<_, i64>(1)))
        })?;
        let records = records_json
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|value| {
                Some(DeliveredPushContextRecord {
                    delivery_log_id: value.get("delivery_log_id")?.as_i64()?,
                    source_id: value.get("source_id")?.as_str()?.to_string(),
                    delivered_at_ms: value.get("delivered_at_ms")?.as_i64()?,
                    body: value.get("body")?.as_str()?.to_string(),
                })
            })
            .collect();
        Ok(DeliveredPushContextClaim {
            records,
            remaining_count: usize::try_from(remaining_count).unwrap_or(usize::MAX),
        })
    }

    pub fn complete_delivered_push_context(
        &self,
        actor: &ActorIdentity,
        turn_id: &str,
    ) -> anyhow::Result<usize> {
        let actor = delivery_actor_key(actor);
        let turn_id = turn_id.to_string();
        let now_ms = Utc::now().timestamp_millis();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let count = client
                .execute(
                    r#"
UPDATE delivered_push_context
SET consumed_turn_id=$2, consumed_at_ms=$3,
    claimed_turn_id=NULL, claim_expires_at_ms=NULL
WHERE actor=$1 AND claimed_turn_id=$2 AND consumed_at_ms IS NULL
"#,
                    &[&actor, &turn_id, &now_ms],
                )
                .await
                .map_err(|error| pg_store_error("complete delivered push context", error))?;
            Ok(usize::try_from(count).unwrap_or(usize::MAX))
        })
    }

    pub fn release_delivered_push_context(
        &self,
        actor: &ActorIdentity,
        turn_id: &str,
    ) -> anyhow::Result<usize> {
        let actor = delivery_actor_key(actor);
        let turn_id = turn_id.to_string();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let count = client
                .execute(
                    r#"
UPDATE delivered_push_context
SET claimed_turn_id=NULL, claim_expires_at_ms=NULL
WHERE actor=$1 AND claimed_turn_id=$2 AND consumed_at_ms IS NULL
"#,
                    &[&actor, &turn_id],
                )
                .await
                .map_err(|error| pg_store_error("release delivered push context", error))?;
            Ok(usize::try_from(count).unwrap_or(usize::MAX))
        })
    }

    pub fn list_recent_delivery_logs(
        &self,
        filter: &DeliveryLogFilter,
    ) -> anyhow::Result<Vec<DeliveryLogRecord>> {
        let filter = filter.clone();
        self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            let actor = filter.actor.filter(|value| !value.is_empty());
            let actor_channel = filter
                .actor_channel
                .filter(|value| !value.is_empty())
                .map(|value| format!("{value}::%"));
            let actor_user_id = filter
                .actor_user_id
                .filter(|value| !value.is_empty())
                .map(|value| format!("%::{value}"));
            let event_id = filter.event_id.filter(|value| !value.is_empty());
            let status = filter.status.filter(|value| !value.is_empty());
            let delivery_channel = filter.delivery_channel.filter(|value| !value.is_empty());
            let top_level_only = filter.top_level_only;
            let limit = i64::try_from(filter.limit.max(1)).unwrap_or(i64::MAX);
            let rows = client
                .query(
                    r#"
SELECT
  d.id, d.event_id, d.actor, d.channel, d.severity,
  d.sent_at_ts, d.status, d.body,
  e.title, e.summary, e.kind_json, e.source, e.url, e.symbols_json
FROM delivery_log d
LEFT JOIN events e ON e.id=d.event_id
WHERE ($1::bigint IS NULL OR d.sent_at_ts >= $1)
  AND ($2::bigint IS NULL OR d.sent_at_ts <= $2)
  AND (
    ($3::text IS NOT NULL AND d.actor=$3)
    OR (
      $3::text IS NULL
      AND ($4::text IS NULL OR d.actor LIKE $4)
      AND ($5::text IS NULL OR d.actor LIKE $5)
    )
  )
  AND ($6::text IS NULL OR d.event_id=$6)
  AND ($7::text IS NULL OR d.status=$7)
  AND ($8::text IS NULL OR d.channel=$8)
  AND (
    NOT $9::boolean
    OR d.channel NOT IN ('router','digest_item','global_digest_item')
  )
ORDER BY d.sent_at_ts DESC, d.id DESC
LIMIT $10
"#,
                    &[
                        &filter.since_ts,
                        &filter.until_ts,
                        &actor,
                        &actor_channel,
                        &actor_user_id,
                        &event_id,
                        &status,
                        &delivery_channel,
                        &top_level_only,
                        &limit,
                    ],
                )
                .await
                .map_err(|error| pg_store_error("list recent delivery logs", error))?;
            rows.iter().map(decode_delivery_log).collect()
        })
    }

    fn query_events(
        &self,
        sql: &'static str,
        first: i64,
        second: i64,
        operation_name: &'static str,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(sql, &[&first, &second])
                .await
                .map_err(|error| pg_store_error(operation_name, error))
        })?;
        Ok(rows
            .iter()
            .filter_map(|row| decode_market_event(row).ok())
            .collect())
    }

    fn event_ids_since(
        &self,
        sql: &'static str,
        scope: &str,
        since: DateTime<Utc>,
        operation_name: &'static str,
    ) -> anyhow::Result<HashSet<String>> {
        let scope = scope.to_string();
        let since = since.timestamp();
        let rows = self.run(move |postgres| async move {
            let client = postgres.connect_cached_client().await?;
            client
                .query(sql, &[&scope, &since])
                .await
                .map_err(|error| pg_store_error(operation_name, error))
        })?;
        Ok(rows.into_iter().map(|row| row.get(0)).collect())
    }
}

pub fn event_breakdown_by_source(
    store: &EventStore,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> anyhow::Result<Vec<(String, i64)>> {
    let since = since.timestamp();
    let until = until.timestamp();
    store.run(move |postgres| async move {
        let client = postgres.connect_cached_client().await?;
        let rows = client
            .query(
                r#"
SELECT source, count(*)::bigint FROM events
WHERE created_at_ts >= $1 AND created_at_ts < $2
GROUP BY source ORDER BY 2 DESC
"#,
                &[&since, &until],
            )
            .await
            .map_err(|error| pg_store_error("event breakdown by source", error))?;
        Ok(rows
            .into_iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect())
    })
}

pub fn delivery_breakdown_per_actor(
    store: &EventStore,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> anyhow::Result<Vec<(String, String, i64)>> {
    let since = since.timestamp();
    let until = until.timestamp();
    store.run(move |postgres| async move {
        let client = postgres.connect_cached_client().await?;
        let rows = client
            .query(
                r#"
SELECT actor, status, count(*)::bigint FROM delivery_log
WHERE sent_at_ts >= $1 AND sent_at_ts < $2
GROUP BY actor, status ORDER BY actor, status
"#,
                &[&since, &until],
            )
            .await
            .map_err(|error| pg_store_error("delivery breakdown per actor", error))?;
        Ok(rows
            .into_iter()
            .map(|row| (row.get(0), row.get(1), row.get(2)))
            .collect())
    })
}

fn event_store_operation_timeout() -> Duration {
    std::env::var("HONE_EVENT_STORE_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_EVENT_STORE_TIMEOUT_SECS))
}

fn acquire_test_connection_lease(
    namespace: &str,
    postgres: CloudPgRuntime,
) -> Arc<TestConnectionLease> {
    let mut leases = TEST_CONNECTION_LEASES
        .lock()
        .expect("event store test connection lease lock");
    if let Some(lease) = leases.get(namespace).and_then(Weak::upgrade) {
        return lease;
    }
    let lease = Arc::new(TestConnectionLease {
        namespace: namespace.to_string(),
        postgres,
    });
    leases.insert(namespace.to_string(), Arc::downgrade(&lease));
    lease
}

fn pg_store_error(context: &str, error: tokio_postgres::Error) -> HoneError {
    HoneError::Storage(format!("Postgres event store {context} 失败: {error}"))
}

fn decode_market_event(row: &Row) -> HoneResult<MarketEvent> {
    let kind_json: String = row
        .try_get(1)
        .map_err(|error| pg_store_error("decode event kind", error))?;
    let kind = serde_json::from_str::<EventKind>(&kind_json)
        .map_err(|error| HoneError::Serialization(error.to_string()))?;
    let severity_label: String = row
        .try_get(2)
        .map_err(|error| pg_store_error("decode event severity", error))?;
    let symbols_json: String = row
        .try_get(3)
        .map_err(|error| pg_store_error("decode event symbols", error))?;
    let occurred_at_ts: i64 = row
        .try_get(4)
        .map_err(|error| pg_store_error("decode event timestamp", error))?;
    let occurred_at = DateTime::<Utc>::from_timestamp(occurred_at_ts, 0).ok_or_else(|| {
        HoneError::Serialization(format!("event occurred_at_ts 越界: {occurred_at_ts}"))
    })?;
    let payload_json: String = row
        .try_get(9)
        .map_err(|error| pg_store_error("decode event payload", error))?;
    Ok(MarketEvent {
        id: row
            .try_get(0)
            .map_err(|error| pg_store_error("decode event id", error))?,
        kind,
        severity: match severity_label.as_str() {
            "high" => crate::event::Severity::High,
            "medium" => crate::event::Severity::Medium,
            _ => crate::event::Severity::Low,
        },
        symbols: serde_json::from_str(&symbols_json).unwrap_or_default(),
        occurred_at,
        title: row
            .try_get(5)
            .map_err(|error| pg_store_error("decode event title", error))?,
        summary: row
            .try_get(6)
            .map_err(|error| pg_store_error("decode event summary", error))?,
        url: row
            .try_get(7)
            .map_err(|error| pg_store_error("decode event url", error))?,
        source: row
            .try_get(8)
            .map_err(|error| pg_store_error("decode event source", error))?,
        payload: serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null),
    })
}

fn decode_delivery_log(row: &Row) -> HoneResult<DeliveryLogRecord> {
    let kind_json: Option<String> = row
        .try_get(10)
        .map_err(|error| pg_store_error("decode delivery event kind", error))?;
    let symbols_json: Option<String> = row
        .try_get(13)
        .map_err(|error| pg_store_error("decode delivery event symbols", error))?;
    let event_kind = kind_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(|value| value.get("type")?.as_str().map(str::to_string));
    let event_symbols = symbols_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
        .unwrap_or_default();
    Ok(DeliveryLogRecord {
        id: row
            .try_get(0)
            .map_err(|error| pg_store_error("decode delivery id", error))?,
        event_id: row
            .try_get(1)
            .map_err(|error| pg_store_error("decode delivery event id", error))?,
        actor: row
            .try_get(2)
            .map_err(|error| pg_store_error("decode delivery actor", error))?,
        channel: row
            .try_get(3)
            .map_err(|error| pg_store_error("decode delivery channel", error))?,
        severity: row
            .try_get(4)
            .map_err(|error| pg_store_error("decode delivery severity", error))?,
        sent_at_ts: row
            .try_get(5)
            .map_err(|error| pg_store_error("decode delivery timestamp", error))?,
        status: row
            .try_get(6)
            .map_err(|error| pg_store_error("decode delivery status", error))?,
        body: row
            .try_get(7)
            .map_err(|error| pg_store_error("decode delivery body", error))?,
        event_title: row
            .try_get(8)
            .map_err(|error| pg_store_error("decode delivery event title", error))?,
        event_summary: row
            .try_get(9)
            .map_err(|error| pg_store_error("decode delivery event summary", error))?,
        event_kind,
        event_source: row
            .try_get(11)
            .map_err(|error| pg_store_error("decode delivery event source", error))?,
        event_url: row
            .try_get(12)
            .map_err(|error| pg_store_error("decode delivery event url", error))?,
        event_symbols,
    })
}

fn ensure_payload_object(
    payload: &mut serde_json::Value,
) -> &mut serde_json::Map<String, serde_json::Value> {
    if !payload.is_object() {
        *payload = serde_json::Value::Object(serde_json::Map::new());
    }
    payload
        .as_object_mut()
        .expect("payload was normalized to an object")
}

fn truncate_store_error(error: &str) -> String {
    error.chars().take(300).collect()
}

fn severity_tag(severity: &crate::event::Severity) -> &'static str {
    match severity {
        crate::event::Severity::Low => "low",
        crate::event::Severity::Medium => "medium",
        crate::event::Severity::High => "high",
    }
}

fn delivery_actor_key(actor: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        actor.channel,
        actor.channel_scope.as_deref().unwrap_or_default(),
        actor.user_id
    )
}

fn category_kind_tags(category: &str) -> Option<&'static [&'static str]> {
    match category {
        "price" => Some(&["price_alert", "weekly52_high", "weekly52_low"]),
        "news" => Some(&["news_critical", "social_post"]),
        "filing" => Some(&["sec_filing"]),
        "earnings" => Some(&[
            "earnings_upcoming",
            "earnings_released",
            "earnings_call_transcript",
        ]),
        "macro" => Some(&["macro_event"]),
        "corp_action" => Some(&["dividend", "split"]),
        "analyst" => Some(&["analyst_grade"]),
        _ => None,
    }
}

fn parse_bps_from_band_id(id: &str) -> Option<i64> {
    if !id.starts_with("price_band:") {
        return None;
    }
    id.rsplit(':').next().and_then(|value| value.parse().ok())
}

fn price_band_id_pattern(symbol: &str, direction: &str) -> Option<String> {
    let direction = match direction {
        "up" | "down" => direction,
        _ => return None,
    };
    Some(format!(
        "price_band:{}:%:{}:%",
        symbol.to_uppercase(),
        direction
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use tempfile::tempdir;

    fn sample_event(id: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "Apple earnings".into(),
            summary: String::new(),
            url: None,
            source: "fmp.earning_calendar".into(),
            payload: serde_json::Value::Null,
        }
    }

    fn reviewed_release(id: &str, occurred_at: DateTime<Utc>) -> MarketEvent {
        let mut event = sample_event(id);
        event.kind = EventKind::EarningsReleased;
        event.symbols = vec!["SNDK".into()];
        event.occurred_at = occurred_at;
        event.url = Some("https://sec.gov/Archives/sndk-q4-ex991.htm".into());
        event.payload = serde_json::json!({
            "earnings_quality_review_applied": true,
            EARNINGS_DOCUMENT_KEY: "https://sec.gov/archives/sndk-q4-ex991.htm"
        });
        event
    }

    #[test]
    fn insert_is_idempotent_per_id() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let event = sample_event("earnings:AAPL:2026-04-30");
        assert!(store.insert_event(&event).unwrap()); // 首次
        assert!(!store.insert_event(&event).unwrap()); // 重复
        assert_eq!(store.count_events().unwrap(), 1);
    }

    #[test]
    fn transcript_and_formal_filing_link_to_the_nearest_reviewed_release() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let occurred_at = Utc::now();
        let mut release = reviewed_release("release", occurred_at);
        let release_key = store
            .link_earnings_research_object(&mut release)
            .unwrap()
            .unwrap();
        store.insert_event(&release).unwrap();

        let mut transcript = sample_event("transcript");
        transcript.kind = EventKind::EarningsCallTranscript;
        transcript.symbols = vec!["SNDK".into()];
        transcript.occurred_at = occurred_at + chrono::Duration::hours(3);
        transcript.payload = serde_json::json!({"fmp": {"year": 2026, "quarter": 4}});
        // 全局同一时段即使有超过查询上限的其它 ticker 噪声，也必须先在 SQL
        // 层按 symbol 收窄，不能把 SNDK release 挤出候选集。
        for index in 0..250 {
            let mut noise = sample_event(&format!("noise-nearest-{index}"));
            noise.symbols = vec!["AMD".into()];
            noise.occurred_at = transcript.occurred_at;
            store.insert_event(&noise).unwrap();
        }
        assert_eq!(
            store
                .link_earnings_research_object(&mut transcript)
                .unwrap()
                .as_deref(),
            Some(release_key.as_str())
        );

        let mut filing = sample_event("10q");
        filing.kind = EventKind::SecFiling {
            form: "10-Q".into(),
        };
        filing.symbols = vec!["SNDK".into()];
        filing.occurred_at = occurred_at + chrono::Duration::days(2);
        assert_eq!(
            store
                .link_earnings_research_object(&mut filing)
                .unwrap()
                .as_deref(),
            Some(release_key.as_str())
        );
    }

    #[test]
    fn reviewed_release_backfills_a_transcript_that_arrived_first() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let occurred_at = Utc.with_ymd_and_hms(2026, 8, 5, 20, 0, 0).unwrap();
        let mut transcript = sample_event("transcript-first");
        transcript.kind = EventKind::EarningsCallTranscript;
        transcript.symbols = vec!["SNDK".into()];
        transcript.occurred_at = occurred_at - chrono::Duration::hours(2);
        assert!(
            store
                .link_earnings_research_object(&mut transcript)
                .unwrap()
                .is_none()
        );
        store.insert_event(&transcript).unwrap();
        for index in 0..250 {
            let mut noise = sample_event(&format!("noise-backfill-{index}"));
            noise.symbols = vec!["AMD".into()];
            noise.occurred_at = occurred_at;
            store.insert_event(&noise).unwrap();
        }

        let mut release = reviewed_release("release-later", occurred_at);
        let release_key = store
            .link_earnings_research_object(&mut release)
            .unwrap()
            .unwrap();
        store.insert_event(&release).unwrap();
        assert_eq!(
            store
                .event_research_object_key("transcript-first")
                .unwrap()
                .as_deref(),
            Some(release_key.as_str())
        );
        let materials = store
            .list_earnings_research_materials(&release_key)
            .unwrap();
        assert_eq!(materials.len(), 1);
        assert_eq!(materials[0].id, "transcript-first");
    }

    #[test]
    fn continuity_job_survives_restart_retries_and_recovers_an_expired_lease() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("event-store");
        let actor = ActorIdentity::new("discord", "pro", None::<&str>).unwrap();
        let occurred_at = Utc::now();
        let fixture = EventStore::open(&path).unwrap();
        let mut release = reviewed_release("release-job", occurred_at);
        fixture.link_earnings_research_object(&mut release).unwrap();
        let job_key = fixture
            .enqueue_earnings_continuity_job(&actor, &release)
            .unwrap()
            .unwrap();

        let store = EventStore::open(&path).unwrap();
        // enqueue 使用真实时钟写入 next_attempt_ts；给模拟领取时钟留出一秒，
        // 避免测试恰好跨过整秒边界时把新任务误判为尚未到期。
        let first_claim_at = Utc::now() + chrono::Duration::seconds(1);
        let first = store
            .claim_due_earnings_continuity_jobs(first_claim_at, 4)
            .unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].job_key, job_key);
        assert_eq!(first[0].attempts, 1);
        assert!(
            store
                .claim_due_earnings_continuity_jobs(
                    first_claim_at + chrono::Duration::minutes(14),
                    4,
                )
                .unwrap()
                .is_empty()
        );

        // 模拟进程在 running 状态崩溃：租约过期后可由新 worker 重新领取。
        drop(store);
        let store = EventStore::open(&path).unwrap();
        let recovered = store
            .claim_due_earnings_continuity_jobs(first_claim_at + chrono::Duration::minutes(16), 4)
            .unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].attempts, 2);
        // 第一份 worker 即使在租约过期后迟到，也不能完成或重排第二份 worker
        // 已领取的 attempt。
        assert!(
            !store
                .complete_earnings_continuity_job(&job_key, first[0].attempts)
                .unwrap()
        );
        assert!(
            store
                .retry_earnings_continuity_job(
                    &job_key,
                    recovered[0].attempts,
                    "temporary provider failure",
                    first_claim_at + chrono::Duration::minutes(16),
                )
                .unwrap()
        );
        assert_eq!(
            store
                .earnings_continuity_job_status(&job_key)
                .unwrap()
                .as_deref(),
            Some("retry")
        );
        assert!(
            store
                .claim_due_earnings_continuity_jobs(
                    first_claim_at + chrono::Duration::minutes(17),
                    4,
                )
                .unwrap()
                .is_empty()
        );
        let retried = store
            .claim_due_earnings_continuity_jobs(first_claim_at + chrono::Duration::minutes(19), 4)
            .unwrap();
        assert_eq!(retried.len(), 1);
        assert_eq!(retried[0].attempts, 3);
        assert!(
            store
                .complete_earnings_continuity_job(&job_key, retried[0].attempts)
                .unwrap()
        );
        assert_eq!(
            store
                .earnings_continuity_job_status(&job_key)
                .unwrap()
                .as_deref(),
            Some("completed")
        );
    }

    #[test]
    fn release_and_reviewed_transcript_have_distinct_continuity_jobs() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = ActorIdentity::new("discord", "pro", None::<&str>).unwrap();
        let occurred_at = Utc::now();
        let mut release = reviewed_release("release-job-stage", occurred_at);
        let research_object_key = store
            .link_earnings_research_object(&mut release)
            .unwrap()
            .unwrap();
        let release_job = store
            .enqueue_earnings_continuity_job(&actor, &release)
            .unwrap()
            .unwrap();

        let mut transcript = sample_event("transcript-job-stage");
        transcript.kind = EventKind::EarningsCallTranscript;
        transcript.symbols = release.symbols.clone();
        transcript.payload = serde_json::json!({
            (EARNINGS_RESEARCH_OBJECT_KEY): research_object_key,
            "earnings_transcript_review_applied": true
        });
        let transcript_job = store
            .enqueue_earnings_continuity_job(&actor, &transcript)
            .unwrap()
            .unwrap();
        assert_ne!(release_job, transcript_job);
        assert!(transcript_job.ends_with("::earnings_transcript"));

        let jobs = store
            .claim_due_earnings_continuity_jobs(Utc::now() + chrono::Duration::seconds(1), 4)
            .unwrap();
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn contains_event_supports_cross_restart_poller_short_circuit() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        assert!(!store.contains_event("earnings:SNDK:q4").unwrap());
        store
            .insert_event(&sample_event("earnings:SNDK:q4"))
            .unwrap();
        assert!(store.contains_event("earnings:SNDK:q4").unwrap());
    }

    #[test]
    fn sec_fallback_is_superseded_only_after_actor_received_structured_earnings() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let mut earnings = sample_event("earnings_surprise:SNDK:2026-08-05");
        earnings.kind = EventKind::EarningsReleased;
        earnings.severity = Severity::High;
        earnings.url =
            Some("https://www.sec.gov/Archives/sndkq4-26ex991xpressrelease.htm?source=fmp".into());
        earnings.payload = serde_json::json!({
            "earnings_quality_review_applied": true,
            "hone_earnings_release_document_key":
                "https://www.sec.gov/archives/sndkq4-26ex991xpressrelease.htm"
        });
        store.insert_event(&earnings).unwrap();

        let delivered_actor = "discord::::delivered";
        let failed_actor = "discord::::failed";
        store
            .log_delivery(
                &earnings.id,
                delivered_actor,
                "sink",
                Severity::High,
                "sent",
                None,
            )
            .unwrap();
        store
            .log_delivery(
                &earnings.id,
                failed_actor,
                "sink",
                Severity::High,
                "failed",
                None,
            )
            .unwrap();

        let lookup_url = "HTTPS://WWW.SEC.GOV/Archives/sndkq4-26ex991xpressrelease.htm#document";
        assert!(
            store
                .actor_has_delivered_earnings_for_document(delivered_actor, lookup_url)
                .unwrap()
        );
        assert!(
            !store
                .actor_has_delivered_earnings_for_document(failed_actor, lookup_url)
                .unwrap()
        );
    }

    #[test]
    fn distinct_ids_are_all_stored() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        assert!(store.insert_event(&sample_event("a")).unwrap());
        assert!(store.insert_event(&sample_event("b")).unwrap());
        assert!(store.insert_event(&sample_event("c")).unwrap());
        assert_eq!(store.count_events().unwrap(), 3);
    }

    #[test]
    fn baseline_is_set_on_first_open_and_preserved() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("event-store");
        // pg_temp 的生命周期绑定测试连接；保留 fixture lease，同时用第二个
        // EventStore 句柄覆盖生产中的重复构造不会改写 baseline。
        let fixture = EventStore::open(&path).unwrap();
        let baseline_a = fixture.baseline_at().unwrap();
        // 重新打开不应重写 baseline
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let store = EventStore::open(&path).unwrap();
        let baseline_b = store.baseline_at().unwrap();
        assert_eq!(baseline_a, baseline_b);
    }

    #[test]
    fn delivery_log_is_append_only_across_retries() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        store
            .log_delivery(
                "ev1",
                "imessage:u1",
                "imessage",
                Severity::High,
                "failed",
                Some("body v1"),
            )
            .unwrap();
        // 同一 (event, actor) 二次写入应保留两行，而非覆盖
        store
            .log_delivery(
                "ev1",
                "imessage:u1",
                "imessage",
                Severity::High,
                "sent",
                Some("body v2"),
            )
            .unwrap();
        let (attempt_count, last_status) = store
            .test_delivery_attempt_summary("ev1", "imessage:u1")
            .unwrap();
        assert_eq!(attempt_count, 2, "delivery_log 应 append-only 保留每次尝试");
        assert_eq!(last_status, "sent");
    }

    #[test]
    fn delivered_push_context_claims_only_explicit_confirmations_in_delivery_order() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = ActorIdentity::new("discord", "u1", Some("dm-1")).unwrap();
        let other_scope = ActorIdentity::new("discord", "u1", Some("room-2")).unwrap();
        let actor_key = delivery_actor_key(&actor);

        store
            .log_delivery(
                "failed",
                &actor_key,
                "sink",
                Severity::High,
                "failed",
                Some("FAILED"),
            )
            .unwrap();
        store
            .log_delivery(
                "queued",
                &actor_key,
                "digest",
                Severity::Medium,
                "queued",
                Some("QUEUED"),
            )
            .unwrap();
        store
            .log_delivery(
                "dryrun",
                &actor_key,
                "sink",
                Severity::High,
                "dryrun",
                Some("DRYRUN"),
            )
            .unwrap();
        store
            .log_delivery(
                "blank",
                &actor_key,
                "sink",
                Severity::High,
                "sent",
                Some("  \n"),
            )
            .unwrap();
        store
            .log_delivery(
                "audit-only-sent",
                &actor_key,
                "sink",
                Severity::High,
                "sent",
                Some("AUDIT ONLY"),
            )
            .unwrap();
        for (source, body) in [("p1", "P1"), ("p2", "P2"), ("p3", "P3")] {
            store
                .log_confirmed_delivery(source, &actor, "sink", Severity::High, body, None)
                .unwrap();
        }
        // delivery_log 保留 retry attempt；上下文按业务 source id 去重。
        store
            .log_confirmed_delivery("p1", &actor, "sink", Severity::High, "P1 DUPLICATE", None)
            .unwrap();
        store
            .log_confirmed_delivery(
                "other",
                &other_scope,
                "sink",
                Severity::High,
                "OTHER SCOPE",
                None,
            )
            .unwrap();

        let cutoff = Utc::now().timestamp_millis().saturating_add(1_000);
        let first = store
            .claim_delivered_push_context(&actor, "turn-1", cutoff, 2, 12_000, 60_000)
            .unwrap();
        assert_eq!(
            first
                .records
                .iter()
                .map(|record| record.body.as_str())
                .collect::<Vec<_>>(),
            vec!["P1", "P2"]
        );
        assert_eq!(first.remaining_count, 1);

        // 同一 turn 重入必须返回相同批次，不能把 P3 偷塞进内部 retry。
        let reentered = store
            .claim_delivered_push_context(&actor, "turn-1", cutoff, 20, 12_000, 60_000)
            .unwrap();
        assert_eq!(reentered.records, first.records);
        assert_eq!(reentered.remaining_count, 1);

        assert_eq!(
            store
                .complete_delivered_push_context(&actor, "turn-1")
                .unwrap(),
            2
        );
        let second = store
            .claim_delivered_push_context(&actor, "turn-2", cutoff, 20, 12_000, 60_000)
            .unwrap();
        assert_eq!(second.records.len(), 1);
        assert_eq!(second.records[0].body, "P3");
        assert_eq!(second.remaining_count, 0);

        assert_eq!(
            store
                .release_delivered_push_context(&actor, "turn-2")
                .unwrap(),
            1
        );
        let reclaimed = store
            .claim_delivered_push_context(&actor, "turn-3", cutoff, 20, 12_000, 60_000)
            .unwrap();
        assert_eq!(reclaimed.records[0].body, "P3");
        store
            .complete_delivered_push_context(&actor, "turn-3")
            .unwrap();
        assert!(
            store
                .claim_delivered_push_context(&actor, "turn-4", cutoff, 20, 12_000, 60_000)
                .unwrap()
                .records
                .is_empty()
        );

        let other = store
            .claim_delivered_push_context(&other_scope, "turn-other", cutoff, 20, 12_000, 60_000)
            .unwrap();
        assert_eq!(other.records[0].body, "OTHER SCOPE");
    }

    #[test]
    fn delivered_push_after_user_cutoff_waits_for_next_turn() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = ActorIdentity::new("telegram", "u1", None::<String>).unwrap();
        store
            .log_confirmed_delivery(
                "future-push",
                &actor,
                "sink",
                Severity::High,
                "PUSH AFTER U1",
                None,
            )
            .unwrap();
        let user_cutoff = Utc::now().timestamp_millis();
        store
            .test_set_delivered_at_ms("future-push", user_cutoff + 1)
            .unwrap();

        assert!(
            store
                .claim_delivered_push_context(&actor, "u1", user_cutoff, 20, 12_000, 60_000)
                .unwrap()
                .records
                .is_empty()
        );
        let next = store
            .claim_delivered_push_context(&actor, "u2", user_cutoff + 1, 20, 12_000, 60_000)
            .unwrap();
        assert_eq!(next.records[0].body, "PUSH AFTER U1");
    }

    #[test]
    fn delivered_push_context_crosses_store_connections_and_respects_body_budget() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("event-store");
        let writer = EventStore::open(&path).unwrap();
        let reader = EventStore::open(&path).unwrap();
        let actor = ActorIdentity::new("web", "u1", Some("chat-1")).unwrap();

        for (source, body) in [("p1", "123456"), ("p2", "abcdef")] {
            writer
                .log_confirmed_delivery(source, &actor, "sink", Severity::Medium, body, None)
                .unwrap();
        }

        let cutoff = Utc::now().timestamp_millis().saturating_add(1_000);
        let first = reader
            .claim_delivered_push_context(&actor, "u1", cutoff, 20, 8, 60_000)
            .unwrap();
        assert_eq!(first.records.len(), 1);
        assert_eq!(first.records[0].body, "123456");
        assert_eq!(first.remaining_count, 1);

        reader
            .complete_delivered_push_context(&actor, "u1")
            .unwrap();
        let second = writer
            .claim_delivered_push_context(&actor, "u2", cutoff, 20, 8, 60_000)
            .unwrap();
        assert_eq!(second.records[0].body, "abcdef");
        assert_eq!(second.remaining_count, 0);
    }

    #[test]
    fn native_session_observation_skips_duplicate_but_replay_still_claims() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let native_actor = ActorIdentity::new("cli", "native", None::<String>).unwrap();
        let replay_actor = ActorIdentity::new("web", "replay", None::<String>).unwrap();
        store
            .log_confirmed_delivery(
                "scheduled:native",
                &native_actor,
                "scheduler",
                Severity::Medium,
                "NATIVE ALREADY SAW THIS",
                Some(&native_actor.session_id()),
            )
            .unwrap();
        store
            .log_confirmed_delivery(
                "scheduled:replay",
                &replay_actor,
                "scheduler",
                Severity::Medium,
                "REPLAY NEEDS THIS",
                Some(&replay_actor.session_id()),
            )
            .unwrap();
        let cutoff = Utc::now().timestamp_millis().saturating_add(1_000);

        let native = store
            .claim_delivered_push_context_with_native_observation(
                &native_actor,
                "native-next-turn",
                cutoff,
                20,
                12_000,
                60_000,
                Some(&native_actor.session_id()),
            )
            .unwrap();
        assert!(native.records.is_empty());

        let replay = store
            .claim_delivered_push_context(
                &replay_actor,
                "replay-next-turn",
                cutoff,
                20,
                12_000,
                60_000,
            )
            .unwrap();
        assert_eq!(replay.records.len(), 1);
        assert_eq!(replay.records[0].body, "REPLAY NEEDS THIS");
    }

    #[test]
    fn opening_an_old_delivery_log_does_not_backfill_historical_context() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("event-store");
        EventStore::open(&path)
            .unwrap()
            .test_insert_historical_delivery()
            .unwrap();

        let store = EventStore::open(&path).unwrap();
        let actor = ActorIdentity::new("discord", "u1", None::<String>).unwrap();
        assert!(
            store
                .claim_delivered_push_context(
                    &actor,
                    "new-turn",
                    Utc::now().timestamp_millis(),
                    20,
                    12_000,
                    60_000,
                )
                .unwrap()
                .records
                .is_empty()
        );
    }

    #[test]
    fn list_recent_delivery_logs_keeps_operator_level_rows() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        store
            .log_delivery(
                "ev-no-actor",
                "event_engine::::no_actor",
                "router",
                Severity::Low,
                "no_actor",
                None,
            )
            .unwrap();
        store
            .log_delivery(
                "ev-item",
                "discord::::u1",
                "digest_item",
                Severity::Medium,
                "omitted",
                None,
            )
            .unwrap();
        store
            .log_delivery(
                "ev-sink",
                "discord::::u1",
                "sink",
                Severity::High,
                "sent",
                Some("body"),
            )
            .unwrap();

        let rows = store
            .list_recent_delivery_logs(&DeliveryLogFilter {
                top_level_only: true,
                limit: 20,
                ..DeliveryLogFilter::default()
            })
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_id, "ev-sink");
        assert_eq!(rows[0].channel, "sink");
    }

    #[test]
    fn list_recent_delivery_logs_exposes_event_kind_type() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let mut event = sample_event("ev-kind");
        event.kind = EventKind::SecFiling {
            form: "8-K".to_string(),
        };
        store.insert_event(&event).unwrap();
        store
            .log_delivery(
                "ev-kind",
                "discord::::u1",
                "sink",
                Severity::High,
                "sent",
                Some("body"),
            )
            .unwrap();

        let rows = store
            .list_recent_delivery_logs(&DeliveryLogFilter {
                actor: Some("discord::::u1".to_string()),
                top_level_only: true,
                limit: 20,
                ..DeliveryLogFilter::default()
            })
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_kind.as_deref(), Some("sec_filing"));
    }

    #[test]
    fn jsonl_mirror_appends_once_per_new_event() {
        let dir = tempdir().unwrap();
        let mirror = dir.path().join("events.jsonl");
        let store = EventStore::open(dir.path().join("event-store"))
            .unwrap()
            .with_jsonl_path(&mirror);
        let event = sample_event("e-jsonl");
        assert!(store.insert_event(&event).unwrap());
        // 重复入库走 IGNORE，不再 append 镜像
        assert!(!store.insert_event(&event).unwrap());
        let lines = std::fs::read_to_string(&mirror).unwrap();
        assert_eq!(lines.lines().count(), 1);
        assert!(lines.contains("e-jsonl"));
    }

    #[test]
    fn count_high_sent_since_only_counts_high_sink_sent() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = "tg::::u1";
        // 真正算数的:高优 + sink + sent —— 4 条
        for i in 0..4 {
            store
                .log_delivery(
                    &format!("e{i}"),
                    actor,
                    "sink",
                    Severity::High,
                    "sent",
                    None,
                )
                .unwrap();
        }
        // 不算数的对照组
        store
            .log_delivery("e-medium", actor, "sink", Severity::Medium, "sent", None)
            .unwrap();
        store
            .log_delivery("e-failed", actor, "sink", Severity::High, "failed", None)
            .unwrap();
        store
            .log_delivery("e-digest", actor, "digest", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery(
                "e-filtered",
                actor,
                "prefs",
                Severity::High,
                "filtered",
                None,
            )
            .unwrap();
        store
            .log_delivery("e-other", "tg::::u2", "sink", Severity::High, "sent", None)
            .unwrap();

        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(store.count_high_sent_since(actor, since).unwrap(), 4);

        // 未来时间点:当然 0
        let future = Utc::now() + chrono::Duration::days(1);
        assert_eq!(store.count_high_sent_since(actor, future).unwrap(), 0);
    }

    #[test]
    fn high_counts_are_bucketed_by_event_category() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = "tg::::u1";
        let mut price = sample_event("price-aapl");
        price.kind = EventKind::PriceAlert {
            pct_change_bps: 700,
            window: "day".into(),
        };
        let mut filing = sample_event("sec-aapl");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        store.insert_event(&price).unwrap();
        store.insert_event(&filing).unwrap();
        store
            .log_delivery(&price.id, actor, "sink", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery(&filing.id, actor, "sink", Severity::High, "sent", None)
            .unwrap();

        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(
            store
                .count_high_sent_since_for_category(actor, since, "price")
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .count_high_sent_since_for_category(actor, since, "filing")
                .unwrap(),
            1
        );
        assert_eq!(store.count_high_sent_since(actor, since).unwrap(), 2);
    }

    #[test]
    fn last_high_sink_send_for_symbol_matches_case_insensitive_and_ignores_other_rows() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = "tg::::u1";

        // 给 AAPL 和 NVDA 分别入库一条事件
        let mut aapl = sample_event("ev-aapl");
        aapl.symbols = vec!["AAPL".into()];
        let mut nvda = sample_event("ev-nvda");
        nvda.symbols = vec!["NVDA".into()];
        store.insert_event(&aapl).unwrap();
        store.insert_event(&nvda).unwrap();

        // 初始状态:无记录
        assert!(
            store
                .last_high_sink_send_for_symbol(actor, "AAPL")
                .unwrap()
                .is_none()
        );

        // High + sink + sent AAPL —— 应命中
        store
            .log_delivery("ev-aapl", actor, "sink", Severity::High, "sent", None)
            .unwrap();
        // Medium 不算,failed 不算,digest 渠道不算
        let mut medium_ev = sample_event("ev-medium");
        medium_ev.symbols = vec!["AAPL".into()];
        store.insert_event(&medium_ev).unwrap();
        store
            .log_delivery("ev-medium", actor, "sink", Severity::Medium, "sent", None)
            .unwrap();
        let mut failed_ev = sample_event("ev-failed");
        failed_ev.symbols = vec!["AAPL".into()];
        store.insert_event(&failed_ev).unwrap();
        store
            .log_delivery("ev-failed", actor, "sink", Severity::High, "failed", None)
            .unwrap();
        // 另一个 actor 的 sent 不算
        store
            .log_delivery("ev-aapl", "tg::::u2", "sink", Severity::High, "sent", None)
            .unwrap();
        // NVDA 的不应串到 AAPL
        store
            .log_delivery("ev-nvda", actor, "sink", Severity::High, "sent", None)
            .unwrap();

        let t_aapl = store.last_high_sink_send_for_symbol(actor, "aapl").unwrap();
        assert!(t_aapl.is_some(), "AAPL(小写查询)应命中");
        // 不存在的 symbol
        assert!(
            store
                .last_high_sink_send_for_symbol(actor, "TSLA")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn last_high_sink_send_with_firm_filter_distinguishes_grading_company() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = "tg::::u1";

        let mk = |id: &str, firm: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::AnalystGrade,
            severity: Severity::High,
            symbols: vec!["SNDK".into()],
            occurred_at: Utc::now(),
            title: "grade".into(),
            summary: String::new(),
            url: None,
            source: "fmp.grade".into(),
            payload: serde_json::json!({"gradingCompany": firm}),
        };
        let goldman = mk("g1", "Goldman Sachs");
        let raymond = mk("r1", "Raymond James");
        store.insert_event(&goldman).unwrap();
        store.insert_event(&raymond).unwrap();
        store
            .log_delivery("g1", actor, "sink", Severity::High, "sent", None)
            .unwrap();

        // 不带 firm 过滤 → 命中 Goldman 的 sent
        assert!(
            store
                .last_high_sink_send_for_symbol_category(actor, "SNDK", "analyst", None)
                .unwrap()
                .is_some()
        );
        // 带 firm = Goldman → 命中
        assert!(
            store
                .last_high_sink_send_for_symbol_category(
                    actor,
                    "SNDK",
                    "analyst",
                    Some("Goldman Sachs"),
                )
                .unwrap()
                .is_some()
        );
        // 带 firm = Raymond James → 没记录,应返回 None
        assert!(
            store
                .last_high_sink_send_for_symbol_category(
                    actor,
                    "SNDK",
                    "analyst",
                    Some("Raymond James"),
                )
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn last_high_sink_send_for_analyst_news_url_matches_same_article_fanout() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = "tg::::u1";
        let url = "https://thefly.com/ajax/news_get.php?id=4346982";

        let mk = |id: &str, firm: &str, news_url: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::AnalystGrade,
            severity: Severity::High,
            symbols: vec!["AMD".into()],
            occurred_at: Utc::now(),
            title: format!("AMD {firm} action"),
            summary: String::new(),
            url: Some(news_url.to_string()),
            source: "fmp.upgrades_downgrades".into(),
            payload: serde_json::json!({
                "gradingCompany": firm,
                "newsURL": news_url
            }),
        };
        let needham = mk("grade:AMD:t:Needham", "Needham", url);
        let jefferies = mk("grade:AMD:t:Jefferies", "Jefferies", url);
        let other_url = mk(
            "grade:AMD:t:RBC",
            "RBC Capital",
            "https://thefly.com/ajax/news_get.php?id=4346812",
        );
        store.insert_event(&needham).unwrap();
        store.insert_event(&jefferies).unwrap();
        store.insert_event(&other_url).unwrap();
        store
            .log_delivery(&needham.id, actor, "sink", Severity::High, "sent", None)
            .unwrap();

        let since = Utc::now() - chrono::Duration::minutes(5);
        assert!(
            store
                .last_high_sink_send_for_analyst_news_url(actor, "AMD", url, since)
                .unwrap()
                .is_some(),
            "same ticker + same newsURL fanout should be found"
        );
        assert!(
            store
                .last_high_sink_send_for_analyst_news_url(
                    actor,
                    "AMD",
                    "https://thefly.com/ajax/news_get.php?id=4346812",
                    since,
                )
                .unwrap()
                .is_none(),
            "different source article should not be cooled"
        );
        assert!(
            store
                .last_high_sink_send_for_analyst_news_url(actor, "NVDA", url, since)
                .unwrap()
                .is_none(),
            "same URL for another ticker should not match"
        );
    }

    #[test]
    fn event_breakdown_counts_by_source() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let mut a = sample_event("a");
        a.source = "fmp.stock_news".into();
        let mut b = sample_event("b");
        b.source = "fmp.stock_news".into();
        let mut c = sample_event("c");
        c.source = "fmp.earning_calendar".into();
        store.insert_event(&a).unwrap();
        store.insert_event(&b).unwrap();
        store.insert_event(&c).unwrap();
        let since = Utc::now() - chrono::Duration::minutes(1);
        let until = Utc::now() + chrono::Duration::minutes(1);
        let breakdown = event_breakdown_by_source(&store, since, until).unwrap();
        // news=2 排在 earnings=1 前面
        assert_eq!(breakdown[0], ("fmp.stock_news".into(), 2));
        assert_eq!(breakdown[1], ("fmp.earning_calendar".into(), 1));
    }

    #[test]
    fn delivery_breakdown_groups_per_actor_and_status() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        store
            .log_delivery("e1", "u1", "tg", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery("e2", "u1", "tg", Severity::Medium, "queued", None)
            .unwrap();
        store
            .log_delivery("e3", "u1", "tg", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery("e4", "u2", "tg", Severity::High, "failed", None)
            .unwrap();
        let since = Utc::now() - chrono::Duration::minutes(1);
        let until = Utc::now() + chrono::Duration::minutes(1);
        let breakdown = delivery_breakdown_per_actor(&store, since, until).unwrap();
        assert!(breakdown.contains(&("u1".into(), "sent".into(), 2)));
        assert!(breakdown.contains(&("u1".into(), "queued".into(), 1)));
        assert!(breakdown.contains(&("u2".into(), "failed".into(), 1)));
    }

    #[test]
    fn today_signal_kinds_returns_same_day_symbol_hits() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();

        // 今日 AAPL 价格异动
        let mut price = sample_event("price:AAPL:today");
        price.kind = EventKind::PriceAlert {
            pct_change_bps: 650,
            window: "day".into(),
        };
        price.occurred_at = Utc::now();
        store.insert_event(&price).unwrap();

        // 今日 AAPL 8-K
        let mut filing = sample_event("sec:AAPL:today");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        filing.occurred_at = Utc::now();
        store.insert_event(&filing).unwrap();

        // 其他 ticker（不应命中）
        let mut other = sample_event("price:NVDA:today");
        other.kind = EventKind::PriceAlert {
            pct_change_bps: 300,
            window: "day".into(),
        };
        other.symbols = vec!["NVDA".into()];
        other.occurred_at = Utc::now();
        store.insert_event(&other).unwrap();

        // 昨日 AAPL（不应命中）
        let mut stale = sample_event("earnings:AAPL:yesterday");
        stale.kind = EventKind::EarningsReleased;
        stale.occurred_at = Utc::now() - chrono::Duration::days(2);
        store.insert_event(&stale).unwrap();

        let since = Utc::now() - chrono::Duration::hours(12);
        let mut tags = store.today_signal_kinds("AAPL", since).unwrap();
        tags.sort();
        assert_eq!(tags, vec!["price_alert", "sec_filing"]);
    }

    #[test]
    fn list_upcoming_earnings_returns_in_window_only() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();

        // 未来 5 天后的 AAPL earnings —— 应命中(within_days=14)
        let mut future = sample_event("earnings:AAPL:2026-04-26");
        future.kind = EventKind::EarningsUpcoming;
        future.symbols = vec!["AAPL".into()];
        future.occurred_at = Utc::now() + chrono::Duration::days(5);
        store.insert_event(&future).unwrap();

        // 未来 30 天后的 NVDA —— 超出 14 天窗口,应不命中
        let mut far_future = sample_event("earnings:NVDA:2026-05-21");
        far_future.kind = EventKind::EarningsUpcoming;
        far_future.symbols = vec!["NVDA".into()];
        far_future.occurred_at = Utc::now() + chrono::Duration::days(30);
        store.insert_event(&far_future).unwrap();

        // 昨天的 TSLA earnings —— 过去,不命中
        let mut past = sample_event("earnings:TSLA:2026-04-20");
        past.kind = EventKind::EarningsUpcoming;
        past.symbols = vec!["TSLA".into()];
        past.occurred_at = Utc::now() - chrono::Duration::days(1);
        store.insert_event(&past).unwrap();

        // 未来 2 天的 AAPL 8-K —— 不是 earnings_upcoming,不命中
        let mut filing = sample_event("sec:AAPL:future");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        filing.symbols = vec!["AAPL".into()];
        filing.occurred_at = Utc::now() + chrono::Duration::days(2);
        store.insert_event(&filing).unwrap();

        let upcoming = store.list_upcoming_earnings(Utc::now(), 14).unwrap();
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].id, "earnings:AAPL:2026-04-26");
        assert!(matches!(upcoming[0].kind, EventKind::EarningsUpcoming));
    }

    #[test]
    fn purge_events_removes_older_rows() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        assert!(store.insert_event(&sample_event("old")).unwrap());
        // 人工把这条改到 40 天前
        let cutoff = Utc::now().timestamp() - 40 * 86_400;
        store.test_set_event_created_at("old", cutoff).unwrap();
        assert!(store.insert_event(&sample_event("new")).unwrap());
        let removed = store.purge_events_older_than(30).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.count_events().unwrap(), 1);
    }

    /// `delivered_event_ids_since` 是 digest synth 跨 flush 去重的底座 ——
    /// 必须只收 status=sent/dryrun、必须按 actor 隔离、必须**不需要 events 表
    /// 行**(synth 事件不会写 events 表,只在 delivery_log 留痕)。
    #[test]
    fn delivered_event_ids_since_filters_by_actor_status_and_time() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("event-store")).unwrap();
        let actor = "tg::::u1";
        let other = "tg::::u2";
        let earlier = chrono::Utc::now() - chrono::Duration::hours(1);

        // synth 事件本身不写 events 表,直接 log_delivery 也应能查出来
        store
            .log_delivery(
                "synth:earnings:GOOGL:2026-04-29:countdown:2026-04-26",
                actor,
                "digest_item",
                Severity::Medium,
                "sent",
                None,
            )
            .unwrap();
        // queued 不算已投递
        store
            .log_delivery(
                "synth:earnings:BE:2026-04-28:countdown:2026-04-26",
                actor,
                "digest_item",
                Severity::Medium,
                "queued",
                None,
            )
            .unwrap();
        // 其他 actor 不应混入本 actor 的结果
        store
            .log_delivery(
                "ev-other",
                other,
                "digest_item",
                Severity::Medium,
                "sent",
                None,
            )
            .unwrap();

        let ids = store.delivered_event_ids_since(actor, earlier).unwrap();
        assert!(ids.contains("synth:earnings:GOOGL:2026-04-29:countdown:2026-04-26"));
        assert!(!ids.contains("synth:earnings:BE:2026-04-28:countdown:2026-04-26"));
        assert!(!ids.contains("ev-other"));
    }
}
