//! Hone Channels — 渠道运行时
//!
//! 提供 HoneBotCore 配置工厂、流式分段处理引擎。

pub mod agent_session;
pub mod attachments;
pub mod core;
pub mod ingress;
pub mod kb_analysis;
pub mod mcp_bridge;
pub mod outbound;
pub mod prompt;
pub(crate) mod runners;
pub mod runtime;
pub(crate) mod sandbox;
pub mod scheduler;

pub use self::core::HoneBotCore;
pub use self::core::load_runtime_config;
pub use self::sandbox::channel_download_dir;
pub use agent_session::{
    AgentRunOptions, AgentSession, AgentSessionError, AgentSessionErrorKind, AgentSessionEvent,
    AgentSessionListener, AgentSessionResult, GeminiStreamOptions, MessageMetadata,
    restore_context,
};
pub use ingress::{
    ActorScopeResolver, ChatMode, GroupTrigger, GroupTriggerMode, IncomingEnvelope,
    MessageDeduplicator, SessionLockRegistry,
};
pub use kb_analysis::run_kb_analysis;
pub use outbound::{
    OutboundAdapter, OutboundRunSummary, StreamActivityProbe, attach_stream_activity_probe,
    run_session_with_outbound,
};
