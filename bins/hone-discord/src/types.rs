use serenity::all::Message;

pub(crate) use hone_channels::ingress::GroupTriggerMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplyMentionMode {
    Adaptive,
    Always,
    Never,
}

#[derive(Debug, Clone)]
pub(crate) struct GroupReplyRuntimeConfig {
    pub enabled: bool,
    #[allow(dead_code)]
    pub trigger_mode: GroupTriggerMode,
    pub window_seconds: u64,
    pub mention_fast_delay_seconds: u64,
    pub queue_capacity_per_channel: usize,
    pub max_batch_messages: usize,
    pub backlog_keep_latest: usize,
    pub backlog_summary_max_chars: usize,
    pub reply_mention_mode: ReplyMentionMode,
    pub question_signal_enabled: bool,
}

impl GroupReplyRuntimeConfig {
    pub(crate) fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let cfg = &config.discord.group_reply;
        let trigger_mode = GroupTriggerMode::from_config_value(&cfg.trigger_mode);
        let reply_mention_mode = match cfg.reply_mention_mode.trim() {
            "always" => ReplyMentionMode::Always,
            "never" => ReplyMentionMode::Never,
            _ => ReplyMentionMode::Adaptive,
        };
        Self {
            enabled: cfg.enabled,
            trigger_mode,
            window_seconds: cfg.window_seconds.max(1),
            mention_fast_delay_seconds: cfg.mention_fast_delay_seconds,
            queue_capacity_per_channel: cfg.queue_capacity_per_channel.max(1),
            max_batch_messages: cfg.max_batch_messages.max(1),
            backlog_keep_latest: cfg.backlog_keep_latest.max(1),
            backlog_summary_max_chars: cfg.backlog_summary_max_chars.max(80),
            reply_mention_mode,
            question_signal_enabled: cfg.question_signal_enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ChannelKey {
    pub guild_id: u64,
    pub channel_id: u64,
}

impl ChannelKey {
    pub(crate) fn from_message(msg: &Message) -> Option<Self> {
        Some(Self {
            guild_id: msg.guild_id?.get(),
            channel_id: msg.channel_id.get(),
        })
    }

    pub(crate) fn scope(&self) -> String {
        format!("g:{}:c:{}", self.guild_id, self.channel_id)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GroupQueuedMessage {
    pub channel_key: ChannelKey,
    pub author_id: String,
    pub author_name: String,
    pub author_mention: String,
    pub direct_mention: bool,
    pub question_signal: bool,
    pub user_input: String,
}
