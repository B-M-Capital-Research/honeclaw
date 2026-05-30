use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use clap::{Args, Subcommand};
use futures::{StreamExt, stream};
use hone_core::cloud_runtime::{
    CloudConversationQuotaImport, CloudDocumentIndex, CloudPgRuntime, CloudSessionRecord,
    OssObjectStore, RuntimeRole, local_durable_dependencies, sanitize_key_component, sha256_hex,
};
use hone_core::config::OssConfig;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use serde::Serialize;
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

pub(crate) async fn run_cloud_command(
    config_path: Option<&Path>,
    command: CloudCommands,
) -> Result<(), String> {
    match command {
        CloudCommands::Doctor(args) => run_doctor(config_path, args).await,
        CloudCommands::Migrate(args) => run_migrate(config_path, args).await,
        CloudCommands::ObjectBench(args) => run_object_bench(config_path, args).await,
    }
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
    let narrow_modes = [args.quota_only, args.session_only, args.web_auth_only]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();
    if narrow_modes > 1 {
        return Err("--quota-only / --session-only / --web-auth-only 不能同时使用".to_string());
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
        skipped_objects: 0,
        conflicts: Vec::new(),
    };

    let candidates = collect_candidates(&args.from_data_dir, &mut report.counted)
        .map_err(|err| err.to_string())?;
    if args.apply {
        let pg = CloudPgRuntime::from_cloud_config(&config.cloud)
            .ok_or_else(|| "Postgres 未配置，不能 apply migration".to_string())?;
        pg.ensure_schema().await.map_err(|err| err.to_string())?;
        if !args.quota_only && !args.web_auth_only {
            let session_imports = collect_session_imports(&candidates);
            let session_report = pg
                .import_session_records(&session_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_session_rows = session_report.changed_rows;
            report.skipped_session_rows = session_report.skipped_rows;
        }
        if !args.session_only && !args.web_auth_only {
            let quota_imports = collect_quota_imports(&candidates);
            let quota_report = pg
                .import_conversation_quota(&quota_imports)
                .await
                .map_err(|err| err.to_string())?;
            report.changed_quota_rows = quota_report.changed_rows;
            report.skipped_quota_rows = quota_report.skipped_rows;
        }
        if !args.quota_only && !args.session_only {
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
        if args.quota_only || args.session_only || args.web_auth_only {
            return if args.json {
                print_json(&report)
            } else {
                println!(
                    "mode={} sessions={} changed_session_rows={} skipped_session_rows={} quota_json={} changed_quota_rows={} skipped_quota_rows={} changed_web_auth_users={} skipped_web_auth_users={} changed_web_auth_sessions={} skipped_web_auth_sessions={}",
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
                    report.skipped_web_auth_sessions
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
            "sqlite" => counts.sqlite_files += 1,
            _ => counts.other_files += 1,
        }
        candidates.push(MigrationCandidate {
            path,
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
