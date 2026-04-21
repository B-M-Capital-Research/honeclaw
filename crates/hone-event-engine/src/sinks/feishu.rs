//! Feishu OutboundSink —— 直接打 Feishu Open API。
//!
//! - 获取 `tenant_access_token` 并本地缓存(提前 5 分钟过期,避开边界失败)
//! - `POST /open-apis/im/v1/messages?receive_id_type={open_id|chat_id}` 发 text
//!   类型消息
//! - 私聊:`receive_id_type=open_id`,`receive_id = actor.user_id`(prefs 文件里
//!   的 user_id 在 Feishu 场景就是 open_id)
//! - 群聊:`receive_id_type=chat_id`,从 `actor.channel_scope` 取(剥 `chat_`
//!   前缀跟 Telegram 保持一致;纯 chat_id 也兼容)
//!
//! 为什么不走 Go facade:facade 主要承接交互式对话的复杂路径(卡片 / thread /
//! placeholder),engine 只需要最朴素的一段 text,多一跳 JSON-RPC 反而引入依赖。

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use hone_core::ActorIdentity;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::renderer::RenderFormat;
use crate::router::OutboundSink;

const FEISHU_TOKEN_URL: &str =
    "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
const FEISHU_SEND_URL: &str = "https://open.feishu.cn/open-apis/im/v1/messages";

pub struct FeishuSink {
    app_id: String,
    app_secret: String,
    client: reqwest::Client,
    token_cache: Arc<RwLock<Option<(String, Instant)>>>,
}

#[derive(Deserialize)]
struct TokenResp {
    code: i64,
    msg: String,
    tenant_access_token: Option<String>,
    expire: Option<u64>,
}

#[derive(Deserialize)]
struct SendResp {
    code: i64,
    msg: String,
}

impl FeishuSink {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn token(&self) -> anyhow::Result<String> {
        {
            let cache = self.token_cache.read().await;
            if let Some((t, exp)) = &*cache {
                if Instant::now() < *exp {
                    return Ok(t.clone());
                }
            }
        }
        let resp = self
            .client
            .post(FEISHU_TOKEN_URL)
            .json(&serde_json::json!({
                "app_id": &self.app_id,
                "app_secret": &self.app_secret,
            }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("feishu token HTTP {status}");
        }
        let parsed: TokenResp = resp.json().await?;
        if parsed.code != 0 {
            anyhow::bail!("feishu token error code={} msg={}", parsed.code, parsed.msg);
        }
        let token = parsed
            .tenant_access_token
            .ok_or_else(|| anyhow::anyhow!("feishu token missing"))?;
        let expire_secs = parsed.expire.unwrap_or(7200).max(300);
        let ttl = Duration::from_secs(expire_secs - 300);
        *self.token_cache.write().await = Some((token.clone(), Instant::now() + ttl));
        Ok(token)
    }

    fn receive_target(actor: &ActorIdentity) -> (&'static str, String) {
        match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => {
                let chat_id = scope.strip_prefix("chat_").unwrap_or(scope).to_string();
                ("chat_id", chat_id)
            }
            _ => ("open_id", actor.user_id.clone()),
        }
    }
}

#[async_trait]
impl OutboundSink for FeishuSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        let token = self.token().await?;
        let (receive_id_type, receive_id) = Self::receive_target(actor);
        let content = serde_json::json!({ "text": body }).to_string();
        let resp = self
            .client
            .post(format!("{FEISHU_SEND_URL}?receive_id_type={receive_id_type}"))
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "receive_id": receive_id,
                "msg_type": "text",
                "content": content,
            }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("feishu send HTTP {status}: {detail}");
        }
        let parsed: SendResp = resp.json().await?;
        if parsed.code != 0 {
            anyhow::bail!("feishu send error code={} msg={}", parsed.code, parsed.msg);
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

    #[test]
    fn receive_target_direct_uses_open_id() {
        let actor = ActorIdentity::new("feishu", "ou_abc", None::<String>).unwrap();
        let (ty, id) = FeishuSink::receive_target(&actor);
        assert_eq!(ty, "open_id");
        assert_eq!(id, "ou_abc");
    }

    #[test]
    fn receive_target_group_strips_chat_prefix() {
        let actor = ActorIdentity::new("feishu", "ou_abc", Some("chat_oc_123")).unwrap();
        let (ty, id) = FeishuSink::receive_target(&actor);
        assert_eq!(ty, "chat_id");
        assert_eq!(id, "oc_123");
    }
}
