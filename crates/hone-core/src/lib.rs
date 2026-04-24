//! Hone Core — 配置、日志、错误类型、共享类型
//!
//! 提供所有 crate 共用的基础设施。

pub mod actor;
pub mod agent;
pub mod api_key_pool;
pub mod audit;
pub mod channel_process;
pub mod config;
pub mod error;
pub mod heartbeat;
pub mod logging;
pub mod process_lock;
pub mod text;
pub mod time;
pub mod tool_event;

pub const CHANNEL_DISABLED_EXIT_CODE: i32 = 20;

pub use actor::{ActorIdentity, SessionIdentity, SessionKind};
pub use api_key_pool::ApiKeyPool;
pub use audit::{LlmAuditRecord, LlmAuditSink};
pub use channel_process::{ObservedChannelProcess, channel_binary_name, scan_channel_processes};
pub use config::{ChatScope, HoneConfig};
pub use error::{HoneError, HoneResult};
pub use heartbeat::{
    HEARTBEAT_INTERVAL_SECS, HEARTBEAT_STALE_AFTER_SECS, HeartbeatErrorRecord, HeartbeatMetrics,
    ProcessHeartbeat, ProcessHeartbeatSnapshot, read_heartbeat_error, read_process_heartbeat,
    runtime_heartbeat_dir, runtime_heartbeat_error_path, runtime_heartbeat_path,
    spawn_process_heartbeat,
};
pub use process_lock::{
    PROCESS_LOCK_CONSOLE_PAGE, PROCESS_LOCK_DESKTOP, PROCESS_LOCK_DISCORD, PROCESS_LOCK_FEISHU,
    PROCESS_LOCK_IMESSAGE, PROCESS_LOCK_TELEGRAM, ProcessLockError, ProcessLockGuard,
    acquire_process_lock, acquire_runtime_process_lock, format_lock_failure_message,
    preflight_process_locks, process_lock_path, runtime_lock_dir,
};
pub use text::{truncate_chars, truncate_chars_append};
pub use time::{BEIJING_OFFSET_SECS, beijing_now, beijing_now_rfc3339, beijing_offset};
pub use tool_event::ToolExecutionObserver;
