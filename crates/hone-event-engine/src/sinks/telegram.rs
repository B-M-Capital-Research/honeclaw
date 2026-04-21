//! Telegram OutboundSink —— 直接调 Bot API `sendMessage`。
//!
//! - 私聊:`chat_id = actor.user_id`(就是 Telegram 用户数字 id)
//! - 群聊:`actor.channel_scope = Some("chat_<chat_id>")`,剥掉 `chat_` 前缀后
//!   就是 Telegram 的负数 chat_id
//!
//! 不做消息分段,`renderer::RenderFormat::Plain` 已经保证长度可控;真超长
//! Telegram 会返 400,外层看日志就能发现。

use async_trait::async_trait;
use hone_core::ActorIdentity;

use crate::renderer::RenderFormat;
use crate::router::OutboundSink;

pub struct TelegramSink {
    bot_token: String,
    client: reqwest::Client,
}

impl TelegramSink {
    pub fn new(bot_token: impl Into<String>) -> Self {
        Self {
            bot_token: bot_token.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
        }
    }

    fn chat_id_for(actor: &ActorIdentity) -> String {
        match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => {
                scope.strip_prefix("chat_").unwrap_or(scope).to_string()
            }
            _ => actor.user_id.clone(),
        }
    }
}

#[async_trait]
impl OutboundSink for TelegramSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let chat_id = Self::chat_id_for(actor);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": body,
                "disable_web_page_preview": true,
            }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("telegram sendMessage {status}: {detail}");
        }
        Ok(())
    }

    fn format(&self) -> RenderFormat {
        RenderFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::ActorIdentity;

    #[test]
    fn chat_id_for_direct_uses_user_id() {
        let actor = ActorIdentity::new("telegram", "8039067465", None::<String>).unwrap();
        assert_eq!(TelegramSink::chat_id_for(&actor), "8039067465");
    }

    #[test]
    fn chat_id_for_group_strips_chat_prefix() {
        let actor =
            ActorIdentity::new("telegram", "8039067465", Some("chat_-1002012381143")).unwrap();
        assert_eq!(TelegramSink::chat_id_for(&actor), "-1002012381143");
    }

    #[test]
    fn chat_id_for_group_without_prefix_passes_through() {
        let actor = ActorIdentity::new("telegram", "8039067465", Some("-1001234567890")).unwrap();
        assert_eq!(TelegramSink::chat_id_for(&actor), "-1001234567890");
    }
}
