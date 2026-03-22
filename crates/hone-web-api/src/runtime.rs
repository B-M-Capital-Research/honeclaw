use std::path::PathBuf;

use hone_core::config::HoneConfig;

const DEFAULT_PORT: u16 = 8077;

pub fn web_dist_dir() -> PathBuf {
    std::env::var("HONE_WEB_DIST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("packages/app/dist"))
}

pub fn runtime_config_path() -> String {
    std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string())
}

pub fn runtime_port() -> u16 {
    std::env::var("HONE_WEB_PORT")
        .ok()
        .or_else(|| std::env::var("WEB_TEST_PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

pub fn runtime_deployment_mode() -> String {
    std::env::var("HONE_DEPLOYMENT_MODE").unwrap_or_else(|_| "local".to_string())
}

pub fn runtime_disable_auto_open() -> bool {
    std::env::var("HONE_DISABLE_AUTO_OPEN")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

pub fn apply_runtime_config_overrides(config: &mut HoneConfig) {
    if let Ok(path) = std::env::var("HONE_CONFIG_PATH") {
        config
            .extra
            .insert("config_path".to_string(), serde_yaml::Value::String(path));
    }

    if let Ok(skills_dir) = std::env::var("HONE_SKILLS_DIR") {
        config.extra.insert(
            "skills_dir".to_string(),
            serde_yaml::Value::String(skills_dir),
        );
    }
}

pub fn ensure_runtime_dirs(config: &HoneConfig) {
    let _ = std::fs::create_dir_all(&config.storage.sessions_dir);
    let _ = std::fs::create_dir_all(&config.storage.portfolio_dir);
    let _ = std::fs::create_dir_all(&config.storage.cron_jobs_dir);
    let _ = std::fs::create_dir_all(&config.storage.reports_dir);
    let _ = std::fs::create_dir_all(&config.storage.x_drafts_dir);
    let _ = std::fs::create_dir_all(&config.storage.gen_images_dir);
    let _ = std::fs::create_dir_all(&config.storage.kb_dir);
    if let Some(parent) = PathBuf::from(&config.storage.llm_audit_db_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

pub fn web_index_path() -> PathBuf {
    web_dist_dir().join("index.html")
}
