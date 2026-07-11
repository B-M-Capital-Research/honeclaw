//! 进程启动时从磁盘加载 `HoneConfig` 的入口。
//!
//! 被通道/后端 runtime 进程 (`hone-console-page` / `hone-feishu` /
//! `hone-telegram` / …) 共用。CLI/Desktop 设置面负责读写 canonical
//! `config.yaml`; 启动时再生成 `data/runtime/effective-config.yaml` 并通过
//! `HONE_CONFIG_PATH` 注入这些子进程。
//!
//! 解析优先级:
//! 1. 环境变量 `HONE_CONFIG_PATH`(通常是 generated effective config)
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
    hone_core::harden_private_file(&config_path).map_err(|err| {
        hone_core::HoneError::Config(format!("failed to protect runtime config: {err}"))
    })?;
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
