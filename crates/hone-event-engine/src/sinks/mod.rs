//! 生产用 OutboundSink 实现,按渠道各一个。
//!
//! 每个 sink 直接打渠道上游 API(Telegram / Feishu / Discord 走 HTTP;iMessage
//! 走本机 osascript)。`MultiChannelSink` 做 `ActorIdentity::channel` → sink 的
//! 派发;未注册的渠道 fall back 到 `LogSink` 语义,方便调试但不会静默丢数据。
//!
//! 与 `bins/hone-*/src/outbound.rs` 的差别:这里 **只发送 engine 自己组装的摘要
//! 文本**,不做分段 / 占位符 / 线程拼接;那些复杂度属于对话侧,engine 的 sink
//! 只需"把已经渲染好的一段文字投到对端"即可。

pub mod discord;
pub mod discord_embed;
pub mod feishu;
pub mod feishu_card;
pub mod imessage;
pub mod multi;
pub mod telegram;

pub use discord::DiscordSink;
pub use feishu::FeishuSink;
pub use imessage::IMessageSink;
pub use multi::MultiChannelSink;
pub use telegram::TelegramSink;
