use chrono::Utc;
use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::{HoneError, HoneResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

const SKILL_REGISTRY_VERSION: u32 = 1;
static CLOUD_SKILL_REGISTRY: OnceLock<RwLock<Option<CloudPgRuntime>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRegistryEntry {
    pub enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRegistry {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub entries: BTreeMap<String, SkillRegistryEntry>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self {
            version: SKILL_REGISTRY_VERSION,
            updated_at: None,
            entries: BTreeMap::new(),
        }
    }
}

impl SkillRegistry {
    pub fn is_enabled(&self, skill_id: &str) -> bool {
        self.entries
            .get(skill_id)
            .map(|entry| entry.enabled)
            .unwrap_or(true)
    }
}

pub fn configure_cloud_skill_registry(postgres: Option<CloudPgRuntime>) {
    let lock = CLOUD_SKILL_REGISTRY.get_or_init(|| RwLock::new(None));
    match lock.write() {
        Ok(mut guard) => *guard = postgres,
        Err(error) => tracing::warn!("skill registry cloud runtime lock poisoned: {error}"),
    }
}

fn cloud_skill_registry() -> Option<CloudPgRuntime> {
    CLOUD_SKILL_REGISTRY
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()))
}

pub fn default_skill_registry_path(custom_dir: &Path) -> PathBuf {
    custom_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("./data"))
        .join("runtime")
        .join("skill_registry.json")
}

pub fn read_skill_registry(path: &Path) -> Result<SkillRegistry, String> {
    if let Some(postgres) = cloud_skill_registry() {
        return read_cloud_skill_registry(postgres).map_err(|err| err.to_string());
    }

    if !path.exists() {
        return Ok(SkillRegistry::default());
    }

    let raw = fs::read_to_string(path)
        .map_err(|err| format!("读取 skill registry 失败 ({}): {err}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|err| format!("解析 skill registry 失败 ({}): {err}", path.display()))
}

pub fn load_skill_registry(path: &Path) -> SkillRegistry {
    match read_skill_registry(path) {
        Ok(registry) => registry,
        Err(error) => {
            tracing::warn!("{error}");
            SkillRegistry::default()
        }
    }
}

pub fn write_skill_registry(path: &Path, registry: &SkillRegistry) -> Result<(), String> {
    if let Some(postgres) = cloud_skill_registry() {
        return write_cloud_skill_registry(postgres, registry).map_err(|err| err.to_string());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("创建 skill registry 目录失败 ({}): {err}", parent.display()))?;
    }

    let payload = serde_json::to_string_pretty(registry)
        .map_err(|err| format!("序列化 skill registry 失败: {err}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let tmp_path = path.with_extension(format!("json.tmp.{stamp}"));
    fs::write(&tmp_path, payload).map_err(|err| {
        format!(
            "写入 skill registry 临时文件失败 ({}): {err}",
            tmp_path.display()
        )
    })?;
    fs::rename(&tmp_path, path).map_err(|err| {
        format!(
            "提交 skill registry 变更失败 ({} -> {}): {err}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

pub fn set_skill_enabled(
    path: &Path,
    skill_id: &str,
    enabled: bool,
) -> Result<SkillRegistry, String> {
    let normalized = skill_id.trim();
    if normalized.is_empty() {
        return Err("skill_id 不能为空".to_string());
    }

    let mut registry = read_skill_registry(path)?;
    let now = Utc::now().to_rfc3339();
    registry.updated_at = Some(now.clone());
    if enabled {
        registry.entries.remove(normalized);
    } else {
        registry.entries.insert(
            normalized.to_string(),
            SkillRegistryEntry {
                enabled: false,
                updated_at: now,
            },
        );
    }
    write_skill_registry(path, &registry)?;
    Ok(registry)
}

pub fn reset_skill_registry(path: &Path) -> Result<SkillRegistry, String> {
    let registry = SkillRegistry {
        version: SKILL_REGISTRY_VERSION,
        updated_at: Some(Utc::now().to_rfc3339()),
        entries: BTreeMap::new(),
    };
    write_skill_registry(path, &registry)?;
    Ok(registry)
}

fn read_cloud_skill_registry(postgres: CloudPgRuntime) -> HoneResult<SkillRegistry> {
    let value = run_cloud_skill_registry(async move { postgres.get_skill_registry().await })?;
    match value {
        Some(value) => serde_json::from_value::<SkillRegistry>(value)
            .map_err(|err| HoneError::Serialization(err.to_string())),
        None => Ok(SkillRegistry::default()),
    }
}

fn write_cloud_skill_registry(
    postgres: CloudPgRuntime,
    registry: &SkillRegistry,
) -> HoneResult<()> {
    let value =
        serde_json::to_value(registry).map_err(|err| HoneError::Serialization(err.to_string()))?;
    run_cloud_skill_registry(async move { postgres.import_skill_registry(Some(value)).await })?;
    Ok(())
}

fn run_cloud_skill_registry<T, F>(future: F) -> HoneResult<T>
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
        .map_err(|_| HoneError::Storage("cloud skill registry worker panicked".to_string()))?;
    }
    let runtime =
        tokio::runtime::Runtime::new().map_err(|err| HoneError::Config(err.to_string()))?;
    runtime.block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), stamp));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn default_registry_path_follows_custom_dir_parent() {
        let root = make_temp_dir("hone_skill_registry_path");
        let custom_dir = root.join("custom_skills");
        let path = default_skill_registry_path(&custom_dir);
        assert_eq!(path, root.join("runtime").join("skill_registry.json"));
    }

    #[test]
    fn set_skill_enabled_persists_sparse_disabled_entries() {
        let root = make_temp_dir("hone_skill_registry_set");
        let path = root.join("runtime").join("skill_registry.json");

        let registry = set_skill_enabled(&path, "alpha", false).expect("disable alpha");
        assert!(!registry.is_enabled("alpha"));
        assert!(path.exists());

        let registry = set_skill_enabled(&path, "alpha", true).expect("enable alpha");
        assert!(registry.is_enabled("alpha"));
        assert!(!registry.entries.contains_key("alpha"));
    }

    #[test]
    fn load_skill_registry_falls_back_to_default_for_invalid_json() {
        let root = make_temp_dir("hone_skill_registry_invalid");
        let path = root.join("runtime").join("skill_registry.json");
        fs::create_dir_all(path.parent().expect("parent")).expect("runtime dir");
        fs::write(&path, "{not json").expect("write invalid");

        let registry = load_skill_registry(&path);
        assert!(registry.entries.is_empty());
        assert!(registry.is_enabled("alpha"));
    }

    #[test]
    fn configure_cloud_skill_registry_accepts_none() {
        configure_cloud_skill_registry(None);
        let root = make_temp_dir("hone_skill_registry_cloud_none");
        let path = root.join("runtime").join("skill_registry.json");
        let registry = load_skill_registry(&path);
        assert!(registry.entries.is_empty());
    }
}
