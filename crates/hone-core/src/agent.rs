//! 通用 Agent 接口定义
//!
//! 保存会话历史、工具调用和 legacy [`Agent`] 适配器共享的响应形状。
//! 渠道运行时的主抽象在 `hone-channels::runners::AgentRunner`。

use crate::ActorIdentity;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Runtime-only marker for an invoked-skill prompt restored from durable
/// Session metadata. Agent implementations use it to keep explicit skill
/// context separate from bounded conversational user references.
pub const RESTORED_INVOKED_SKILL_PROMPT_METADATA_KEY: &str = "skill_runtime.restored_prompt";

/// 已执行的工具调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMade {
    pub name: String,
    pub arguments: Value,
    pub result: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Legacy Agent 流式事件形状。
///
/// 现代渠道运行时使用 `AgentRunnerEvent` 做流式输出；这里保留为旧 API 的
/// 兼容类型。
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// 文本 token
    Token { content: String },
    /// 工具调用开始
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
    },
    /// 工具调用结果
    ToolResult {
        id: String,
        name: String,
        result: Value,
    },
    /// 错误
    Error { message: String },
    /// 完成
    Done {
        full_response: String,
        tool_calls_made: Vec<ToolCallMade>,
    },
}

/// Agent 单轮响应。
///
/// 即使底层 runner 已经流式输出，收尾阶段也会归并成这个结构用于持久化、
/// 审计和调用方状态判断。
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls_made: Vec<ToolCallMade>,
    pub iterations: u32,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedConversationPart {
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedConversationMessage {
    pub role: String,
    pub content: Vec<NormalizedConversationPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

/// Agent 消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

/// Agent 上下文管理
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub session_id: String,
    pub messages: Vec<AgentMessage>,
    pub metadata: HashMap<String, Value>,
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            messages: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(AgentMessage {
            role: "user".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: None,
        });
    }

    pub fn add_assistant_message(&mut self, content: &str, tool_calls: Option<Vec<Value>>) {
        self.add_assistant_message_with_metadata(content, tool_calls, None);
    }

    pub fn add_assistant_message_with_metadata(
        &mut self,
        content: &str,
        tool_calls: Option<Vec<Value>>,
        metadata: Option<HashMap<String, Value>>,
    ) {
        self.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: Some(content.to_string()),
            tool_calls,
            tool_call_id: None,
            name: None,
            metadata,
        });
    }

    pub fn add_tool_result(&mut self, tool_call_id: &str, tool_name: &str, result: &str) {
        self.messages.push(AgentMessage {
            role: "tool".to_string(),
            content: Some(result.to_string()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
            name: Some(tool_name.to_string()),
            metadata: None,
        });
    }

    pub fn set_actor_identity(&mut self, actor: &ActorIdentity) {
        self.metadata.insert(
            "actor".to_string(),
            serde_json::to_value(actor).unwrap_or(Value::Null),
        );
    }

    pub fn actor_identity(&self) -> Option<ActorIdentity> {
        self.metadata
            .get("actor")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }

    /// 转换为旧版通用 JSON 消息格式。
    pub fn to_messages(&self) -> Vec<Value> {
        self.messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect()
    }

    pub fn normalized_history(&self) -> Vec<NormalizedConversationMessage> {
        normalize_agent_messages(&self.messages)
    }

    pub fn normalized_history_json(&self) -> Option<String> {
        let normalized = self.normalized_history();
        if normalized.is_empty() {
            None
        } else {
            serde_json::to_string_pretty(&normalized).ok()
        }
    }
}

fn parse_json_or_string(input: &str) -> Value {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Value::String(String::new())
    } else {
        serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(trimmed.to_string()))
    }
}

fn parse_tool_call_arguments(arguments: &Value) -> Value {
    if let Some(text) = arguments.as_str() {
        parse_json_or_string(text)
    } else {
        arguments.clone()
    }
}

fn assistant_text_part(message: &AgentMessage) -> Option<NormalizedConversationPart> {
    message
        .content
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| NormalizedConversationPart {
            part_type: "text".to_string(),
            text: Some(text.to_string()),
            id: None,
            name: None,
            args: None,
            result: None,
            metadata: message.metadata.clone(),
        })
}

fn assistant_tool_call_parts(message: &AgentMessage) -> Vec<NormalizedConversationPart> {
    message
        .tool_calls
        .as_ref()
        .into_iter()
        .flatten()
        .map(|tool_call| NormalizedConversationPart {
            part_type: "tool_call".to_string(),
            text: None,
            id: tool_call
                .get("id")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            name: tool_call
                .get("function")
                .and_then(|value| value.get("name"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            args: tool_call
                .get("function")
                .and_then(|value| value.get("arguments"))
                .map(parse_tool_call_arguments),
            result: None,
            metadata: None,
        })
        .collect()
}

fn tool_result_part(message: &AgentMessage) -> Option<NormalizedConversationPart> {
    let has_identity = message
        .tool_call_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || message
            .name
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
    let has_result = message
        .content
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    if !has_identity && !has_result {
        return None;
    }

    Some(NormalizedConversationPart {
        part_type: "tool_result".to_string(),
        text: None,
        id: message.tool_call_id.clone(),
        name: message.name.clone(),
        args: None,
        result: message
            .content
            .as_deref()
            .map(parse_json_or_string)
            .or(Some(Value::Null)),
        metadata: message.metadata.clone(),
    })
}

fn finalize_assistant_turn(parts: &mut [NormalizedConversationPart]) {
    let last_tool_index = parts
        .iter()
        .rposition(|part| matches!(part.part_type.as_str(), "tool_call" | "tool_result"));
    let last_text_index = parts.iter().rposition(|part| part.part_type == "text");

    for (index, part) in parts.iter_mut().enumerate() {
        if part.part_type != "text" {
            continue;
        }
        let becomes_final = Some(index) == last_text_index
            && last_text_index.is_some_and(|text_index| {
                last_tool_index
                    .map(|tool_index| text_index > tool_index)
                    .unwrap_or(true)
            });
        part.part_type = if becomes_final {
            "final".to_string()
        } else {
            "progress".to_string()
        };
    }
}

pub fn normalize_agent_messages(messages: &[AgentMessage]) -> Vec<NormalizedConversationMessage> {
    let mut normalized = Vec::new();
    let mut current_assistant: Option<NormalizedConversationMessage> = None;

    let flush_assistant =
        |normalized: &mut Vec<NormalizedConversationMessage>,
         current: &mut Option<NormalizedConversationMessage>| {
            if let Some(mut assistant) = current.take() {
                finalize_assistant_turn(&mut assistant.content);
                if !assistant.content.is_empty() {
                    normalized.push(assistant);
                }
            }
        };

    for message in messages {
        match message.role.as_str() {
            "user" => {
                flush_assistant(&mut normalized, &mut current_assistant);
                let Some(text) = message
                    .content
                    .as_deref()
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                else {
                    continue;
                };
                normalized.push(NormalizedConversationMessage {
                    role: "user".to_string(),
                    content: vec![NormalizedConversationPart {
                        part_type: "text".to_string(),
                        text: Some(text.to_string()),
                        id: None,
                        name: None,
                        args: None,
                        result: None,
                        metadata: message.metadata.clone(),
                    }],
                    status: Some("completed".to_string()),
                    metadata: None,
                });
            }
            "assistant" => {
                let assistant =
                    current_assistant.get_or_insert_with(|| NormalizedConversationMessage {
                        role: "assistant".to_string(),
                        content: Vec::new(),
                        status: Some("completed".to_string()),
                        metadata: None,
                    });
                if let Some(part) = assistant_text_part(message) {
                    assistant.content.push(part);
                }
                assistant
                    .content
                    .extend(assistant_tool_call_parts(message).into_iter());
            }
            "tool" => {
                let assistant =
                    current_assistant.get_or_insert_with(|| NormalizedConversationMessage {
                        role: "assistant".to_string(),
                        content: Vec::new(),
                        status: Some("completed".to_string()),
                        metadata: None,
                    });
                if let Some(part) = tool_result_part(message) {
                    assistant.content.push(part);
                }
            }
            _ => {}
        }
    }

    flush_assistant(&mut normalized, &mut current_assistant);
    normalized
}

pub fn final_assistant_message_content(messages: &[AgentMessage], fallback: String) -> String {
    messages
        .iter()
        .rev()
        .filter(|message| message.role == "assistant")
        .filter_map(|message| message.content.as_deref())
        .map(str::trim)
        .find(|content| !content.is_empty())
        .map(ToString::to_string)
        .unwrap_or(fallback)
}

fn build_tool_call_value_from_part(part: &NormalizedConversationPart) -> Value {
    let arguments = part.args.clone().unwrap_or(Value::Null);
    serde_json::json!({
        "id": part.id.clone().unwrap_or_default(),
        "type": "function",
        "function": {
            "name": part.name.clone().unwrap_or_default(),
            "arguments": serde_json::to_string(&arguments).unwrap_or_else(|_| "null".to_string()),
        }
    })
}

fn tool_result_to_content(part: &NormalizedConversationPart) -> String {
    match part.result.as_ref() {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Object(map)) => {
            for key in ["formatted_output", "aggregated_output", "stdout", "text"] {
                if let Some(text) = map.get(key).and_then(|value| value.as_str())
                    && !text.trim().is_empty()
                {
                    return text.to_string();
                }
            }
            serde_json::to_string(&Value::Object(map.clone()))
                .unwrap_or_else(|_| "null".to_string())
        }
        Some(value) => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
        None => String::new(),
    }
}

fn effective_part_metadata(
    part_metadata: &Option<HashMap<String, Value>>,
    message_metadata: &Option<HashMap<String, Value>>,
) -> Option<HashMap<String, Value>> {
    part_metadata.clone().or_else(|| message_metadata.clone())
}

fn denormalized_text_from_parts(parts: &[NormalizedConversationPart]) -> String {
    parts
        .iter()
        .filter_map(|part| part.text.as_deref())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn first_part_metadata_or_message(
    message: &NormalizedConversationMessage,
) -> Option<HashMap<String, Value>> {
    message
        .content
        .iter()
        .find_map(|part| part.metadata.clone())
        .or_else(|| message.metadata.clone())
}

fn denormalize_text_role(message: &NormalizedConversationMessage, role: &str) -> Vec<AgentMessage> {
    let text = denormalized_text_from_parts(&message.content);
    if text.is_empty() {
        return Vec::new();
    }

    vec![AgentMessage {
        role: role.to_string(),
        content: Some(text),
        tool_calls: None,
        tool_call_id: None,
        name: None,
        metadata: first_part_metadata_or_message(message),
    }]
}

#[derive(Default)]
struct AssistantDenormalizationState {
    out: Vec<AgentMessage>,
    pending_text: String,
    pending_tool_calls: Vec<Value>,
    pending_metadata: Option<HashMap<String, Value>>,
}

impl AssistantDenormalizationState {
    fn flush(&mut self, message_metadata: &Option<HashMap<String, Value>>) {
        if self.pending_text.trim().is_empty() && self.pending_tool_calls.is_empty() {
            return;
        }

        self.out.push(AgentMessage {
            role: "assistant".to_string(),
            content: Some(std::mem::take(&mut self.pending_text)),
            tool_calls: if self.pending_tool_calls.is_empty() {
                None
            } else {
                Some(std::mem::take(&mut self.pending_tool_calls))
            },
            tool_call_id: None,
            name: None,
            metadata: self
                .pending_metadata
                .take()
                .or_else(|| message_metadata.clone()),
        });
    }

    fn push_text_part(
        &mut self,
        part: &NormalizedConversationPart,
        message_metadata: &Option<HashMap<String, Value>>,
    ) {
        if !self.pending_text.trim().is_empty() {
            self.flush(message_metadata);
        }
        self.pending_text = part.text.clone().unwrap_or_default();
        self.pending_metadata = effective_part_metadata(&part.metadata, message_metadata);
    }

    fn push_tool_call_part(
        &mut self,
        part: &NormalizedConversationPart,
        message_metadata: &Option<HashMap<String, Value>>,
    ) {
        self.pending_tool_calls
            .push(build_tool_call_value_from_part(part));
        if self.pending_metadata.is_none() {
            self.pending_metadata = effective_part_metadata(&part.metadata, message_metadata);
        }
    }

    fn push_tool_result_part(
        &mut self,
        part: &NormalizedConversationPart,
        message_metadata: &Option<HashMap<String, Value>>,
    ) {
        self.flush(message_metadata);
        self.out.push(AgentMessage {
            role: "tool".to_string(),
            content: Some(tool_result_to_content(part)),
            tool_calls: None,
            tool_call_id: part.id.clone(),
            name: part.name.clone(),
            metadata: effective_part_metadata(&part.metadata, message_metadata),
        });
    }
}

fn denormalize_assistant_message(message: &NormalizedConversationMessage) -> Vec<AgentMessage> {
    let mut state = AssistantDenormalizationState::default();

    for part in &message.content {
        match part.part_type.as_str() {
            "text" | "progress" | "final" => state.push_text_part(part, &message.metadata),
            "tool_call" => state.push_tool_call_part(part, &message.metadata),
            "tool_result" => state.push_tool_result_part(part, &message.metadata),
            _ => {}
        }
    }

    state.flush(&message.metadata);
    state.out
}

pub fn denormalize_normalized_message(
    message: &NormalizedConversationMessage,
) -> Vec<AgentMessage> {
    match message.role.as_str() {
        "user" => denormalize_text_role(message, "user"),
        "assistant" => denormalize_assistant_message(message),
        other => denormalize_text_role(message, other),
    }
}

/// Legacy 可插拔 Agent 接口。
///
/// 新的渠道执行路径应实现 `hone-channels::runners::AgentRunner`。
#[async_trait]
pub trait Agent: Send + Sync {
    /// 运行单次交互
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(part_type: &str, text: Option<&str>) -> NormalizedConversationPart {
        NormalizedConversationPart {
            part_type: part_type.to_string(),
            text: text.map(ToString::to_string),
            id: None,
            name: None,
            args: None,
            result: None,
            metadata: None,
        }
    }

    #[test]
    fn denormalize_text_role_joins_non_empty_text_parts() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), Value::String("part".to_string()));
        let mut first = part("text", Some(" first "));
        first.metadata = Some(metadata.clone());

        let messages = denormalize_normalized_message(&NormalizedConversationMessage {
            role: "user".to_string(),
            content: vec![
                first,
                part("text", Some("  ")),
                part("text", Some("second")),
            ],
            status: None,
            metadata: None,
        });

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content.as_deref(), Some("first\nsecond"));
        assert_eq!(messages[0].metadata, Some(metadata));
    }

    #[test]
    fn denormalize_assistant_preserves_tool_call_and_result_order() {
        let mut tool_call = part("tool_call", None);
        tool_call.id = Some("call-1".to_string());
        tool_call.name = Some("lookup".to_string());
        tool_call.args = Some(serde_json::json!({ "ticker": "HONE" }));

        let mut tool_result = part("tool_result", None);
        tool_result.id = Some("call-1".to_string());
        tool_result.name = Some("lookup".to_string());
        tool_result.result = Some(Value::String("result text".to_string()));

        let messages = denormalize_normalized_message(&NormalizedConversationMessage {
            role: "assistant".to_string(),
            content: vec![
                part("text", Some("thinking")),
                tool_call,
                tool_result,
                part("final", Some("done")),
            ],
            status: Some("completed".to_string()),
            metadata: None,
        });

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "assistant");
        assert_eq!(messages[0].content.as_deref(), Some("thinking"));
        assert_eq!(messages[0].tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(messages[1].role, "tool");
        assert_eq!(messages[1].content.as_deref(), Some("result text"));
        assert_eq!(messages[1].tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(messages[2].role, "assistant");
        assert_eq!(messages[2].content.as_deref(), Some("done"));
        assert!(messages[2].tool_calls.is_none());
    }
}
