use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hone_channels::outbound::{OutboundAdapter, split_segments};
use hone_channels::think::{ThinkRenderStyle, render_think_blocks};
use hone_core::ActorIdentity;
use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, CreateAllowedMentions, CreateCommand,
    CreateCommandOption, CreateMessage, EditMessage, Message, ResolvedValue,
};
use serenity::http::Http;
use tracing::warn;

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
        let rendered = render_think_blocks(text, ThinkRenderStyle::MarkdownQuote);
        let content = prepend_reply_prefix(self.reply_prefix.as_deref(), &rendered);
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

pub(crate) fn discord_actor(author_id: &str, channel_scope: Option<&str>) -> ActorIdentity {
    hone_channels::HoneBotCore::create_actor("discord", author_id, channel_scope)
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
    let mut parts = vec![format!("/{}", skill_name)];
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
    fn group_session_id_is_channel_scoped() {
        let actor = discord_actor("alice", Some("g:123:c:456"));
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
