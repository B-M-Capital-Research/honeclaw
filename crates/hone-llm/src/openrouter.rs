//! OpenRouter LLM Provider
//!
//! 默认请求使用 async-openai SDK 连接 OpenRouter；profile options、SDK
//! 反序列化失败兜底和原始错误体保留走 raw HTTP 路径。
//! OpenRouter 兼容 OpenAI API 格式，只需调整 base_url。
//!
//! 非 streaming 请求支持多 API Key fallback：若某个 Key 返回错误，自动尝试下一个。
//! Streaming 请求已开始传输后不适合切 key，因此只使用第一个可用 client。

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
use eventsource_stream::Eventsource;
use futures::StreamExt;
use futures::stream::BoxStream;
use serde_json::{Map, Value};

use crate::provider::{
    ChatResponse, ChatResult, ChatStreamEvent, FunctionCall, LlmProvider, LlmRequestOptions,
    Message, ToolCall, ToolChoiceMode, chat_stream_events_from_sse_data,
    effective_tool_choice_mode_from_body, explicit_provider_error_text,
};

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

fn remove_tool_fields_without_tools(body: &mut Map<String, Value>, has_tools: bool) {
    if has_tools {
        return;
    }
    body.remove("tools");
    body.remove("tool_choice");
    body.remove("parallel_tool_calls");
}

/// Whether OpenRouter (or its selected upstream) explicitly rejected the
/// `tool_choice=required` capability. Keep this narrow because changing to
/// Auto weakens the wire-level Agent constraint and must never be triggered by
/// authentication, rate-limit, server, transport, or unrelated input errors.
fn rejects_required_tool_choice(status: reqwest::StatusCode, response_body: &str) -> bool {
    if status.as_u16() != 400 && status.as_u16() != 422 {
        return false;
    }

    let message = explicit_provider_error_text(response_body).to_ascii_lowercase();
    let names_tool_choice = message.contains("tool_choice")
        || message.contains("tool choice")
        || message.contains("tool-choice");
    let names_required = message.contains("required");
    let reports_incompatibility = [
        "not supported",
        "unsupported",
        "does not support",
        "doesn't support",
        "invalid",
        "not allowed",
        "not permitted",
        "unrecognized",
        "unknown",
        "unexpected",
        "must be",
        "should be",
        "only support",
        "valid option",
        "requires --",
    ]
    .iter()
    .any(|marker| message.contains(marker));

    names_tool_choice && names_required && reports_incompatibility
}

fn normalize_base_url(configured: &str, fallback: &str) -> String {
    let configured = configured.trim();
    if configured.is_empty() {
        fallback.trim_end_matches('/').to_string()
    } else {
        configured.trim_end_matches('/').to_string()
    }
}

struct OpenRouterClient {
    sdk: Client<OpenAIConfig>,
    http_client: reqwest::Client,
    api_key: String,
    base_url: String,
}

/// OpenRouter Provider（非 streaming chat/chat_with_tools 支持多 Key fallback）
pub struct OpenRouterProvider {
    /// 每个 Key 对应一个 Client，按顺序尝试
    clients: Vec<OpenRouterClient>,
    pub model: String,
    pub max_tokens: u16,
    request_options: LlmRequestOptions,
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
        Self::from_config_with_model_and_options(
            config,
            &config.llm.openrouter.model,
            config.llm.openrouter.max_tokens as u16,
            LlmRequestOptions::default(),
        )
    }

    /// 从配置创建 Provider，并为本次用途覆盖 completion token 上限。
    pub fn from_config_with_max_tokens(
        config: &hone_core::config::HoneConfig,
        max_tokens: u16,
    ) -> hone_core::HoneResult<Self> {
        Self::from_config_with_model_and_options(
            config,
            &config.llm.openrouter.model,
            max_tokens,
            LlmRequestOptions::default(),
        )
    }

    pub fn from_config_with_model_and_max_tokens(
        config: &hone_core::config::HoneConfig,
        model: &str,
        max_tokens: u16,
    ) -> hone_core::HoneResult<Self> {
        Self::from_config_with_model_and_options(
            config,
            model,
            max_tokens,
            LlmRequestOptions::default(),
        )
    }

    pub fn from_config_with_model_and_options(
        config: &hone_core::config::HoneConfig,
        model: &str,
        max_tokens: u16,
        request_options: LlmRequestOptions,
    ) -> hone_core::HoneResult<Self> {
        let pool = config.llm.openrouter_key_pool();

        if pool.is_empty() {
            return Err(hone_core::HoneError::Config(
                "LLM API key 未配置：请在 config.yaml 的 llm.providers.openrouter.api_key/api_keys 或 legacy llm.openrouter.api_key/api_keys 中填写；运行时不再读取 *_API_KEY 环境变量".to_string(),
            ));
        }

        Self::from_key_pool(
            pool.keys(),
            OPENROUTER_BASE_URL,
            model,
            config.llm.openrouter.timeout,
            max_tokens,
            request_options,
        )
    }

    pub fn from_key_pool(
        keys: &[String],
        base_url: &str,
        model: &str,
        timeout_secs: u64,
        max_tokens: u16,
        request_options: LlmRequestOptions,
    ) -> hone_core::HoneResult<Self> {
        let pool =
            hone_core::api_key_pool::ApiKeyPool::new(keys.iter().map(|key| key.trim().to_string()));

        if pool.is_empty() {
            return Err(hone_core::HoneError::Config(
                "LLM API key 未配置：请在 config.yaml 的 llm.providers.<name>.api_key 或 api_keys 中填写；运行时不再读取 *_API_KEY 环境变量".to_string(),
            ));
        }

        let base_url = normalize_base_url(base_url, OPENROUTER_BASE_URL);
        let http_client = Self::build_http_client(timeout_secs)?;

        let clients: Vec<OpenRouterClient> = pool
            .keys()
            .iter()
            .map(|key| {
                let openai_config = OpenAIConfig::new()
                    .with_api_key(key)
                    .with_api_base(&base_url);
                OpenRouterClient {
                    sdk: Client::with_config(openai_config).with_http_client(http_client.clone()),
                    http_client: http_client.clone(),
                    api_key: key.clone(),
                    base_url: base_url.clone(),
                }
            })
            .collect();

        Ok(Self {
            clients,
            model: model.to_string(),
            max_tokens,
            request_options,
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
            request_options: LlmRequestOptions::default(),
        }
    }

    pub fn with_request_options(mut self, request_options: LlmRequestOptions) -> Self {
        self.request_options = request_options;
        self
    }

    /// 将我们的 Message 转换为 async-openai 的请求消息
    fn convert_messages(
        messages: &[Message],
    ) -> hone_core::HoneResult<Vec<ChatCompletionRequestMessage>> {
        let mut request_messages = Vec::with_capacity(messages.len());
        for message in messages {
            let request_message = match message.role.as_str() {
                "system" => {
                    let content = message.content.as_deref().unwrap_or("");
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(content)
                        .build()
                        .map(ChatCompletionRequestMessage::System)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                "user" => {
                    let content = message.content.as_deref().unwrap_or("");
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(content)
                        .build()
                        .map(ChatCompletionRequestMessage::User)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                "assistant" => {
                    let mut builder = ChatCompletionRequestAssistantMessageArgs::default();
                    if let Some(content) = &message.content {
                        builder.content(content.as_str());
                    }
                    if let Some(tool_calls) = &message.tool_calls {
                        let request_tool_calls: Vec<
                            async_openai::types::ChatCompletionMessageToolCall,
                        > = tool_calls
                            .iter()
                            .map(
                                |tool_call| async_openai::types::ChatCompletionMessageToolCall {
                                    id: tool_call.id.clone(),
                                    r#type: async_openai::types::ChatCompletionToolType::Function,
                                    function: async_openai::types::FunctionCall {
                                        name: tool_call.function.name.clone(),
                                        arguments: tool_call.function.arguments.clone(),
                                    },
                                },
                            )
                            .collect();
                        builder.tool_calls(request_tool_calls);
                    }
                    builder
                        .build()
                        .map(ChatCompletionRequestMessage::Assistant)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?
                }
                "tool" => {
                    let content = message.content.as_deref().unwrap_or("");
                    let tool_call_id = message.tool_call_id.as_deref().unwrap_or("");
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
            request_messages.push(request_message);
        }
        Ok(request_messages)
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

    async fn post_chat_completion_value(
        client: &OpenRouterClient,
        request: &Value,
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

    fn build_profile_request_body(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Option<Vec<ChatCompletionTool>>,
        model: &str,
    ) -> hone_core::HoneResult<Value> {
        let mut body = Map::new();
        body.insert("model".to_string(), Value::String(model.to_string()));
        body.insert(
            "messages".to_string(),
            serde_json::to_value(messages).map_err(|e| hone_core::HoneError::Llm(e.to_string()))?,
        );
        if let Some(tools) = tools {
            body.insert(
                "tools".to_string(),
                serde_json::to_value(tools)
                    .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?,
            );
        }
        self.request_options
            .apply_to_body(&mut body, self.max_tokens);
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

    fn tool_calls_from_value(value: &Value) -> Option<Vec<ToolCall>> {
        let response_tool_calls = value
            .get("choices")?
            .as_array()?
            .first()?
            .get("message")?
            .get("tool_calls")?
            .as_array()?;
        Some(
            response_tool_calls
                .iter()
                .map(|tool_call_value| {
                    let function_payload = tool_call_value.get("function").unwrap_or(&Value::Null);
                    let arguments = match function_payload.get("arguments") {
                        Some(Value::String(text)) => text.clone(),
                        Some(other) => other.to_string(),
                        None => String::new(),
                    };
                    ToolCall {
                        id: tool_call_value
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        call_type: tool_call_value
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("function")
                            .to_string(),
                        function: FunctionCall {
                            name: function_payload
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
        let mut chat_tools = Vec::with_capacity(tools.len());
        for tool_schema in tools {
            let function_schema = tool_schema
                .get("function")
                .ok_or_else(|| hone_core::HoneError::Llm("工具缺少 function 字段".to_string()))?;

            let name = function_schema
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let description = function_schema
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let parameters = function_schema.get("parameters").cloned();

            let mut function_object = FunctionObject {
                name,
                description,
                parameters: None,
                strict: None,
            };
            if let Some(params) = parameters {
                function_object.parameters = Some(
                    serde_json::from_value(params)
                        .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?,
                );
            }

            chat_tools.push(ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: function_object,
            });
        }
        Ok(chat_tools)
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
            let converted_messages = Self::convert_messages(messages)?;
            if !self.request_options.is_empty() {
                let request =
                    self.build_profile_request_body(converted_messages, None, model_str)?;
                match Self::post_chat_completion_value(client, &request).await {
                    Ok(value) => {
                        return Ok(ChatResult {
                            content: Self::content_from_value(&value),
                            usage: Self::usage_from_value(&value),
                        });
                    }
                    Err(e) => {
                        last_err = e.to_string();
                        continue;
                    }
                }
            }
            let request = CreateChatCompletionRequestArgs::default()
                .model(model_str)
                .messages(converted_messages)
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
        let chat_tools = Self::convert_tools(tools)?;
        let mut last_err = String::new();

        for client in &self.clients {
            let converted_messages = Self::convert_messages(messages)?;
            if !self.request_options.is_empty() {
                let request = self.build_profile_request_body(
                    converted_messages,
                    Some(chat_tools.clone()),
                    model_str,
                )?;
                match Self::post_chat_completion_value(client, &request).await {
                    Ok(value) => {
                        return Ok(ChatResponse {
                            content: Self::content_from_value(&value),
                            reasoning_content: None,
                            tool_calls: Self::tool_calls_from_value(&value),
                            usage: Self::usage_from_value(&value),
                        });
                    }
                    Err(e) => {
                        last_err = e.to_string();
                        continue;
                    }
                }
            }
            let request = CreateChatCompletionRequestArgs::default()
                .model(model_str)
                .messages(converted_messages)
                .tools(chat_tools.clone())
                .max_tokens(self.max_tokens)
                .build()
                .map_err(|e| hone_core::HoneError::Llm(e.to_string()))?;

            match client.sdk.chat().create(request.clone()).await {
                Ok(response) => {
                    let choice = response.choices.first().ok_or_else(|| {
                        hone_core::HoneError::Llm("LLM 返回空 choices".to_string())
                    })?;

                    let content = choice.message.content.clone().unwrap_or_default();

                    let tool_calls = choice.message.tool_calls.as_ref().map(|sdk_tool_calls| {
                        sdk_tool_calls
                            .iter()
                            .map(|sdk_tool_call| ToolCall {
                                id: sdk_tool_call.id.clone(),
                                call_type: "function".to_string(),
                                function: FunctionCall {
                                    name: sdk_tool_call.function.name.clone(),
                                    arguments: sdk_tool_call.function.arguments.clone(),
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
                        reasoning_content: None,
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
                                    reasoning_content: None,
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

    fn chat_with_tools_stream<'a>(
        &'a self,
        messages: &'a [Message],
        tools: &'a [Value],
        model: Option<&'a str>,
        tool_choice_mode: ToolChoiceMode,
    ) -> BoxStream<'a, hone_core::HoneResult<ChatStreamEvent>> {
        let fut = async move {
            if self.clients.is_empty() {
                return Err(hone_core::HoneError::Config(
                    "未配置 OpenRouter API Key".to_string(),
                ));
            }
            let mut body = Map::new();
            body.insert(
                "model".to_string(),
                Value::String(model.unwrap_or(&self.model).to_string()),
            );
            body.insert(
                "messages".to_string(),
                serde_json::to_value(messages)
                    .map_err(|error| hone_core::HoneError::Llm(error.to_string()))?,
            );
            if !tools.is_empty() {
                body.insert("tools".to_string(), Value::Array(tools.to_vec()));
            }
            self.request_options
                .apply_to_body(&mut body, self.max_tokens);
            remove_tool_fields_without_tools(&mut body, !tools.is_empty());
            body.insert("stream".to_string(), Value::Bool(true));
            let use_required_tool_choice =
                !tools.is_empty() && tool_choice_mode == ToolChoiceMode::Required;
            let auto_body = use_required_tool_choice.then(|| {
                let mut auto_body = body.clone();
                // Remove both scalar and named profile choices so the retry is
                // genuinely Auto and its lifecycle metadata remains truthful.
                auto_body.remove("tool_choice");
                auto_body
            });
            if use_required_tool_choice {
                body.insert(
                    "tool_choice".to_string(),
                    Value::String("required".to_string()),
                );
            }
            let initial_effective_tool_choice =
                effective_tool_choice_mode_from_body(&body, !tools.is_empty());

            let mut last_error = String::new();
            let mut successful_response = None;
            for client in &self.clients {
                let response = match client
                    .http_client
                    .post(format!("{}/chat/completions", client.base_url))
                    .bearer_auth(&client.api_key)
                    .json(&Value::Object(body.clone()))
                    .send()
                    .await
                {
                    Ok(response) => response,
                    Err(error) => {
                        last_error = error.to_string();
                        continue;
                    }
                };
                let status = response.status();
                if status.is_success() {
                    successful_response = Some((response, initial_effective_tool_choice, false));
                    break;
                }
                let error_body = response.text().await.unwrap_or_default();
                last_error = format!(
                    "upstream HTTP {}: {}",
                    status.as_u16(),
                    extract_error_message(&error_body)
                );

                let Some(auto_body) = auto_body
                    .as_ref()
                    .filter(|_| rejects_required_tool_choice(status, &error_body))
                else {
                    continue;
                };
                tracing::warn!(
                    "[openrouter] upstream rejected tool_choice=required with HTTP {}; retrying the same client once in Auto mode",
                    status.as_u16()
                );
                let fallback_response = match client
                    .http_client
                    .post(format!("{}/chat/completions", client.base_url))
                    .bearer_auth(&client.api_key)
                    .json(&Value::Object(auto_body.clone()))
                    .send()
                    .await
                {
                    Ok(response) => response,
                    Err(error) => {
                        last_error = error.to_string();
                        continue;
                    }
                };
                let fallback_status = fallback_response.status();
                if fallback_status.is_success() {
                    successful_response = Some((fallback_response, ToolChoiceMode::Auto, true));
                    break;
                }
                let fallback_body = fallback_response.text().await.unwrap_or_default();
                last_error = format!(
                    "upstream HTTP {} after Auto fallback: {}",
                    fallback_status.as_u16(),
                    extract_error_message(&fallback_body)
                );
            }
            let (response, effective_tool_choice, used_fallback) =
                successful_response.ok_or_else(|| {
                    hone_core::HoneError::Llm(format!(
                        "所有 OpenRouter API Key 的流式请求均失败：{last_error}"
                    ))
                })?;
            let provider_stream = response
                .bytes_stream()
                .eventsource()
                .filter_map(|result| async move {
                    match result {
                        Ok(event) => chat_stream_events_from_sse_data(&event.data),
                        Err(error) => Some(Err(hone_core::HoneError::Llm(format!(
                            "stream transport error: {error}"
                        )))),
                    }
                })
                .flat_map(|result| match result {
                    Ok(events) => futures::stream::iter(events.into_iter().map(Ok)).boxed(),
                    Err(error) => futures::stream::once(async move { Err(error) }).boxed(),
                })
                .boxed();
            let metadata = ChatStreamEvent::ToolChoiceMetadata {
                requested: tool_choice_mode,
                effective: effective_tool_choice,
                fallback: used_fallback,
            };
            let stream = futures::stream::once(async move { Ok(metadata) })
                .chain(provider_stream)
                .boxed();
            Ok::<_, hone_core::HoneError>(stream)
        };

        futures::stream::once(fut)
            .flat_map(|result| match result {
                Ok(stream) => stream,
                Err(error) => futures::stream::once(async move { Err(error) }).boxed(),
            })
            .boxed()
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

            let converted_messages = Self::convert_messages(messages)?;
            let request = CreateChatCompletionRequestArgs::default()
                .model(model.unwrap_or(&self.model))
                .messages(converted_messages)
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

    use crate::provider::ChatStreamFinishReason;

    use super::*;

    async fn read_json_request(socket: &mut tokio::net::TcpStream) -> Value {
        let mut request = Vec::new();
        loop {
            let mut chunk = [0_u8; 4096];
            let n = socket.read(&mut chunk).await.expect("read request");
            assert!(n > 0, "connection closed before request body completed");
            request.extend_from_slice(&chunk[..n]);

            let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
            else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().expect("content length"))
                })
                .expect("content-length header");
            let body_start = header_end + 4;
            if request.len() < body_start + content_length {
                continue;
            }
            return serde_json::from_slice(&request[body_start..body_start + content_length])
                .expect("json request body");
        }
    }

    #[test]
    fn required_fallback_only_accepts_explicit_capability_validation_errors() {
        assert!(rejects_required_tool_choice(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"tool_choice=required is unsupported"}}"#,
        ));
        assert!(rejects_required_tool_choice(
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            r#"{"error":{"message":"invalid tool choice: required"}}"#,
        ));

        for status in [
            reqwest::StatusCode::UNAUTHORIZED,
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            assert!(!rejects_required_tool_choice(
                status,
                r#"{"error":{"message":"tool_choice=required is unsupported"}}"#,
            ));
        }
        assert!(!rejects_required_tool_choice(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"model is not available"}}"#,
        ));
        assert!(!rejects_required_tool_choice(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"tool_choice=required"}}"#,
        ));
        assert!(!rejects_required_tool_choice(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":{"message":"invalid model identifier"},"request":{"tool_choice":"required"}}"#,
        ));
        assert!(!rejects_required_tool_choice(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"detail":[{"loc":["body","model"],"msg":"invalid model identifier"}],"request":{"tool_choice":"required"}}"#,
        ));
    }

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
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
                None,
            )
            .await
            .expect_err("numeric provider error should remain an error");
        let error_message = err.to_string();
        assert!(
            error_message.contains("upstream HTTP 400"),
            "{error_message}"
        );
        assert!(
            error_message.contains("maximum context length exceeded"),
            "{error_message}"
        );
        assert!(error_message.contains("code: 400"), "{error_message}");
        assert!(
            !error_message.contains("invalid type: integer"),
            "{error_message}"
        );
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
        let error_message = err.to_string();
        assert!(
            error_message.contains("upstream HTTP 400"),
            "{error_message}"
        );
        assert!(
            error_message.contains("maximum context length exceeded"),
            "{error_message}"
        );
        assert!(error_message.contains("code: 400"), "{error_message}");
        assert!(
            !error_message.contains("invalid type: integer"),
            "{error_message}"
        );
        assert!(
            requests.load(Ordering::SeqCst) >= 2,
            "SDK path plus raw HTTP fallback should both hit the mock server"
        );
    }

    #[tokio::test]
    async fn required_tool_stream_overrides_auto_request_option() {
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
            let payload: Value =
                serde_json::from_str(request.split("\r\n\r\n").nth(1).expect("request body"))
                    .expect("stream request json");
            assert_eq!(payload["stream"], true);
            assert_eq!(payload["tools"][0]["function"]["name"], "finish_research");
            assert_eq!(payload["tool_choice"], "required");

            let body = concat!(
                "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_finish\",\"function\":{\"name\":\"finish_research\",\"arguments\":\"{}\"}}]}}]}\n\n",
                "data: {\"choices\":[{\"finish_reason\":\"tool_calls\"}]}\n\n",
                "data: [DONE]\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        });

        let provider = OpenRouterProvider::new_with_base_url(
            "test-key",
            &format!("http://{addr}"),
            "test-model",
            64,
        )
        .with_request_options(LlmRequestOptions {
            tool_choice: Some(serde_json::json!("auto")),
            ..LlmRequestOptions::default()
        });
        let events = provider
            .chat_with_tools_stream(
                &[Message {
                    role: "user".to_string(),
                    content: Some("finish".to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
                &[serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": "finish_research",
                        "description": "finish",
                        "parameters": { "type": "object", "properties": {} }
                    }
                })],
                None,
                ToolChoiceMode::Required,
            )
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<hone_core::HoneResult<Vec<_>>>()
            .expect("stream events");

        assert_eq!(
            events[0],
            ChatStreamEvent::ToolChoiceMetadata {
                requested: ToolChoiceMode::Required,
                effective: ToolChoiceMode::Required,
                fallback: false,
            }
        );
        assert!(matches!(
            &events[1],
            ChatStreamEvent::ToolCallDelta { name: Some(name), .. } if name == "finish_research"
        ));
        assert_eq!(
            events[2],
            ChatStreamEvent::Finish(ChatStreamFinishReason::ToolCalls)
        );
        assert_eq!(events[3], ChatStreamEvent::Done);
    }

    #[tokio::test]
    async fn required_stream_retries_same_client_once_in_auto_on_capability_rejection() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        let requests = Arc::new(AtomicUsize::new(0));
        let requests_for_task = requests.clone();
        tokio::spawn(async move {
            let (mut required_socket, _) =
                listener.accept().await.expect("accept Required request");
            let required_payload = read_json_request(&mut required_socket).await;
            requests_for_task.fetch_add(1, Ordering::SeqCst);
            assert_eq!(required_payload["stream"], true);
            assert_eq!(required_payload["tool_choice"], "required");
            assert_eq!(
                required_payload["tools"][0]["function"]["name"],
                "demo_tool"
            );
            let rejection =
                r#"{"error":{"message":"tool_choice=required is unsupported by this model"}}"#;
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                rejection.len(),
                rejection
            );
            required_socket
                .write_all(response.as_bytes())
                .await
                .expect("write Required rejection");
            required_socket
                .shutdown()
                .await
                .expect("close first socket");

            let (mut auto_socket, _) = listener.accept().await.expect("accept Auto retry");
            let auto_payload = read_json_request(&mut auto_socket).await;
            requests_for_task.fetch_add(1, Ordering::SeqCst);
            assert_eq!(auto_payload["stream"], true);
            assert!(
                auto_payload.get("tool_choice").is_none(),
                "Auto retry must remove every profile tool choice: {auto_payload}"
            );
            assert_eq!(auto_payload["tools"][0]["function"]["name"], "demo_tool");

            let body = concat!(
                "data: {\"choices\":[{\"delta\":{\"content\":\"fallback-ok\"}}]}\n\n",
                "data: {\"choices\":[{\"finish_reason\":\"stop\"}]}\n\n",
                "data: [DONE]\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            auto_socket
                .write_all(response.as_bytes())
                .await
                .expect("write Auto stream");
            auto_socket.shutdown().await.expect("close second socket");
        });

        let provider = OpenRouterProvider::new_with_base_url(
            "test-key",
            &format!("http://{addr}"),
            "test-model",
            64,
        )
        .with_request_options(LlmRequestOptions {
            tool_choice: Some(serde_json::json!({
                "type": "function",
                "function": { "name": "demo_tool" }
            })),
            ..LlmRequestOptions::default()
        });
        let messages = [Message {
            role: "user".to_string(),
            content: Some("lookup".to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let tools = [serde_json::json!({
            "type": "function",
            "function": {
                "name": "demo_tool",
                "description": "demo",
                "parameters": { "type": "object", "properties": {} }
            }
        })];
        let events = provider
            .chat_with_tools_stream(&messages, &tools, None, ToolChoiceMode::Required)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<hone_core::HoneResult<Vec<_>>>()
            .expect("Auto fallback stream events");

        assert_eq!(
            events,
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Auto,
                    fallback: true,
                },
                ChatStreamEvent::ContentDelta("fallback-ok".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ]
        );
        assert_eq!(requests.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn empty_tool_stream_omits_tool_controls_and_keeps_generation_options() {
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
            let payload: Value =
                serde_json::from_str(request.split("\r\n\r\n").nth(1).expect("request body"))
                    .expect("stream request json");
            let object = payload.as_object().expect("request object");
            assert!(!object.contains_key("tools"), "{payload}");
            assert!(!object.contains_key("tool_choice"), "{payload}");
            assert!(!object.contains_key("parallel_tool_calls"), "{payload}");
            assert_eq!(payload["max_tokens"], 321);
            assert_eq!(payload["reasoning"]["effort"], "low");
            assert_eq!(payload["temperature"], 0.25);
            assert_eq!(payload["stream"], true);

            let body = concat!(
                "data: {\"choices\":[{\"delta\":{\"content\":\"done\"}}]}\n\n",
                "data: {\"choices\":[{\"finish_reason\":\"stop\"}]}\n\n",
                "data: [DONE]\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        });

        let provider = OpenRouterProvider::new_with_base_url(
            "test-key",
            &format!("http://{addr}"),
            "test-model",
            64,
        )
        .with_request_options(LlmRequestOptions {
            max_tokens: Some(321),
            temperature: Some(0.25),
            reasoning: Some(serde_json::json!({ "effort": "low" })),
            tool_choice: Some(serde_json::json!("required")),
            parallel_tool_calls: Some(true),
            ..LlmRequestOptions::default()
        });
        let events = provider
            .chat_with_tools_stream(
                &[Message {
                    role: "user".to_string(),
                    content: Some("write the final answer".to_string()),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
                &[],
                None,
                ToolChoiceMode::Auto,
            )
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<hone_core::HoneResult<Vec<_>>>()
            .expect("stream events");

        assert_eq!(
            events,
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta("done".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ]
        );
    }

    #[tokio::test]
    async fn stream_eof_without_done_remains_detectable() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut buf = [0_u8; 8192];
            let _ = socket.read(&mut buf).await.expect("read request");
            let body = concat!(
                "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n",
                "data: {\"choices\":[{\"finish_reason\":\"stop\"}]}\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write incomplete stream");
            socket.shutdown().await.expect("close stream");
        });

        let provider = OpenRouterProvider::new_with_base_url(
            "test-key",
            &format!("http://{addr}"),
            "test-model",
            64,
        );
        let messages = [Message {
            role: "user".to_string(),
            content: Some("answer".to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let events = provider
            .chat_with_tools_stream(&messages, &[], None, ToolChoiceMode::Auto)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<hone_core::HoneResult<Vec<_>>>()
            .expect("transport closes cleanly");

        assert_eq!(
            events,
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta("partial".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ]
        );
        assert!(!events.contains(&ChatStreamEvent::Done));
    }

    #[tokio::test]
    async fn top_level_sse_error_is_forwarded_as_an_error_event() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut buf = [0_u8; 8192];
            let _ = socket.read(&mut buf).await.expect("read request");
            let body = concat!(
                "data: {\"error\":{\"message\":\"rate limit exceeded\",\"code\":429,\"metadata\":{\"authorization\":\"secret-token\"}}}\n\n",
                "data: [DONE]\n\n"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write error stream");
            socket.shutdown().await.expect("close stream");
        });

        let provider = OpenRouterProvider::new_with_base_url(
            "test-key",
            &format!("http://{addr}"),
            "test-model",
            64,
        );
        let messages = [Message {
            role: "user".to_string(),
            content: Some("answer".to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let events = provider
            .chat_with_tools_stream(&messages, &[], None, ToolChoiceMode::Auto)
            .collect::<Vec<_>>()
            .await;

        assert!(matches!(
            &events[0],
            Ok(ChatStreamEvent::ToolChoiceMetadata {
                requested: ToolChoiceMode::Auto,
                effective: ToolChoiceMode::Auto,
                fallback: false,
            })
        ));
        let error = events[1]
            .as_ref()
            .expect_err("top-level error payload must become Err")
            .to_string();
        assert!(error.contains("rate limit exceeded"), "{error}");
        assert!(error.contains("code: 429"), "{error}");
        assert!(!error.contains("secret-token"), "{error}");
        assert!(matches!(events.last(), Some(Ok(ChatStreamEvent::Done))));
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
