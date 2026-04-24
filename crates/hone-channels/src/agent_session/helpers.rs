//! Agent session 纯函数 helper 集合。
//!
//! 这里只放「无 self / 无 I/O / 无 async」的纯函数和只读常量:
//! - 关于 runner 结果如何判读(`should_return_runner_result` /
//!   `is_context_overflow_error_text`)
//! - 关于哪些 tool 调用可以落盘、哪些只做一次性副作用
//!   (`should_persist_tool_result` / `matches_skill_runtime_tool_name`)
//! - 关于如何把 runner 响应序列化成 session 可持久化的 normalized message
//!   (`persistable_turn_from_response` + `merge_message_metadata`)
//! - 其他一眼能看完的小工具(`restore_limit_before_compaction` /
//!   `sanitize_assistant_context_content` / `CompactCommand`)
//!
//! 保持纯函数是刻意的:测试里可以直接对它们断言,不需要构造
//! `AgentSession` / `HoneBotCore` 这类「带整个世界」的上下文。

use hone_core::agent::{
    AgentResponse, NormalizedConversationMessage, NormalizedConversationPart, ToolCallMade,
};
use hone_core::{HoneConfig, SessionIdentity};
use hone_memory::build_assistant_message_metadata;
use serde_json::Value;
use std::collections::HashMap;

use crate::outbound::{LOCAL_IMAGE_CONTEXT_PLACEHOLDER, replace_local_image_markers};
use crate::runners::AgentRunnerResult;
use crate::runtime::sanitize_user_visible_output;

pub(super) const EMPTY_SUCCESS_RETRY_LIMIT: usize = 2;
pub(super) const CONTEXT_OVERFLOW_RECOVERY_LIMIT: usize = 1;
pub(super) const DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT: usize = 20;
pub(super) const CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT: usize = 6;
pub(super) const CONTEXT_OVERFLOW_FALLBACK_MESSAGE: &str = "当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。";

/// 决定一次 run 在送去 runner 前,restore_context 时保留多少条历史。
///
/// 群聊场景要跟着用户配置的 compress 阈值走(保留到下次 compact 前的最大
/// window);直聊则用常量上限,避免在 compact 还没发生时就把最近历史切断。
pub(super) fn restore_limit_before_compaction(
    config: &HoneConfig,
    session_identity: &SessionIdentity,
) -> Option<usize> {
    if session_identity.is_group() {
        Some(
            config
                .group_context
                .recent_context_limit
                .max(config.group_context.compress_threshold_messages)
                .max(1),
        )
    } else {
        Some(DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT)
    }
}

pub(super) fn should_return_runner_result(result: &AgentRunnerResult) -> bool {
    // 失败直接返回；成功时必须拿到正文，不能因为只有工具调用就把空答复当成成功。
    //
    // 注意：`streamed_output` 仅表示 runner 具备流式能力，不代表这次真的输出过内容。
    // opencode_acp 会始终把它设为 true，因此不能再把它当成“已有输出”的依据，
    // 否则空回复成功态会被直接放过，前端就可能一直停留在“思考中”。
    !result.response.success || !result.response.content.trim().is_empty()
}

pub(super) fn is_context_overflow_error_text(text: &str) -> bool {
    crate::runtime::is_context_overflow_error(text)
}

pub(super) fn should_persist_tool_result(call: &ToolCallMade) -> bool {
    if matches_skill_runtime_tool_name(&call.name) {
        return false;
    }
    if call.name == "web_search" {
        if call
            .result
            .get("status")
            .and_then(|value| value.as_str())
            .is_some_and(|status| status == "unavailable")
        {
            return false;
        }
        if call.result.get("error").is_some() {
            return false;
        }
    }
    true
}

pub(super) fn matches_skill_runtime_tool_name(name: &str) -> bool {
    matches!(
        name.trim(),
        "skill_tool"
            | "load_skill"
            | "discover_skills"
            | "hone/skill_tool"
            | "hone/load_skill"
            | "hone/discover_skills"
            | "Tool: hone/skill_tool"
            | "Tool: hone/load_skill"
            | "Tool: hone/discover_skills"
    )
}

/// 用户输入里解析出来的 `/compact` 指令（含可选的人类提示）。
#[derive(Debug, Clone)]
pub(super) struct CompactCommand {
    pub(super) instructions: Option<String>,
}

pub(super) fn merge_message_metadata(
    base: Option<HashMap<String, Value>>,
    extra: HashMap<String, Value>,
) -> Option<HashMap<String, Value>> {
    let mut merged = base.unwrap_or_default();
    for (key, value) in extra {
        merged.insert(key, value);
    }
    Some(merged)
}

/// 把一轮 runner 响应转成可持久化的 `assistant` message。
///
/// 注意点:
/// - `tool_calls_made` 里被 `should_persist_tool_result` 过滤掉的(skill_runtime /
///   不可用的 web_search)不写进 metadata,避免下次 restore 时还原出一个已经失效
///   的 tool 轮次;
/// - `content` 为空的情况直接返回 None —— runner 侧已经用 fallback 文案兜底,
///   这里不再塞一个「空 assistant 消息」给 session。
pub(super) fn persistable_turn_from_response(
    response: &AgentResponse,
    metadata: Option<HashMap<String, Value>>,
) -> Option<NormalizedConversationMessage> {
    let persisted_tool_calls = response
        .tool_calls_made
        .iter()
        .filter(|call| should_persist_tool_result(call))
        .map(|call| {
            serde_json::json!({
                "id": call.tool_call_id.clone().unwrap_or_default(),
                "type": "function",
                "function": {
                    "name": call.name,
                    "arguments": serde_json::to_string(&call.arguments)
                        .unwrap_or_else(|_| "null".to_string()),
                }
            })
        })
        .collect::<Vec<_>>();
    let tool_call_metadata = build_assistant_message_metadata(&persisted_tool_calls);
    let metadata = if tool_call_metadata.is_empty() {
        metadata
    } else {
        merge_message_metadata(metadata, tool_call_metadata)
    };

    let mut content = Vec::new();

    if !response.content.trim().is_empty() {
        content.push(NormalizedConversationPart {
            part_type: "final".to_string(),
            text: Some(response.content.trim().to_string()),
            id: None,
            name: None,
            args: None,
            result: None,
            metadata: None,
        });
    }

    if content.is_empty() {
        None
    } else {
        Some(NormalizedConversationMessage {
            role: "assistant".to_string(),
            content,
            status: Some("completed".to_string()),
            metadata,
        })
    }
}

/// 恢复上下文时对 assistant 历史内容做的二次脱敏。把本地图片 marker 压成
/// 统一占位符,避免历史里出现真实沙盒路径泄露给下一轮 runner。
pub(super) fn sanitize_assistant_context_content(content: &str) -> String {
    let sanitized = sanitize_user_visible_output(content).content;
    replace_local_image_markers(&sanitized, LOCAL_IMAGE_CONTEXT_PLACEHOLDER)
}
