//! ACP tool call 状态机:把「流式 tool_call → tool_call_update」事件
//! 累积成完整的 `ToolCallMade` + 可落盘的 context assistant/tool 消息对。
//!
//! 关键不变量:
//! - 一个 `tool_call_id` 只 finish 一次(`completed_tool_call_ids` 去重)
//! - assistant 消息里的 `tool_calls` 用 OpenAI 兼容形状(`id`/`type`/`function`)
//!   通过 `build_openai_tool_call_value` 构造,这样 restore_context 下次能直接塞给
//!   function_calling runner

use hone_core::agent::{AgentMessage, ToolCallMade};
use serde_json::{Value, json};

use super::extract::{extract_tool_arguments, extract_tool_call_id, extract_tool_name};
use super::state::{AcpPromptState, AcpToolCallRecord};

pub(super) fn capture_tool_start(state: &mut AcpPromptState, update: &Value, fallback_name: &str) {
    let Some(tool_call_id) = extract_tool_call_id(update) else {
        return;
    };
    let tool_name = extract_tool_name(update).unwrap_or_else(|| fallback_name.to_string());
    let arguments = extract_tool_arguments(update);
    state
        .pending_assistant_tool_calls
        .push(build_openai_tool_call_value(
            &tool_call_id,
            &tool_name,
            &arguments,
        ));
    state.pending_tool_calls.insert(
        tool_call_id,
        AcpToolCallRecord {
            name: tool_name,
            arguments,
        },
    );
}

pub(super) fn flush_pending_assistant_message(state: &mut AcpPromptState) {
    if state.pending_assistant_content.is_empty() && state.pending_assistant_tool_calls.is_empty() {
        return;
    }

    let content = std::mem::take(&mut state.pending_assistant_content);
    let tool_calls = if state.pending_assistant_tool_calls.is_empty() {
        None
    } else {
        Some(std::mem::take(&mut state.pending_assistant_tool_calls))
    };

    state.context_messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some(content),
        tool_calls,
        tool_call_id: None,
        name: None,
        metadata: None,
    });
}

pub(super) fn capture_tool_finish(
    state: &mut AcpPromptState,
    update: &Value,
    fallback_name: &str,
    result: Value,
) {
    let Some(tool_call_id) = extract_tool_call_id(update) else {
        return;
    };
    if state.completed_tool_call_ids.contains(&tool_call_id) {
        return;
    }

    let pending = state.pending_tool_calls.remove(&tool_call_id);
    let tool_name = pending
        .as_ref()
        .map(|record| record.name.clone())
        .or_else(|| extract_tool_name(update))
        .unwrap_or_else(|| fallback_name.to_string());
    let arguments = pending
        .map(|record| record.arguments)
        .unwrap_or_else(|| extract_tool_arguments(update));

    state.completed_tool_call_ids.insert(tool_call_id.clone());
    state.finished_tool_calls.push(ToolCallMade {
        name: tool_name,
        arguments,
        result,
        tool_call_id: Some(tool_call_id),
    });
    flush_pending_assistant_message(state);
    state.context_messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some(stringify_tool_result(
            &state
                .finished_tool_calls
                .last()
                .map(|call| call.result.clone())
                .unwrap_or(Value::Null),
        )),
        tool_calls: None,
        tool_call_id: state
            .finished_tool_calls
            .last()
            .and_then(|call| call.tool_call_id.clone()),
        name: state
            .finished_tool_calls
            .last()
            .map(|call| call.name.clone()),
        metadata: None,
    });
}

pub(crate) fn extract_finished_tool_calls(state: AcpPromptState) -> Vec<ToolCallMade> {
    state.finished_tool_calls
}

pub(crate) fn finalize_context_messages(state: &mut AcpPromptState) -> Vec<AgentMessage> {
    flush_pending_assistant_message(state);
    state.context_messages.clone()
}

fn build_openai_tool_call_value(tool_call_id: &str, tool_name: &str, arguments: &Value) -> Value {
    json!({
        "id": tool_call_id,
        "type": "function",
        "function": {
            "name": tool_name,
            "arguments": stringify_tool_arguments(arguments),
        }
    })
}

fn stringify_tool_arguments(arguments: &Value) -> String {
    if let Some(text) = arguments.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    serde_json::to_string(arguments).unwrap_or_else(|_| "null".to_string())
}

pub(super) fn stringify_tool_result(result: &Value) -> String {
    if let Some(text) = result.as_str() {
        return text.to_string();
    }
    serde_json::to_string(result).unwrap_or_else(|_| "null".to_string())
}
