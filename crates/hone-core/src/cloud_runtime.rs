use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, DATE, HOST, HeaderMap, HeaderName, HeaderValue,
};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_postgres::{Client as PgClient, Config as PgConfig, NoTls};
use url::Url;

use crate::config::{CloudConfig, HoneConfig, OssConfig, PostgresConfig};
use crate::{ActorIdentity, HoneError, HoneResult, LlmAuditRecord};

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<sha2::Sha256>;

const RESERVE_CONVERSATION_QUOTA_SQL: &str = r#"
WITH inserted AS (
  INSERT INTO conversation_quota(actor_storage_key, quota_date, limit_count, reserved_count)
  VALUES ($1, $2::text::date, $3, 1)
  ON CONFLICT (actor_storage_key, quota_date) DO UPDATE
  SET
    reserved_count = conversation_quota.reserved_count + 1,
    limit_count = $3,
    updated_at = now()
  WHERE conversation_quota.committed_count + conversation_quota.reserved_count < $3
  RETURNING true AS reserved, quota_date::text, limit_count, reserved_count, committed_count
),
current_row AS (
  SELECT false AS reserved, quota_date::text, $3 AS limit_count, reserved_count, committed_count
  FROM conversation_quota
  WHERE actor_storage_key = $1
    AND quota_date = $2::text::date
    AND NOT EXISTS (SELECT 1 FROM inserted)
  FOR UPDATE
)
SELECT reserved, quota_date, limit_count, reserved_count, committed_count FROM inserted
UNION ALL
SELECT reserved, quota_date, limit_count, reserved_count, committed_count FROM current_row
LIMIT 1
"#;

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

/// A complete, read-only community post. Text and its ordered mixed-media
/// resources deliberately remain one logical content unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommunityContentRecord {
    pub content_id: i64,
    pub author_name: Option<String>,
    pub published_at: Option<String>,
    pub published_at_raw: Option<String>,
    pub content_type: String,
    pub body_text: String,
    pub body_blocks: serde_json::Value,
    pub crawl_status: String,
    pub resources: Vec<CloudCommunityResourceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommunityResourceRecord {
    pub resource_id: i64,
    pub ordinal: i32,
    pub resource_kind: String,
    pub display_name: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub sha256: Option<String>,
    pub oss_uri: Option<String>,
    pub access_state: String,
}

/// Internal snapshot used by the explicit community asset backfill workflow.
/// It is deliberately separate from the public timeline record so optimistic
/// concurrency and source identifiers never leak into the public API.
#[derive(Debug, Clone)]
pub struct CloudCommunityResourceBackfillTarget {
    pub resource_id: i64,
    pub display_name: Option<String>,
    pub source_resource_id: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub sha256: Option<String>,
    pub oss_uri: Option<String>,
    pub access_state: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CloudCommunityResourceBackfillUpdate {
    pub resource_id: i64,
    pub expected_updated_at: String,
    pub source_resource_id: Option<String>,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub oss_uri: String,
    /// Safe, non-secret provenance fields stored below the dedicated
    /// `community_asset_backfill` metadata key.
    pub audit_metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudCommunityResourceBackfillOutcome {
    Updated,
    Unchanged,
    Conflict,
    NotFound,
}

/// One topic captured from the complete, chronologically ordered source
/// timeline. The fingerprint intentionally excludes unstable DOM positions;
/// duplicate fingerprints are disambiguated by their occurrence in the full
/// manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommunityReconcileCandidate {
    pub source_topic_index: i32,
    pub source_file_position: Option<i32>,
    pub author_name: String,
    pub published_at_raw: String,
    pub body_text: String,
    pub files: Vec<CloudCommunityReconcileFile>,
    pub images: Vec<CloudCommunityReconcileImage>,
    pub candidate_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommunityReconcileFile {
    pub ordinal: i32,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommunityReconcileImage {
    pub ordinal: i32,
    pub source_base_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCommunityReconcileResource {
    pub source_topic_index: i32,
    pub source_ordinal: i32,
    pub resource_id: i64,
    pub resource_kind: String,
    pub display_name: Option<String>,
    pub source_resource_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCommunityReconcileItem {
    pub source_topic_index: i32,
    pub source_file_position: Option<i32>,
    pub source_item_key: String,
    pub action: String,
    pub content_id: Option<i64>,
    pub resources: Vec<CloudCommunityReconcileResource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCommunityReconcileReport {
    pub mode: &'static str,
    pub source_topic_count: usize,
    pub source_file_count: usize,
    pub existing_by_file_position: usize,
    pub existing_by_source_key: usize,
    pub would_insert: usize,
    pub inserted: usize,
    pub items: Vec<CloudCommunityReconcileItem>,
}

#[derive(Debug, Clone)]
struct PlannedCommunityReconcileCandidate {
    candidate: CloudCommunityReconcileCandidate,
    source_item_key: String,
    fingerprint_occurrence: usize,
}

fn plan_community_reconcile_candidates(
    source: &str,
    candidates: &[CloudCommunityReconcileCandidate],
) -> HoneResult<Vec<PlannedCommunityReconcileCandidate>> {
    if source.is_empty() || sanitize_key_component(source) != source {
        return Err(HoneError::Config(
            "community reconcile source 不安全".to_string(),
        ));
    }
    if candidates.is_empty() || candidates.len() > 10_000 {
        return Err(HoneError::Config(
            "community reconcile manifest 条目数必须在 1..=10000".to_string(),
        ));
    }

    let mut ordered = candidates.to_vec();
    ordered.sort_by_key(|candidate| candidate.source_topic_index);
    let mut file_positions = BTreeSet::new();
    for (expected_index, candidate) in ordered.iter().enumerate() {
        if candidate.source_topic_index != expected_index as i32 {
            return Err(HoneError::Config(format!(
                "community reconcile source_topic_index 必须从 0 连续递增，期望 {expected_index}，实际 {}",
                candidate.source_topic_index
            )));
        }
        if candidate.author_name.trim().is_empty() || candidate.author_name.len() > 1_000 {
            return Err(HoneError::Config(format!(
                "community reconcile topic {expected_index} author_name 无效"
            )));
        }
        chrono::NaiveDateTime::parse_from_str(&candidate.published_at_raw, "%Y-%m-%d %H:%M")
            .map_err(|_| {
                HoneError::Config(format!(
                    "community reconcile topic {expected_index} published_at_raw 无效"
                ))
            })?;
        if candidate.candidate_fingerprint.len() != 64
            || !candidate
                .candidate_fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(HoneError::Config(format!(
                "community reconcile topic {expected_index} fingerprint 必须是小写 SHA-256"
            )));
        }
        if candidate.files.is_empty()
            && candidate.images.is_empty()
            && candidate.body_text.is_empty()
        {
            return Err(HoneError::Config(format!(
                "community reconcile topic {expected_index} 不能是空内容"
            )));
        }
        if candidate.files.len() > 100 || candidate.images.len() > 100 {
            return Err(HoneError::Config(format!(
                "community reconcile topic {expected_index} 资源数超过上限"
            )));
        }
        for (ordinal, file) in candidate.files.iter().enumerate() {
            if file.ordinal != ordinal as i32
                || file.display_name.trim().is_empty()
                || file.display_name.len() > 4_096
                || file.display_name.contains('\0')
            {
                return Err(HoneError::Config(format!(
                    "community reconcile topic {expected_index} file ordinal/name 无效"
                )));
            }
        }
        for (ordinal, image) in candidate.images.iter().enumerate() {
            if image.ordinal != ordinal as i32
                || image.source_base_key.is_empty()
                || image.source_base_key.len() > 256
                || !image
                    .source_base_key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            {
                return Err(HoneError::Config(format!(
                    "community reconcile topic {expected_index} image ordinal/source key 无效"
                )));
            }
        }
        match candidate.source_file_position {
            Some(position) if position >= 0 && !candidate.files.is_empty() => {
                if !file_positions.insert(position) {
                    return Err(HoneError::Config(format!(
                        "community reconcile source_file_position={position} 重复"
                    )));
                }
            }
            Some(_) => {
                return Err(HoneError::Config(format!(
                    "community reconcile topic {expected_index} file position 与 files 不一致"
                )));
            }
            None if !candidate.files.is_empty() => {
                return Err(HoneError::Config(format!(
                    "community reconcile topic {expected_index} 有文件但缺少 file position"
                )));
            }
            None => {}
        }
    }
    for (expected, actual) in file_positions.iter().copied().enumerate() {
        if actual != expected as i32 {
            return Err(HoneError::Config(format!(
                "community reconcile source_file_position 必须从 0 连续递增，期望 {expected}，实际 {actual}"
            )));
        }
    }

    let mut fingerprint_occurrences = BTreeMap::<String, usize>::new();
    Ok(ordered
        .into_iter()
        .map(|candidate| {
            let occurrence = fingerprint_occurrences
                .entry(candidate.candidate_fingerprint.clone())
                .and_modify(|value| *value += 1)
                .or_insert(1);
            PlannedCommunityReconcileCandidate {
                source_item_key: format!(
                    "{source}-dom-v2:{}:occurrence:{}",
                    candidate.candidate_fingerprint, *occurrence
                ),
                fingerprint_occurrence: *occurrence,
                candidate,
            }
        })
        .collect())
}

fn community_file_content_type(display_name: &str) -> Option<String> {
    let extension = PathBuf::from(display_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("pdf") => Some("application/pdf".to_string()),
        Some("docx") => Some(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        ),
        Some("pptx") => Some(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
        ),
        Some("xlsx") => {
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string())
        }
        Some("xls") => Some("application/vnd.ms-excel".to_string()),
        _ => None,
    }
}

async fn load_reconciled_community_resources(
    transaction: &tokio_postgres::Transaction<'_>,
    content_id: i64,
    source_topic_index: i32,
) -> HoneResult<Vec<CloudCommunityReconcileResource>> {
    let rows = transaction
        .query(
            r#"
SELECT resource_id, ordinal, resource_kind, display_name, source_resource_id, raw_metadata
FROM community_content_resources
WHERE content_id = $1
ORDER BY ordinal
"#,
            &[&content_id],
        )
        .await
        .map_err(|err| {
            HoneError::Config(format!(
                "Postgres community reconcile resources 读取失败: {err}"
            ))
        })?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let ordinal: i32 = row.get(1);
            let metadata: serde_json::Value = row.get(5);
            let source_ordinal = metadata
                .get("source_ordinal")
                .and_then(serde_json::Value::as_i64)
                .and_then(|value| i32::try_from(value).ok())
                .unwrap_or(ordinal);
            CloudCommunityReconcileResource {
                source_topic_index,
                source_ordinal,
                resource_id: row.get(0),
                resource_kind: row.get(2),
                display_name: row.get(3),
                source_resource_id: row.get(4),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCommunityUnreadState {
    pub latest_content_id: Option<i64>,
    pub last_seen_content_id: Option<i64>,
    pub unread: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudSkillRegistryImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudNotificationPrefsRecord {
    pub actor_storage_key: String,
    pub prefs: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudNotificationPrefsImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudPortfolioRecord {
    pub actor_storage_key: String,
    pub actor: serde_json::Value,
    pub portfolio: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudPortfolioImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCompanyProfileFileRecord {
    pub actor_storage_key: String,
    pub actor: serde_json::Value,
    pub profile_id: String,
    pub relative_path: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudCompanyProfileSpaceRecord {
    pub actor_storage_key: String,
    pub actor: serde_json::Value,
    pub profile_count: usize,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudCompanyProfileImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloudLlmAuditRecord {
    pub id: String,
    pub actor_storage_key: Option<String>,
    pub created_at: String,
    pub record: serde_json::Value,
}

impl CloudLlmAuditRecord {
    pub fn from_audit_record(record: &LlmAuditRecord) -> HoneResult<Self> {
        Ok(Self {
            id: record.id.clone(),
            actor_storage_key: record.actor.as_ref().map(ActorIdentity::storage_key),
            created_at: record.created_at.clone(),
            record: serde_json::to_value(record)
                .map_err(|err| HoneError::Serialization(err.to_string()))?,
        })
    }

    fn payload_json_for_postgres(&self) -> HoneResult<String> {
        serde_json::to_string(&self.record)
            .map_err(|err| HoneError::Serialization(format!("LLM audit JSON 序列化失败: {err}")))
    }

    fn created_at_text_for_postgres(&self) -> HoneResult<String> {
        DateTime::parse_from_rfc3339(&self.created_at).map_err(|err| {
            HoneError::Config(format!(
                "Postgres LLM audit created_at 非法 RFC3339 时间戳: {} ({err})",
                self.created_at
            ))
        })?;
        Ok(self.created_at.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CloudLlmAuditFilter {
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct CloudLlmAuditImportReport {
    pub changed_rows: usize,
    pub skipped_rows: usize,
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

fn normalize_cloud_cron_execution_input_for_storage(
    actor: &ActorIdentity,
    mut input: CloudCronExecutionInput,
) -> CloudCronExecutionInput {
    if input.message_send_status != "send_failed" || input.delivered {
        return input;
    }

    let sent_segments = input
        .detail
        .get("sent_segments")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let total_segments = input
        .detail
        .get("total_segments")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    if sent_segments != 0 || total_segments == 0 {
        return input;
    }

    let fallback_error = match actor.channel.as_str() {
        "discord" => "Discord 定时任务发送失败",
        _ => "定时任务发送失败",
    };
    if input
        .error_message
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        input.error_message = Some(fallback_error.to_string());
    }

    if let serde_json::Value::Object(detail) = &mut input.detail
        && detail
            .get("failure_kind")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        let failure_kind = match actor.channel.as_str() {
            "discord" => "discord_send_failed",
            "feishu" => "feishu_send_failed",
            "telegram" => "telegram_send_failed",
            "web" => "web_send_failed",
            _ => "channel_send_failed",
        };
        detail.insert(
            "failure_kind".to_string(),
            serde_json::Value::String(failure_kind.to_string()),
        );
    }

    input
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
pub struct CloudWebPushMessage {
    pub push_id: String,
    pub actor_storage_key: String,
    pub job_id: String,
    pub job_name: String,
    pub summary: String,
    pub content: String,
    pub created_at: String,
    pub read_at: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct CloudSessionListEntry {
    pub session_id: String,
    pub actor: Option<serde_json::Value>,
    pub session_identity: Option<serde_json::Value>,
    pub updated_at: String,
    pub last_message: Option<serde_json::Value>,
    pub message_count: usize,
}

static PG_CLIENT_CACHE: LazyLock<Mutex<BTreeMap<String, Arc<PgClient>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

impl CloudPgRuntime {
    pub fn from_cloud_config(config: &CloudConfig) -> Option<Self> {
        config.postgres.is_configured().then(|| Self {
            config: config.postgres.clone(),
        })
    }

    async fn connect_client(&self) -> HoneResult<Arc<PgClient>> {
        self.connect_new_client().await.map(Arc::new)
    }

    async fn connect_cached_client(&self) -> HoneResult<Arc<PgClient>> {
        let cache_key = self.client_cache_key();
        if let Some(client) = PG_CLIENT_CACHE
            .lock()
            .map_err(|err| HoneError::Config(format!("Postgres client cache 锁失败: {err}")))?
            .get(&cache_key)
            .cloned()
        {
            return Ok(client);
        }

        let client = Arc::new(self.connect_new_client().await?);
        PG_CLIENT_CACHE
            .lock()
            .map_err(|err| HoneError::Config(format!("Postgres client cache 锁失败: {err}")))?
            .insert(cache_key, client.clone());
        Ok(client)
    }

    fn client_cache_key(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.config.resolved_proxy(),
            self.config.resolved_host(),
            self.config.resolved_port().unwrap_or(5432),
            self.config.resolved_user(),
            self.config.resolved_database(),
            self.config.resolved_database_url(),
        )
    }

    async fn connect_new_client(&self) -> HoneResult<PgClient> {
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
CREATE TABLE IF NOT EXISTS cloud_web_push_messages (
  actor_storage_key TEXT NOT NULL,
  push_id TEXT NOT NULL,
  job_id TEXT NOT NULL,
  job_name TEXT NOT NULL,
  summary TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TEXT NOT NULL,
  read_at TEXT,
  PRIMARY KEY (actor_storage_key, push_id)
);
CREATE INDEX IF NOT EXISTS idx_cloud_web_push_actor_time
  ON cloud_web_push_messages(actor_storage_key, created_at DESC, push_id DESC);
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
CREATE INDEX IF NOT EXISTS idx_cloud_llm_audit_created_at
  ON cloud_llm_audit_records(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_cloud_llm_audit_actor_time
  ON cloud_llm_audit_records(actor_storage_key, created_at DESC);
CREATE TABLE IF NOT EXISTS cloud_skill_registry (
  registry_key TEXT PRIMARY KEY,
  registry JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_notification_prefs (
  actor_storage_key TEXT PRIMARY KEY,
  prefs JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_portfolios (
  actor_storage_key TEXT PRIMARY KEY,
  actor JSONB NOT NULL,
  portfolio JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_company_profile_files (
  actor_storage_key TEXT NOT NULL,
  actor JSONB NOT NULL,
  profile_id TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  content TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (actor_storage_key, profile_id, relative_path)
);
CREATE INDEX IF NOT EXISTS idx_cloud_company_profile_files_actor
  ON cloud_company_profile_files(actor_storage_key, updated_at DESC);
CREATE TABLE IF NOT EXISTS community_spaces (
  community_id BIGSERIAL PRIMARY KEY,
  source TEXT NOT NULL,
  external_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  source_url TEXT,
  raw_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(source, external_id)
);
CREATE TABLE IF NOT EXISTS community_contents (
  content_id BIGSERIAL PRIMARY KEY,
  community_id BIGINT NOT NULL REFERENCES community_spaces(community_id) ON DELETE CASCADE,
  source_item_key TEXT NOT NULL,
  source_item_id TEXT,
  source_url TEXT,
  author_name TEXT,
  author_external_id TEXT,
  published_at TIMESTAMPTZ,
  published_at_raw TEXT,
  content_type TEXT NOT NULL DEFAULT 'post',
  body_text TEXT NOT NULL DEFAULT '',
  body_blocks JSONB NOT NULL DEFAULT '[]'::jsonb,
  raw_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  source_hash TEXT,
  crawl_status TEXT NOT NULL DEFAULT 'complete',
  first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(community_id, source_item_key)
);
CREATE INDEX IF NOT EXISTS idx_community_contents_timeline
  ON community_contents(community_id, published_at DESC, content_id DESC);
CREATE TABLE IF NOT EXISTS community_content_resources (
  resource_id BIGSERIAL PRIMARY KEY,
  content_id BIGINT NOT NULL REFERENCES community_contents(content_id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL,
  resource_kind TEXT NOT NULL,
  source_resource_id TEXT,
  display_name TEXT,
  original_url TEXT,
  content_type TEXT,
  byte_size BIGINT,
  sha256 TEXT,
  oss_uri TEXT,
  access_state TEXT NOT NULL DEFAULT 'metadata_only',
  raw_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  fetched_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(content_id, ordinal)
);
CREATE INDEX IF NOT EXISTS idx_community_resources_content
  ON community_content_resources(content_id, ordinal);
CREATE TABLE IF NOT EXISTS community_read_states (
  actor_storage_key TEXT NOT NULL,
  community_id BIGINT NOT NULL REFERENCES community_spaces(community_id) ON DELETE CASCADE,
  last_seen_content_id BIGINT REFERENCES community_contents(content_id) ON DELETE SET NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY(actor_storage_key, community_id)
);
INSERT INTO cloud_schema_migrations(version)
VALUES ('20260529_pg_oss_runtime_foundation')
ON CONFLICT (version) DO NOTHING;
INSERT INTO cloud_schema_migrations(version)
VALUES ('20260712_community_content_archive')
ON CONFLICT (version) DO NOTHING;
"#,
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres schema 初始化失败: {err}")))?;
        Ok(())
    }

    /// Return a cursor page from a community timeline. `before_content_id` is
    /// intentionally opaque to callers: ordering stays timestamp-first even
    /// when imports are replayed out of source order.
    pub async fn list_community_contents(
        &self,
        source: &str,
        external_id: &str,
        before_content_id: Option<i64>,
        limit: usize,
    ) -> HoneResult<Vec<CloudCommunityContentRecord>> {
        let client = self.connect_cached_client().await?;
        let limit = i64::try_from(limit.clamp(1, 50))
            .map_err(|_| HoneError::Config("community page limit invalid".to_string()))?;
        let rows = client
            .query(
                r#"
WITH target_space AS (
  SELECT community_id FROM community_spaces
  WHERE source = $1 AND external_id = $2
), page AS (
  SELECT c.*
  FROM community_contents c
  JOIN target_space s ON s.community_id = c.community_id
  WHERE $3::bigint IS NULL
     OR (c.published_at, c.content_id) < (
       SELECT published_at, content_id FROM community_contents WHERE content_id = $3
     )
  ORDER BY c.published_at DESC NULLS LAST, c.content_id DESC
  LIMIT $4
)
SELECT
  p.content_id,
  p.author_name,
  p.published_at::text,
  p.published_at_raw,
  p.content_type,
  p.body_text,
  p.body_blocks,
  p.crawl_status,
  COALESCE(
    jsonb_agg(jsonb_build_object(
      'resource_id', r.resource_id,
      'ordinal', r.ordinal,
      'resource_kind', r.resource_kind,
      'display_name', r.display_name,
      'content_type', r.content_type,
      'byte_size', r.byte_size,
      'sha256', r.sha256,
      'oss_uri', r.oss_uri,
      'access_state', r.access_state
    ) ORDER BY r.ordinal) FILTER (WHERE r.resource_id IS NOT NULL),
    '[]'::jsonb
  ) AS resources
FROM page p
LEFT JOIN community_content_resources r ON r.content_id = p.content_id
GROUP BY
  p.content_id,
  p.author_name,
  p.published_at,
  p.published_at_raw,
  p.content_type,
  p.body_text,
  p.body_blocks,
  p.crawl_status
ORDER BY p.published_at DESC NULLS LAST, p.content_id DESC
"#,
                &[&source, &external_id, &before_content_id, &limit],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres community timeline 读取失败: {err}"))
            })?;

        rows.into_iter()
            .map(|row| {
                let resources: serde_json::Value = row.get(8);
                Ok(CloudCommunityContentRecord {
                    content_id: row.get(0),
                    author_name: row.get(1),
                    published_at: row.get(2),
                    published_at_raw: row.get(3),
                    content_type: row.get(4),
                    body_text: row.get(5),
                    body_blocks: row.get(6),
                    crawl_status: row.get(7),
                    resources: serde_json::from_value(resources)
                        .map_err(|err| HoneError::Serialization(err.to_string()))?,
                })
            })
            .collect()
    }

    pub async fn get_community_resource(
        &self,
        source: &str,
        external_id: &str,
        resource_id: i64,
    ) -> HoneResult<Option<CloudCommunityResourceRecord>> {
        let client = self.connect_cached_client().await?;
        let row = client
            .query_opt(
                r#"
SELECT r.resource_id, r.ordinal, r.resource_kind, r.display_name,
       r.content_type, r.byte_size, r.sha256, r.oss_uri, r.access_state
FROM community_content_resources r
JOIN community_contents c ON c.content_id = r.content_id
JOIN community_spaces s ON s.community_id = c.community_id
WHERE s.source = $1 AND s.external_id = $2 AND r.resource_id = $3
"#,
                &[&source, &external_id, &resource_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres community resource 读取失败: {err}"))
            })?;
        Ok(row.map(|row| CloudCommunityResourceRecord {
            resource_id: row.get(0),
            ordinal: row.get(1),
            resource_kind: row.get(2),
            display_name: row.get(3),
            content_type: row.get(4),
            byte_size: row.get(5),
            sha256: row.get(6),
            oss_uri: row.get(7),
            access_state: row.get(8),
        }))
    }

    pub async fn get_community_resource_backfill_target(
        &self,
        source: &str,
        external_id: &str,
        resource_id: i64,
    ) -> HoneResult<Option<CloudCommunityResourceBackfillTarget>> {
        let client = self.connect_cached_client().await?;
        let row = client
            .query_opt(
                r#"
SELECT r.resource_id, r.display_name, r.source_resource_id, r.content_type,
       r.byte_size, r.sha256, r.oss_uri, r.access_state, r.updated_at::text
FROM community_content_resources r
JOIN community_contents c ON c.content_id = r.content_id
JOIN community_spaces s ON s.community_id = c.community_id
WHERE s.source = $1 AND s.external_id = $2 AND r.resource_id = $3
"#,
                &[&source, &external_id, &resource_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community backfill target 读取失败: {err}"
                ))
            })?;
        Ok(row.map(|row| CloudCommunityResourceBackfillTarget {
            resource_id: row.get(0),
            display_name: row.get(1),
            source_resource_id: row.get(2),
            content_type: row.get(3),
            byte_size: row.get(4),
            sha256: row.get(5),
            oss_uri: row.get(6),
            access_state: row.get(7),
            updated_at: row.get(8),
        }))
    }

    /// Atomically promotes a verified community resource to stored bytes.
    /// The caller must verify the immutable object before invoking this
    /// method. A row lock plus `updated_at` snapshot prevents a concurrent
    /// backfill from being silently overwritten.
    pub async fn backfill_community_resource(
        &self,
        source: &str,
        external_id: &str,
        update: &CloudCommunityResourceBackfillUpdate,
    ) -> HoneResult<CloudCommunityResourceBackfillOutcome> {
        if !update.audit_metadata.is_object() {
            return Err(HoneError::Config(
                "community asset backfill audit metadata 必须是 JSON object".to_string(),
            ));
        }

        let mut client = self.connect_new_client().await?;
        let transaction = client.transaction().await.map_err(|err| {
            HoneError::Config(format!(
                "Postgres community backfill transaction 创建失败: {err}"
            ))
        })?;
        let row = transaction
            .query_opt(
                r#"
SELECT r.source_resource_id, r.content_type, r.byte_size, r.sha256,
       r.oss_uri, r.access_state, r.updated_at::text
FROM community_content_resources r
JOIN community_contents c ON c.content_id = r.content_id
JOIN community_spaces s ON s.community_id = c.community_id
WHERE s.source = $1 AND s.external_id = $2 AND r.resource_id = $3
FOR UPDATE OF r
"#,
                &[&source, &external_id, &update.resource_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres community backfill row lock 失败: {err}"))
            })?;
        let Some(row) = row else {
            transaction.commit().await.map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community backfill transaction 提交失败: {err}"
                ))
            })?;
            return Ok(CloudCommunityResourceBackfillOutcome::NotFound);
        };

        let current_source_resource_id: Option<String> = row.get(0);
        let current_content_type: Option<String> = row.get(1);
        let current_byte_size: Option<i64> = row.get(2);
        let current_sha256: Option<String> = row.get(3);
        let current_oss_uri: Option<String> = row.get(4);
        let current_access_state: String = row.get(5);
        let current_updated_at: String = row.get(6);
        let desired_source_resource_id = update
            .source_resource_id
            .as_ref()
            .or(current_source_resource_id.as_ref());

        let already_current = current_content_type.as_deref() == Some(update.content_type.as_str())
            && current_byte_size == Some(update.byte_size)
            && current_sha256.as_deref() == Some(update.sha256.as_str())
            && current_oss_uri.as_deref() == Some(update.oss_uri.as_str())
            && current_access_state == "stored"
            && current_source_resource_id.as_ref() == desired_source_resource_id;
        if already_current {
            transaction.commit().await.map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community backfill transaction 提交失败: {err}"
                ))
            })?;
            return Ok(CloudCommunityResourceBackfillOutcome::Unchanged);
        }

        let source_id_conflicts = match (
            current_source_resource_id.as_deref(),
            update.source_resource_id.as_deref(),
        ) {
            (Some(current), Some(desired)) => current != desired,
            _ => false,
        };
        if current_updated_at != update.expected_updated_at || source_id_conflicts {
            transaction.commit().await.map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community backfill transaction 提交失败: {err}"
                ))
            })?;
            return Ok(CloudCommunityResourceBackfillOutcome::Conflict);
        }

        transaction
            .execute(
                r#"
UPDATE community_content_resources
SET source_resource_id = COALESCE($2, source_resource_id),
    content_type = $3,
    byte_size = $4,
    sha256 = $5,
    oss_uri = $6,
    access_state = 'stored',
    raw_metadata = jsonb_set(
      COALESCE(raw_metadata, '{}'::jsonb),
      '{community_asset_backfill}',
      $7::jsonb,
      true
    ),
    fetched_at = now(),
    updated_at = now()
WHERE resource_id = $1
"#,
                &[
                    &update.resource_id,
                    &update.source_resource_id,
                    &update.content_type,
                    &update.byte_size,
                    &update.sha256,
                    &update.oss_uri,
                    &update.audit_metadata,
                ],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres community backfill update 失败: {err}"))
            })?;
        transaction.commit().await.map_err(|err| {
            HoneError::Config(format!(
                "Postgres community backfill transaction 提交失败: {err}"
            ))
        })?;
        Ok(CloudCommunityResourceBackfillOutcome::Updated)
    }

    /// Reconcile a complete source timeline against the archived community.
    ///
    /// Existing file-backed rows are matched only by their captured source
    /// file position. Missing/non-file topics use a deterministic fingerprint
    /// plus occurrence key, so two visually identical adjacent topics remain
    /// distinct without depending on a transient DOM index. Apply mode holds a
    /// row lock on the community space and inserts every missing content and
    /// resource in one transaction.
    pub async fn reconcile_community_contents(
        &self,
        source: &str,
        external_id: &str,
        candidates: &[CloudCommunityReconcileCandidate],
        apply: bool,
    ) -> HoneResult<CloudCommunityReconcileReport> {
        let planned = plan_community_reconcile_candidates(source, candidates)?;
        let source_file_positions = planned
            .iter()
            .filter_map(|candidate| candidate.candidate.source_file_position)
            .collect::<BTreeSet<_>>();

        let mut client = self.connect_new_client().await?;
        let transaction = client.transaction().await.map_err(|err| {
            HoneError::Config(format!(
                "Postgres community reconcile transaction 创建失败: {err}"
            ))
        })?;
        let space = transaction
            .query_opt(
                r#"
SELECT community_id, source_url
FROM community_spaces
WHERE source = $1 AND external_id = $2
FOR UPDATE
"#,
                &[&source, &external_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community reconcile space 读取失败: {err}"
                ))
            })?;
        let Some(space) = space else {
            transaction.rollback().await.map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community reconcile transaction 回滚失败: {err}"
                ))
            })?;
            return Err(HoneError::Config(format!(
                "community space 不存在: {source}/{external_id}"
            )));
        };
        let community_id: i64 = space.get(0);
        let source_url: Option<String> = space.get(1);

        let rows = transaction
            .query(
                r#"
SELECT content_id, source_item_key, raw_metadata
FROM community_contents
WHERE community_id = $1
ORDER BY content_id
"#,
                &[&community_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community reconcile contents 读取失败: {err}"
                ))
            })?;
        let mut existing_by_file_position = BTreeMap::<i32, i64>::new();
        let mut existing_by_source_key = BTreeMap::<String, i64>::new();
        for row in rows {
            let content_id: i64 = row.get(0);
            let source_item_key: String = row.get(1);
            let raw_metadata: serde_json::Value = row.get(2);
            existing_by_source_key.insert(source_item_key, content_id);
            let position = raw_metadata
                .get("feed_position")
                .or_else(|| raw_metadata.get("source_file_position"))
                .and_then(serde_json::Value::as_i64)
                .and_then(|value| i32::try_from(value).ok());
            if let Some(position) = position {
                if !source_file_positions.contains(&position) {
                    transaction.rollback().await.map_err(|err| {
                        HoneError::Config(format!(
                            "Postgres community reconcile transaction 回滚失败: {err}"
                        ))
                    })?;
                    return Err(HoneError::Config(format!(
                        "数据库 feed_position={position} 不在完整源清单中"
                    )));
                }
                if let Some(previous) = existing_by_file_position.insert(position, content_id) {
                    transaction.rollback().await.map_err(|err| {
                        HoneError::Config(format!(
                            "Postgres community reconcile transaction 回滚失败: {err}"
                        ))
                    })?;
                    return Err(HoneError::Config(format!(
                        "数据库 feed_position={position} 重复关联 content_id={previous}/{content_id}"
                    )));
                }
            }
        }

        let mut report = CloudCommunityReconcileReport {
            mode: if apply { "apply" } else { "dry-run" },
            source_topic_count: planned.len(),
            source_file_count: source_file_positions.len(),
            existing_by_file_position: 0,
            existing_by_source_key: 0,
            would_insert: 0,
            inserted: 0,
            items: Vec::new(),
        };

        for planned_candidate in planned {
            let candidate = &planned_candidate.candidate;
            if candidate
                .source_file_position
                .is_some_and(|position| existing_by_file_position.contains_key(&position))
            {
                report.existing_by_file_position += 1;
                continue;
            }

            if let Some(content_id) = existing_by_source_key
                .get(&planned_candidate.source_item_key)
                .copied()
            {
                let resources = load_reconciled_community_resources(
                    &transaction,
                    content_id,
                    candidate.source_topic_index,
                )
                .await?;
                report.existing_by_source_key += 1;
                report.items.push(CloudCommunityReconcileItem {
                    source_topic_index: candidate.source_topic_index,
                    source_file_position: candidate.source_file_position,
                    source_item_key: planned_candidate.source_item_key,
                    action: "already_present".to_string(),
                    content_id: Some(content_id),
                    resources,
                });
                continue;
            }

            if !apply {
                report.would_insert += 1;
                report.items.push(CloudCommunityReconcileItem {
                    source_topic_index: candidate.source_topic_index,
                    source_file_position: candidate.source_file_position,
                    source_item_key: planned_candidate.source_item_key,
                    action: "would_insert".to_string(),
                    content_id: None,
                    resources: Vec::new(),
                });
                continue;
            }

            let body_blocks = if candidate.body_text.is_empty() {
                serde_json::json!([])
            } else {
                serde_json::json!([{"type": "text", "text": candidate.body_text}])
            };
            let raw_metadata = serde_json::json!({
                "captured_from": "full_timeline_reconciliation",
                "source_topic_index": candidate.source_topic_index,
                "source_file_position": candidate.source_file_position,
                "feed_position": candidate.source_file_position,
                "candidate_fingerprint": candidate.candidate_fingerprint,
                "fingerprint_occurrence": planned_candidate.fingerprint_occurrence,
            });
            let content_row = transaction
                .query_one(
                    r#"
INSERT INTO community_contents(
  community_id, source_item_key, source_item_id, source_url, author_name,
  published_at, published_at_raw, content_type, body_text, body_blocks,
  raw_metadata, source_hash, crawl_status
)
VALUES (
  $1, $2, $3, $4, $5,
  $6::text::timestamp AT TIME ZONE 'Asia/Shanghai', $6, 'post', $7, $8,
  $9, $3, 'complete'
)
RETURNING content_id
"#,
                    &[
                        &community_id,
                        &planned_candidate.source_item_key,
                        &candidate.candidate_fingerprint,
                        &source_url,
                        &candidate.author_name,
                        &candidate.published_at_raw,
                        &candidate.body_text,
                        &body_blocks,
                        &raw_metadata,
                    ],
                )
                .await
                .map_err(|err| {
                    HoneError::Config(format!(
                        "Postgres community reconcile content 写入失败: {err}"
                    ))
                })?;
            let content_id: i64 = content_row.get(0);
            let mut resources = Vec::new();
            let mut combined_ordinal = 0_i32;
            for file in &candidate.files {
                let content_type = community_file_content_type(&file.display_name);
                let resource_metadata = serde_json::json!({
                    "captured_from": "full_timeline_reconciliation",
                    "source_topic_index": candidate.source_topic_index,
                    "source_ordinal": file.ordinal,
                });
                let resource_row = transaction
                    .query_one(
                        r#"
INSERT INTO community_content_resources(
  content_id, ordinal, resource_kind, display_name, content_type,
  access_state, raw_metadata
)
VALUES ($1, $2, 'file', $3, $4, 'metadata_only', $5)
RETURNING resource_id
"#,
                        &[
                            &content_id,
                            &combined_ordinal,
                            &file.display_name,
                            &content_type,
                            &resource_metadata,
                        ],
                    )
                    .await
                    .map_err(|err| {
                        HoneError::Config(format!(
                            "Postgres community reconcile file resource 写入失败: {err}"
                        ))
                    })?;
                resources.push(CloudCommunityReconcileResource {
                    source_topic_index: candidate.source_topic_index,
                    source_ordinal: file.ordinal,
                    resource_id: resource_row.get(0),
                    resource_kind: "file".to_string(),
                    display_name: Some(file.display_name.clone()),
                    source_resource_id: None,
                });
                combined_ordinal += 1;
            }
            for image in &candidate.images {
                let display_name = format!("image-{}", image.ordinal + 1);
                let resource_metadata = serde_json::json!({
                    "captured_from": "full_timeline_reconciliation",
                    "source_topic_index": candidate.source_topic_index,
                    "source_ordinal": image.ordinal,
                    "source_base_key": image.source_base_key,
                });
                let resource_row = transaction
                    .query_one(
                        r#"
INSERT INTO community_content_resources(
  content_id, ordinal, resource_kind, source_resource_id, display_name,
  access_state, raw_metadata
)
VALUES ($1, $2, 'image', $3, $4, 'metadata_only', $5)
RETURNING resource_id
"#,
                        &[
                            &content_id,
                            &combined_ordinal,
                            &image.source_base_key,
                            &display_name,
                            &resource_metadata,
                        ],
                    )
                    .await
                    .map_err(|err| {
                        HoneError::Config(format!(
                            "Postgres community reconcile image resource 写入失败: {err}"
                        ))
                    })?;
                resources.push(CloudCommunityReconcileResource {
                    source_topic_index: candidate.source_topic_index,
                    source_ordinal: image.ordinal,
                    resource_id: resource_row.get(0),
                    resource_kind: "image".to_string(),
                    display_name: Some(display_name),
                    source_resource_id: Some(image.source_base_key.clone()),
                });
                combined_ordinal += 1;
            }

            report.inserted += 1;
            report.items.push(CloudCommunityReconcileItem {
                source_topic_index: candidate.source_topic_index,
                source_file_position: candidate.source_file_position,
                source_item_key: planned_candidate.source_item_key.clone(),
                action: "inserted".to_string(),
                content_id: Some(content_id),
                resources,
            });
            existing_by_source_key.insert(planned_candidate.source_item_key, content_id);
        }

        if apply {
            transaction.commit().await.map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community reconcile transaction 提交失败: {err}"
                ))
            })?;
        } else {
            transaction.rollback().await.map_err(|err| {
                HoneError::Config(format!(
                    "Postgres community reconcile transaction 回滚失败: {err}"
                ))
            })?;
        }
        Ok(report)
    }

    pub async fn community_unread_state(
        &self,
        source: &str,
        external_id: &str,
        actor_storage_key: &str,
    ) -> HoneResult<CloudCommunityUnreadState> {
        let client = self.connect_cached_client().await?;
        let row = client
            .query_one(
                r#"
WITH target_space AS (
  SELECT community_id FROM community_spaces WHERE source = $1 AND external_id = $2
), latest AS (
  SELECT content_id FROM community_contents
  WHERE community_id = (SELECT community_id FROM target_space)
  ORDER BY published_at DESC NULLS LAST, content_id DESC LIMIT 1
)
SELECT
  (SELECT content_id FROM latest),
  (SELECT last_seen_content_id FROM community_read_states
   WHERE actor_storage_key = $3 AND community_id = (SELECT community_id FROM target_space))
"#,
                &[&source, &external_id, &actor_storage_key],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres community unread 读取失败: {err}"))
            })?;
        let latest_content_id: Option<i64> = row.get(0);
        let last_seen_content_id: Option<i64> = row.get(1);
        Ok(CloudCommunityUnreadState {
            latest_content_id,
            last_seen_content_id,
            unread: latest_content_id.is_some() && latest_content_id != last_seen_content_id,
        })
    }

    pub async fn mark_community_seen(
        &self,
        source: &str,
        external_id: &str,
        actor_storage_key: &str,
        content_id: i64,
    ) -> HoneResult<()> {
        let client = self.connect_cached_client().await?;
        client
            .execute(
                r#"
INSERT INTO community_read_states(actor_storage_key, community_id, last_seen_content_id)
SELECT $3, s.community_id, c.content_id
FROM community_spaces s
JOIN community_contents c ON c.community_id = s.community_id AND c.content_id = $4
WHERE s.source = $1 AND s.external_id = $2
ON CONFLICT(actor_storage_key, community_id) DO UPDATE SET
  last_seen_content_id = EXCLUDED.last_seen_content_id,
  updated_at = now()
"#,
                &[&source, &external_id, &actor_storage_key, &content_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres community read state 写入失败: {err}"))
            })?;
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
                RESERVE_CONVERSATION_QUOTA_SQL,
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

    pub async fn list_session_summaries(&self) -> HoneResult<Vec<CloudSessionListEntry>> {
        let client = self.connect_cached_client().await?;
        let rows = client
            .query(
                r#"
SELECT
  session_id,
  content->'actor' AS actor,
  content->'session_identity' AS session_identity,
  COALESCE(content->>'updated_at', updated_at::text) AS updated_at,
  (
    SELECT message
    FROM jsonb_array_elements(COALESCE(content->'messages', '[]'::jsonb)) WITH ORDINALITY AS messages(message, ord)
    WHERE message->>'role' IN ('user', 'assistant')
    ORDER BY ord DESC
    LIMIT 1
  ) AS last_message,
  (
    SELECT count(*)::bigint
    FROM jsonb_array_elements(COALESCE(content->'messages', '[]'::jsonb)) AS messages(message)
    WHERE message->>'role' IN ('user', 'assistant')
  ) AS message_count
FROM cloud_sessions
ORDER BY updated_at DESC
"#,
                &[],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres session 摘要列表读取失败: {err}")))?;
        rows.into_iter()
            .map(|row| {
                let message_count = row.get::<_, i64>("message_count").max(0) as usize;
                Ok(CloudSessionListEntry {
                    session_id: row.get("session_id"),
                    actor: row.get("actor"),
                    session_identity: row.get("session_identity"),
                    updated_at: row.get("updated_at"),
                    last_message: row.get("last_message"),
                    message_count,
                })
            })
            .collect()
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

    pub async fn list_web_invite_user_records_cached(&self) -> HoneResult<Vec<serde_json::Value>> {
        let client = self.connect_cached_client().await?;
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
        let input = normalize_cloud_cron_execution_input_for_storage(actor, input);
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

    pub async fn mark_cron_started_execution_failed_by_delivery_key(
        &self,
        actor: &ActorIdentity,
        job_id: &str,
        channel_target: &str,
        heartbeat: bool,
        delivery_key: &str,
        recovered_by: &str,
        reason: &str,
    ) -> HoneResult<usize> {
        let client = self.connect_client().await?;
        let recovered_at = crate::beijing_now_rfc3339();
        let detail = serde_json::json!({
            "phase": "scheduler_handler_watchdog_timeout",
            "recovered_at": recovered_at,
            "recovered_by": recovered_by,
            "delivery_key": delivery_key,
        });
        let updated = client
            .execute(
                r#"
UPDATE cloud_cron_job_runs
SET
  executed_at = $1,
  execution_status = 'execution_failed',
  message_send_status = 'skipped_error',
  should_deliver = false,
  delivered = false,
  response_preview = NULL,
  error_message = $2,
  detail = detail || $3::jsonb
WHERE job_id = $4
  AND actor_channel = $5
  AND actor_user_id = $6
  AND COALESCE(actor_channel_scope, '') = COALESCE($7, '')
  AND channel_target = $8
  AND heartbeat = $9
  AND execution_status = 'running'
  AND message_send_status = 'pending'
  AND detail->>'phase' = 'started'
  AND detail->>'delivery_key' = $10
"#,
                &[
                    &recovered_at,
                    &reason,
                    &detail,
                    &job_id,
                    &actor.channel,
                    &actor.user_id,
                    &actor.channel_scope,
                    &channel_target,
                    &heartbeat,
                    &delivery_key,
                ],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!(
                    "Postgres cron delivery_key watchdog 恢复失败: {err}"
                ))
            })?;
        Ok(updated as usize)
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

    pub async fn upsert_web_push_message(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
        job_id: &str,
        job_name: &str,
        summary: &str,
        content: &str,
        created_at: &str,
    ) -> HoneResult<CloudWebPushMessage> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let row = client
            .query_one(
                r#"
INSERT INTO cloud_web_push_messages (
  actor_storage_key, push_id, job_id, job_name,
  summary, content, created_at
) VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (actor_storage_key, push_id) DO UPDATE SET
  job_id = EXCLUDED.job_id,
  job_name = EXCLUDED.job_name,
  summary = EXCLUDED.summary,
  content = EXCLUDED.content,
  created_at = EXCLUDED.created_at
RETURNING push_id, actor_storage_key, job_id, job_name,
          summary, content, created_at, read_at
"#,
                &[
                    &actor_storage_key,
                    &push_id,
                    &job_id,
                    &job_name,
                    &summary,
                    &content,
                    &created_at,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres Web 推送写入失败: {err}")))?;
        Ok(cloud_web_push_message_from_row(&row))
    }

    pub async fn upsert_web_push_messages(
        &self,
        actor: &ActorIdentity,
        messages: Vec<CloudWebPushMessage>,
    ) -> HoneResult<usize> {
        if messages.is_empty() {
            return Ok(0);
        }
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let push_ids = messages
            .iter()
            .map(|message| message.push_id.clone())
            .collect::<Vec<_>>();
        let job_ids = messages
            .iter()
            .map(|message| message.job_id.clone())
            .collect::<Vec<_>>();
        let job_names = messages
            .iter()
            .map(|message| message.job_name.clone())
            .collect::<Vec<_>>();
        let summaries = messages
            .iter()
            .map(|message| message.summary.clone())
            .collect::<Vec<_>>();
        let contents = messages
            .iter()
            .map(|message| message.content.clone())
            .collect::<Vec<_>>();
        let created_ats = messages
            .iter()
            .map(|message| message.created_at.clone())
            .collect::<Vec<_>>();
        let changed = client
            .execute(
                r#"
INSERT INTO cloud_web_push_messages (
  actor_storage_key, push_id, job_id, job_name,
  summary, content, created_at
) SELECT $1, push_id, job_id, job_name, summary, content, created_at
FROM UNNEST(
  $2::text[], $3::text[], $4::text[], $5::text[], $6::text[], $7::text[]
) AS batch(push_id, job_id, job_name, summary, content, created_at)
ON CONFLICT (actor_storage_key, push_id) DO UPDATE SET
  job_id = EXCLUDED.job_id,
  job_name = EXCLUDED.job_name,
  summary = EXCLUDED.summary,
  content = EXCLUDED.content,
  created_at = EXCLUDED.created_at
"#,
                &[
                    &actor_storage_key,
                    &push_ids,
                    &job_ids,
                    &job_names,
                    &summaries,
                    &contents,
                    &created_ats,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres Web 推送批量写入失败: {err}")))?;
        Ok(changed as usize)
    }

    pub async fn has_legacy_web_push_messages(&self, actor: &ActorIdentity) -> HoneResult<bool> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let row = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM cloud_web_push_messages WHERE actor_storage_key = $1 AND push_id LIKE 'legacy:%')",
                &[&actor_storage_key],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres 历史 Web 推送检查失败: {err}")))?;
        Ok(row.get(0))
    }

    pub async fn list_web_push_messages(
        &self,
        actor: &ActorIdentity,
        before_push_id: Option<String>,
        limit: usize,
    ) -> HoneResult<Vec<CloudWebPushMessage>> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let limit = i64::try_from(limit.max(1)).unwrap_or(100);
        let rows = client
            .query(
                r#"
SELECT push_id, actor_storage_key, job_id, job_name,
       summary, content, created_at, read_at
FROM cloud_web_push_messages
WHERE actor_storage_key = $1
  AND (
    $2::text IS NULL
    OR (created_at, push_id) < (
      SELECT created_at, push_id
      FROM cloud_web_push_messages
      WHERE actor_storage_key = $1 AND push_id = $2
    )
  )
ORDER BY created_at DESC, push_id DESC
LIMIT $3
"#,
                &[&actor_storage_key, &before_push_id, &limit],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres Web 推送列表读取失败: {err}")))?;
        Ok(rows.iter().map(cloud_web_push_message_from_row).collect())
    }

    pub async fn get_web_push_message(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
    ) -> HoneResult<Option<CloudWebPushMessage>> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let row = client
            .query_opt(
                r#"
SELECT push_id, actor_storage_key, job_id, job_name,
       summary, content, created_at, read_at
FROM cloud_web_push_messages
WHERE actor_storage_key = $1 AND push_id = $2
"#,
                &[&actor_storage_key, &push_id],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres Web 推送详情读取失败: {err}")))?;
        Ok(row.as_ref().map(cloud_web_push_message_from_row))
    }

    pub async fn count_unread_web_push_messages(&self, actor: &ActorIdentity) -> HoneResult<usize> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let row = client
            .query_one(
                "SELECT COUNT(*) FROM cloud_web_push_messages WHERE actor_storage_key = $1 AND read_at IS NULL",
                &[&actor_storage_key],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres Web 推送未读统计失败: {err}")))?;
        let count: i64 = row.get(0);
        Ok(usize::try_from(count).unwrap_or(usize::MAX))
    }

    pub async fn mark_web_push_messages_read_through(
        &self,
        actor: &ActorIdentity,
        push_id: &str,
        read_at: &str,
    ) -> HoneResult<usize> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor.storage_key();
        let updated = client
            .execute(
                r#"
UPDATE cloud_web_push_messages
SET read_at = $3
WHERE actor_storage_key = $1
  AND read_at IS NULL
  AND created_at <= (
    SELECT created_at
    FROM cloud_web_push_messages
    WHERE actor_storage_key = $1 AND push_id = $2
  )
"#,
                &[&actor_storage_key, &push_id, &read_at],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres Web 推送已读更新失败: {err}")))?;
        Ok(updated as usize)
    }

    pub async fn get_skill_registry(&self) -> HoneResult<Option<serde_json::Value>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                "SELECT registry FROM cloud_skill_registry WHERE registry_key = 'global'",
                &[],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres skill registry 读取失败: {err}")))?;
        Ok(row.map(|row| row.get(0)))
    }

    pub async fn import_skill_registry(
        &self,
        registry: Option<serde_json::Value>,
    ) -> HoneResult<CloudSkillRegistryImportReport> {
        let Some(registry) = registry else {
            return Ok(CloudSkillRegistryImportReport::default());
        };
        let client = self.connect_client().await?;
        let row = client
            .query_one(
                r#"
WITH upserted AS (
INSERT INTO cloud_skill_registry(registry_key, registry)
VALUES ('global', $1)
ON CONFLICT(registry_key)
DO UPDATE SET registry = EXCLUDED.registry, updated_at = now()
WHERE cloud_skill_registry.registry IS DISTINCT FROM EXCLUDED.registry
RETURNING 1
)
SELECT (SELECT count(*)::bigint FROM upserted)
"#,
                &[&registry],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres skill registry import 失败: {err:?}"))
            })?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        Ok(CloudSkillRegistryImportReport {
            changed_rows,
            skipped_rows: if changed_rows == 0 { 1 } else { 0 },
        })
    }

    pub async fn get_notification_prefs(
        &self,
        actor_storage_key: &str,
    ) -> HoneResult<Option<serde_json::Value>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                "SELECT prefs FROM cloud_notification_prefs WHERE actor_storage_key = $1",
                &[&actor_storage_key],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres notification prefs 读取失败: {err}"))
            })?;
        Ok(row.map(|row| row.get(0)))
    }

    pub async fn get_notification_prefs_many_cached(
        &self,
        actor_storage_keys: &[String],
    ) -> HoneResult<BTreeMap<String, serde_json::Value>> {
        if actor_storage_keys.is_empty() {
            return Ok(BTreeMap::new());
        }
        let client = self.connect_cached_client().await?;
        let rows = client
            .query(
                r#"
SELECT actor_storage_key, prefs
FROM cloud_notification_prefs
WHERE actor_storage_key = ANY($1)
"#,
                &[&actor_storage_keys],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres notification prefs 批量读取失败: {err}"))
            })?;
        Ok(rows
            .into_iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect())
    }

    pub async fn upsert_notification_prefs(
        &self,
        actor_storage_key: &str,
        prefs: serde_json::Value,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_notification_prefs(actor_storage_key, prefs)
VALUES ($1, $2)
ON CONFLICT(actor_storage_key)
DO UPDATE SET prefs = EXCLUDED.prefs, updated_at = now()
"#,
                &[&actor_storage_key, &prefs],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres notification prefs 写入失败: {err}"))
            })?;
        Ok(())
    }

    pub async fn import_notification_prefs(
        &self,
        records: &[CloudNotificationPrefsRecord],
    ) -> HoneResult<CloudNotificationPrefsImportReport> {
        if records.is_empty() {
            return Ok(CloudNotificationPrefsImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(records)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT * FROM jsonb_to_recordset($1::jsonb) AS x(
    actor_storage_key TEXT,
    prefs JSONB
  )
),
upserted AS (
INSERT INTO cloud_notification_prefs(actor_storage_key, prefs)
SELECT actor_storage_key, prefs FROM input_rows
ON CONFLICT(actor_storage_key)
DO UPDATE SET prefs = EXCLUDED.prefs, updated_at = now()
WHERE cloud_notification_prefs.prefs IS DISTINCT FROM EXCLUDED.prefs
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&payload],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres notification prefs import 失败: {err:?}"))
            })?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudNotificationPrefsImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
    }

    pub async fn get_portfolio(
        &self,
        actor_storage_key: &str,
    ) -> HoneResult<Option<CloudPortfolioRecord>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                r#"
SELECT actor_storage_key, actor, portfolio
FROM cloud_portfolios
WHERE actor_storage_key = $1
"#,
                &[&actor_storage_key],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres portfolio 读取失败: {err}")))?;
        Ok(row.map(|row| CloudPortfolioRecord {
            actor_storage_key: row.get(0),
            actor: row.get(1),
            portfolio: row.get(2),
        }))
    }

    pub async fn list_portfolios(&self) -> HoneResult<Vec<CloudPortfolioRecord>> {
        let client = self.connect_client().await?;
        self.list_portfolios_with_client(&client).await
    }

    pub async fn list_portfolios_cached(&self) -> HoneResult<Vec<CloudPortfolioRecord>> {
        let client = self.connect_cached_client().await?;
        self.list_portfolios_with_client(&client).await
    }

    async fn list_portfolios_with_client(
        &self,
        client: &PgClient,
    ) -> HoneResult<Vec<CloudPortfolioRecord>> {
        let rows = client
            .query(
                r#"
SELECT actor_storage_key, actor, portfolio
FROM cloud_portfolios
ORDER BY COALESCE(portfolio->>'updated_at', '') DESC, updated_at DESC
"#,
                &[],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres portfolio 列表读取失败: {err}")))?;
        Ok(rows
            .into_iter()
            .map(|row| CloudPortfolioRecord {
                actor_storage_key: row.get(0),
                actor: row.get(1),
                portfolio: row.get(2),
            })
            .collect())
    }

    pub async fn upsert_portfolio(&self, record: CloudPortfolioRecord) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_portfolios(actor_storage_key, actor, portfolio)
VALUES ($1, $2, $3)
ON CONFLICT(actor_storage_key)
DO UPDATE SET
  actor = EXCLUDED.actor,
  portfolio = EXCLUDED.portfolio,
  updated_at = now()
"#,
                &[&record.actor_storage_key, &record.actor, &record.portfolio],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres portfolio 写入失败: {err}")))?;
        Ok(())
    }

    pub async fn import_portfolios(
        &self,
        records: &[CloudPortfolioRecord],
    ) -> HoneResult<CloudPortfolioImportReport> {
        if records.is_empty() {
            return Ok(CloudPortfolioImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(records)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT * FROM jsonb_to_recordset($1::jsonb) AS x(
    actor_storage_key TEXT,
    actor JSONB,
    portfolio JSONB
  )
),
upserted AS (
INSERT INTO cloud_portfolios(actor_storage_key, actor, portfolio)
SELECT actor_storage_key, actor, portfolio FROM input_rows
ON CONFLICT(actor_storage_key)
DO UPDATE SET
  actor = EXCLUDED.actor,
  portfolio = EXCLUDED.portfolio,
  updated_at = now()
WHERE cloud_portfolios.actor IS DISTINCT FROM EXCLUDED.actor
   OR cloud_portfolios.portfolio IS DISTINCT FROM EXCLUDED.portfolio
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&payload],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres portfolio import 失败: {err:?}")))?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudPortfolioImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
    }

    pub async fn list_company_profile_files(
        &self,
        actor_storage_key: Option<&str>,
    ) -> HoneResult<Vec<CloudCompanyProfileFileRecord>> {
        let client = self.connect_client().await?;
        let actor_storage_key = actor_storage_key.map(str::to_string);
        let rows = client
            .query(
                r#"
SELECT
  actor_storage_key,
  actor,
  profile_id,
  relative_path,
  content,
  updated_at::text
FROM cloud_company_profile_files
WHERE ($1::text IS NULL OR actor_storage_key = $1)
ORDER BY actor_storage_key ASC, profile_id ASC, relative_path ASC
"#,
                &[&actor_storage_key],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres company profile 列表读取失败: {err}"))
            })?;
        Ok(rows
            .into_iter()
            .map(|row| CloudCompanyProfileFileRecord {
                actor_storage_key: row.get(0),
                actor: row.get(1),
                profile_id: row.get(2),
                relative_path: row.get(3),
                content: row.get(4),
                updated_at: row.get(5),
            })
            .collect())
    }

    pub async fn list_company_profile_spaces_cached(
        &self,
    ) -> HoneResult<Vec<CloudCompanyProfileSpaceRecord>> {
        let client = self.connect_cached_client().await?;
        let rows = client
            .query(
                r#"
SELECT
  actor_storage_key,
  actor,
  count(DISTINCT profile_id)::bigint AS profile_count,
  max(updated_at)::text AS updated_at
FROM cloud_company_profile_files
WHERE relative_path = 'profile.md'
GROUP BY actor_storage_key, actor
HAVING count(DISTINCT profile_id) > 0
ORDER BY max(updated_at) DESC, actor_storage_key ASC
"#,
                &[],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!(
                    "Postgres company profile space 列表读取失败: {err}"
                ))
            })?;
        Ok(rows
            .into_iter()
            .map(|row| CloudCompanyProfileSpaceRecord {
                actor_storage_key: row.get(0),
                actor: row.get(1),
                profile_count: row.get::<_, i64>(2).max(0) as usize,
                updated_at: row.get(3),
            })
            .collect())
    }

    pub async fn get_company_profile_file(
        &self,
        actor_storage_key: &str,
        profile_id: &str,
        relative_path: &str,
    ) -> HoneResult<Option<CloudCompanyProfileFileRecord>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                r#"
SELECT
  actor_storage_key,
  actor,
  profile_id,
  relative_path,
  content,
  updated_at::text
FROM cloud_company_profile_files
WHERE actor_storage_key = $1
  AND profile_id = $2
  AND relative_path = $3
"#,
                &[&actor_storage_key, &profile_id, &relative_path],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres company profile 文件读取失败: {err}"))
            })?;
        Ok(row.map(|row| CloudCompanyProfileFileRecord {
            actor_storage_key: row.get(0),
            actor: row.get(1),
            profile_id: row.get(2),
            relative_path: row.get(3),
            content: row.get(4),
            updated_at: row.get(5),
        }))
    }

    pub async fn upsert_company_profile_file(
        &self,
        record: CloudCompanyProfileFileRecord,
    ) -> HoneResult<()> {
        let client = self.connect_client().await?;
        client
            .execute(
                r#"
INSERT INTO cloud_company_profile_files(
  actor_storage_key,
  actor,
  profile_id,
  relative_path,
  content,
  updated_at
)
VALUES ($1, $2, $3, $4, $5, $6::timestamptz)
ON CONFLICT(actor_storage_key, profile_id, relative_path)
DO UPDATE SET
  actor = EXCLUDED.actor,
  content = EXCLUDED.content,
  updated_at = EXCLUDED.updated_at
"#,
                &[
                    &record.actor_storage_key,
                    &record.actor,
                    &record.profile_id,
                    &record.relative_path,
                    &record.content,
                    &record.updated_at,
                ],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres company profile 文件写入失败: {err}"))
            })?;
        Ok(())
    }

    pub async fn delete_company_profile(
        &self,
        actor_storage_key: &str,
        profile_id: &str,
    ) -> HoneResult<bool> {
        let client = self.connect_client().await?;
        let deleted = client
            .execute(
                r#"
DELETE FROM cloud_company_profile_files
WHERE actor_storage_key = $1 AND profile_id = $2
"#,
                &[&actor_storage_key, &profile_id],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres company profile 删除失败: {err}"))
            })?;
        Ok(deleted > 0)
    }

    pub async fn import_company_profile_files(
        &self,
        records: &[CloudCompanyProfileFileRecord],
    ) -> HoneResult<CloudCompanyProfileImportReport> {
        if records.is_empty() {
            return Ok(CloudCompanyProfileImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(records)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT * FROM jsonb_to_recordset($1::jsonb) AS x(
    actor_storage_key TEXT,
    actor JSONB,
    profile_id TEXT,
    relative_path TEXT,
    content TEXT,
    updated_at TEXT
  )
),
upserted AS (
INSERT INTO cloud_company_profile_files(
  actor_storage_key,
  actor,
  profile_id,
  relative_path,
  content,
  updated_at
)
SELECT
  actor_storage_key,
  actor,
  profile_id,
  relative_path,
  content,
  updated_at::timestamptz
FROM input_rows
ON CONFLICT(actor_storage_key, profile_id, relative_path)
DO UPDATE SET
  actor = EXCLUDED.actor,
  content = EXCLUDED.content,
  updated_at = EXCLUDED.updated_at
WHERE cloud_company_profile_files.actor IS DISTINCT FROM EXCLUDED.actor
   OR cloud_company_profile_files.content IS DISTINCT FROM EXCLUDED.content
   OR cloud_company_profile_files.updated_at IS DISTINCT FROM EXCLUDED.updated_at
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&payload],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!("Postgres company profile import 失败: {err:?}"))
            })?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudCompanyProfileImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
    }

    pub async fn upsert_llm_audit_record(&self, record: LlmAuditRecord) -> HoneResult<()> {
        let cloud_record = CloudLlmAuditRecord::from_audit_record(&record)?;
        let client = self.connect_client().await?;
        let payload = cloud_record.payload_json_for_postgres()?;
        let created_at = cloud_record.created_at_text_for_postgres()?;
        client
            .execute(
                r#"
INSERT INTO cloud_llm_audit_records(id, actor_storage_key, record, created_at)
VALUES ($1, $2, $3::text::jsonb, $4::text::timestamptz)
ON CONFLICT(id)
DO UPDATE SET
  actor_storage_key = EXCLUDED.actor_storage_key,
  record = EXCLUDED.record,
  created_at = EXCLUDED.created_at
"#,
                &[
                    &cloud_record.id,
                    &cloud_record.actor_storage_key,
                    &payload,
                    &created_at,
                ],
            )
            .await
            .map_err(|err| {
                HoneError::Config(format!(
                    "Postgres LLM audit 写入失败: id={} actor_storage_key={:?}: {err}",
                    cloud_record.id, cloud_record.actor_storage_key
                ))
            })?;
        Ok(())
    }

    pub async fn get_llm_audit_record(&self, id: &str) -> HoneResult<Option<serde_json::Value>> {
        let client = self.connect_client().await?;
        let row = client
            .query_opt(
                "SELECT record FROM cloud_llm_audit_records WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres LLM audit 详情读取失败: {err}")))?;
        Ok(row.map(|row| row.get(0)))
    }

    pub async fn list_llm_audit_records(
        &self,
        filter: CloudLlmAuditFilter,
    ) -> HoneResult<(Vec<serde_json::Value>, i64)> {
        let client = self.connect_client().await?;
        let page = filter.page.unwrap_or(1).max(1);
        let page_size = filter.page_size.unwrap_or(50).clamp(1, 100);
        let limit = i64::from(page_size);
        let offset = i64::from((page - 1) * page_size);
        let success = filter.success;
        let count_row = client
            .query_one(
                r#"
SELECT count(*)::bigint
FROM cloud_llm_audit_records
WHERE ($1::text IS NULL OR record->'actor'->>'channel' = $1)
  AND ($2::text IS NULL OR record->'actor'->>'user_id' = $2)
  AND ($3::text IS NULL OR COALESCE(record->'actor'->>'channel_scope', '') = $3)
  AND ($4::text IS NULL OR record->>'session_id' = $4)
  AND ($5::boolean IS NULL OR (record->>'success')::boolean = $5)
  AND ($6::text IS NULL OR record->>'source' = $6)
  AND ($7::text IS NULL OR record->>'provider' = $7)
  AND ($8::text IS NULL OR created_at >= $8::timestamptz)
  AND ($9::text IS NULL OR created_at <= $9::timestamptz)
"#,
                &[
                    &filter.actor_channel,
                    &filter.actor_user_id,
                    &filter.actor_scope,
                    &filter.session_id,
                    &success,
                    &filter.source,
                    &filter.provider,
                    &filter.date_from,
                    &filter.date_to,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres LLM audit 计数失败: {err}")))?;
        let total = count_row.get::<_, i64>(0).max(0);
        let rows = client
            .query(
                r#"
SELECT record
FROM cloud_llm_audit_records
WHERE ($1::text IS NULL OR record->'actor'->>'channel' = $1)
  AND ($2::text IS NULL OR record->'actor'->>'user_id' = $2)
  AND ($3::text IS NULL OR COALESCE(record->'actor'->>'channel_scope', '') = $3)
  AND ($4::text IS NULL OR record->>'session_id' = $4)
  AND ($5::boolean IS NULL OR (record->>'success')::boolean = $5)
  AND ($6::text IS NULL OR record->>'source' = $6)
  AND ($7::text IS NULL OR record->>'provider' = $7)
  AND ($8::text IS NULL OR created_at >= $8::timestamptz)
  AND ($9::text IS NULL OR created_at <= $9::timestamptz)
ORDER BY created_at DESC, id DESC
LIMIT $10 OFFSET $11
"#,
                &[
                    &filter.actor_channel,
                    &filter.actor_user_id,
                    &filter.actor_scope,
                    &filter.session_id,
                    &success,
                    &filter.source,
                    &filter.provider,
                    &filter.date_from,
                    &filter.date_to,
                    &limit,
                    &offset,
                ],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres LLM audit 列表读取失败: {err}")))?;
        Ok((rows.into_iter().map(|row| row.get(0)).collect(), total))
    }

    pub async fn import_llm_audit_records(
        &self,
        records: &[CloudLlmAuditRecord],
    ) -> HoneResult<CloudLlmAuditImportReport> {
        if records.is_empty() {
            return Ok(CloudLlmAuditImportReport::default());
        }
        let client = self.connect_client().await?;
        let payload = serde_json::to_value(records)
            .map_err(|err| HoneError::Serialization(err.to_string()))?;
        let row = client
            .query_one(
                r#"
WITH input_rows AS (
  SELECT * FROM jsonb_to_recordset($1::jsonb) AS x(
    id TEXT,
    actor_storage_key TEXT,
    created_at TEXT,
    record JSONB
  )
),
upserted AS (
INSERT INTO cloud_llm_audit_records(id, actor_storage_key, record, created_at)
SELECT id, actor_storage_key, record, created_at::timestamptz
FROM input_rows
ON CONFLICT(id)
DO UPDATE SET
  actor_storage_key = EXCLUDED.actor_storage_key,
  record = EXCLUDED.record,
  created_at = EXCLUDED.created_at
WHERE cloud_llm_audit_records.actor_storage_key IS DISTINCT FROM EXCLUDED.actor_storage_key
   OR cloud_llm_audit_records.record IS DISTINCT FROM EXCLUDED.record
   OR cloud_llm_audit_records.created_at IS DISTINCT FROM EXCLUDED.created_at
RETURNING 1
)
SELECT
  (SELECT count(*)::bigint FROM upserted),
  (SELECT count(*)::bigint FROM input_rows)
"#,
                &[&payload],
            )
            .await
            .map_err(|err| HoneError::Config(format!("Postgres LLM audit import 失败: {err:?}")))?;
        let changed_rows = row.get::<_, i64>(0).max(0) as usize;
        let total_rows = row.get::<_, i64>(1).max(0) as usize;
        Ok(CloudLlmAuditImportReport {
            changed_rows,
            skipped_rows: total_rows.saturating_sub(changed_rows),
        })
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

fn cloud_web_push_message_from_row(row: &tokio_postgres::Row) -> CloudWebPushMessage {
    CloudWebPushMessage {
        push_id: row.get(0),
        actor_storage_key: row.get(1),
        job_id: row.get(2),
        job_name: row.get(3),
        summary: row.get(4),
        content: row.get(5),
        created_at: row.get(6),
        read_at: row.get(7),
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
        self.get_object_with_limit(key, None).await
    }

    pub async fn get_object_limited(
        &self,
        key: &str,
        max_bytes: usize,
    ) -> Result<OssObject, String> {
        self.get_object_with_limit(key, Some(max_bytes)).await
    }

    async fn get_object_with_limit(
        &self,
        key: &str,
        max_bytes: Option<usize>,
    ) -> Result<OssObject, String> {
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
        if let (Some(max_bytes), Some(content_length)) = (max_bytes, response.content_length())
            && content_length > max_bytes as u64
        {
            return Err(format!(
                "OSS 对象大小超过允许上限: {content_length} > {max_bytes}"
            ));
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
            .map_err(|error| format!("OSS 响应读取失败: {error}"))?;
        if let Some(max_bytes) = max_bytes
            && bytes.len() > max_bytes
        {
            return Err(format!(
                "OSS 对象大小超过允许上限: {} > {max_bytes}",
                bytes.len()
            ));
        }
        Ok(OssObject {
            bytes: bytes.to_vec(),
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
    let mut deps = Vec::new();
    if !config.cloud.oss.is_configured() {
        deps.push(config.storage.gen_images_dir.clone());
    }
    if !config.cloud.postgres.is_configured() {
        deps.push("./data/agent-sandboxes".to_string());
        deps.push(config.storage.llm_audit_db_path.clone());
        deps.push(config.storage.portfolio_dir.clone());
        deps.push("./data/runtime/skill_registry.json".to_string());
        deps.push(config.storage.notif_prefs_dir.clone());
        deps.push(config.storage.cron_jobs_dir.clone());
        deps.push(config.storage.sessions_dir.clone());
        deps.push(config.storage.session_sqlite_db_path.clone());
        deps.push(config.storage.conversation_quota_dir.clone());
    }
    deps.retain(|dep| !dep.trim().is_empty());
    deps.sort();
    deps.dedup();
    deps
}

pub fn load_dotenv_if_present() {
    let path = PathBuf::from(".env");
    let _ = crate::harden_private_file(&path);
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
    use bytes::BytesMut;
    use tokio_postgres::types::Json;
    use tokio_postgres::types::{ToSql, Type};

    #[test]
    fn runtime_role_defaults_to_all() {
        unsafe {
            std::env::remove_var("HONE_RUNTIME_ROLE");
        }
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::All);
    }

    #[tokio::test]
    async fn community_backfill_rejects_non_object_audit_before_connecting() {
        let runtime = CloudPgRuntime {
            config: PostgresConfig::default(),
        };
        let update = CloudCommunityResourceBackfillUpdate {
            resource_id: 1,
            expected_updated_at: "2026-07-12 00:00:00+00".to_string(),
            source_resource_id: None,
            content_type: "image/png".to_string(),
            byte_size: 8,
            sha256: "a".repeat(64),
            oss_uri: "oss://bucket/key".to_string(),
            audit_metadata: serde_json::json!([]),
        };
        let error = runtime
            .backfill_community_resource("zsxq", "group", &update)
            .await
            .expect_err("non-object audit must fail before DB connection");
        assert!(error.to_string().contains("JSON object"));
    }

    fn reconcile_candidate(
        source_topic_index: i32,
        source_file_position: Option<i32>,
        fingerprint: &str,
    ) -> CloudCommunityReconcileCandidate {
        CloudCommunityReconcileCandidate {
            source_topic_index,
            source_file_position,
            author_name: "author".to_string(),
            published_at_raw: "2026-07-12 08:30".to_string(),
            body_text: "post".to_string(),
            files: source_file_position
                .map(|_| {
                    vec![CloudCommunityReconcileFile {
                        ordinal: 0,
                        display_name: "report.pdf".to_string(),
                    }]
                })
                .unwrap_or_default(),
            images: Vec::new(),
            candidate_fingerprint: fingerprint.to_string(),
        }
    }

    #[test]
    fn community_reconcile_keys_distinguish_duplicate_topics_by_occurrence() {
        let fingerprint = "a".repeat(64);
        let planned = plan_community_reconcile_candidates(
            "zsxq",
            &[
                reconcile_candidate(0, Some(0), &fingerprint),
                reconcile_candidate(1, Some(1), &fingerprint),
            ],
        )
        .expect("valid complete manifest");
        assert_eq!(planned[0].fingerprint_occurrence, 1);
        assert_eq!(planned[1].fingerprint_occurrence, 2);
        assert_ne!(planned[0].source_item_key, planned[1].source_item_key);
        assert!(planned[1].source_item_key.ends_with(":occurrence:2"));
    }

    #[test]
    fn community_reconcile_rejects_incomplete_file_positions() {
        let error = plan_community_reconcile_candidates(
            "zsxq",
            &[
                reconcile_candidate(0, Some(0), &"a".repeat(64)),
                reconcile_candidate(1, Some(2), &"b".repeat(64)),
            ],
        )
        .expect_err("gap must be rejected");
        assert!(error.to_string().contains("source_file_position"));
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
    fn quota_reserve_sql_returns_inserted_rows_for_new_actor_date() {
        assert!(RESERVE_CONVERSATION_QUOTA_SQL.contains("reserved_count)"));
        assert!(RESERVE_CONVERSATION_QUOTA_SQL.contains("VALUES ($1, $2::text::date, $3, 1)"));
        assert!(
            RESERVE_CONVERSATION_QUOTA_SQL
                .contains("ON CONFLICT (actor_storage_key, quota_date) DO UPDATE")
        );
        assert!(
            RESERVE_CONVERSATION_QUOTA_SQL
                .contains("SELECT reserved, quota_date, limit_count, reserved_count, committed_count FROM inserted")
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
        config.cloud.oss.access_key_id = "access".to_string();
        config.cloud.oss.access_key_secret = "secret".to_string();
        config.cloud.oss.bucket = "bucket".to_string();
        config.cloud.oss.endpoint = "https://example.com".to_string();
        config.storage.sessions_dir = "/tmp/hone/sessions".to_string();
        config.storage.session_sqlite_db_path = "/tmp/hone/sessions.sqlite3".to_string();
        config.storage.conversation_quota_dir = "/tmp/hone/quota".to_string();
        config.storage.cron_jobs_dir = "/tmp/hone/cron".to_string();
        config.storage.gen_images_dir = "/tmp/hone/gen_images".to_string();
        assert!(config.cloud.postgres.is_configured());
        assert!(config.cloud.oss.is_configured());
        let deps = local_durable_dependencies(&config);
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/quota"));
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/sessions"));
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/cron"));
        assert!(!deps.iter().any(|dep| dep == "/tmp/hone/gen_images"));
        assert!(
            !deps
                .iter()
                .any(|dep| dep == &config.storage.session_sqlite_db_path)
        );
        assert!(
            !deps
                .iter()
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

    #[test]
    fn cloud_cron_send_failed_backstop_fills_discord_error_message() {
        let actor = ActorIdentity::new("discord", "g_exec", Some("channel-1")).expect("actor");
        let normalized = normalize_cloud_cron_execution_input_for_storage(
            &actor,
            CloudCronExecutionInput {
                execution_status: "completed".to_string(),
                message_send_status: "send_failed".to_string(),
                should_deliver: true,
                delivered: false,
                response_preview: Some("final report".to_string()),
                error_message: None,
                detail: serde_json::json!({
                    "scheduler": null,
                    "sent_segments": 0,
                    "total_segments": 2,
                }),
            },
        );

        assert_eq!(
            normalized.error_message.as_deref(),
            Some("Discord 定时任务发送失败")
        );
        assert_eq!(normalized.detail["failure_kind"], "discord_send_failed");
    }

    #[test]
    fn cloud_cron_send_failed_backstop_preserves_existing_failure_metadata() {
        let actor = ActorIdentity::new("discord", "g_exec", Some("channel-1")).expect("actor");
        let normalized = normalize_cloud_cron_execution_input_for_storage(
            &actor,
            CloudCronExecutionInput {
                execution_status: "completed".to_string(),
                message_send_status: "send_failed".to_string(),
                should_deliver: true,
                delivered: false,
                response_preview: Some("final report".to_string()),
                error_message: Some("发送 Discord 消息失败: missing access".to_string()),
                detail: serde_json::json!({
                    "scheduler": null,
                    "sent_segments": 0,
                    "total_segments": 2,
                    "failure_kind": "discord_missing_access",
                    "send_error": "missing access",
                }),
            },
        );

        assert_eq!(
            normalized.error_message.as_deref(),
            Some("发送 Discord 消息失败: missing access")
        );
        assert_eq!(normalized.detail["failure_kind"], "discord_missing_access");
        assert_eq!(normalized.detail["send_error"], "missing access");
    }

    #[test]
    fn llm_audit_record_payload_encodes_as_jsonb_parameter() {
        let actor = ActorIdentity::new("web", "audit-user", Some("scope-1")).expect("actor");
        let mut record = LlmAuditRecord::new(
            "session-1",
            Some(actor),
            "function_calling",
            "chat",
            "openrouter",
            Some("gpt-test".to_string()),
            serde_json::json!({
                "messages": [{"role": "user", "content": "hello"}],
            }),
        );
        record.success = true;
        record.response = Some(serde_json::json!({
            "output": [{"type": "text", "text": "world"}],
        }));
        record.metadata = serde_json::json!({
            "tool_calls": [{"name": "search", "arguments": {"q": "hello"}}],
            "heartbeat": {"job_id": "hb-1", "triggered": true},
        });

        let cloud_record = CloudLlmAuditRecord::from_audit_record(&record).expect("cloud record");
        let mut bytes = BytesMut::new();
        Json(&cloud_record.record)
            .to_sql_checked(&Type::JSONB, &mut bytes)
            .expect("jsonb parameter encoding");

        assert!(!bytes.is_empty(), "jsonb payload should not be empty");
    }

    #[test]
    fn llm_audit_record_created_at_encodes_as_timestamptz_parameter() {
        let actor = ActorIdentity::new("web", "audit-user", Some("scope-1")).expect("actor");
        let record = LlmAuditRecord::new(
            "session-1",
            Some(actor),
            "function_calling",
            "chat",
            "openrouter",
            Some("gpt-test".to_string()),
            serde_json::json!({
                "messages": [{"role": "user", "content": "hello"}],
            }),
        );
        let cloud_record = CloudLlmAuditRecord::from_audit_record(&record).expect("cloud record");
        let created_at = DateTime::parse_from_rfc3339(&cloud_record.created_at)
            .expect("created_at should parse as rfc3339")
            .with_timezone(&Utc);
        let mut bytes = BytesMut::new();
        created_at
            .to_sql_checked(&Type::TIMESTAMPTZ, &mut bytes)
            .expect("timestamptz parameter encoding");

        assert!(!bytes.is_empty(), "timestamptz payload should not be empty");
    }

    #[test]
    fn llm_audit_record_uses_text_cast_inputs_for_postgres_insert() {
        let actor = ActorIdentity::new("web", "audit-user", Some("scope-1")).expect("actor");
        let mut record = LlmAuditRecord::new(
            "session-1",
            Some(actor),
            "agent.function_calling",
            "chat",
            "openrouter",
            Some("gpt-test".to_string()),
            serde_json::json!({
                "messages": [{"role": "user", "content": "hello"}],
            }),
        );
        record.success = true;
        record.response = Some(serde_json::json!({
            "output": [{"type": "text", "text": "world"}],
        }));
        record.metadata = serde_json::json!({
            "tool_calls": [{"name": "search", "arguments": {"q": "hello"}}],
            "heartbeat": {"job_id": "hb-1", "triggered": true},
            "budget": {"global_used": 3, "tool_used": 2},
        });

        let cloud_record = CloudLlmAuditRecord::from_audit_record(&record).expect("cloud record");
        let payload = cloud_record
            .payload_json_for_postgres()
            .expect("payload text");
        let created_at = cloud_record
            .created_at_text_for_postgres()
            .expect("created_at text");

        let mut payload_bytes = BytesMut::new();
        payload
            .to_sql_checked(&Type::TEXT, &mut payload_bytes)
            .expect("text payload encoding");
        assert!(
            !payload_bytes.is_empty(),
            "text payload should not be empty"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&payload).expect("payload json"),
            cloud_record.record
        );

        let mut created_at_bytes = BytesMut::new();
        created_at
            .to_sql_checked(&Type::TEXT, &mut created_at_bytes)
            .expect("text timestamp encoding");
        assert!(
            !created_at_bytes.is_empty(),
            "text timestamp payload should not be empty"
        );
        DateTime::parse_from_rfc3339(&created_at).expect("created_at should stay rfc3339");
    }
}
