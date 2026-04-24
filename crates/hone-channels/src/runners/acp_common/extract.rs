//! ACP JSON session update 里抽字段的纯函数集。
//!
//! 各家 ACP runner(codex / gemini / opencode)对 tool call 的 key 命名
//! 不一致:有的叫 `toolCallId`,有的叫 `callId`,有的藏在 `toolCall.id`。
//! 这里统一用「尝试一组候选 key」的方式,把分歧收敛到 caller 看不见的地方。

use serde_json::{Value, json};

/// 沿 `keys` 顺序找第一个是非空字符串的字段。trim 后返回;空串不算命中。
pub(super) fn extract_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(found) = value.get(*key).and_then(|value| value.as_str()) {
            let trimmed = found.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

pub(super) fn extract_value_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(found) = value.get(*key) {
            if !found.is_null() {
                return Some(found);
            }
        }
    }
    None
}

/// 如果字段值恰好是一个字符串形式的 JSON(`"\"{\\\"x\\\":1}\""`),
/// 解开成真正的 `Value`;解不开就原样返回字符串本体。
/// 各家 runner 对 tool arguments 的序列化方式不一致,这里兜底。
pub(super) fn parse_embedded_json(value: &Value) -> Value {
    if let Some(text) = value.as_str() {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(trimmed.to_string()))
        }
    } else {
        value.clone()
    }
}

pub(super) fn extract_tool_call_id(update: &Value) -> Option<String> {
    extract_string_field(update, &["toolCallId", "callId", "id"]).or_else(|| {
        update
            .get("toolCall")
            .and_then(|value| extract_string_field(value, &["id", "toolCallId", "callId"]))
    })
}

pub(super) fn extract_tool_name(update: &Value) -> Option<String> {
    extract_string_field(update, &["title", "name", "toolName", "kind"]).or_else(|| {
        update
            .get("toolCall")
            .and_then(|value| extract_string_field(value, &["title", "name", "toolName", "kind"]))
    })
}

pub(super) fn extract_tool_arguments(update: &Value) -> Value {
    if let Some(raw) = extract_value_field(update, &["arguments", "args", "input", "parameters"]) {
        return parse_embedded_json(raw);
    }
    if let Some(tool_call) = update.get("toolCall") {
        if let Some(raw) =
            extract_value_field(tool_call, &["arguments", "args", "input", "parameters"])
        {
            return parse_embedded_json(raw);
        }
    }
    Value::Null
}

pub(super) fn extract_tool_result(update: &Value) -> Option<Value> {
    if let Some(raw) = extract_value_field(update, &["result", "output", "response"]) {
        return Some(parse_embedded_json(raw));
    }
    if let Some(content) = update.get("content") {
        if let Some(raw) = extract_value_field(content, &["result", "output", "text"]) {
            return Some(parse_embedded_json(raw));
        }
        if content.is_string() || content.is_array() {
            return Some(parse_embedded_json(content));
        }
    }
    update
        .get("toolCall")
        .and_then(|tool_call| extract_value_field(tool_call, &["result", "output", "response"]))
        .map(parse_embedded_json)
}

pub(super) fn extract_tool_failure(update: &Value) -> Option<Value> {
    extract_string_field(update, &["message", "detail", "description", "subtitle"])
        .map(|message| json!({ "error": message }))
}

pub(super) fn extract_acp_reasoning(update: &Value) -> Option<String> {
    for key in ["message", "text", "detail", "description", "subtitle"] {
        if let Some(value) = update.get(key).and_then(|value| value.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    update
        .get("content")
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}
