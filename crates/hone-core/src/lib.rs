//! Hone Core — 配置、日志、错误类型、共享类型
//!
//! 提供所有 crate 共用的基础设施。

pub mod actor;
pub mod agent;
pub mod api_key_pool;
pub mod audit;
pub mod config;
pub mod error;
pub mod heartbeat;
pub mod logging;
pub mod time;
pub mod tool_event;

pub use actor::{ActorIdentity, SessionIdentity, SessionKind};
pub use api_key_pool::ApiKeyPool;
pub use audit::{LlmAuditRecord, LlmAuditSink};
pub use config::{ChatScope, HoneConfig};
pub use error::{HoneError, HoneResult};
pub use heartbeat::{
    HEARTBEAT_INTERVAL_SECS, HEARTBEAT_STALE_AFTER_SECS, ProcessHeartbeat,
    ProcessHeartbeatSnapshot, read_process_heartbeat, runtime_heartbeat_dir,
    runtime_heartbeat_path, spawn_process_heartbeat,
};
pub use time::{BEIJING_OFFSET_SECS, beijing_now, beijing_now_rfc3339, beijing_offset};
pub use tool_event::ToolExecutionObserver;
