//! Agent session 共享的数据类型:Error/Event/Listener/Options/Metadata。
//!
//! 这些纯数据结构被外部 binary（hone-cli / hone-telegram / hone-feishu / …）
//! 直接消费,公开路径保持在 `hone_channels::agent_session::*`
//! （通过 `mod.rs` 的 `pub use` 再导出）。

use async_trait::async_trait;
use hone_core::agent::AgentResponse;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::run_event::RunEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionErrorKind {
    SpawnFailed,
    StdoutUnavailable,
    TimeoutOverall,
    TimeoutPerLine,
    GeminiError,
    ContextWindowOverflow,
    Io,
    ExitFailure,
    AgentTimeout,
    AgentFailed,
}

#[derive(Debug, Clone)]
pub struct AgentSessionError {
    pub kind: AgentSessionErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum AgentSessionEvent {
    Run(RunEvent),
    UserMessage { content: String },
    Segment { text: String },
    Done { response: AgentResponse },
}

#[async_trait]
pub trait AgentSessionListener: Send + Sync {
    async fn on_event(&self, event: AgentSessionEvent);
}

/// 构造一个 `Run(Progress)` 事件。放在 types 里而不是 core 里,
/// 是因为 emitter / core / run_manual_compact 三处都要用。
pub(super) fn session_progress_event(
    stage: &'static str,
    detail: Option<String>,
) -> AgentSessionEvent {
    AgentSessionEvent::Run(RunEvent::Progress { stage, detail })
}

pub(super) fn session_error_event(error: AgentSessionError) -> AgentSessionEvent {
    AgentSessionEvent::Run(RunEvent::Error { error })
}

#[derive(Debug, Clone, Default)]
pub struct MessageMetadata {
    pub user: Option<HashMap<String, Value>>,
    pub assistant: Option<HashMap<String, Value>>,
}

#[derive(Clone)]
pub struct AgentRunOptions {
    pub timeout: Option<Duration>,
    pub segmenter: Option<Arc<dyn Fn(&str) -> Vec<String> + Send + Sync>>,
    pub quota_mode: AgentRunQuotaMode,
    pub model_override: Option<String>,
}

impl Default for AgentRunOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            segmenter: None,
            quota_mode: AgentRunQuotaMode::UserConversation,
            model_override: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentRunQuotaMode {
    #[default]
    UserConversation,
    ScheduledTask,
}

#[derive(Clone)]
pub struct GeminiStreamOptions {
    pub max_iterations: u32,
    pub overall_timeout: Duration,
    pub per_line_timeout: Duration,
}

impl Default for GeminiStreamOptions {
    fn default() -> Self {
        Self {
            max_iterations: 18,
            overall_timeout: Duration::from_secs(600),
            per_line_timeout: Duration::from_secs(180),
        }
    }
}

pub struct AgentSessionResult {
    pub response: AgentResponse,
    pub elapsed_ms: u128,
    pub session_id: String,
}
