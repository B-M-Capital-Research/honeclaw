//! Generic OpenAI-compatible LLM provider.

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionTool, ChatCompletionToolType,
        CreateChatCompletionRequestArgs, FunctionObject,
    },
};
use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;
use serde_json::Value;

use crate::provider::{ChatResponse, ChatResult, FunctionCall, LlmProvider, Message, ToolCall};

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    client: Client<OpenAIConfig>,
    pub model: String,
    pub max_tokens: u16,
}

impl OpenAiCompatibleProvider {
    fn build_http_client(timeout_secs: u64) -> hone_core::HoneResult<reqwest::Client> {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(timeout)
            .build()
            .map_err(|e| hone_core::HoneError::Config(format!("构建 HTTP 客户端失败: {e}")))
    }

    pub fn new(
        api_key: &str,
        base_url: &str,
        model: &str,
        timeout_secs: u64,
        max_tokens: u16,
    ) -> hone_core::HoneResult<Self> {
        let http_client = Self::build_http_client(timeout_secs)?;
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url.trim_end_matches('/'));
        Ok(Self {
            client: Client::with_config(openai_config).with_http_client(http_client),
            model: model.to_string(),
            max_tokens,
        })
    }

    fn convert_messages(
        messages: &[Message],
    ) -> hone_core::HoneResult<Vec<ChatCompletionRequestMessage>> {
        let mut out = Vec::with_capacity(messages.len());
        for msg in messages {
            let m = match msg.role.as_str() {
                "system" => {
                    let content = msg.content.as_deref().unwrap_or("");
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(content)
                        .build()
                        .map(ChatCompletionRequestMessage::System)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                "user" => {
                    let content = msg.content.as_deref().unwrap_or("");
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(content)
                        .build()
                        .map(ChatCompletionRequestMessage::User)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                "assistant" => {
                    let mut builder = ChatCompletionRequestAssistantMessageArgs::default();
                    if let Some(content) = &msg.content {
                        builder.content(content.as_str());
                    }
                    if let Some(tool_calls) = &msg.tool_calls {
                        let tc: Vec<async_openai::types::ChatCompletionMessageToolCall> =
                            tool_calls
                                .iter()
                                .map(|tc| async_openai::types::ChatCompletionMessageToolCall {
                                    id: tc.id.clone(),
                                    r#type: async_openai::types::ChatCompletionToolType::Function,
                                    function: async_openai::types::FunctionCall {
                                        name: tc.function.name.clone(),
                                        arguments: tc.function.arguments.clone(),
                                    },
                                })
                                .collect();
                        builder.tool_calls(tc);
                    }
                    builder
                        .build()
                        .map(ChatCompletionRequestMessage::Assistant)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                "tool" => {
                    let content = msg.content.as_deref().unwrap_or("");
                    let tool_call_id = msg.tool_call_id.as_deref().unwrap_or("");
                    ChatCompletionRequestToolMessageArgs::default()
                        .content(content)
                        .tool_call_id(tool_call_id)
                        .build()
                        .map(ChatCompletionRequestMessage::Tool)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                other => {
                    return Err(hone_core::HoneError::Llm(format!(
                        "未知的消息角色: {other}"
                    )));
                }
            };
            out.push(m);
        }
        Ok(out)
    }

    fn convert_tools(tools: &[Value]) -> hone_core::HoneResult<Vec<ChatCompletionTool>> {
        let mut out = Vec::with_capacity(tools.len());
        for tool_val in tools {
            let func = tool_val
                .get("function")
                .ok_or_else(|| hone_core::HoneError::Llm("工具缺少 function 字段".to_string()))?;
            let name = func
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let description = func
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let parameters = func.get("parameters").cloned();

            let mut fo = FunctionObject {
                name,
                description,
                parameters: None,
                strict: None,
            };
            if let Some(params) = parameters {
                fo.parameters = Some(
                    serde_json::from_value(params)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?,
                );
            }

            out.push(ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: fo,
            });
        }
        Ok(out)
    }
}

/// Returns true for transient HTTP transport errors that are worth retrying once.
/// Covers "error sending request for url", connection resets, and EOF-before-response.
fn is_retryable_transport_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("error sending request")
        || lower.contains("connection reset")
        || lower.contains("connection closed before message completed")
        || lower.contains("operation timed out")
        || lower.contains("tcp connect error")
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn chat(
        &self,
        messages: &[Message],
        model: Option<&str>,
    ) -> hone_core::HoneResult<ChatResult> {
        let converted = Self::convert_messages(messages)?;
        let request = CreateChatCompletionRequestArgs::default()
            .model(model.unwrap_or(&self.model))
            .messages(converted)
            .max_tokens(self.max_tokens)
            .build()
            .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

        let mut last_err: Option<hone_core::HoneError> = None;
        for attempt in 0..=1 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            match self.client.chat().create(request.clone()).await {
                Ok(response) => {
                    let content = response
                        .choices
                        .first()
                        .and_then(|c| c.message.content.clone())
                        .unwrap_or_default();
                    let usage = response.usage.map(|u| crate::provider::TokenUsage {
                        prompt_tokens: Some(u.prompt_tokens),
                        completion_tokens: Some(u.completion_tokens),
                        total_tokens: Some(u.total_tokens),
                    });
                    return Ok(ChatResult { content, usage });
                }
                Err(err) => {
                    let msg = err.to_string();
                    let hone_err = hone_core::HoneError::Llm(msg.clone());
                    if attempt == 0 && is_retryable_transport_error(&msg) {
                        tracing::warn!(
                            "[openai_compatible] chat transport error, retrying: {msg}"
                        );
                        last_err = Some(hone_err);
                    } else {
                        return Err(hone_err);
                    }
                }
            }
        }
        Err(last_err.unwrap())
    }

    async fn chat_with_tools(
        &self,
        messages: &[Message],
        tools: &[Value],
        model: Option<&str>,
    ) -> hone_core::HoneResult<ChatResponse> {
        let converted = Self::convert_messages(messages)?;
        let tool_defs = Self::convert_tools(tools)?;
        let request = CreateChatCompletionRequestArgs::default()
            .model(model.unwrap_or(&self.model))
            .messages(converted)
            .tools(tool_defs)
            .max_tokens(self.max_tokens)
            .build()
            .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

        let mut last_err: Option<hone_core::HoneError> = None;
        for attempt in 0..=1 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            match self.client.chat().create(request.clone()).await {
                Ok(response) => {
                    let choice = response.choices.first().ok_or_else(|| {
                        hone_core::HoneError::Llm("LLM 返回空 choices".to_string())
                    })?;
                    let content = choice.message.content.clone().unwrap_or_default();
                    let tool_calls = choice.message.tool_calls.as_ref().map(|tcs| {
                        tcs.iter()
                            .map(|tc| ToolCall {
                                id: tc.id.clone(),
                                call_type: "function".to_string(),
                                function: FunctionCall {
                                    name: tc.function.name.clone(),
                                    arguments: tc.function.arguments.clone(),
                                },
                            })
                            .collect()
                    });
                    let usage = response.usage.map(|u| crate::provider::TokenUsage {
                        prompt_tokens: Some(u.prompt_tokens),
                        completion_tokens: Some(u.completion_tokens),
                        total_tokens: Some(u.total_tokens),
                    });
                    return Ok(ChatResponse {
                        content,
                        tool_calls,
                        usage,
                    });
                }
                Err(err) => {
                    let msg = err.to_string();
                    let hone_err = hone_core::HoneError::Llm(msg.clone());
                    if attempt == 0 && is_retryable_transport_error(&msg) {
                        tracing::warn!(
                            "[openai_compatible] chat_with_tools transport error, retrying: {msg}"
                        );
                        last_err = Some(hone_err);
                    } else {
                        return Err(hone_err);
                    }
                }
            }
        }
        Err(last_err.unwrap())
    }

    fn chat_stream<'a>(
        &'a self,
        messages: &'a [Message],
        model: Option<&'a str>,
    ) -> BoxStream<'a, hone_core::HoneResult<String>> {
        let fut = async move {
            let converted = Self::convert_messages(messages)?;
            let request = CreateChatCompletionRequestArgs::default()
                .model(model.unwrap_or(&self.model))
                .messages(converted)
                .max_tokens(self.max_tokens)
                .build()
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;
            let stream = self
                .client
                .chat()
                .create_stream(request)
                .await
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;
            Ok::<_, hone_core::HoneError>(stream.filter_map(|result| async {
                match result {
                    Ok(response) => response
                        .choices
                        .first()
                        .and_then(|c| c.delta.content.clone())
                        .map(Ok),
                    Err(e) => Some(Err(hone_core::HoneError::Llm(e.to_string()))),
                }
            }))
        };

        futures::stream::once(fut)
            .flat_map(|result| match result {
                Ok(stream) => stream.boxed(),
                Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
            })
            .boxed()
    }
}
