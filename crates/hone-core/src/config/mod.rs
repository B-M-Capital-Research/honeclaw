//! 配置加载与验证
//!
//! 从 config.yaml 加载配置，使用 serde 反序列化。
//!
//! 子模块布局：
//! - [`agent`] / [`channels`] / [`event_engine`] / [`server`] —— 领域子配置类型
//! - [`yaml`] —— YAML 读写 / overlay / path 解析等底层工具
//! - [`materialize`] —— canonical ↔ effective ↔ legacy 配置流转
//! - [`mutation`] —— 路径级 set/unset + 影响面分类 + 脱敏

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

pub mod agent;
pub mod channels;
pub mod event_engine;
pub mod materialize;
pub mod mutation;
pub mod server;
pub mod yaml;

pub use agent::{
    AdminConfig, AgentConfig, AgentRunnerKind, AgentRunnerProbe, AuxiliaryLlmConfig,
    CodexAcpConfig, GeminiAcpConfig, KimiConfig, LlmConfig, MultiAgentAnswerConfig,
    MultiAgentConfig, MultiAgentSearchConfig, OpenRouterConfig, OpencodeAcpConfig,
};
pub use channels::{
    ChatScope, DiscordConfig, DiscordGroupReplyConfig, DiscordWatchConfig, FeishuConfig,
    GroupContextConfig, IMessageConfig, TelegramConfig,
};
pub use event_engine::{
    DigestConfig as EventEngineDigestConfig, EventEngineConfig,
    PollIntervals as EventEnginePollIntervals, RendererConfig as EventEngineRendererConfig,
    Sources as EventEngineSources, TelegramChannelConfig, Thresholds as EventEngineThresholds,
    TruthSocialAccountConfig, tz_offset_hours,
};
pub use materialize::{
    canonical_config_candidate, effective_config_path, generate_effective_config,
    promote_legacy_runtime_agent_settings, seed_canonical_config_from_source,
};
pub use mutation::{
    ConfigApplyPlan, ConfigMutation, ConfigMutationResult, apply_config_mutations,
    classify_config_paths, is_sensitive_config_path, read_config_path_value,
    redact_sensitive_value,
};
pub use server::{
    FmpConfig, LoggingConfig, NanoBananaConfig, SearchConfig, SecurityConfig, StorageConfig,
    ToolGuardConfig, WebConfig,
};
pub use yaml::{
    diff_yaml_value, merge_yaml_value, read_merged_yaml_value, read_yaml_value,
    runtime_overlay_path, write_overlay_patch,
};

/// 顶层配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneConfig {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub imessage: IMessageConfig,
    #[serde(default)]
    pub feishu: FeishuConfig,
    #[serde(default)]
    pub telegram: TelegramConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub group_context: GroupContextConfig,
    #[serde(default)]
    pub nano_banana: NanoBananaConfig,
    #[serde(default)]
    pub fmp: FmpConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    /// Agent system prompt 模板
    #[serde(default)]
    pub agent: AgentConfig,
    /// 管理员配置
    #[serde(default)]
    pub admins: AdminConfig,
    /// Web 控制台配置
    #[serde(default)]
    pub web: WebConfig,
    /// 安全策略配置
    #[serde(default)]
    pub security: SecurityConfig,
    /// 主动事件引擎配置
    #[serde(default)]
    pub event_engine: EventEngineConfig,
    /// 额外的未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl HoneConfig {
    /// 从已经完成 overlay 合并的 YAML 值加载配置。
    ///
    /// 与 `from_file()` 不同，这里不会尝试解析并内联 `system_prompt_path`
    /// 指向的文件内容，适合配置编辑流程里的“纯配置校验”。
    pub fn from_merged_value(value: Value) -> crate::HoneResult<Self> {
        let config: Self = serde_yaml::from_value(value)
            .map_err(|e| crate::HoneError::Config(format!("配置文件解析失败: {e}")))?;
        let config = config;
        config.validate()?;
        Ok(config)
    }

    /// 从 YAML 文件加载配置
    pub fn from_file(path: impl AsRef<Path>) -> crate::HoneResult<Self> {
        let path = path.as_ref();
        let value = read_merged_yaml_value(path)?;
        let mut config = Self::from_merged_value(value)?;
        if let Err(err) = materialize::apply_system_prompt_path(&mut config, path) {
            return Err(crate::HoneError::Config(err));
        }
        Ok(config)
    }

    pub fn validate(&self) -> crate::HoneResult<()> {
        mutation::validate_channel_chat_scope("feishu", self.feishu.chat_scope)?;
        mutation::validate_channel_chat_scope("telegram", self.telegram.chat_scope)?;
        mutation::validate_channel_chat_scope("discord", self.discord.chat_scope)?;
        Ok(())
    }

    pub fn apply_runtime_overrides(
        &mut self,
        data_dir: Option<&Path>,
        skills_dir: Option<&Path>,
        config_path: Option<&Path>,
    ) {
        if let Some(data_dir) = data_dir {
            self.storage.apply_data_root(data_dir);
        }
        if let Some(skills_dir) = skills_dir {
            self.extra.insert(
                "skills_dir".to_string(),
                serde_yaml::Value::String(skills_dir.to_string_lossy().to_string()),
            );
        }
        if let Some(config_path) = config_path {
            self.extra.insert(
                "config_path".to_string(),
                serde_yaml::Value::String(config_path.to_string_lossy().to_string()),
            );
        }
    }

    pub fn ensure_runtime_dirs(&self) {
        self.storage.ensure_runtime_dirs();
    }
}

impl Default for HoneConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            imessage: IMessageConfig::default(),
            feishu: FeishuConfig::default(),
            telegram: TelegramConfig::default(),
            discord: DiscordConfig::default(),
            group_context: GroupContextConfig::default(),
            nano_banana: NanoBananaConfig::default(),
            fmp: FmpConfig::default(),
            search: SearchConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            agent: AgentConfig::default(),
            admins: AdminConfig::default(),
            web: WebConfig::default(),
            security: SecurityConfig::default(),
            event_engine: EventEngineConfig::default(),
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests;
