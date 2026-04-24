//! 运行时目录解析 trait。
//!
//! 所有「应该把文件写到哪里」的判定都应走这里,不要在调用点自己拼
//! `./data/...` 或直接读环境变量——否则 packaged 模式（desktop / brew
//! 安装）下路径会偏。

use std::path::PathBuf;

use crate::core::HoneBotCore;

/// 从 `HoneConfig` 派生出的一组运行时路径。
///
/// 这些路径在启动阶段就确定,生命周期内不变;trait 方法返回 owned `PathBuf`
/// 是为了让调用方可以直接传给 `std::fs::*` 而不用担心借用冲突。
pub trait PathResolver: Send + Sync {
    /// 内置 / 系统 skill 的搜索根目录
    /// （`config.extra.skills_dir` 或默认 `./skills`)。
    fn configured_system_skills_dir(&self) -> PathBuf;

    /// 用户自定义 skill 根目录(`{data_dir}/custom_skills`)。
    fn configured_custom_skills_dir(&self) -> PathBuf;

    /// 顶层 data 目录。环境变量 `HONE_DATA_DIR` 优先,否则从
    /// `storage.sessions_dir` 的父目录推断。
    fn configured_data_dir(&self) -> PathBuf;

    /// Runtime 目录(heartbeat / locks / 临时状态根)。
    fn configured_runtime_dir(&self) -> PathBuf;

    /// Skill 启用 / 禁用 override 文件路径
    /// (`{runtime_dir}/skill_registry.json`)。
    fn configured_skill_registry_path(&self) -> PathBuf;
}

impl PathResolver for HoneBotCore {
    fn configured_system_skills_dir(&self) -> PathBuf {
        HoneBotCore::configured_system_skills_dir(self)
    }

    fn configured_custom_skills_dir(&self) -> PathBuf {
        HoneBotCore::configured_custom_skills_dir(self)
    }

    fn configured_data_dir(&self) -> PathBuf {
        HoneBotCore::configured_data_dir(self)
    }

    fn configured_runtime_dir(&self) -> PathBuf {
        HoneBotCore::configured_runtime_dir(self)
    }

    fn configured_skill_registry_path(&self) -> PathBuf {
        HoneBotCore::configured_skill_registry_path(self)
    }
}
