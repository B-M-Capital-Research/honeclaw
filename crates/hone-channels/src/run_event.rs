use crate::agent_session::AgentSessionError;

/// Canonical runtime event emitted by agent runners and forwarded by sessions.
#[derive(Debug, Clone)]
pub enum RunEvent {
    Progress {
        stage: &'static str,
        detail: Option<String>,
    },
    StreamDelta {
        content: String,
    },
    /// A runner-authored prefix that is safe to expose before the complete
    /// response reaches the AgentSession terminal boundary.
    ///
    /// This is an internal runner/session control event. Session emitters map
    /// it back to an ordinary `StreamDelta` before notifying user-facing
    /// listeners, while deferred emitters use the distinct variant to keep
    /// speculative model text hidden.
    CommittedStreamDelta {
        content: String,
    },
    StreamReset,
    StreamThought {
        thought: String,
    },
    ToolStatus {
        tool: String,
        status: String,
        message: Option<String>,
        reasoning: Option<String>,
    },
    Error {
        error: AgentSessionError,
    },
}
