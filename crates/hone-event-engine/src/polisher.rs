//! BodyPolisher — 消息润色扩展点。
//!
//! 路由器在分发 High severity 事件时，若配置了 `llm_polish_for` 且包含该事件的
//! severity，则调用 `polish()`，用返回文本替代默认模板。Polisher 返回 `None`
//! 表示"保持原文"（例如 LLM 调用失败），路由器照常发送默认模板。
//!
//! MVP 提供两种实现：
//! - `NoopPolisher`：什么都不做，始终返回 `None`
//! - `LlmPolisher`：调用 hone-llm 的 `LlmProvider` 做短提示润色

use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;

use hone_llm::{LlmProvider, Message};

use crate::event::{MarketEvent, Severity};

#[async_trait]
pub trait BodyPolisher: Send + Sync {
    /// 对 `default_body`（模板渲染结果）进行润色；返回 `None` 表示不改。
    async fn polish(&self, event: &MarketEvent, default_body: &str) -> Option<String>;
}

/// 默认 polisher：始终返回 None。
pub struct NoopPolisher;

#[async_trait]
impl BodyPolisher for NoopPolisher {
    async fn polish(&self, _event: &MarketEvent, _default_body: &str) -> Option<String> {
        None
    }
}

/// 基于 `LlmProvider` 的 polisher。
///
/// - `polish_levels`：只对这些 severity 做润色；空集合等同 `NoopPolisher`。
/// - `model`：可选模型覆盖；`None` 时走 provider 默认。
pub struct LlmPolisher {
    provider: Arc<dyn LlmProvider>,
    polish_levels: HashSet<Severity>,
    model: Option<String>,
}

impl LlmPolisher {
    pub fn new(provider: Arc<dyn LlmProvider>, polish_levels: HashSet<Severity>) -> Self {
        Self {
            provider,
            polish_levels,
            model: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        let m = model.into();
        self.model = if m.trim().is_empty() { None } else { Some(m) };
        self
    }

    fn build_prompt(event: &MarketEvent, default_body: &str) -> Vec<Message> {
        let system = Message {
            role: "system".into(),
            content: Some(
                "你是一个中文财经助理。对输入的市场事件默认渲染文本做一次简短润色：\n\
                 1) 保持事实不改；2) 输出不超过 140 字；3) 保留符号、标题的关键信息；\n\
                 4) 不做任何投资建议；5) 直接输出润色后的正文，不要添加前缀/后缀。"
                    .into(),
            ),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        let user_payload = serde_json::json!({
            "default_body": default_body,
            "symbols": event.symbols,
            "title": event.title,
            "summary": event.summary,
            "kind": event.kind,
            "severity": event.severity,
            "url": event.url,
        });
        let user = Message {
            role: "user".into(),
            content: Some(user_payload.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        vec![system, user]
    }
}

#[async_trait]
impl BodyPolisher for LlmPolisher {
    async fn polish(&self, event: &MarketEvent, default_body: &str) -> Option<String> {
        if !self.polish_levels.contains(&event.severity) {
            return None;
        }
        let messages = Self::build_prompt(event, default_body);
        match self
            .provider
            .chat(&messages, self.model.as_deref())
            .await
        {
            Ok(res) => {
                let t = res.content.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            }
            Err(e) => {
                tracing::warn!("llm polish failed, keep default body: {e:#}");
                None
            }
        }
    }
}

/// 把 config 里的字符串 severity 列表转换为 HashSet<Severity>。
/// 不识别的字符串被忽略（记一条 warn）。
pub fn parse_polish_levels(names: &[String]) -> HashSet<Severity> {
    let mut out = HashSet::new();
    for name in names {
        match name.trim().to_ascii_lowercase().as_str() {
            "low" => {
                out.insert(Severity::Low);
            }
            "medium" | "med" => {
                out.insert(Severity::Medium);
            }
            "high" => {
                out.insert(Severity::High);
            }
            other if !other.is_empty() => {
                tracing::warn!("unknown polish severity level: {other}");
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use chrono::Utc;
    use hone_llm::provider::ChatResult;
    use hone_llm::{ChatResponse, Message, ToolCall};
    use std::sync::Mutex;

    fn ev(sev: Severity) -> MarketEvent {
        MarketEvent {
            id: "e1".into(),
            kind: EventKind::EarningsReleased,
            severity: sev,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "earnings".into(),
            summary: "beat".into(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    struct FakeProvider {
        reply: String,
        calls: Mutex<u32>,
        fail: bool,
    }

    #[async_trait]
    impl LlmProvider for FakeProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResult> {
            *self.calls.lock().unwrap() += 1;
            if self.fail {
                Err(hone_core::HoneError::Llm("boom".into()))
            } else {
                Ok(ChatResult {
                    content: self.reply.clone(),
                    usage: None,
                })
            }
        }

        async fn chat_with_tools(
            &self,
            _m: &[Message],
            _t: &[serde_json::Value],
            _mo: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            unimplemented!("not used")
        }

        fn chat_stream<'a>(
            &'a self,
            _m: &'a [Message],
            _mo: Option<&'a str>,
        ) -> futures::stream::BoxStream<'a, hone_core::HoneResult<String>> {
            unimplemented!("not used")
        }
    }

    fn _unused_tool_call() -> ToolCall {
        unimplemented!("present to keep ToolCall imported when test only compiles")
    }

    #[tokio::test]
    async fn noop_always_returns_none() {
        let p = NoopPolisher;
        assert!(p.polish(&ev(Severity::High), "body").await.is_none());
    }

    #[tokio::test]
    async fn llm_polisher_only_acts_on_configured_levels() {
        let provider = Arc::new(FakeProvider {
            reply: "polished!".into(),
            calls: Mutex::new(0),
            fail: false,
        });
        let mut levels = HashSet::new();
        levels.insert(Severity::High);
        let p = LlmPolisher::new(provider.clone(), levels);

        let polished = p.polish(&ev(Severity::High), "default").await;
        assert_eq!(polished.as_deref(), Some("polished!"));
        assert_eq!(*provider.calls.lock().unwrap(), 1);

        // Medium 不在 level 集合内，不调用 LLM
        let medium = p.polish(&ev(Severity::Medium), "default").await;
        assert!(medium.is_none());
        assert_eq!(*provider.calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn llm_polisher_falls_back_to_none_on_error() {
        let provider = Arc::new(FakeProvider {
            reply: String::new(),
            calls: Mutex::new(0),
            fail: true,
        });
        let mut levels = HashSet::new();
        levels.insert(Severity::High);
        let p = LlmPolisher::new(provider, levels);
        assert!(p.polish(&ev(Severity::High), "default").await.is_none());
    }

    #[tokio::test]
    async fn llm_polisher_ignores_empty_reply() {
        let provider = Arc::new(FakeProvider {
            reply: "   \n".into(),
            calls: Mutex::new(0),
            fail: false,
        });
        let mut levels = HashSet::new();
        levels.insert(Severity::High);
        let p = LlmPolisher::new(provider, levels);
        assert!(p.polish(&ev(Severity::High), "default").await.is_none());
    }

    #[test]
    fn parse_polish_levels_handles_mixed_case_and_noise() {
        let set = parse_polish_levels(&[
            "High".into(),
            "MEDIUM".into(),
            "low".into(),
            "bogus".into(),
            "".into(),
        ]);
        assert!(set.contains(&Severity::High));
        assert!(set.contains(&Severity::Medium));
        assert!(set.contains(&Severity::Low));
        assert_eq!(set.len(), 3);
    }
}
