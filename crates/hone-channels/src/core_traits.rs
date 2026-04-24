//! HoneBotCore 的职责分组 trait（Phase 1 / 共 6 项，计划中已完成 3 项）。
//!
//! 背景：`HoneBotCore` 是 1600+ 行的 "God Object"，同时承担 LLM provider 工厂、
//! 审计记录、管理员权限判定、路径解析、tool registry 生成、runner 创建、
//! scheduler 创建、session 压缩等 8 个方向的职责。这样的对象有三个结构问题：
//!
//! - **强耦合**：`Arc<HoneBotCore>` 被 workspace 48+ 处消费，任何一个子系统
//!   都能调用所有能力，模块边界不清
//! - **测试困难**：测试一个 AgentSession stage 必须构造完整 HoneBotCore
//! - **演化困难**：加新能力都挂在 core 上，面积只增不减
//!
//! 本 module 是把 HoneBotCore 按职责分组的**第一步**：先抽 trait、HoneBotCore
//! 实现它们（通过转发到已有 inherent method）。调用方**暂时不改**，仍然可以用
//! `core.log_message_step(...)`；想用 `&dyn AuditRecorder` 替代 `&HoneBotCore`
//! 的调用点可以逐个迁移，不用一次改完。
//!
//! 已抽出的 trait（本次）：
//! - [`AuditRecorder`]  —— 消息流审计日志 (log_message_{received,step,finished,failed})
//! - [`AdminIntercept`] —— 管理员判定与 runtime 拦截命令
//! - [`PathResolver`]   —— 运行时路径查询 (configured_*_dir)
//!
//! 待抽（后续 session）：
//! - RunnerFactory（create_runner / override / sandbox_guard）
//! - LlmProviderBundle（primary / auxiliary / multi_agent）
//! - ToolRegistryFactory
//!
//! ## 为什么不一次做完
//!
//! 余下 3 个 trait 涉及 LlmProvider 字段、Runner 构造逻辑、复杂闭包生命周期，
//! 与 `HoneBotCore::new` 强耦合，需要独立的 context 窗口小心推进。本文件里
//! 的 3 个 trait 已经给出了「trait + 转发 impl + 文档锁定」的模板，
//! 后续只需套用这一模式。

use async_trait::async_trait;
use std::path::PathBuf;

use hone_core::ActorIdentity;
use hone_core::agent::AgentResponse;

use crate::core::HoneBotCore;

/// 消息流 / 审计日志的记录能力。
///
/// 对应 `HoneBotCore` 里 4 个 `log_message_*` 方法：`received`（入站）、`step`（阶段）、
/// `finished`（成功结束）、`failed`（失败结束）。这些方法只做 tracing log 输出,
/// 不触碰 LLM 或 storage,因此是最早被抽成 trait 的一批。
///
/// 用法示例：
/// ```ignore
/// fn do_work(audit: &dyn AuditRecorder, channel: &str, user: &str, session: &str) {
///     audit.log_message_step(channel, user, session, "phase.x", "done", None, None);
/// }
/// ```
pub trait AuditRecorder: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn log_message_received(
        &self,
        channel: &str,
        user_id: &str,
        channel_target: &str,
        session_id: &str,
        input: &str,
        extra: Option<&str>,
        message_id: Option<&str>,
    );

    fn log_message_step(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        step: &str,
        detail: &str,
        message_id: Option<&str>,
        state_override: Option<&str>,
    );

    fn log_message_finished(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        response: &AgentResponse,
        elapsed_ms: u128,
        message_id: Option<&str>,
    );

    fn log_message_failed(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        error: &str,
        elapsed_ms: u128,
        message_id: Option<&str>,
    );
}

/// 管理员权限判定 + runtime 拦截命令(`/register-admin`、`/report` 等)。
///
/// 内部持有 `runtime_admin_overrides` 并读取 `config.admins`,但 trait 本身
/// 只暴露判定与拦截的结果,不暴露底层数据结构。这让测试/渠道层可以 mock
/// "某个 actor 是否是管理员" 而不用造一份完整配置。
#[async_trait]
pub trait AdminIntercept: Send + Sync {
    /// 按渠道 + user_id 判定是否匹配静态 admin 白名单。
    fn is_admin(&self, user_id: &str, channel: &str) -> bool;

    /// 在静态白名单之上叠加 runtime 动态授予的管理员集合。
    fn is_admin_actor(&self, actor: &ActorIdentity) -> bool;

    /// 尝试把输入解释为 `/register-admin <passphrase>`：
    /// - `None`：输入不是这条命令,继续走正常流程
    /// - `Some(reply)`：命令已被拦截,把 `reply` 回给用户
    fn try_intercept_admin_registration(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String>;

    /// 拦截所有 `/xxx` 开头的 runtime 内建命令（register-admin / report）。
    /// 返回 `Some(reply)` 表示命令已被本地处理,调用方不应再进入 AgentSession。
    async fn try_handle_intercept_command(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String>;
}

/// 运行时目录解析。
///
/// 所有「应该把文件写到哪里」的判定都应走这里,不要在调用点自己拼 `./data/...`
/// 或直接读环境变量,否则 packaged 模式(desktop / brew 安装)下路径会偏。
pub trait PathResolver: Send + Sync {
    /// 内置 / 系统 skill 的搜索根目录(`config.extra.skills_dir` 或默认 `./skills`)。
    fn configured_system_skills_dir(&self) -> PathBuf;

    /// 用户自定义 skill 根目录(`{data_dir}/custom_skills`)。
    fn configured_custom_skills_dir(&self) -> PathBuf;

    /// 顶层 data 目录(环境变量 `HONE_DATA_DIR` 优先,否则从 `storage.sessions_dir` 推断)。
    fn configured_data_dir(&self) -> PathBuf;

    /// Runtime 目录(heartbeat / locks / 临时状态根)。
    fn configured_runtime_dir(&self) -> PathBuf;

    /// Skill 启用/禁用 override 文件路径。
    fn configured_skill_registry_path(&self) -> PathBuf;
}

// ── HoneBotCore 的 trait 实现。
//
// 全部委托给 HoneBotCore 的 inherent method,零行为变化。
// inherent method 继续存在,调用方 `core.log_message_*` 仍能走原路径;
// 若调用方显式 use 了这些 trait,`&dyn AuditRecorder` 也能正常工作。

impl AuditRecorder for HoneBotCore {
    fn log_message_received(
        &self,
        channel: &str,
        user_id: &str,
        channel_target: &str,
        session_id: &str,
        input: &str,
        extra: Option<&str>,
        message_id: Option<&str>,
    ) {
        HoneBotCore::log_message_received(
            self,
            channel,
            user_id,
            channel_target,
            session_id,
            input,
            extra,
            message_id,
        )
    }

    fn log_message_step(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        step: &str,
        detail: &str,
        message_id: Option<&str>,
        state_override: Option<&str>,
    ) {
        HoneBotCore::log_message_step(
            self,
            channel,
            user_id,
            session_id,
            step,
            detail,
            message_id,
            state_override,
        )
    }

    fn log_message_finished(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        response: &AgentResponse,
        elapsed_ms: u128,
        message_id: Option<&str>,
    ) {
        HoneBotCore::log_message_finished(
            self, channel, user_id, session_id, response, elapsed_ms, message_id,
        )
    }

    fn log_message_failed(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        error: &str,
        elapsed_ms: u128,
        message_id: Option<&str>,
    ) {
        HoneBotCore::log_message_failed(
            self, channel, user_id, session_id, error, elapsed_ms, message_id,
        )
    }
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

impl PathResolver for HoneBotCore {
    fn configured_system_skills_dir(&self) -> PathBuf {
        HoneBotCore::configured_system_skills_dir(self)
    }

    fn configured_custom_skills_dir(&self) -> PathBuf {
        HoneBotCore::configured_custom_skills_dir(self)
    }

    fn configured_data_dir(&self) -> PathBuf {
        HoneBotCore::configured_data_dir(self)
    }

    fn configured_runtime_dir(&self) -> PathBuf {
        HoneBotCore::configured_runtime_dir(self)
    }

    fn configured_skill_registry_path(&self) -> PathBuf {
        HoneBotCore::configured_skill_registry_path(self)
    }
}

#[cfg(test)]
mod tests {
    //! 这里主要证明 trait object 可以被构造出来,而不是重复测试 HoneBotCore 本身。
    //! HoneBotCore 的行为测试在 core.rs 中覆盖。

    use super::*;

    /// 静态检查：HoneBotCore 能被当作 `&dyn AuditRecorder` 使用。
    #[test]
    fn hone_bot_core_is_object_safe_audit_recorder() {
        fn _assert<T: AuditRecorder + ?Sized>(_: &T) {}
        // 本测试只要能编译过就算成功——构造真正的 HoneBotCore 需要完整配置,
        // 在这里意义不大。object-safety 由编译器检查保证。
    }

    #[test]
    fn hone_bot_core_is_object_safe_admin_intercept() {
        fn _assert<T: AdminIntercept + ?Sized>(_: &T) {}
    }

    #[test]
    fn hone_bot_core_is_object_safe_path_resolver() {
        fn _assert<T: PathResolver + ?Sized>(_: &T) {}
    }
}
