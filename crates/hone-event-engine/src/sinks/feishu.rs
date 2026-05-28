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

use std::collections::{BTreeSet, HashMap};
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
use crate::sinks::http_error::{
    format_provider_api_error, format_transport_error, format_upstream_http_error,
};

const FEISHU_TOKEN_URL: &str =
    "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
const FEISHU_SEND_URL: &str = "https://open.feishu.cn/open-apis/im/v1/messages";

pub struct FeishuSink {
    app_id: String,
    app_secret: String,
    client: reqwest::Client,
    token_cache: Arc<RwLock<Option<(String, Instant)>>>,
    direct_contacts: Option<FeishuDirectContacts>,
    direct_actor_contacts: HashMap<String, FeishuDirectContacts>,
    direct_open_id_cache: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FeishuDirectContacts {
    emails: Vec<String>,
    mobiles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedFeishuTarget {
    receive_id_type: &'static str,
    receive_id: String,
    direct_open_id_cache_key: Option<String>,
}

#[derive(Debug)]
struct FeishuSendError {
    message: String,
    open_id_cross_app: bool,
}

impl FeishuSendError {
    fn new(message: impl Into<String>, open_id_cross_app: bool) -> Self {
        Self {
            message: message.into(),
            open_id_cross_app,
        }
    }
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
            direct_contacts: None,
            direct_actor_contacts: HashMap::new(),
            direct_open_id_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_single_direct_contact_fallback(
        mut self,
        allow_emails: &[String],
        allow_mobiles: &[String],
    ) -> Self {
        self.direct_contacts = stable_direct_contacts(allow_emails, allow_mobiles);
        self
    }

    pub fn with_direct_actor_contact_targets<I, A, T>(mut self, targets: I) -> Self
    where
        I: IntoIterator<Item = (A, T)>,
        A: Into<String>,
        T: Into<String>,
    {
        let mut contacts_by_actor: HashMap<String, FeishuDirectContacts> = HashMap::new();
        for (actor_user_id, target) in targets {
            let actor_user_id = actor_user_id.into().trim().to_string();
            if actor_user_id.is_empty() {
                continue;
            }
            let Some(contacts) = direct_contact_from_target(&target.into()) else {
                continue;
            };
            let entry =
                contacts_by_actor
                    .entry(actor_user_id)
                    .or_insert_with(|| FeishuDirectContacts {
                        emails: Vec::new(),
                        mobiles: Vec::new(),
                    });
            entry.emails.extend(contacts.emails);
            entry.mobiles.extend(contacts.mobiles);
            entry.emails.sort();
            entry.emails.dedup();
            entry.mobiles.sort();
            entry.mobiles.dedup();
        }
        self.direct_actor_contacts = contacts_by_actor;
        self
    }

    async fn token(&self) -> anyhow::Result<String> {
        {
            let cache = self.token_cache.read().await;
            if let Some((t, exp)) = &*cache
                && Instant::now() < *exp
            {
                return Ok(t.clone());
            }
        }
        let response = self
            .client
            .post(FEISHU_TOKEN_URL)
            .json(&serde_json::json!({
                "app_id": &self.app_id,
                "app_secret": &self.app_secret,
            }))
            .send()
            .await
            .map_err(|err| anyhow::anyhow!(format_transport_error("feishu", "token", &err)))?;
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            anyhow::bail!(format_upstream_http_error(
                "feishu", "token", status, &detail
            ));
        }
        let parsed: TokenResp = response.json().await?;
        if parsed.code != 0 {
            anyhow::bail!(format_provider_api_error(
                "feishu",
                "token",
                parsed.code,
                &parsed.msg
            ));
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

    async fn receive_target(&self, actor: &ActorIdentity) -> anyhow::Result<ResolvedFeishuTarget> {
        match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => {
                let chat_id = scope.strip_prefix("chat_").unwrap_or(scope).to_string();
                Ok(ResolvedFeishuTarget {
                    receive_id_type: "chat_id",
                    receive_id: chat_id,
                    direct_open_id_cache_key: None,
                })
            }
            _ => {
                if let Some(contacts) = self.direct_actor_contacts.get(actor.user_id.trim()) {
                    let cache_key = format!("actor:{}", actor.user_id.trim());
                    if let Some(open_id) = self
                        .resolve_direct_open_id_for_contacts(&cache_key, contacts)
                        .await?
                    {
                        return Ok(ResolvedFeishuTarget {
                            receive_id_type: "open_id",
                            receive_id: open_id,
                            direct_open_id_cache_key: Some(cache_key),
                        });
                    }
                }
                if let Some(contacts) = &self.direct_contacts {
                    let cache_key = "config:single_direct_contact";
                    if let Some(open_id) = self
                        .resolve_direct_open_id_for_contacts(cache_key, contacts)
                        .await?
                    {
                        return Ok(ResolvedFeishuTarget {
                            receive_id_type: "open_id",
                            receive_id: open_id,
                            direct_open_id_cache_key: Some(cache_key.to_string()),
                        });
                    }
                }
                Ok(ResolvedFeishuTarget {
                    receive_id_type: "open_id",
                    receive_id: actor.user_id.clone(),
                    direct_open_id_cache_key: None,
                })
            }
        }
    }

    async fn clear_cached_direct_open_id(&self, cache_key: &str) {
        self.direct_open_id_cache.write().await.remove(cache_key);
    }

    async fn resolve_direct_open_id_for_contacts(
        &self,
        cache_key: &str,
        contacts: &FeishuDirectContacts,
    ) -> anyhow::Result<Option<String>> {
        if let Some(cached) = self
            .direct_open_id_cache
            .read()
            .await
            .get(cache_key)
            .cloned()
        {
            return Ok(Some(cached));
        }
        let token = self.token().await?;
        let mut body = serde_json::Map::new();
        if !contacts.emails.is_empty() {
            body.insert("emails".to_string(), serde_json::json!(contacts.emails));
        }
        if !contacts.mobiles.is_empty() {
            body.insert("mobiles".to_string(), serde_json::json!(contacts.mobiles));
        }
        let response = self
            .client
            .post("https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id")
            .bearer_auth(&token)
            .json(&Value::Object(body))
            .send()
            .await
            .map_err(|err| {
                anyhow::anyhow!(format_transport_error(
                    "feishu",
                    "resolve direct contact",
                    &err
                ))
            })?;
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            anyhow::bail!(format_upstream_http_error(
                "feishu",
                "resolve direct contact",
                status,
                &detail
            ));
        }
        let parsed: BatchGetIdResp = response.json().await?;
        if parsed.code != 0 {
            anyhow::bail!(format_provider_api_error(
                "feishu",
                "resolve direct contact",
                parsed.code,
                &parsed.msg
            ));
        }
        let Some(open_id) = unique_batch_get_open_id(parsed.data) else {
            return Ok(None);
        };
        self.direct_open_id_cache
            .write()
            .await
            .insert(cache_key.to_string(), open_id.clone());
        Ok(Some(open_id))
    }
}

#[derive(Deserialize)]
struct BatchGetIdResp {
    code: i64,
    msg: String,
    data: Option<Value>,
}

fn stable_direct_contacts(
    allow_emails: &[String],
    allow_mobiles: &[String],
) -> Option<FeishuDirectContacts> {
    let emails: Vec<_> = allow_emails
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "*")
        .map(str::to_string)
        .collect();
    let mobiles: Vec<_> = allow_mobiles
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "*")
        .map(str::to_string)
        .collect();
    if emails.is_empty() && mobiles.is_empty() {
        None
    } else {
        Some(FeishuDirectContacts { emails, mobiles })
    }
}

fn direct_contact_from_target(target: &str) -> Option<FeishuDirectContacts> {
    let target = target.trim();
    if target.is_empty() || target == "*" {
        return None;
    }
    if target.contains('@') {
        return Some(FeishuDirectContacts {
            emails: vec![target.to_string()],
            mobiles: Vec::new(),
        });
    }
    if looks_like_mobile(target) {
        return Some(FeishuDirectContacts {
            emails: Vec::new(),
            mobiles: vec![target.to_string()],
        });
    }
    None
}

fn looks_like_mobile(target: &str) -> bool {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '+' | ' ' | '-' | '(' | ')'))
    {
        return false;
    }
    trimmed.chars().filter(|ch| ch.is_ascii_digit()).count() >= 7
}

fn unique_batch_get_open_id(data: Option<Value>) -> Option<String> {
    let ids: BTreeSet<String> = data
        .and_then(|data| data.get("user_list").cloned())
        .and_then(|value| value.as_array().cloned())
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            entry
                .get("user_id")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .collect();
    if ids.len() == 1 {
        ids.into_iter().next()
    } else {
        None
    }
}

impl FeishuSink {
    async fn post_message(
        &self,
        actor: &ActorIdentity,
        msg_type: &str,
        content: String,
    ) -> anyhow::Result<()> {
        let mut last_error: Option<FeishuSendError> = None;
        for attempt in 0..2 {
            let target = self.receive_target(actor).await?;
            match self
                .post_message_to_target(&target, msg_type, content.clone())
                .await
            {
                Ok(()) => return Ok(()),
                Err(err)
                    if attempt == 0
                        && err.open_id_cross_app
                        && target.direct_open_id_cache_key.is_some() =>
                {
                    if let Some(cache_key) = target.direct_open_id_cache_key.as_deref() {
                        self.clear_cached_direct_open_id(cache_key).await;
                    }
                    last_error = Some(err);
                }
                Err(err) => {
                    anyhow::bail!(err.message);
                }
            }
        }
        if let Some(err) = last_error {
            anyhow::bail!(err.message);
        }
        Ok(())
    }

    async fn post_message_to_target(
        &self,
        target: &ResolvedFeishuTarget,
        msg_type: &str,
        content: String,
    ) -> Result<(), FeishuSendError> {
        let token = self
            .token()
            .await
            .map_err(|err| FeishuSendError::new(err.to_string(), false))?;
        let response = self
            .client
            .post(format!(
                "{FEISHU_SEND_URL}?receive_id_type={}",
                target.receive_id_type
            ))
            .bearer_auth(&token)
            .json(&serde_json::json!({
                "receive_id": target.receive_id,
                "msg_type": msg_type,
                "content": content,
            }))
            .send()
            .await
            .map_err(|err| {
                FeishuSendError::new(format_transport_error("feishu", "send", &err), false)
            })?;
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            return Err(FeishuSendError::new(
                format_upstream_http_error("feishu", "send", status, &detail),
                feishu_error_is_open_id_cross_app(&detail),
            ));
        }
        let parsed: SendResp = response.json().await.map_err(|err| {
            FeishuSendError::new(format!("feishu send parse error: {err}"), false)
        })?;
        if parsed.code != 0 {
            return Err(FeishuSendError::new(
                format_provider_api_error("feishu", "send", parsed.code, &parsed.msg),
                parsed.code == 99992361 || feishu_error_is_open_id_cross_app(&parsed.msg),
            ));
        }
        Ok(())
    }
}

fn feishu_error_is_open_id_cross_app(detail: &str) -> bool {
    let lower = detail.to_ascii_lowercase();
    lower.contains("99992361") || lower.contains("open_id cross app")
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
    fn direct_contact_fallback_uses_stable_contacts() {
        assert_eq!(
            stable_direct_contacts(&["alice@example.com".to_string()], &[]),
            Some(FeishuDirectContacts {
                emails: vec!["alice@example.com".to_string()],
                mobiles: vec![],
            })
        );
        assert_eq!(
            stable_direct_contacts(&[], &["+8613800138000".to_string()]),
            Some(FeishuDirectContacts {
                emails: vec![],
                mobiles: vec!["+8613800138000".to_string()],
            })
        );
        assert_eq!(
            stable_direct_contacts(
                &["alice@example.com".to_string()],
                &["+8613800138000".to_string()]
            ),
            Some(FeishuDirectContacts {
                emails: vec!["alice@example.com".to_string()],
                mobiles: vec!["+8613800138000".to_string()],
            })
        );
        assert_eq!(
            stable_direct_contacts(
                &[
                    "alice@example.com".to_string(),
                    "bob@example.com".to_string()
                ],
                &[]
            ),
            Some(FeishuDirectContacts {
                emails: vec![
                    "alice@example.com".to_string(),
                    "bob@example.com".to_string()
                ],
                mobiles: vec![],
            })
        );
        assert_eq!(stable_direct_contacts(&["*".to_string()], &[]), None);
    }

    #[test]
    fn direct_actor_contact_targets_keep_only_resolvable_contacts() {
        let sink = FeishuSink::new("app", "secret").with_direct_actor_contact_targets(vec![
            ("ou_email", "alice@example.com"),
            ("ou_both", "alice@example.com"),
            ("ou_both", "+8613800138000"),
            ("ou_mobile", "+8613800138000"),
            ("ou_open", "ou_stale"),
            ("", "bob@example.com"),
        ]);
        assert_eq!(
            sink.direct_actor_contacts.get("ou_email"),
            Some(&FeishuDirectContacts {
                emails: vec!["alice@example.com".to_string()],
                mobiles: vec![],
            })
        );
        assert_eq!(
            sink.direct_actor_contacts.get("ou_mobile"),
            Some(&FeishuDirectContacts {
                emails: vec![],
                mobiles: vec!["+8613800138000".to_string()],
            })
        );
        assert_eq!(
            sink.direct_actor_contacts.get("ou_both"),
            Some(&FeishuDirectContacts {
                emails: vec!["alice@example.com".to_string()],
                mobiles: vec!["+8613800138000".to_string()],
            })
        );
        assert!(!sink.direct_actor_contacts.contains_key("ou_open"));
        assert!(!sink.direct_actor_contacts.contains_key(""));
    }

    #[tokio::test]
    async fn receive_target_prefers_actor_contact_cache() {
        let sink = FeishuSink::new("app", "secret")
            .with_direct_actor_contact_targets(vec![("ou_stale", "+8613800138000")]);
        sink.direct_open_id_cache
            .write()
            .await
            .insert("actor:ou_stale".to_string(), "ou_current".to_string());
        let actor = ActorIdentity::new("feishu", "ou_stale", None::<String>).unwrap();

        let target = sink.receive_target(&actor).await.unwrap();

        assert_eq!(target.receive_id_type, "open_id");
        assert_eq!(target.receive_id, "ou_current");
        assert_eq!(
            target.direct_open_id_cache_key.as_deref(),
            Some("actor:ou_stale")
        );
    }

    #[tokio::test]
    async fn open_id_cross_app_cache_can_be_invalidated_for_retry() {
        let sink = FeishuSink::new("app", "secret")
            .with_direct_actor_contact_targets(vec![("ou_stale", "+8613800138000")]);
        sink.direct_open_id_cache
            .write()
            .await
            .insert("actor:ou_stale".to_string(), "ou_old_app".to_string());

        sink.clear_cached_direct_open_id("actor:ou_stale").await;

        assert!(
            !sink
                .direct_open_id_cache
                .read()
                .await
                .contains_key("actor:ou_stale")
        );
        assert!(feishu_error_is_open_id_cross_app(
            r#"{"code":99992361,"msg":"open_id cross app"}"#
        ));
    }

    #[test]
    fn unique_batch_get_open_id_extracts_single_user_id() {
        let data = serde_json::json!({
            "user_list": [
                { "user_id": "ou_current" },
                { "user_id": "ou_current" }
            ]
        });
        assert_eq!(
            unique_batch_get_open_id(Some(data)).as_deref(),
            Some("ou_current")
        );
    }

    #[test]
    fn unique_batch_get_open_id_rejects_ambiguous_users() {
        let data = serde_json::json!({
            "user_list": [
                { "user_id": "ou_a" },
                { "user_id": "ou_b" }
            ]
        });
        assert_eq!(unique_batch_get_open_id(Some(data)), None);
    }

    #[test]
    fn detects_post_content_payload() {
        assert!(is_feishu_post_content(
            r#"{"zh_cn":{"title":"t","content":[]}}"#
        ));
        assert!(!is_feishu_post_content("plain text"));
    }
}
