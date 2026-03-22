use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub openrouter: OpenRouterConfig,
    #[serde(default)]
    pub kimi: KimiConfig,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            openrouter: OpenRouterConfig::default(),
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
}

fn default_api_key_env() -> String {
    "OPENROUTER_API_KEY".to_string()
}
fn default_model() -> String {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    #[serde(default)]
    pub gemini_acp: GeminiAcpConfig,
    #[serde(default)]
    pub codex_acp: CodexAcpConfig,
    #[serde(default)]
    pub opencode: OpencodeAcpConfig,
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
    #[serde(default = "default_gemini_acp_startup_timeout")]
    pub startup_timeout_seconds: u64,
    #[serde(default = "default_gemini_acp_request_timeout")]
    pub request_timeout_seconds: u64,
}

impl Default for GeminiAcpConfig {
    fn default() -> Self {
        Self {
            command: default_gemini_acp_command(),
            args: default_gemini_acp_args(),
            model: String::new(),
            api_key_env: default_gemini_api_key_env(),
            startup_timeout_seconds: default_gemini_acp_startup_timeout(),
            request_timeout_seconds: default_gemini_acp_request_timeout(),
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
    #[serde(default = "default_codex_acp_startup_timeout")]
    pub startup_timeout_seconds: u64,
    #[serde(default = "default_codex_acp_request_timeout")]
    pub request_timeout_seconds: u64,
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
            startup_timeout_seconds: default_codex_acp_startup_timeout(),
            request_timeout_seconds: default_codex_acp_request_timeout(),
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
    /// OpenAI 协议渠道的 Base URL（兼容 OpenRouter 及任意 OpenAI-compatible 端点）
    #[serde(default = "default_opencode_api_base_url")]
    pub api_base_url: String,
    /// OpenAI 协议渠道的 API Key（写入 YAML，用户在设置页直接配置）
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_opencode_startup_timeout")]
    pub startup_timeout_seconds: u64,
    #[serde(default = "default_opencode_request_timeout")]
    pub request_timeout_seconds: u64,
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
            startup_timeout_seconds: default_opencode_startup_timeout(),
            request_timeout_seconds: default_opencode_request_timeout(),
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
}

fn default_max_iterations() -> u32 {
    10
}
fn default_agent_runner() -> String {
    "function_calling".to_string()
}

fn default_opencode_command() -> String {
    "opencode".to_string()
}
fn default_opencode_api_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
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

fn default_opencode_startup_timeout() -> u64 {
    15
}

fn default_opencode_request_timeout() -> u64 {
    300
}

fn default_gemini_acp_startup_timeout() -> u64 {
    15
}

fn default_gemini_acp_request_timeout() -> u64 {
    300
}

fn default_codex_acp_startup_timeout() -> u64 {
    15
}

fn default_codex_acp_request_timeout() -> u64 {
    300
}
