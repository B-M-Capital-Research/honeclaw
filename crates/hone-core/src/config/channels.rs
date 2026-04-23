use serde::de::Deserializer;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChatScope {
    #[serde(rename = "DM_ONLY", alias = "dm_only")]
    #[default]
    DmOnly,
    #[serde(rename = "GROUPCHAT_ONLY", alias = "groupchat_only")]
    GroupchatOnly,
    #[serde(rename = "ALL", alias = "all")]
    All,
}

impl ChatScope {
    pub fn allows_direct(self) -> bool {
        matches!(self, Self::DmOnly | Self::All)
    }

    pub fn allows_group(self) -> bool {
        matches!(self, Self::GroupchatOnly | Self::All)
    }
}

#[derive(Debug, Deserialize)]
struct FeishuConfigWire {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    app_id: String,
    #[serde(default)]
    app_secret: String,
    #[serde(default)]
    allow_emails: Vec<String>,
    #[serde(default)]
    allow_mobiles: Vec<String>,
    #[serde(default)]
    allow_open_ids: Vec<String>,
    #[serde(default)]
    chat_scope: Option<ChatScope>,
    #[serde(default)]
    dm_only: Option<bool>,
    #[serde(default = "default_feishu_max_msg_len")]
    max_message_length: usize,
    #[serde(default = "default_feishu_facade_url")]
    facade_url: String,
    #[serde(default = "default_feishu_callback_addr")]
    callback_addr: String,
    #[serde(default = "default_feishu_facade_addr")]
    facade_addr: String,
    #[serde(default = "default_feishu_startup_timeout")]
    startup_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeishuConfig {
    pub enabled: bool,
    pub app_id: String,
    pub app_secret: String,
    pub allow_emails: Vec<String>,
    pub allow_mobiles: Vec<String>,
    pub allow_open_ids: Vec<String>,
    pub chat_scope: ChatScope,
    pub max_message_length: usize,
    pub facade_url: String,
    pub callback_addr: String,
    pub facade_addr: String,
    pub startup_timeout_seconds: u64,
}

impl<'de> Deserialize<'de> for FeishuConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = FeishuConfigWire::deserialize(deserializer)?;
        Ok(Self {
            enabled: wire.enabled,
            app_id: wire.app_id,
            app_secret: wire.app_secret,
            allow_emails: wire.allow_emails,
            allow_mobiles: wire.allow_mobiles,
            allow_open_ids: wire.allow_open_ids,
            chat_scope: resolve_chat_scope("feishu", wire.chat_scope, wire.dm_only),
            max_message_length: wire.max_message_length,
            facade_url: wire.facade_url,
            callback_addr: wire.callback_addr,
            facade_addr: wire.facade_addr,
            startup_timeout_seconds: wire.startup_timeout_seconds,
        })
    }
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
            chat_scope: ChatScope::DmOnly,
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

#[derive(Debug, Deserialize)]
struct TelegramConfigWire {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    bot_token: String,
    #[serde(default)]
    allow_from: Vec<String>,
    #[serde(default)]
    chat_scope: Option<ChatScope>,
    #[serde(default)]
    dm_only: Option<bool>,
    #[serde(default = "default_tg_max_msg_len")]
    max_message_length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub allow_from: Vec<String>,
    pub chat_scope: ChatScope,
    pub max_message_length: usize,
}

impl<'de> Deserialize<'de> for TelegramConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TelegramConfigWire::deserialize(deserializer)?;
        Ok(Self {
            enabled: wire.enabled,
            bot_token: wire.bot_token,
            allow_from: wire.allow_from,
            chat_scope: resolve_chat_scope("telegram", wire.chat_scope, wire.dm_only),
            max_message_length: wire.max_message_length,
        })
    }
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allow_from: Vec::new(),
            chat_scope: ChatScope::DmOnly,
            max_message_length: default_tg_max_msg_len(),
        }
    }
}

fn default_tg_max_msg_len() -> usize {
    3500
}

#[derive(Debug, Deserialize)]
struct DiscordConfigWire {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    bot_token: String,
    #[serde(default)]
    allow_from: Vec<String>,
    #[serde(default)]
    chat_scope: Option<ChatScope>,
    #[serde(default)]
    dm_only: Option<bool>,
    #[serde(default = "default_dc_max_msg_len")]
    max_message_length: usize,
    #[serde(default)]
    group_reply: DiscordGroupReplyConfig,
    #[serde(default)]
    watch: DiscordWatchConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub bot_token: String,
    pub allow_from: Vec<String>,
    pub chat_scope: ChatScope,
    pub max_message_length: usize,
    pub group_reply: DiscordGroupReplyConfig,
    pub watch: DiscordWatchConfig,
}

impl<'de> Deserialize<'de> for DiscordConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DiscordConfigWire::deserialize(deserializer)?;
        Ok(Self {
            enabled: wire.enabled,
            bot_token: wire.bot_token,
            allow_from: wire.allow_from,
            chat_scope: resolve_chat_scope("discord", wire.chat_scope, wire.dm_only),
            max_message_length: wire.max_message_length,
            group_reply: wire.group_reply,
            watch: wire.watch,
        })
    }
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allow_from: Vec::new(),
            chat_scope: ChatScope::DmOnly,
            max_message_length: default_dc_max_msg_len(),
            group_reply: DiscordGroupReplyConfig::default(),
            watch: DiscordWatchConfig::default(),
        }
    }
}

fn default_dc_max_msg_len() -> usize {
    1800
}

fn resolve_chat_scope(
    channel: &str,
    chat_scope: Option<ChatScope>,
    legacy_dm_only: Option<bool>,
) -> ChatScope {
    if chat_scope.is_some() && legacy_dm_only.is_some() {
        tracing::warn!(
            "[Config/{channel}] chat_scope and legacy dm_only are both set; chat_scope takes precedence"
        );
    }
    if let Some(scope) = chat_scope {
        return scope;
    }
    match legacy_dm_only {
        Some(true) => ChatScope::DmOnly,
        Some(false) => ChatScope::All,
        None => ChatScope::DmOnly,
    }
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
