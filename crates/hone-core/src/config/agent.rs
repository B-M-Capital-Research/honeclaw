use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub openrouter: OpenRouterConfig,
    #[serde(default)]
    pub auxiliary: AuxiliaryLlmConfig,
    #[serde(default)]
    pub kimi: KimiConfig,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            openrouter: OpenRouterConfig::default(),
            auxiliary: AuxiliaryLlmConfig::default(),
            kimi: KimiConfig::default(),
        }
    }
}

fn default_provider() -> String {
    "openrouter".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    /// 单 Key 向后兼容字段（优先于 api_keys[0]）
    #[serde(default)]
    pub api_key: String,
    /// 多 Key 列表，支持多账号 fallback（与 api_key 合并后去重使用）
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_sub_model")]
    pub sub_model: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_keys: Vec::new(),
            api_key_env: default_api_key_env(),
            model: default_model(),
            sub_model: default_sub_model(),
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            max_tokens: default_max_tokens(),
        }
    }
}

impl OpenRouterConfig {
    /// 合并 `api_key` 和 `api_keys`，返回去重后的有效 Key 池
    pub fn effective_key_pool(&self) -> crate::api_key_pool::ApiKeyPool {
        crate::api_key_pool::ApiKeyPool::merged(&self.api_key, &self.api_keys)
    }

    pub fn auxiliary_model(&self) -> &str {
        let sub_model = self.sub_model.trim();
        if sub_model.is_empty() {
            self.model.trim()
        } else {
            sub_model
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxiliaryLlmConfig {
    #[serde(default = "default_auxiliary_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_auxiliary_api_key_env")]
    pub api_key_env: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

impl Default for AuxiliaryLlmConfig {
    fn default() -> Self {
        Self {
            base_url: default_auxiliary_base_url(),
            api_key: String::new(),
            api_key_env: default_auxiliary_api_key_env(),
            model: String::new(),
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            max_tokens: default_max_tokens(),
        }
    }
}

impl AuxiliaryLlmConfig {
    pub fn is_configured(&self) -> bool {
        !self.base_url.trim().is_empty()
            && !self.model.trim().is_empty()
            && (!self.api_key.trim().is_empty() || !self.api_key_env.trim().is_empty())
    }

    pub fn resolved_api_key(&self) -> String {
        let direct = self.api_key.trim();
        if !direct.is_empty() {
            return direct.to_string();
        }

        let env_name = self.api_key_env.trim();
        if env_name.is_empty() {
            return String::new();
        }

        std::env::var(env_name)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

fn default_api_key_env() -> String {
    "OPENROUTER_API_KEY".to_string()
}
fn default_auxiliary_base_url() -> String {
    String::new()
}
fn default_auxiliary_api_key_env() -> String {
    "MINIMAX_API_KEY".to_string()
}
fn default_model() -> String {
    "moonshotai/kimi-k2.5".to_string()
}
fn default_sub_model() -> String {
    "moonshotai/kimi-k2.5".to_string()
}
fn default_timeout() -> u64 {
    120
}
fn default_max_retries() -> u32 {
    3
}
fn default_max_tokens() -> u32 {
    32768
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KimiConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub system_prompt_path: String,
    #[serde(default = "default_agent_runner", alias = "provider")]
    pub runner: String,
    #[serde(default)]
    pub codex_model: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_daily_conversation_limit")]
    pub daily_conversation_limit: u32,
    #[serde(default = "default_agent_step_timeout_seconds")]
    pub step_timeout_seconds: u64,
    #[serde(default = "default_agent_overall_timeout_seconds")]
    pub overall_timeout_seconds: u64,
    #[serde(default)]
    pub gemini_acp: GeminiAcpConfig,
    #[serde(default)]
    pub codex_acp: CodexAcpConfig,
    #[serde(default)]
    pub opencode: OpencodeAcpConfig,
    #[serde(default)]
    pub hone_cloud: HoneCloudConfig,
    #[serde(default)]
    pub multi_agent: MultiAgentConfig,
}

impl AgentConfig {
    pub fn runner_kind(&self) -> AgentRunnerKind {
        AgentRunnerKind::from_config_value(&self.runner)
    }

    pub fn step_timeout(&self) -> Duration {
        Duration::from_secs(self.step_timeout_seconds.max(1))
    }

    pub fn overall_timeout(&self) -> Duration {
        Duration::from_secs(
            self.overall_timeout_seconds
                .max(self.step_timeout_seconds.max(1)),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunnerKind {
    FunctionCalling,
    GeminiCli,
    GeminiAcp,
    CodexCli,
    CodexAcp,
    OpencodeAcp,
    HoneCloud,
    MultiAgent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentRunnerProbe {
    pub binary: &'static str,
    pub arg: &'static str,
}

impl AgentRunnerKind {
    pub fn from_config_value(value: &str) -> Self {
        match value.trim() {
            "function_calling" => Self::FunctionCalling,
            "gemini_cli" => Self::GeminiCli,
            "gemini_acp" => Self::GeminiAcp,
            "codex_cli" => Self::CodexCli,
            "codex_acp" => Self::CodexAcp,
            "opencode_acp" => Self::OpencodeAcp,
            "hone_cloud" => Self::HoneCloud,
            "multi-agent" => Self::MultiAgent,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FunctionCalling => "function_calling",
            Self::GeminiCli => "gemini_cli",
            Self::GeminiAcp => "gemini_acp",
            Self::CodexCli => "codex_cli",
            Self::CodexAcp => "codex_acp",
            Self::OpencodeAcp => "opencode_acp",
            Self::HoneCloud => "hone_cloud",
            Self::MultiAgent => "multi-agent",
            Self::Unknown => "unknown",
        }
    }

    pub fn manages_own_context(self) -> bool {
        matches!(self, Self::CodexAcp | Self::OpencodeAcp)
    }

    pub fn cli_probe(self) -> Option<AgentRunnerProbe> {
        match self {
            Self::GeminiCli | Self::GeminiAcp => Some(AgentRunnerProbe {
                binary: "gemini",
                arg: "--version",
            }),
            Self::CodexCli => Some(AgentRunnerProbe {
                binary: "codex",
                arg: "--version",
            }),
            Self::CodexAcp => Some(AgentRunnerProbe {
                binary: "codex-acp",
                arg: "--help",
            }),
            Self::OpencodeAcp | Self::MultiAgent => Some(AgentRunnerProbe {
                binary: "opencode",
                arg: "--version",
            }),
            Self::HoneCloud => None,
            Self::FunctionCalling | Self::Unknown => None,
        }
    }
}

impl Serialize for AgentRunnerKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AgentRunnerKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_config_value(&value))
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            system_prompt_path: String::new(),
            runner: default_agent_runner(),
            codex_model: String::new(),
            max_iterations: default_max_iterations(),
            daily_conversation_limit: default_daily_conversation_limit(),
            step_timeout_seconds: default_agent_step_timeout_seconds(),
            overall_timeout_seconds: default_agent_overall_timeout_seconds(),
            gemini_acp: GeminiAcpConfig::default(),
            codex_acp: CodexAcpConfig::default(),
            opencode: OpencodeAcpConfig::default(),
            hone_cloud: HoneCloudConfig::default(),
            multi_agent: MultiAgentConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneCloudConfig {
    #[serde(default = "default_hone_cloud_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_hone_cloud_model")]
    pub model: String,
}

impl Default for HoneCloudConfig {
    fn default() -> Self {
        Self {
            base_url: default_hone_cloud_base_url(),
            api_key: String::new(),
            model: default_hone_cloud_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    #[serde(default)]
    pub search: MultiAgentSearchConfig,
    #[serde(default)]
    pub answer: MultiAgentAnswerConfig,
}

impl Default for MultiAgentConfig {
    fn default() -> Self {
        Self {
            search: MultiAgentSearchConfig::default(),
            answer: MultiAgentAnswerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentSearchConfig {
    #[serde(default = "default_multi_agent_search_base_url")]
    pub base_url: String,
    #[serde(default = "default_multi_agent_search_api_key")]
    pub api_key: String,
    #[serde(default = "default_multi_agent_search_model")]
    pub model: String,
    #[serde(default = "default_multi_agent_search_max_iterations")]
    pub max_iterations: u32,
}

impl Default for MultiAgentSearchConfig {
    fn default() -> Self {
        Self {
            base_url: default_multi_agent_search_base_url(),
            api_key: default_multi_agent_search_api_key(),
            model: default_multi_agent_search_model(),
            max_iterations: default_multi_agent_search_max_iterations(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentAnswerConfig {
    #[serde(default = "default_multi_agent_answer_api_base_url")]
    pub api_base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub variant: String,
    #[serde(default = "default_multi_agent_answer_max_tool_calls")]
    pub max_tool_calls: u32,
}

impl Default for MultiAgentAnswerConfig {
    fn default() -> Self {
        Self {
            api_base_url: default_multi_agent_answer_api_base_url(),
            api_key: String::new(),
            model: String::new(),
            variant: String::new(),
            max_tool_calls: default_multi_agent_answer_max_tool_calls(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiAcpConfig {
    #[serde(default = "default_gemini_acp_command")]
    pub command: String,
    #[serde(default = "default_gemini_acp_args")]
    pub args: Vec<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_gemini_api_key_env")]
    pub api_key_env: String,
}

impl Default for GeminiAcpConfig {
    fn default() -> Self {
        Self {
            command: default_gemini_acp_command(),
            args: default_gemini_acp_args(),
            model: String::new(),
            api_key_env: default_gemini_api_key_env(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAcpConfig {
    #[serde(default = "default_codex_acp_command")]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_codex_command")]
    pub codex_command: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub sandbox_mode: String,
    #[serde(default)]
    pub approval_policy: String,
    #[serde(default)]
    pub dangerously_bypass_approvals_and_sandbox: bool,
    #[serde(default)]
    pub sandbox_permissions: Vec<String>,
    #[serde(default)]
    pub extra_config_overrides: Vec<String>,
}

impl Default for CodexAcpConfig {
    fn default() -> Self {
        Self {
            command: default_codex_acp_command(),
            args: Vec::new(),
            codex_command: default_codex_command(),
            model: String::new(),
            variant: String::new(),
            sandbox_mode: String::new(),
            approval_policy: String::new(),
            dangerously_bypass_approvals_and_sandbox: false,
            sandbox_permissions: Vec::new(),
            extra_config_overrides: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeAcpConfig {
    #[serde(default = "default_opencode_command")]
    pub command: String,
    #[serde(default = "default_opencode_args")]
    pub args: Vec<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub variant: String,
    /// 可选的 Hone 侧 provider/base URL 覆盖；留空则继承用户本机 opencode 配置
    #[serde(default = "default_opencode_api_base_url")]
    pub api_base_url: String,
    /// 可选的 Hone 侧 API key 覆盖；留空则继承用户本机 opencode 登录态 / provider 配置
    #[serde(default)]
    pub api_key: String,
    /// OpenRouter API Key（运行时注入，来自 llm.openrouter.api_key 配置，不写入 YAML）
    #[serde(skip)]
    pub openrouter_api_key: Option<String>,
}

impl Default for OpencodeAcpConfig {
    fn default() -> Self {
        Self {
            command: default_opencode_command(),
            args: default_opencode_args(),
            model: String::new(),
            variant: String::new(),
            api_base_url: default_opencode_api_base_url(),
            api_key: String::new(),
            openrouter_api_key: None,
        }
    }
}

/// 管理员配置 — 按渠道配置管理员身份列表
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminConfig {
    /// iMessage 管理员 handle 列表（手机号或 Apple ID，如 "+13234567890"）
    #[serde(default)]
    pub imessage_handles: Vec<String>,
    /// Telegram 管理员 user ID 列表（数字字符串，如 "8039067465"）
    #[serde(default)]
    pub telegram_user_ids: Vec<String>,
    /// Feishu 管理员邮箱列表
    #[serde(default)]
    pub feishu_emails: Vec<String>,
    /// Feishu 管理员手机号列表
    #[serde(default)]
    pub feishu_mobiles: Vec<String>,
    /// Feishu 管理员 open_id 列表
    #[serde(default)]
    pub feishu_open_ids: Vec<String>,
    /// Discord 管理员用户 ID 列表（数字字符串，如 "123456789012345678"）
    #[serde(default)]
    pub discord_user_ids: Vec<String>,
    /// 运行时管理员注册口令；建议留空并改用环境变量
    #[serde(default)]
    pub runtime_admin_registration_passphrase: String,
    /// 运行时管理员注册口令环境变量名
    #[serde(default = "default_runtime_admin_registration_passphrase_env")]
    pub runtime_admin_registration_passphrase_env: String,
}

impl AdminConfig {
    pub fn resolved_runtime_admin_registration_passphrase(&self) -> String {
        let direct = self.runtime_admin_registration_passphrase.trim();
        if !direct.is_empty() {
            return direct.to_string();
        }

        let env_name = self.runtime_admin_registration_passphrase_env.trim();
        if env_name.is_empty() {
            return String::new();
        }

        std::env::var(env_name)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

fn default_runtime_admin_registration_passphrase_env() -> String {
    "HONE_ADMIN_REGISTER_PASSPHRASE".to_string()
}

fn default_max_iterations() -> u32 {
    10
}

fn default_daily_conversation_limit() -> u32 {
    12
}

fn default_agent_step_timeout_seconds() -> u64 {
    180
}

fn default_agent_overall_timeout_seconds() -> u64 {
    1200
}

fn default_agent_runner() -> String {
    "function_calling".to_string()
}

fn default_hone_cloud_base_url() -> String {
    "https://hone-claw.com".to_string()
}

fn default_hone_cloud_model() -> String {
    "hone-cloud".to_string()
}

fn default_multi_agent_search_base_url() -> String {
    "https://api.minimaxi.com/v1".to_string()
}

fn default_multi_agent_search_api_key() -> String {
    String::new()
}

fn default_multi_agent_search_model() -> String {
    "MiniMax-M2.7-highspeed".to_string()
}

fn default_multi_agent_search_max_iterations() -> u32 {
    8
}

fn default_multi_agent_answer_max_tool_calls() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::{AgentRunnerKind, MultiAgentAnswerConfig};

    #[test]
    fn multi_agent_answer_default_tool_limit_is_three() {
        assert_eq!(MultiAgentAnswerConfig::default().max_tool_calls, 3);
    }

    #[test]
    fn agent_default_daily_conversation_limit_is_twelve() {
        assert_eq!(super::AgentConfig::default().daily_conversation_limit, 12);
    }

    #[test]
    fn agent_runner_kind_keeps_wire_values_and_probe_mapping() {
        let kind = AgentRunnerKind::from_config_value("codex_acp");
        assert_eq!(kind.as_str(), "codex_acp");
        assert!(kind.manages_own_context());
        let probe = kind.cli_probe().expect("codex acp probe");
        assert_eq!(probe.binary, "codex-acp");
        assert_eq!(probe.arg, "--help");
        let cloud = AgentRunnerKind::from_config_value("hone_cloud");
        assert_eq!(cloud.as_str(), "hone_cloud");
        assert!(cloud.cli_probe().is_none());
        assert_eq!(
            serde_yaml::to_string(&AgentRunnerKind::MultiAgent)
                .expect("serialize")
                .trim(),
            "multi-agent"
        );
    }
}

fn default_multi_agent_answer_api_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

fn default_opencode_command() -> String {
    "opencode".to_string()
}
fn default_opencode_api_base_url() -> String {
    String::new()
}

fn default_gemini_acp_command() -> String {
    "gemini".to_string()
}

fn default_gemini_acp_args() -> Vec<String> {
    vec![
        "--experimental-acp".to_string(),
        "--sandbox".to_string(),
        "--approval-mode".to_string(),
        "plan".to_string(),
    ]
}

fn default_gemini_api_key_env() -> String {
    "GEMINI_API_KEY".to_string()
}

fn default_codex_acp_command() -> String {
    "codex-acp".to_string()
}

fn default_codex_command() -> String {
    "codex".to_string()
}

fn default_opencode_args() -> Vec<String> {
    vec!["acp".to_string()]
}
