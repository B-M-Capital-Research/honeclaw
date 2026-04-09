use async_trait::async_trait;
use hone_channels::outbound::{OutboundAdapter, split_segments};
use hone_channels::think::{ThinkRenderStyle, render_think_blocks};
use teloxide::prelude::*;
use teloxide::types::{MessageId, ParseMode, ReplyParameters};
use tracing::warn;

use super::markdown_v2::{
    prepend_reply_prefix_placeholder, sanitize_telegram_html_public, truncate_telegram_progress,
};

#[derive(Clone)]
pub(crate) struct TelegramOutboundAdapter {
    pub(crate) bot: Bot,
    pub(crate) chat_id: ChatId,
    pub(crate) max_len: usize,
    pub(crate) reply_prefix: Option<String>,
    pub(crate) show_reasoning: bool,
}

#[async_trait]
impl OutboundAdapter for TelegramOutboundAdapter {
    type Placeholder = MessageId;

    async fn send_placeholder(&self, text: &str) -> Option<Self::Placeholder> {
        send_placeholder_message(&self.bot, self.chat_id, text).await
    }

    async fn update_progress(&self, placeholder: Option<&Self::Placeholder>, text: &str) {
        let content =
            sanitize_telegram_html_public(&truncate_telegram_progress(text, self.max_len));
        if let Some(message_id) = placeholder {
            let _ =
                edit_message_with_fallback(&self.bot, self.chat_id, *message_id, &content).await;
        } else {
            let _ = send_message_with_fallback(&self.bot, self.chat_id, &content, None).await;
        }
    }

    async fn send_response(&self, placeholder: Option<&Self::Placeholder>, text: &str) -> usize {
        let rendered = render_think_blocks(text, ThinkRenderStyle::TelegramHtmlQuote);
        let content = sanitize_telegram_html_public(&prepend_reply_prefix_placeholder(
            self.reply_prefix.as_deref(),
            &rendered,
        ));
        let segments = split_segments(&content, self.max_len, 3500);
        if let Some(message_id) = placeholder {
            let first = segments
                .first()
                .cloned()
                .unwrap_or_else(|| "收到。".to_string());
            if edit_message_with_fallback(&self.bot, self.chat_id, *message_id, &first).await {
                let remaining = segments.iter().skip(1).cloned().collect::<Vec<_>>();
                1 + send_segments(&self.bot, self.chat_id, remaining, Some(*message_id)).await
            } else {
                send_segments(&self.bot, self.chat_id, segments, None).await
            }
        } else {
            send_segments(&self.bot, self.chat_id, segments, None).await
        }
    }

    async fn send_error(&self, placeholder: Option<&Self::Placeholder>, text: &str) {
        let content = sanitize_telegram_html_public(&prepend_reply_prefix_placeholder(
            self.reply_prefix.as_deref(),
            text,
        ));
        if let Some(message_id) = placeholder {
            if !edit_message_with_fallback(&self.bot, self.chat_id, *message_id, &content).await {
                let _ = send_segments(&self.bot, self.chat_id, vec![content], None).await;
            }
        } else {
            let _ = send_segments(&self.bot, self.chat_id, vec![content], None).await;
        }
    }

    fn show_reasoning(&self) -> bool {
        self.show_reasoning
    }
}

pub(crate) async fn send_segments(
    bot: &Bot,
    chat_id: ChatId,
    segments: Vec<String>,
    reply_to: Option<MessageId>,
) -> usize {
    let mut sent = 0usize;
    let mut previous = reply_to;
    for seg in segments {
        if let Some(message_id) = send_message_with_fallback(bot, chat_id, &seg, previous).await {
            previous = Some(message_id);
            sent += 1;
        } else {
            break;
        }
    }
    sent
}

pub(crate) async fn send_message_with_fallback(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    reply_to: Option<MessageId>,
) -> Option<MessageId> {
    let mut request = bot.send_message(chat_id, text).parse_mode(ParseMode::Html);
    if let Some(message_id) = reply_to {
        request = request.reply_parameters(ReplyParameters::new(message_id));
    }
    let res = request.await;
    match res {
        Ok(msg) => Some(msg.id),
        Err(e) => {
            warn!("[Telegram] HTML 发送失败，回退纯文本: {e}");
            let mut fallback = bot.send_message(chat_id, text);
            if let Some(message_id) = reply_to {
                fallback = fallback.reply_parameters(ReplyParameters::new(message_id));
            }
            fallback.await.ok().map(|msg| msg.id)
        }
    }
}

pub(crate) async fn send_placeholder_message(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
) -> Option<MessageId> {
    match bot
        .send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await
    {
        Ok(msg) => Some(msg.id),
        Err(e) => {
            warn!("[Telegram] 发送占位消息失败: {e}");
            None
        }
    }
}

pub(crate) async fn edit_message_with_fallback(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    text: &str,
) -> bool {
    let res = bot
        .edit_message_text(chat_id, message_id, text)
        .parse_mode(ParseMode::Html)
        .await;
    match res {
        Ok(_) => true,
        Err(e) => {
            if telegram_message_not_modified(&e.to_string()) {
                return true;
            }
            warn!("[Telegram] HTML 编辑失败，回退纯文本: {e}");
            match bot.edit_message_text(chat_id, message_id, text).await {
                Ok(_) => true,
                Err(err) if telegram_message_not_modified(&err.to_string()) => true,
                Err(_) => false,
            }
        }
    }
}

pub(crate) fn telegram_message_not_modified(error_text: &str) -> bool {
    error_text.contains("message is not modified")
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
