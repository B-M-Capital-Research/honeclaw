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
    let config_path = std::env::var_os("HONE_CONFIG_PATH").map(PathBuf::from);
    let skills_dir = std::env::var_os("HONE_SKILLS_DIR").map(PathBuf::from);
    config.apply_runtime_overrides(None, skills_dir.as_deref(), config_path.as_deref());
}

pub fn ensure_runtime_dirs(config: &HoneConfig) {
    config.ensure_runtime_dirs();
}

pub fn web_index_path() -> PathBuf {
    web_dist_dir().join("index.html")
}
