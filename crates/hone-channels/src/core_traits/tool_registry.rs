//! ToolRegistry 工厂 trait。
//!
//! 同一个 `HoneBotCore` 会为每次 run 单独构造一份 `ToolRegistry`,因为
//! tool 的权限过滤（管理员专属的 restart_hone、cron 是否允许注册、
//! sandbox 根目录等）都依赖具体 actor 身份。
//!
//! 抽成 trait 之后,测试可以注入固定的一份 registry 来绕开 skill
//! 目录扫描和 security config 校验。

use hone_core::ActorIdentity;
use hone_tools::ToolRegistry;

use crate::core::HoneBotCore;

/// 为一次运行构造一份独立的 [`ToolRegistry`]。
///
/// 参数意义：
/// - `actor`：本次 run 的发起者;`None` 代表 system / cli 用途,
///   部分工具（PortfolioTool、LocalReadFileTool 等）在 `None` 时不会注册
///   或改走 system fallback actor
/// - `channel_target`：渠道侧 target（如 Feishu 的 open_id、Telegram 的
///   chat_id）。只在注册 CronJobTool 时作为 fallback channel_target 使用
/// - `allow_cron`：是否允许注册 CronJobTool。群聊 / 无授权场景应关掉
pub trait ToolRegistryFactory: Send + Sync {
    fn create_tool_registry(
        &self,
        actor: Option<&ActorIdentity>,
        channel_target: &str,
        allow_cron: bool,
    ) -> ToolRegistry;
}

impl ToolRegistryFactory for HoneBotCore {
    fn create_tool_registry(
        &self,
        actor: Option<&ActorIdentity>,
        channel_target: &str,
        allow_cron: bool,
    ) -> ToolRegistry {
        HoneBotCore::create_tool_registry(self, actor, channel_target, allow_cron)
    }
}
