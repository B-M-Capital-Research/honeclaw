use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hone_channels::agent_session::{AgentSessionEvent, AgentSessionListener};
use hone_channels::outbound::{OutboundAdapter, split_segments};
use hone_core::ActorIdentity;
use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, CreateAllowedMentions, CreateCommand,
    CreateCommandOption, CreateMessage, EditMessage, Message, ResolvedValue,
};
use serenity::http::Http;
use tokio::sync::Mutex;
use tracing::warn;

use crate::types::{ChannelKey, GroupTriggerMode};

pub(crate) const DISCORD_SKILL_COMMAND: &str = "skill";

#[derive(Clone)]
pub(crate) struct DiscordOutboundAdapter {
    pub(crate) http: Arc<Http>,
    pub(crate) channel_id: ChannelId,
    pub(crate) max_len: usize,
    pub(crate) reply_prefix: Option<String>,
    pub(crate) show_reasoning: bool,
}

#[async_trait]
impl OutboundAdapter for DiscordOutboundAdapter {
    type Placeholder = Message;

    async fn send_placeholder(&self, text: &str) -> Option<Self::Placeholder> {
        send_placeholder_message(self.http.as_ref(), self.channel_id, text, self.max_len).await
    }

    async fn update_progress(&self, placeholder: Option<&Self::Placeholder>, text: &str) {
        let mut owned = placeholder.cloned();
        update_or_send_plain_text(
            self.http.as_ref(),
            self.channel_id,
            owned.as_mut(),
            text,
            self.max_len,
        )
        .await;
    }

    async fn send_response(&self, placeholder: Option<&Self::Placeholder>, text: &str) -> usize {
        let content = prepend_reply_prefix(self.reply_prefix.as_deref(), text);
        let segments = split_segments(&content, self.max_len, 1900);
        let mut owned = placeholder.cloned();
        let (sent, _) = send_or_edit_segments(
            self.http.as_ref(),
            self.channel_id,
            owned.as_mut(),
            segments,
        )
        .await;
        sent
    }

    async fn send_error(&self, placeholder: Option<&Self::Placeholder>, text: &str) {
        let content = prepend_reply_prefix(self.reply_prefix.as_deref(), text);
        let mut owned = placeholder.cloned();
        update_or_send_plain_text(
            self.http.as_ref(),
            self.channel_id,
            owned.as_mut(),
            &content,
            self.max_len,
        )
        .await;
    }

    fn show_reasoning(&self) -> bool {
        self.show_reasoning
    }
}

pub(crate) struct DiscordReasoningListener {
    pub(crate) http: Arc<Http>,
    pub(crate) channel_id: ChannelId,
    pub(crate) placeholder: Arc<Mutex<Option<Message>>>,
    pub(crate) progress: Arc<Mutex<DiscordProgressTranscript>>,
    pub(crate) max_len: usize,
    pub(crate) show_reasoning: bool,
}

#[derive(Clone)]
pub(crate) struct DiscordProgressTranscript {
    base_text: String,
    entries: Vec<String>,
}

impl DiscordProgressTranscript {
    pub(crate) fn new(base_text: &str) -> Self {
        Self {
            base_text: base_text.trim().to_string(),
            entries: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, entry: &str) -> Option<String> {
        let normalized = entry.trim();
        if normalized.is_empty() {
            return None;
        }
        if self.entries.iter().any(|existing| existing == normalized) {
            return None;
        }
        self.entries.push(normalized.to_string());
        Some(self.render())
    }

    fn render(&self) -> String {
        let mut lines = Vec::new();
        if !self.base_text.is_empty() {
            lines.push(self.base_text.clone());
        }
        lines.extend(self.entries.iter().map(|entry| format!("- {entry}")));
        lines.join("\n")
    }
}

#[async_trait]
impl AgentSessionListener for DiscordReasoningListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        let AgentSessionEvent::ToolStatus {
            status, reasoning, ..
        } = event
        else {
            return;
        };
        if !self.show_reasoning {
            return;
        }
        if status != "start" {
            return;
        }
        let Some(text) = reasoning.filter(|value| !value.trim().is_empty()) else {
            return;
        };
        let Some(content) = self.progress.lock().await.push(&text) else {
            return;
        };
        let content = truncate_chars(&content, self.max_len);
        let mut guard = self.placeholder.lock().await;
        if let Some(msg) = guard.as_mut() {
            if let Err(e) = msg
                .edit(&self.http, EditMessage::new().content(&content))
                .await
            {
                warn!("[Discord] 编辑占位消息失败: {}", e);
            }
        } else {
            let _ = self.channel_id.say(&self.http, &content).await;
        }
    }
}

pub(crate) fn discord_actor(author_id: &str, channel_key: Option<ChannelKey>) -> ActorIdentity {
    hone_channels::HoneBotCore::create_actor(
        "discord",
        author_id,
        channel_key.as_ref().map(ChannelKey::scope).as_deref(),
    )
    .expect("discord actor should be valid")
}

pub(crate) fn parse_channel_id_from_target(target: &str) -> Option<u64> {
    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    if target.chars().all(|c| c.is_ascii_digit()) {
        return target.parse().ok();
    }

    if let Some(rest) = target.strip_prefix("dm:") {
        return rest.split(':').next().and_then(|id| id.parse().ok());
    }

    if let Some(rest) = target.strip_prefix("guild:") {
        let parts: Vec<&str> = rest.split(':').collect();
        for idx in 0..parts.len().saturating_sub(1) {
            if parts[idx] == "channel" {
                return parts.get(idx + 1).and_then(|id| id.parse().ok());
            }
        }
    }

    None
}

#[allow(dead_code)]
pub(crate) fn should_trigger_by_mode(
    mode: GroupTriggerMode,
    direct_mention: bool,
    question_signal: bool,
) -> bool {
    mode.should_trigger(&hone_channels::ingress::GroupTrigger {
        direct_mention,
        reply_to_bot: false,
        question_signal,
    })
}

pub(crate) fn build_skill_slash_command() -> CreateCommand {
    CreateCommand::new(DISCORD_SKILL_COMMAND)
        .description("搜索并触发一个 skill")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "name",
                "输入 skill id、名称或别名搜索",
            )
            .required(true)
            .set_autocomplete(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "prompt",
                "加载 skill 后立刻附带的问题或任务",
            )
            .required(false),
        )
}

pub(crate) fn configured_skill_dirs(core: &hone_channels::HoneBotCore) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let system_dir = core
        .config
        .extra
        .get("skills_dir")
        .and_then(|value| value.as_str())
        .unwrap_or("./skills");
    dirs.push(PathBuf::from(system_dir));

    let custom_dir = std::env::var("HONE_DATA_DIR")
        .map(|root| PathBuf::from(root).join("custom_skills"))
        .unwrap_or_else(|_| PathBuf::from("./data/custom_skills"));
    if !dirs.iter().any(|dir| dir == &custom_dir) {
        dirs.push(custom_dir);
    }

    dirs
}

pub(crate) fn slash_option_string(
    command: &CommandInteraction,
    option_name: &str,
) -> Option<String> {
    command.data.options().into_iter().find_map(|option| {
        if option.name != option_name {
            return None;
        }
        match option.value {
            ResolvedValue::String(value) => Some(value.trim().to_string()),
            _ => None,
        }
    })
}

pub(crate) fn build_skill_command_input(skill_name: &str, prompt: Option<&str>) -> String {
    let mut parts = vec![format!("load_skill(\"{}\")", skill_name)];
    if let Some(prompt) = prompt.map(str::trim).filter(|prompt| !prompt.is_empty()) {
        parts.push(prompt.to_string());
    }
    parts.join("\n\n")
}

pub(crate) fn is_allowed_author(author_id: &str, allow_from: &[String]) -> bool {
    allow_from.is_empty() || allow_from.iter().any(|v| v == "*" || v == author_id)
}

pub(crate) fn is_direct_mention_message(msg: &Message, bot_user_id: u64) -> bool {
    if msg.mentions.iter().any(|u| u.id.get() == bot_user_id) {
        return true;
    }

    let mention_token = format!("<@{bot_user_id}>");
    let nickname_mention_token = format!("<@!{bot_user_id}>");
    if msg.content.contains(&mention_token) || msg.content.contains(&nickname_mention_token) {
        return true;
    }

    msg.referenced_message
        .as_ref()
        .map(|ref_msg| ref_msg.author.id.get() == bot_user_id)
        .unwrap_or(false)
}

pub(crate) fn prepend_reply_prefix(prefix: Option<&str>, text: &str) -> String {
    let Some(prefix) = prefix.map(str::trim).filter(|value| !value.is_empty()) else {
        return text.to_string();
    };

    let body = text.trim_start();
    if body.starts_with(prefix) {
        text.to_string()
    } else if body.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {body}")
    }
}

pub(crate) fn has_question_signal(content: &str) -> bool {
    let text = content.trim();
    if text.is_empty() {
        return false;
    }
    if text.contains('?') || text.contains('？') {
        return true;
    }

    let lower = text.to_lowercase();
    const QUESTION_SIGNALS_ZH: &[&str] = &[
        "请问",
        "怎么看",
        "怎么处理",
        "能不能",
        "可以吗",
        "帮我看",
        "给个建议",
        "分析下",
        "分析一下",
        "有啥看法",
        "有何建议",
    ];
    const QUESTION_SIGNALS_EN: &[&str] = &[
        "what", "why", "how", "which", "should", "could", "can i", "advice", "opinion",
    ];

    QUESTION_SIGNALS_ZH.iter().any(|kw| text.contains(kw))
        || QUESTION_SIGNALS_EN.iter().any(|kw| lower.contains(kw))
}

pub(crate) fn split_into_segments(text: &str, max_segment_size: usize) -> Vec<String> {
    split_segments(text, max_segment_size, 1900)
}

pub(crate) fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

pub(crate) async fn send_placeholder_message(
    http: &Http,
    channel_id: ChannelId,
    text: &str,
    max_len: usize,
) -> Option<Message> {
    let content = truncate_chars(text, max_len);
    match channel_id.say(http, &content).await {
        Ok(msg) => Some(msg),
        Err(e) => {
            warn!("[Discord] 发送占位消息失败: {}", e);
            None
        }
    }
}

pub(crate) async fn update_or_send_plain_text(
    http: &Http,
    channel_id: ChannelId,
    placeholder: Option<&mut Message>,
    text: &str,
    max_len: usize,
) {
    let content = truncate_chars(text, max_len);
    if let Some(msg) = placeholder {
        if let Err(e) = msg.edit(http, EditMessage::new().content(&content)).await {
            warn!("[Discord] 编辑占位消息失败: {}", e);
        } else {
            return;
        }
    }

    if let Err(e) = channel_id.say(http, &content).await {
        warn!("[Discord] 发送消息失败: {}", e);
    }
}

pub(crate) async fn send_or_edit_segments(
    http: &Http,
    channel_id: ChannelId,
    placeholder: Option<&mut Message>,
    segments: Vec<String>,
) -> (usize, usize) {
    let total = segments.len();
    if total == 0 {
        return (0, 0);
    }

    let mut sent = 0usize;
    let mut previous: Option<Message> = None;
    let mut iter = segments.into_iter();
    if let Some(first) = iter.next() {
        if let Some(msg) = placeholder {
            match msg
                .edit(
                    http,
                    EditMessage::new()
                        .content(&first)
                        .allowed_mentions(reply_allowed_mentions()),
                )
                .await
            {
                Ok(_) => {
                    sent += 1;
                    previous = Some(msg.clone());
                }
                Err(e) => {
                    warn!("[Discord] 编辑占位消息失败: {}", e);
                    match channel_id
                        .send_message(
                            http,
                            CreateMessage::new()
                                .content(&first)
                                .allowed_mentions(reply_allowed_mentions()),
                        )
                        .await
                    {
                        Ok(message) => {
                            previous = Some(message);
                            sent += 1;
                        }
                        Err(err) => {
                            warn!("[Discord] 发送消息失败: {}", err);
                        }
                    }
                }
            }
        } else {
            match channel_id
                .send_message(
                    http,
                    CreateMessage::new()
                        .content(&first)
                        .allowed_mentions(reply_allowed_mentions()),
                )
                .await
            {
                Ok(message) => {
                    previous = Some(message);
                    sent += 1;
                }
                Err(err) => warn!("[Discord] 发送消息失败: {}", err),
            }
        }
    }

    for seg in iter {
        let Some(prev) = previous.as_ref() else {
            break;
        };
        let builder = CreateMessage::new()
            .content(&seg)
            .reference_message(prev)
            .allowed_mentions(reply_allowed_mentions());
        match channel_id.send_message(http, builder).await {
            Ok(message) => {
                previous = Some(message);
                sent += 1;
            }
            Err(e) => {
                warn!("发送 Discord 回复消息失败: {}", e);
                break;
            }
        }
    }

    (sent, total)
}

fn reply_allowed_mentions() -> CreateAllowedMentions {
    CreateAllowedMentions::new()
        .everyone(true)
        .all_users(true)
        .all_roles(true)
        .replied_user(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_from_empty_means_allow_all() {
        assert!(is_allowed_author("123", &[]));
    }

    #[test]
    fn allow_from_supports_star_and_exact_match() {
        assert!(is_allowed_author("123", &["*".to_string()]));
        assert!(is_allowed_author(
            "123",
            &["123".to_string(), "456".to_string()]
        ));
        assert!(!is_allowed_author("123", &["456".to_string()]));
    }

    #[test]
    fn question_signal_detection_works() {
        assert!(has_question_signal("请问这个怎么看"));
        assert!(has_question_signal("what do you think"));
        assert!(has_question_signal("这可以吗？"));
        assert!(!has_question_signal("收到，已处理"));
    }

    #[test]
    fn group_session_id_is_channel_scoped() {
        let key = ChannelKey {
            guild_id: 123,
            channel_id: 456,
        };
        assert_eq!(key.scope(), "g:123:c:456");
        let actor = discord_actor("alice", Some(key));
        assert!(actor.session_id().contains("g_3a123_3ac_3a456"));
    }

    #[test]
    fn parse_channel_id_from_target_handles_dm() {
        assert_eq!(parse_channel_id_from_target("dm:123"), Some(123));
        assert_eq!(parse_channel_id_from_target("dm:456:slash"), Some(456));
    }

    #[test]
    fn parse_channel_id_from_target_handles_guild() {
        assert_eq!(
            parse_channel_id_from_target("guild:111:channel:222"),
            Some(222)
        );
        assert_eq!(
            parse_channel_id_from_target("guild:111:channel:222:slash"),
            Some(222)
        );
    }
}
