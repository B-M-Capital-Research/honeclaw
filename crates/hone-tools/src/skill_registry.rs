use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SKILL_REGISTRY_VERSION: u32 = 1;

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

pub fn default_skill_registry_path(custom_dir: &Path) -> PathBuf {
    custom_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("./data"))
        .join("runtime")
        .join("skill_registry.json")
}

pub fn read_skill_registry(path: &Path) -> Result<SkillRegistry, String> {
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
        assert_eq!(registry.is_enabled("alpha"), false);
        assert!(path.exists());

        let registry = set_skill_enabled(&path, "alpha", true).expect("enable alpha");
        assert_eq!(registry.is_enabled("alpha"), true);
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
}
