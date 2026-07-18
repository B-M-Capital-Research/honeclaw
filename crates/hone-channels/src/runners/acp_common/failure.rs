//! Convert an ACP transport/protocol failure into a runner result without
//! discarding tool traces observed before the failure.

use std::sync::Arc;

use hone_core::agent::AgentResponse;

use crate::runners::types::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerResult};
use crate::tool_trace::{
    PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE, response_has_persistent_side_effect,
};

use super::state::AcpRunFailure;
use super::tool_state::{finalize_context_messages, finalize_pending_tool_calls};

pub(crate) async fn acp_failure_to_runner_result(
    mut failure: AcpRunFailure,
    emitter: Arc<dyn AgentRunnerEmitter>,
) -> AgentRunnerResult {
    finalize_pending_tool_calls(&mut failure.state, "unknown_after_acp_failure");
    let context_messages = finalize_context_messages(&mut failure.state);
    let tool_calls_made = failure.state.finished_tool_calls.clone();
    let has_persistent_side_effect = response_has_persistent_side_effect(&tool_calls_made);
    let message = if has_persistent_side_effect {
        PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE.to_string()
    } else {
        failure.error.message.clone()
    };

    // The session layer owns the single safe terminal error for an operation
    // whose state is uncertain.  For ordinary failures preserve legacy runner
    // error emission.
    if !has_persistent_side_effect {
        emitter
            .emit(AgentRunnerEvent::Error {
                error: failure.error,
            })
            .await;
    }

    AgentRunnerResult {
        response: AgentResponse {
            content: String::new(),
            tool_calls_made,
            iterations: 1,
            success: false,
            error: Some(message),
        },
        streamed_output: !has_persistent_side_effect,
        committed_visible_prefix: None,
        terminal_error_emitted: !has_persistent_side_effect,
        session_metadata_updates: failure.metadata_updates,
        context_messages: Some(context_messages),
    }
}
