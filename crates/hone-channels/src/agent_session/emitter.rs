//! `SessionEventEmitter` —— 把 runner 侧的 `AgentRunnerEvent` 翻译成
//! session 侧的 `AgentSessionEvent` 并多播给所有 listener 的转发器。
//!
//! 两个职责:
//! 1. **路径脱敏**:把事件里的绝对路径压成相对路径(见
//!    `relativize_user_visible_paths`),避免沙盒细节泄给 UI / 下游;
//! 2. **结构化日志**:每次 runner 事件额外在 sidecar.log 打一条 `runner_*`
//!    state,方便运维拿 message_id 定位整条 flow。

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

fn truncate_event_detail(detail: &str, max_chars: usize) -> String {
    if detail.chars().count() <= max_chars {
        return detail.to_string();
    }
    detail.chars().take(max_chars).collect::<String>() + "..."
}

fn contains_internal_progress_marker(text: &str) -> bool {
    const MARKERS: &[&str] = &[
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
    ];
    MARKERS.iter().any(|marker| text.contains(marker))
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

fn sanitize_user_visible_event_text(value: &str, working_directory: &str) -> Option<String> {
    let relativized = relativize_user_visible_paths(value, working_directory);
    let sanitized = sanitize_user_visible_output(&relativized);
    let text = sanitized.content.trim();
    if text.is_empty() || contains_internal_progress_marker(text) {
        return None;
    }
    if looks_like_structured_tool_payload(text) {
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
            AgentRunnerEvent::Error { mut error } => {
                error.message =
                    relativize_user_visible_paths(&error.message, &self.working_directory);
                AgentRunnerEvent::Error { error }
            }
            other => other,
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
            AgentRunnerEvent::StreamDelta { .. } | AgentRunnerEvent::StreamThought { .. } => {}
        }

        let mapped = AgentSessionEvent::Run(event);

        for listener in &self.listeners {
            listener.on_event(mapped.clone()).await;
        }
    }
}
