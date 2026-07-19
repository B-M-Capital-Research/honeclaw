use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::{Args, Subcommand};
use futures::{StreamExt, stream};
use hone_core::cloud_runtime::{
    CloudCommunityPublishLock, CloudCommunityReconcileCandidate,
    CloudCommunityResourceBackfillOutcome, CloudCommunityResourceBackfillTarget,
    CloudCommunityResourceBackfillUpdate, CloudCompanyProfileFileRecord,
    CloudConversationQuotaImport, CloudCronJobRecord, CloudDocumentIndex, CloudLlmAuditRecord,
    CloudNotificationPrefsRecord, CloudPgRuntime, CloudPortfolioRecord, CloudSessionRecord,
    OssObjectStore, RuntimeRole, local_durable_dependencies, sanitize_key_component, sha256_hex,
};
use hone_core::config::OssConfig;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use walkdir::WalkDir;

use crate::common::load_cli_config;
use crate::yaml_io::print_json;

#[derive(Subcommand, Debug)]
pub(crate) enum CloudCommands {
    /// 检查 cloud.mode、runtime role、PG、OSS、schema 和本地 durable 依赖。
    Doctor(CloudDoctorArgs),
    /// 从本机 data/ dry-run 或幂等导入 PG/OSS。
    Migrate(CloudMigrateArgs),
    /// 对比当前 Aliyun OSS 和 Cloudflare R2 的小对象读写延迟。
    ObjectBench(ObjectBenchArgs),
    /// 校验本地 manifest，并将社区资源幂等回填到 OSS/Postgres（默认 dry-run）。
    CommunityAssets(CommunityAssetsArgs),
    /// 对账完整社区时间线，并在单事务中补齐缺失的内容/资源元数据（默认 dry-run）。
    CommunityContents(CommunityContentsArgs),
    /// 将社区只读时间线发布为 private-R2 edge snapshot（默认 dry-run）。
    CommunityPublish(CommunityPublishArgs),
}

#[derive(Args, Debug)]
pub(crate) struct CloudDoctorArgs {
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long = "ensure-schema")]
    pub(crate) ensure_schema: bool,
}

#[derive(Args, Debug)]
pub(crate) struct CloudMigrateArgs {
    #[arg(long = "from-data-dir", value_name = "DIR")]
    pub(crate) from_data_dir: PathBuf,
    #[arg(long = "upload-oss")]
    pub(crate) upload_oss: bool,
    /// Reuse existing OSS objects after a HEAD check instead of blindly overwriting.
    #[arg(long = "reuse-existing")]
    pub(crate) reuse_existing: bool,
    /// Number of concurrent object uploads. Applies only with --upload-oss --apply.
    #[arg(long, default_value_t = 6)]
    pub(crate) concurrency: usize,
    /// Only import conversation quota JSON into PG; skip object uploads and document indexing.
    #[arg(long = "quota-only")]
    pub(crate) quota_only: bool,
    /// Only import session JSON into PG; skip object uploads and document indexing.
    #[arg(long = "session-only")]
    pub(crate) session_only: bool,
    /// Only import Web invite users and auth sessions from the configured SQLite DB into PG.
    #[arg(long = "web-auth-only")]
    pub(crate) web_auth_only: bool,
    /// Only import cron job JSON into PG.
    #[arg(long = "cron-only")]
    pub(crate) cron_only: bool,
    /// Only import runtime skill registry JSON into PG.
    #[arg(long = "skill-registry-only")]
    pub(crate) skill_registry_only: bool,
    /// Only import notification preferences JSON into PG.
    #[arg(long = "notification-prefs-only")]
    pub(crate) notification_prefs_only: bool,
    /// Only import portfolio JSON into PG.
    #[arg(long = "portfolio-only")]
    pub(crate) portfolio_only: bool,
    /// Only import LLM audit SQLite rows into PG.
    #[arg(long = "llm-audit-only")]
    pub(crate) llm_audit_only: bool,
    /// Only import actor-scoped company profile markdown files into PG.
    #[arg(long = "company-profiles-only")]
    pub(crate) company_profiles_only: bool,
    #[arg(long)]
    pub(crate) apply: bool,
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args, Debug)]
pub(crate) struct ObjectBenchArgs {
    #[arg(long, default_value_t = 64)]
    pub(crate) size_kib: usize,
    #[arg(long, default_value_t = 3)]
    pub(crate) iterations: usize,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub(crate) cleanup: bool,
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args, Debug)]
pub(crate) struct CommunityAssetsArgs {
    /// JSON array containing verified local community asset records.
    #[arg(long, value_name = "FILE")]
    pub(crate) manifest: PathBuf,
    #[arg(long, default_value = "zsxq")]
    pub(crate) source: String,
    #[arg(long = "external-id", default_value = "51115212285814")]
    pub(crate) external_id: String,
    /// Maximum accepted bytes for each local file.
    #[arg(long = "max-bytes", default_value_t = 134_217_728)]
    pub(crate) max_bytes: u64,
    /// Upload verified objects and update PostgreSQL. Omit for read-only dry-run.
    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(Args, Debug)]
pub(crate) struct CommunityContentsArgs {
    /// Complete source timeline manifest, including file and non-file topics.
    #[arg(long, value_name = "FILE")]
    pub(crate) manifest: PathBuf,
    #[arg(long, default_value = "zsxq")]
    pub(crate) source: String,
    #[arg(long = "external-id", default_value = "51115212285814")]
    pub(crate) external_id: String,
    /// Insert all missing contents/resources in one PostgreSQL transaction.
    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(Args, Debug)]
pub(crate) struct CommunityPublishArgs {
    #[arg(long, default_value = "zsxq")]
    pub(crate) source: String,
    #[arg(long = "external-id", default_value = "51115212285814")]
    pub(crate) external_id: String,
    /// Number of contents in each edge feed page.
    #[arg(long = "page-size", default_value_t = 20)]
    pub(crate) page_size: usize,
    /// Private R2 prefix containing the published feed and descriptors.
    #[arg(
        long = "feed-prefix",
        default_value = "community/zsxq/51115212285814/delivery/v1"
    )]
    pub(crate) feed_prefix: String,
    /// Private R2 prefix containing the already-managed original resources.
    #[arg(
        long = "asset-prefix",
        default_value = "community/zsxq/51115212285814/resources"
    )]
    pub(crate) asset_prefix: String,
    /// Write descriptors/pages and update latest.json last. Omit for read-only dry-run.
    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(Debug, Serialize)]
struct CloudDoctorReport {
    cloud_mode: String,
    cloud_enabled: bool,
    strict_no_local_storage: bool,
    runtime_role: String,
    postgres_configured: bool,
    postgres_proxy_configured: bool,
    postgres_health: Option<hone_core::cloud_runtime::CloudHealth>,
    schema_ensured: bool,
    oss_configured: bool,
    oss_proxy_configured: bool,
    oss_health: Option<hone_core::cloud_runtime::CloudHealth>,
    local_durable_dependency_count: usize,
    local_durable_dependencies: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct MigrationCounts {
    sessions: usize,
    uploads_and_attachments: usize,
    generated_images: usize,
    company_profiles: usize,
    portfolio_json: usize,
    cron_json: usize,
    notification_prefs: usize,
    quota_json: usize,
    skill_registry_json: usize,
    sqlite_files: usize,
    other_files: usize,
}

#[derive(Debug, Serialize)]
struct MigrationReport {
    mode: &'static str,
    from_data_dir: String,
    upload_oss: bool,
    reuse_existing: bool,
    concurrency: usize,
    postgres_configured: bool,
    oss_configured: bool,
    counted: MigrationCounts,
    uploaded_objects: usize,
    reused_objects: usize,
    indexed_documents: usize,
    changed_quota_rows: usize,
    skipped_quota_rows: usize,
    changed_session_rows: usize,
    skipped_session_rows: usize,
    changed_web_auth_users: usize,
    skipped_web_auth_users: usize,
    changed_web_auth_sessions: usize,
    skipped_web_auth_sessions: usize,
    changed_cron_rows: usize,
    skipped_cron_rows: usize,
    changed_skill_registry_rows: usize,
    skipped_skill_registry_rows: usize,
    changed_notification_prefs_rows: usize,
    skipped_notification_prefs_rows: usize,
    changed_portfolio_rows: usize,
    skipped_portfolio_rows: usize,
    changed_company_profile_files: usize,
    skipped_company_profile_files: usize,
    changed_llm_audit_rows: usize,
    skipped_llm_audit_rows: usize,
    skipped_objects: usize,
    conflicts: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct LegacyQuotaJson {
    #[serde(default)]
    quota_date: String,
    #[serde(default)]
    success_count: u32,
    #[serde(default)]
    in_flight: u32,
}

#[derive(Debug, Serialize)]
struct ObjectBenchReport {
    size_kib: usize,
    iterations: usize,
    results: Vec<ObjectBenchProviderReport>,
}

#[derive(Debug, Serialize)]
struct ObjectBenchProviderReport {
    provider: String,
    configured: bool,
    ok: bool,
    bucket: Option<String>,
    endpoint: Option<String>,
    proxy_configured: bool,
    iterations: Vec<ObjectBenchIteration>,
    avg_put_ms: Option<u128>,
    avg_head_ms: Option<u128>,
    avg_get_ms: Option<u128>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ObjectBenchIteration {
    put_ms: u128,
    head_ms: u128,
    get_ms: u128,
    bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct CommunityAssetManifestEntry {
    resource_id: i64,
    path: PathBuf,
    content_type: String,
    byte_size: u64,
    sha256: String,
    #[serde(default)]
    source_base_key: Option<String>,
    #[serde(default)]
    source_resource_id: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    captured_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommunityAssetsReport {
    ok: bool,
    mode: &'static str,
    manifest: String,
    source: String,
    external_id: String,
    total: usize,
    validated: usize,
    uploaded: usize,
    reused: usize,
    updated: usize,
    skipped: usize,
    would_upload: usize,
    would_update: usize,
    conflicts: Vec<CommunityAssetConflict>,
    items: Vec<CommunityAssetReportItem>,
}

#[derive(Debug, Serialize)]
struct CommunityAssetConflict {
    resource_id: Option<i64>,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CommunityAssetReportItem {
    resource_id: i64,
    action: &'static str,
    byte_size: u64,
    sha256: String,
    oss_key: Option<String>,
}

struct ValidatedCommunityAsset {
    resource_id: i64,
    path: PathBuf,
    content_type: String,
    byte_size: u64,
    sha256: String,
    extension: &'static str,
    source_base_key: Option<String>,
    source_resource_id: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    captured_at: Option<String>,
}

const COMMUNITY_PUBLISH_AUTHOR: &str = "HONE 官方";
const COMMUNITY_PUBLISH_JSON_CONTENT_TYPE: &str = "application/json; charset=utf-8";
// Publisher verification downloads and hashes archived assets before exposing
// them to the edge. Keep this deliberately low: historical resources may be
// as large as the 128 MiB community backfill ceiling.
const COMMUNITY_PUBLISH_RESOURCE_CONCURRENCY: usize = 2;
const COMMUNITY_PUBLISH_MAX_FEED_BYTES: usize = 8 * 1024 * 1024;
const COMMUNITY_PUBLISH_MAX_DESCRIPTOR_BYTES: usize = 64 * 1024;
const COMMUNITY_PUBLISH_MAX_ACTIVE_INDEX_BYTES: usize = 1024 * 1024;
const COMMUNITY_PUBLISH_MAX_ASSET_BYTES: i64 = 128 * 1024 * 1024;
const COMMUNITY_PUBLISH_OSS_ATTEMPTS: usize = 3;
const COMMUNITY_PUBLISH_OSS_RETRY_BASE_DELAY_MS: u64 = 200;
const MAX_JAVASCRIPT_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

#[derive(Debug, Serialize)]
struct CommunityPublishReport {
    ok: bool,
    mode: &'static str,
    source: String,
    external_id: String,
    page_size: usize,
    feed_prefix: String,
    asset_prefix: String,
    latest_key: String,
    content_count: usize,
    page_count: usize,
    resource_count: usize,
    edge_resource_count: usize,
    legacy_resource_count: usize,
    descriptor_count: usize,
    planned_objects: usize,
    existing_objects: usize,
    would_write: usize,
    written: usize,
    latest_updated: bool,
    no_op: bool,
    legacy_resources: Vec<CommunityPublishLegacyResource>,
    conflicts: Vec<CommunityPublishConflict>,
    resource_verification: &'static str,
    verification_hint: &'static str,
}

#[derive(Debug, Serialize)]
struct CommunityPublishLegacyResource {
    resource_id: i64,
    reason: String,
}

#[derive(Debug, Serialize)]
struct CommunityPublishConflict {
    resource_id: Option<i64>,
    object_key: Option<String>,
    reason: String,
}

#[derive(Debug)]
struct CommunityPublishRawPage {
    before: Option<i64>,
    items: Vec<hone_core::cloud_runtime::CloudCommunityContentRecord>,
}

#[derive(Debug, Serialize)]
struct PublishedCommunityPage {
    community: PublishedCommunityIdentity,
    items: Vec<PublishedCommunityContent>,
    next_before: Option<i64>,
    unread: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_content_id: Option<i64>,
}

#[derive(Debug, Serialize)]
struct PublishedCommunityIdentity {
    id: String,
    name: &'static str,
}

#[derive(Debug, Serialize)]
struct PublishedCommunityContent {
    content_id: i64,
    author_name: &'static str,
    published_at: Option<String>,
    published_at_raw: Option<String>,
    content_type: String,
    body_text: String,
    body_blocks: serde_json::Value,
    crawl_status: String,
    resources: Vec<PublishedCommunityResource>,
}

#[derive(Debug, Serialize)]
struct PublishedCommunityResource {
    resource_id: i64,
    ordinal: i32,
    resource_kind: String,
    display_name: Option<String>,
    content_type: Option<String>,
    byte_size: Option<i64>,
    version: Option<String>,
    access_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    delivery_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CommunityResourceDescriptor {
    resource_id: i64,
    version: String,
    sha256: String,
    object_key: String,
    content_type: String,
    byte_size: i64,
    display_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommunityResourceActiveIndex {
    v: u8,
    resources: BTreeMap<i64, String>,
}

#[derive(Debug)]
enum CommunityEdgeResourceCheck {
    Eligible(CommunityResourceDescriptor),
    Legacy { resource_id: i64, reason: String },
    Conflict { resource_id: i64, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommunityPublishObjectKind {
    Descriptor,
    Page,
    ActiveIndex,
    Latest,
}

#[derive(Debug)]
struct PlannedCommunityPublishObject {
    kind: CommunityPublishObjectKind,
    key: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct PreflightCommunityPublishObject {
    planned: PlannedCommunityPublishObject,
    existed: bool,
    should_write: bool,
}

pub(crate) async fn run_cloud_command(
    config_path: Option<&Path>,
    command: CloudCommands,
) -> Result<(), String> {
    match command {
        CloudCommands::Doctor(args) => run_doctor(config_path, args).await,
        CloudCommands::Migrate(args) => run_migrate(config_path, args).await,
        CloudCommands::ObjectBench(args) => run_object_bench(config_path, args).await,
        CloudCommands::CommunityAssets(args) => run_community_assets(config_path, args).await,
        CloudCommands::CommunityContents(args) => run_community_contents(config_path, args).await,
        CloudCommands::CommunityPublish(args) => run_community_publish(config_path, args).await,
    }
}

fn validate_community_publish_scope(source: &str, external_id: &str) -> Result<(), String> {
    if source.is_empty() || external_id.is_empty() {
        return Err("--source 和 --external-id 不能为空".to_string());
    }
    if sanitize_key_component(source) != source
        || sanitize_key_component(external_id) != external_id
        || matches!(source, "." | "..")
        || matches!(external_id, "." | "..")
    {
        return Err(
            "--source 和 --external-id 只能包含 ASCII 字母、数字、点、横线和下划线，且不能是 . 或 .."
                .to_string(),
        );
    }
    Ok(())
}

fn validate_community_publish_provider(raw: &str) -> Result<(), String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "r2" | "cloudflare_r2" => Ok(()),
        _ => Err(
            "community-publish 只允许 active OSS provider 明确设为 r2 或 cloudflare_r2".to_string(),
        ),
    }
}

fn validate_community_publish_page_size(page_size: usize) -> Result<(), String> {
    if (1..=50).contains(&page_size) {
        Ok(())
    } else {
        Err("--page-size 必须在 1..=50 范围内".to_string())
    }
}

fn normalize_community_publish_prefix(raw: &str, label: &str) -> Result<String, String> {
    let prefix = raw.trim().trim_matches('/');
    if prefix.is_empty() {
        return Err(format!("{label} 不能为空"));
    }
    validate_community_object_key(prefix).map_err(|reason| format!("{label} 无效: {reason}"))?;
    Ok(prefix.to_string())
}

fn validate_community_publish_scoped_prefix(
    prefix: &str,
    source: &str,
    external_id: &str,
    label: &str,
) -> Result<(), String> {
    let root = format!("community/{source}/{external_id}");
    if prefix == root || prefix.starts_with(&format!("{root}/")) {
        Ok(())
    } else {
        Err(format!("{label} 必须位于 {root}/ 下"))
    }
}

fn validate_community_object_key(key: &str) -> Result<(), String> {
    if key.starts_with('/') || key.ends_with('/') || key.contains('\\') || key.contains('\0') {
        return Err("object key 不能含首尾斜线、反斜线或 NUL".to_string());
    }
    if key
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err("object key 不能含空段、. 或 .. 段".to_string());
    }
    if key.chars().any(char::is_control) {
        return Err("object key 不能含控制字符".to_string());
    }
    Ok(())
}

fn community_publish_latest_key(feed_prefix: &str) -> String {
    format!("{feed_prefix}/feed/latest.json")
}

fn community_publish_page_key(feed_prefix: &str, before: i64) -> String {
    format!("{feed_prefix}/feed/pages/{before}.json")
}

fn community_publish_descriptor_key(feed_prefix: &str, resource_id: i64, version: &str) -> String {
    format!("{feed_prefix}/resources/{resource_id}/{version}.json")
}

fn community_publish_active_index_key(feed_prefix: &str) -> String {
    format!("{feed_prefix}/resources/active.json")
}

fn community_publish_delivery_path(resource_id: i64, version: &str) -> String {
    format!("/_community/v1/resources/{resource_id}/{version}")
}

fn normalized_community_publish_sha256(raw: Option<&str>) -> Option<String> {
    let sha256 = raw?.trim();
    (sha256.len() == 64 && sha256.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| sha256.to_ascii_lowercase())
}

fn validate_non_empty_community_publish_archive(
    pages: &[CommunityPublishRawPage],
) -> Result<(), String> {
    if pages.first().is_some_and(|page| !page.items.is_empty()) {
        Ok(())
    } else {
        Err("community archive 为空，拒绝发布或覆盖 latest.json".to_string())
    }
}

async fn load_community_publish_pages(
    pg: &CloudPgRuntime,
    source: &str,
    external_id: &str,
    page_size: usize,
) -> Result<Vec<CommunityPublishRawPage>, String> {
    let mut pages = Vec::new();
    let mut before = None;
    let mut seen_cursors = BTreeSet::new();
    loop {
        let items = pg
            .list_community_contents(source, external_id, before, page_size)
            .await
            .map_err(|_| "读取 Postgres community timeline 失败".to_string())?;
        if items.is_empty() {
            break;
        }
        for item in &items {
            if item.content_id <= 0 || item.content_id > MAX_JAVASCRIPT_SAFE_INTEGER {
                return Err(format!(
                    "community content_id={} 超出安全正整数范围",
                    item.content_id
                ));
            }
        }
        let next_before = (items.len() == page_size)
            .then(|| items.last().map(|item| item.content_id))
            .flatten();
        pages.push(CommunityPublishRawPage { before, items });
        let Some(next_before) = next_before else {
            break;
        };
        if !seen_cursors.insert(next_before) {
            return Err(format!(
                "community timeline cursor={next_before} 重复，拒绝生成循环分页"
            ));
        }
        before = Some(next_before);
    }
    validate_non_empty_community_publish_archive(&pages)?;
    Ok(pages)
}

fn validate_community_resource_descriptor(
    descriptor: &CommunityResourceDescriptor,
    asset_prefix: &str,
) -> Result<(), String> {
    if descriptor.resource_id <= 0 || descriptor.resource_id > MAX_JAVASCRIPT_SAFE_INTEGER {
        return Err("resource_id 必须是 JavaScript 安全正整数".to_string());
    }
    if descriptor.version.len() != 12
        || !descriptor
            .version
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("version 必须是 12 位小写十六进制".to_string());
    }
    if descriptor.sha256.len() != 64
        || !descriptor
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("sha256 必须是 64 位小写十六进制".to_string());
    }
    if descriptor.version != descriptor.sha256[..12] {
        return Err("version 必须等于 sha256 前 12 位".to_string());
    }
    validate_community_object_key(&descriptor.object_key)?;
    validate_community_publish_asset_key(descriptor, asset_prefix)?;
    let content_type = descriptor.content_type.trim();
    if content_type.is_empty()
        || content_type.len() > 255
        || content_type.chars().any(char::is_control)
    {
        return Err("content_type 无效".to_string());
    }
    if !(1..=COMMUNITY_PUBLISH_MAX_ASSET_BYTES).contains(&descriptor.byte_size) {
        return Err(format!(
            "byte_size 必须在 1..={COMMUNITY_PUBLISH_MAX_ASSET_BYTES} 范围内"
        ));
    }
    if descriptor.display_name.as_deref().is_some_and(|name| {
        name.len() > 1_024 || name.contains('\0') || name.contains('\r') || name.contains('\n')
    }) {
        return Err("display_name 含不安全字符或超过 1024 UTF-8 bytes".to_string());
    }
    Ok(())
}

fn normalized_community_publish_content_type(raw: &str) -> String {
    let normalized = raw
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if normalized == "image/jpg" {
        "image/jpeg".to_string()
    } else {
        normalized
    }
}

fn validate_community_publish_asset_key(
    descriptor: &CommunityResourceDescriptor,
    asset_prefix: &str,
) -> Result<(), String> {
    let prefix = format!("{asset_prefix}/");
    let Some(basename) = descriptor.object_key.strip_prefix(&prefix) else {
        return Err("object_key 不在 exact asset-prefix 下".to_string());
    };
    if basename.contains('/') {
        return Err("object_key 必须直接位于 asset-prefix 下，不能含额外子目录".to_string());
    }
    let immutable_stem = format!("{}-{}.", descriptor.resource_id, descriptor.sha256);
    let Some(extension) = basename.strip_prefix(&immutable_stem) else {
        return Err(
            "object_key basename 必须匹配 {resource_id}-{full_sha256}.<safe ext>".to_string(),
        );
    };
    if extension.is_empty()
        || extension.len() > 10
        || !extension
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return Err("object_key 扩展名必须是安全的小写字母数字".to_string());
    }
    let expected_content_type = content_type_for_extension(extension)
        .ok_or_else(|| format!("object_key 扩展名 .{extension} 不在 community asset allowlist"))?;
    let descriptor_content_type =
        normalized_community_publish_content_type(&descriptor.content_type);
    if descriptor_content_type != expected_content_type {
        return Err(format!(
            "object_key 扩展名 .{extension} 与 descriptor content_type 不一致"
        ));
    }
    Ok(())
}

fn validate_community_publish_asset_bytes(
    descriptor: &CommunityResourceDescriptor,
    bytes: &[u8],
    object_content_type: &str,
) -> Result<(), String> {
    let actual_size = i64::try_from(bytes.len())
        .map_err(|_| "R2 immutable object 大小无法转换为 i64".to_string())?;
    if actual_size != descriptor.byte_size {
        return Err(format!(
            "R2 immutable object 大小冲突: expected={} actual={actual_size}",
            descriptor.byte_size
        ));
    }
    let actual_sha256 = sha256_hex(bytes);
    if actual_sha256 != descriptor.sha256 {
        return Err(format!(
            "R2 immutable object sha256 冲突: expected={} actual={actual_sha256}",
            descriptor.sha256
        ));
    }
    let actual_content_type = normalized_community_publish_content_type(object_content_type);
    let expected_content_type = normalized_community_publish_content_type(&descriptor.content_type);
    if actual_content_type != expected_content_type {
        return Err(format!(
            "R2 immutable object content_type 冲突: expected={expected_content_type} actual={actual_content_type}"
        ));
    }
    Ok(())
}

async fn community_publish_with_retry<T, Operation, OperationFuture>(
    attempts: usize,
    retry_base_delay_ms: u64,
    empty_error: &str,
    mut operation: Operation,
) -> Result<T, String>
where
    Operation: FnMut() -> OperationFuture,
    OperationFuture: std::future::Future<Output = Result<T, String>>,
{
    let mut last_error = None;
    for attempt in 1..=attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => last_error = Some(error),
        }
        if attempt < attempts {
            tokio::time::sleep(Duration::from_millis(
                retry_base_delay_ms.saturating_mul(attempt as u64),
            ))
            .await;
        }
    }
    Err(last_error.unwrap_or_else(|| empty_error.to_string()))
}

async fn community_publish_object_exists_with_retry(
    oss: &OssObjectStore,
    key: &str,
) -> Result<bool, String> {
    community_publish_with_retry(
        COMMUNITY_PUBLISH_OSS_ATTEMPTS,
        COMMUNITY_PUBLISH_OSS_RETRY_BASE_DELAY_MS,
        "R2 HEAD 未返回结果",
        || oss.object_exists(key),
    )
    .await
}

async fn community_publish_get_object_with_retry(
    oss: &OssObjectStore,
    key: &str,
    max_bytes: usize,
) -> Result<hone_core::cloud_runtime::OssObject, String> {
    community_publish_with_retry(
        COMMUNITY_PUBLISH_OSS_ATTEMPTS,
        COMMUNITY_PUBLISH_OSS_RETRY_BASE_DELAY_MS,
        "R2 GET 未返回结果",
        || oss.get_object_limited(key, max_bytes),
    )
    .await
}

async fn community_publish_put_object_with_retry(
    oss: &OssObjectStore,
    key: &str,
    bytes: &[u8],
) -> Result<(), String> {
    community_publish_with_retry(
        COMMUNITY_PUBLISH_OSS_ATTEMPTS,
        COMMUNITY_PUBLISH_OSS_RETRY_BASE_DELAY_MS,
        "R2 PUT 未返回结果",
        || oss.put_object(key, bytes.to_vec(), COMMUNITY_PUBLISH_JSON_CONTENT_TYPE),
    )
    .await
}

async fn verify_community_publish_asset_object(
    oss: &OssObjectStore,
    descriptor: &CommunityResourceDescriptor,
) -> Result<(), String> {
    let max_bytes = usize::try_from(descriptor.byte_size)
        .map_err(|_| "descriptor byte_size 无法转换为本机 usize".to_string())?;
    let object = community_publish_get_object_with_retry(oss, &descriptor.object_key, max_bytes)
        .await
        .map_err(|_| "R2 immutable object GET 预检失败（endpoint 已隐藏）".to_string())?;
    validate_community_publish_asset_bytes(descriptor, &object.bytes, &object.content_type)
}

async fn inspect_community_edge_resource(
    oss: &OssObjectStore,
    asset_prefix: &str,
    resource: hone_core::cloud_runtime::CloudCommunityResourceRecord,
    verify_bytes: bool,
) -> CommunityEdgeResourceCheck {
    let resource_id = resource.resource_id;
    if resource.access_state != "stored" {
        return CommunityEdgeResourceCheck::Legacy {
            resource_id,
            reason: "access_state 不是 stored".to_string(),
        };
    }
    let Some(sha256) = normalized_community_publish_sha256(resource.sha256.as_deref()) else {
        return CommunityEdgeResourceCheck::Legacy {
            resource_id,
            reason: "缺少有效 sha256".to_string(),
        };
    };
    let Some(object_key) = resource
        .oss_uri
        .as_deref()
        .and_then(|uri| oss.parse_managed_uri(uri))
        .map(str::to_string)
    else {
        return CommunityEdgeResourceCheck::Legacy {
            resource_id,
            reason: "缺少当前 R2 bucket 的 managed URI".to_string(),
        };
    };
    let Some(content_type) = resource
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return CommunityEdgeResourceCheck::Legacy {
            resource_id,
            reason: "缺少 content_type".to_string(),
        };
    };
    let Some(byte_size) = resource.byte_size else {
        return CommunityEdgeResourceCheck::Legacy {
            resource_id,
            reason: "缺少 byte_size".to_string(),
        };
    };
    if !(1..=COMMUNITY_PUBLISH_MAX_ASSET_BYTES).contains(&byte_size) {
        return CommunityEdgeResourceCheck::Legacy {
            resource_id,
            reason: format!(
                "byte_size 不在 edge delivery 1..={COMMUNITY_PUBLISH_MAX_ASSET_BYTES} 范围内"
            ),
        };
    }
    let descriptor = CommunityResourceDescriptor {
        resource_id,
        version: sha256[..12].to_string(),
        sha256,
        object_key,
        content_type,
        byte_size,
        display_name: resource.display_name,
    };
    if let Err(reason) = validate_community_resource_descriptor(&descriptor, asset_prefix) {
        return CommunityEdgeResourceCheck::Conflict {
            resource_id,
            reason: format!("descriptor 无效: {reason}"),
        };
    }
    let verification = if verify_bytes {
        verify_community_publish_asset_object(oss, &descriptor).await
    } else {
        match community_publish_object_exists_with_retry(oss, &descriptor.object_key).await {
            Ok(true) => Ok(()),
            Ok(false) => Err("R2 immutable object HEAD 预检发现对象不存在".to_string()),
            Err(_) => Err("R2 immutable object HEAD 预检失败（endpoint 已隐藏）".to_string()),
        }
    };
    match verification {
        Ok(()) => CommunityEdgeResourceCheck::Eligible(descriptor),
        Err(reason) => CommunityEdgeResourceCheck::Conflict {
            resource_id,
            reason,
        },
    }
}

fn community_edge_resource_check_id(check: &CommunityEdgeResourceCheck) -> i64 {
    match check {
        CommunityEdgeResourceCheck::Eligible(descriptor) => descriptor.resource_id,
        CommunityEdgeResourceCheck::Legacy { resource_id, .. }
        | CommunityEdgeResourceCheck::Conflict { resource_id, .. } => *resource_id,
    }
}

fn build_published_community_pages(
    raw_pages: Vec<CommunityPublishRawPage>,
    external_id: &str,
    descriptors: &BTreeMap<i64, CommunityResourceDescriptor>,
) -> Vec<(Option<i64>, PublishedCommunityPage)> {
    let latest_content_id = raw_pages
        .first()
        .and_then(|page| page.items.first())
        .map(|item| item.content_id);
    let page_cursors = raw_pages.iter().map(|page| page.before).collect::<Vec<_>>();
    raw_pages
        .into_iter()
        .enumerate()
        .map(|(page_index, page)| {
            let next_before = page_cursors.get(page_index + 1).copied().flatten();
            let items = page
                .items
                .into_iter()
                .map(|item| PublishedCommunityContent {
                    content_id: item.content_id,
                    author_name: COMMUNITY_PUBLISH_AUTHOR,
                    published_at: item.published_at,
                    published_at_raw: item.published_at_raw,
                    content_type: item.content_type,
                    body_text: item.body_text,
                    body_blocks: item.body_blocks,
                    crawl_status: item.crawl_status,
                    resources: item
                        .resources
                        .into_iter()
                        .map(|resource| {
                            let version =
                                normalized_community_publish_sha256(resource.sha256.as_deref())
                                    .map(|sha256| sha256[..12].to_string());
                            let delivery_path =
                                descriptors.get(&resource.resource_id).map(|descriptor| {
                                    community_publish_delivery_path(
                                        descriptor.resource_id,
                                        &descriptor.version,
                                    )
                                });
                            PublishedCommunityResource {
                                resource_id: resource.resource_id,
                                ordinal: resource.ordinal,
                                resource_kind: resource.resource_kind,
                                display_name: resource.display_name,
                                content_type: resource.content_type,
                                byte_size: resource.byte_size,
                                version,
                                access_state: resource.access_state,
                                delivery_path,
                            }
                        })
                        .collect(),
                })
                .collect();
            (
                page.before,
                PublishedCommunityPage {
                    community: PublishedCommunityIdentity {
                        id: external_id.to_string(),
                        name: "HONE 官方社区",
                    },
                    items,
                    next_before,
                    unread: false,
                    latest_content_id: (page_index == 0).then_some(latest_content_id).flatten(),
                },
            )
        })
        .collect()
}

fn order_community_publish_objects(
    mut descriptors: Vec<PlannedCommunityPublishObject>,
    active_index: PlannedCommunityPublishObject,
    pages: Vec<PlannedCommunityPublishObject>,
) -> Result<Vec<PlannedCommunityPublishObject>, String> {
    if descriptors
        .iter()
        .any(|object| object.kind != CommunityPublishObjectKind::Descriptor)
    {
        return Err("community descriptor publication plan 类型无效".to_string());
    }
    if active_index.kind != CommunityPublishObjectKind::ActiveIndex {
        return Err("community active index publication plan 类型无效".to_string());
    }
    descriptors.sort_by(|left, right| left.key.cmp(&right.key));
    let mut immutable_pages = Vec::new();
    let mut latest = None;
    for page in pages {
        match page.kind {
            CommunityPublishObjectKind::Page => immutable_pages.push(page),
            CommunityPublishObjectKind::Latest if latest.is_none() => latest = Some(page),
            CommunityPublishObjectKind::Latest => {
                return Err("community publication plan 出现多个 latest.json".to_string());
            }
            CommunityPublishObjectKind::Descriptor | CommunityPublishObjectKind::ActiveIndex => {
                return Err("community page publication plan 类型无效".to_string());
            }
        }
    }
    let latest = latest.ok_or_else(|| "community publication plan 缺少 latest.json".to_string())?;
    descriptors.push(active_index);
    descriptors.extend(immutable_pages);
    descriptors.push(latest);
    Ok(descriptors)
}

fn plan_community_publish_objects(
    feed_prefix: &str,
    descriptors: &BTreeMap<i64, CommunityResourceDescriptor>,
    pages: Vec<(Option<i64>, PublishedCommunityPage)>,
) -> Result<Vec<PlannedCommunityPublishObject>, String> {
    let descriptor_objects = descriptors
        .values()
        .map(|descriptor| {
            let bytes = serialize_community_publish_json(
                descriptor,
                "community resource descriptor",
                COMMUNITY_PUBLISH_MAX_DESCRIPTOR_BYTES,
            )?;
            Ok(PlannedCommunityPublishObject {
                kind: CommunityPublishObjectKind::Descriptor,
                key: community_publish_descriptor_key(
                    feed_prefix,
                    descriptor.resource_id,
                    &descriptor.version,
                ),
                bytes,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let active_index = CommunityResourceActiveIndex {
        v: 1,
        resources: descriptors
            .iter()
            .map(|(resource_id, descriptor)| (*resource_id, descriptor.version.clone()))
            .collect(),
    };
    let active_index_object = PlannedCommunityPublishObject {
        kind: CommunityPublishObjectKind::ActiveIndex,
        key: community_publish_active_index_key(feed_prefix),
        bytes: serialize_community_publish_json(
            &active_index,
            "community resource active index",
            COMMUNITY_PUBLISH_MAX_ACTIVE_INDEX_BYTES,
        )?,
    };
    let page_objects = pages
        .into_iter()
        .map(|(before, page)| {
            let bytes = serialize_community_publish_json(
                &page,
                "community feed page",
                COMMUNITY_PUBLISH_MAX_FEED_BYTES,
            )?;
            Ok(match before {
                Some(before) => PlannedCommunityPublishObject {
                    kind: CommunityPublishObjectKind::Page,
                    key: community_publish_page_key(feed_prefix, before),
                    bytes,
                },
                None => PlannedCommunityPublishObject {
                    kind: CommunityPublishObjectKind::Latest,
                    key: community_publish_latest_key(feed_prefix),
                    bytes,
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    order_community_publish_objects(descriptor_objects, active_index_object, page_objects)
}

fn serialize_community_publish_json<T: Serialize>(
    value: &T,
    label: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(value).map_err(|_| format!("序列化 {label} 失败"))?;
    if bytes.len() > max_bytes {
        return Err(format!("{label} 超过 {max_bytes}B 发布上限"));
    }
    Ok(bytes)
}

fn community_publish_object_max_bytes(kind: CommunityPublishObjectKind) -> usize {
    match kind {
        CommunityPublishObjectKind::Descriptor => COMMUNITY_PUBLISH_MAX_DESCRIPTOR_BYTES,
        CommunityPublishObjectKind::Page | CommunityPublishObjectKind::Latest => {
            COMMUNITY_PUBLISH_MAX_FEED_BYTES
        }
        CommunityPublishObjectKind::ActiveIndex => COMMUNITY_PUBLISH_MAX_ACTIVE_INDEX_BYTES,
    }
}

fn community_publish_object_is_mutable(kind: CommunityPublishObjectKind) -> bool {
    matches!(
        kind,
        CommunityPublishObjectKind::ActiveIndex | CommunityPublishObjectKind::Latest
    )
}

async fn preflight_community_publish_objects(
    oss: &OssObjectStore,
    planned: Vec<PlannedCommunityPublishObject>,
) -> (
    Vec<PreflightCommunityPublishObject>,
    Vec<CommunityPublishConflict>,
) {
    let mut objects = Vec::with_capacity(planned.len());
    let mut conflicts = Vec::new();
    for planned in planned {
        let exists = match community_publish_object_exists_with_retry(oss, &planned.key).await {
            Ok(exists) => exists,
            Err(_) => {
                conflicts.push(CommunityPublishConflict {
                    resource_id: None,
                    object_key: Some(planned.key.clone()),
                    reason: "R2 destination HEAD 预检失败（endpoint 已隐藏）".to_string(),
                });
                objects.push(PreflightCommunityPublishObject {
                    planned,
                    existed: false,
                    should_write: false,
                });
                continue;
            }
        };
        if !exists {
            objects.push(PreflightCommunityPublishObject {
                planned,
                existed: false,
                should_write: true,
            });
            continue;
        }
        let existing = match community_publish_get_object_with_retry(
            oss,
            &planned.key,
            community_publish_object_max_bytes(planned.kind),
        )
        .await
        {
            Ok(existing) => existing,
            Err(_) => {
                conflicts.push(CommunityPublishConflict {
                    resource_id: None,
                    object_key: Some(planned.key.clone()),
                    reason: "R2 destination GET 预检失败（endpoint 已隐藏）".to_string(),
                });
                objects.push(PreflightCommunityPublishObject {
                    planned,
                    existed: true,
                    should_write: false,
                });
                continue;
            }
        };
        if existing.bytes == planned.bytes {
            objects.push(PreflightCommunityPublishObject {
                planned,
                existed: true,
                should_write: false,
            });
        } else if community_publish_object_is_mutable(planned.kind) {
            objects.push(PreflightCommunityPublishObject {
                planned,
                existed: true,
                should_write: true,
            });
        } else {
            conflicts.push(CommunityPublishConflict {
                resource_id: None,
                object_key: Some(planned.key.clone()),
                reason: "immutable publication key 已存在但内容不同；请修正 archive 或提升 feed-prefix 版本"
                    .to_string(),
            });
            objects.push(PreflightCommunityPublishObject {
                planned,
                existed: true,
                should_write: false,
            });
        }
    }
    (objects, conflicts)
}

async fn write_community_publish_object(
    oss: &OssObjectStore,
    object: &PreflightCommunityPublishObject,
) -> Result<bool, String> {
    if !object.should_write {
        return Ok(false);
    }
    if !community_publish_object_is_mutable(object.planned.kind) {
        let exists = community_publish_object_exists_with_retry(oss, &object.planned.key)
            .await
            .map_err(|_| {
                format!(
                    "写入前复检 R2 object 失败（endpoint 已隐藏）: {}",
                    object.planned.key
                )
            })?;
        if exists {
            let existing = community_publish_get_object_with_retry(
                oss,
                &object.planned.key,
                community_publish_object_max_bytes(object.planned.kind),
            )
            .await
            .map_err(|_| {
                format!(
                    "写入前读取 R2 object 失败（endpoint 已隐藏）: {}",
                    object.planned.key
                )
            })?;
            if existing.bytes == object.planned.bytes {
                return Ok(false);
            }
            return Err(format!(
                "immutable publication key 在预检后被并发写入不同内容: {}",
                object.planned.key
            ));
        }
    }
    community_publish_put_object_with_retry(oss, &object.planned.key, &object.planned.bytes)
        .await
        .map_err(|_| {
            format!(
                "写入 R2 publication object 失败（endpoint 已隐藏）: {}",
                object.planned.key
            )
        })?;
    let stored = community_publish_get_object_with_retry(
        oss,
        &object.planned.key,
        community_publish_object_max_bytes(object.planned.kind),
    )
    .await
    .map_err(|_| {
        format!(
            "写入后回读 R2 object 失败（endpoint 已隐藏）: {}",
            object.planned.key
        )
    })?;
    if stored.bytes != object.planned.bytes {
        return Err(format!(
            "写入后 R2 object 内容校验失败: {}",
            object.planned.key
        ));
    }
    Ok(true)
}

async fn run_community_publish(
    config_path: Option<&Path>,
    args: CommunityPublishArgs,
) -> Result<(), String> {
    let source = args.source.trim().to_string();
    let external_id = args.external_id.trim().to_string();
    validate_community_publish_scope(&source, &external_id)?;
    validate_community_publish_page_size(args.page_size)?;
    let feed_prefix = normalize_community_publish_prefix(&args.feed_prefix, "--feed-prefix")?;
    let asset_prefix = normalize_community_publish_prefix(&args.asset_prefix, "--asset-prefix")?;
    validate_community_publish_scoped_prefix(&feed_prefix, &source, &external_id, "--feed-prefix")?;
    validate_community_publish_scoped_prefix(
        &asset_prefix,
        &source,
        &external_id,
        "--asset-prefix",
    )?;
    if feed_prefix == asset_prefix {
        return Err("--feed-prefix 和 --asset-prefix 不能相同".to_string());
    }

    let (config, _) = load_cli_config(config_path, false).map_err(|err| err.to_string())?;
    validate_community_publish_provider(&config.cloud.oss.resolved_provider())?;
    let pg = CloudPgRuntime::from_cloud_config(&config.cloud)
        .ok_or_else(|| "Postgres 未配置，不能发布 community snapshot".to_string())?;
    let oss = OssObjectStore::from_config(&config.cloud.oss)
        .ok_or_else(|| "R2 未完整配置，不能发布 community snapshot".to_string())?;

    let mut publish_lock = if args.apply {
        Some(
            pg.acquire_community_publish_lock(&source, &external_id)
                .await
                .map_err(|error| {
                    if error.to_string().contains("已有 community publisher") {
                        "已有 community publisher 正在运行；本次 apply 未执行任何 R2 写入"
                            .to_string()
                    } else {
                        "获取 Postgres community publish lock 失败（连接细节已隐藏）".to_string()
                    }
                })?,
        )
    } else {
        None
    };
    let operation = async {
        let raw_pages = if let Some(lock) = publish_lock.as_mut() {
            lock.load_snapshot_pages(&source, &external_id, args.page_size)
                .await
                .map_err(|error| {
                    tracing::warn!("community publisher snapshot read failed: {error}");
                    "在 Postgres repeatable-read snapshot 中读取 community timeline 失败"
                        .to_string()
                })?
                .into_iter()
                .map(|page| CommunityPublishRawPage {
                    before: page.before,
                    items: page.items,
                })
                .collect()
        } else {
            load_community_publish_pages(&pg, &source, &external_id, args.page_size).await?
        };
        run_community_publish_inner(
            &oss,
            &source,
            &external_id,
            args.page_size,
            &feed_prefix,
            &asset_prefix,
            args.apply,
            raw_pages,
            publish_lock.as_ref(),
        )
        .await
    }
    .await;

    let release_error = if let Some(publish_lock) = publish_lock {
        publish_lock.release().await.err().map(|error| {
            tracing::warn!("community publisher advisory lock explicit release failed: {error}");
            "Postgres community publish advisory lock 显式释放失败；本次不得报告成功".to_string()
        })
    } else {
        None
    };
    match (operation, release_error) {
        (Ok(report), None) => print_json(&report),
        (Ok(_), Some(release_error)) => Err(release_error),
        (Err(error), None) => Err(error),
        (Err(error), Some(release_error)) => Err(format!("{error}; {release_error}")),
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_community_publish_inner(
    oss: &OssObjectStore,
    source: &str,
    external_id: &str,
    page_size: usize,
    feed_prefix: &str,
    asset_prefix: &str,
    apply: bool,
    raw_pages: Vec<CommunityPublishRawPage>,
    publish_lock: Option<&CloudCommunityPublishLock>,
) -> Result<CommunityPublishReport, String> {
    validate_non_empty_community_publish_archive(&raw_pages)?;
    for item in raw_pages.iter().flat_map(|page| page.items.iter()) {
        if item.content_id <= 0 || item.content_id > MAX_JAVASCRIPT_SAFE_INTEGER {
            return Err(format!(
                "community content_id={} 超出安全正整数范围",
                item.content_id
            ));
        }
    }
    let content_count = raw_pages.iter().map(|page| page.items.len()).sum();
    let page_count = raw_pages.len();
    let mut report = CommunityPublishReport {
        ok: false,
        mode: if apply { "apply" } else { "dry-run" },
        source: source.to_string(),
        external_id: external_id.to_string(),
        page_size,
        feed_prefix: feed_prefix.to_string(),
        asset_prefix: asset_prefix.to_string(),
        latest_key: community_publish_latest_key(feed_prefix),
        content_count,
        page_count,
        resource_count: 0,
        edge_resource_count: 0,
        legacy_resource_count: 0,
        descriptor_count: 0,
        planned_objects: 0,
        existing_objects: 0,
        would_write: 0,
        written: 0,
        latest_updated: false,
        no_op: false,
        legacy_resources: Vec::new(),
        conflicts: Vec::new(),
        resource_verification: if apply {
            "full_bytes_sha256"
        } else {
            "head_exists_only"
        },
        verification_hint: "apply 成功后请用相同参数去掉 --apply 再跑一次；预期 no_op=true 且 would_write=0",
    };

    let mut resource_records = BTreeMap::new();
    for page in &raw_pages {
        for item in &page.items {
            for resource in &item.resources {
                report.resource_count += 1;
                if resource.resource_id <= 0 || resource.resource_id > MAX_JAVASCRIPT_SAFE_INTEGER {
                    report.conflicts.push(CommunityPublishConflict {
                        resource_id: Some(resource.resource_id),
                        object_key: None,
                        reason: "resource_id 超出 JavaScript 安全正整数范围".to_string(),
                    });
                    continue;
                }
                if resource_records
                    .insert(resource.resource_id, resource.clone())
                    .is_some()
                {
                    report.conflicts.push(CommunityPublishConflict {
                        resource_id: Some(resource.resource_id),
                        object_key: None,
                        reason: "timeline 中 resource_id 重复".to_string(),
                    });
                }
            }
        }
    }

    let mut checks =
        stream::iter(
            resource_records.into_values().map(|resource| {
                let oss = oss.clone();
                let asset_prefix = asset_prefix.to_string();
                async move {
                    inspect_community_edge_resource(&oss, &asset_prefix, resource, apply).await
                }
            }),
        )
        .buffer_unordered(COMMUNITY_PUBLISH_RESOURCE_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    checks.sort_by_key(community_edge_resource_check_id);
    let mut descriptors = BTreeMap::new();
    for check in checks {
        match check {
            CommunityEdgeResourceCheck::Eligible(descriptor) => {
                descriptors.insert(descriptor.resource_id, descriptor);
            }
            CommunityEdgeResourceCheck::Legacy {
                resource_id,
                reason,
            } => report
                .legacy_resources
                .push(CommunityPublishLegacyResource {
                    resource_id,
                    reason,
                }),
            CommunityEdgeResourceCheck::Conflict {
                resource_id,
                reason,
            } => report.conflicts.push(CommunityPublishConflict {
                resource_id: Some(resource_id),
                object_key: None,
                reason,
            }),
        }
    }
    report.edge_resource_count = descriptors.len();
    report.legacy_resource_count = report.legacy_resources.len();
    report.descriptor_count = descriptors.len();

    let published_pages = build_published_community_pages(raw_pages, external_id, &descriptors);
    let planned = plan_community_publish_objects(feed_prefix, &descriptors, published_pages)?;
    report.planned_objects = planned.len();
    let (preflight, preflight_conflicts) = preflight_community_publish_objects(oss, planned).await;
    report.conflicts.extend(preflight_conflicts);
    report.existing_objects = preflight.iter().filter(|object| object.existed).count();
    report.would_write = preflight
        .iter()
        .filter(|object| object.should_write)
        .count();

    if !report.conflicts.is_empty() {
        print_json(&report)?;
        return Err("community snapshot 预检失败，未执行任何 R2 写入".to_string());
    }
    if !apply {
        report.ok = true;
        report.no_op = report.would_write == 0;
        return Ok(report);
    }

    let mut checked_lock_before_first_write = false;
    for object in &preflight {
        if object.should_write
            && (!checked_lock_before_first_write
                || community_publish_object_is_mutable(object.planned.kind))
        {
            let lock = publish_lock.ok_or_else(|| {
                "apply 写入前缺少 Postgres community publish advisory lock".to_string()
            })?;
            lock.assert_held().await.map_err(|error| {
                tracing::warn!("community publisher advisory lock liveness check failed: {error}");
                format!(
                    "R2 写入前 Postgres community publish advisory lock 已失效: {}",
                    object.planned.key
                )
            })?;
            checked_lock_before_first_write = true;
        }
        match write_community_publish_object(oss, object).await {
            Ok(false) => {}
            Ok(true) => {
                report.written += 1;
                if object.planned.kind == CommunityPublishObjectKind::Latest {
                    report.latest_updated = true;
                }
            }
            Err(reason) => {
                report.conflicts.push(CommunityPublishConflict {
                    resource_id: None,
                    object_key: Some(object.planned.key.clone()),
                    reason,
                });
                print_json(&report)?;
                return Err(
                    "community snapshot 写入失败；latest.json 未确认更新，可安全幂等重试"
                        .to_string(),
                );
            }
        }
    }
    report.ok = true;
    report.no_op = report.would_write == 0;
    Ok(report)
}

async fn run_community_contents(
    config_path: Option<&Path>,
    args: CommunityContentsArgs,
) -> Result<(), String> {
    let source = args.source.trim();
    let external_id = args.external_id.trim();
    if source.is_empty() || external_id.is_empty() {
        return Err("--source 和 --external-id 不能为空".to_string());
    }
    if sanitize_key_component(source) != source
        || sanitize_key_component(external_id) != external_id
    {
        return Err(
            "--source 和 --external-id 只能包含 ASCII 字母、数字、点、横线和下划线".to_string(),
        );
    }
    let metadata = std::fs::symlink_metadata(&args.manifest)
        .map_err(|err| format!("读取 community content manifest 元数据失败: {err}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("community content manifest 必须是普通文件，不能是符号链接".to_string());
    }
    if metadata.len() == 0 || metadata.len() > 64 * 1024 * 1024 {
        return Err("community content manifest 必须在 1B..=64MiB 范围内".to_string());
    }
    let manifest_bytes = std::fs::read(&args.manifest)
        .map_err(|err| format!("读取 community content manifest 失败: {err}"))?;
    let candidates: Vec<CloudCommunityReconcileCandidate> = serde_json::from_slice(&manifest_bytes)
        .map_err(|err| format!("解析 community content manifest 失败: {err}"))?;

    let (config, _) = load_cli_config(config_path, false).map_err(|err| err.to_string())?;
    let pg = CloudPgRuntime::from_cloud_config(&config.cloud)
        .ok_or_else(|| "Postgres 未配置，不能对账 community contents".to_string())?;
    let report = pg
        .reconcile_community_contents(source, external_id, &candidates, args.apply)
        .await
        .map_err(|err| err.to_string())?;
    print_json(&report)
}

async fn run_community_assets(
    config_path: Option<&Path>,
    args: CommunityAssetsArgs,
) -> Result<(), String> {
    let source = args.source.trim();
    let external_id = args.external_id.trim();
    if source.is_empty() || external_id.is_empty() {
        return Err("--source 和 --external-id 不能为空".to_string());
    }
    if sanitize_key_component(source) != source
        || sanitize_key_component(external_id) != external_id
    {
        return Err(
            "--source 和 --external-id 只能包含 ASCII 字母、数字、点、横线和下划线".to_string(),
        );
    }
    if args.max_bytes == 0 || args.max_bytes > i64::MAX as u64 {
        return Err("--max-bytes 必须在 1..=i64::MAX 范围内".to_string());
    }

    let manifest_bytes = std::fs::read(&args.manifest)
        .map_err(|err| format!("读取 community asset manifest 失败: {err}"))?;
    let entries: Vec<CommunityAssetManifestEntry> = serde_json::from_slice(&manifest_bytes)
        .map_err(|err| format!("解析 community asset manifest 失败: {err}"))?;
    if entries.is_empty() {
        return Err("community asset manifest 不能为空".to_string());
    }
    if entries.len() > 10_000 {
        return Err("community asset manifest 条目超过 10000 上限".to_string());
    }

    let (config, _) = load_cli_config(config_path, false).map_err(|err| err.to_string())?;
    let pg = CloudPgRuntime::from_cloud_config(&config.cloud)
        .ok_or_else(|| "Postgres 未配置，不能校验 community assets".to_string())?;
    let oss = OssObjectStore::from_config(&config.cloud.oss)
        .ok_or_else(|| "OSS/R2 未配置，不能校验 community assets".to_string())?;

    let manifest_parent = args
        .manifest
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut report = CommunityAssetsReport {
        ok: false,
        mode: if args.apply { "apply" } else { "dry-run" },
        manifest: args.manifest.to_string_lossy().to_string(),
        source: source.to_string(),
        external_id: external_id.to_string(),
        total: entries.len(),
        validated: 0,
        uploaded: 0,
        reused: 0,
        updated: 0,
        skipped: 0,
        would_upload: 0,
        would_update: 0,
        conflicts: Vec::new(),
        items: Vec::new(),
    };
    let mut seen_resource_ids = BTreeSet::new();
    for entry in &entries {
        if !seen_resource_ids.insert(entry.resource_id) {
            push_community_asset_conflict(
                &mut report,
                Some(entry.resource_id),
                "manifest 中 resource_id 重复；为避免部分写入，整批拒绝",
            );
        }
    }
    if !report.conflicts.is_empty() {
        print_json(&report)?;
        return Err("community asset manifest 存在重复 resource_id".to_string());
    }

    let mut validated_entries = Vec::with_capacity(entries.len());
    for entry in entries {
        let validated = match validate_community_asset_entry(entry, manifest_parent, args.max_bytes)
        {
            Ok(validated) => validated,
            Err((resource_id, reason)) => {
                push_community_asset_conflict(&mut report, Some(resource_id), reason);
                continue;
            }
        };
        validated_entries.push(validated);
    }
    if args.apply && !report.conflicts.is_empty() {
        print_json(&report)?;
        return Err("community asset manifest 本地预检失败，未执行任何写入".to_string());
    }

    let mut targeted_entries = Vec::with_capacity(validated_entries.len());
    for validated in validated_entries {
        let target = match pg
            .get_community_resource_backfill_target(source, external_id, validated.resource_id)
            .await
        {
            Ok(Some(target)) => target,
            Ok(None) => {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    "目标社区或 resource_id 不存在",
                );
                continue;
            }
            Err(error) => {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    format!("读取目标资源失败: {error}"),
                );
                continue;
            }
        };
        if let Err(reason) = validate_asset_against_target(&validated, &target) {
            push_community_asset_conflict(&mut report, Some(validated.resource_id), reason);
            continue;
        }
        report.validated += 1;
        targeted_entries.push((validated, target));
    }
    if args.apply && !report.conflicts.is_empty() {
        print_json(&report)?;
        return Err("community asset manifest 数据库预检失败，未执行任何写入".to_string());
    }

    let mut object_entries = Vec::with_capacity(targeted_entries.len());
    for (validated, target) in targeted_entries {
        let key = immutable_community_asset_key(
            source,
            external_id,
            validated.resource_id,
            &validated.sha256,
            validated.extension,
        );
        let object_uri = oss.object_uri(&key);
        let object_exists = match oss.object_exists(&key).await {
            Ok(exists) => exists,
            Err(error) => {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    format!("检查 immutable OSS 对象失败: {error}"),
                );
                continue;
            }
        };
        if object_exists {
            if let Err(reason) = verify_community_asset_object(&oss, &key, &validated).await {
                push_community_asset_conflict(&mut report, Some(validated.resource_id), reason);
                continue;
            }
            report.reused += 1;
        } else if !args.apply {
            report.would_upload += 1;
        }
        object_entries.push((validated, target, key, object_uri, object_exists));
    }
    if args.apply && !report.conflicts.is_empty() {
        print_json(&report)?;
        return Err("community asset manifest OSS 预检失败，未执行任何写入".to_string());
    }

    let mut verified_entries = Vec::with_capacity(object_entries.len());
    for (validated, target, key, object_uri, object_exists) in object_entries {
        if args.apply && !object_exists {
            let bytes = match read_verified_community_asset_bytes(&validated) {
                Ok(bytes) => bytes,
                Err(reason) => {
                    push_community_asset_conflict(
                        &mut report,
                        Some(validated.resource_id),
                        format!("上传前本地文件复检失败: {reason}"),
                    );
                    continue;
                }
            };
            if let Err(error) = oss.put_object(&key, bytes, &validated.content_type).await {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    format!("上传 immutable OSS 对象失败: {error}"),
                );
                continue;
            }
            if let Err(reason) = verify_community_asset_object(&oss, &key, &validated).await {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    format!("上传后校验失败: {reason}"),
                );
                continue;
            }
            report.uploaded += 1;
        }
        verified_entries.push((validated, target, key, object_uri, object_exists));
    }
    if args.apply && !report.conflicts.is_empty() {
        print_json(&report)?;
        return Err(
            "community asset 上传或回读校验失败；数据库保持不变，可修复后幂等重试".to_string(),
        );
    }

    for (validated, target, key, object_uri, object_exists) in verified_entries {
        if !args.apply {
            if target_matches_asset(&target, &validated, &object_uri) {
                report.skipped += 1;
                report.items.push(CommunityAssetReportItem {
                    resource_id: validated.resource_id,
                    action: "already_current",
                    byte_size: validated.byte_size,
                    sha256: validated.sha256,
                    oss_key: Some(key),
                });
            } else {
                report.would_update += 1;
                report.items.push(CommunityAssetReportItem {
                    resource_id: validated.resource_id,
                    action: if object_exists {
                        "would_reuse_and_update"
                    } else {
                        "would_upload_and_update"
                    },
                    byte_size: validated.byte_size,
                    sha256: validated.sha256,
                    oss_key: Some(key),
                });
            }
            continue;
        }

        let audit_metadata =
            community_asset_audit_metadata(source, external_id, &validated, &target);
        let update = CloudCommunityResourceBackfillUpdate {
            resource_id: validated.resource_id,
            expected_updated_at: target.updated_at.clone(),
            source_resource_id: validated
                .source_resource_id
                .clone()
                .or_else(|| validated.source_base_key.clone()),
            content_type: validated.content_type.clone(),
            byte_size: validated.byte_size as i64,
            sha256: validated.sha256.clone(),
            oss_uri: object_uri,
            audit_metadata,
        };
        match pg
            .backfill_community_resource(source, external_id, &update)
            .await
        {
            Ok(CloudCommunityResourceBackfillOutcome::Updated) => {
                report.updated += 1;
                report.items.push(CommunityAssetReportItem {
                    resource_id: validated.resource_id,
                    action: if object_exists {
                        "reused_and_updated"
                    } else {
                        "uploaded_and_updated"
                    },
                    byte_size: validated.byte_size,
                    sha256: validated.sha256,
                    oss_key: Some(key),
                });
            }
            Ok(CloudCommunityResourceBackfillOutcome::Unchanged) => {
                report.skipped += 1;
                report.items.push(CommunityAssetReportItem {
                    resource_id: validated.resource_id,
                    action: "already_current",
                    byte_size: validated.byte_size,
                    sha256: validated.sha256,
                    oss_key: Some(key),
                });
            }
            Ok(CloudCommunityResourceBackfillOutcome::Conflict) => {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    "数据库记录在校验后发生变化，未覆盖并发更新",
                );
            }
            Ok(CloudCommunityResourceBackfillOutcome::NotFound) => {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    "数据库记录在校验后被删除",
                );
            }
            Err(error) => {
                push_community_asset_conflict(
                    &mut report,
                    Some(validated.resource_id),
                    format!("更新数据库失败: {error}"),
                );
            }
        }
    }

    report.ok = report.conflicts.is_empty();
    print_json(&report)?;
    if report.ok {
        Ok(())
    } else {
        Err(format!(
            "community asset backfill 有 {} 个冲突",
            report.conflicts.len()
        ))
    }
}

fn push_community_asset_conflict(
    report: &mut CommunityAssetsReport,
    resource_id: Option<i64>,
    reason: impl Into<String>,
) {
    report.conflicts.push(CommunityAssetConflict {
        resource_id,
        reason: reason.into(),
    });
}

fn validate_community_asset_entry(
    entry: CommunityAssetManifestEntry,
    manifest_parent: &Path,
    max_bytes: u64,
) -> Result<ValidatedCommunityAsset, (i64, String)> {
    let resource_id = entry.resource_id;
    if resource_id <= 0 {
        return Err((resource_id, "resource_id 必须大于 0".to_string()));
    }
    if entry.byte_size == 0 || entry.byte_size > max_bytes {
        return Err((
            resource_id,
            format!("byte_size 必须在 1..={max_bytes} 范围内"),
        ));
    }
    let sha256 = normalized_manifest_sha256(&entry.sha256)
        .ok_or_else(|| (resource_id, "sha256 必须是 64 位十六进制".to_string()))?;
    validate_safe_source_identifier(entry.source_base_key.as_deref())
        .map_err(|reason| (resource_id, format!("source_base_key 不安全: {reason}")))?;
    validate_safe_source_identifier(entry.source_resource_id.as_deref())
        .map_err(|reason| (resource_id, format!("source_resource_id 不安全: {reason}")))?;
    match (entry.width, entry.height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {}
        (None, None) => {}
        _ => {
            return Err((resource_id, "width/height 必须同时存在且大于 0".to_string()));
        }
    }
    if let Some(captured_at) = entry.captured_at.as_deref() {
        chrono::DateTime::parse_from_rfc3339(captured_at)
            .map_err(|_| (resource_id, "captured_at 必须是 RFC3339 时间".to_string()))?;
    }

    let path = if entry.path.is_absolute() {
        entry.path.clone()
    } else {
        manifest_parent.join(&entry.path)
    };
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|err| (resource_id, format!("读取本地文件元数据失败: {err}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err((
            resource_id,
            "本地 path 必须是普通文件，不能是符号链接".to_string(),
        ));
    }
    if metadata.len() != entry.byte_size {
        return Err((
            resource_id,
            format!(
                "manifest/file byte_size 不一致: {} != {}",
                entry.byte_size,
                metadata.len()
            ),
        ));
    }
    let bytes =
        std::fs::read(&path).map_err(|err| (resource_id, format!("读取本地文件失败: {err}")))?;
    if bytes.len() as u64 != entry.byte_size {
        return Err((
            resource_id,
            format!(
                "读取期间文件大小发生变化: expected={} actual={}",
                entry.byte_size,
                bytes.len()
            ),
        ));
    }
    let actual_sha256 = sha256_hex(&bytes);
    if actual_sha256 != sha256 {
        return Err((
            resource_id,
            format!("manifest/file sha256 不一致: expected={sha256} actual={actual_sha256}"),
        ));
    }
    let (content_type, extension) = validate_content_type_and_magic(&entry.content_type, &bytes)
        .map_err(|reason| (resource_id, reason))?;

    Ok(ValidatedCommunityAsset {
        resource_id,
        path,
        content_type: content_type.to_string(),
        byte_size: entry.byte_size,
        sha256,
        extension,
        source_base_key: entry.source_base_key,
        source_resource_id: entry.source_resource_id,
        width: entry.width,
        height: entry.height,
        captured_at: entry.captured_at,
    })
}

fn read_verified_community_asset_bytes(asset: &ValidatedCommunityAsset) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(&asset.path)
        .map_err(|error| format!("读取本地文件元数据失败: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("本地 path 不再是普通文件".to_string());
    }
    if metadata.len() != asset.byte_size {
        return Err(format!(
            "本地文件大小已变化: expected={} actual={}",
            asset.byte_size,
            metadata.len()
        ));
    }
    let bytes = std::fs::read(&asset.path).map_err(|error| format!("读取本地文件失败: {error}"))?;
    if bytes.len() as u64 != asset.byte_size {
        return Err(format!(
            "读取期间文件大小发生变化: expected={} actual={}",
            asset.byte_size,
            bytes.len()
        ));
    }
    let actual_sha256 = sha256_hex(&bytes);
    if actual_sha256 != asset.sha256 {
        return Err(format!(
            "本地文件 sha256 已变化: expected={} actual={actual_sha256}",
            asset.sha256
        ));
    }
    let (content_type, extension) = validate_content_type_and_magic(&asset.content_type, &bytes)?;
    if content_type != asset.content_type || extension != asset.extension {
        return Err("本地文件 MIME/magic 在预检后发生变化".to_string());
    }
    Ok(bytes)
}

fn normalized_manifest_sha256(raw: &str) -> Option<String> {
    let value = raw.trim();
    (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| value.to_ascii_lowercase())
}

fn validate_safe_source_identifier(raw: Option<&str>) -> Result<(), &'static str> {
    let Some(value) = raw else {
        return Ok(());
    };
    let value = value.trim();
    if value.is_empty() || value.len() > 512 {
        return Err("不能为空且不能超过 512 字节");
    }
    if value.contains("://") || value.chars().any(char::is_control) {
        return Err("不能包含 URL 或控制字符");
    }
    Ok(())
}

fn validate_content_type_and_magic(
    raw_content_type: &str,
    bytes: &[u8],
) -> Result<(&'static str, &'static str), String> {
    let content_type = raw_content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let zip_has = |needle: &[u8]| bytes.windows(needle.len()).any(|window| window == needle);
    let ole = bytes.starts_with(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
    let valid = match content_type.as_str() {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" | "image/jpg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "image/webp" => bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP",
        "image/avif" => {
            bytes.len() >= 12
                && &bytes[4..8] == b"ftyp"
                && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
        }
        "application/pdf" => bytes[..bytes.len().min(1024)]
            .windows(5)
            .any(|window| window == b"%PDF-"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            bytes.starts_with(b"PK") && zip_has(b"word/") && zip_has(b"[Content_Types].xml")
        }
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            bytes.starts_with(b"PK") && zip_has(b"xl/") && zip_has(b"[Content_Types].xml")
        }
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            bytes.starts_with(b"PK") && zip_has(b"ppt/") && zip_has(b"[Content_Types].xml")
        }
        "application/msword" | "application/vnd.ms-excel" | "application/vnd.ms-powerpoint" => ole,
        _ => {
            return Err(format!(
                "content_type 不在 community asset 安全 allowlist: {content_type}"
            ));
        }
    };
    if !valid {
        return Err(format!("文件 magic 与 content_type 不一致: {content_type}"));
    }
    Ok(match content_type.as_str() {
        "image/png" => ("image/png", "png"),
        "image/jpeg" | "image/jpg" => ("image/jpeg", "jpg"),
        "image/gif" => ("image/gif", "gif"),
        "image/webp" => ("image/webp", "webp"),
        "image/avif" => ("image/avif", "avif"),
        "application/pdf" => ("application/pdf", "pdf"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "docx",
        ),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xlsx",
        ),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => (
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "pptx",
        ),
        "application/msword" => ("application/msword", "doc"),
        "application/vnd.ms-excel" => ("application/vnd.ms-excel", "xls"),
        "application/vnd.ms-powerpoint" => ("application/vnd.ms-powerpoint", "ppt"),
        _ => unreachable!("content type was allowlisted above"),
    })
}

fn validate_asset_against_target(
    asset: &ValidatedCommunityAsset,
    target: &CloudCommunityResourceBackfillTarget,
) -> Result<(), String> {
    if target.resource_id != asset.resource_id {
        return Err("数据库返回的 resource_id 与 manifest 不一致".to_string());
    }
    if let (Some(current), Some(desired)) = (
        target.source_resource_id.as_deref(),
        asset
            .source_resource_id
            .as_deref()
            .or(asset.source_base_key.as_deref()),
    ) && current != desired
    {
        return Err("source_resource_id 与数据库已有值冲突".to_string());
    }
    if let Some(display_name) = target.display_name.as_deref()
        && let Some(extension) = Path::new(display_name)
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
    {
        let expected_content_type = content_type_for_extension(&extension).ok_or_else(|| {
            format!("display_name 扩展名 .{extension} 不在 community asset 安全 allowlist")
        })?;
        let source_extension_alias = extension == "xls"
            && asset.content_type
                == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
        if expected_content_type != asset.content_type && !source_extension_alias {
            return Err(format!(
                "display_name 扩展名 .{extension} 与 content_type {} 不一致",
                asset.content_type
            ));
        }
    }
    Ok(())
}

fn content_type_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "avif" => Some("image/avif"),
        "pdf" => Some("application/pdf"),
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        "pptx" => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation"),
        "doc" => Some("application/msword"),
        "xls" => Some("application/vnd.ms-excel"),
        "ppt" => Some("application/vnd.ms-powerpoint"),
        _ => None,
    }
}

fn immutable_community_asset_key(
    source: &str,
    external_id: &str,
    resource_id: i64,
    sha256: &str,
    extension: &str,
) -> String {
    format!(
        "community/{}/{}/resources/{}-{}.{}",
        sanitize_key_component(source),
        sanitize_key_component(external_id),
        resource_id,
        sha256,
        extension
    )
}

async fn verify_community_asset_object(
    oss: &OssObjectStore,
    key: &str,
    asset: &ValidatedCommunityAsset,
) -> Result<(), String> {
    let max_bytes = usize::try_from(asset.byte_size)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| "asset byte_size 无法转换为本机 usize".to_string())?;
    let object = oss
        .get_object_limited(key, max_bytes)
        .await
        .map_err(|error| format!("读取 immutable OSS 对象失败: {error}"))?;
    if object.bytes.len() as u64 != asset.byte_size {
        return Err(format!(
            "immutable OSS 对象大小冲突: expected={} actual={}",
            asset.byte_size,
            object.bytes.len()
        ));
    }
    let actual_sha256 = sha256_hex(&object.bytes);
    if actual_sha256 != asset.sha256 {
        return Err(format!(
            "immutable OSS 对象 sha256 冲突: expected={} actual={actual_sha256}",
            asset.sha256
        ));
    }
    let object_content_type = object
        .content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if object_content_type != asset.content_type {
        return Err(format!(
            "immutable OSS 对象 content_type 冲突: expected={} actual={object_content_type}",
            asset.content_type
        ));
    }
    Ok(())
}

fn target_matches_asset(
    target: &CloudCommunityResourceBackfillTarget,
    asset: &ValidatedCommunityAsset,
    object_uri: &str,
) -> bool {
    let desired_source_resource_id = asset
        .source_resource_id
        .as_deref()
        .or(asset.source_base_key.as_deref());
    target.content_type.as_deref() == Some(asset.content_type.as_str())
        && target.byte_size == Some(asset.byte_size as i64)
        && target.sha256.as_deref() == Some(asset.sha256.as_str())
        && target.oss_uri.as_deref() == Some(object_uri)
        && target.access_state == "stored"
        && desired_source_resource_id
            .map(|desired| target.source_resource_id.as_deref() == Some(desired))
            .unwrap_or(true)
}

fn community_asset_audit_metadata(
    source: &str,
    external_id: &str,
    asset: &ValidatedCommunityAsset,
    target: &CloudCommunityResourceBackfillTarget,
) -> serde_json::Value {
    json!({
        "tool": "hone-cli cloud community-assets",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "source": source,
        "external_id": external_id,
        "source_base_key": asset.source_base_key,
        "source_resource_id": asset.source_resource_id,
        "width": asset.width,
        "height": asset.height,
        "captured_at": asset.captured_at,
        "verified_at": chrono::Utc::now().to_rfc3339(),
        "previous_sha256": target.sha256,
        "previous_byte_size": target.byte_size,
        "previous_oss_uri": target.oss_uri,
        "previous_access_state": target.access_state,
    })
}

async fn run_object_bench(config_path: Option<&Path>, args: ObjectBenchArgs) -> Result<(), String> {
    let (config, _) = load_cli_config(config_path, false).map_err(|err| err.to_string())?;
    let size = args.size_kib.max(1) * 1024;
    let iterations = args.iterations.max(1);
    let payload = deterministic_payload(size);
    let mut results = Vec::new();
    let aliyun_config = aliyun_config_for_bench(&config.cloud.oss);
    results.push(
        bench_provider(
            "aliyun_oss",
            &aliyun_config,
            &payload,
            iterations,
            args.cleanup,
        )
        .await,
    );
    let r2_config = r2_config_from_env(&config.cloud.oss);
    results.push(
        bench_provider(
            "cloudflare_r2",
            &r2_config,
            &payload,
            iterations,
            args.cleanup,
        )
        .await,
    );
    let report = ObjectBenchReport {
        size_kib: args.size_kib.max(1),
        iterations,
        results,
    };
    if args.json {
        print_json(&report)
    } else {
        for result in &report.results {
            println!(
                "{} configured={} ok={} avg_put_ms={:?} avg_head_ms={:?} avg_get_ms={:?}",
                result.provider,
                result.configured,
                result.ok,
                result.avg_put_ms,
                result.avg_head_ms,
                result.avg_get_ms
            );
            for error in &result.errors {
                println!("{} error: {error}", result.provider);
            }
        }
        Ok(())
    }
}

async fn bench_provider(
    label: &str,
    config: &OssConfig,
    payload: &[u8],
    iterations: usize,
    cleanup: bool,
) -> ObjectBenchProviderReport {
    let configured = config.is_configured();
    let proxy_configured = !config.resolved_proxy().trim().is_empty();
    let mut report = ObjectBenchProviderReport {
        provider: label.to_string(),
        configured,
        ok: false,
        bucket: configured.then(|| config.resolved_bucket()),
        endpoint: configured.then(|| config.resolved_endpoint()),
        proxy_configured,
        iterations: Vec::new(),
        avg_put_ms: None,
        avg_head_ms: None,
        avg_get_ms: None,
        errors: Vec::new(),
    };
    let Some(store) = OssObjectStore::from_config(config) else {
        report.errors.push("not configured".to_string());
        return report;
    };
    let run_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or_default();
    for index in 0..iterations {
        let key = format!("bench/codex/{run_id}-{label}-{index}.bin");
        let put_start = Instant::now();
        if let Err(error) = store
            .put_object(&key, payload.to_vec(), "application/octet-stream")
            .await
        {
            report.errors.push(format!("put {index}: {error}"));
            continue;
        }
        let put_ms = put_start.elapsed().as_millis();

        let head_start = Instant::now();
        match store.object_exists(&key).await {
            Ok(true) => {}
            Ok(false) => {
                report
                    .errors
                    .push(format!("head {index}: uploaded object not found"));
                continue;
            }
            Err(error) => {
                report.errors.push(format!("head {index}: {error}"));
                continue;
            }
        }
        let head_ms = head_start.elapsed().as_millis();

        let get_start = Instant::now();
        match store.get_object(&key).await {
            Ok(object) if object.bytes.len() == payload.len() => {}
            Ok(object) => {
                report.errors.push(format!(
                    "get {index}: size mismatch expected={} actual={}",
                    payload.len(),
                    object.bytes.len()
                ));
                continue;
            }
            Err(error) => {
                report.errors.push(format!("get {index}: {error}"));
                continue;
            }
        }
        let get_ms = get_start.elapsed().as_millis();
        if cleanup {
            let _ = store.delete_object(&key).await;
        }
        report.iterations.push(ObjectBenchIteration {
            put_ms,
            head_ms,
            get_ms,
            bytes: payload.len(),
        });
    }
    if !report.iterations.is_empty() {
        report.ok = report.errors.is_empty();
        report.avg_put_ms = Some(avg_ms(report.iterations.iter().map(|item| item.put_ms)));
        report.avg_head_ms = Some(avg_ms(report.iterations.iter().map(|item| item.head_ms)));
        report.avg_get_ms = Some(avg_ms(report.iterations.iter().map(|item| item.get_ms)));
    }
    report
}

fn avg_ms(values: impl Iterator<Item = u128>) -> u128 {
    let mut count = 0u128;
    let mut sum = 0u128;
    for value in values {
        count += 1;
        sum += value;
    }
    if count == 0 { 0 } else { sum / count }
}

fn deterministic_payload(size: usize) -> Vec<u8> {
    (0..size)
        .map(|idx| ((idx.wrapping_mul(31).wrapping_add(17)) % 251) as u8)
        .collect()
}

fn r2_config_from_env(fallback: &OssConfig) -> OssConfig {
    OssConfig {
        provider: "r2".to_string(),
        provider_env: "HONE_R2_PROVIDER".to_string(),
        access_key_id: String::new(),
        access_key_id_env: "HONE_R2_ACCESS_KEY_ID".to_string(),
        access_key_secret: String::new(),
        access_key_secret_env: "HONE_R2_ACCESS_KEY_SECRET".to_string(),
        bucket: fallback.resolved_bucket(),
        bucket_env: "HONE_R2_BUCKET".to_string(),
        endpoint: String::new(),
        endpoint_env: "HONE_R2_ENDPOINT".to_string(),
        region: "auto".to_string(),
        region_env: "HONE_R2_REGION".to_string(),
        public_upload_prefix: fallback.public_upload_prefix.clone(),
        proxy: String::new(),
        proxy_env: "HONE_R2_PROXY".to_string(),
    }
}

fn aliyun_config_for_bench(fallback: &OssConfig) -> OssConfig {
    if std::env::var("HONE_ALIYUN_OSS_ACCESS_KEY_ID")
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return fallback.clone();
    }
    OssConfig {
        provider: "aliyun_oss".to_string(),
        provider_env: "HONE_ALIYUN_OSS_PROVIDER".to_string(),
        access_key_id: String::new(),
        access_key_id_env: "HONE_ALIYUN_OSS_ACCESS_KEY_ID".to_string(),
        access_key_secret: String::new(),
        access_key_secret_env: "HONE_ALIYUN_OSS_ACCESS_KEY_SECRET".to_string(),
        bucket: fallback.resolved_bucket(),
        bucket_env: "HONE_ALIYUN_OSS_BUCKET".to_string(),
        endpoint: String::new(),
        endpoint_env: "HONE_ALIYUN_OSS_ENDPOINT".to_string(),
        region: String::new(),
        region_env: "HONE_ALIYUN_OSS_REGION".to_string(),
        public_upload_prefix: fallback.public_upload_prefix.clone(),
        proxy: String::new(),
        proxy_env: "HONE_ALIYUN_OSS_PROXY".to_string(),
    }
}

async fn run_doctor(config_path: Option<&Path>, args: CloudDoctorArgs) -> Result<(), String> {
    let (config, _) = load_cli_config(config_path, false).map_err(|err| err.to_string())?;
    let pg = CloudPgRuntime::from_cloud_config(&config.cloud);
    let oss = OssObjectStore::from_config(&config.cloud.oss);
    let mut schema_ensured = false;
    let postgres_health = if let Some(pg_runtime) = &pg {
        let health = pg_runtime.health().await;
        if args.ensure_schema && health.ok {
            tokio::time::timeout(
                std::time::Duration::from_secs(15),
                pg_runtime.ensure_schema(),
            )
            .await
            .map_err(|_| "Postgres schema 初始化超时".to_string())?
            .map_err(|err| err.to_string())?;
            schema_ensured = true;
        }
        Some(health)
    } else {
        None
    };
    let oss_health = if let Some(oss_store) = &oss {
        Some(oss_store.health().await)
    } else {
        None
    };
    let local_deps = local_durable_dependencies(&config);
    let report = CloudDoctorReport {
        cloud_mode: config.cloud.effective_mode().as_str().to_string(),
        cloud_enabled: config.cloud.effective_enabled(),
        strict_no_local_storage: config.cloud.effective_strict_no_local_storage(),
        runtime_role: RuntimeRole::from_env().as_str().to_string(),
        postgres_configured: config.cloud.postgres.is_configured(),
        postgres_proxy_configured: !config.cloud.postgres.resolved_proxy().is_empty(),
        postgres_health,
        schema_ensured,
        oss_configured: config.cloud.oss.is_configured(),
        oss_proxy_configured: !config.cloud.oss.resolved_proxy().is_empty(),
        oss_health,
        local_durable_dependency_count: local_deps.len(),
        local_durable_dependencies: local_deps,
    };
    if args.json {
        print_json(&report)
    } else {
        println!("cloud_mode={}", report.cloud_mode);
        println!("runtime_role={}", report.runtime_role);
        println!("postgres_configured={}", report.postgres_configured);
        println!("oss_configured={}", report.oss_configured);
        println!(
            "local_durable_dependency_count={}",
            report.local_durable_dependency_count
        );
        if let Some(health) = report.postgres_health {
            println!("postgres_ok={} detail={}", health.ok, health.detail);
        }
        if let Some(health) = report.oss_health {
            println!("oss_ok={} detail={}", health.ok, health.detail);
        }
        Ok(())
    }
}

async fn run_migrate(config_path: Option<&Path>, args: CloudMigrateArgs) -> Result<(), String> {
    let narrow_modes = [
        args.quota_only,
        args.session_only,
        args.web_auth_only,
        args.cron_only,
        args.skill_registry_only,
        args.notification_prefs_only,
        args.portfolio_only,
        args.llm_audit_only,
        args.company_profiles_only,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if narrow_modes > 1 {
        return Err(
            "--quota-only / --session-only / --web-auth-only / --cron-only / --skill-registry-only / --notification-prefs-only / --portfolio-only / --llm-audit-only / --company-profiles-only 不能同时使用"
                .to_string(),
        );
    }
    let (config, _) = load_cli_config(config_path, false).map_err(|err| err.to_string())?;
    let mut report = MigrationReport {
        mode: if args.apply { "apply" } else { "dry-run" },
        from_data_dir: args.from_data_dir.to_string_lossy().to_string(),
        upload_oss: args.upload_oss,
        reuse_existing: args.reuse_existing,
        concurrency: args.concurrency.max(1),
        postgres_configured: config.cloud.postgres.is_configured(),
        oss_configured: config.cloud.oss.is_configured(),
        counted: MigrationCounts::default(),
        uploaded_objects: 0,
        reused_objects: 0,
        indexed_documents: 0,
        changed_quota_rows: 0,
        skipped_quota_rows: 0,
        changed_session_rows: 0,
        skipped_session_rows: 0,
        changed_web_auth_users: 0,
        skipped_web_auth_users: 0,
        changed_web_auth_sessions: 0,
        skipped_web_auth_sessions: 0,
        changed_cron_rows: 0,
        skipped_cron_rows: 0,
        changed_skill_registry_rows: 0,
        skipped_skill_registry_rows: 0,
        changed_notification_prefs_rows: 0,
        skipped_notification_prefs_rows: 0,
        changed_portfolio_rows: 0,
        skipped_portfolio_rows: 0,
        changed_company_profile_files: 0,
        skipped_company_profile_files: 0,
        changed_llm_audit_rows: 0,
        skipped_llm_audit_rows: 0,
        skipped_objects: 0,
        conflicts: Vec::new(),
    };

    let candidates = collect_candidates(&args.from_data_dir, &mut report.counted)
        .map_err(|err| err.to_string())?;
    if args.apply {
        let pg = CloudPgRuntime::from_cloud_config(&config.cloud)
            .ok_or_else(|| "Postgres 未配置，不能 apply migration".to_string())?;
        pg.ensure_schema().await.map_err(|err| err.to_string())?;
        if !args.quota_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            let session_imports = collect_session_imports(&candidates);
            let session_report = pg
                .import_session_records(&session_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_session_rows = session_report.changed_rows;
            report.skipped_session_rows = session_report.skipped_rows;
        }
        if !args.session_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            let quota_imports = collect_quota_imports(&candidates);
            let quota_report = pg
                .import_conversation_quota(&quota_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_quota_rows = quota_report.changed_rows;
            report.skipped_quota_rows = quota_report.skipped_rows;
        }
        if !args.quota_only
            && !args.session_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            let web_auth_storage =
                hone_memory::WebAuthStorage::new(&config.storage.session_sqlite_db_path)
                    .map_err(|err| err.to_string())?;
            let (users, sessions) = web_auth_storage
                .export_cloud_records()
                .map_err(|err| err.to_string())?;
            let auth_report = pg
                .import_web_auth_records(&users, &sessions)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_web_auth_users = auth_report.changed_users;
            report.skipped_web_auth_users = auth_report.skipped_users;
            report.changed_web_auth_sessions = auth_report.changed_sessions;
            report.skipped_web_auth_sessions = auth_report.skipped_sessions;
        }
        if !args.quota_only
            && !args.session_only
            && !args.web_auth_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            let cron_imports = collect_cron_imports(&candidates);
            let cron_report = pg
                .import_cron_job_records(&cron_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_cron_rows = cron_report.changed_rows;
            report.skipped_cron_rows = cron_report.skipped_rows;
        }
        if !args.quota_only
            && !args.session_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            import_skill_registry(&pg, &args.from_data_dir, &mut report)
                .await
                .map_err(|err| err.to_string())?;
        }
        if !args.quota_only
            && !args.session_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.portfolio_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            let prefs_imports = collect_notification_prefs_imports(&candidates);
            let prefs_report = pg
                .import_notification_prefs(&prefs_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_notification_prefs_rows = prefs_report.changed_rows;
            report.skipped_notification_prefs_rows = prefs_report.skipped_rows;
        }
        if !args.quota_only
            && !args.session_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.llm_audit_only
            && !args.company_profiles_only
        {
            let portfolio_imports = collect_portfolio_imports(&candidates);
            let portfolio_report = pg
                .import_portfolios(&portfolio_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_portfolio_rows = portfolio_report.changed_rows;
            report.skipped_portfolio_rows = portfolio_report.skipped_rows;
        }
        if !args.quota_only
            && !args.session_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.llm_audit_only
        {
            let company_profile_imports = collect_company_profile_imports(&candidates);
            let company_profile_report = pg
                .import_company_profile_files(&company_profile_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_company_profile_files = company_profile_report.changed_rows;
            report.skipped_company_profile_files = company_profile_report.skipped_rows;
        }
        if !args.quota_only
            && !args.session_only
            && !args.web_auth_only
            && !args.cron_only
            && !args.skill_registry_only
            && !args.notification_prefs_only
            && !args.portfolio_only
            && !args.company_profiles_only
        {
            import_llm_audit(&pg, &args.from_data_dir, &mut report)
                .await
                .map_err(|err| err.to_string())?;
        }
        if args.quota_only
            || args.session_only
            || args.web_auth_only
            || args.cron_only
            || args.skill_registry_only
            || args.notification_prefs_only
            || args.portfolio_only
            || args.llm_audit_only
            || args.company_profiles_only
        {
            return if args.json {
                print_json(&report)
            } else {
                println!(
                    "mode={} sessions={} changed_session_rows={} skipped_session_rows={} quota_json={} changed_quota_rows={} skipped_quota_rows={} changed_web_auth_users={} skipped_web_auth_users={} changed_web_auth_sessions={} skipped_web_auth_sessions={} cron_json={} changed_cron_rows={} skipped_cron_rows={} skill_registry_json={} changed_skill_registry_rows={} skipped_skill_registry_rows={} notification_prefs={} changed_notification_prefs_rows={} skipped_notification_prefs_rows={} portfolio_json={} changed_portfolio_rows={} skipped_portfolio_rows={} company_profiles={} changed_company_profile_files={} skipped_company_profile_files={} changed_llm_audit_rows={} skipped_llm_audit_rows={}",
                    report.mode,
                    report.counted.sessions,
                    report.changed_session_rows,
                    report.skipped_session_rows,
                    report.counted.quota_json,
                    report.changed_quota_rows,
                    report.skipped_quota_rows,
                    report.changed_web_auth_users,
                    report.skipped_web_auth_users,
                    report.changed_web_auth_sessions,
                    report.skipped_web_auth_sessions,
                    report.counted.cron_json,
                    report.changed_cron_rows,
                    report.skipped_cron_rows,
                    report.counted.skill_registry_json,
                    report.changed_skill_registry_rows,
                    report.skipped_skill_registry_rows,
                    report.counted.notification_prefs,
                    report.changed_notification_prefs_rows,
                    report.skipped_notification_prefs_rows,
                    report.counted.portfolio_json,
                    report.changed_portfolio_rows,
                    report.skipped_portfolio_rows,
                    report.counted.company_profiles,
                    report.changed_company_profile_files,
                    report.skipped_company_profile_files,
                    report.changed_llm_audit_rows,
                    report.skipped_llm_audit_rows
                );
                Ok(())
            };
        }
        let oss = if args.upload_oss {
            Some(
                OssObjectStore::from_config(&config.cloud.oss)
                    .ok_or_else(|| "OSS 未配置，不能 --upload-oss apply".to_string())?,
            )
        } else {
            None
        };

        let total = candidates.len();
        let mut records = Vec::new();
        let oss = oss.map(Arc::new);
        let mut completed = 0usize;
        let mut results =
            stream::iter(candidates.into_iter().enumerate().map(|(idx, candidate)| {
                let oss = oss.clone();
                let reuse_existing = args.reuse_existing;
                async move { migrate_one_candidate(idx, candidate, oss, reuse_existing).await }
            }))
            .buffer_unordered(args.concurrency.max(1));

        while let Some(result) = results.next().await {
            completed += 1;
            if completed % 100 == 0 || completed == total {
                eprintln!("[cloud migrate] processed {completed}/{total}");
            }
            let result = result;
            report.uploaded_objects += result.uploaded_objects;
            report.reused_objects += result.reused_objects;
            report.skipped_objects += result.skipped_objects;
            report.conflicts.extend(result.conflicts);
            let Some(record) = result.record else {
                continue;
            };
            records.push(record);
            report.indexed_documents += 1;
            if records.len() >= 100 {
                pg.upsert_document_indexes(&records)
                    .await
                    .map_err(|err| err.to_string())?;
                records.clear();
            }
        }
        pg.upsert_document_indexes(&records)
            .await
            .map_err(|err| err.to_string())?;
    }

    if args.json {
        print_json(&report)
    } else {
        println!(
            "mode={} sessions={} uploads={} company_profiles={} sqlite_files={} uploaded={} reused={} indexed={}",
            report.mode,
            report.counted.sessions,
            report.counted.uploads_and_attachments,
            report.counted.company_profiles,
            report.counted.sqlite_files,
            report.uploaded_objects,
            report.reused_objects,
            report.indexed_documents
        );
        println!(
            "sessions={} changed_session_rows={} skipped_session_rows={} quota_json={} changed_quota_rows={} skipped_quota_rows={} changed_web_auth_users={} skipped_web_auth_users={} changed_web_auth_sessions={} skipped_web_auth_sessions={}",
            report.counted.sessions,
            report.changed_session_rows,
            report.skipped_session_rows,
            report.counted.quota_json,
            report.changed_quota_rows,
            report.skipped_quota_rows,
            report.changed_web_auth_users,
            report.skipped_web_auth_users,
            report.changed_web_auth_sessions,
            report.skipped_web_auth_sessions
        );
        println!(
            "cron_json={} changed_cron_rows={} skipped_cron_rows={}",
            report.counted.cron_json, report.changed_cron_rows, report.skipped_cron_rows
        );
        println!(
            "skill_registry_json={} changed_skill_registry_rows={} skipped_skill_registry_rows={}",
            report.counted.skill_registry_json,
            report.changed_skill_registry_rows,
            report.skipped_skill_registry_rows
        );
        println!(
            "notification_prefs={} changed_notification_prefs_rows={} skipped_notification_prefs_rows={}",
            report.counted.notification_prefs,
            report.changed_notification_prefs_rows,
            report.skipped_notification_prefs_rows
        );
        println!(
            "portfolio_json={} changed_portfolio_rows={} skipped_portfolio_rows={}",
            report.counted.portfolio_json,
            report.changed_portfolio_rows,
            report.skipped_portfolio_rows
        );
        println!(
            "company_profiles={} changed_company_profile_files={} skipped_company_profile_files={}",
            report.counted.company_profiles,
            report.changed_company_profile_files,
            report.skipped_company_profile_files
        );
        println!(
            "changed_llm_audit_rows={} skipped_llm_audit_rows={}",
            report.changed_llm_audit_rows, report.skipped_llm_audit_rows
        );
        for conflict in &report.conflicts {
            println!("conflict: {conflict}");
        }
        Ok(())
    }
}

fn collect_session_imports(candidates: &[MigrationCandidate]) -> Vec<CloudSessionRecord> {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == "session")
        .filter_map(|candidate| {
            let text = std::fs::read_to_string(&candidate.path).ok()?;
            let value = serde_json::from_str::<serde_json::Value>(&text).ok()?;
            let session =
                serde_json::from_value::<hone_memory::session::Session>(value.clone()).ok()?;
            let actor_storage_key = session
                .actor
                .as_ref()
                .map(ActorIdentity::storage_key)
                .or_else(|| {
                    session
                        .session_identity
                        .as_ref()
                        .map(|identity| identity.session_id())
                })
                .unwrap_or_else(|| session.id.clone());
            Some(CloudSessionRecord {
                session_id: session.id,
                actor_storage_key,
                content: value,
            })
        })
        .collect()
}

fn collect_quota_imports(candidates: &[MigrationCandidate]) -> Vec<CloudConversationQuotaImport> {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == "quota")
        .filter_map(|candidate| {
            let actor_storage_key = candidate.actor_storage_key.clone()?;
            let text = std::fs::read_to_string(&candidate.path).ok()?;
            let parsed = serde_json::from_str::<LegacyQuotaJson>(&text).ok()?;
            let quota_date = if parsed.quota_date.trim().is_empty() {
                candidate
                    .path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_string()
            } else {
                parsed.quota_date
            };
            if quota_date.trim().is_empty() {
                return None;
            }
            Some(CloudConversationQuotaImport {
                actor_storage_key,
                quota_date,
                success_count: parsed.success_count,
                in_flight: parsed.in_flight,
                limit: parsed.success_count.saturating_add(parsed.in_flight),
            })
        })
        .collect()
}

fn collect_cron_imports(candidates: &[MigrationCandidate]) -> Vec<CloudCronJobRecord> {
    let mut records = Vec::new();
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.kind == "cron")
    {
        let Ok(text) = std::fs::read_to_string(&candidate.path) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<hone_memory::cron_job::CronJobData>(&text) else {
            continue;
        };
        let Some(actor) = cron_actor_from_data(&data) else {
            continue;
        };
        let actor_storage_key = actor.storage_key();
        let Ok(actor_value) = serde_json::to_value(&actor) else {
            continue;
        };
        for job in data.jobs {
            let Ok(job_value) = serde_json::to_value(&job) else {
                continue;
            };
            records.push(CloudCronJobRecord {
                actor_storage_key: actor_storage_key.clone(),
                job_id: job.id.clone(),
                actor: actor_value.clone(),
                job: job_value,
            });
        }
    }
    records
}

async fn import_skill_registry(
    pg: &CloudPgRuntime,
    from_data_dir: &Path,
    report: &mut MigrationReport,
) -> HoneResult<()> {
    let path = from_data_dir.join("runtime").join("skill_registry.json");
    if !path.exists() {
        return Ok(());
    }
    report.counted.skill_registry_json = 1;
    let raw = std::fs::read_to_string(&path)?;
    let registry = serde_json::from_str::<serde_json::Value>(&raw)
        .map_err(|err| HoneError::Serialization(err.to_string()))?;
    let import_report = pg.import_skill_registry(Some(registry)).await?;
    report.changed_skill_registry_rows = import_report.changed_rows;
    report.skipped_skill_registry_rows = import_report.skipped_rows;
    Ok(())
}

fn collect_notification_prefs_imports(
    candidates: &[MigrationCandidate],
) -> Vec<CloudNotificationPrefsRecord> {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == "notification_prefs")
        .filter_map(|candidate| {
            let actor_storage_key = candidate.path.file_stem()?.to_string_lossy().to_string();
            if actor_storage_key.trim().is_empty() {
                return None;
            }
            let text = std::fs::read_to_string(&candidate.path).ok()?;
            let prefs = serde_json::from_str::<serde_json::Value>(&text).ok()?;
            Some(CloudNotificationPrefsRecord {
                actor_storage_key,
                prefs,
            })
        })
        .collect()
}

fn collect_portfolio_imports(candidates: &[MigrationCandidate]) -> Vec<CloudPortfolioRecord> {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == "portfolio")
        .filter_map(|candidate| {
            let text = std::fs::read_to_string(&candidate.path).ok()?;
            let mut portfolio = serde_json::from_str::<serde_json::Value>(&text).ok()?;
            let stem = candidate.path.file_stem()?.to_string_lossy().to_string();
            let legacy_key = stem.strip_prefix("portfolio_").unwrap_or(&stem);
            let actor = portfolio
                .get("actor")
                .cloned()
                .and_then(|value| serde_json::from_value::<ActorIdentity>(value).ok())
                .or_else(|| ActorIdentity::from_session_id(&format!("Actor_{legacy_key}")))
                .or_else(|| {
                    if legacy_key.trim().is_empty() {
                        None
                    } else {
                        ActorIdentity::new("legacy", legacy_key.to_string(), None::<String>).ok()
                    }
                })?;
            if let Some(object) = portfolio.as_object_mut() {
                object.insert("actor".to_string(), serde_json::to_value(&actor).ok()?);
                object.insert(
                    "user_id".to_string(),
                    serde_json::Value::String(actor.user_id.clone()),
                );
            }
            Some(CloudPortfolioRecord {
                actor_storage_key: actor.storage_key(),
                actor: serde_json::to_value(&actor).ok()?,
                portfolio,
            })
        })
        .collect()
}

fn collect_company_profile_imports(
    candidates: &[MigrationCandidate],
) -> Vec<CloudCompanyProfileFileRecord> {
    let mut records = Vec::new();
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.kind == "company_profile")
    {
        let Some((actor, profile_id, relative_path)) =
            company_profile_identity_from_rel_path(&candidate.relative_path)
        else {
            continue;
        };
        if !relative_path.ends_with(".md") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&candidate.path) else {
            continue;
        };
        let updated_at = candidate
            .path
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(system_time_to_rfc3339)
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        let Ok(actor_value) = serde_json::to_value(&actor) else {
            continue;
        };
        records.push(CloudCompanyProfileFileRecord {
            actor_storage_key: actor.storage_key(),
            actor: actor_value,
            profile_id,
            relative_path,
            content,
            updated_at,
        });
    }
    records
}

fn company_profile_identity_from_rel_path(rel: &str) -> Option<(ActorIdentity, String, String)> {
    let parts = rel.split('/').collect::<Vec<_>>();
    let cp_idx = parts.iter().position(|part| *part == "company_profiles")?;
    if cp_idx < 2 || parts.len() < cp_idx + 3 {
        return None;
    }
    let channel = decode_fs_component(parts[cp_idx - 2]);
    let scoped_user = parts[cp_idx - 1];
    let (channel_scope, user_id) = actor_scope_and_user_from_scoped_key(scoped_user)?;
    let actor = ActorIdentity::new(channel, user_id, channel_scope).ok()?;
    let profile_id = parts[cp_idx + 1].to_string();
    let relative_path = parts[(cp_idx + 2)..].join("/");
    if profile_id.trim().is_empty() || relative_path.trim().is_empty() {
        return None;
    }
    Some((actor, profile_id, relative_path))
}

fn actor_scope_and_user_from_scoped_key(key: &str) -> Option<(Option<String>, String)> {
    let (scope_raw, user_raw) = key.split_once("__")?;
    let scope = decode_fs_component(scope_raw);
    let user_id = decode_fs_component(user_raw);
    if user_id.trim().is_empty() {
        return None;
    }
    let channel_scope = if scope == "direct" { None } else { Some(scope) };
    Some((channel_scope, user_id))
}

fn decode_fs_component(encoded: &str) -> String {
    let mut out = String::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' && i + 2 < bytes.len() {
            let hi = bytes[i + 1];
            let lo = bytes[i + 2];
            if let (Some(h), Some(l)) = (hex_digit(hi), hex_digit(lo)) {
                out.push(char::from(h * 16 + l));
                i += 3;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn system_time_to_rfc3339(value: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = value.into();
    datetime.to_rfc3339()
}

async fn import_llm_audit(
    pg: &CloudPgRuntime,
    from_data_dir: &Path,
    report: &mut MigrationReport,
) -> HoneResult<()> {
    let path = from_data_dir.join("llm_audit.sqlite3");
    if !path.exists() {
        return Ok(());
    }
    let storage = hone_memory::LlmAuditStorage::new_readonly_local(&path)?;
    let batch_size = 500usize;
    let mut offset = 0usize;
    loop {
        let records: Vec<CloudLlmAuditRecord> =
            storage.export_cloud_records_page(batch_size, offset)?;
        if records.is_empty() {
            break;
        }
        let import_report = pg.import_llm_audit_records(&records).await?;
        report.changed_llm_audit_rows += import_report.changed_rows;
        report.skipped_llm_audit_rows += import_report.skipped_rows;
        offset += records.len();
        if records.len() < batch_size {
            break;
        }
    }
    Ok(())
}

fn cron_actor_from_data(data: &hone_memory::cron_job::CronJobData) -> Option<ActorIdentity> {
    if let Some(actor) = data.actor.clone() {
        return Some(actor);
    }
    if data.user_id.trim().is_empty() {
        return None;
    }
    let channel = data
        .jobs
        .first()
        .map(|job| job.channel.clone())
        .filter(|channel| !channel.trim().is_empty())
        .unwrap_or_else(|| "imessage".to_string());
    let scope = data.jobs.first().and_then(|job| job.channel_scope.clone());
    ActorIdentity::new(channel, data.user_id.clone(), scope).ok()
}

struct MigrationOneResult {
    record: Option<CloudDocumentIndex>,
    uploaded_objects: usize,
    reused_objects: usize,
    skipped_objects: usize,
    conflicts: Vec<String>,
}

async fn migrate_one_candidate(
    _idx: usize,
    candidate: MigrationCandidate,
    oss: Option<Arc<OssObjectStore>>,
    reuse_existing: bool,
) -> MigrationOneResult {
    let mut result = MigrationOneResult {
        record: None,
        uploaded_objects: 0,
        reused_objects: 0,
        skipped_objects: 0,
        conflicts: Vec::new(),
    };
    if candidate.kind == "sqlite" {
        result.skipped_objects += 1;
        result.conflicts.push(format!(
            "sqlite structured import pending, skipped blob upload: {}",
            candidate.path.display()
        ));
        return result;
    }
    if candidate.kind == "skill_registry" {
        result.skipped_objects += 1;
        return result;
    }
    if candidate.kind == "notification_prefs" {
        result.skipped_objects += 1;
        return result;
    }
    if candidate.kind == "portfolio" {
        result.skipped_objects += 1;
        return result;
    }
    let bytes = match std::fs::read(&candidate.path) {
        Ok(bytes) => bytes,
        Err(error) => {
            result
                .conflicts
                .push(format!("read failed {}: {error}", candidate.path.display()));
            return result;
        }
    };
    let hash = sha256_hex(&bytes);
    let actor_key = candidate
        .actor_storage_key
        .unwrap_or_else(|| "migration".to_string());
    let doc_id = candidate.document_id;
    let mut oss_uri = format!("local://{}", candidate.path.display());
    if let Some(oss_store) = &oss {
        let key = format!(
            "users/{}/documents/{}/{}",
            sanitize_key_component(&actor_key),
            sanitize_key_component(&candidate.kind),
            sanitize_key_component(&doc_id)
        );
        let mut should_upload = true;
        if reuse_existing {
            let exists = tokio::time::timeout(
                std::time::Duration::from_secs(8),
                oss_store.object_exists(&key),
            )
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or(false);
            if exists {
                should_upload = false;
                result.reused_objects += 1;
                oss_uri = oss_store.object_uri(&key);
            }
        }
        if should_upload {
            match tokio::time::timeout(
                std::time::Duration::from_secs(90),
                oss_store.put_object(&key, bytes.clone(), candidate.content_type),
            )
            .await
            {
                Ok(Ok(())) => {
                    result.uploaded_objects += 1;
                    oss_uri = oss_store.object_uri(&key);
                }
                Ok(Err(error)) => {
                    result
                        .conflicts
                        .push(format!("oss upload failed {key}: {error}"));
                    return result;
                }
                Err(_) => {
                    result.conflicts.push(format!("oss upload timeout {key}"));
                    return result;
                }
            }
        }
    } else {
        result.skipped_objects += 1;
    }
    result.record = Some(CloudDocumentIndex {
        actor_storage_key: actor_key,
        kind: candidate.kind,
        document_id: doc_id,
        oss_uri,
        sha256: hash,
        size_bytes: bytes.len() as i64,
        metadata: json!({ "source_path": candidate.path.to_string_lossy() }),
    });
    result
}

struct MigrationCandidate {
    path: PathBuf,
    relative_path: String,
    actor_storage_key: Option<String>,
    kind: String,
    document_id: String,
    content_type: &'static str,
}

fn collect_candidates(
    root: &Path,
    counts: &mut MigrationCounts,
) -> HoneResult<Vec<MigrationCandidate>> {
    if !root.exists() {
        return Err(HoneError::Config(format!(
            "data dir 不存在: {}",
            root.display()
        )));
    }
    let mut candidates = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_string = rel.to_string_lossy().replace('\\', "/");
        let Some(classification) = classify_path(&rel_string) else {
            counts.other_files += 1;
            continue;
        };
        match classification.kind.as_str() {
            "session" => counts.sessions += 1,
            "upload" => counts.uploads_and_attachments += 1,
            "generated_image" => counts.generated_images += 1,
            "company_profile" => counts.company_profiles += 1,
            "portfolio" => counts.portfolio_json += 1,
            "cron" => counts.cron_json += 1,
            "notification_prefs" => counts.notification_prefs += 1,
            "quota" => counts.quota_json += 1,
            "skill_registry" => counts.skill_registry_json += 1,
            "sqlite" => counts.sqlite_files += 1,
            _ => counts.other_files += 1,
        }
        candidates.push(MigrationCandidate {
            path,
            relative_path: rel_string,
            actor_storage_key: classification.actor_storage_key,
            kind: classification.kind,
            document_id: classification.document_id,
            content_type: classification.content_type,
        });
    }
    Ok(candidates)
}

struct Classification {
    actor_storage_key: Option<String>,
    kind: String,
    document_id: String,
    content_type: &'static str,
}

fn classify_path(rel: &str) -> Option<Classification> {
    let ext = Path::new(rel)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let content_type = match ext.as_str() {
        "json" => "application/json",
        "md" | "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "sqlite" | "sqlite3" | "db" => "application/octet-stream",
        _ => "application/octet-stream",
    };
    let parts = rel.split('/').collect::<Vec<_>>();
    let mut actor = parts
        .iter()
        .find_map(|part| part.strip_prefix("Actor_"))
        .and_then(ActorIdentity::from_session_id)
        .map(|actor| actor.storage_key());
    let doc_id = rel.replace('/', "__");
    let kind = if rel.starts_with("sessions/") && ext == "json" {
        "session"
    } else if rel.contains("/uploads/") || rel.starts_with("uploads/") {
        "upload"
    } else if rel.starts_with("gen_images/") {
        "generated_image"
    } else if rel.contains("company_profiles/") && (ext == "md" || ext == "json") {
        "company_profile"
    } else if rel.starts_with("portfolio/") && ext == "json" {
        "portfolio"
    } else if rel.starts_with("cron_jobs/") && ext == "json" {
        "cron"
    } else if rel.starts_with("notif_prefs/") && ext == "json" {
        "notification_prefs"
    } else if rel == "runtime/skill_registry.json" {
        "skill_registry"
    } else if rel.starts_with("conversation_quota/") && ext == "json" {
        if actor.is_none() {
            actor = parts.get(1).map(|value| (*value).to_string());
        }
        "quota"
    } else if matches!(ext.as_str(), "sqlite" | "sqlite3" | "db") {
        "sqlite"
    } else {
        return None;
    };
    Some(Classification {
        actor_storage_key: actor,
        kind: kind.to_string(),
        document_id: doc_id,
        content_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes() -> Vec<u8> {
        b"\x89PNG\r\n\x1a\nproduction-safe-test-payload".to_vec()
    }

    #[test]
    fn community_asset_validation_checks_size_sha_and_magic() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bytes = png_bytes();
        let path = temp.path().join("asset.png");
        std::fs::write(&path, &bytes).expect("write fixture");
        let entry = CommunityAssetManifestEntry {
            resource_id: 7,
            path: PathBuf::from("asset.png"),
            content_type: "image/png".to_string(),
            byte_size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            source_base_key: Some("safe_source_key".to_string()),
            source_resource_id: None,
            width: Some(3142),
            height: Some(1684),
            captured_at: Some("2026-07-12T13:50:43.385Z".to_string()),
        };

        let validated =
            validate_community_asset_entry(entry, temp.path(), 1024).expect("valid asset");
        assert_eq!(validated.content_type, "image/png");
        assert_eq!(validated.extension, "png");
        assert_eq!(validated.resource_id, 7);
        assert_eq!(
            read_verified_community_asset_bytes(&validated).expect("verified reread"),
            bytes
        );

        std::fs::write(&path, b"changed").expect("replace fixture");
        assert!(
            read_verified_community_asset_bytes(&validated)
                .expect_err("changed file rejected")
                .contains("变化")
        );
    }

    #[test]
    fn community_asset_validation_rejects_magic_mismatch() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bytes = png_bytes();
        let path = temp.path().join("asset.jpg");
        std::fs::write(&path, &bytes).expect("write fixture");
        let entry = CommunityAssetManifestEntry {
            resource_id: 8,
            path,
            content_type: "image/jpeg".to_string(),
            byte_size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            source_base_key: None,
            source_resource_id: None,
            width: None,
            height: None,
            captured_at: None,
        };

        let error = validate_community_asset_entry(entry, temp.path(), 1024)
            .err()
            .expect("magic mismatch");
        assert!(error.1.contains("magic"));
    }

    #[test]
    fn community_asset_key_is_immutable_and_scoped() {
        let sha = "a".repeat(64);
        assert_eq!(
            immutable_community_asset_key("zsxq", "group 1", 42, &sha, "pdf"),
            format!("community/zsxq/group-1/resources/42-{sha}.pdf")
        );
    }

    #[test]
    fn community_asset_target_rejects_display_extension_mismatch() {
        let asset = ValidatedCommunityAsset {
            resource_id: 9,
            path: PathBuf::from("/tmp/not-read-by-this-test.png"),
            content_type: "image/png".to_string(),
            byte_size: 32,
            sha256: "b".repeat(64),
            extension: "png",
            source_base_key: None,
            source_resource_id: None,
            width: None,
            height: None,
            captured_at: None,
        };
        let target = CloudCommunityResourceBackfillTarget {
            resource_id: 9,
            display_name: Some("report.pdf".to_string()),
            source_resource_id: None,
            content_type: None,
            byte_size: None,
            sha256: None,
            oss_uri: None,
            access_state: "metadata_only".to_string(),
            updated_at: "2026-07-12 00:00:00+00".to_string(),
        };
        let error = validate_asset_against_target(&asset, &target).expect_err("mismatch");
        assert!(error.contains("扩展名"));
    }

    #[test]
    fn community_asset_target_accepts_source_xls_name_for_verified_ooxml_workbook() {
        let asset = ValidatedCommunityAsset {
            resource_id: 295,
            path: PathBuf::from("/tmp/not-read-by-this-test.xls"),
            content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                .to_string(),
            byte_size: 32,
            sha256: "d".repeat(64),
            extension: "xlsx",
            source_base_key: None,
            source_resource_id: None,
            width: None,
            height: None,
            captured_at: None,
        };
        let target = CloudCommunityResourceBackfillTarget {
            resource_id: 295,
            display_name: Some("workbook.xls".to_string()),
            source_resource_id: None,
            content_type: None,
            byte_size: None,
            sha256: None,
            oss_uri: None,
            access_state: "metadata_only".to_string(),
            updated_at: "2026-07-12 00:00:00+00".to_string(),
        };

        validate_asset_against_target(&asset, &target)
            .expect("the source mislabeled this verified OOXML workbook as .xls");
    }

    #[test]
    fn community_asset_source_identifier_rejects_urls() {
        assert!(validate_safe_source_identifier(Some("source-key")).is_ok());
        assert!(
            validate_safe_source_identifier(Some("https://files.example/token"))
                .expect_err("url rejected")
                .contains("URL")
        );
    }

    #[test]
    fn community_asset_target_match_is_idempotent_and_source_aware() {
        let asset = ValidatedCommunityAsset {
            resource_id: 10,
            path: PathBuf::from("/tmp/not-read-by-this-test.png"),
            content_type: "image/png".to_string(),
            byte_size: 32,
            sha256: "c".repeat(64),
            extension: "png",
            source_base_key: Some("source-object-10".to_string()),
            source_resource_id: None,
            width: Some(10),
            height: Some(10),
            captured_at: None,
        };
        let mut target = CloudCommunityResourceBackfillTarget {
            resource_id: 10,
            display_name: Some("image-10".to_string()),
            source_resource_id: Some("source-object-10".to_string()),
            content_type: Some("image/png".to_string()),
            byte_size: Some(32),
            sha256: Some("c".repeat(64)),
            oss_uri: Some("oss://bucket/key".to_string()),
            access_state: "stored".to_string(),
            updated_at: "2026-07-12 00:00:00+00".to_string(),
        };
        assert!(target_matches_asset(&target, &asset, "oss://bucket/key"));

        target.source_resource_id = None;
        assert!(!target_matches_asset(&target, &asset, "oss://bucket/key"));
    }

    #[test]
    fn community_asset_report_keeps_required_operation_counters() {
        let report = CommunityAssetsReport {
            ok: true,
            mode: "dry-run",
            manifest: "manifest.json".to_string(),
            source: "zsxq".to_string(),
            external_id: "group".to_string(),
            total: 1,
            validated: 1,
            uploaded: 0,
            reused: 1,
            updated: 0,
            skipped: 1,
            would_upload: 0,
            would_update: 0,
            conflicts: Vec::new(),
            items: Vec::new(),
        };
        let value = serde_json::to_value(report).expect("serialize report");
        for key in ["uploaded", "reused", "updated", "skipped", "conflicts"] {
            assert!(value.get(key).is_some(), "missing report field {key}");
        }
    }

    fn publish_content(content_id: i64) -> hone_core::cloud_runtime::CloudCommunityContentRecord {
        hone_core::cloud_runtime::CloudCommunityContentRecord {
            content_id,
            author_name: Some("source author".to_string()),
            published_at: Some("2026-07-19 00:00:00+00".to_string()),
            published_at_raw: Some("2026-07-19 08:00".to_string()),
            content_type: "post".to_string(),
            body_text: format!("content {content_id}"),
            body_blocks: serde_json::json!([]),
            crawl_status: "complete".to_string(),
            resources: Vec::new(),
        }
    }

    #[test]
    fn community_publish_keys_and_page_cursor_match_worker_contract() {
        let prefix = "community/zsxq/51115212285814/delivery/v1";
        assert_eq!(
            community_publish_latest_key(prefix),
            format!("{prefix}/feed/latest.json")
        );
        assert_eq!(
            community_publish_page_key(prefix, 42),
            format!("{prefix}/feed/pages/42.json")
        );
        assert_eq!(
            community_publish_descriptor_key(prefix, 9, "abcdef012345"),
            format!("{prefix}/resources/9/abcdef012345.json")
        );
        assert_eq!(
            community_publish_active_index_key(prefix),
            format!("{prefix}/resources/active.json")
        );

        let pages = build_published_community_pages(
            vec![
                CommunityPublishRawPage {
                    before: None,
                    items: vec![publish_content(50)],
                },
                CommunityPublishRawPage {
                    before: Some(42),
                    items: vec![publish_content(41)],
                },
            ],
            "51115212285814",
            &BTreeMap::new(),
        );
        let latest = serde_json::to_value(&pages[0].1).expect("serialize latest");
        assert_eq!(latest["next_before"], 42);
        assert_eq!(latest["latest_content_id"], 50);
        let older = serde_json::to_value(&pages[1].1).expect("serialize older");
        assert!(older["next_before"].is_null());
        assert!(older.get("latest_content_id").is_none());
    }

    #[test]
    fn community_publish_descriptor_validation_is_fail_closed() {
        let asset_prefix = "community/zsxq/51115212285814/resources";
        let sha256 = "abcdef012345".to_string() + &"0".repeat(52);
        let mut descriptor = CommunityResourceDescriptor {
            resource_id: 7,
            version: "abcdef012345".to_string(),
            object_key: format!("{asset_prefix}/7-{sha256}.png"),
            sha256: sha256.clone(),
            content_type: "image/png".to_string(),
            byte_size: 123,
            display_name: Some("chart.png".to_string()),
        };
        validate_community_resource_descriptor(&descriptor, asset_prefix)
            .expect("valid descriptor");

        descriptor.object_key = format!("community/other/resources/7-{sha256}.png");
        assert!(
            validate_community_resource_descriptor(&descriptor, asset_prefix)
                .expect_err("cross-prefix key rejected")
                .contains("asset-prefix")
        );
        descriptor.object_key = format!("{asset_prefix}/7-{sha256}.png");
        descriptor.version = "ABCDEF012345".to_string();
        assert!(validate_community_resource_descriptor(&descriptor, asset_prefix).is_err());
        descriptor.version = "abcdef012345".to_string();
        descriptor.byte_size = COMMUNITY_PUBLISH_MAX_ASSET_BYTES + 1;
        assert!(
            validate_community_resource_descriptor(&descriptor, asset_prefix)
                .expect_err("oversized edge asset rejected")
                .contains("byte_size")
        );
        descriptor.byte_size = 123;
        descriptor.display_name = Some("密".repeat(342));
        assert!(
            validate_community_resource_descriptor(&descriptor, asset_prefix)
                .expect_err("display name above 1024 UTF-8 bytes rejected")
                .contains("1024")
        );
    }

    #[test]
    fn community_publish_asset_key_binds_exact_prefix_resource_sha_and_safe_extension() {
        let asset_prefix = "community/zsxq/51115212285814/resources";
        let sha256 = "a".repeat(64);
        let valid = CommunityResourceDescriptor {
            resource_id: 42,
            version: sha256[..12].to_string(),
            object_key: format!("{asset_prefix}/42-{sha256}.pdf"),
            sha256: sha256.clone(),
            content_type: "application/pdf".to_string(),
            byte_size: 10,
            display_name: Some("report.pdf".to_string()),
        };
        validate_community_publish_asset_key(&valid, asset_prefix)
            .expect("historical immutable key contract accepted");
        for (extension, content_type) in [
            ("png", "image/png"),
            ("jpg", "image/jpeg"),
            ("pdf", "application/pdf"),
            (
                "docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            (
                "xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                "pptx",
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ),
        ] {
            let mut historical = valid.clone();
            historical.object_key = format!("{asset_prefix}/42-{sha256}.{extension}");
            historical.content_type = content_type.to_string();
            validate_community_publish_asset_key(&historical, asset_prefix)
                .unwrap_or_else(|error| panic!("historical .{extension} rejected: {error}"));
        }

        let mut wrong_resource = valid.clone();
        wrong_resource.object_key = format!("{asset_prefix}/41-{sha256}.pdf");
        assert!(
            validate_community_publish_asset_key(&wrong_resource, asset_prefix)
                .expect_err("wrong resource id rejected")
                .contains("resource_id")
        );

        let mut wrong_sha = valid.clone();
        wrong_sha.object_key = format!("{asset_prefix}/42-{}.pdf", "b".repeat(64));
        assert!(validate_community_publish_asset_key(&wrong_sha, asset_prefix).is_err());

        let mut nested = valid.clone();
        nested.object_key = format!("{asset_prefix}/nested/42-{sha256}.pdf");
        assert!(
            validate_community_publish_asset_key(&nested, asset_prefix)
                .expect_err("nested object rejected")
                .contains("额外子目录")
        );

        let mut unsafe_extension = valid.clone();
        unsafe_extension.object_key = format!("{asset_prefix}/42-{sha256}.svg");
        unsafe_extension.content_type = "image/svg+xml".to_string();
        assert!(
            validate_community_publish_asset_key(&unsafe_extension, asset_prefix)
                .expect_err("unsafe extension rejected")
                .contains("allowlist")
        );

        let mut mismatched_type = valid;
        mismatched_type.content_type = "image/png".to_string();
        assert!(
            validate_community_publish_asset_key(&mismatched_type, asset_prefix)
                .expect_err("content type mismatch rejected")
                .contains("content_type")
        );
    }

    #[test]
    fn community_publish_asset_bytes_require_exact_size_sha_and_content_type() {
        let bytes = b"verified-payload";
        let sha256 = sha256_hex(bytes);
        let descriptor = CommunityResourceDescriptor {
            resource_id: 88,
            version: sha256[..12].to_string(),
            object_key: format!("community/zsxq/51115212285814/resources/88-{sha256}.png"),
            sha256,
            content_type: "image/png".to_string(),
            byte_size: bytes.len() as i64,
            display_name: Some("chart.png".to_string()),
        };
        validate_community_publish_asset_bytes(&descriptor, bytes, "image/png; charset=binary")
            .expect("exact archived bytes accepted");

        let same_size_wrong_bytes = b"tampered-payload";
        assert_eq!(same_size_wrong_bytes.len(), bytes.len());
        assert!(
            validate_community_publish_asset_bytes(&descriptor, same_size_wrong_bytes, "image/png")
                .expect_err("same-size wrong object rejected")
                .contains("sha256")
        );
        assert!(
            validate_community_publish_asset_bytes(&descriptor, b"short", "image/png")
                .expect_err("wrong size rejected")
                .contains("大小冲突")
        );
        assert!(
            validate_community_publish_asset_bytes(&descriptor, bytes, "application/pdf")
                .expect_err("object content type mismatch rejected")
                .contains("content_type")
        );
    }

    #[test]
    fn community_publish_rejects_generic_s3_and_empty_archive() {
        validate_community_publish_provider("r2").expect("r2 accepted");
        validate_community_publish_provider("cloudflare_r2").expect("cloudflare r2 accepted");
        assert!(validate_community_publish_provider("s3").is_err());
        assert!(validate_community_publish_provider("aliyun_oss").is_err());
        assert!(validate_non_empty_community_publish_archive(&[]).is_err());
        assert!(validate_community_publish_page_size(0).is_err());
        assert!(validate_community_publish_page_size(51).is_err());
    }

    #[tokio::test]
    async fn community_publish_retry_stops_on_success_and_returns_last_error() {
        let first_attempts = std::cell::Cell::new(0);
        let first = community_publish_with_retry(3, 0, "empty", || {
            first_attempts.set(first_attempts.get() + 1);
            std::future::ready(Ok::<_, String>("first"))
        })
        .await
        .expect("first attempt succeeds");
        assert_eq!(first, "first");
        assert_eq!(first_attempts.get(), 1);

        let eventual_attempts = std::cell::Cell::new(0);
        let eventual = community_publish_with_retry(3, 0, "empty", || {
            let attempt = eventual_attempts.get() + 1;
            eventual_attempts.set(attempt);
            std::future::ready(if attempt < 3 {
                Err(format!("transient {attempt}"))
            } else {
                Ok("eventual")
            })
        })
        .await
        .expect("third attempt succeeds");
        assert_eq!(eventual, "eventual");
        assert_eq!(eventual_attempts.get(), 3);

        let failed_attempts = std::cell::Cell::new(0);
        let failure = community_publish_with_retry::<(), _, _>(3, 0, "empty", || {
            let attempt = failed_attempts.get() + 1;
            failed_attempts.set(attempt);
            std::future::ready(Err(format!("failure {attempt}")))
        })
        .await
        .expect_err("all attempts fail");
        assert_eq!(failure, "failure 3");
        assert_eq!(failed_attempts.get(), 3);
    }

    #[test]
    fn community_publish_plan_always_places_latest_last() {
        let object = |kind, key: &str| PlannedCommunityPublishObject {
            kind,
            key: key.to_string(),
            bytes: Vec::new(),
        };
        let ordered = order_community_publish_objects(
            vec![
                object(CommunityPublishObjectKind::Descriptor, "descriptor-b"),
                object(CommunityPublishObjectKind::Descriptor, "descriptor-a"),
            ],
            object(CommunityPublishObjectKind::ActiveIndex, "active"),
            vec![
                object(CommunityPublishObjectKind::Latest, "latest"),
                object(CommunityPublishObjectKind::Page, "page"),
            ],
        )
        .expect("valid plan");
        assert_eq!(
            ordered
                .iter()
                .map(|object| object.key.as_str())
                .collect::<Vec<_>>(),
            vec!["descriptor-a", "descriptor-b", "active", "page", "latest"]
        );
        assert_eq!(
            ordered.last().map(|object| object.kind),
            Some(CommunityPublishObjectKind::Latest)
        );
    }

    #[test]
    fn community_publish_active_index_is_mutable_and_tombstones_removed_resources() {
        let prefix = "community/zsxq/51115212285814/delivery/v1";
        let sha256 = "a".repeat(64);
        let descriptor = CommunityResourceDescriptor {
            resource_id: 42,
            version: sha256[..12].to_string(),
            sha256: sha256.clone(),
            object_key: format!("community/zsxq/51115212285814/resources/42-{sha256}.png"),
            content_type: "image/png".to_string(),
            byte_size: 9,
            display_name: Some("chart.png".to_string()),
        };
        let pages = || {
            build_published_community_pages(
                vec![CommunityPublishRawPage {
                    before: None,
                    items: vec![publish_content(50)],
                }],
                "51115212285814",
                &BTreeMap::new(),
            )
        };
        let active_bytes = |descriptors: &BTreeMap<i64, CommunityResourceDescriptor>| {
            plan_community_publish_objects(prefix, descriptors, pages())
                .expect("publication plan")
                .into_iter()
                .find(|object| object.kind == CommunityPublishObjectKind::ActiveIndex)
                .expect("active index")
                .bytes
        };

        let with_resource = BTreeMap::from([(42, descriptor)]);
        let active: serde_json::Value =
            serde_json::from_slice(&active_bytes(&with_resource)).expect("active JSON");
        assert_eq!(active["v"], 1);
        assert_eq!(active["resources"]["42"], sha256[..12]);

        let tombstone: serde_json::Value =
            serde_json::from_slice(&active_bytes(&BTreeMap::new())).expect("empty active JSON");
        assert_eq!(tombstone["resources"], serde_json::json!({}));
        assert!(community_publish_object_is_mutable(
            CommunityPublishObjectKind::ActiveIndex
        ));
    }
}
