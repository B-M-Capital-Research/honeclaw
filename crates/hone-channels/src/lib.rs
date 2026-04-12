//! Hone Channels — 渠道运行时
//!
//! 提供 HoneBotCore 配置工厂、流式分段处理引擎。

pub mod agent_session;
pub mod attachments;
pub mod bootstrap;
pub mod core;
pub(crate) mod execution;
pub mod ingress;
pub mod mcp_bridge;
pub mod outbound;
pub mod prompt;
pub(crate) mod prompt_audit;
pub(crate) mod runners;
pub mod runtime;
pub(crate) mod sandbox;
pub mod scheduler;
pub(crate) mod session_compactor;
pub mod think;

pub use self::core::HoneBotCore;
pub use self::core::load_runtime_config;
pub use self::sandbox::{channel_download_dir, sandbox_base_dir};
pub use agent_session::{
    AgentRunOptions, AgentSession, AgentSessionError, AgentSessionErrorKind, AgentSessionEvent,
    AgentSessionListener, AgentSessionResult, GeminiStreamOptions, MessageMetadata,
    restore_context,
};
pub use bootstrap::{ChannelRuntimeBootstrap, bootstrap_channel_runtime};
pub use ingress::{
    ActiveSessionInfo, ActorScopeResolver, BufferedGroupMessage, ChatMode,
    GroupPretriggerWindowRegistry, GroupTrigger, GroupTriggerMode, IncomingEnvelope,
    MessageDeduplicator, SessionLockRegistry, persist_buffered_group_messages,
};
pub use outbound::{
    OutboundAdapter, OutboundRunSummary, StreamActivityProbe, attach_stream_activity_probe,
    run_session_with_outbound,
};
