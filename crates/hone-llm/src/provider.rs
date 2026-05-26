//! LLM Provider trait 定义

use async_trait::async_trait;
use futures::stream::BoxStream;
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    /// 流式对话
    fn chat_stream<'a>(
        &'a self,
        messages: &'a [Message],
        model: Option<&'a str>,
    ) -> BoxStream<'a, hone_core::HoneResult<String>>;
}
