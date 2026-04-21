//! Discord OutboundSink —— 直接调 Bot API。
//!
//! - 群聊:`actor.channel_scope` 存 channel_id(scope 字符串可能带 `channel_`
//!   前缀,按 Telegram sink 同样剥一下);直接 `POST /channels/{id}/messages`
//! - 私聊:Discord DM 需要先 `POST /users/@me/channels { recipient_id }` 换一个
//!   DM channel_id。结果缓存在 `DashMap` 里,后续复用。
//!
//! Discord Bot token 要加 `Bot ` 前缀(authorization header);API base v10。

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use hone_core::ActorIdentity;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::renderer::RenderFormat;
use crate::router::OutboundSink;

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

pub struct DiscordSink {
    bot_token: String,
    client: reqwest::Client,
    dm_channel_cache: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Deserialize)]
struct CreateDmResp {
    id: String,
}

impl DiscordSink {
    pub fn new(bot_token: impl Into<String>) -> Self {
        Self {
            bot_token: bot_token.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            dm_channel_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bot {}", self.bot_token)
    }

    async fn dm_channel_id(&self, user_id: &str) -> anyhow::Result<String> {
        {
            let cache = self.dm_channel_cache.read().await;
            if let Some(id) = cache.get(user_id) {
                return Ok(id.clone());
            }
        }
        let resp = self
            .client
            .post(format!("{DISCORD_API_BASE}/users/@me/channels"))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "recipient_id": user_id }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("discord create DM {status}: {detail}");
        }
        let parsed: CreateDmResp = resp.json().await?;
        self.dm_channel_cache
            .write()
            .await
            .insert(user_id.to_string(), parsed.id.clone());
        Ok(parsed.id)
    }

    fn channel_id_for_group(scope: &str) -> String {
        scope.strip_prefix("channel_").unwrap_or(scope).to_string()
    }

    async fn send_to_channel(&self, channel_id: &str, body: &str) -> anyhow::Result<()> {
        let resp = self
            .client
            .post(format!("{DISCORD_API_BASE}/channels/{channel_id}/messages"))
            .header("Authorization", self.auth_header())
            .json(&serde_json::json!({ "content": body }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("discord send {status}: {detail}");
        }
        Ok(())
    }
}

#[async_trait]
impl OutboundSink for DiscordSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        let channel_id = match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => Self::channel_id_for_group(scope),
            _ => self.dm_channel_id(&actor.user_id).await?,
        };
        self.send_to_channel(&channel_id, body).await
    }

    fn format(&self) -> RenderFormat {
        RenderFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_id_strips_prefix() {
        assert_eq!(
            DiscordSink::channel_id_for_group("channel_123456789"),
            "123456789"
        );
    }

    #[test]
    fn channel_id_without_prefix_passes_through() {
        assert_eq!(DiscordSink::channel_id_for_group("987654321"), "987654321");
    }
}
