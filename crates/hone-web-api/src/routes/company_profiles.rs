use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use chrono::Utc;
use hone_core::ActorIdentity;
use hone_core::cloud_runtime::{CloudCompanyProfileSpaceRecord, CloudPgRuntime};
use hone_memory::{
    CompanyProfileConflictDecision, CompanyProfileImportApplyInput, CompanyProfileImportMode,
    ProfileSpaceSummary,
};
use serde_json::json;
use tracing::warn;

use crate::routes::{json_error, require_actor};
use crate::state::AppState;
use crate::types::UserIdQuery;

const COMPANY_PROFILE_TRANSFER_MAX_BYTES: usize = 20 * 1024 * 1024;
const COMPANY_PROFILE_SPACES_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
struct CompanyProfileSpacesCache {
    value: Option<serde_json::Value>,
    updated_at: Option<Instant>,
    refreshing: bool,
}

static COMPANY_PROFILE_SPACES_CACHE: LazyLock<Mutex<CompanyProfileSpacesCache>> =
    LazyLock::new(|| Mutex::new(CompanyProfileSpacesCache::default()));

pub(crate) async fn handle_company_profile_spaces(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(cached) = cached_company_profile_spaces(false) {
        return Json(cached);
    }
    if !mark_company_profile_spaces_refreshing() {
        return Json(
            cached_company_profile_spaces(true).unwrap_or_else(|| json!({ "actors": [] })),
        );
    }

    let result = if state
        .core
        .config
        .cloud
        .effective_mode()
        .is_cloud_authoritative()
    {
        if let Some(postgres) = CloudPgRuntime::from_cloud_config(&state.core.config.cloud) {
            list_cloud_company_profile_spaces(postgres).await
        } else {
            Ok(Vec::new())
        }
    } else {
        Ok(state.core.company_profile_storage.list_profile_spaces_raw())
    };

    let spaces = match result {
        Ok(spaces) => spaces,
        Err(error) => {
            warn!(%error, "failed to list company profile spaces");
            clear_company_profile_spaces_refreshing();
            return Json(
                cached_company_profile_spaces(true).unwrap_or_else(|| json!({ "actors": [] })),
            );
        }
    };
    let value = json!({ "actors": spaces });
    update_company_profile_spaces_cache(value.clone());
    Json(value)
}

pub(crate) async fn handle_company_profiles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let profiles = state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .list_profiles_raw();
    Json(json!({ "profiles": profiles })).into_response()
}

pub(crate) async fn handle_company_profile_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .get_profile_raw(&id)
    {
        Ok(Some(profile)) => Json(json!({ "profile": profile })).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "company profile not found"),
        Err(err) => json_error(StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_delete_company_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .delete_profile(&id)
    {
        Ok(true) => {
            clear_company_profile_spaces_cache();
            Json(json!({ "ok": true })).into_response()
        }
        Ok(false) => json_error(StatusCode::NOT_FOUND, "company profile not found"),
        Err(err) => json_error(StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_export_company_profiles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };

    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .export_bundle()
    {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/zip"),
            );
            let disposition = format!(
                "attachment; filename=\"{}\"",
                build_company_profile_export_filename(&actor)
            );
            match HeaderValue::from_str(&disposition) {
                Ok(value) => {
                    headers.insert(header::CONTENT_DISPOSITION, value);
                    (StatusCode::OK, headers, bytes).into_response()
                }
                Err(err) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("构造下载文件名失败: {err}"),
                ),
            }
        }
        Err(err) => json_error(StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_preview_import_company_profiles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
    multipart: Multipart,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let (bundle, _, _) = match read_transfer_form(multipart).await {
        Ok(payload) => payload,
        Err(err) => return json_error(StatusCode::BAD_REQUEST, err),
    };

    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .preview_import_bundle(&bundle)
    {
        Ok(preview) => Json(json!({ "preview": preview })).into_response(),
        Err(err) => json_error(StatusCode::BAD_REQUEST, err),
    }
}

pub(crate) async fn handle_apply_import_company_profiles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
    multipart: Multipart,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let (bundle, mode_text, decisions_text) = match read_transfer_form(multipart).await {
        Ok(payload) => payload,
        Err(err) => return json_error(StatusCode::BAD_REQUEST, err),
    };

    let mode = match mode_text {
        Some(raw) => match parse_import_mode(&raw) {
            Ok(mode) => Some(mode),
            Err(err) => return json_error(StatusCode::BAD_REQUEST, err),
        },
        None => None,
    };
    let decisions = match decisions_text {
        Some(raw) => {
            match serde_json::from_str::<BTreeMap<String, CompanyProfileConflictDecision>>(&raw) {
                Ok(decisions) => decisions,
                Err(err) => {
                    return json_error(
                        StatusCode::BAD_REQUEST,
                        format!("解析导入冲突决策失败: {err}"),
                    );
                }
            }
        }
        None => BTreeMap::new(),
    };

    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .apply_import_bundle(&bundle, CompanyProfileImportApplyInput { mode, decisions })
    {
        Ok(result) => {
            clear_company_profile_spaces_cache();
            Json(json!({ "result": result })).into_response()
        }
        Err(err) => json_error(StatusCode::BAD_REQUEST, err),
    }
}

async fn list_cloud_company_profile_spaces(
    postgres: CloudPgRuntime,
) -> Result<Vec<ProfileSpaceSummary>, String> {
    let records = tokio::time::timeout(
        Duration::from_secs(8),
        postgres.list_company_profile_spaces_cached(),
    )
    .await
    .map_err(|_| "company profile space list timed out".to_string())?
    .map_err(|error| error.to_string())?;
    Ok(records
        .into_iter()
        .filter_map(company_profile_space_from_cloud_record)
        .collect())
}

fn company_profile_space_from_cloud_record(
    record: CloudCompanyProfileSpaceRecord,
) -> Option<ProfileSpaceSummary> {
    let actor = serde_json::from_value::<ActorIdentity>(record.actor).ok()?;
    Some(ProfileSpaceSummary {
        channel: actor.channel,
        user_id: actor.user_id,
        channel_scope: actor.channel_scope,
        profile_count: record.profile_count,
        updated_at: record.updated_at,
    })
}

fn cached_company_profile_spaces(allow_stale: bool) -> Option<serde_json::Value> {
    let guard = COMPANY_PROFILE_SPACES_CACHE.lock().ok()?;
    let updated_at = guard.updated_at?;
    if allow_stale || updated_at.elapsed() < COMPANY_PROFILE_SPACES_CACHE_TTL {
        return guard.value.clone();
    }
    None
}

fn mark_company_profile_spaces_refreshing() -> bool {
    let Ok(mut guard) = COMPANY_PROFILE_SPACES_CACHE.lock() else {
        return true;
    };
    if guard.refreshing {
        return false;
    }
    guard.refreshing = true;
    true
}

fn clear_company_profile_spaces_refreshing() {
    if let Ok(mut guard) = COMPANY_PROFILE_SPACES_CACHE.lock() {
        guard.refreshing = false;
    }
}

fn update_company_profile_spaces_cache(value: serde_json::Value) {
    if let Ok(mut guard) = COMPANY_PROFILE_SPACES_CACHE.lock() {
        guard.value = Some(value);
        guard.updated_at = Some(Instant::now());
        guard.refreshing = false;
    }
}

fn clear_company_profile_spaces_cache() {
    if let Ok(mut guard) = COMPANY_PROFILE_SPACES_CACHE.lock() {
        guard.value = None;
        guard.updated_at = None;
        guard.refreshing = false;
    }
}

async fn read_transfer_form(
    mut multipart: Multipart,
) -> Result<(Vec<u8>, Option<String>, Option<String>), String> {
    let mut bundle = None;
    let mut mode = None;
    let mut decisions = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| format!("读取 multipart 失败: {err}"))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "bundle" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|err| format!("读取画像包文件失败: {err}"))?;
                if bytes.len() > COMPANY_PROFILE_TRANSFER_MAX_BYTES {
                    return Err(format!(
                        "画像包过大，最大只支持 {} MB",
                        COMPANY_PROFILE_TRANSFER_MAX_BYTES / 1024 / 1024
                    ));
                }
                bundle = Some(bytes.to_vec());
            }
            "mode" => {
                mode = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| format!("读取导入模式失败: {err}"))?,
                );
            }
            "decisions" => {
                decisions = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| format!("读取冲突决策失败: {err}"))?,
                );
            }
            _ => {}
        }
    }

    let bundle = bundle.ok_or_else(|| "缺少画像包文件字段 bundle".to_string())?;
    Ok((bundle, mode, decisions))
}

fn parse_import_mode(raw: &str) -> Result<CompanyProfileImportMode, String> {
    match raw.trim() {
        "keep_existing" => Ok(CompanyProfileImportMode::KeepExisting),
        "replace_all" => Ok(CompanyProfileImportMode::ReplaceAll),
        "interactive" => Ok(CompanyProfileImportMode::Interactive),
        other => Err(format!("不支持的导入模式: {other}")),
    }
}

fn build_company_profile_export_filename(actor: &hone_core::ActorIdentity) -> String {
    let channel = download_component(&actor.channel);
    let scope = download_component(actor.channel_scope.as_deref().unwrap_or("direct"));
    let user_id = download_component(&actor.user_id);
    let date = Utc::now().format("%Y%m%d").to_string();
    format!("company-profiles-{channel}-{scope}-{user_id}-{date}.zip")
}

fn download_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            out.push(char::from(*byte));
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

#[cfg(test)]
mod tests {
    use super::{build_company_profile_export_filename, parse_import_mode};

    #[test]
    fn parse_import_mode_accepts_supported_values() {
        assert!(matches!(
            parse_import_mode("keep_existing"),
            Ok(hone_memory::CompanyProfileImportMode::KeepExisting)
        ));
        assert!(matches!(
            parse_import_mode("replace_all"),
            Ok(hone_memory::CompanyProfileImportMode::ReplaceAll)
        ));
        assert!(matches!(
            parse_import_mode("interactive"),
            Ok(hone_memory::CompanyProfileImportMode::Interactive)
        ));
        assert!(parse_import_mode("invalid").is_err());
    }

    #[test]
    fn export_filename_is_sanitized_and_stable() {
        let actor = hone_core::ActorIdentity::new("discord", "ou_123:abc", Some("group:watch"))
            .expect("actor");
        let filename = build_company_profile_export_filename(&actor);
        assert!(filename.starts_with("company-profiles-discord-group-watch-ou_123-abc-"));
        assert!(filename.ends_with(".zip"));
    }
}
