//! 社交源 pollers —— 非 FMP 的第三方监听链路。
//!
//! 与 `pollers/*` 下的 FMP-专用 poller 并列,但数据来自 HTML / 公开 API,
//! 事件统一为 `EventKind::SocialPost` + `payload.source_class = "uncertain"`,
//! 交由 router 的 LLM 仲裁链路决定是否升 Medium 即时推。

pub mod telegram_channel;
pub mod truth_social;

pub use telegram_channel::TelegramChannelPoller;
pub use truth_social::TruthSocialPoller;

const SOCIAL_TITLE_MAX_CHARS: usize = 240;
const SOCIAL_SUMMARY_MAX_CHARS: usize = 280;
