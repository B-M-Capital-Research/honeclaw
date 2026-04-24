//! AgentRunner 工厂 trait。
//!
//! 根据 `config.agent.runner` 选择具体实现(function_calling / codex_cli /
//! codex_acp / opencode_acp / multi_agent / gemini_cli / gemini_acp),
//! 并注入 tool_registry、LLM provider、超时等运行时依赖。
//!
//! 把这一行为放到 trait 是为了让 ExecutionService / 测试可以 mock 一个
//! 特定 runner,而不用构造完整 HoneBotCore（那需要真实 LLM provider、
//! audit sink 等)。

use hone_tools::ToolRegistry;

use crate::core::HoneBotCore;
use crate::runners::AgentRunner;

/// 根据配置生产 [`AgentRunner`]。
pub trait RunnerFactory: Send + Sync {
    /// 当前 runner 是否支持「每个 actor 一份 sandbox cwd」的强隔离模型。
    /// 目前所有实现都返回 `true`;历史上曾有 gemini_acp 返回 `false`,
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

    /// 用指定 model override 构造 runner。适用于 multi-agent / opencode_acp
    /// 等支持在 skill 级别切模型的场景。
    fn create_runner_with_model_override(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
        model_override: Option<&str>,
    ) -> Result<Box<dyn AgentRunner>, String>;
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
