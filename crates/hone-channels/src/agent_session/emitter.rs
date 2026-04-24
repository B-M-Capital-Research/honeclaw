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
use crate::runtime::relativize_user_visible_paths;

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

#[async_trait]
impl AgentRunnerEmitter for SessionEventEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        let event = match event {
            AgentRunnerEvent::Progress { stage, detail } => AgentRunnerEvent::Progress {
                stage,
                detail: detail
                    .map(|value| relativize_user_visible_paths(&value, &self.working_directory)),
            },
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message: message
                    .map(|value| relativize_user_visible_paths(&value, &self.working_directory)),
                reasoning: reasoning
                    .map(|value| relativize_user_visible_paths(&value, &self.working_directory)),
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
