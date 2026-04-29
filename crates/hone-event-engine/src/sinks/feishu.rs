//! Feishu OutboundSink —— 直接打 Feishu Open API。
//!
//! - 获取 `tenant_access_token` 并本地缓存(提前 5 分钟过期,避开边界失败)
//! - `POST /open-apis/im/v1/messages?receive_id_type={open_id|chat_id}` 发 post
//!   富文本消息
//! - 私聊:默认使用 actor.user_id；单用户安装若配置了唯一 allow_mobile/allow_email，
//!   会先解析 current-app-scoped open_id，避免 app 迁移后的旧 open_id 跨 app。
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
use serde_json::Value;
use tokio::sync::RwLock;

use crate::digest::DigestPayload;
use crate::renderer::RenderFormat;
use crate::router::OutboundSink;
use crate::sinks::feishu_card::build_feishu_card;

const FEISHU_TOKEN_URL: &str =
    "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
const FEISHU_SEND_URL: &str = "https://open.feishu.cn/open-apis/im/v1/messages";

pub struct FeishuSink {
    app_id: String,
    app_secret: String,
    client: reqwest::Client,
    token_cache: Arc<RwLock<Option<(String, Instant)>>>,
    direct_contact: Option<FeishuDirectContact>,
    direct_open_id_cache: Arc<RwLock<Option<String>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FeishuDirectContactKind {
    Email,
    Mobile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FeishuDirectContact {
    kind: FeishuDirectContactKind,
    value: String,
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
            direct_contact: None,
            direct_open_id_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_single_direct_contact_fallback(
        mut self,
        allow_emails: &[String],
        allow_mobiles: &[String],
    ) -> Self {
        self.direct_contact = single_direct_contact(allow_emails, allow_mobiles);
        self
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

    #[cfg(test)]
    fn static_receive_target(actor: &ActorIdentity) -> (&'static str, String) {
        match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => {
                let chat_id = scope.strip_prefix("chat_").unwrap_or(scope).to_string();
                ("chat_id", chat_id)
            }
            _ => ("open_id", actor.user_id.clone()),
        }
    }

    async fn receive_target(
        &self,
        actor: &ActorIdentity,
    ) -> anyhow::Result<(&'static str, String)> {
        match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => {
                let chat_id = scope.strip_prefix("chat_").unwrap_or(scope).to_string();
                Ok(("chat_id", chat_id))
            }
            _ => {
                if let Some(open_id) = self.resolve_direct_open_id().await? {
                    Ok(("open_id", open_id))
                } else {
                    Ok(("open_id", actor.user_id.clone()))
                }
            }
        }
    }

    async fn resolve_direct_open_id(&self) -> anyhow::Result<Option<String>> {
        let Some(contact) = &self.direct_contact else {
            return Ok(None);
        };
        if let Some(cached) = self.direct_open_id_cache.read().await.clone() {
            return Ok(Some(cached));
        }
        let token = self.token().await?;
        let body = match contact.kind {
            FeishuDirectContactKind::Email => serde_json::json!({ "emails": [&contact.value] }),
            FeishuDirectContactKind::Mobile => serde_json::json!({ "mobiles": [&contact.value] }),
        };
        let resp = self
            .client
            .post("https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id")
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("feishu resolve direct contact HTTP {status}: {detail}");
        }
        let parsed: BatchGetIdResp = resp.json().await?;
        if parsed.code != 0 {
            anyhow::bail!(
                "feishu resolve direct contact error code={} msg={}",
                parsed.code,
                parsed.msg
            );
        }
        let open_id = first_batch_get_open_id(parsed.data)
            .ok_or_else(|| anyhow::anyhow!("feishu direct contact did not resolve to open_id"))?;
        *self.direct_open_id_cache.write().await = Some(open_id.clone());
        Ok(Some(open_id))
    }
}

#[derive(Deserialize)]
struct BatchGetIdResp {
    code: i64,
    msg: String,
    data: Option<Value>,
}

fn single_direct_contact(
    allow_emails: &[String],
    allow_mobiles: &[String],
) -> Option<FeishuDirectContact> {
    let emails: Vec<_> = allow_emails
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "*")
        .collect();
    let mobiles: Vec<_> = allow_mobiles
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "*")
        .collect();
    match (emails.as_slice(), mobiles.as_slice()) {
        ([email], []) => Some(FeishuDirectContact {
            kind: FeishuDirectContactKind::Email,
            value: (*email).to_string(),
        }),
        ([], [mobile]) => Some(FeishuDirectContact {
            kind: FeishuDirectContactKind::Mobile,
            value: (*mobile).to_string(),
        }),
        _ => None,
    }
}

fn first_batch_get_open_id(data: Option<Value>) -> Option<String> {
    data.and_then(|data| data.get("user_list").cloned())
        .and_then(|value| value.as_array().cloned())
        .and_then(|list| {
            list.into_iter().next().and_then(|entry| {
                entry
                    .get("user_id")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
            })
        })
}

impl FeishuSink {
    async fn post_message(
        &self,
        actor: &ActorIdentity,
        msg_type: &str,
        content: String,
    ) -> anyhow::Result<()> {
        let token = self.token().await?;
        let (receive_id_type, receive_id) = self.receive_target(actor).await?;
        let resp = self
            .client
            .post(format!(
                "{FEISHU_SEND_URL}?receive_id_type={receive_id_type}"
            ))
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "receive_id": receive_id,
                "msg_type": msg_type,
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
}

#[async_trait]
impl OutboundSink for FeishuSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        let (msg_type, content) = if is_feishu_post_content(body) {
            ("post", body.to_string())
        } else {
            ("text", serde_json::json!({ "text": body }).to_string())
        };
        self.post_message(actor, msg_type, content).await
    }

    fn format(&self) -> RenderFormat {
        RenderFormat::FeishuPost
    }

    /// digest 走 interactive card 富文本路径 —— 三色 header + bucket 化 markdown
    /// 块,链接走原生 markdown 锚文本。`fallback_body` 仅在卡片构造失败兜底,
    /// 当前实现不会失败。
    async fn send_digest(
        &self,
        actor: &ActorIdentity,
        payload: &DigestPayload,
        _fallback_body: &str,
    ) -> anyhow::Result<()> {
        let card = build_feishu_card(payload);
        self.post_message(actor, "interactive", card.to_string())
            .await
    }
}

fn is_feishu_post_content(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("zh_cn").cloned())
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receive_target_direct_uses_open_id() {
        let actor = ActorIdentity::new("feishu", "ou_abc", None::<String>).unwrap();
        let (ty, id) = FeishuSink::static_receive_target(&actor);
        assert_eq!(ty, "open_id");
        assert_eq!(id, "ou_abc");
    }

    #[test]
    fn receive_target_group_strips_chat_prefix() {
        let actor = ActorIdentity::new("feishu", "ou_abc", Some("chat_oc_123")).unwrap();
        let (ty, id) = FeishuSink::static_receive_target(&actor);
        assert_eq!(ty, "chat_id");
        assert_eq!(id, "oc_123");
    }

    #[test]
    fn direct_contact_fallback_only_uses_single_stable_contact() {
        assert_eq!(
            single_direct_contact(&["alice@example.com".to_string()], &[]),
            Some(FeishuDirectContact {
                kind: FeishuDirectContactKind::Email,
                value: "alice@example.com".to_string(),
            })
        );
        assert_eq!(
            single_direct_contact(&[], &["+8613800138000".to_string()]),
            Some(FeishuDirectContact {
                kind: FeishuDirectContactKind::Mobile,
                value: "+8613800138000".to_string(),
            })
        );
        assert_eq!(
            single_direct_contact(
                &["alice@example.com".to_string()],
                &["+8613800138000".to_string()]
            ),
            None
        );
        assert_eq!(
            single_direct_contact(
                &[
                    "alice@example.com".to_string(),
                    "bob@example.com".to_string()
                ],
                &[]
            ),
            None
        );
    }

    #[test]
    fn first_batch_get_open_id_extracts_user_id() {
        let data = serde_json::json!({
            "user_list": [
                { "user_id": "ou_current" }
            ]
        });
        assert_eq!(
            first_batch_get_open_id(Some(data)).as_deref(),
            Some("ou_current")
        );
    }

    #[test]
    fn detects_post_content_payload() {
        assert!(is_feishu_post_content(
            r#"{"zh_cn":{"title":"t","content":[]}}"#
        ));
        assert!(!is_feishu_post_content("plain text"));
    }
}
