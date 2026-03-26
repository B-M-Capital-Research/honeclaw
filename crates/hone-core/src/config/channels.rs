use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupContextConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub shared_session_enabled: bool,
    #[serde(default = "default_true")]
    pub pretrigger_window_enabled: bool,
    #[serde(default = "default_group_context_pretrigger_window_max_messages")]
    pub pretrigger_window_max_messages: usize,
    #[serde(default = "default_group_context_pretrigger_window_max_age_seconds")]
    pub pretrigger_window_max_age_seconds: u64,
    #[serde(default = "default_group_context_recent_context_limit")]
    pub recent_context_limit: usize,
    #[serde(default = "default_group_context_compress_threshold_messages")]
    pub compress_threshold_messages: usize,
    #[serde(default = "default_group_context_compress_threshold_bytes")]
    pub compress_threshold_bytes: usize,
    #[serde(default = "default_group_context_retain_recent")]
    pub retain_recent_after_compress: usize,
}

impl Default for GroupContextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shared_session_enabled: true,
            pretrigger_window_enabled: true,
            pretrigger_window_max_messages: default_group_context_pretrigger_window_max_messages(),
            pretrigger_window_max_age_seconds:
                default_group_context_pretrigger_window_max_age_seconds(),
            recent_context_limit: default_group_context_recent_context_limit(),
            compress_threshold_messages: default_group_context_compress_threshold_messages(),
            compress_threshold_bytes: default_group_context_compress_threshold_bytes(),
            retain_recent_after_compress: default_group_context_retain_recent(),
        }
    }
}

fn default_group_context_pretrigger_window_max_messages() -> usize {
    10
}
fn default_group_context_pretrigger_window_max_age_seconds() -> u64 {
    300
}
fn default_group_context_recent_context_limit() -> usize {
    18
}
fn default_group_context_compress_threshold_messages() -> usize {
    24
}
fn default_group_context_compress_threshold_bytes() -> usize {
    48 * 1024
}
fn default_group_context_retain_recent() -> usize {
    8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMessageConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_db_path")]
    pub db_path: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
    #[serde(default)]
    pub target_handle: String,
    /// hone-imessage 内置 HTTP 服务监听地址（供 hone-console-page 定时任务回调使用）
    #[serde(default = "default_imessage_listen_addr")]
    pub listen_addr: String,
}

impl Default for IMessageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            db_path: default_db_path(),
            poll_interval: default_poll_interval(),
            target_handle: String::new(),
            listen_addr: default_imessage_listen_addr(),
        }
    }
}

fn default_imessage_listen_addr() -> String {
    "127.0.0.1:8078".to_string()
}

fn default_true() -> bool {
    true
}
fn default_db_path() -> String {
    "~/Library/Messages/chat.db".to_string()
}
fn default_poll_interval() -> u64 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub app_secret: String,
    #[serde(default)]
    pub allow_emails: Vec<String>,
    #[serde(default)]
    pub allow_mobiles: Vec<String>,
    #[serde(default)]
    pub allow_open_ids: Vec<String>,
    #[serde(default = "default_true")]
    pub dm_only: bool,
    #[serde(default = "default_feishu_max_msg_len")]
    pub max_message_length: usize,
    #[serde(default = "default_feishu_facade_url")]
    pub facade_url: String,
    #[serde(default = "default_feishu_callback_addr")]
    pub callback_addr: String,
    #[serde(default = "default_feishu_facade_addr")]
    pub facade_addr: String,
    #[serde(default = "default_feishu_startup_timeout")]
    pub startup_timeout_seconds: u64,
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_id: String::new(),
            app_secret: String::new(),
            allow_emails: Vec::new(),
            allow_mobiles: Vec::new(),
            allow_open_ids: Vec::new(),
            dm_only: true,
            max_message_length: default_feishu_max_msg_len(),
            facade_url: default_feishu_facade_url(),
            callback_addr: default_feishu_callback_addr(),
            facade_addr: default_feishu_facade_addr(),
            startup_timeout_seconds: default_feishu_startup_timeout(),
        }
    }
}

fn default_feishu_max_msg_len() -> usize {
    3500
}
fn default_feishu_facade_url() -> String {
    "http://127.0.0.1:18115/rpc".to_string()
}
fn default_feishu_callback_addr() -> String {
    "127.0.0.1:18116".to_string()
}
fn default_feishu_facade_addr() -> String {
    "127.0.0.1:18115".to_string()
}
fn default_feishu_startup_timeout() -> u64 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub allow_from: Vec<String>,
    #[serde(default = "default_true")]
    pub dm_only: bool,
    #[serde(default = "default_tg_max_msg_len")]
    pub max_message_length: usize,
}

fn default_tg_max_msg_len() -> usize {
    3500
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscordConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub allow_from: Vec<String>,
    #[serde(default = "default_true")]
    pub dm_only: bool,
    #[serde(default = "default_dc_max_msg_len")]
    pub max_message_length: usize,
    #[serde(default)]
    pub group_reply: DiscordGroupReplyConfig,
    #[serde(default)]
    pub watch: DiscordWatchConfig,
}

fn default_dc_max_msg_len() -> usize {
    1800
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordGroupReplyConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for DiscordGroupReplyConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscordWatchConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub channel_ids: Vec<String>,
    #[serde(default = "default_true")]
    pub r#loop: bool,
    #[serde(default)]
    pub verbose: bool,
    /// 额外的未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct XConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub dry_run: bool,
    #[serde(default = "default_image_count")]
    pub default_image_count: u32,
    #[serde(default = "default_x_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub oauth1: XOAuth1Config,
}

fn default_image_count() -> u32 {
    3
}
fn default_x_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct XOAuth1Config {
    #[serde(default)]
    pub consumer_key: String,
    #[serde(default)]
    pub consumer_secret: String,
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub access_token_secret: String,
    #[serde(default = "default_consumer_key_env")]
    pub consumer_key_env: String,
    #[serde(default = "default_consumer_secret_env")]
    pub consumer_secret_env: String,
    #[serde(default = "default_access_token_env")]
    pub access_token_env: String,
    #[serde(default = "default_access_token_secret_env")]
    pub access_token_secret_env: String,
}

fn default_consumer_key_env() -> String {
    "X_CONSUMER_KEY".to_string()
}
fn default_consumer_secret_env() -> String {
    "X_CONSUMER_SECRET".to_string()
}
fn default_access_token_env() -> String {
    "X_ACCESS_TOKEN".to_string()
}
fn default_access_token_secret_env() -> String {
    "X_ACCESS_TOKEN_SECRET".to_string()
}
