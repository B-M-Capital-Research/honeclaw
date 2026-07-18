//! LLM Provider trait 定义

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// OpenAI 兼容的消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// 函数调用详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Token 使用统计
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// 普通对话响应
#[derive(Debug, Clone)]
pub struct ChatResult {
    pub content: String,
    pub usage: Option<TokenUsage>,
}

/// LLM 带工具调用的响应
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Option<TokenUsage>,
}

/// Structured events produced by a tool-capable chat completion stream.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatStreamEvent {
    ContentDelta(String),
    ReasoningDelta(String),
    ToolCallDelta {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments: String,
    },
    Usage(TokenUsage),
}

/// Per-round tool selection mode for native function-calling streams.
///
/// `Required` is used only by Agent protocols that expose an explicit terminal
/// control tool. It constrains the wire protocol (the model must choose a
/// tool), but does not choose which business tool the Agent should call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolChoiceMode {
    #[default]
    Auto,
    Required,
}

pub fn chat_stream_events_from_value(value: &Value) -> Vec<ChatStreamEvent> {
    let mut events = Vec::new();
    if let Some(usage) = value.get("usage") {
        events.push(ChatStreamEvent::Usage(TokenUsage {
            prompt_tokens: usage
                .get("prompt_tokens")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok()),
            completion_tokens: usage
                .get("completion_tokens")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok()),
            total_tokens: usage
                .get("total_tokens")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok()),
        }));
    }
    let Some(delta) = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("delta"))
    else {
        return events;
    };
    if let Some(reasoning) = delta
        .get("reasoning_content")
        .or_else(|| delta.get("reasoning"))
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
    {
        events.push(ChatStreamEvent::ReasoningDelta(reasoning.to_string()));
    }
    if let Some(content) = delta
        .get("content")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
    {
        events.push(ChatStreamEvent::ContentDelta(content.to_string()));
    }
    if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
        for tool_call in tool_calls {
            let function = tool_call.get("function");
            events.push(ChatStreamEvent::ToolCallDelta {
                index: tool_call
                    .get("index")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(0),
                id: tool_call
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                name: function
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                arguments: function
                    .and_then(|value| value.get("arguments"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            });
        }
    }
    events
}

/// Per-request generation options resolved from an LLM profile.
///
/// Legacy callers still pass only `model` through the trait. Providers created
/// from `llm.profiles.*` carry these options internally and merge them into
/// non-streaming chat/chat_with_tools request bodies for OpenRouter and generic
/// OpenAI-compatible providers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LlmRequestOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Vec<String>,
    pub seed: Option<u64>,
    pub reasoning: Option<Value>,
    pub response_format: Option<Value>,
    pub tool_choice: Option<Value>,
    pub parallel_tool_calls: Option<bool>,
    pub extra_body: Map<String, Value>,
}

impl LlmRequestOptions {
    pub fn with_max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.max_tokens.is_none()
            && self.temperature.is_none()
            && self.top_p.is_none()
            && self.stop.is_empty()
            && self.seed.is_none()
            && self.reasoning.is_none()
            && self.response_format.is_none()
            && self.tool_choice.is_none()
            && self.parallel_tool_calls.is_none()
            && self.extra_body.is_empty()
    }

    pub fn apply_to_body(&self, body: &mut Map<String, Value>, fallback_max_tokens: u16) {
        body.insert(
            "max_tokens".to_string(),
            Value::from(self.max_tokens.unwrap_or(fallback_max_tokens as u32)),
        );
        if let Some(value) = self.temperature {
            body.insert("temperature".to_string(), Value::from(value));
        }
        if let Some(value) = self.top_p {
            body.insert("top_p".to_string(), Value::from(value));
        }
        if !self.stop.is_empty() {
            body.insert("stop".to_string(), Value::from(self.stop.clone()));
        }
        if let Some(value) = self.seed {
            body.insert("seed".to_string(), Value::from(value));
        }
        if let Some(value) = &self.reasoning {
            body.insert("reasoning".to_string(), value.clone());
        }
        if let Some(value) = &self.response_format {
            body.insert("response_format".to_string(), value.clone());
        }
        if let Some(value) = &self.tool_choice {
            body.insert("tool_choice".to_string(), value.clone());
        }
        if let Some(value) = self.parallel_tool_calls {
            body.insert("parallel_tool_calls".to_string(), Value::from(value));
        }
        for (key, value) in &self.extra_body {
            body.insert(key.clone(), value.clone());
        }
    }
}

/// LLM Provider trait
///
/// 所有 LLM 提供商需要实现此 trait。
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 普通对话
    async fn chat(
        &self,
        messages: &[Message],
        model: Option<&str>,
    ) -> hone_core::HoneResult<ChatResult>;

    /// 带工具的对话（Function Calling）
    async fn chat_with_tools(
        &self,
        messages: &[Message],
        tools: &[Value],
        model: Option<&str>,
    ) -> hone_core::HoneResult<ChatResponse>;

    /// Tool-capable streaming. Providers without native support retain the
    /// existing behavior through a single-event fallback.
    fn chat_with_tools_stream<'a>(
        &'a self,
        messages: &'a [Message],
        tools: &'a [Value],
        model: Option<&'a str>,
        _tool_choice_mode: ToolChoiceMode,
    ) -> BoxStream<'a, hone_core::HoneResult<ChatStreamEvent>> {
        stream::once(async move { self.chat_with_tools(messages, tools, model).await })
            .flat_map(|result| match result {
                Ok(response) => {
                    let mut events = Vec::new();
                    if let Some(reasoning) = response.reasoning_content {
                        events.push(Ok(ChatStreamEvent::ReasoningDelta(reasoning)));
                    }
                    if !response.content.is_empty() {
                        events.push(Ok(ChatStreamEvent::ContentDelta(response.content)));
                    }
                    for (index, tool_call) in response
                        .tool_calls
                        .unwrap_or_default()
                        .into_iter()
                        .enumerate()
                    {
                        events.push(Ok(ChatStreamEvent::ToolCallDelta {
                            index: index as u32,
                            id: Some(tool_call.id),
                            name: Some(tool_call.function.name),
                            arguments: tool_call.function.arguments,
                        }));
                    }
                    if let Some(usage) = response.usage {
                        events.push(Ok(ChatStreamEvent::Usage(usage)));
                    }
                    stream::iter(events).boxed()
                }
                Err(error) => stream::once(async move { Err(error) }).boxed(),
            })
            .boxed()
    }

    /// 流式对话
    fn chat_stream<'a>(
        &'a self,
        messages: &'a [Message],
        model: Option<&'a str>,
    ) -> BoxStream<'a, hone_core::HoneResult<String>>;
}

#[cfg(test)]
mod tests {
    use super::{ChatStreamEvent, chat_stream_events_from_value};

    #[test]
    fn parses_content_reasoning_parallel_tool_deltas_and_usage() {
        let events = chat_stream_events_from_value(&serde_json::json!({
            "choices": [{
                "delta": {
                    "reasoning_content": "internal",
                    "content": "visible",
                    "tool_calls": [{
                        "index": 1,
                        "id": "call_2",
                        "function": { "name": "data_fetch", "arguments": "{\"sym" }
                    }]
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 4,
                "total_tokens": 14
            }
        }));

        assert!(matches!(
            &events[0],
            ChatStreamEvent::Usage(usage) if usage.total_tokens == Some(14)
        ));
        assert_eq!(
            events[1],
            ChatStreamEvent::ReasoningDelta("internal".to_string())
        );
        assert_eq!(
            events[2],
            ChatStreamEvent::ContentDelta("visible".to_string())
        );
        assert_eq!(
            events[3],
            ChatStreamEvent::ToolCallDelta {
                index: 1,
                id: Some("call_2".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{\"sym".to_string(),
            }
        );
    }
}
