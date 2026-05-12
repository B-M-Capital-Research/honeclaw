//! Generic OpenAI-compatible LLM provider.

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};
use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;
use serde_json::{Map, Value};

use crate::provider::{
    ChatResponse, ChatResult, FunctionCall, LlmProvider, LlmRequestOptions, Message, ToolCall,
};

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    client: Client<OpenAIConfig>,
    http_client: reqwest::Client,
    api_key: String,
    base_url: String,
    pub model: String,
    pub max_tokens: u16,
    request_options: LlmRequestOptions,
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
        let base_url = base_url.trim_end_matches('/').to_string();
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(&base_url);
        Ok(Self {
            client: Client::with_config(openai_config).with_http_client(http_client.clone()),
            http_client,
            api_key: api_key.to_string(),
            base_url,
            model: model.to_string(),
            max_tokens,
            request_options: LlmRequestOptions::default(),
        })
    }

    pub fn with_request_options(mut self, request_options: LlmRequestOptions) -> Self {
        self.request_options = request_options;
        self
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

    async fn post_chat_completion(
        &self,
        request: &async_openai::types::CreateChatCompletionRequest,
    ) -> hone_core::HoneResult<Value> {
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(request)
            .send()
            .await
            .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;
        if !status.is_success() {
            return Err(hone_core::HoneError::Llm(format!(
                "upstream HTTP {}: {}",
                status.as_u16(),
                extract_error_message(&body)
            )));
        }
        serde_json::from_str::<Value>(&body).map_err(|e| {
            hone_core::HoneError::Llm(format!(
                "failed to deserialize api response: {}; body_prefix={}",
                e,
                truncate_error_body(&body, 500)
            ))
        })
    }

    async fn post_chat_completion_value(&self, request: &Value) -> hone_core::HoneResult<Value> {
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(request)
            .send()
            .await
            .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;
        if !status.is_success() {
            return Err(hone_core::HoneError::Llm(format!(
                "upstream HTTP {}: {}",
                status.as_u16(),
                extract_error_message(&body)
            )));
        }
        serde_json::from_str::<Value>(&body).map_err(|e| {
            hone_core::HoneError::Llm(format!(
                "failed to deserialize api response: {}; body_prefix={}",
                e,
                truncate_error_body(&body, 500)
            ))
        })
    }

    fn build_profile_request_body(&self, body: &mut Map<String, Value>) {
        self.request_options.apply_to_body(body, self.max_tokens);
    }

    fn build_request_body(
        &self,
        messages: &[Message],
        tools: Option<&[Value]>,
        model: &str,
    ) -> hone_core::HoneResult<Value> {
        let mut body = Map::new();
        body.insert("model".to_string(), Value::String(model.to_string()));
        body.insert(
            "messages".to_string(),
            serde_json::to_value(messages).map_err(|e| hone_core::HoneError::Llm(e.to_string()))?,
        );
        if let Some(tools) = tools {
            body.insert("tools".to_string(), Value::Array(tools.to_vec()));
        }
        self.build_profile_request_body(&mut body);
        Ok(Value::Object(body))
    }

    fn usage_from_value(value: &Value) -> Option<crate::provider::TokenUsage> {
        let usage = value.get("usage")?;
        Some(crate::provider::TokenUsage {
            prompt_tokens: usage
                .get("prompt_tokens")
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok()),
            completion_tokens: usage
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok()),
            total_tokens: usage
                .get("total_tokens")
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok()),
        })
    }

    fn content_from_value(value: &Value) -> String {
        value
            .get("choices")
            .and_then(|v| v.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or_default()
            .to_string()
    }

    fn reasoning_content_from_value(value: &Value) -> Option<String> {
        value
            .get("choices")
            .and_then(|v| v.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("reasoning_content"))
            .and_then(|content| content.as_str())
            .map(ToString::to_string)
    }

    fn tool_calls_from_value(value: &Value) -> Option<Vec<ToolCall>> {
        let calls = value
            .get("choices")?
            .as_array()?
            .first()?
            .get("message")?
            .get("tool_calls")?
            .as_array()?;
        Some(
            calls
                .iter()
                .map(|tc| {
                    let function = tc.get("function").unwrap_or(&Value::Null);
                    let arguments = match function.get("arguments") {
                        Some(Value::String(s)) => s.clone(),
                        Some(other) => other.to_string(),
                        None => String::new(),
                    };
                    ToolCall {
                        id: tc
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        call_type: tc
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("function")
                            .to_string(),
                        function: FunctionCall {
                            name: function
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            arguments,
                        },
                    }
                })
                .collect(),
        )
    }
}

fn truncate_error_body(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

fn extract_error_message(body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return truncate_error_body(body.trim(), 500);
    };
    let error = value.get("error").unwrap_or(&value);
    let message = error
        .get("message")
        .or_else(|| error.get("msg"))
        .or_else(|| error.get("detail"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| truncate_error_body(body.trim(), 500));
    let code = error.get("code").or_else(|| value.get("code"));
    match code {
        Some(Value::String(code)) if !code.is_empty() => format!("{message} (code: {code})"),
        Some(Value::Number(code)) => format!("{message} (code: {code})"),
        _ => message,
    }
}

fn should_retry_with_raw_http(error: &async_openai::error::OpenAIError) -> bool {
    matches!(
        error,
        async_openai::error::OpenAIError::JSONDeserialize(_)
            | async_openai::error::OpenAIError::ApiError(_)
    )
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    #[test]
    fn extracts_numeric_provider_error_code_without_serde_shape_failure() {
        let body = r#"{"error":{"message":"maximum context length exceeded","code":400}}"#;
        assert_eq!(
            extract_error_message(body),
            "maximum context length exceeded (code: 400)"
        );
    }

    #[test]
    fn extracts_alt_error_message_fields() {
        let body = r#"{"msg":"bad request","code":999}"#;
        assert_eq!(extract_error_message(body), "bad request (code: 999)");
    }

    #[tokio::test]
    async fn chat_with_tools_preserves_numeric_provider_error_body_after_sdk_deserialize_failure() {
        let (base_url, requests) = spawn_numeric_error_server().await;
        let provider = OpenAiCompatibleProvider::new("test-key", &base_url, "test-model", 30, 16)
            .expect("provider");
        let err = provider
            .chat_with_tools(
                &[Message {
                    role: "user".to_string(),
                    content: Some("hello".to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
                &[serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "demo_tool",
                        "description": "demo",
                        "parameters": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                })],
                None,
            )
            .await
            .expect_err("numeric provider error should remain an error");
        let msg = err.to_string();
        assert!(msg.contains("upstream HTTP 400"), "{msg}");
        assert!(msg.contains("maximum context length exceeded"), "{msg}");
        assert!(msg.contains("code: 400"), "{msg}");
        assert!(!msg.contains("invalid type: integer"), "{msg}");
        assert!(
            requests.load(Ordering::SeqCst) >= 2,
            "SDK path plus raw HTTP fallback should both hit the mock server"
        );
    }

    #[tokio::test]
    async fn chat_with_tools_replays_reasoning_content_in_raw_request_body() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let mut buf = vec![0_u8; 65536];
            let n = socket.read(&mut buf).await.expect("read request");
            let request = String::from_utf8_lossy(&buf[..n]);
            let body = request
                .split("\r\n\r\n")
                .nth(1)
                .expect("http body present")
                .trim_matches(char::from(0));
            let payload: Value = serde_json::from_str(body).expect("json payload");
            assert_eq!(
                payload["messages"][1]["reasoning_content"].as_str(),
                Some("need tool lookup first")
            );
            let body = r#"{"id":"resp","choices":[{"index":0,"finish_reason":"stop","message":{"role":"assistant","content":"done","tool_calls":null}}],"usage":{"prompt_tokens":10,"completion_tokens":2,"total_tokens":12}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        });

        let provider = OpenAiCompatibleProvider::new(
            "test-key",
            &format!("http://{addr}"),
            "test-model",
            30,
            64,
        )
        .expect("provider");

        let response = provider
            .chat_with_tools(
                &[
                    Message {
                        role: "user".to_string(),
                        content: Some("find data".to_string()),
                        reasoning_content: None,
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    },
                    Message {
                        role: "assistant".to_string(),
                        content: Some(String::new()),
                        reasoning_content: Some("need tool lookup first".to_string()),
                        tool_calls: Some(vec![ToolCall {
                            id: "call_1".to_string(),
                            call_type: "function".to_string(),
                            function: FunctionCall {
                                name: "demo_tool".to_string(),
                                arguments: r#"{"symbol":"NVDA"}"#.to_string(),
                            },
                        }]),
                        tool_call_id: None,
                        name: None,
                    },
                    Message {
                        role: "tool".to_string(),
                        content: Some(r#"{"ok":true}"#.to_string()),
                        reasoning_content: None,
                        tool_calls: None,
                        tool_call_id: Some("call_1".to_string()),
                        name: Some("demo_tool".to_string()),
                    },
                ],
                &[serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "demo_tool",
                        "description": "demo",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "symbol": { "type": "string" }
                            }
                        }
                    }
                })],
                None,
            )
            .await
            .expect("chat_with_tools");

        assert_eq!(response.content, "done");
    }

    async fn spawn_numeric_error_server() -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        let requests = Arc::new(AtomicUsize::new(0));
        let requests_for_task = requests.clone();
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                requests_for_task.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buf = [0_u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let body =
                        r#"{"error":{"message":"maximum context length exceeded","code":400}}"#;
                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        (format!("http://{addr}"), requests)
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
        let model = model.unwrap_or(&self.model);
        if self.request_options.is_empty()
            && !messages.iter().any(|msg| msg.reasoning_content.is_some())
        {
            let request = CreateChatCompletionRequestArgs::default()
                .model(model)
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
                        if should_retry_with_raw_http(&err) {
                            let value = self.post_chat_completion(&request).await?;
                            return Ok(ChatResult {
                                content: Self::content_from_value(&value),
                                usage: Self::usage_from_value(&value),
                            });
                        }
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
            return Err(last_err.unwrap());
        }

        let request = self.build_request_body(messages, None, model)?;
        let mut last_err: Option<hone_core::HoneError> = None;
        for attempt in 0..=1 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            match self.post_chat_completion_value(&request).await {
                Ok(value) => {
                    return Ok(ChatResult {
                        content: Self::content_from_value(&value),
                        usage: Self::usage_from_value(&value),
                    });
                }
                Err(err) if attempt == 0 && is_retryable_transport_error(&err.to_string()) => {
                    tracing::warn!(
                        "[openai_compatible] raw chat transport error, retrying: {}",
                        err
                    );
                    last_err = Some(err);
                }
                Err(err) => return Err(err),
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
        let model = model.unwrap_or(&self.model);
        let request = self.build_request_body(messages, Some(tools), model)?;
        let mut last_err: Option<hone_core::HoneError> = None;
        for attempt in 0..=1 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            match self.post_chat_completion_value(&request).await {
                Ok(value) => {
                    return Ok(ChatResponse {
                        content: Self::content_from_value(&value),
                        reasoning_content: Self::reasoning_content_from_value(&value),
                        tool_calls: Self::tool_calls_from_value(&value),
                        usage: Self::usage_from_value(&value),
                    });
                }
                Err(err) if attempt == 0 && is_retryable_transport_error(&err.to_string()) => {
                    tracing::warn!(
                        "[openai_compatible] raw chat_with_tools transport error, retrying: {}",
                        err
                    );
                    last_err = Some(err);
                }
                Err(err) => return Err(err),
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
