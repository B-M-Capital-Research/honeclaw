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

/// OpenRouter Provider（支持多 Key fallback）
pub struct OpenRouterProvider {
    /// 每个 Key 对应一个 Client，按顺序尝试
    clients: Vec<Client<OpenAIConfig>>,
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

        let clients: Vec<Client<OpenAIConfig>> = pool
            .keys()
            .iter()
            .map(|key| {
                let openai_config = OpenAIConfig::new()
                    .with_api_key(key)
                    .with_api_base(OPENROUTER_BASE_URL);
                Client::with_config(openai_config).with_http_client(http_client.clone())
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
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(OPENROUTER_BASE_URL);

        let http_client = Self::build_http_client(120).unwrap_or_else(|_| reqwest::Client::new());
        Self {
            clients: vec![Client::with_config(openai_config).with_http_client(http_client)],
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

            match client.chat().create(request).await {
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

            match client.chat().create(request).await {
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
