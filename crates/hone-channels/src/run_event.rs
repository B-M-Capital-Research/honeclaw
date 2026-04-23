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
