use std::sync::Arc;

use async_trait::async_trait;
use hone_core::agent::{AgentMessage, AgentResponse};
use hone_core::config::HoneCloudConfig;
use serde_json::{Value, json};

use super::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerRequest, AgentRunnerResult, RunnerTimeouts,
};

const HONE_CLOUD_ERROR_DETAIL_CHARS: usize = 500;

pub(crate) struct HoneCloudRunner {
    config: HoneCloudConfig,
    timeouts: RunnerTimeouts,
}

impl HoneCloudRunner {
    pub(crate) fn new(config: HoneCloudConfig, timeouts: RunnerTimeouts) -> Self {
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
        .map_err(|error| format_hone_cloud_transport_error("请求", &error))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format_hone_cloud_transport_error("响应读取", &error))?;
    if !status.is_success() {
        return Err(format!(
            "Hone Cloud HTTP {}: {}",
            status.as_u16(),
            sanitize_hone_cloud_error_detail(&text)
        ));
    }
    let value: Value = serde_json::from_str(&text).map_err(|error| {
        format!(
            "解析 Hone Cloud 响应失败: {error}; body={}",
            sanitize_hone_cloud_error_detail(&text)
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
                sanitize_hone_cloud_error_detail(&text)
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

fn format_hone_cloud_transport_error(operation: &str, error: &reqwest::Error) -> String {
    let detail = sanitize_hone_cloud_error_detail(&error.to_string());
    if detail.is_empty() {
        format!("Hone Cloud {operation}失败")
    } else {
        format!("Hone Cloud {operation}失败: {detail}")
    }
}

fn sanitize_hone_cloud_error_detail(text: &str) -> String {
    let redacted = redact_common_hone_cloud_secrets(text);
    if redacted.chars().count() <= HONE_CLOUD_ERROR_DETAIL_CHARS {
        return redacted;
    }
    redacted
        .chars()
        .take(HONE_CLOUD_ERROR_DETAIL_CHARS)
        .collect::<String>()
        + "..."
}

fn redact_common_hone_cloud_secrets(text: &str) -> String {
    let mut output = redact_marker_value(text, "Bearer ");
    for key in [
        "access_token",
        "accessToken",
        "api_key",
        "apiKey",
        "apikey",
        "token",
        "secret",
        "password",
    ] {
        output = redact_marker_value(&output, &format!("{key}="));
        output = redact_marker_value(&output, &format!("{key}:"));
        output = redact_json_string_field(&output, key);
    }
    output
}

fn redact_marker_value(text: &str, marker: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(marker) {
        let value_start = index + marker.len();
        output.push_str(&remaining[..value_start]);
        let leading_whitespace = remaining[value_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        output.push_str(&remaining[value_start..value_start + leading_whitespace]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start + leading_whitespace..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch == '&'
                    || ch == ')'
                    || ch == ','
                    || ch == '"'
                    || ch == '\''
                    || ch == '}'
                    || ch == ']'
                    || ch.is_whitespace())
                .then_some(idx)
            })
            .unwrap_or(remaining[value_start + leading_whitespace..].len());
        remaining = &remaining[value_start + leading_whitespace + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_json_string_field(text: &str, key: &str) -> String {
    let key_marker = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&key_marker) {
        let after_key = index + key_marker.len();
        let tail = &remaining[after_key..];
        let Some((value_quote_offset, _)) = tail.char_indices().find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !tail[value_quote_offset..].starts_with(':') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let after_colon = &tail[value_quote_offset + 1..];
        let Some((quote_offset, _)) = after_colon
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !after_colon[quote_offset..].starts_with('"') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let value_start = after_key + value_quote_offset + 1 + quote_offset + 1;
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| (ch == '"').then_some(idx))
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

#[cfg(test)]
mod tests {
    use super::{resolve_hone_cloud_chat_url, sanitize_hone_cloud_error_detail};

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

    #[test]
    fn hone_cloud_error_detail_redacts_common_credentials() {
        let detail = sanitize_hone_cloud_error_detail(
            r#"request failed for https://example.test/chat?api_key=abc&ok=1 Authorization: Bearer xyz apiKey: header-secret {"token": "tok","safe":"kept"}"#,
        );

        assert!(detail.contains("api_key=<redacted>"));
        assert!(detail.contains("Bearer <redacted>"));
        assert!(detail.contains("apiKey: <redacted>"));
        assert!(detail.contains("\"token\": \"<redacted>\""));
        assert!(detail.contains("\"safe\":\"kept\""));
        assert!(!detail.contains("abc"));
        assert!(!detail.contains("xyz"));
        assert!(!detail.contains("header-secret"));
        assert!(!detail.contains(":\"tok\""));
    }
}
