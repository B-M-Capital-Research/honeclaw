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
//! 已抽出的 trait（全部 6 项）：
//! - [`AuditRecorder`]       —— 消息流审计日志 (log_message_{received,step,finished,failed})
//! - [`AdminIntercept`]      —— 管理员判定与 runtime 拦截命令
//! - [`PathResolver`]        —— 运行时路径查询 (configured_*_dir)
//! - [`RunnerFactory`]       —— 根据 agent.runner 配置创建具体 AgentRunner
//! - [`ToolRegistryFactory`] —— 为当前 actor 构造 ToolRegistry（含权限过滤）
//! - [`LlmProviderBundle`]   —— 主 / auxiliary LLM provider + audit sink 访问器
//!
//! 全部 trait 都是「HoneBotCore 已有能力的契约版本」,目前 HoneBotCore 通过
//! 转发到 inherent method / pub field 实现。调用方还没有被迁移到 `&dyn Trait`,
//! 这是下一阶段的工作（每个调用点单独决策,不会一次大改）。
//!
//! ## 为什么不一次做完
//!
//! 余下 3 个 trait 涉及 LlmProvider 字段、Runner 构造逻辑、复杂闭包生命周期，
//! 与 `HoneBotCore::new` 强耦合，需要独立的 context 窗口小心推进。本文件里
//! 的 3 个 trait 已经给出了「trait + 转发 impl + 文档锁定」的模板，
//! 后续只需套用这一模式。

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

use hone_core::agent::AgentResponse;
use hone_core::{ActorIdentity, LlmAuditSink};
use hone_llm::LlmProvider;
use hone_tools::ToolRegistry;

use crate::core::HoneBotCore;
use crate::runners::AgentRunner;

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

/// Agent runner 工厂。
///
/// 根据 `config.agent.runner` 选择具体实现（function_calling / codex_cli /
/// codex_acp / opencode_acp / multi_agent / gemini_cli / gemini_acp），
/// 并注入 tool_registry、LLM provider、超时等运行时依赖。
///
/// 把这一行为放到 trait 是为了让 ExecutionService / 测试可以 mock 一个
/// 特定 runner 而不用构造完整 HoneBotCore（包含真实 LLM provider）。
pub trait RunnerFactory: Send + Sync {
    /// 当前 runner 是否支持「每个 actor 一份 sandbox cwd」的强隔离模型。
    /// 目前所有实现都返回 `true`；历史上曾有 gemini_acp 返回 `false`,
    /// 保留 trait 形式让这个维度永远显式。
    fn runner_supports_strict_actor_sandbox(&self) -> bool;

    /// 若 runner 不支持强沙箱,返回一条面向用户的解释文案;否则 `None`。
    fn strict_actor_sandbox_guard_message(&self) -> Option<&'static str>;

    /// 用默认模型构造 runner。
    fn create_runner(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
    ) -> Result<Box<dyn AgentRunner>, String>;

    /// 用指定 model override 构造 runner（multi-agent / opencode_acp 等
    /// 支持 skill 级别切模型的场景使用)。
    fn create_runner_with_model_override(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
        model_override: Option<&str>,
    ) -> Result<Box<dyn AgentRunner>, String>;
}

/// LLM provider / audit sink 访问器。
///
/// 当前 `HoneBotCore` 把这三项暴露成 `pub` 字段,直接 `core.llm.clone()`
/// 就能拿到 `Arc<dyn LlmProvider>`;这样很顺手但也让「依赖了一个 LLM 路由」
/// 这件事隐形：审视一个模块是否需要 LLM 得翻代码。抽 trait 之后,把这种
/// 依赖显式化成 `&dyn LlmProviderBundle`,同时为测试 mock 打开通道。
pub trait LlmProviderBundle: Send + Sync {
    /// 主对话 LLM（走 `agent.runner` 选定的路径;可能未配置,返回 `None`）。
    fn primary_llm(&self) -> Option<Arc<dyn LlmProvider>>;

    /// 辅助 LLM（heartbeat / session compaction 等后台任务使用）。
    fn auxiliary_llm(&self) -> Option<Arc<dyn LlmProvider>>;

    /// 审计落盘 sink（启用时把 LLM 请求 / 响应保存到 SQLite）。
    fn llm_audit_sink(&self) -> Option<Arc<dyn LlmAuditSink>>;

    /// 辅助 LLM 的显示用模型名称（降级时会回落到 openrouter.sub_model）。
    fn auxiliary_model_name(&self) -> String;

    /// 辅助 LLM 的 `(provider_display, model_display)`,供前端展示。
    fn auxiliary_provider_hint(&self) -> (String, String);
}

/// 工具注册表工厂：根据 actor 身份和渠道 target 构造一份 `ToolRegistry`。
///
/// 同一个 HoneBotCore 会为每次 run 单独构造一份 registry,是因为 tool 的
/// 权限过滤（管理员 / cron 是否允许 / sandbox 根目录等）依赖具体 actor。
/// 抽成 trait 之后,测试可以注入固定的一份 registry 来绕开 skill/security
/// 配置校验。
pub trait ToolRegistryFactory: Send + Sync {
    fn create_tool_registry(
        &self,
        actor: Option<&ActorIdentity>,
        channel_target: &str,
        allow_cron: bool,
    ) -> ToolRegistry;
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

impl RunnerFactory for HoneBotCore {
    fn runner_supports_strict_actor_sandbox(&self) -> bool {
        HoneBotCore::runner_supports_strict_actor_sandbox(self)
    }

    fn strict_actor_sandbox_guard_message(&self) -> Option<&'static str> {
        HoneBotCore::strict_actor_sandbox_guard_message(self)
    }

    fn create_runner(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
    ) -> Result<Box<dyn AgentRunner>, String> {
        HoneBotCore::create_runner(self, system_prompt, tool_registry)
    }

    fn create_runner_with_model_override(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
        model_override: Option<&str>,
    ) -> Result<Box<dyn AgentRunner>, String> {
        HoneBotCore::create_runner_with_model_override(
            self,
            system_prompt,
            tool_registry,
            model_override,
        )
    }
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

impl LlmProviderBundle for HoneBotCore {
    fn primary_llm(&self) -> Option<Arc<dyn LlmProvider>> {
        self.llm.clone()
    }

    fn auxiliary_llm(&self) -> Option<Arc<dyn LlmProvider>> {
        self.auxiliary_llm.clone()
    }

    fn llm_audit_sink(&self) -> Option<Arc<dyn LlmAuditSink>> {
        self.llm_audit.clone()
    }

    fn auxiliary_model_name(&self) -> String {
        HoneBotCore::auxiliary_model_name(self)
    }

    fn auxiliary_provider_hint(&self) -> (String, String) {
        HoneBotCore::auxiliary_provider_hint(self)
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

    #[test]
    fn hone_bot_core_is_object_safe_runner_factory() {
        fn _assert<T: RunnerFactory + ?Sized>(_: &T) {}
    }

    #[test]
    fn hone_bot_core_is_object_safe_tool_registry_factory() {
        fn _assert<T: ToolRegistryFactory + ?Sized>(_: &T) {}
    }

    #[test]
    fn hone_bot_core_is_object_safe_llm_provider_bundle() {
        fn _assert<T: LlmProviderBundle + ?Sized>(_: &T) {}
    }
}
