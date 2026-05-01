//! OpenRouter LLM Provider
//!
//! 使用 async-openai 库与 OpenRouter API 通信。
//! OpenRouter 兼容 OpenAI API 格式，只需修改 base_url。
//!
//! 支持多 API Key fallback：若某个 Key 返回错误，自动尝试下一个。

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

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

struct OpenRouterClient {
    sdk: Client<OpenAIConfig>,
    http_client: reqwest::Client,
    api_key: String,
    base_url: String,
}

/// OpenRouter Provider（支持多 Key fallback）
pub struct OpenRouterProvider {
    /// 每个 Key 对应一个 Client，按顺序尝试
    clients: Vec<OpenRouterClient>,
    pub model: String,
    pub max_tokens: u16,
}

impl OpenRouterProvider {
    /// 构建自定义 reqwest 客户端：禁用代理、设置超时
    /// reqwest 默认会读取 http_proxy/https_proxy，若代理不可达会导致 "error sending request"
    fn build_http_client(timeout_secs: u64) -> hone_core::HoneResult<reqwest::Client> {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(timeout)
            .build()
            .map_err(|e| hone_core::HoneError::Config(format!("构建 HTTP 客户端失败: {e}")))
    }

    /// 从配置创建 Provider（支持多 Key）
    pub fn from_config(config: &hone_core::config::HoneConfig) -> hone_core::HoneResult<Self> {
        let pool = config.llm.openrouter.effective_key_pool();

        if pool.is_empty() {
            return Err(hone_core::HoneError::Config(
                "LLM API key 未配置（环境变量或 config.yaml）".to_string(),
            ));
        }

        let http_client = Self::build_http_client(config.llm.openrouter.timeout)?;

        let clients: Vec<OpenRouterClient> = pool
            .keys()
            .iter()
            .map(|key| {
                let openai_config = OpenAIConfig::new()
                    .with_api_key(key)
                    .with_api_base(OPENROUTER_BASE_URL);
                OpenRouterClient {
                    sdk: Client::with_config(openai_config).with_http_client(http_client.clone()),
                    http_client: http_client.clone(),
                    api_key: key.clone(),
                    base_url: OPENROUTER_BASE_URL.to_string(),
                }
            })
            .collect();

        Ok(Self {
            clients,
            model: config.llm.openrouter.model.clone(),
            max_tokens: config.llm.openrouter.max_tokens as u16,
        })
    }

    /// 手动构造（单 Key）
    pub fn new(api_key: &str, model: &str, max_tokens: u16) -> Self {
        Self::new_with_base_url(api_key, OPENROUTER_BASE_URL, model, max_tokens)
    }

    fn new_with_base_url(api_key: &str, base_url: &str, model: &str, max_tokens: u16) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(&base_url);

        let http_client = Self::build_http_client(120).unwrap_or_else(|_| reqwest::Client::new());
        Self {
            clients: vec![OpenRouterClient {
                sdk: Client::with_config(openai_config).with_http_client(http_client.clone()),
                http_client,
                api_key: api_key.to_string(),
                base_url,
            }],
            model: model.to_string(),
            max_tokens,
        }
    }

    /// 将我们的 Message 转换为 async-openai 的请求消息
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
        client: &OpenRouterClient,
        request: &async_openai::types::CreateChatCompletionRequest,
    ) -> hone_core::HoneResult<Value> {
        let response = client
            .http_client
            .post(format!("{}/chat/completions", client.base_url))
            .bearer_auth(&client.api_key)
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

    /// 将 serde_json::Value (OpenAI tool schema) 转换为 ChatCompletionTool
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

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn chat(
        &self,
        messages: &[Message],
        model: Option<&str>,
    ) -> hone_core::HoneResult<ChatResult> {
        let model_str = model.unwrap_or(&self.model);
        let mut last_err = String::new();

        for client in &self.clients {
            let converted = Self::convert_messages(messages)?;
            let request = CreateChatCompletionRequestArgs::default()
                .model(model_str)
                .messages(converted)
                .max_tokens(self.max_tokens)
                .build()
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

            match client.sdk.chat().create(request.clone()).await {
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
                Err(e) => {
                    last_err = e.to_string();
                    if should_retry_with_raw_http(&e) {
                        match Self::post_chat_completion(client, &request).await {
                            Ok(value) => {
                                return Ok(ChatResult {
                                    content: Self::content_from_value(&value),
                                    usage: Self::usage_from_value(&value),
                                });
                            }
                            Err(raw_err) => {
                                last_err = raw_err.to_string();
                            }
                        }
                    }
                    // 继续尝试下一个 client/key
                }
            }
        }

        Err(hone_core::HoneError::Llm(format!(
            "所有 OpenRouter API Key 均失败（共 {} 个）。最后错误：{last_err}",
            self.clients.len()
        )))
    }

    async fn chat_with_tools(
        &self,
        messages: &[Message],
        tools: &[Value],
        model: Option<&str>,
    ) -> hone_core::HoneResult<ChatResponse> {
        let model_str = model.unwrap_or(&self.model);
        let tool_defs = Self::convert_tools(tools)?;
        let mut last_err = String::new();

        for client in &self.clients {
            let converted = Self::convert_messages(messages)?;
            let request = CreateChatCompletionRequestArgs::default()
                .model(model_str)
                .messages(converted)
                .tools(tool_defs.clone())
                .max_tokens(self.max_tokens)
                .build()
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

            match client.sdk.chat().create(request.clone()).await {
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
                Err(e) => {
                    last_err = e.to_string();
                    if should_retry_with_raw_http(&e) {
                        match Self::post_chat_completion(client, &request).await {
                            Ok(value) => {
                                return Ok(ChatResponse {
                                    content: Self::content_from_value(&value),
                                    tool_calls: Self::tool_calls_from_value(&value),
                                    usage: Self::usage_from_value(&value),
                                });
                            }
                            Err(raw_err) => {
                                last_err = raw_err.to_string();
                            }
                        }
                    }
                    // 继续尝试下一个 client/key
                }
            }
        }

        Err(hone_core::HoneError::Llm(format!(
            "所有 OpenRouter API Key 均失败（共 {} 个）。最后错误：{last_err}",
            self.clients.len()
        )))
    }

    fn chat_stream<'a>(
        &'a self,
        messages: &'a [Message],
        model: Option<&'a str>,
    ) -> BoxStream<'a, hone_core::HoneResult<String>> {
        // streaming 场景使用第一个可用 client（不易在流中途切换 key）
        let client = self.clients.first();
        let fut = async move {
            let client = match client {
                Some(c) => c,
                None => {
                    return Err(hone_core::HoneError::Llm(
                        "未配置 OpenRouter API Key".to_string(),
                    ));
                }
            };

            let converted = Self::convert_messages(messages)?;
            let request = CreateChatCompletionRequestArgs::default()
                .model(model.unwrap_or(&self.model))
                .messages(converted)
                .max_tokens(self.max_tokens)
                .build()
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

            let stream = client
                .sdk
                .chat()
                .create_stream(request)
                .await
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

            Ok::<_, hone_core::HoneError>(stream.filter_map(|result| async {
                match result {
                    Ok(response) => {
                        let delta = response
                            .choices
                            .first()
                            .and_then(|c| c.delta.content.clone());
                        delta.map(Ok)
                    }
                    Err(e) => Some(Err(hone_core::HoneError::Llm(e.to_string()))),
                }
            }))
        };

        // 展平 Future<Result<Stream>> 为 Stream
        futures::stream::once(fut)
            .flat_map(|result| match result {
                Ok(stream) => stream.boxed(),
                Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
            })
            .boxed()
    }
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

    #[tokio::test]
    async fn chat_preserves_openrouter_numeric_error_body_after_sdk_deserialize_failure() {
        let (base_url, requests) = spawn_numeric_error_server().await;
        let provider =
            OpenRouterProvider::new_with_base_url("test-key", &base_url, "test-model", 16);
        let err = provider
            .chat(
                &[Message {
                    role: "user".to_string(),
                    content: Some("hello".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
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
    async fn chat_with_tools_preserves_openrouter_numeric_error_body_after_sdk_deserialize_failure()
    {
        let (base_url, requests) = spawn_numeric_error_server().await;
        let provider =
            OpenRouterProvider::new_with_base_url("test-key", &base_url, "test-model", 16);
        let err = provider
            .chat_with_tools(
                &[Message {
                    role: "user".to_string(),
                    content: Some("hello".to_string()),
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
