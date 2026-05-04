use std::sync::Arc;

use async_trait::async_trait;
use hone_core::agent::{AgentMessage, AgentResponse};
use hone_core::config::HoneCloudConfig;
use serde_json::{Value, json};

use super::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerRequest, AgentRunnerResult, RunnerTimeouts,
};

pub struct HoneCloudRunner {
    config: HoneCloudConfig,
    timeouts: RunnerTimeouts,
}

impl HoneCloudRunner {
    pub fn new(config: HoneCloudConfig, timeouts: RunnerTimeouts) -> Self {
        Self { config, timeouts }
    }
}

#[async_trait]
impl AgentRunner for HoneCloudRunner {
    fn name(&self) -> &'static str {
        "hone_cloud"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        _emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        match call_hone_cloud(&self.config, self.timeouts, &request).await {
            Ok(content) => {
                let context_messages = vec![AgentMessage {
                    role: "assistant".to_string(),
                    content: Some(content.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    metadata: None,
                }];
                AgentRunnerResult {
                    response: AgentResponse {
                        content,
                        tool_calls_made: Vec::new(),
                        iterations: 1,
                        success: true,
                        error: None,
                    },
                    streamed_output: false,
                    terminal_error_emitted: false,
                    session_metadata_updates: Default::default(),
                    context_messages: Some(context_messages),
                }
            }
            Err(error) => AgentRunnerResult {
                response: AgentResponse {
                    content: String::new(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: false,
                    error: Some(error),
                },
                streamed_output: false,
                terminal_error_emitted: false,
                session_metadata_updates: Default::default(),
                context_messages: None,
            },
        }
    }
}

async fn call_hone_cloud(
    config: &HoneCloudConfig,
    timeouts: RunnerTimeouts,
    request: &AgentRunnerRequest,
) -> Result<String, String> {
    let api_key = config.api_key.trim();
    if api_key.is_empty() {
        return Err(
            "Hone Cloud API Key 为空；请联系 bm@hone-claw.com 获取邀请码和 API Key".to_string(),
        );
    }
    let url = resolve_hone_cloud_chat_url(&config.base_url);
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(timeouts.step)
        .timeout(request.timeout.unwrap_or(timeouts.overall))
        .build()
        .map_err(|error| format!("构建 Hone Cloud HTTP 客户端失败: {error}"))?;
    let body = json!({
        "model": if config.model.trim().is_empty() { "hone-cloud" } else { config.model.trim() },
        "stream": false,
        "messages": build_hone_cloud_messages(request),
    });
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("Hone Cloud 请求失败: {error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("读取 Hone Cloud 响应失败: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "Hone Cloud HTTP {}: {}",
            status.as_u16(),
            truncate(&text, 500)
        ));
    }
    let value: Value = serde_json::from_str(&text).map_err(|error| {
        format!(
            "解析 Hone Cloud 响应失败: {error}; body={}",
            truncate(&text, 500)
        )
    })?;
    value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(ToString::to_string)
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "Hone Cloud 响应缺少 choices[0].message.content: {}",
                truncate(&text, 500)
            )
        })
}

pub(crate) fn resolve_hone_cloud_chat_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    let base = if trimmed.is_empty() {
        "https://hone-claw.com"
    } else {
        trimmed
    };
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/api/public/v1/chat/completions")
    }
}

fn build_hone_cloud_messages(request: &AgentRunnerRequest) -> Vec<Value> {
    let mut messages = Vec::new();
    if !request.system_prompt.trim().is_empty() {
        messages.push(json!({
            "role": "system",
            "content": request.system_prompt,
        }));
    }
    for message in &request.context.messages {
        if matches!(message.role.as_str(), "user" | "assistant")
            && let Some(content) = message
                .content
                .as_deref()
                .filter(|value| !value.trim().is_empty())
        {
            messages.push(json!({
                "role": message.role,
                "content": content,
            }));
        }
    }
    messages.push(json!({
        "role": "user",
        "content": request.runtime_input,
    }));
    messages
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut out = value.chars().take(max_chars).collect::<String>();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::resolve_hone_cloud_chat_url;

    #[test]
    fn resolves_hone_cloud_chat_url_without_duplicate_path() {
        assert_eq!(
            resolve_hone_cloud_chat_url("https://hone-claw.com"),
            "https://hone-claw.com/api/public/v1/chat/completions"
        );
        assert_eq!(
            resolve_hone_cloud_chat_url("https://hone-claw.com/api/public/v1"),
            "https://hone-claw.com/api/public/v1/chat/completions"
        );
        assert_eq!(
            resolve_hone_cloud_chat_url("https://hone-claw.com/api/public/v1/chat/completions"),
            "https://hone-claw.com/api/public/v1/chat/completions"
        );
    }
}
