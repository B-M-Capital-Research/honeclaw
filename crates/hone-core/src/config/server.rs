use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    #[serde(default)]
    pub auth_token: String,
    #[serde(default = "default_research_api_base")]
    pub research_api_base: String,
    #[serde(default)]
    pub research_api_key: String,
    #[serde(default = "default_local_workflow_api_base")]
    pub local_workflow_api_base: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            auth_token: String::new(),
            research_api_base: default_research_api_base(),
            research_api_key: String::new(),
            local_workflow_api_base: default_local_workflow_api_base(),
        }
    }
}

fn default_research_api_base() -> String {
    "https://research.example.com".to_string()
}

fn default_local_workflow_api_base() -> String {
    "http://127.0.0.1:3213".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NanoBananaConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_nb_base_url")]
    pub base_url: String,
    #[serde(default = "default_nb_model")]
    pub model: String,
    #[serde(default = "default_image_count")]
    pub default_image_count: u32,
    #[serde(default)]
    pub download_dir: String,
}

fn default_true() -> bool {
    true
}
fn default_image_count() -> u32 {
    3
}

fn default_nb_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}
fn default_nb_model() -> String {
    "google/gemini-2.0-flash-exp".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FmpConfig {
    /// 单 Key 向后兼容字段
    #[serde(default)]
    pub api_key: String,
    /// 多 Key 列表，支持多账号 fallback（与 api_key 合并后去重使用）
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default = "default_fmp_base")]
    pub base_url: String,
    #[serde(default = "default_fmp_timeout")]
    pub timeout: u64,
}

impl FmpConfig {
    /// 合并 `api_key` 和 `api_keys`，返回去重后的有效 Key 池
    pub fn effective_key_pool(&self) -> crate::api_key_pool::ApiKeyPool {
        crate::api_key_pool::ApiKeyPool::merged(&self.api_key, &self.api_keys)
    }
}

fn default_fmp_base() -> String {
    "https://financialmodelingprep.com/api".to_string()
}
fn default_fmp_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchConfig {
    #[serde(default = "default_tavily")]
    pub provider: String,
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default = "default_search_depth")]
    pub search_depth: String,
    #[serde(default = "default_search_topic")]
    pub topic: String,
    #[serde(default = "default_search_max_results")]
    pub max_results: u32,
}

fn default_tavily() -> String {
    "tavily".to_string()
}
fn default_search_depth() -> String {
    "basic".to_string()
}
fn default_search_topic() -> String {
    "general".to_string()
}
fn default_search_max_results() -> u32 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default = "default_true")]
    pub console: bool,
    #[serde(default)]
    pub udp_port: Option<u16>,
}

fn default_log_level() -> String {
    "INFO".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub tool_guard: ToolGuardConfig,
    #[serde(default = "default_true")]
    pub kb_actor_isolation: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tool_guard: ToolGuardConfig::default(),
            kb_actor_isolation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolGuardConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_tool_guard_mode")]
    pub mode: String,
    #[serde(default = "default_tool_guard_apply_tools")]
    pub apply_tools: Vec<String>,
    #[serde(default = "default_tool_guard_patterns")]
    pub deny_patterns: Vec<String>,
}

impl Default for ToolGuardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: default_tool_guard_mode(),
            apply_tools: default_tool_guard_apply_tools(),
            deny_patterns: default_tool_guard_patterns(),
        }
    }
}

fn default_tool_guard_mode() -> String {
    "block".to_string()
}

fn default_tool_guard_apply_tools() -> Vec<String> {
    vec![
        "*".to_string(),
        "!web_search".to_string(),
        "!data_fetch".to_string(),
        "!kb_search".to_string(),
    ]
}

fn default_tool_guard_patterns() -> Vec<String> {
    vec![
        "rm -rf".to_string(),
        "rm -fr".to_string(),
        "rm -r /".to_string(),
        "rm -rf /".to_string(),
        "rm -fr /".to_string(),
        "mkfs".to_string(),
        "dd if=/dev/zero".to_string(),
        "dd if=/dev/random".to_string(),
        "shutdown -h".to_string(),
        "shutdown -r".to_string(),
        "reboot".to_string(),
        "poweroff".to_string(),
        "halt".to_string(),
        ":(){ :|:& };:".to_string(),
        "del /s /q".to_string(),
        "rd /s /q".to_string(),
        "format c:".to_string(),
        "curl | sh".to_string(),
        "wget | sh".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageConfig {
    #[serde(default = "default_sessions_dir")]
    pub sessions_dir: String,
    #[serde(default = "default_session_sqlite_db_path")]
    pub session_sqlite_db_path: String,
    #[serde(default)]
    pub session_sqlite_shadow_write_enabled: bool,
    #[serde(default = "default_session_runtime_backend")]
    pub session_runtime_backend: String,
    #[serde(
        default = "default_conversation_quota_dir",
        alias = "conversation_quota_db_path"
    )]
    pub conversation_quota_dir: String,
    #[serde(default = "default_llm_audit_db_path")]
    pub llm_audit_db_path: String,
    #[serde(default = "default_llm_audit_retention_days")]
    pub llm_audit_retention_days: u32,
    #[serde(default = "default_true")]
    pub llm_audit_enabled: bool,
    #[serde(default = "default_portfolio_dir")]
    pub portfolio_dir: String,
    #[serde(default = "default_cron_jobs_dir")]
    pub cron_jobs_dir: String,
    #[serde(default = "default_reports_dir")]
    pub reports_dir: String,
    #[serde(default = "default_x_drafts_dir")]
    pub x_drafts_dir: String,
    #[serde(default = "default_gen_images_dir")]
    pub gen_images_dir: String,
    #[serde(default = "default_kb_dir")]
    pub kb_dir: String,
}

impl StorageConfig {
    pub fn apply_data_root(&mut self, root: impl AsRef<Path>) {
        let root = root.as_ref();
        self.sessions_dir = root.join("sessions").to_string_lossy().to_string();
        self.session_sqlite_db_path = root.join("sessions.sqlite3").to_string_lossy().to_string();
        self.conversation_quota_dir = root
            .join("conversation_quota")
            .to_string_lossy()
            .to_string();
        self.llm_audit_db_path = root.join("llm_audit.sqlite3").to_string_lossy().to_string();
        self.portfolio_dir = root.join("portfolio").to_string_lossy().to_string();
        self.cron_jobs_dir = root.join("cron_jobs").to_string_lossy().to_string();
        self.reports_dir = root.join("reports").to_string_lossy().to_string();
        self.x_drafts_dir = root.join("x_drafts").to_string_lossy().to_string();
        self.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();
        self.kb_dir = root.join("kb").to_string_lossy().to_string();
    }

    pub fn ensure_runtime_dirs(&self) {
        let _ = std::fs::create_dir_all(&self.sessions_dir);
        let _ = std::fs::create_dir_all(&self.portfolio_dir);
        let _ = std::fs::create_dir_all(&self.cron_jobs_dir);
        let _ = std::fs::create_dir_all(&self.reports_dir);
        let _ = std::fs::create_dir_all(&self.x_drafts_dir);
        let _ = std::fs::create_dir_all(&self.gen_images_dir);
        let _ = std::fs::create_dir_all(&self.kb_dir);
        let _ = std::fs::create_dir_all(&self.conversation_quota_dir);
        if let Some(parent) = PathBuf::from(&self.llm_audit_db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Some(parent) = PathBuf::from(&self.session_sqlite_db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
}

fn default_sessions_dir() -> String {
    "./data/sessions".to_string()
}
fn default_session_sqlite_db_path() -> String {
    "./data/sessions.sqlite3".to_string()
}
fn default_session_runtime_backend() -> String {
    "json".to_string()
}
fn default_conversation_quota_dir() -> String {
    "./data/conversation_quota".to_string()
}
fn default_llm_audit_db_path() -> String {
    "./data/llm_audit.sqlite3".to_string()
}
fn default_llm_audit_retention_days() -> u32 {
    30
}
fn default_portfolio_dir() -> String {
    "./data/portfolio".to_string()
}
fn default_cron_jobs_dir() -> String {
    "./data/cron_jobs".to_string()
}
fn default_reports_dir() -> String {
    "./data/reports".to_string()
}
fn default_x_drafts_dir() -> String {
    "./data/x_drafts".to_string()
}
fn default_gen_images_dir() -> String {
    "./data/gen_images".to_string()
}
fn default_kb_dir() -> String {
    "./data/kb".to_string()
}
