//! HoneBotCore 的职责分组 trait（共 6 项）。
//!
//! 背景：`HoneBotCore` 是 1600+ 行的 "God Object"，同时承担 LLM provider 工厂、
//! 审计记录、管理员权限判定、路径解析、tool registry 生成、runner 创建、
//! scheduler 创建、session 压缩等方向的职责。这样的对象有三个结构问题：
//!
//! - **强耦合**：`Arc<HoneBotCore>` 被 workspace 48+ 处消费，任何一个子系统
//!   都能调用所有能力，模块边界不清
//! - **测试困难**：测试一个 AgentSession stage 必须构造完整 HoneBotCore
//! - **演化困难**：加新能力都挂在 core 上，面积只增不减
//!
//! 本模块按职责把 `HoneBotCore` 切成 6 个 trait。每个 trait 独立一个子文件,
//! HoneBotCore 通过转发 inherent method / pub field 实现它们。调用方
//! 暂时不改,仍然可以用 `core.log_message_step(...)`;想用 `&dyn AuditRecorder`
//! 替代 `&HoneBotCore` 的调用点可以逐个迁移,不用一次改完。
//!
//! 各 trait 及其职责：
//! - [`AuditRecorder`]       —— 消息流审计日志 (log_message_{received,step,finished,failed})
//! - [`AdminIntercept`]      —— 管理员判定与 runtime 拦截命令
//! - [`PathResolver`]        —— 运行时路径查询 (configured_*_dir)
//! - [`RunnerFactory`]       —— 根据 agent.runner 配置创建具体 AgentRunner
//! - [`ToolRegistryFactory`] —— 为当前 actor 构造 ToolRegistry（含权限过滤）
//! - [`LlmProviderBundle`]   —— 主 / auxiliary LLM provider + audit sink 访问器
//!
//! 所有 trait 都是 `Send + Sync` 且 object-safe（tests 模块里有 compile-time 验证）。

pub mod admin;
pub mod audit;
pub mod llm;
pub mod path;
pub mod runner;
pub mod tool_registry;

pub use admin::AdminIntercept;
pub use audit::AuditRecorder;
pub use llm::LlmProviderBundle;
pub use path::PathResolver;
pub use runner::RunnerFactory;
pub use tool_registry::ToolRegistryFactory;

#[cfg(test)]
mod tests;
