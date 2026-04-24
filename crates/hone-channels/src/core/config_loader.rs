//! 进程启动时从磁盘加载 `HoneConfig` 的入口。
//!
//! 被所有 binary (`hone-cli` / `hone-feishu` / `hone-telegram` / …) 共用,
//! 解析优先级:
//! 1. `--config <path>` / 环境变量 `HONE_CONFIG_PATH`(走 `runtime_config_path`)
//! 2. 退化到 cwd 下的 `config.yaml`
//!
//! 加载后会应用两条 runtime override:`HONE_DATA_DIR` 和 `HONE_SKILLS_DIR`,
//! 方便 packaged install / 测试环境指向自定义路径而不是改 yaml。

use std::path::PathBuf;

use hone_core::config::HoneConfig;

pub fn runtime_config_path() -> String {
    std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string())
}

pub fn load_runtime_config() -> hone_core::HoneResult<(HoneConfig, String)> {
    let config_path = runtime_config_path();
    let mut config = HoneConfig::from_file(&config_path)?;
    let data_dir = std::env::var_os("HONE_DATA_DIR").map(PathBuf::from);
    let skills_dir = std::env::var_os("HONE_SKILLS_DIR").map(PathBuf::from);
    config.apply_runtime_overrides(
        data_dir.as_deref(),
        skills_dir.as_deref(),
        Some(PathBuf::from(&config_path).as_path()),
    );
    config.ensure_runtime_dirs();
    Ok((config, config_path))
}
