//! MultiChannelSink —— 按 `ActorIdentity::channel` 派发到具体 sink。
//!
//! 典型组装:
//! ```ignore
//! let sink = MultiChannelSink::new(Arc::new(LogSink))
//!     .with_channel("telegram", Arc::new(TelegramSink::new(token)))
//!     .with_channel("feishu", Arc::new(FeishuSink::new(app_id, app_secret)))
//!     ...;
//! engine = engine.with_sink(Arc::new(sink));
//! ```
//!
//! 未注册的渠道会走 fallback(默认传入的 `LogSink`),这样新增渠道或渠道暂时
//! 下线时 engine 不会失败,只是在日志里留下记录。

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use hone_core::ActorIdentity;
use tracing::{debug, warn};

use crate::renderer::RenderFormat;
use crate::router::{LogSink, OutboundSink};

pub struct MultiChannelSink {
    sinks: HashMap<String, Arc<dyn OutboundSink>>,
    fallback: Arc<dyn OutboundSink>,
}

impl MultiChannelSink {
    pub fn new(fallback: Arc<dyn OutboundSink>) -> Self {
        Self {
            sinks: HashMap::new(),
            fallback,
        }
    }

    /// 默认 fallback = LogSink,便捷构造。
    pub fn with_log_fallback() -> Self {
        Self::new(Arc::new(LogSink))
    }

    pub fn with_channel(
        mut self,
        channel: impl Into<String>,
        sink: Arc<dyn OutboundSink>,
    ) -> Self {
        self.sinks.insert(channel.into(), sink);
        self
    }

    pub fn channels_registered(&self) -> Vec<String> {
        let mut v: Vec<String> = self.sinks.keys().cloned().collect();
        v.sort();
        v
    }
}

#[async_trait]
impl OutboundSink for MultiChannelSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        if let Some(s) = self.sinks.get(&actor.channel) {
            match s.send(actor, body).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    warn!(
                        channel = %actor.channel,
                        user = %actor.user_id,
                        "channel sink failed, falling back to log: {e:#}"
                    );
                    self.fallback.send(actor, body).await
                }
            }
        } else {
            debug!(
                channel = %actor.channel,
                "no sink registered for channel; dispatching to fallback"
            );
            self.fallback.send(actor, body).await
        }
    }

    fn format(&self) -> RenderFormat {
        // 所有 sink 统一 Plain,避免 renderer 需要知道具体 actor → sink 的映射。
        RenderFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Spy(Mutex<Vec<String>>);
    #[async_trait]
    impl OutboundSink for Spy {
        async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.0.lock().unwrap().push(format!("{}:{}", actor.channel, body));
            Ok(())
        }
    }

    struct Failing;
    #[async_trait]
    impl OutboundSink for Failing {
        async fn send(&self, _actor: &ActorIdentity, _body: &str) -> anyhow::Result<()> {
            anyhow::bail!("boom")
        }
    }

    #[tokio::test]
    async fn routes_to_registered_channel() {
        let tg = Arc::new(Spy(Mutex::new(Vec::new())));
        let fb = Arc::new(Spy(Mutex::new(Vec::new())));
        let sink = MultiChannelSink::new(fb.clone()).with_channel("telegram", tg.clone());
        let actor = ActorIdentity::new("telegram", "u1", None::<String>).unwrap();
        sink.send(&actor, "hi").await.unwrap();
        assert_eq!(tg.0.lock().unwrap().len(), 1);
        assert!(fb.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn falls_back_when_channel_unregistered() {
        let fb = Arc::new(Spy(Mutex::new(Vec::new())));
        let sink = MultiChannelSink::new(fb.clone());
        let actor = ActorIdentity::new("telegram", "u1", None::<String>).unwrap();
        sink.send(&actor, "hi").await.unwrap();
        assert_eq!(fb.0.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn falls_back_when_channel_sink_errors() {
        let fb = Arc::new(Spy(Mutex::new(Vec::new())));
        let sink = MultiChannelSink::new(fb.clone()).with_channel("feishu", Arc::new(Failing));
        let actor = ActorIdentity::new("feishu", "u1", None::<String>).unwrap();
        sink.send(&actor, "hi").await.unwrap();
        assert_eq!(fb.0.lock().unwrap().len(), 1, "errored channel falls back");
    }
}
