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
    /// Wire-level tool-choice behavior for this concrete stream. This is the
    /// first successful event emitted by every provider implementation.
    ToolChoiceMetadata {
        requested: ToolChoiceMode,
        effective: ToolChoiceMode,
        fallback: bool,
    },
    ContentDelta(String),
    ReasoningDelta(String),
    ToolCallDelta {
        index: u32,
        id: Option<String>,
        name: Option<String>,
        arguments: String,
    },
    Usage(TokenUsage),
    /// The provider's typed completion reason. This does not prove that the
    /// transport reached its terminal sentinel; consumers that require a
    /// complete response must also observe [`ChatStreamEvent::Done`].
    Finish(ChatStreamFinishReason),
    /// Explicit provider terminal sentinel (`data: [DONE]`) or an equivalent
    /// adapter-validated boundary. Generic OpenAI-compatible streams may use
    /// exactly one typed finish followed by error-free clean EOF; non-native
    /// fallbacks synthesize the same internal boundary after full completion.
    Done,
}

/// Typed completion reasons used by OpenAI-compatible chat streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatStreamFinishReason {
    Stop,
    ToolCalls,
    Length,
    ContentFilter,
    Error,
    Other(String),
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

pub(crate) fn effective_tool_choice_mode_from_body(
    body: &Map<String, Value>,
    has_tools: bool,
) -> ToolChoiceMode {
    if !has_tools {
        return ToolChoiceMode::Auto;
    }
    match body.get("tool_choice") {
        Some(Value::String(value)) if value == "required" || value == "any" => {
            ToolChoiceMode::Required
        }
        // A named function choice also requires a tool call even though it is
        // more specific than the public Auto/Required abstraction.
        Some(Value::Object(_)) => ToolChoiceMode::Required,
        _ => ToolChoiceMode::Auto,
    }
}

/// Extract only fields that a provider explicitly designates as error
/// details. Once a body parses as JSON, arbitrary request echoes must never be
/// scanned: they can contain `tool_choice=required` even when the actual error
/// is unrelated (for example, an invalid model identifier).
pub(crate) fn explicit_provider_error_text(response_body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(response_body) else {
        return response_body.to_string();
    };

    fn push_scalar(parts: &mut Vec<String>, value: Option<&Value>) {
        let Some(value) = value else {
            return;
        };
        match value {
            Value::String(value) => parts.push(value.clone()),
            Value::Number(value) => parts.push(value.to_string()),
            Value::Bool(value) => parts.push(value.to_string()),
            _ => {}
        }
    }

    let mut parts = Vec::new();
    if let Some(error) = value.get("error") {
        if let Some(error) = error.as_object() {
            for field in ["message", "msg", "detail", "param"] {
                push_scalar(&mut parts, error.get(field));
            }
        } else {
            push_scalar(&mut parts, Some(error));
        }
    }
    for field in ["message", "msg"] {
        push_scalar(&mut parts, value.get(field));
    }
    if let Some(detail) = value.get("detail") {
        if let Some(items) = detail.as_array() {
            for item in items {
                let Some(item) = item.as_object() else {
                    push_scalar(&mut parts, Some(item));
                    continue;
                };
                if let Some(location) = item.get("loc").and_then(Value::as_array) {
                    for segment in location {
                        push_scalar(&mut parts, Some(segment));
                    }
                }
                for field in ["msg", "message", "detail", "param", "input"] {
                    push_scalar(&mut parts, item.get(field));
                }
            }
        } else {
            push_scalar(&mut parts, Some(detail));
        }
    }

    parts.join(" ")
}

fn concise_stream_error_text(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => return None,
    };
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return None;
    }
    const MAX_CHARS: usize = 300;
    if normalized.chars().count() <= MAX_CHARS {
        Some(normalized)
    } else {
        Some(normalized.chars().take(MAX_CHARS).collect::<String>() + "...")
    }
}

fn top_level_stream_error(value: &Value) -> Option<String> {
    let error = value.get("error")?;
    if error.is_null() {
        return None;
    }
    let message = error
        .get("message")
        .or_else(|| error.get("msg"))
        .or_else(|| error.get("detail"))
        .and_then(concise_stream_error_text)
        .or_else(|| concise_stream_error_text(error))
        .unwrap_or_else(|| "provider reported an unknown streaming error".to_string());
    let code = error
        .get("code")
        .or_else(|| value.get("code"))
        .and_then(concise_stream_error_text);
    Some(match code {
        Some(code) => format!("{message} (code: {code})"),
        None => message,
    })
}

fn parse_finish_reason(value: &Value) -> Option<ChatStreamFinishReason> {
    let reason = value.as_str()?;
    Some(match reason {
        "stop" => ChatStreamFinishReason::Stop,
        "tool_calls" => ChatStreamFinishReason::ToolCalls,
        "length" => ChatStreamFinishReason::Length,
        "content_filter" => ChatStreamFinishReason::ContentFilter,
        "error" => ChatStreamFinishReason::Error,
        other => ChatStreamFinishReason::Other(other.to_string()),
    })
}

pub fn chat_stream_events_from_value(value: &Value) -> hone_core::HoneResult<Vec<ChatStreamEvent>> {
    if let Some(error) = top_level_stream_error(value) {
        return Err(hone_core::HoneError::Llm(format!(
            "stream provider error: {error}"
        )));
    }

    let mut events = Vec::new();
    let choice = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first());
    if let Some(delta) = choice.and_then(|choice| choice.get("delta")) {
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
    }
    if let Some(reason) = choice
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(parse_finish_reason)
    {
        events.push(ChatStreamEvent::Finish(reason));
    }
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
    Ok(events)
}

/// Parse one SSE `data:` payload. Empty keep-alive payloads are ignored while
/// `[DONE]` remains explicit so consumers can distinguish a normal terminal
/// boundary from an abnormal EOF.
pub(crate) fn chat_stream_events_from_sse_data(
    data: &str,
) -> Option<hone_core::HoneResult<Vec<ChatStreamEvent>>> {
    let data = data.trim();
    if data.is_empty() {
        return None;
    }
    if data == "[DONE]" {
        return Some(Ok(vec![ChatStreamEvent::Done]));
    }
    Some(
        serde_json::from_str::<Value>(data)
            .map_err(|error| {
                hone_core::HoneError::Llm(format!("invalid streaming response: {error}"))
            })
            .and_then(|value| chat_stream_events_from_value(&value)),
    )
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
        tool_choice_mode: ToolChoiceMode,
    ) -> BoxStream<'a, hone_core::HoneResult<ChatStreamEvent>> {
        stream::once(async move { self.chat_with_tools(messages, tools, model).await })
            .flat_map(move |result| match result {
                Ok(response) => {
                    let effective = ToolChoiceMode::Auto;
                    let mut events = vec![Ok(ChatStreamEvent::ToolChoiceMetadata {
                        requested: tool_choice_mode,
                        effective,
                        fallback: tool_choice_mode != effective,
                    })];
                    let finish_reason = if response
                        .tool_calls
                        .as_ref()
                        .is_some_and(|tool_calls| !tool_calls.is_empty())
                    {
                        ChatStreamFinishReason::ToolCalls
                    } else {
                        ChatStreamFinishReason::Stop
                    };
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
                    events.push(Ok(ChatStreamEvent::Finish(finish_reason)));
                    events.push(Ok(ChatStreamEvent::Done));
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
    use async_trait::async_trait;
    use futures::StreamExt;
    use futures::stream::{self, BoxStream};

    use super::{
        ChatResponse, ChatResult, ChatStreamEvent, ChatStreamFinishReason, FunctionCall,
        LlmProvider, Message, ToolCall, ToolChoiceMode, chat_stream_events_from_sse_data,
        chat_stream_events_from_value,
    };

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
        }))
        .expect("valid stream chunk");

        assert_eq!(
            events[0],
            ChatStreamEvent::ReasoningDelta("internal".to_string())
        );
        assert_eq!(
            events[1],
            ChatStreamEvent::ContentDelta("visible".to_string())
        );
        assert_eq!(
            events[2],
            ChatStreamEvent::ToolCallDelta {
                index: 1,
                id: Some("call_2".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{\"sym".to_string(),
            }
        );
        assert!(matches!(
            &events[3],
            ChatStreamEvent::Usage(usage) if usage.total_tokens == Some(14)
        ));
    }

    #[test]
    fn parses_finish_reasons_without_requiring_a_delta() {
        let cases = [
            ("stop", ChatStreamFinishReason::Stop),
            ("tool_calls", ChatStreamFinishReason::ToolCalls),
            ("length", ChatStreamFinishReason::Length),
            ("content_filter", ChatStreamFinishReason::ContentFilter),
            ("error", ChatStreamFinishReason::Error),
            (
                "provider_specific",
                ChatStreamFinishReason::Other("provider_specific".to_string()),
            ),
        ];

        for (wire_reason, expected) in cases {
            let events = chat_stream_events_from_value(&serde_json::json!({
                "choices": [{ "finish_reason": wire_reason }]
            }))
            .expect("valid finish chunk");
            assert_eq!(events, vec![ChatStreamEvent::Finish(expected)]);
        }
    }

    #[test]
    fn top_level_stream_error_is_an_error_and_does_not_leak_nested_metadata() {
        let long_message = format!("  provider  failed\n{}", "x".repeat(400));
        let error = chat_stream_events_from_value(&serde_json::json!({
            "error": {
                "message": long_message,
                "code": 429,
                "metadata": { "authorization": "secret-token" }
            }
        }))
        .expect_err("top-level provider error must stop the stream")
        .to_string();

        assert!(error.contains("provider failed"), "{error}");
        assert!(error.contains("code: 429"), "{error}");
        assert!(error.contains("..."), "{error}");
        assert!(!error.contains("secret-token"), "{error}");
        assert!(!error.contains('\n'), "{error}");
    }

    #[test]
    fn null_top_level_error_is_not_treated_as_a_provider_failure() {
        let events = chat_stream_events_from_value(&serde_json::json!({
            "error": null,
            "choices": [{ "delta": { "content": "ok" } }]
        }))
        .expect("null error is a successful provider chunk");

        assert_eq!(
            events,
            vec![ChatStreamEvent::ContentDelta("ok".to_string())]
        );
    }

    #[test]
    fn done_sentinel_is_explicit_and_keep_alive_is_ignored() {
        assert_eq!(
            chat_stream_events_from_sse_data(" [DONE] ")
                .expect("terminal payload")
                .expect("valid terminal payload"),
            vec![ChatStreamEvent::Done]
        );
        assert!(chat_stream_events_from_sse_data("  \n ").is_none());
    }

    struct NonNativeProvider;

    #[async_trait]
    impl LlmProvider for NonNativeProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResult> {
            Ok(ChatResult {
                content: String::new(),
                usage: None,
            })
        }

        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[serde_json::Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            Ok(ChatResponse {
                content: String::new(),
                reasoning_content: None,
                tool_calls: Some(vec![ToolCall {
                    id: "call_1".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "lookup".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
                usage: None,
            })
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            stream::empty().boxed()
        }
    }

    #[tokio::test]
    async fn non_native_stream_emits_metadata_finish_and_done() {
        let provider = NonNativeProvider;
        let messages = [];
        let tools = [];
        let events = provider
            .chat_with_tools_stream(&messages, &tools, None, ToolChoiceMode::Required)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<hone_core::HoneResult<Vec<_>>>()
            .expect("fallback stream events");

        assert_eq!(
            events,
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Auto,
                    fallback: true,
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("call_1".to_string()),
                    name: Some("lookup".to_string()),
                    arguments: "{}".to_string(),
                },
                ChatStreamEvent::Finish(ChatStreamFinishReason::ToolCalls),
                ChatStreamEvent::Done,
            ]
        );
    }
}
