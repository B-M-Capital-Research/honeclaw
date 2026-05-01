//! 把 ACP 的 `session/update` notification 翻译成 `AgentRunnerEvent`,
//! 同时把 compact 检测信号、tool call 状态变迁写进 [`AcpPromptState`]。
//!
//! 两个对外入口:
//! - `handle_acp_session_update` / `handle_acp_session_update_with_renderer`:完整的
//!   `session/update` 分发器,被 `protocol::process_acp_payload` 调用;
//! - `ingest_acp_message_chunk` / `ingest_acp_usage_update`:供 opencode runner
//!   复用的两个流式子步骤(opencode 走自定义的 stream loop,但这两段逻辑和 codex 共享)。

use std::sync::Arc;

use serde_json::Value;

use crate::runners::types::{AgentRunnerEmitter, AgentRunnerEvent};
use crate::runtime::resolve_tool_reasoning;

use super::extract::{extract_acp_reasoning, extract_tool_failure, extract_tool_result};
use super::state::{
    ACP_BOUNDARY_SCAN_TAIL_BYTES, AcpPromptState, AcpToolRenderPhase, AcpToolStatusRenderer,
    RE_ACP_COMPACT_STATUS_TEXT, RE_OPENCODE_SUMMARY_BOUNDARY, floor_char_boundary,
};
use super::tool_state::{capture_tool_finish, capture_tool_start, flush_pending_assistant_message};

pub(crate) fn acp_prompt_succeeded(stop_reason: Option<&str>) -> bool {
    matches!(stop_reason, Some(reason) if reason != "cancelled")
}

#[allow(dead_code)]
pub(crate) async fn handle_acp_session_update(
    params: &Value,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    state: Option<&mut AcpPromptState>,
) {
    handle_acp_session_update_with_renderer(params, emitter, state, None).await;
}

/// 处理一段 ACP `agent_message_chunk` 文本（codex / opencode / gemini 共用）。
///
/// compact 发生后的文本现在按用户可见内容透传，不再在 ACP ingest 层做裁剪。
/// 这里仍保留 compact 检测，用于 runner 在本轮结束时写回
/// `ACP_NEEDS_SP_RESEED_KEY`，保证下一轮 system prompt 能正确 reseed。
pub(crate) async fn ingest_acp_message_chunk(
    text: &str,
    state: &mut AcpPromptState,
    emitter: &Arc<dyn AgentRunnerEmitter>,
) {
    let Some(text) = user_visible_acp_message_chunk(text) else {
        tracing::warn!("[acp] suppressed internal prompt echo agent_message_chunk");
        return;
    };
    let text = text.as_str();

    if RE_ACP_COMPACT_STATUS_TEXT.is_match(text) {
        if !state.compact_detected {
            tracing::info!(
                "[acp] runner internal compact signalled via status text: {:?}",
                text
            );
            state.compact_detected = true;
        }
    }

    let pre_full_len = state.full_reply.len();
    state.full_reply.push_str(text);
    state.pending_assistant_content.push_str(text);

    let scan_start = floor_char_boundary(
        &state.full_reply,
        pre_full_len.saturating_sub(ACP_BOUNDARY_SCAN_TAIL_BYTES),
    );
    if !state.compact_detected
        && RE_OPENCODE_SUMMARY_BOUNDARY
            .find(&state.full_reply[scan_start..])
            .is_some()
    {
        tracing::info!("[acp] opencode compact summary boundary detected in accumulated buffer");
        state.compact_detected = true;
    }

    emitter
        .emit(AgentRunnerEvent::StreamDelta {
            content: text.to_string(),
        })
        .await;
}

fn user_visible_acp_message_chunk(text: &str) -> Option<String> {
    let first_marker = [
        "### System Instructions ###",
        "### System Prompt ###",
        "### Skill Context ###",
        "### Conversation Context ###",
        "### User Prompt ###",
        "### Available Skills ###",
        "【Session 上下文】",
        "turn-0 可用技能索引",
    ]
    .iter()
    .filter_map(|marker| text.find(marker))
    .min();

    match first_marker {
        Some(idx) => {
            let trimmed = text[..idx].trim_end();
            (!trimmed.trim().is_empty()).then(|| trimmed.to_string())
        }
        None => Some(text.to_string()),
    }
}

/// 处理一条 ACP `usage_update`。共用 peak 跟踪 + "流内首次 used 与 prev_peak 比较"
/// 的 compact 骤降识别（覆盖 opencode 不发字面量的场景）。
///
/// `progress_stage` 由调用方决定（`"acp.usage"` / `"opencode.usage"` 等）以维持
/// 现有运营 metrics 兼容。
pub(crate) async fn ingest_acp_usage_update(
    used: u64,
    state: &mut AcpPromptState,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    progress_stage: &'static str,
) {
    emitter
        .emit(AgentRunnerEvent::Progress {
            stage: progress_stage,
            detail: Some(format!("used={used}")),
        })
        .await;

    state.current_prompt_peak_used = state.current_prompt_peak_used.max(used);
    let was_first = !state.usage_update_seen;
    state.usage_update_seen = true;

    if was_first
        && !state.compact_detected
        && let Some(prev_peak) = state.prev_prompt_peak_used
        && prev_peak >= 30_000
        && used * 2 < prev_peak
    {
        tracing::info!(
            "[acp] runner internal compact signalled via usage drop: prev_peak={} now_used={}",
            prev_peak,
            used
        );
        state.compact_detected = true;
    }
}

pub(crate) async fn handle_acp_session_update_with_renderer(
    params: &Value,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    mut state: Option<&mut AcpPromptState>,
    tool_status_renderer: Option<AcpToolStatusRenderer>,
) {
    let Some(update) = params.get("update") else {
        return;
    };
    let Some(kind) = update.get("sessionUpdate").and_then(|value| value.as_str()) else {
        return;
    };

    tracing::debug!("[acp] session/update kind={kind}");

    match kind {
        "agent_message_chunk" => {
            // Try nested content.text first (older protocol), then flat text/delta fields
            let text = update
                .get("content")
                .and_then(|value| value.get("text"))
                .and_then(|value| value.as_str())
                .or_else(|| update.get("text").and_then(|value| value.as_str()))
                .or_else(|| update.get("delta").and_then(|value| value.as_str()));
            let Some(text) = text else {
                tracing::debug!(
                    "[acp] agent_message_chunk: unrecognised payload format, skipping: {}",
                    update
                );
                return;
            };

            if let Some(state) = state.as_deref_mut() {
                ingest_acp_message_chunk(text, state, emitter).await;
            } else if let Some(text) = user_visible_acp_message_chunk(text) {
                emitter
                    .emit(AgentRunnerEvent::StreamDelta { content: text })
                    .await;
            } else {
                tracing::warn!("[acp] suppressed internal prompt echo agent_message_chunk");
            }
        }
        "agent_thought_chunk" => {
            // Try nested content.text first, then flat text field
            let text = update
                .get("content")
                .and_then(|value| value.get("text"))
                .and_then(|value| value.as_str())
                .or_else(|| update.get("text").and_then(|value| value.as_str()));
            let Some(text) = text else {
                return;
            };
            emitter
                .emit(AgentRunnerEvent::StreamThought {
                    thought: text.to_string(),
                })
                .await;
        }
        "tool_call" => {
            let tool = update
                .get("title")
                .and_then(|value| value.as_str())
                .or_else(|| update.get("kind").and_then(|value| value.as_str()))
                .unwrap_or("tool")
                .to_string();
            if let Some(state) = state.as_deref_mut() {
                if !state.pending_assistant_content.is_empty() {
                    flush_pending_assistant_message(state);
                }
                capture_tool_start(state, update, &tool);
            }
            let default_reasoning = resolve_tool_reasoning(&tool, extract_acp_reasoning(update));
            let rendered = tool_status_renderer.map(|renderer| {
                renderer(
                    update,
                    AcpToolRenderPhase::Start,
                    &tool,
                    None,
                    default_reasoning.clone(),
                )
            });
            emitter
                .emit(AgentRunnerEvent::ToolStatus {
                    tool: rendered
                        .as_ref()
                        .map(|value| value.tool.clone())
                        .unwrap_or_else(|| tool.clone()),
                    status: "start".to_string(),
                    message: rendered.as_ref().and_then(|value| value.message.clone()),
                    reasoning: rendered
                        .as_ref()
                        .and_then(|value| value.reasoning.clone())
                        .or(default_reasoning),
                })
                .await;
        }
        "tool_call_update" => {
            let tool = update
                .get("title")
                .and_then(|value| value.as_str())
                .or_else(|| update.get("kind").and_then(|value| value.as_str()))
                .unwrap_or("tool")
                .to_string();
            let status = update
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if status == "completed" {
                if let Some(state) = state.as_deref_mut() {
                    if let Some(result) = extract_tool_result(update) {
                        capture_tool_finish(state, update, &tool, result);
                    }
                }
                let rendered = tool_status_renderer.map(|renderer| {
                    renderer(
                        update,
                        AcpToolRenderPhase::Done,
                        &tool,
                        Some("工具执行完成".to_string()),
                        None,
                    )
                });
                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool: rendered
                            .as_ref()
                            .map(|value| value.tool.clone())
                            .unwrap_or_else(|| tool.clone()),
                        status: "done".to_string(),
                        message: rendered
                            .as_ref()
                            .and_then(|value| value.message.clone())
                            .or_else(|| Some("工具执行完成".to_string())),
                        reasoning: rendered.as_ref().and_then(|value| value.reasoning.clone()),
                    })
                    .await;
            } else if status == "failed" {
                if let Some(state) = state.as_deref_mut() {
                    if let Some(result) = extract_tool_failure(update) {
                        capture_tool_finish(state, update, &tool, result);
                    }
                }
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "acp.tool_failed",
                        detail: Some(format!("tool={tool}")),
                    })
                    .await;
            }
        }
        "usage_update" => {
            if let Some(used) = update.get("used").and_then(|value| value.as_u64())
                && let Some(state) = state.as_deref_mut()
            {
                ingest_acp_usage_update(used, state, emitter, "acp.usage").await;
            } else if let Some(used) = update.get("used").and_then(|value| value.as_u64()) {
                // 无 state 上下文时只投 Progress（保持现状）
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "acp.usage",
                        detail: Some(format!("used={used}")),
                    })
                    .await;
            }
        }
        _ => {}
    }
}
