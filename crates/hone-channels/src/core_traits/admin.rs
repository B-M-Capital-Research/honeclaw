//! 管理员判定 + runtime 拦截命令 trait。
//!
//! 对应 `HoneBotCore` 里的 is_admin / is_admin_actor /
//! try_intercept_admin_registration / try_handle_intercept_command 四组逻辑。
//!
//! 内部会读 `config.admins` 白名单和 `runtime_admin_overrides` 动态集合,
//! 但 trait 只暴露「判定结果」和「拦截结果」,不泄露底层数据结构。
//! 这让测试 / 渠道层可以 mock 「某个 actor 是否是管理员」而不用构造完整
//! admin 配置。

use async_trait::async_trait;

use hone_core::ActorIdentity;

use crate::core::HoneBotCore;

/// 管理员权限判定 + `/register-admin` / `/report` 等 runtime 命令的拦截。
#[async_trait]
pub trait AdminIntercept: Send + Sync {
    /// 按渠道 + user_id 判定是否匹配静态 admin 白名单。
    /// 对 `channel == "cli"` 总是返回 true（本地 CLI 默认信任）。
    fn is_admin(&self, user_id: &str, channel: &str) -> bool;

    /// 在静态白名单之上叠加 runtime 动态授予的管理员集合（
    /// 通过 `/register-admin <passphrase>` 授权的 actor）。
    fn is_admin_actor(&self, actor: &ActorIdentity) -> bool;

    /// 尝试把输入解释为 `/register-admin <passphrase>`：
    /// - `None`：输入不是这条命令,继续走正常流程
    /// - `Some(reply)`：命令已被拦截,把 `reply` 回给用户（
    ///   成功 / 未在白名单 / 未配置口令 / 口令无效都是不同的 reply）
    fn try_intercept_admin_registration(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String>;

    /// 拦截所有 `/xxx` 开头的 runtime 内建命令（目前包括 register-admin 和
    /// report）。返回 `Some(reply)` 表示命令已被本地处理,调用方不应再进
    /// 入 AgentSession。
    async fn try_handle_intercept_command(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String>;
}

#[async_trait]
impl AdminIntercept for HoneBotCore {
    fn is_admin(&self, user_id: &str, channel: &str) -> bool {
        HoneBotCore::is_admin(self, user_id, channel)
    }

    fn is_admin_actor(&self, actor: &ActorIdentity) -> bool {
        HoneBotCore::is_admin_actor(self, actor)
    }

    fn try_intercept_admin_registration(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String> {
        HoneBotCore::try_intercept_admin_registration(self, actor, input)
    }

    async fn try_handle_intercept_command(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String> {
        HoneBotCore::try_handle_intercept_command(self, actor, input).await
    }
}
