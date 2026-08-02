//! `SessionEventEmitter` —— 把 runner 侧的 `AgentRunnerEvent` 翻译成
//! session 侧的 `AgentSessionEvent` 并多播给所有 listener 的转发器。
//!
//! 两个职责:
//! 1. **用户态净化**:把流式正文 / 工具进度里的内部 prompt、原始 JSON payload
//!    和绝对路径收口在 runner 边界内,避免沙盒细节泄给 UI / 下游;
//! 2. **结构化日志**:每次 runner 事件额外打 `runner_*` state,方便运维拿
//!    message_id 定位整条 flow。

use async_trait::async_trait;
use std::sync::Arc;

use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent};
use crate::runtime::{relativize_user_visible_paths, sanitize_user_visible_output};

use super::types::{AgentSessionEvent, AgentSessionListener};

pub(super) struct SessionEventEmitter {
    pub(super) listeners: Vec<Arc<dyn AgentSessionListener>>,
    pub(super) channel: String,
    pub(super) user_id: String,
    pub(super) session_id: String,
    pub(super) message_id: Option<String>,
    pub(super) working_directory: String,
}

/// Buffers answer-bearing runner events until the Agent has completed the
/// current tool loop. Progress and tool status remain visible, but draft text,
/// resets, thoughts, and attempt-local errors must never flicker to the user.
/// This emitter does not inspect or alter the completed answer; the session
/// publishes that answer once at the terminal boundary.
pub(super) struct DeferredUserOutputEmitter {
    inner: Arc<dyn AgentRunnerEmitter>,
}

impl DeferredUserOutputEmitter {
    pub(super) fn new(inner: Arc<dyn AgentRunnerEmitter>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl AgentRunnerEmitter for DeferredUserOutputEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        match event {
            forward @ AgentRunnerEvent::Progress { .. }
            | forward @ AgentRunnerEvent::ToolStatus { .. }
            | forward @ AgentRunnerEvent::CommittedStreamDelta { .. } => {
                self.inner.emit(forward).await;
            }
            AgentRunnerEvent::StreamDelta { .. }
            | AgentRunnerEvent::StreamReset
            | AgentRunnerEvent::StreamThought { .. }
            | AgentRunnerEvent::Error { .. } => {}
        }
    }

    async fn emit_committed(&self, event: AgentRunnerEvent) -> bool {
        match event {
            forward @ AgentRunnerEvent::CommittedStreamDelta { .. } => {
                self.inner.emit_committed(forward).await
            }
            other => {
                self.emit(other).await;
                true
            }
        }
    }
}

fn truncate_event_detail(detail: &str, max_chars: usize) -> String {
    if detail.chars().count() <= max_chars {
        return detail.to_string();
    }
    detail.chars().take(max_chars).collect::<String>() + "..."
}

const INTERNAL_PROGRESS_MARKERS: &[&str] = &[
    "### System Instructions ###",
    "### System Prompt ###",
    "### Skill Context ###",
    "### Conversation Context ###",
    "### User Prompt ###",
    "### Available Skills ###",
    "【Session 上下文】",
    "【Invoked Skill Context】",
    "turn-0 可用技能索引",
    "Base directory for this skill:",
    "codex:approved-for-session",
    "opencode:approved-for-session",
    "Approve MCP tool call",
    "approved-for-session",
    "rawOutput",
    "rawInput",
];

fn first_internal_progress_marker(text: &str) -> Option<usize> {
    INTERNAL_PROGRESS_MARKERS
        .iter()
        .filter_map(|marker| text.find(marker))
        .min()
}

fn strip_internal_progress_suffix(text: &str) -> Option<String> {
    match first_internal_progress_marker(text) {
        Some(idx) => {
            let prefix = text[..idx].trim_end();
            (!prefix.trim().is_empty()).then(|| prefix.to_string())
        }
        None => Some(text.to_string()),
    }
}

fn contains_internal_progress_marker(text: &str) -> bool {
    first_internal_progress_marker(text).is_some()
}

fn looks_like_structured_tool_payload(text: &str) -> bool {
    let trimmed = text.trim();
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(trimmed)
        .map(|value| value.is_object() || value.is_array())
        .unwrap_or(false)
}

fn looks_like_internal_runtime_detail(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    text.contains("LLM 错误")
        || text.contains("HTTP 错误")
        || text.contains("渠道错误")
        || text.contains("工具执行错误")
        || text.contains("序列化错误")
        || text.contains("IO 错误")
        || lowered.contains("bad_request_error")
        || lowered.contains("invalid params")
        || lowered.contains("tool_call_id")
        || lowered.contains("tool call result")
        || lowered.contains("function arguments")
        || lowered.contains("session/prompt")
        || lowered.contains("codex acp")
        || lowered.contains("opencode acp")
        || lowered.contains("acp stream")
        || lowered.contains("approve mcp tool call")
        || lowered.contains("approved-for-session")
        || lowered.contains("rawoutput")
        || lowered.contains("rawinput")
}

fn sanitize_user_visible_event_text(value: &str, working_directory: &str) -> Option<String> {
    let clipped = strip_internal_progress_suffix(value)?;
    let relativized = relativize_user_visible_paths(&clipped, working_directory);
    let sanitized = sanitize_user_visible_output(&relativized);
    let text = sanitized.content.trim();
    if text.is_empty() || contains_internal_progress_marker(text) {
        return None;
    }
    if looks_like_structured_tool_payload(text) {
        return None;
    }
    if looks_like_internal_runtime_detail(text) {
        return None;
    }
    Some(text.to_string())
}

#[async_trait]
impl AgentRunnerEmitter for SessionEventEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        let event = match event {
            AgentRunnerEvent::Progress { stage, detail } => AgentRunnerEvent::Progress {
                stage,
                detail: detail.and_then(|value| {
                    sanitize_user_visible_event_text(&value, &self.working_directory)
                }),
            },
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => AgentRunnerEvent::ToolStatus {
                tool: sanitize_user_visible_event_text(&tool, &self.working_directory)
                    .unwrap_or_else(|| "工具".to_string()),
                status,
                message: message.and_then(|value| {
                    sanitize_user_visible_event_text(&value, &self.working_directory)
                }),
                reasoning: reasoning.and_then(|value| {
                    sanitize_user_visible_event_text(&value, &self.working_directory)
                }),
            },
            AgentRunnerEvent::StreamDelta { content } => {
                let Some(content) =
                    sanitize_user_visible_event_text(&content, &self.working_directory)
                else {
                    return;
                };
                AgentRunnerEvent::StreamDelta { content }
            }
            AgentRunnerEvent::CommittedStreamDelta { content } => {
                // This typed event is produced only after the runner has
                // validated a canonical header or a byte-stable continuation
                // line. Preserve its exact bytes. Silently filtering or
                // rewriting it here would make the runner believe answer bytes
                // were committed when listeners never received them.
                tracing::info!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_terminal_prefix_committed",
                    channel = %self.channel,
                    user = %self.user_id,
                    session = %self.session_id,
                    bytes = content.len(),
                    "Agent committed a terminal answer delta"
                );
                AgentRunnerEvent::StreamDelta { content }
            }
            AgentRunnerEvent::StreamThought { thought } => {
                let Some(thought) =
                    sanitize_user_visible_event_text(&thought, &self.working_directory)
                else {
                    return;
                };
                AgentRunnerEvent::StreamThought { thought }
            }
            AgentRunnerEvent::StreamReset => AgentRunnerEvent::StreamReset,
            AgentRunnerEvent::Error { mut error } => {
                error.message =
                    relativize_user_visible_paths(&error.message, &self.working_directory);
                AgentRunnerEvent::Error { error }
            }
        };

        match &event {
            AgentRunnerEvent::Progress { stage, detail } => {
                tracing::info!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_progress",
                    "[MsgFlow/{}] runner.stage={} user={} session={} detail={}",
                    self.channel,
                    stage,
                    self.user_id,
                    self.session_id,
                    detail
                        .as_deref()
                        .map(|value| truncate_event_detail(value, 280))
                        .unwrap_or_else(|| "-".to_string())
                );
            }
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => {
                tracing::info!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_tool",
                    "[MsgFlow/{}] runner.tool user={} session={} tool={} status={} message={} reasoning={}",
                    self.channel,
                    self.user_id,
                    self.session_id,
                    tool,
                    status,
                    message
                        .as_deref()
                        .map(|value| truncate_event_detail(value, 200))
                        .unwrap_or_else(|| "-".to_string()),
                    reasoning
                        .as_deref()
                        .map(|value| truncate_event_detail(value, 200))
                        .unwrap_or_else(|| "-".to_string())
                );
            }
            AgentRunnerEvent::Error { error } => {
                tracing::warn!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_error",
                    "[MsgFlow/{}] runner.error user={} session={} kind={:?} message=\"{}\"",
                    self.channel,
                    self.user_id,
                    self.session_id,
                    error.kind,
                    truncate_event_detail(&error.message, 280)
                );
            }
            AgentRunnerEvent::StreamDelta { .. }
            | AgentRunnerEvent::CommittedStreamDelta { .. }
            | AgentRunnerEvent::StreamReset
            | AgentRunnerEvent::StreamThought { .. } => {}
        }

        let mapped = AgentSessionEvent::Run(event);

        for listener in &self.listeners {
            listener.on_event(mapped.clone()).await;
        }
    }

    async fn emit_committed(&self, event: AgentRunnerEvent) -> bool {
        let AgentRunnerEvent::CommittedStreamDelta { content } = event else {
            self.emit(event).await;
            return true;
        };

        tracing::info!(
            message_id = %self.message_id.as_deref().unwrap_or("-"),
            state = "runner_terminal_prefix_commit_attempt",
            channel = %self.channel,
            user = %self.user_id,
            session = %self.session_id,
            bytes = content.len(),
            "Agent is attempting to commit an irreversible terminal answer delta"
        );
        let mapped = AgentSessionEvent::Run(AgentRunnerEvent::StreamDelta { content });
        let mut sinks = self
            .listeners
            .iter()
            .filter(|listener| listener.supports_committed_delivery());
        let Some(sink) = sinks.next() else {
            tracing::warn!(
                message_id = %self.message_id.as_deref().unwrap_or("-"),
                state = "runner_terminal_prefix_commit_unsupported",
                channel = %self.channel,
                user = %self.user_id,
                session = %self.session_id,
                "no listener supports committed terminal answer delivery"
            );
            return false;
        };
        if sinks.next().is_some() {
            tracing::error!(
                message_id = %self.message_id.as_deref().unwrap_or("-"),
                state = "runner_terminal_prefix_commit_ambiguous",
                channel = %self.channel,
                user = %self.user_id,
                session = %self.session_id,
                "multiple listeners claim committed terminal answer delivery"
            );
            return false;
        }
        let accepted = sink.on_committed_event(mapped).await;
        if !accepted {
            tracing::warn!(
                message_id = %self.message_id.as_deref().unwrap_or("-"),
                state = "runner_terminal_prefix_commit_rejected",
                channel = %self.channel,
                user = %self.user_id,
                session = %self.session_id,
                "downstream transport rejected the terminal answer delta"
            );
        }
        accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
    use std::sync::Mutex;

    #[derive(Default)]
    struct CaptureRunnerEmitter {
        events: Mutex<Vec<AgentRunnerEvent>>,
    }

    #[async_trait]
    impl AgentRunnerEmitter for CaptureRunnerEmitter {
        async fn emit(&self, event: AgentRunnerEvent) {
            self.events.lock().expect("runner events").push(event);
        }
    }

    #[derive(Default)]
    struct CaptureSessionListener {
        events: tokio::sync::Mutex<Vec<AgentSessionEvent>>,
    }

    #[async_trait]
    impl AgentSessionListener for CaptureSessionListener {
        async fn on_event(&self, event: AgentSessionEvent) {
            self.events.lock().await.push(event);
        }
    }

    struct RejectingCommittedSessionListener;

    #[async_trait]
    impl AgentSessionListener for RejectingCommittedSessionListener {
        async fn on_event(&self, _event: AgentSessionEvent) {}

        fn supports_committed_delivery(&self) -> bool {
            true
        }

        async fn on_committed_event(&self, event: AgentSessionEvent) -> bool {
            assert!(matches!(
                event,
                AgentSessionEvent::Run(AgentRunnerEvent::StreamDelta { .. })
            ));
            false
        }
    }

    #[tokio::test]
    async fn committed_delivery_rejection_propagates_to_the_runner() {
        let emitter = SessionEventEmitter {
            listeners: vec![Arc::new(RejectingCommittedSessionListener)],
            channel: "web".to_string(),
            user_id: "closed-sse".to_string(),
            session_id: "session".to_string(),
            message_id: None,
            working_directory: ".".to_string(),
        };

        assert!(
            !emitter
                .emit_committed(AgentRunnerEvent::CommittedStreamDelta {
                    content: "数据时间：北京时间 2026-07-22 10:00；行情口径：最新可得\n"
                        .to_string(),
                })
                .await
        );
    }

    #[tokio::test]
    async fn deferred_output_forwards_only_committed_answer_content() {
        let inner = Arc::new(CaptureRunnerEmitter::default());
        let deferred = DeferredUserOutputEmitter::new(inner.clone());
        let committed = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";

        deferred
            .emit(AgentRunnerEvent::StreamDelta {
                content: "我先核验".to_string(),
            })
            .await;
        deferred.emit(AgentRunnerEvent::StreamReset).await;
        deferred
            .emit(AgentRunnerEvent::StreamThought {
                thought: "internal".to_string(),
            })
            .await;
        deferred
            .emit(AgentRunnerEvent::Error {
                error: AgentSessionError {
                    kind: AgentSessionErrorKind::AgentFailed,
                    message: "attempt-local".to_string(),
                },
            })
            .await;
        deferred
            .emit(AgentRunnerEvent::CommittedStreamDelta {
                content: committed.to_string(),
            })
            .await;

        let events = inner.events.lock().expect("runner events");
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::CommittedStreamDelta { content } if content == committed
        ));
    }

    #[tokio::test]
    async fn session_emitter_preserves_committed_prefix_bytes_as_visible_delta() {
        let listener = Arc::new(CaptureSessionListener::default());
        let emitter = SessionEventEmitter {
            listeners: vec![listener.clone()],
            channel: "web".to_string(),
            user_id: "committed-prefix".to_string(),
            session_id: "web:committed-prefix".to_string(),
            message_id: None,
            working_directory: String::new(),
        };
        let committed = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";

        emitter
            .emit(AgentRunnerEvent::CommittedStreamDelta {
                content: committed.to_string(),
            })
            .await;

        let events = listener.events.lock().await;
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentSessionEvent::Run(AgentRunnerEvent::StreamDelta { content })
                if content == committed
        ));
    }

    #[tokio::test]
    async fn session_emitter_sanitizes_external_reasoning_before_channel_projection() {
        let listener = Arc::new(CaptureSessionListener::default());
        let emitter = SessionEventEmitter {
            listeners: vec![listener.clone()],
            channel: "discord".to_string(),
            user_id: "reasoning".to_string(),
            session_id: "discord:reasoning".to_string(),
            message_id: None,
            working_directory: "/private/tmp/hone-workspace".to_string(),
        };

        emitter
            .emit(AgentRunnerEvent::StreamThought {
                thought: "正在检查 /private/tmp/hone-workspace/docs/invariants.md".to_string(),
            })
            .await;
        emitter
            .emit(AgentRunnerEvent::StreamThought {
                thought: "### System Prompt ###\nsecret".to_string(),
            })
            .await;

        let events = listener.events.lock().await;
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentSessionEvent::Run(AgentRunnerEvent::StreamThought { thought })
                if thought == "正在检查 docs/invariants.md"
        ));
    }
}
