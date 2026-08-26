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
    UserMessage {
        content: String,
    },
    Segment {
        text: String,
    },
    /// A user-visible prefix crossed an irreversible stream boundary, but the
    /// underlying run did not complete. Browser adapters close without an
    /// error flash while callers still receive the failed AgentSessionResult.
    PartialDone {
        response: AgentResponse,
    },
    Done {
        response: AgentResponse,
    },
}

#[async_trait]
pub trait AgentSessionListener: Send + Sync {
    async fn on_event(&self, event: AgentSessionEvent);

    /// Whether this listener is the unique publication sink capable of
    /// accepting an irreversible prefix and the later Segment/PartialDone
    /// continuation. Observers and non-streaming channel adapters stay false.
    fn supports_committed_delivery(&self) -> bool {
        false
    }

    /// Typed delivery acknowledgement for the unique publication sink. The
    /// default fails closed; callers must check `supports_committed_delivery`
    /// before invoking it.
    async fn on_committed_event(&self, event: AgentSessionEvent) -> bool {
        let _ = event;
        false
    }
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
    pub runner_override: Option<AgentRunRunnerOverride>,
    pub model_override: Option<String>,
    pub turn_origin: AgentTurnOrigin,
    pub entity_resolution_input: Option<String>,
    /// Run the current turn without replaying persisted conversation messages or
    /// compact summaries. Trusted, self-contained workflows use this to prevent
    /// completed instructions from an unrelated prior turn becoming active work.
    pub isolate_prior_history: bool,
    /// Apply the server-owned earnings workflow contract instead of the generic
    /// interactive investment preflight, first-line timestamp, and answer shape.
    /// Only trusted administrator routes may enable this profile.
    pub dedicated_earnings_workflow: bool,
    /// Images the user attached to THIS turn, handed to a vision-capable model
    /// as real image parts. The text pipeline still summarizes attachments for
    /// the prompt; this is what lets the model actually see them instead of
    /// reasoning from a filename.
    pub turn_images: Vec<TurnImage>,
}

/// A user-attached image awaiting hand-off to the model. Held as a path so the
/// bytes are read once, at the runner boundary, rather than carried through
/// every layer of the turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnImage {
    pub local_path: String,
    pub mime_type: String,
    pub display_name: String,
}

impl Default for AgentRunOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            segmenter: None,
            quota_mode: AgentRunQuotaMode::UserConversation,
            runner_override: None,
            model_override: None,
            turn_origin: AgentTurnOrigin::Interactive,
            entity_resolution_input: None,
            isolate_prior_history: false,
            dedicated_earnings_workflow: false,
            turn_images: Vec::new(),
        }
    }
}

/// A server-owned runner route for a single Agent turn. This is intentionally
/// not deserialized from channel input; callers must establish the trust
/// boundary before setting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunRunnerOverride {
    OpencodeAcp,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentTurnOrigin {
    #[default]
    Interactive,
    Scheduled,
    Heartbeat,
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
