//! `OutboundSink` 协议 + 默认 `LogSink` 实现 + 两个全 module 都用得着的字符串
//! 工具(`actor_key` / `body_preview`)。
//!
//! 把这些放一个文件的理由:它们是 router 的「**最外层 IO 抽象**」——任何关心
//! 「事件最终怎么发出去 / 落日志时怎么标 actor」的代码都需要从这里 use,
//! 单独成文件能让其他 sibling module 的 import 行简短。

use async_trait::async_trait;
use hone_core::ActorIdentity;
use tracing::info;

use crate::renderer::RenderFormat;

#[async_trait]
pub trait OutboundSink: Send + Sync {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()>;

    /// 成功送达后写入 delivery_log 的 status。真实 sink 返回 `sent`;LogSink
    /// fallback 返回 `dryrun`,避免"渠道没注册被打到 fallback"的事件被
    /// `count_high_sent_since` 等查询当成真实 ack 计数。
    fn success_status(&self) -> &'static str {
        "sent"
    }

    /// 该 Sink 期望的消息格式。若渠道使用富文本,override 这里同时在 send()
    /// 带上对应的 parse_mode / msg_type,否则会出现 `<b>` 当字面量泄露。
    fn format(&self) -> RenderFormat {
        RenderFormat::Plain
    }

    /// MultiChannelSink 这类按 actor.channel 分发的 sink 需要按目标渠道选择格式。
    fn format_for(&self, _actor: &ActorIdentity) -> RenderFormat {
        self.format()
    }
}

/// 默认 Sink:把渲染后的消息写 tracing::info,用作 `MultiChannelSink` 在
/// 用户的 actor.channel 没注册到真 sink 时的 fallback,以及测试。落 delivery_log
/// 时 status 标 `dryrun`,与真实 `sent` 区分。
pub struct LogSink;

#[async_trait]
impl OutboundSink for LogSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        info!(
            actor = %actor_key(actor),
            "[dryrun sink] {body}"
        );
        Ok(())
    }

    fn success_status(&self) -> &'static str {
        "dryrun"
    }
}

/// 把 `ActorIdentity` 拉成统一的 `channel::scope::user_id` key,既给 tracing 标识
/// 用,也作为 `delivery_log` 表里的 `actor` 列写入 —— 跨表 grep 一致。
pub(crate) fn actor_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

/// 取 body 头 120 字符做 tracing 预览,换行折成单行 ⏎,避免日志多行难抓。
/// 全文一律已经在 SQLite `delivery_log` 里,这里只是肉眼速读用。
pub(crate) fn body_preview(body: &str) -> String {
    let mut s: String = body.chars().take(120).collect();
    if body.chars().count() > 120 {
        s.push('…');
    }
    s.replace('\n', " ⏎ ")
}
