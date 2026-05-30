use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, DATE, HOST, HeaderMap, HeaderName, HeaderValue,
};
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_postgres::{Client as PgClient, Config as PgConfig, NoTls};
use url::Url;

use crate::config::{CloudConfig, HoneConfig, OssConfig, PostgresConfig};
use crate::{ActorIdentity, HoneError, HoneResult};

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<sha2::Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeRole {
    Web,
    Worker,
    All,
}

impl RuntimeRole {
    pub fn from_env() -> Self {
        match std::env::var("HONE_RUNTIME_ROLE")
            .unwrap_or_else(|_| "all".to_string())
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "web" => Self::Web,
            "worker" => Self::Worker,
            _ => Self::All,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Worker => "worker",
            Self::All => "all",
        }
    }

    pub fn runs_worker_tasks(&self) -> bool {
        matches!(self, Self::Worker | Self::All)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudHealth {
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct CloudPgRuntime {
    config: PostgresConfig,
}

#[derive(Debug, Clone)]
pub struct CloudDocumentIndex {
    pub actor_storage_key: String,
    pub kind: String,
    pub document_id: String,
    pub oss_uri: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudSessionRecord {
    pub session_id: String,
    pub actor_storage_key: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudSessionImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudWebInviteUserRecord {
    pub user_id: String,
    pub phone_number: String,
    pub record: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudWebAuthSessionRecord {
    pub session_hash: String,
    pub user_id: String,
    pub expires_at: Option<String>,
    pub record: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudWebAuthImportReport {
    pub changed_users: usize,
    pub skipped_users: usize,
    pub changed_sessions: usize,
    pub skipped_sessions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCronJobRecord {
    pub actor_storage_key: String,
    pub job_id: String,
    pub actor: serde_json::Value,
    pub job: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudCronJobImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone)]
pub struct CloudCronExecutionInput {
    pub execution_status: String,
    pub message_send_status: String,
    pub should_deliver: bool,
    pub delivered: bool,
    pub response_preview: Option<String>,
    pub error_message: Option<String>,
    pub detail: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct CloudCronExecutionFilter {
    pub since: Option<String>,
    pub until: Option<String>,
    pub channel: Option<String>,
    pub user_id: Option<String>,
    pub job_id: Option<String>,
    pub execution_status: Option<String>,
    pub message_send_status: Option<String>,
    pub heartbeat_only: Option<bool>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCronExecutionRecord {
    pub run_id: i64,
    pub job_id: String,
    pub job_name: String,
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub channel_target: String,
    pub heartbeat: bool,
    pub executed_at: String,
    pub execution_status: String,
    pub message_send_status: String,
    pub should_deliver: bool,
    pub delivered: bool,
    pub response_preview: Option<String>,
    pub error_message: Option<String>,
    pub detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudConversationQuotaSnapshot {
    pub quota_date: String,
    pub success_count: u32,
    pub in_flight: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudConversationQuotaReserveOutcome {
    pub reserved: bool,
    pub snapshot: CloudConversationQuotaSnapshot,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudConversationQuotaImport {
    pub actor_storage_key: String,
    pub quota_date: String,
    pub success_count: u32,
    pub in_flight: u32,
    #[serde(rename = "limit_count")]
    pub limit: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudConversationQuotaImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

impl CloudPgRuntime {
    pub fn from_cloud_config(config: &CloudConfig) -> Option<Self> {
        config.postgres.is_configured().then(|| Self {
            config: config.postgres.clone(),
        })
    }

    async fn connect_client(&self) -> HoneResult<PgClient> {
        let proxy = self.config.resolved_proxy();
        if proxy.trim().is_empty() {
            let (client, connection) =
                tokio_postgres::connect(&self.config.resolved_database_url(), NoTls)
                    .await
                    .map_err(|err| HoneError::Config(format!("Postgres 连接失败: {err}")))?;
            tokio::spawn(async move {
                if let Err(error) = connection.await {
                    tracing::warn!("postgres connection task ended: {error}");
                }
            });
            return Ok(client);
        }

        let host = self.config.resolved_host();
        let port = self.config.resolved_port().unwrap_or(5432);
        let stream = connect_via_proxy(&proxy, &host, port).await?;
        let mut pg = PgConfig::new();
        pg.host(&host);
        pg.port(port);
        pg.user(&self.config.resolved_user());
        pg.password(self.config.resolved_password());
        pg.dbname(&self.config.resolved_database());
        let (client, connection) = pg
            .connect_raw(stream, NoTls)
            .await
            .map_err(|err| HoneError::Config(format!("Postgres 代理连接失败: {err}")))?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                tracing::warn!("postgres proxied connection task ended: {error}");
            }
        });
        Ok(client)
    }

    pub async fn health(&self) -> CloudHealth {
        match tokio::time::timeout(Duration::from_secs(5), self.connect_client()).await {
            Ok(Ok(client)) => match client.query_one("SELECT 1", &[]).await {
                Ok(_) => CloudHealth {
                    ok: true,
                    detail: "postgres connected".to_string(),
                },
                Err(error) => CloudHealth {
                    ok: false,
                    detail: format!("postgres query failed: {error}"),
                },
            },
            Ok(Err(error)) => CloudHealth {
                ok: false,
                detail: error.to_string(),
            },
            Err(_) => CloudHealth {
                ok: false,
                detail: "postgres health timeout".to_string(),
            },
        }
    }

    pub async fn ensure_schema(&self) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .batch_execute(
                r#"
CREATE TABLE IF NOT EXISTS cloud_schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_documents (
  actor_storage_key TEXT NOT NULL,
  kind TEXT NOT NULL,
  document_id TEXT NOT NULL,
  oss_uri TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  size_bytes BIGINT NOT NULL,
  version BIGINT NOT NULL DEFAULT 1,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (actor_storage_key, kind, document_id)
);
CREATE TABLE IF NOT EXISTS distributed_leases (
  lease_name TEXT PRIMARY KEY,
  owner_id TEXT NOT NULL,
  fencing_token BIGSERIAL NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS runtime_locks (
  lock_name TEXT PRIMARY KEY,
  owner_id TEXT NOT NULL,
  fencing_token BIGSERIAL NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS conversation_quota (
  actor_storage_key TEXT NOT NULL,
  quota_date DATE NOT NULL,
  limit_count INTEGER NOT NULL,
  reserved_count INTEGER NOT NULL DEFAULT 0,
  committed_count INTEGER NOT NULL DEFAULT 0,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (actor_storage_key, quota_date)
);
CREATE TABLE IF NOT EXISTS cron_job_claims (
  job_id TEXT NOT NULL,
  due_at TIMESTAMPTZ NOT NULL,
  owner_id TEXT NOT NULL,
  claimed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (job_id, due_at)
);
CREATE TABLE IF NOT EXISTS cloud_cron_jobs (
  actor_storage_key TEXT NOT NULL,
  job_id TEXT NOT NULL,
  actor JSONB NOT NULL,
  job JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (actor_storage_key, job_id)
);
CREATE TABLE IF NOT EXISTS cloud_cron_job_claims (
  job_key TEXT NOT NULL,
  due_key TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  claimed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (job_key, due_key)
);
CREATE TABLE IF NOT EXISTS cloud_cron_job_runs (
  run_id BIGSERIAL PRIMARY KEY,
  job_id TEXT NOT NULL,
  job_name TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_channel_scope TEXT,
  channel_target TEXT NOT NULL,
  heartbeat BOOLEAN NOT NULL DEFAULT false,
  executed_at TEXT NOT NULL,
  execution_status TEXT NOT NULL,
  message_send_status TEXT NOT NULL,
  should_deliver BOOLEAN NOT NULL DEFAULT false,
  delivered BOOLEAN NOT NULL DEFAULT false,
  response_preview TEXT,
  error_message TEXT,
  detail JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE INDEX IF NOT EXISTS idx_cloud_cron_jobs_actor
  ON cloud_cron_jobs(actor_storage_key, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_cloud_cron_job_runs_job_time
  ON cloud_cron_job_runs(job_id, executed_at DESC, run_id DESC);
CREATE INDEX IF NOT EXISTS idx_cloud_cron_job_runs_actor_time
  ON cloud_cron_job_runs(actor_channel, actor_user_id, executed_at DESC);
CREATE TABLE IF NOT EXISTS cloud_sessions (
  session_id TEXT PRIMARY KEY,
  actor_storage_key TEXT NOT NULL,
  version BIGINT NOT NULL DEFAULT 1,
  fencing_token BIGINT,
  content JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_web_invite_users (
  user_id TEXT PRIMARY KEY,
  phone_number TEXT,
  record JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_web_auth_sessions (
  session_hash TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  record JSONB NOT NULL,
  expires_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_llm_audit_records (
  id TEXT PRIMARY KEY,
  actor_storage_key TEXT,
  record JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO cloud_schema_migrations(version)
VALUES ('20260529_pg_oss_runtime_foundation')
ON CONFLICT (version) DO NOTHING;
"#,
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres schema 初始化失败: {err}")))?;
        Ok(())
    }

    pub async fn try_reserve_conversation_quota(
        &self,
        actor_storage_key: &str,
        quota_date: &str,
        daily_limit: u32,
    ) -> HoneResult<CloudConversationQuotaReserveOutcome> {
        let client = self.connect_client().await?;
        let daily_limit = i32::try_from(daily_limit)
            .map_err(|_| HoneError::Config("daily conversation limit exceeds i32".to_string()))?;
        let row = client
            .query_one(
                r#"
WITH inserted AS (
  INSERT INTO conversation_quota(actor_storage_key, quota_date, limit_count)
  VALUES ($1, $2::text::date, $3)
  ON CONFLICT (actor_storage_key, quota_date) DO NOTHING
),
updated AS (
  UPDATE conversation_quota
  SET
    reserved_count = reserved_count + 1,
    limit_count = $3,
    updated_at = now()
  WHERE actor_storage_key = $1
    AND quota_date = $2::text::date
    AND committed_count + reserved_count < $3
  RETURNING true AS reserved, quota_date::text, limit_count, reserved_count, committed_count
),
current_row AS (
  SELECT false AS reserved, quota_date::text, $3 AS limit_count, reserved_count, committed_count
  FROM conversation_quota
  WHERE actor_storage_key = $1
    AND quota_date = $2::text::date
    AND NOT EXISTS (SELECT 1 FROM updated)
  FOR UPDATE
)
SELECT reserved, quota_date, limit_count, reserved_count, committed_count FROM updated
UNION ALL
SELECT reserved, quota_date, limit_count, reserved_count, committed_count FROM current_row
LIMIT 1
"#,
                &[&actor_storage_key, &quota_date, &daily_limit],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres quota reserve 失败: {err}")))?;
        let reserved: bool = row.get(0);
        let quota_date: String = row.get(1);
        let limit_count: i32 = row.get(2);
        let reserved_count: i32 = row.get(3);
        let committed_count: i32 = row.get(4);
        Ok(CloudConversationQuotaReserveOutcome {
            reserved,
            snapshot: CloudConversationQuotaSnapshot {
                quota_date,
                success_count: committed_count.max(0) as u32,
                in_flight: reserved_count.max(0) as u32,
                limit: limit_count.max(0) as u32,
            },
        })
    }

    pub async fn finish_conversation_quota(
        &self,
        actor_storage_key: &str,
        quota_date: &str,
        committed: bool,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        let committed_increment = if committed { 1i32 } else { 0i32 };
        client
            .execute(
                r#"
UPDATE conversation_quota
SET
  reserved_count = GREATEST(reserved_count - 1, 0),
  committed_count = committed_count + $3,
  updated_at = now()
WHERE actor_storage_key = $1
  AND quota_date = $2::text::date
"#,
                &[&actor_storage_key, &quota_date, &committed_increment],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres quota finish 失败: {err}")))?;
        Ok(())
    }

    pub async fn conversation_quota_snapshot(
        &self,
        actor_storage_key: &str,
        quota_date: &str,
    ) -> HoneResult<Option<CloudConversationQuotaSnapshot>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                r#"
SELECT quota_date::text, limit_count, reserved_count, committed_count
FROM conversation_quota
WHERE actor_storage_key = $1
  AND quota_date = $2::text::date
"#,
                &[&actor_storage_key, &quota_date],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres quota snapshot 失败: {err}")))?;
        Ok(row.map(|row| {
            let quota_date: String = row.get(0);
            let limit_count: i32 = row.get(1);
            let reserved_count: i32 = row.get(2);
            let committed_count: i32 = row.get(3);
            CloudConversationQuotaSnapshot {
                quota_date,
                success_count: committed_count.max(0) as u32,
                in_flight: reserved_count.max(0) as u32,
                limit: limit_count.max(0) as u32,
            }
        }))
    }

    pub async fn import_conversation_quota(
        &self,
        snapshots: &[CloudConversationQuotaImport],
    ) -> HoneResult<CloudConversationQuotaImportReport> {
        if snapshots.is_empty() {
            return Ok(CloudConversationQuotaImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(snapshots)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT *
  FROM jsonb_to_recordset($1::jsonb) AS x(
    actor_storage_key TEXT,
    quota_date TEXT,
    success_count INTEGER,
    in_flight INTEGER,
    limit_count INTEGER
  )
),
upserted AS (
INSERT INTO conversation_quota(
  actor_storage_key,
  quota_date,
  limit_count,
  reserved_count,
  committed_count
)
SELECT
  actor_storage_key,
  quota_date::date,
  limit_count,
  in_flight,
  success_count
FROM input_rows
ON CONFLICT (actor_storage_key, quota_date)
DO UPDATE SET
  limit_count = GREATEST(conversation_quota.limit_count, EXCLUDED.limit_count),
  reserved_count = GREATEST(conversation_quota.reserved_count, EXCLUDED.reserved_count),
  committed_count = GREATEST(conversation_quota.committed_count, EXCLUDED.committed_count),
  updated_at = now()
WHERE conversation_quota.limit_count < EXCLUDED.limit_count
   OR conversation_quota.reserved_count < EXCLUDED.reserved_count
   OR conversation_quota.committed_count < EXCLUDED.committed_count
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted) AS changed_rows,
  (SELECT count(*)::bigint FROM input_rows) AS total_rows
"#,
                &[&payload],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres quota import 失败: {err}")))?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudConversationQuotaImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
    }

    pub async fn upsert_session_record(
        &self,
        session_id: &str,
        actor_storage_key: &str,
        content: serde_json::Value,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_sessions(session_id, actor_storage_key, content)
VALUES ($1, $2, $3)
ON CONFLICT (session_id)
DO UPDATE SET
  actor_storage_key = EXCLUDED.actor_storage_key,
  content = EXCLUDED.content,
  version = cloud_sessions.version + CASE WHEN cloud_sessions.content = EXCLUDED.content THEN 0 ELSE 1 END,
  updated_at = now()
"#,
                &[&session_id, &actor_storage_key, &content],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres session 写入失败: {err}")))?;
        Ok(())
    }

    pub async fn load_session_record(
        &self,
        session_id: &str,
    ) -> HoneResult<Option<serde_json::Value>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                "SELECT content FROM cloud_sessions WHERE session_id = $1",
                &[&session_id],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres session 读取失败: {err}")))?;
        Ok(row.map(|row| row.get(0)))
    }

    pub async fn list_session_records(&self) -> HoneResult<Vec<serde_json::Value>> {
        let client = self.connect_client().await?;
        let rows = client
            .query(
                "SELECT content FROM cloud_sessions ORDER BY updated_at DESC",
                &[],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres session 列表读取失败: {err}")))?;
        Ok(rows.into_iter().map(|row| row.get(0)).collect())
    }

    pub async fn import_session_records(
        &self,
        records: &[CloudSessionRecord],
    ) -> HoneResult<CloudSessionImportReport> {
        if records.is_empty() {
            return Ok(CloudSessionImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(records)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT *
  FROM jsonb_to_recordset($1::jsonb) AS x(
    session_id TEXT,
    actor_storage_key TEXT,
    content JSONB
  )
),
upserted AS (
INSERT INTO cloud_sessions(session_id, actor_storage_key, content)
SELECT session_id, actor_storage_key, content
FROM input_rows
ON CONFLICT (session_id)
DO UPDATE SET
  actor_storage_key = EXCLUDED.actor_storage_key,
  content = EXCLUDED.content,
  version = cloud_sessions.version + CASE WHEN cloud_sessions.content = EXCLUDED.content THEN 0 ELSE 1 END,
  updated_at = now()
WHERE cloud_sessions.actor_storage_key IS DISTINCT FROM EXCLUDED.actor_storage_key
   OR cloud_sessions.content IS DISTINCT FROM EXCLUDED.content
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted) AS changed_rows,
  (SELECT count(*)::bigint FROM input_rows) AS total_rows
"#,
                &[&payload],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres session import 失败: {err}")))?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudSessionImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
    }

    pub async fn upsert_web_invite_user_record(
        &self,
        user_id: &str,
        phone_number: &str,
        record: serde_json::Value,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_web_invite_users(user_id, phone_number, record)
VALUES ($1, $2, $3)
ON CONFLICT (user_id)
DO UPDATE SET
  phone_number = EXCLUDED.phone_number,
  record = EXCLUDED.record,
  updated_at = now()
"#,
                &[&user_id, &phone_number, &record],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web invite 写入失败: {err}")))?;
        Ok(())
    }

    pub async fn list_web_invite_user_records(&self) -> HoneResult<Vec<serde_json::Value>> {
        let client = self.connect_client().await?;
        let rows = client
            .query(
                "SELECT record FROM cloud_web_invite_users ORDER BY record->>'created_at' DESC",
                &[],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web invite 列表读取失败: {err}")))?;
        Ok(rows.into_iter().map(|row| row.get(0)).collect())
    }

    pub async fn find_web_invite_user_record(
        &self,
        field: &str,
        value: &str,
    ) -> HoneResult<Option<serde_json::Value>> {
        let client = self.connect_client().await?;
        let sql = match field {
            "user_id" => "SELECT record FROM cloud_web_invite_users WHERE user_id = $1",
            "invite_code" => {
                "SELECT record FROM cloud_web_invite_users WHERE record->>'invite_code' = $1"
            }
            "phone_number" => "SELECT record FROM cloud_web_invite_users WHERE phone_number = $1",
            "api_key_hash" => {
                "SELECT record FROM cloud_web_invite_users WHERE record->>'api_key_hash' = $1"
            }
            _ => {
                return Err(HoneError::Config(format!(
                    "unsupported web invite lookup field: {field}"
                )));
            }
        };
        let row = client
            .query_opt(sql, &[&value])
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web invite 读取失败: {err}")))?;
        Ok(row.map(|row| row.get(0)))
    }

    pub async fn delete_web_auth_sessions_for_user(&self, user_id: &str) -> HoneResult<u64> {
        let client = self.connect_client().await?;
        client
            .execute(
                "DELETE FROM cloud_web_auth_sessions WHERE user_id = $1",
                &[&user_id],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web auth session 删除失败: {err}")))
    }

    pub async fn upsert_web_auth_session_record(
        &self,
        session_hash: &str,
        user_id: &str,
        record: serde_json::Value,
        expires_at: Option<&str>,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_web_auth_sessions(session_hash, user_id, record, expires_at)
VALUES ($1, $2, $3, $4::text::timestamptz)
ON CONFLICT (session_hash)
DO UPDATE SET
  user_id = EXCLUDED.user_id,
  record = EXCLUDED.record,
  expires_at = EXCLUDED.expires_at,
  updated_at = now()
"#,
                &[&session_hash, &user_id, &record, &expires_at],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres web auth session 写入失败: {err}"))
            })?;
        Ok(())
    }

    pub async fn find_web_auth_session_record(
        &self,
        session_hash: &str,
        legacy_token: &str,
    ) -> HoneResult<Option<serde_json::Value>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                "SELECT record FROM cloud_web_auth_sessions WHERE session_hash = $1 OR session_hash = $2",
                &[&session_hash, &legacy_token],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web auth session 读取失败: {err}")))?;
        Ok(row.map(|row| row.get(0)))
    }

    pub async fn delete_web_auth_session(
        &self,
        session_hash: &str,
        legacy_token: &str,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                "DELETE FROM cloud_web_auth_sessions WHERE session_hash = $1 OR session_hash = $2",
                &[&session_hash, &legacy_token],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres web auth session 删除失败: {err}"))
            })?;
        Ok(())
    }

    pub async fn purge_expired_web_auth_sessions(&self, now: &str) -> HoneResult<u64> {
        let client = self.connect_client().await?;
        client
            .execute(
                "DELETE FROM cloud_web_auth_sessions WHERE record->>'expires_at' <= $1",
                &[&now],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web auth session 清理失败: {err}")))
    }

    pub async fn count_active_web_auth_sessions(
        &self,
        user_id: &str,
        now: &str,
    ) -> HoneResult<u32> {
        let client = self.connect_client().await?;
        let row = client
            .query_one(
                "SELECT count(*)::bigint FROM cloud_web_auth_sessions WHERE user_id = $1 AND record->>'expires_at' > $2",
                &[&user_id, &now],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web auth session 计数失败: {err}")))?;
        let count = row.get::<_, i64>(0).max(0) as u32;
        Ok(count)
    }

    pub async fn import_web_auth_records(
        &self,
        users: &[CloudWebInviteUserRecord],
        sessions: &[CloudWebAuthSessionRecord],
    ) -> HoneResult<CloudWebAuthImportReport> {
        let client = self.connect_client().await?;
        let user_payload =
            serde_json::to_value(users).map_err(|err| HoneError::Serialization(err.to_string()))?;
        let session_payload = serde_json::to_value(sessions)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let user_row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT *
  FROM jsonb_to_recordset($1::jsonb) AS x(
    user_id TEXT,
    phone_number TEXT,
    record JSONB
  )
),
upserted AS (
INSERT INTO cloud_web_invite_users(user_id, phone_number, record)
SELECT user_id, phone_number, record FROM input_rows
ON CONFLICT (user_id)
DO UPDATE SET
  phone_number = EXCLUDED.phone_number,
  record = EXCLUDED.record,
  updated_at = now()
WHERE cloud_web_invite_users.phone_number IS DISTINCT FROM EXCLUDED.phone_number
   OR cloud_web_invite_users.record IS DISTINCT FROM EXCLUDED.record
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&user_payload],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres web invite import 失败: {err}")))?;
        let session_row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT *
  FROM jsonb_to_recordset($1::jsonb) AS x(
    session_hash TEXT,
    user_id TEXT,
    expires_at TEXT,
    record JSONB
  )
),
upserted AS (
INSERT INTO cloud_web_auth_sessions(session_hash, user_id, expires_at, record)
SELECT session_hash, user_id, expires_at::timestamptz, record FROM input_rows
ON CONFLICT (session_hash)
DO UPDATE SET
  user_id = EXCLUDED.user_id,
  expires_at = EXCLUDED.expires_at,
  record = EXCLUDED.record,
  updated_at = now()
WHERE cloud_web_auth_sessions.user_id IS DISTINCT FROM EXCLUDED.user_id
   OR cloud_web_auth_sessions.expires_at IS DISTINCT FROM EXCLUDED.expires_at
   OR cloud_web_auth_sessions.record IS DISTINCT FROM EXCLUDED.record
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&session_payload],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres web auth session import 失败: {err}"))
            })?;
        let changed_users = user_row.get::<_, i64>(0).max(0) as usize;
        let total_users = user_row.get::<_, i64>(1).max(0) as usize;
        let changed_sessions = session_row.get::<_, i64>(0).max(0) as usize;
        let total_sessions = session_row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudWebAuthImportReport {
            changed_users,
            skipped_users: total_users.saturating_sub(changed_users),
            changed_sessions,
            skipped_sessions: total_sessions.saturating_sub(changed_sessions),
        })
    }

    pub async fn list_cron_job_records(&self) -> HoneResult<Vec<CloudCronJobRecord>> {
        let client = self.connect_client().await?;
        let rows = client
            .query(
                r#"
SELECT actor_storage_key, job_id, actor, job
FROM cloud_cron_jobs
ORDER BY updated_at DESC
"#,
                &[],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron 列表读取失败: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| CloudCronJobRecord {
                actor_storage_key: row.get(0),
                job_id: row.get(1),
                actor: row.get(2),
                job: row.get(3),
            })
            .collect())
    }

    pub async fn list_cron_job_records_for_actor(
        &self,
        actor_storage_key: &str,
    ) -> HoneResult<Vec<CloudCronJobRecord>> {
        let client = self.connect_client().await?;
        let rows = client
            .query(
                r#"
SELECT actor_storage_key, job_id, actor, job
FROM cloud_cron_jobs
WHERE actor_storage_key = $1
ORDER BY updated_at DESC
"#,
                &[&actor_storage_key],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron actor 列表读取失败: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| CloudCronJobRecord {
                actor_storage_key: row.get(0),
                job_id: row.get(1),
                actor: row.get(2),
                job: row.get(3),
            })
            .collect())
    }

    pub async fn upsert_cron_job_record(
        &self,
        actor_storage_key: &str,
        job_id: &str,
        actor: serde_json::Value,
        job: serde_json::Value,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_cron_jobs(actor_storage_key, job_id, actor, job)
VALUES ($1, $2, $3, $4)
ON CONFLICT (actor_storage_key, job_id)
DO UPDATE SET
  actor = EXCLUDED.actor,
  job = EXCLUDED.job,
  updated_at = now()
"#,
                &[&actor_storage_key, &job_id, &actor, &job],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron 写入失败: {err}")))?;
        Ok(())
    }

    pub async fn delete_cron_job_record(
        &self,
        actor_storage_key: &str,
        job_id: &str,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                "DELETE FROM cloud_cron_jobs WHERE actor_storage_key = $1 AND job_id = $2",
                &[&actor_storage_key, &job_id],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron 删除失败: {err}")))?;
        Ok(())
    }

    pub async fn import_cron_job_records(
        &self,
        records: &[CloudCronJobRecord],
    ) -> HoneResult<CloudCronJobImportReport> {
        if records.is_empty() {
            return Ok(CloudCronJobImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(records)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT *
  FROM jsonb_to_recordset($1::jsonb) AS x(
    actor_storage_key TEXT,
    job_id TEXT,
    actor JSONB,
    job JSONB
  )
),
upserted AS (
INSERT INTO cloud_cron_jobs(actor_storage_key, job_id, actor, job)
SELECT actor_storage_key, job_id, actor, job FROM input_rows
ON CONFLICT (actor_storage_key, job_id)
DO UPDATE SET
  actor = EXCLUDED.actor,
  job = EXCLUDED.job,
  updated_at = now()
WHERE cloud_cron_jobs.actor IS DISTINCT FROM EXCLUDED.actor
   OR cloud_cron_jobs.job IS DISTINCT FROM EXCLUDED.job
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&payload],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron import 失败: {err}")))?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudCronJobImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
    }

    pub async fn try_claim_cron_due_job(
        &self,
        job_key: &str,
        due_key: &str,
        owner_id: &str,
    ) -> HoneResult<bool> {
        let client = self.connect_client().await?;
        let rows = client
            .execute(
                r#"
INSERT INTO cloud_cron_job_claims(job_key, due_key, owner_id)
VALUES ($1, $2, $3)
ON CONFLICT (job_key, due_key) DO NOTHING
"#,
                &[&job_key, &due_key, &owner_id],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron claim 失败: {err}")))?;
        Ok(rows > 0)
    }

    pub async fn record_cron_execution_event(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
        job_name: &str,
        channel_target: &str,
        heartbeat: bool,
        input: CloudCronExecutionInput,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        let executed_at = crate::beijing_now_rfc3339();
        let started_threshold = (crate::beijing_now() - chrono::Duration::hours(2)).to_rfc3339();
        let response_preview = input.response_preview;
        let error_message = input.error_message;
        if input.execution_status != "running" && input.message_send_status != "pending" {
            if let Some(delivery_key) = input
                .detail
                .get("delivery_key")
                .and_then(|value| value.as_str())
                && !delivery_key.trim().is_empty()
            {
                let updated = client
                    .execute(
                        r#"
UPDATE cloud_cron_job_runs
SET
  executed_at = $1,
  execution_status = $2,
  message_send_status = $3,
  should_deliver = $4,
  delivered = $5,
  response_preview = $6,
  error_message = $7,
  detail = $8
WHERE run_id = (
  SELECT run_id
  FROM cloud_cron_job_runs
  WHERE job_id = $9
    AND actor_channel = $10
    AND actor_user_id = $11
    AND COALESCE(actor_channel_scope, '') = COALESCE($12, '')
    AND channel_target = $13
    AND heartbeat = $14
    AND execution_status = 'running'
    AND message_send_status = 'pending'
    AND detail->>'delivery_key' = $15
  ORDER BY executed_at DESC, run_id DESC
  LIMIT 1
)
"#,
                        &[
                            &executed_at,
                            &input.execution_status,
                            &input.message_send_status,
                            &input.should_deliver,
                            &input.delivered,
                            &response_preview,
                            &error_message,
                            &input.detail,
                            &job_id,
                            &actor.channel,
                            &actor.user_id,
                            &actor.channel_scope,
                            &channel_target,
                            &heartbeat,
                            &delivery_key.trim(),
                        ],
                    )
                    .await
                    .map_err(|err| {
                        HoneError::Config(format!("Postgres cron 执行记录更新失败: {err}"))
                    })?;
                if updated > 0 {
                    return Ok(());
                }
            }
            let updated = client
                .execute(
                    r#"
UPDATE cloud_cron_job_runs
SET
  executed_at = $1,
  execution_status = $2,
  message_send_status = $3,
  should_deliver = $4,
  delivered = $5,
  response_preview = $6,
  error_message = $7,
  detail = $8
WHERE run_id = (
  SELECT run_id
  FROM cloud_cron_job_runs
  WHERE job_id = $9
    AND actor_channel = $10
    AND actor_user_id = $11
    AND COALESCE(actor_channel_scope, '') = COALESCE($12, '')
    AND channel_target = $13
    AND heartbeat = $14
    AND execution_status = 'running'
    AND message_send_status = 'pending'
    AND detail->>'phase' = 'started'
    AND executed_at >= $15
  ORDER BY executed_at DESC, run_id DESC
  LIMIT 1
)
"#,
                    &[
                        &executed_at,
                        &input.execution_status,
                        &input.message_send_status,
                        &input.should_deliver,
                        &input.delivered,
                        &response_preview,
                        &error_message,
                        &input.detail,
                        &job_id,
                        &actor.channel,
                        &actor.user_id,
                        &actor.channel_scope,
                        &channel_target,
                        &heartbeat,
                        &started_threshold,
                    ],
                )
                .await
                .map_err(|err| {
                    HoneError::Config(format!("Postgres cron 执行记录更新失败: {err}"))
                })?;
            if updated > 0 {
                return Ok(());
            }
        }
        client
            .execute(
                r#"
INSERT INTO cloud_cron_job_runs (
  job_id, job_name,
  actor_channel, actor_user_id, actor_channel_scope,
  channel_target, heartbeat,
  executed_at, execution_status, message_send_status,
  should_deliver, delivered, response_preview, error_message, detail
) VALUES (
  $1, $2,
  $3, $4, $5,
  $6, $7,
  $8, $9, $10,
  $11, $12, $13, $14, $15
)
"#,
                &[
                    &job_id,
                    &job_name,
                    &actor.channel,
                    &actor.user_id,
                    &actor.channel_scope,
                    &channel_target,
                    &heartbeat,
                    &executed_at,
                    &input.execution_status,
                    &input.message_send_status,
                    &input.should_deliver,
                    &input.delivered,
                    &response_preview,
                    &error_message,
                    &input.detail,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron 执行记录写入失败: {err}")))?;
        Ok(())
    }

    pub async fn recover_stale_cron_started_executions(
        &self,
        channel: &str,
        stale_before_rfc3339: &str,
        recovered_by: &str,
        reason: &str,
    ) -> HoneResult<usize> {
        let client = self.connect_client().await?;
        let interrupted_at = crate::beijing_now_rfc3339();
        let detail = serde_json::json!({
            "phase": "recovered_stale_pending",
            "recovered_at": interrupted_at,
            "recovered_by": recovered_by,
        });
        let updated = client
            .execute(
                r#"
UPDATE cloud_cron_job_runs
SET
  executed_at = $1,
  execution_status = 'execution_failed',
  message_send_status = 'send_failed',
  should_deliver = false,
  delivered = false,
  response_preview = NULL,
  error_message = $2,
  detail = detail || $3::jsonb
WHERE actor_channel = $4
  AND execution_status = 'running'
  AND message_send_status = 'pending'
  AND detail->>'phase' = 'started'
  AND executed_at < $5
"#,
                &[
                    &interrupted_at,
                    &reason,
                    &detail,
                    &channel,
                    &stale_before_rfc3339,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron stale 恢复失败: {err}")))?;
        Ok(updated as usize)
    }

    pub async fn list_cron_execution_records(
        &self,
        filter: CloudCronExecutionFilter,
    ) -> HoneResult<Vec<CloudCronExecutionRecord>> {
        let client = self.connect_client().await?;
        let limit = i64::try_from(filter.limit.max(1)).unwrap_or(1000);
        let rows = client
            .query(
                r#"
SELECT
  run_id, job_id, job_name,
  actor_channel, actor_user_id, actor_channel_scope,
  channel_target, heartbeat,
  executed_at, execution_status, message_send_status,
  should_deliver, delivered, response_preview, error_message, detail
FROM cloud_cron_job_runs
WHERE ($1::text IS NULL OR executed_at >= $1)
  AND ($2::text IS NULL OR executed_at <= $2)
  AND ($3::text IS NULL OR actor_channel = $3)
  AND ($4::text IS NULL OR actor_user_id = $4)
  AND ($5::text IS NULL OR job_id = $5)
  AND ($6::text IS NULL OR execution_status = $6)
  AND ($7::text IS NULL OR message_send_status = $7)
  AND ($8::boolean IS NULL OR heartbeat = $8)
ORDER BY executed_at DESC, run_id DESC
LIMIT $9
"#,
                &[
                    &filter.since,
                    &filter.until,
                    &filter.channel,
                    &filter.user_id,
                    &filter.job_id,
                    &filter.execution_status,
                    &filter.message_send_status,
                    &filter.heartbeat_only,
                    &limit,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres cron 执行记录读取失败: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| CloudCronExecutionRecord {
                run_id: row.get(0),
                job_id: row.get(1),
                job_name: row.get(2),
                channel: row.get(3),
                user_id: row.get(4),
                channel_scope: row.get(5),
                channel_target: row.get(6),
                heartbeat: row.get(7),
                executed_at: row.get(8),
                execution_status: row.get(9),
                message_send_status: row.get(10),
                should_deliver: row.get(11),
                delivered: row.get(12),
                response_preview: row.get(13),
                error_message: row.get(14),
                detail: row.get(15),
            })
            .collect())
    }

    pub async fn upsert_document_index(
        &self,
        actor_storage_key: &str,
        kind: &str,
        document_id: &str,
        oss_uri: &str,
        sha256: &str,
        size_bytes: i64,
        metadata: serde_json::Value,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_documents(actor_storage_key, kind, document_id, oss_uri, sha256, size_bytes, metadata)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (actor_storage_key, kind, document_id)
DO UPDATE SET
  oss_uri = EXCLUDED.oss_uri,
  sha256 = EXCLUDED.sha256,
  size_bytes = EXCLUDED.size_bytes,
  metadata = EXCLUDED.metadata,
  version = cloud_documents.version + CASE WHEN cloud_documents.sha256 = EXCLUDED.sha256 THEN 0 ELSE 1 END,
  updated_at = now()
"#,
                &[
                    &actor_storage_key,
                    &kind,
                    &document_id,
                    &oss_uri,
                    &sha256,
                    &size_bytes,
                    &metadata,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres document index 写入失败: {err}")))?;
        Ok(())
    }

    pub async fn upsert_document_indexes(&self, records: &[CloudDocumentIndex]) -> HoneResult<()> {
        if records.is_empty() {
            return Ok(());
        }
        let client = self.connect_client().await?;
        let statement = client
            .prepare(
                r#"
INSERT INTO cloud_documents(actor_storage_key, kind, document_id, oss_uri, sha256, size_bytes, metadata)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (actor_storage_key, kind, document_id)
DO UPDATE SET
  oss_uri = EXCLUDED.oss_uri,
  sha256 = EXCLUDED.sha256,
  size_bytes = EXCLUDED.size_bytes,
  metadata = EXCLUDED.metadata,
  version = cloud_documents.version + CASE WHEN cloud_documents.sha256 = EXCLUDED.sha256 THEN 0 ELSE 1 END,
  updated_at = now()
"#,
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres document index prepare 失败: {err}")))?;
        for record in records {
            client
                .execute(
                    &statement,
                    &[
                        &record.actor_storage_key,
                        &record.kind,
                        &record.document_id,
                        &record.oss_uri,
                        &record.sha256,
                        &record.size_bytes,
                        &record.metadata,
                    ],
                )
                .await
                .map_err(|err| {
                    HoneError::Config(format!(
                        "Postgres document index 写入失败 kind={} document_id={}: {err}",
                        record.kind, record.document_id
                    ))
                })?;
        }
        Ok(())
    }
}

async fn connect_via_proxy(
    proxy: &str,
    target_host: &str,
    target_port: u16,
) -> HoneResult<TcpStream> {
    let url = Url::parse(proxy)
        .map_err(|err| HoneError::Config(format!("HONE_POSTGRES_PROXY 无效: {err}")))?;
    let proxy_host = url
        .host_str()
        .ok_or_else(|| HoneError::Config("HONE_POSTGRES_PROXY 缺少 host".to_string()))?;
    let proxy_port = url
        .port_or_known_default()
        .ok_or_else(|| HoneError::Config("HONE_POSTGRES_PROXY 缺少 port".to_string()))?;
    let mut stream = TcpStream::connect((proxy_host, proxy_port))
        .await
        .map_err(|err| HoneError::Config(format!("连接 Postgres proxy 失败: {err}")))?;
    match url.scheme() {
        "socks5" | "socks5h" => {
            stream.write_all(&[0x05, 0x01, 0x00]).await?;
            let mut resp = [0u8; 2];
            stream.read_exact(&mut resp).await?;
            if resp != [0x05, 0x00] {
                return Err(HoneError::Config("SOCKS5 proxy 不支持 no-auth".to_string()));
            }
            let host = target_host.as_bytes();
            if host.len() > u8::MAX as usize {
                return Err(HoneError::Config("Postgres host 过长".to_string()));
            }
            let mut req = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
            req.extend_from_slice(host);
            req.extend_from_slice(&target_port.to_be_bytes());
            stream.write_all(&req).await?;
            let mut head = [0u8; 4];
            stream.read_exact(&mut head).await?;
            if head[1] != 0x00 {
                return Err(HoneError::Config(format!(
                    "SOCKS5 connect 失败: reply={}",
                    head[1]
                )));
            }
            match head[3] {
                0x01 => {
                    let mut skip = [0u8; 6];
                    stream.read_exact(&mut skip).await?;
                }
                0x03 => {
                    let mut len = [0u8; 1];
                    stream.read_exact(&mut len).await?;
                    let mut skip = vec![0u8; len[0] as usize + 2];
                    stream.read_exact(&mut skip).await?;
                }
                0x04 => {
                    let mut skip = [0u8; 18];
                    stream.read_exact(&mut skip).await?;
                }
                _ => return Err(HoneError::Config("SOCKS5 地址类型无效".to_string())),
            }
        }
        "http" | "https" => {
            let request = format!(
                "CONNECT {target_host}:{target_port} HTTP/1.1\r\nHost: {target_host}:{target_port}\r\n\r\n"
            );
            stream.write_all(request.as_bytes()).await?;
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await?;
            let text = String::from_utf8_lossy(&buf[..n]);
            if !text.starts_with("HTTP/1.1 200") && !text.starts_with("HTTP/1.0 200") {
                return Err(HoneError::Config(format!(
                    "HTTP CONNECT proxy 失败: {}",
                    text.lines().next().unwrap_or("empty response")
                )));
            }
        }
        other => {
            return Err(HoneError::Config(format!(
                "不支持的 Postgres proxy scheme: {other}"
            )));
        }
    }
    Ok(stream)
}

#[derive(Debug, Clone)]
pub struct OssObjectStore {
    provider: ObjectStoreProvider,
    access_key_id: String,
    access_key_secret: String,
    bucket: String,
    endpoint: String,
    region: String,
    public_upload_prefix: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectStoreProvider {
    AliyunOss,
    S3,
}

impl ObjectStoreProvider {
    fn from_raw(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "r2" | "cloudflare_r2" | "s3" | "s3_compatible" => Self::S3,
            _ => Self::AliyunOss,
        }
    }

    fn service_name(&self) -> &'static str {
        match self {
            Self::AliyunOss => "OSS",
            Self::S3 => "S3",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OssObject {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

impl OssObjectStore {
    pub fn from_config(config: &OssConfig) -> Option<Self> {
        if !config.is_configured() {
            return None;
        }
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .pool_idle_timeout(Duration::from_secs(60));
        let proxy = config.resolved_proxy();
        if !proxy.trim().is_empty()
            && let Ok(req_proxy) = reqwest::Proxy::all(proxy.trim())
        {
            builder = builder.proxy(req_proxy);
        }
        Some(Self {
            provider: ObjectStoreProvider::from_raw(&config.resolved_provider()),
            access_key_id: config.resolved_access_key_id(),
            access_key_secret: config.resolved_access_key_secret(),
            bucket: config.resolved_bucket(),
            endpoint: config.resolved_endpoint(),
            region: {
                let region = config.resolved_region();
                if region.trim().is_empty() {
                    "auto".to_string()
                } else {
                    region
                }
            },
            public_upload_prefix: sanitize_prefix(&config.public_upload_prefix),
            client: builder.build().ok()?,
        })
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn object_uri(&self, key: &str) -> String {
        format!("oss://{}/{}", self.bucket, key.trim_start_matches('/'))
    }

    pub fn actor_prefix(&self, actor: &ActorIdentity) -> String {
        format!("users/{}/", sanitize_key_component(&actor.storage_key()))
    }

    pub fn actor_upload_key(
        &self,
        actor: &ActorIdentity,
        session_id: &str,
        stored_name: &str,
    ) -> String {
        format!(
            "{}uploads/{}/{}",
            self.actor_prefix(actor),
            sanitize_key_component(session_id),
            sanitize_key_component(stored_name)
        )
    }

    pub fn actor_document_key(&self, actor: &ActorIdentity, kind: &str, id: &str) -> String {
        format!(
            "{}documents/{}/{}",
            self.actor_prefix(actor),
            sanitize_key_component(kind),
            sanitize_key_component(id)
        )
    }

    pub fn public_upload_key(&self, user_id: &str, day: &str, stored_name: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            self.public_upload_prefix,
            sanitize_key_component(user_id),
            sanitize_key_component(day),
            sanitize_key_component(stored_name)
        )
    }

    pub fn is_public_upload_uri_for_user(&self, raw: &str, user_id: &str) -> bool {
        let Some((bucket, key)) = parse_oss_uri(raw) else {
            return false;
        };
        bucket == self.bucket
            && key.starts_with(&format!(
                "{}/{}/",
                self.public_upload_prefix,
                sanitize_key_component(user_id)
            ))
    }

    pub fn parse_managed_uri<'a>(&self, raw: &'a str) -> Option<&'a str> {
        let (bucket, key) = parse_oss_uri(raw)?;
        (bucket == self.bucket).then_some(key)
    }

    pub async fn health(&self) -> CloudHealth {
        match tokio::time::timeout(Duration::from_secs(5), self.list_objects("", 1)).await {
            Ok(Ok(_)) => CloudHealth {
                ok: true,
                detail: format!("{} connected", self.provider.service_name()),
            },
            Ok(Err(error)) => CloudHealth {
                ok: false,
                detail: error,
            },
            Err(_) => CloudHealth {
                ok: false,
                detail: "oss health timeout".to_string(),
            },
        }
    }

    pub async fn put_object(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        let date = oss_date();
        let mut headers = HeaderMap::new();
        self.insert_auth_headers(&mut headers, "PUT", content_type, &date, key, None, &bytes)?;
        headers.insert(CONTENT_TYPE, header_value(content_type)?);

        let response = self
            .client
            .put(self.object_url(key))
            .headers(headers)
            .body(bytes)
            .send()
            .await
            .map_err(|error| format!("OSS 上传请求失败: {error}"))?;
        if response.status().is_success() {
            return Ok(());
        }
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("OSS 上传失败: {status} {body}"))
    }

    pub async fn get_object(&self, key: &str) -> Result<OssObject, String> {
        let date = oss_date();
        let mut headers = HeaderMap::new();
        self.insert_auth_headers(&mut headers, "GET", "", &date, key, None, &[])?;

        let response = self
            .client
            .get(self.object_url(key))
            .headers(headers)
            .send()
            .await
            .map_err(|error| format!("OSS 读取请求失败: {error}"))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OSS 读取失败: {status} {body}"));
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("OSS 响应读取失败: {error}"))?
            .to_vec();
        Ok(OssObject {
            bytes,
            content_type,
        })
    }

    pub async fn object_exists(&self, key: &str) -> Result<bool, String> {
        let date = oss_date();
        let mut headers = HeaderMap::new();
        self.insert_auth_headers(&mut headers, "HEAD", "", &date, key, None, &[])?;
        let response = self
            .client
            .head(self.object_url(key))
            .headers(headers)
            .send()
            .await
            .map_err(|error| format!("OSS HEAD 请求失败: {error}"))?;
        if response.status().is_success() {
            return Ok(true);
        }
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(false);
        }
        let status = response.status();
        Err(format!("OSS HEAD 失败: {status}"))
    }

    pub async fn list_objects(&self, prefix: &str, max_keys: usize) -> Result<Vec<String>, String> {
        let prefix = prefix.trim_start_matches('/');
        let query = match self.provider {
            ObjectStoreProvider::AliyunOss => BTreeMap::from([
                ("max-keys".to_string(), max_keys.max(1).to_string()),
                ("prefix".to_string(), prefix.to_string()),
            ]),
            ObjectStoreProvider::S3 => BTreeMap::from([
                ("list-type".to_string(), "2".to_string()),
                ("max-keys".to_string(), max_keys.max(1).to_string()),
                ("prefix".to_string(), prefix.to_string()),
            ]),
        };
        let date = oss_date();
        let mut headers = HeaderMap::new();
        let signed_query = if self.provider == ObjectStoreProvider::AliyunOss {
            None
        } else {
            Some(&query)
        };
        self.insert_auth_headers(&mut headers, "GET", "", &date, "", signed_query, &[])?;
        let response = self
            .client
            .get(self.bucket_url())
            .query(&query)
            .headers(headers)
            .send()
            .await
            .map_err(|error| format!("OSS 列表请求失败: {error}"))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OSS 列表失败: {status} {body}"));
        }
        let text = response
            .text()
            .await
            .map_err(|error| format!("OSS 列表响应读取失败: {error}"))?;
        Ok(text
            .split("<Key>")
            .skip(1)
            .filter_map(|chunk| chunk.split_once("</Key>").map(|(key, _)| key.to_string()))
            .collect())
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), String> {
        let date = oss_date();
        let mut headers = HeaderMap::new();
        self.insert_auth_headers(&mut headers, "DELETE", "", &date, key, None, &[])?;
        let response = self
            .client
            .delete(self.object_url(key))
            .headers(headers)
            .send()
            .await
            .map_err(|error| format!("{} 删除请求失败: {error}", self.provider.service_name()))?;
        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!(
            "{} 删除失败: {status} {body}",
            self.provider.service_name()
        ))
    }

    fn insert_auth_headers(
        &self,
        headers: &mut HeaderMap,
        method: &str,
        content_type: &str,
        date: &str,
        key: &str,
        query: Option<&BTreeMap<String, String>>,
        body: &[u8],
    ) -> Result<(), String> {
        match self.provider {
            ObjectStoreProvider::AliyunOss => {
                let authorization =
                    self.aliyun_authorization(method, content_type, date, key, query)?;
                headers.insert(DATE, header_value(date)?);
                headers.insert(AUTHORIZATION, header_value(&authorization)?);
            }
            ObjectStoreProvider::S3 => {
                self.insert_s3_auth_headers(headers, method, content_type, date, key, query, body)?;
            }
        }
        Ok(())
    }

    fn aliyun_authorization(
        &self,
        method: &str,
        content_type: &str,
        date: &str,
        key: &str,
        query: Option<&BTreeMap<String, String>>,
    ) -> Result<String, String> {
        let mut canonical_resource = if key.trim_start_matches('/').is_empty() {
            format!("/{}/", self.bucket)
        } else {
            format!("/{}/{}", self.bucket, key.trim_start_matches('/'))
        };
        if let Some(query) = query
            && !query.is_empty()
        {
            canonical_resource.push('?');
            canonical_resource.push_str(
                &query
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
        let string_to_sign = format!("{method}\n\n{content_type}\n{date}\n{canonical_resource}");
        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes())
            .map_err(|error| format!("OSS 签名初始化失败: {error}"))?;
        mac.update(string_to_sign.as_bytes());
        let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());
        Ok(format!("OSS {}:{signature}", self.access_key_id))
    }

    fn insert_s3_auth_headers(
        &self,
        headers: &mut HeaderMap,
        method: &str,
        content_type: &str,
        date: &str,
        key: &str,
        query: Option<&BTreeMap<String, String>>,
        body: &[u8],
    ) -> Result<(), String> {
        let amz_date = s3_amz_date_from_oss_date(date)?;
        let date_scope = &amz_date[..8];
        let payload_hash = sha256_hex(body);
        let host = self.request_host()?;
        headers.insert(HOST, header_value(&host)?);
        headers.insert(
            header_name("x-amz-content-sha256")?,
            header_value(&payload_hash)?,
        );
        headers.insert(header_name("x-amz-date")?, header_value(&amz_date)?);
        if !content_type.is_empty() {
            headers.insert(CONTENT_TYPE, header_value(content_type)?);
        }

        let canonical_uri = self.s3_canonical_uri(key);
        let canonical_query = query.map(s3_canonical_query).unwrap_or_default();
        let signed_headers = if content_type.is_empty() {
            "host;x-amz-content-sha256;x-amz-date"
        } else {
            "content-type;host;x-amz-content-sha256;x-amz-date"
        };
        let canonical_headers = if content_type.is_empty() {
            format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n")
        } else {
            format!(
                "content-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
            )
        };
        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let credential_scope = format!("{date_scope}/{}/s3/aws4_request", self.s3_region());
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );
        let signing_key = s3_signing_key(&self.access_key_secret, date_scope, self.s3_region());
        let signature = hmac_sha256_hex(&signing_key, string_to_sign.as_bytes())?;
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key_id
        );
        headers.insert(AUTHORIZATION, header_value(&authorization)?);
        Ok(())
    }

    fn bucket_url(&self) -> String {
        match self.provider {
            ObjectStoreProvider::AliyunOss => bucket_host(&self.endpoint, &self.bucket),
            ObjectStoreProvider::S3 => format!(
                "{}/{}",
                self.endpoint.trim_end_matches('/'),
                s3_uri_encode_segment(&self.bucket)
            ),
        }
    }

    fn object_url(&self, key: &str) -> String {
        match self.provider {
            ObjectStoreProvider::AliyunOss => format!("{}/{}", self.bucket_url(), encode_key(key)),
            ObjectStoreProvider::S3 => {
                format!("{}/{}", self.bucket_url(), encode_key_s3(key))
            }
        }
    }

    fn request_host(&self) -> Result<String, String> {
        let parsed = url::Url::parse(&self.bucket_url())
            .map_err(|error| format!("object store endpoint 无效: {error}"))?;
        Ok(match parsed.port() {
            Some(port) => format!("{}:{port}", parsed.host_str().unwrap_or_default()),
            None => parsed.host_str().unwrap_or_default().to_string(),
        })
    }

    fn s3_canonical_uri(&self, key: &str) -> String {
        if key.trim_start_matches('/').is_empty() {
            format!("/{}", s3_uri_encode_segment(&self.bucket))
        } else {
            format!(
                "/{}/{}",
                s3_uri_encode_segment(&self.bucket),
                encode_key_s3(key)
            )
        }
    }

    fn s3_region(&self) -> &str {
        if self.region.trim().is_empty() {
            "auto"
        } else {
            self.region.trim()
        }
    }
}

pub fn parse_oss_uri(raw: &str) -> Option<(&str, &str)> {
    let value = raw.trim();
    let rest = value.strip_prefix("oss://")?;
    let (bucket, key) = rest.split_once('/')?;
    if bucket.is_empty() || key.trim_matches('/').is_empty() {
        return None;
    }
    Some((bucket, key.trim_start_matches('/')))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn local_durable_dependencies(config: &HoneConfig) -> Vec<String> {
    if !config.cloud.effective_mode().is_cloud_authoritative() {
        return Vec::new();
    }
    let mut deps = vec![
        config.storage.llm_audit_db_path.clone(),
        config.storage.portfolio_dir.clone(),
        config.storage.gen_images_dir.clone(),
        config.storage.notif_prefs_dir.clone(),
        "./data/runtime/skill_registry.json".to_string(),
        "./data/agent-sandboxes".to_string(),
    ];
    if !config.cloud.postgres.is_configured() {
        deps.push(config.storage.cron_jobs_dir.clone());
        deps.push(config.storage.sessions_dir.clone());
        deps.push(config.storage.session_sqlite_db_path.clone());
        deps.push(config.storage.conversation_quota_dir.clone());
    }
    deps.sort();
    deps.dedup();
    deps
}

pub fn load_dotenv_if_present() {
    let path = PathBuf::from(".env");
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let override_existing = text.lines().any(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("HONE_DOTENV_OVERRIDE=")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    });
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || (!override_existing && std::env::var_os(key).is_some()) {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        unsafe {
            std::env::set_var(key, value);
        }
    }
}

fn bucket_host(endpoint: &str, bucket: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if let Some(rest) = endpoint.strip_prefix("https://") {
        format!("https://{bucket}.{rest}")
    } else if let Some(rest) = endpoint.strip_prefix("http://") {
        format!("http://{bucket}.{rest}")
    } else {
        format!("https://{bucket}.{endpoint}")
    }
}

fn oss_date() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn header_value(value: &str) -> Result<HeaderValue, String> {
    HeaderValue::from_str(value).map_err(|error| format!("object store header 无效: {error}"))
}

fn header_name(value: &'static str) -> Result<HeaderName, String> {
    HeaderName::from_lowercase(value.as_bytes())
        .map_err(|error| format!("object store header name 无效: {error}"))
}

fn encode_key(key: &str) -> String {
    key.trim_start_matches('/')
        .split('/')
        .map(|segment| utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn encode_key_s3(key: &str) -> String {
    key.trim_start_matches('/')
        .split('/')
        .map(s3_uri_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn s3_uri_encode_segment(raw: &str) -> String {
    let mut out = String::new();
    for byte in raw.as_bytes() {
        let ch = *byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            out.push(ch);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn s3_canonical_query(query: &BTreeMap<String, String>) -> String {
    query
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                s3_uri_encode_segment(key),
                s3_uri_encode_segment(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn s3_amz_date_from_oss_date(date: &str) -> Result<String, String> {
    let parsed = chrono::DateTime::parse_from_rfc2822(date)
        .map_err(|error| format!("object store date parse failed: {error}"))?;
    Ok(parsed
        .with_timezone(&Utc)
        .format("%Y%m%dT%H%M%SZ")
        .to_string())
}

fn hmac_sha256_bytes(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let mut mac =
        HmacSha256::new_from_slice(key).map_err(|error| format!("S3 签名初始化失败: {error}"))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> Result<String, String> {
    Ok(hmac_sha256_bytes(key, data)?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn s3_signing_key(secret: &str, date_scope: &str, region: &str) -> Vec<u8> {
    let k_date = hmac_sha256_bytes(format!("AWS4{secret}").as_bytes(), date_scope.as_bytes())
        .unwrap_or_default();
    let k_region = hmac_sha256_bytes(&k_date, region.as_bytes()).unwrap_or_default();
    let k_service = hmac_sha256_bytes(&k_region, b"s3").unwrap_or_default();
    hmac_sha256_bytes(&k_service, b"aws4_request").unwrap_or_default()
}

fn sanitize_prefix(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('/');
    if trimmed.is_empty() {
        "public-uploads".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn sanitize_key_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

#[allow(dead_code)]
fn _assert_send_sync_time(_: DateTime<Utc>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_role_defaults_to_all() {
        unsafe {
            std::env::remove_var("HONE_RUNTIME_ROLE");
        }
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::All);
    }

    #[test]
    fn oss_uri_parse_requires_bucket_and_key() {
        assert_eq!(parse_oss_uri("oss://bucket/a/b"), Some(("bucket", "a/b")));
        assert!(parse_oss_uri("oss://bucket").is_none());
        assert!(parse_oss_uri("file:///tmp/a").is_none());
    }

    #[test]
    fn actor_prefix_is_scoped() {
        let actor = ActorIdentity::new("telegram", "u1", Some("group")).expect("actor");
        let cfg = OssConfig {
            access_key_id: "id".into(),
            access_key_secret: "secret".into(),
            bucket: "bucket".into(),
            endpoint: "https://oss-cn-hangzhou.aliyuncs.com".into(),
            ..OssConfig::default()
        };
        let store = OssObjectStore::from_config(&cfg).expect("store");
        assert!(
            store
                .actor_prefix(&actor)
                .starts_with("users/telegram__group__u1/")
        );
    }

    #[test]
    fn oss_provider_env_overrides_default_provider() {
        unsafe {
            std::env::set_var("HONE_TEST_R2_PROVIDER", "r2");
        }
        let cfg = OssConfig {
            provider: "aliyun_oss".into(),
            provider_env: "HONE_TEST_R2_PROVIDER".into(),
            ..OssConfig::default()
        };
        assert_eq!(cfg.resolved_provider(), "r2");
        unsafe {
            std::env::remove_var("HONE_TEST_R2_PROVIDER");
        }
    }

    #[test]
    fn s3_key_and_query_encoding_follow_sigv4_rules() {
        assert_eq!(
            encode_key_s3("/dir/a b/中文.txt"),
            "dir/a%20b/%E4%B8%AD%E6%96%87.txt"
        );
        let query = BTreeMap::from([
            ("list-type".to_string(), "2".to_string()),
            ("prefix".to_string(), "a b/中文".to_string()),
        ]);
        assert_eq!(
            s3_canonical_query(&query),
            "list-type=2&prefix=a%20b%2F%E4%B8%AD%E6%96%87"
        );
    }

    #[test]
    fn r2_provider_uses_s3_path_style_urls() {
        let cfg = OssConfig {
            provider: "r2".into(),
            access_key_id: "id".into(),
            access_key_secret: "secret".into(),
            bucket: "honeclaw".into(),
            endpoint: "https://example.r2.cloudflarestorage.com".into(),
            region: "auto".into(),
            ..OssConfig::default()
        };
        let store = OssObjectStore::from_config(&cfg).expect("store");
        assert_eq!(
            store.bucket_url(),
            "https://example.r2.cloudflarestorage.com/honeclaw"
        );
        assert_eq!(
            store.object_url("users/a b/doc.json"),
            "https://example.r2.cloudflarestorage.com/honeclaw/users/a%20b/doc.json"
        );
        assert_eq!(
            store.s3_canonical_uri("users/a b/doc.json"),
            "/honeclaw/users/a%20b/doc.json"
        );
    }

    #[test]
    fn cloud_local_dependency_report_omits_pg_backed_stores_when_postgres_is_configured() {
        unsafe {
            std::env::set_var("HONE_CLOUD_MODE", "cloud");
        }
        let mut config = HoneConfig::default();
        config.cloud.mode = "cloud".to_string();
        config.cloud.postgres.host = "localhost".to_string();
        config.cloud.postgres.user = "user".to_string();
        config.cloud.postgres.password = "password".to_string();
        config.cloud.postgres.database = "hone".to_string();
        config.storage.sessions_dir = "/tmp/hone/sessions".to_string();
        config.storage.session_sqlite_db_path = "/tmp/hone/sessions.sqlite3".to_string();
        config.storage.conversation_quota_dir = "/tmp/hone/quota".to_string();
        config.storage.cron_jobs_dir = "/tmp/hone/cron".to_string();
        assert!(config.cloud.postgres.is_configured());
        let deps = local_durable_dependencies(&config);
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/quota"));
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/sessions"));
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/cron"));
        assert!(
            !deps
                .iter()
                .any(|dep| dep == &config.storage.session_sqlite_db_path)
        );
        assert!(
            deps.iter()
                .any(|dep| dep == &config.storage.llm_audit_db_path)
        );
        unsafe {
            std::env::remove_var("HONE_CLOUD_MODE");
        }
    }

    #[test]
    fn cloud_local_dependency_report_keeps_quota_without_postgres() {
        unsafe {
            std::env::set_var("HONE_CLOUD_MODE", "cloud");
        }
        let mut config = HoneConfig::default();
        config.cloud.mode = "cloud".to_string();
        config.cloud.postgres.database_url_env = "HONE_TEST_UNUSED_PG_URL".to_string();
        config.cloud.postgres.host_env = "HONE_TEST_UNUSED_PG_HOST".to_string();
        config.cloud.postgres.user_env = "HONE_TEST_UNUSED_PG_USER".to_string();
        config.cloud.postgres.password_env = "HONE_TEST_UNUSED_PG_PASSWORD".to_string();
        config.cloud.postgres.database_env = "HONE_TEST_UNUSED_PG_DATABASE".to_string();
        config.storage.sessions_dir = "/tmp/hone/sessions".to_string();
        config.storage.conversation_quota_dir = "/tmp/hone/quota".to_string();
        config.storage.cron_jobs_dir = "/tmp/hone/cron".to_string();
        let deps = local_durable_dependencies(&config);
        assert!(deps.iter().any(|dep| dep == "/tmp/hone/quota"));
        assert!(deps.iter().any(|dep| dep == "/tmp/hone/sessions"));
        assert!(deps.iter().any(|dep| dep == "/tmp/hone/cron"));
        unsafe {
            std::env::remove_var("HONE_CLOUD_MODE");
        }
    }
}
