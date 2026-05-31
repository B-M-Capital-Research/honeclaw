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
    #[serde(default)]
    pub local_workflow_validate_code: String,
    #[serde(default = "default_local_workflow_validate_code_env")]
    pub local_workflow_validate_code_env: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            auth_token: String::new(),
            research_api_base: default_research_api_base(),
            research_api_key: String::new(),
            local_workflow_api_base: default_local_workflow_api_base(),
            local_workflow_validate_code: String::new(),
            local_workflow_validate_code_env: default_local_workflow_validate_code_env(),
        }
    }
}

impl WebConfig {
    pub fn resolved_local_workflow_validate_code(&self) -> String {
        let direct = self.local_workflow_validate_code.trim();
        if !direct.is_empty() {
            return direct.to_string();
        }

        let env_name = self.local_workflow_validate_code_env.trim();
        if env_name.is_empty() {
            return String::new();
        }

        std::env::var(env_name)
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

fn default_research_api_base() -> String {
    "https://research.example.com".to_string()
}

fn default_local_workflow_api_base() -> String {
    "http://127.0.0.1:3213".to_string()
}

fn default_local_workflow_validate_code_env() -> String {
    "HONE_REPORT_VALIDATE_CODE".to_string()
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
    60
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
    #[serde(default = "default_true")]
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
    #[serde(default = "default_gen_images_dir")]
    pub gen_images_dir: String,
    /// 通知偏好(per-actor NotificationPrefs JSON)的目录。同时给:
    /// * event-engine router / digest scheduler 读取
    /// * HTTP API / 管理端设置页 读写
    /// * NotificationPrefsTool (终端用户自然语言) 读写
    ///   三者必须是同一份文件,否则改完不生效。
    #[serde(default = "default_notif_prefs_dir")]
    pub notif_prefs_dir: String,
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
        self.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();
        self.notif_prefs_dir = root.join("notif_prefs").to_string_lossy().to_string();
    }

    pub fn ensure_runtime_dirs(&self) {
        let _ = std::fs::create_dir_all(&self.sessions_dir);
        let _ = std::fs::create_dir_all(&self.portfolio_dir);
        let _ = std::fs::create_dir_all(&self.cron_jobs_dir);
        let _ = std::fs::create_dir_all(&self.gen_images_dir);
        let _ = std::fs::create_dir_all(&self.notif_prefs_dir);
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
fn default_gen_images_dir() -> String {
    "./data/gen_images".to_string()
}
fn default_notif_prefs_dir() -> String {
    "./data/notif_prefs".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudConfig {
    #[serde(default = "default_cloud_mode")]
    pub mode: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub strict_no_local_storage: bool,
    #[serde(default)]
    pub postgres: PostgresConfig,
    #[serde(default)]
    pub oss: OssConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudMode {
    Local,
    Cloud,
    Auto,
}

impl CloudMode {
    pub fn from_config_value(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cloud" => Self::Cloud,
            "auto" => Self::Auto,
            _ => Self::Local,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloud => "cloud",
            Self::Auto => "auto",
        }
    }

    pub fn is_cloud_authoritative(&self) -> bool {
        matches!(self, Self::Cloud)
    }
}

impl CloudConfig {
    pub fn effective_mode(&self) -> CloudMode {
        let env_mode = env_value("HONE_CLOUD_MODE");
        if !env_mode.is_empty() {
            return CloudMode::from_config_value(&env_mode);
        }
        CloudMode::from_config_value(&self.mode)
    }

    pub fn effective_enabled(&self) -> bool {
        match self.effective_mode() {
            CloudMode::Local => false,
            CloudMode::Cloud => true,
            CloudMode::Auto => {
                self.enabled
                    || env_bool("HONE_CLOUD_ENABLED")
                    || self.postgres.is_configured()
                    || self.oss.is_configured()
            }
        }
    }

    pub fn effective_strict_no_local_storage(&self) -> bool {
        self.strict_no_local_storage || env_bool("HONE_CLOUD_STRICT_NO_LOCAL_STORAGE")
    }

    pub fn validate(&self) -> crate::HoneResult<()> {
        if matches!(self.effective_mode(), CloudMode::Cloud)
            && !(self.postgres.is_configured() && self.oss.is_configured())
        {
            return Err(crate::HoneError::Config(
                "cloud.mode=cloud 需要同时配置 cloud.postgres 和 cloud.oss".to_string(),
            ));
        }
        if self.postgres.is_partially_configured() && !self.postgres.is_configured() {
            return Err(crate::HoneError::Config(
                "cloud.postgres 配置不完整：需要 database_url 或 host/user/password/database"
                    .to_string(),
            ));
        }
        if self.oss.is_partially_configured() && !self.oss.is_configured() {
            return Err(crate::HoneError::Config(
                "cloud.oss 配置不完整：需要 access_key_id/access_key_secret/bucket/endpoint"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    #[serde(default)]
    pub database_url: String,
    #[serde(default = "default_database_url_env")]
    pub database_url_env: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_pg_host_env")]
    pub host_env: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default = "default_pg_port_env")]
    pub port_env: String,
    #[serde(default)]
    pub user: String,
    #[serde(default = "default_pg_user_env")]
    pub user_env: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pg_password_env")]
    pub password_env: String,
    #[serde(default)]
    pub database: String,
    #[serde(default = "default_pg_database_env")]
    pub database_env: String,
    #[serde(default = "default_pg_sslmode")]
    pub sslmode: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default = "default_pg_proxy_env")]
    pub proxy_env: String,
    #[serde(default)]
    pub no_proxy: bool,
    #[serde(default = "default_pg_no_proxy_env")]
    pub no_proxy_env: String,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            database_url_env: default_database_url_env(),
            host: String::new(),
            host_env: default_pg_host_env(),
            port: 0,
            port_env: default_pg_port_env(),
            user: String::new(),
            user_env: default_pg_user_env(),
            password: String::new(),
            password_env: default_pg_password_env(),
            database: String::new(),
            database_env: default_pg_database_env(),
            sslmode: default_pg_sslmode(),
            proxy: String::new(),
            proxy_env: default_pg_proxy_env(),
            no_proxy: false,
            no_proxy_env: default_pg_no_proxy_env(),
        }
    }
}

impl PostgresConfig {
    pub fn resolved_database_url(&self) -> String {
        let direct = self.database_url.trim();
        if !direct.is_empty() {
            return direct.to_string();
        }

        let env_url = env_value(&self.database_url_env);
        if !env_url.is_empty() {
            return env_url;
        }

        let host = self.resolved_host();
        let user = self.resolved_user();
        let password = self.resolved_password();
        let database = self.resolved_database();
        if host.is_empty() || user.is_empty() || password.is_empty() || database.is_empty() {
            return String::new();
        }
        let port = self.resolved_port().unwrap_or(5432);
        let sslmode = self.sslmode.trim();
        let suffix = if sslmode.is_empty() {
            String::new()
        } else {
            format!("?sslmode={sslmode}")
        };
        format!("postgres://{user}:{password}@{host}:{port}/{database}{suffix}")
    }

    pub fn resolved_host(&self) -> String {
        direct_or_env(&self.host, &self.host_env)
    }

    pub fn resolved_port(&self) -> Option<u16> {
        if self.port != 0 {
            return Some(self.port);
        }
        env_value(&self.port_env).parse::<u16>().ok()
    }

    pub fn resolved_user(&self) -> String {
        direct_or_env(&self.user, &self.user_env)
    }

    pub fn resolved_password(&self) -> String {
        direct_or_env(&self.password, &self.password_env)
    }

    pub fn resolved_database(&self) -> String {
        direct_or_env(&self.database, &self.database_env)
    }

    pub fn resolved_proxy(&self) -> String {
        if self.resolved_no_proxy() {
            return String::new();
        }
        direct_or_env(&self.proxy, &self.proxy_env)
    }

    pub fn resolved_no_proxy(&self) -> bool {
        self.no_proxy || env_bool(&self.no_proxy_env)
    }

    pub fn is_configured(&self) -> bool {
        !self.resolved_database_url().is_empty()
    }

    fn is_partially_configured(&self) -> bool {
        let fields = [
            !self.resolved_database_url().is_empty(),
            !self.resolved_host().is_empty(),
            self.resolved_port().is_some(),
            !self.resolved_user().is_empty(),
            !self.resolved_password().is_empty(),
            !self.resolved_database().is_empty(),
        ];
        fields.iter().any(|value| *value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssConfig {
    #[serde(default = "default_oss_provider")]
    pub provider: String,
    #[serde(default = "default_oss_provider_env")]
    pub provider_env: String,
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default = "default_oss_access_key_id_env")]
    pub access_key_id_env: String,
    #[serde(default)]
    pub access_key_secret: String,
    #[serde(default = "default_oss_access_key_secret_env")]
    pub access_key_secret_env: String,
    #[serde(default)]
    pub bucket: String,
    #[serde(default = "default_oss_bucket_env")]
    pub bucket_env: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default = "default_oss_endpoint_env")]
    pub endpoint_env: String,
    #[serde(default)]
    pub region: String,
    #[serde(default = "default_oss_region_env")]
    pub region_env: String,
    #[serde(default = "default_oss_public_upload_prefix")]
    pub public_upload_prefix: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default = "default_oss_proxy_env")]
    pub proxy_env: String,
}

impl Default for OssConfig {
    fn default() -> Self {
        Self {
            provider: default_oss_provider(),
            provider_env: default_oss_provider_env(),
            access_key_id: String::new(),
            access_key_id_env: default_oss_access_key_id_env(),
            access_key_secret: String::new(),
            access_key_secret_env: default_oss_access_key_secret_env(),
            bucket: String::new(),
            bucket_env: default_oss_bucket_env(),
            endpoint: String::new(),
            endpoint_env: default_oss_endpoint_env(),
            region: String::new(),
            region_env: default_oss_region_env(),
            public_upload_prefix: default_oss_public_upload_prefix(),
            proxy: String::new(),
            proxy_env: default_oss_proxy_env(),
        }
    }
}

impl OssConfig {
    pub fn resolved_provider(&self) -> String {
        let env = env_value(&self.provider_env);
        if !env.trim().is_empty() {
            return env.trim().to_ascii_lowercase();
        }
        let value = self.provider.trim();
        if value.trim().is_empty() {
            default_oss_provider()
        } else {
            value.trim().to_ascii_lowercase()
        }
    }

    pub fn resolved_access_key_id(&self) -> String {
        direct_or_env(&self.access_key_id, &self.access_key_id_env)
    }

    pub fn resolved_access_key_secret(&self) -> String {
        direct_or_env(&self.access_key_secret, &self.access_key_secret_env)
    }

    pub fn resolved_bucket(&self) -> String {
        direct_or_env(&self.bucket, &self.bucket_env)
    }

    pub fn resolved_endpoint(&self) -> String {
        direct_or_env(&self.endpoint, &self.endpoint_env)
            .trim_end_matches('/')
            .to_string()
    }

    pub fn resolved_region(&self) -> String {
        direct_or_env(&self.region, &self.region_env)
    }

    pub fn resolved_proxy(&self) -> String {
        direct_or_env(&self.proxy, &self.proxy_env)
    }

    pub fn is_configured(&self) -> bool {
        !self.resolved_access_key_id().is_empty()
            && !self.resolved_access_key_secret().is_empty()
            && !self.resolved_bucket().is_empty()
            && !self.resolved_endpoint().is_empty()
    }

    fn is_partially_configured(&self) -> bool {
        [
            !self.resolved_access_key_id().is_empty(),
            !self.resolved_access_key_secret().is_empty(),
            !self.resolved_bucket().is_empty(),
            !self.resolved_endpoint().is_empty(),
            !self.resolved_region().is_empty(),
        ]
        .iter()
        .any(|value| *value)
    }
}

fn direct_or_env(value: &str, env_name: &str) -> String {
    let direct = value.trim();
    if !direct.is_empty() {
        return direct.to_string();
    }
    env_value(env_name)
}

fn env_value(env_name: &str) -> String {
    let name = env_name.trim();
    if name.is_empty() {
        return String::new();
    }
    std::env::var(name).unwrap_or_default().trim().to_string()
}

fn env_bool(env_name: &str) -> bool {
    matches!(
        env_value(env_name).to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn default_cloud_mode() -> String {
    "local".to_string()
}

fn default_database_url_env() -> String {
    "DATABASE_URL".to_string()
}
fn default_pg_host_env() -> String {
    "HONE_POSTGRES_HOST".to_string()
}
fn default_pg_port_env() -> String {
    "HONE_POSTGRES_PORT".to_string()
}
fn default_pg_user_env() -> String {
    "HONE_POSTGRES_USER".to_string()
}
fn default_pg_password_env() -> String {
    "HONE_POSTGRES_PASSWORD".to_string()
}
fn default_pg_database_env() -> String {
    "HONE_POSTGRES_DATABASE".to_string()
}
fn default_pg_sslmode() -> String {
    "disable".to_string()
}
fn default_pg_proxy_env() -> String {
    "HONE_POSTGRES_PROXY".to_string()
}
fn default_pg_no_proxy_env() -> String {
    "HONE_POSTGRES_NO_PROXY".to_string()
}
fn default_oss_access_key_id_env() -> String {
    "HONE_OSS_ACCESS_KEY_ID".to_string()
}
fn default_oss_provider() -> String {
    "aliyun_oss".to_string()
}
fn default_oss_provider_env() -> String {
    "HONE_OSS_PROVIDER".to_string()
}
fn default_oss_access_key_secret_env() -> String {
    "HONE_OSS_ACCESS_KEY_SECRET".to_string()
}
fn default_oss_bucket_env() -> String {
    "HONE_OSS_BUCKET".to_string()
}
fn default_oss_endpoint_env() -> String {
    "HONE_OSS_ENDPOINT".to_string()
}
fn default_oss_region_env() -> String {
    "HONE_OSS_REGION".to_string()
}
fn default_oss_public_upload_prefix() -> String {
    "public-uploads".to_string()
}
fn default_oss_proxy_env() -> String {
    "HONE_OSS_PROXY".to_string()
}
