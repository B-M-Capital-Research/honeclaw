//! WebSearchTool — 网络搜索工具
//!
//! 通过 Tavily API 进行网络搜索，支持多 Key 自动 fallback：
//! - 依次尝试 `search.api_keys` 中的每个 Key
//! - 若 Key 无效（401/403/exceeded）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

const DEFAULT_TAVILY_SEARCH_ENDPOINT: &str = "https://api.tavily.com/search";
const MAX_TAVILY_ERROR_CHARS: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TavilyErrorKind {
    KeyRejected,
    TemporaryFailure,
}

/// WebSearchTool — 网络搜索（Tavily，多 Key fallback）
pub struct WebSearchTool {
    /// 有效 API Key 列表（过滤空值后）
    keys: Vec<String>,
    max_results: u32,
    endpoint: String,
    http: reqwest::Client,
}

impl WebSearchTool {
    pub fn new(keys: Vec<String>, max_results: u32) -> Self {
        let pool = hone_core::ApiKeyPool::new(keys);
        Self {
            keys: pool.keys().to_vec(),
            max_results,
            endpoint: DEFAULT_TAVILY_SEARCH_ENDPOINT.to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let pool = hone_core::ApiKeyPool::new(config.search.api_keys.iter().cloned());
        Self {
            keys: pool.keys().to_vec(),
            max_results: config.search.max_results,
            endpoint: DEFAULT_TAVILY_SEARCH_ENDPOINT.to_string(),
            http: reqwest::Client::new(),
        }
    }

    fn extract_error_text(value: &Value) -> Option<String> {
        match value {
            Value::String(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            Value::Array(items) => items.iter().find_map(Self::extract_error_text),
            Value::Object(map) => ["error", "message", "detail", "reason"]
                .iter()
                .find_map(|key| map.get(*key).and_then(Self::extract_error_text)),
            _ => None,
        }
    }

    fn response_error_message(data: &Value) -> Option<String> {
        ["detail", "error", "message"]
            .iter()
            .find_map(|key| data.get(*key).and_then(Self::extract_error_text))
            .map(|message| sanitize_tavily_error_detail(&message))
    }

    fn interpret_response(status: reqwest::StatusCode, data: Value) -> Result<Value, String> {
        let provider_error = Self::response_error_message(&data);

        // HTTP 401/403 或 Tavily 显式返回认证错误 → key 无效，触发 fallback
        if status == 401 || status == 403 {
            return Err(
                provider_error.unwrap_or_else(|| format!("Tavily API Key 无效（HTTP {status}）"))
            );
        }

        // Tavily 额度耗尽常见于 HTTP 429/432；也要触发 fallback。
        if status == 429 || status.as_u16() == 432 {
            return Err(provider_error
                .unwrap_or_else(|| format!("Tavily API Key 已达额度限制（HTTP {status}）")));
        }

        if !status.is_success() {
            return Err(
                provider_error.unwrap_or_else(|| format!("Tavily 请求失败（HTTP {status}）"))
            );
        }

        // Tavily 在 HTTP 200 时也可能把错误包在 detail/error/message 字段里。
        if let Some(detail) = provider_error {
            return Err(detail);
        }

        Ok(data)
    }

    /// 用指定 key 执行一次 Tavily 搜索，返回结果或错误
    async fn search_with_key(&self, key: &str, query: &str) -> Result<Value, String> {
        let body = serde_json::json!({
            "api_key": key,
            "query": query,
            "search_depth": "basic",
            "max_results": self.max_results,
            "include_answer": true,
            "include_raw_content": false
        });

        let response = self
            .http
            .post(&self.endpoint)
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Tavily 网络请求失败: {e}"))?;

        let status = response.status();
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| format!("Tavily 响应解析失败: {e}"))?;

        Self::interpret_response(status, response_json)
    }

    fn classify_attempt_error(error: &str) -> TavilyErrorKind {
        let lower = error.to_lowercase();
        if lower.contains("invalid api key")
            || lower.contains("api key")
            || lower.contains("exceeded your plan")
            || lower.contains("quota")
            || lower.contains("rate limit")
            || lower.contains("upgrade your plan")
            || lower.contains("credits")
            || lower.contains("http 401")
            || lower.contains("http 403")
            || lower.contains("http 429")
            || lower.contains("http 432")
        {
            TavilyErrorKind::KeyRejected
        } else {
            TavilyErrorKind::TemporaryFailure
        }
    }

    fn final_user_error_message(
        &self,
        key_rejected_count: usize,
        temporary_failures: usize,
    ) -> String {
        if key_rejected_count > 0 && temporary_failures == 0 {
            format!(
                "Tavily 搜索当前不可用：已尝试 {} 个 API Key，但都因额度或鉴权被拒绝。请更新可用的 Tavily Key 后重试。",
                self.keys.len()
            )
        } else if temporary_failures > 0 && key_rejected_count == 0 {
            "Tavily 搜索当前暂时不可用，请稍后重试。".to_string()
        } else {
            format!(
                "Tavily 搜索当前不可用：已尝试 {} 个 API Key，但未获得可用响应。请稍后重试或检查 Tavily Key 配置。",
                self.keys.len()
            )
        }
    }
}

fn sanitize_tavily_error_detail(text: &str) -> String {
    let mut output = redact_tavily_marker_value(&redact_url_userinfo(text), "Bearer ");
    output = redact_tavily_marker_value(&output, "Basic ");
    for key in SENSITIVE_TAVILY_ERROR_KEYS {
        output = redact_tavily_marker_value(&output, &format!("{key}="));
        output = redact_tavily_marker_value(&output, &format!("{key}:"));
        output = redact_tavily_json_string_field(&output, key);
    }
    for key in ["authorization", "Authorization"] {
        output = redact_tavily_json_string_field(&output, key);
    }
    if output.chars().count() <= MAX_TAVILY_ERROR_CHARS {
        return output;
    }
    output
        .chars()
        .take(MAX_TAVILY_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

const SENSITIVE_TAVILY_ERROR_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "api_key",
    "apiKey",
    "apikey",
    "client_secret",
    "clientSecret",
    "refresh_token",
    "refreshToken",
    "id_token",
    "idToken",
    "session_token",
    "sessionToken",
    "bot_token",
    "botToken",
    "OPENROUTER_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "TAVILY_API_KEY",
    "FMP_API_KEY",
    "HONE_CLOUD_API_KEY",
    "token",
    "secret",
    "password",
    "X-API-Key",
    "x-api-key",
];

fn redact_url_userinfo(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find("://") {
        let authority_start = index + 3;
        let authority = &remaining[authority_start..];
        let authority_end = authority
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch.is_whitespace() || matches!(ch, '/' | '?' | '#' | ')')).then_some(idx)
            })
            .unwrap_or(authority.len());
        let authority_slice = &authority[..authority_end];
        if let Some(at_index) = authority_slice.rfind('@') {
            output.push_str(&remaining[..authority_start]);
            output.push_str("<redacted>@");
            remaining = &remaining[authority_start + at_index + 1..];
        } else {
            output.push_str(&remaining[..authority_start]);
            remaining = &remaining[authority_start..];
        }
    }
    output.push_str(remaining);
    output
}

fn redact_tavily_marker_value(text: &str, marker: &str) -> String {
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

fn redact_tavily_json_string_field(text: &str, key: &str) -> String {
    let key_marker = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&key_marker) {
        let after_key = index + key_marker.len();
        let tail = &remaining[after_key..];
        let Some((colon_offset, _)) = tail.char_indices().find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !tail[colon_offset..].starts_with(':') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let after_colon = &tail[colon_offset + 1..];
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
        let value_start = after_key + colon_offset + 1 + quote_offset + 1;
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

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "搜索互联网获取最新信息。当需要查找实时新闻、股票消息、公司动态或任何需要最新数据的问题时使用。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "query".to_string(),
            param_type: "string".to_string(),
            description: "搜索关键词（英文效果更好），例如 'AAPL latest news' 或 'Bitcoin price prediction 2024'".to_string(),
            required: true,
            r#enum: None,
            items: None,
        }]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");

        if self.keys.is_empty() {
            tracing::warn!(tool = "web_search", "tavily keys are empty");
            return Err(hone_core::HoneError::Tool(
                "Tavily 搜索当前不可用：未配置可用的 Tavily API Key。请更新可用的 Tavily Key 后重试。"
                    .to_string(),
            ));
        }

        let mut key_rejected_count = 0usize;
        let mut temporary_failures = 0usize;

        for (index, key) in self.keys.iter().enumerate() {
            match self.search_with_key(key, query).await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    match Self::classify_attempt_error(&e) {
                        TavilyErrorKind::KeyRejected => key_rejected_count += 1,
                        TavilyErrorKind::TemporaryFailure => temporary_failures += 1,
                    }
                    tracing::warn!(
                        tool = "web_search",
                        key_index = index + 1,
                        key_count = self.keys.len(),
                        "tavily request failed for current api key: {}",
                        e
                    );
                    // 继续尝试下一个 key
                }
            }
        }

        // 所有 key 均失败
        tracing::warn!(
            tool = "web_search",
            key_count = self.keys.len(),
            key_rejected_count,
            temporary_failures,
            "{}",
            self.final_user_error_message(key_rejected_count, temporary_failures)
        );
        Err(hone_core::HoneError::Tool(self.final_user_error_message(
            key_rejected_count,
            temporary_failures,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_text_contains_all, assert_text_contains_none};
    use hone_core::config::HoneConfig;

    fn owned_keys(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|key| (*key).to_string()).collect()
    }

    fn assert_message_hides_raw_tavily_upgrade_copy(message: &str) {
        assert_text_contains_none(message, &["support@tavily.com", "upgrade your plan"]);
    }

    #[test]
    fn from_config_keeps_configured_search_limits() {
        let mut config = HoneConfig::default();
        config.search.api_keys = owned_keys(&["config_key"]);
        config.search.max_results = 10;

        let tool = WebSearchTool::from_config(&config);
        assert_eq!(tool.keys, vec!["config_key"]);
        assert_eq!(tool.max_results, 10);
    }

    #[test]
    fn from_config_filters_empty_api_keys() {
        let mut config = HoneConfig::default();
        config.search.api_keys = owned_keys(&["key1", "key2", ""]);
        config.search.max_results = 5;

        let tool = WebSearchTool::from_config(&config);
        assert_eq!(tool.keys, vec!["key1", "key2"]);
        assert_eq!(tool.max_results, 5);
    }

    #[test]
    fn new_records_empty_key_pool() {
        let tool = WebSearchTool::new(vec![], 5);
        assert!(tool.keys.is_empty());
        assert_eq!(tool.max_results, 5);
    }

    #[test]
    fn classify_quota_error_as_key_rejected() {
        let error = "This request exceeds your plan's set usage limit. Please upgrade your plan or contact support@tavily.com";
        assert_eq!(
            WebSearchTool::classify_attempt_error(error),
            TavilyErrorKind::KeyRejected
        );
    }

    #[test]
    fn classify_http_432_as_key_rejected() {
        assert_eq!(
            WebSearchTool::classify_attempt_error("Tavily API Key 已达额度限制（HTTP 432）"),
            TavilyErrorKind::KeyRejected
        );
    }

    #[test]
    fn response_error_message_reads_nested_detail_error() {
        let payload = serde_json::json!({
            "detail": {
                "error": "This request exceeds your plan's set usage limit. Please upgrade your plan or contact support@tavily.com apiKey: leaked-key TAVILY_API_KEY=env-secret Authorization: Basic basic-secret"
            }
        });
        assert_eq!(
            WebSearchTool::response_error_message(&payload).as_deref(),
            Some(
                "This request exceeds your plan's set usage limit. Please upgrade your plan or contact support@tavily.com apiKey: <redacted> TAVILY_API_KEY=<redacted> Authorization: Basic <redacted>"
            )
        );
    }

    #[test]
    fn response_error_message_redacts_json_secret_fields_in_detail() {
        let payload = serde_json::json!({
            "detail": {
                "error": r#"backend rejected {"api_key":"json-key","token":"tok","client_secret":"json-client","authorization":"Basic json-basic","safe":"kept"}"#
            }
        });

        let message = WebSearchTool::response_error_message(&payload).expect("message");
        assert_text_contains_all(
            &message,
            &[
                "\"api_key\":\"<redacted>\"",
                "\"token\":\"<redacted>\"",
                "\"client_secret\":\"<redacted>\"",
                "\"authorization\":\"<redacted>\"",
                "\"safe\":\"kept\"",
            ],
        );
        assert_text_contains_none(
            &message,
            &["json-key", "\"tok\"", "json-client", "json-basic"],
        );
    }

    #[test]
    fn response_error_message_redacts_url_userinfo_in_detail() {
        let payload = serde_json::json!({
            "detail": {
                "error": "callback failed for https://user:secret@example.test/search"
            }
        });

        let message = WebSearchTool::response_error_message(&payload).expect("message");
        assert_eq!(
            message,
            "callback failed for https://<redacted>@example.test/search"
        );
    }

    #[test]
    fn response_error_message_bounds_provider_detail() {
        let payload = serde_json::json!({
            "detail": {
                "error": format!("{} token=secret", "x".repeat(MAX_TAVILY_ERROR_CHARS + 20))
            }
        });

        let message = WebSearchTool::response_error_message(&payload).expect("message");
        assert!(message.ends_with("..."));
        assert!(message.chars().count() <= MAX_TAVILY_ERROR_CHARS + 3);
        assert_text_contains_none(&message, &["secret"]);
    }

    #[test]
    fn interpret_response_rejects_nested_detail_quota_error() {
        let payload = serde_json::json!({
            "detail": {
                "error": "This request exceeds your plan's set usage limit. Please upgrade your plan or contact support@tavily.com"
            }
        });

        let error = WebSearchTool::interpret_response(
            reqwest::StatusCode::from_u16(432).expect("status"),
            payload,
        )
        .expect_err("quota response should fail");

        assert_text_contains_all(&error, &["exceeds your plan"]);
    }

    #[test]
    fn interpret_response_accepts_success_payload_without_error_fields() {
        let payload = serde_json::json!({
            "results": [{ "title": "Fallback result" }],
            "answer": "ok"
        });

        let result = WebSearchTool::interpret_response(reqwest::StatusCode::OK, payload)
            .expect("success payload should pass");

        assert_eq!(result["results"][0]["title"], "Fallback result");
    }

    #[test]
    fn final_error_message_hides_raw_tavily_text() {
        let tool = WebSearchTool::new(vec!["key1".to_string(), "key2".to_string()], 5);
        let message = tool.final_user_error_message(2, 0);
        assert_text_contains_all(&message, &["已尝试 2 个 API Key"]);
        assert_message_hides_raw_tavily_upgrade_copy(&message);
    }

    #[tokio::test]
    async fn execute_with_empty_keys_returns_sanitized_error() {
        let tool = WebSearchTool::new(vec![], 5);
        let error = tool
            .execute(serde_json::json!({"query": "oil"}))
            .await
            .expect_err("missing keys should be a tool error");
        let message = error.to_string();
        assert_text_contains_all(&message, &["Tavily 搜索当前不可用"]);
        assert_message_hides_raw_tavily_upgrade_copy(&message);
    }

    #[tokio::test]
    async fn execute_with_failed_keys_returns_sanitized_error() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut buf = [0_u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let body = r#"{"detail":{"error":"This request exceeds your plan's set usage limit. Please upgrade your plan or contact support@tavily.com"}}"#;
                    let response = format!(
                        "HTTP/1.1 429 Too Many Requests\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });

        let tool = WebSearchTool {
            keys: vec!["key1".to_string(), "key2".to_string()],
            max_results: 5,
            endpoint: format!("http://{addr}"),
            http: reqwest::Client::new(),
        };

        let error = tool
            .execute(serde_json::json!({"query": "oil"}))
            .await
            .expect_err("failed keys should be a tool error");
        let message = error.to_string();
        assert_text_contains_all(&message, &["Tavily 搜索当前"]);
        assert_message_hides_raw_tavily_upgrade_copy(&message);
    }
}
