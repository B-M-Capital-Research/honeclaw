use async_trait::async_trait;
use hone_channels::outbound::{
    OutboundAdapter, PlatformMessageSplitter, ReasoningVisibility, ResponseContentSegment,
    split_response_content_segments,
};
use hone_channels::think::{ThinkRenderStyle, render_think_blocks};
use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageId, ParseMode, ReplyParameters};
use tracing::warn;

use super::markdown_v2::{
    prepend_reply_prefix_placeholder, sanitize_telegram_html_public, truncate_telegram_progress,
};

/// Telegram 单条消息硬上限（官方 4096,留 buffer 给 `<a>` / `<code>` 等标签转义）。
pub(crate) const TELEGRAM_HARD_MAX_CHARS: usize = 3500;

/// Telegram 分段适配器。输出是 HTML 格式,所以默认走 `split_html`。
pub(crate) struct TelegramSplitter;

impl PlatformMessageSplitter for TelegramSplitter {
    fn hard_max_chars(&self) -> usize {
        TELEGRAM_HARD_MAX_CHARS
    }
}

#[derive(Clone)]
pub(crate) struct TelegramOutboundAdapter {
    pub(crate) bot: Bot,
    pub(crate) chat_id: ChatId,
    pub(crate) max_len: usize,
    pub(crate) reply_prefix: Option<String>,
    pub(crate) reasoning_visibility: ReasoningVisibility,
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
        let rendered = render_think_blocks(text, ThinkRenderStyle::Hidden);
        let content = prepend_reply_prefix_placeholder(self.reply_prefix.as_deref(), &rendered);
        let mut owned = placeholder.copied();
        send_response_segments(
            &self.bot,
            self.chat_id,
            &mut owned,
            split_response_content_segments(&content),
            self.max_len,
        )
        .await
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

    fn reasoning_visibility(&self) -> ReasoningVisibility {
        self.reasoning_visibility
    }
}

pub(crate) async fn send_segments(
    bot: &Bot,
    chat_id: ChatId,
    segments: Vec<String>,
    reply_to: Option<MessageId>,
) -> usize {
    send_segments_with_last(bot, chat_id, segments, reply_to)
        .await
        .0
}

async fn send_segments_with_last(
    bot: &Bot,
    chat_id: ChatId,
    segments: Vec<String>,
    reply_to: Option<MessageId>,
) -> (usize, Option<MessageId>) {
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
    (sent, previous)
}

pub(crate) async fn send_response_segments(
    bot: &Bot,
    chat_id: ChatId,
    placeholder: &mut Option<MessageId>,
    segments: Vec<ResponseContentSegment>,
    max_len: usize,
) -> usize {
    let mut sent = 0usize;
    let mut previous: Option<MessageId> = None;

    for segment in segments {
        match segment {
            ResponseContentSegment::Text(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let html = sanitize_telegram_html_public(trimmed);
                let parts = TelegramSplitter.split_html(&html, max_len);
                let (count, last) =
                    send_text_segments_with_previous(bot, chat_id, placeholder, previous, parts)
                        .await;
                if let Some(message_id) = last {
                    previous = Some(message_id);
                }
                sent += count;
            }
            ResponseContentSegment::LocalImage(marker) => {
                if placeholder.is_some() && previous.is_none() {
                    let (count, last) = send_text_segments_with_previous(
                        bot,
                        chat_id,
                        placeholder,
                        previous,
                        vec!["图表如下：".to_string()],
                    )
                    .await;
                    if let Some(message_id) = last {
                        previous = Some(message_id);
                    }
                    sent += count;
                }

                match send_local_image(bot, chat_id, &marker.path, previous).await {
                    Ok(Some(message_id)) => {
                        previous = Some(message_id);
                        sent += 1;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        warn!("[Telegram] 发送图片失败: {err}");
                        let note = sanitize_telegram_html_public(&format!(
                            "（图表发送失败：{}）",
                            file_label_from_path(&marker.path)
                        ));
                        let parts = TelegramSplitter.split_html(&note, max_len);
                        let (count, last) = send_text_segments_with_previous(
                            bot,
                            chat_id,
                            placeholder,
                            previous,
                            parts,
                        )
                        .await;
                        if let Some(message_id) = last {
                            previous = Some(message_id);
                        }
                        sent += count;
                    }
                }
            }
        }
    }

    if sent == 0 {
        let (count, _last) = send_text_segments_with_previous(
            bot,
            chat_id,
            placeholder,
            previous,
            vec!["收到。".to_string()],
        )
        .await;
        sent += count;
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

async fn send_text_segments_with_previous(
    bot: &Bot,
    chat_id: ChatId,
    placeholder: &mut Option<MessageId>,
    previous: Option<MessageId>,
    segments: Vec<String>,
) -> (usize, Option<MessageId>) {
    if segments.is_empty() {
        return (0, previous);
    }

    if let Some(message_id) = placeholder.take() {
        let first = segments
            .first()
            .cloned()
            .unwrap_or_else(|| "收到。".to_string());
        if edit_message_with_fallback(bot, chat_id, message_id, &first).await {
            let remaining = segments.iter().skip(1).cloned().collect::<Vec<_>>();
            let (follow_sent, last) =
                send_segments_with_last(bot, chat_id, remaining, Some(message_id)).await;
            return (1 + follow_sent, last.or(Some(message_id)));
        }
    }

    send_segments_with_last(bot, chat_id, segments, previous).await
}

async fn send_local_image(
    bot: &Bot,
    chat_id: ChatId,
    path: &str,
    reply_to: Option<MessageId>,
) -> Result<Option<MessageId>, String> {
    let mut request = bot.send_photo(chat_id, InputFile::file(path));
    if let Some(message_id) = reply_to {
        request = request.reply_parameters(ReplyParameters::new(message_id));
    }
    request
        .await
        .map(|message| Some(message.id))
        .map_err(|err| format!("上传 Telegram 图片失败: {err}"))
}

fn file_label_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "chart.png".to_string())
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
