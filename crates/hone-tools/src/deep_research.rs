//! DeepResearchTool — 深度个股研究工具（管理员专属）
//!
//! 通过内部研究 API 启动对指定公司的深度研究任务。
//! API 端点通过 `DEEP_RESEARCH_API_URL` 环境变量配置，
//! 默认为 `http://127.0.0.1:18200/api/research/start`。

use async_trait::async_trait;
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

const MAX_DEEP_RESEARCH_ERROR_CHARS: usize = 300;

/// DeepResearchTool — 启动深度个股研究任务
pub struct DeepResearchTool {
    /// 研究 API 端点（POST）
    api_url: String,
    /// 可选 Bearer 令牌（从环境变量 DEEP_RESEARCH_API_KEY 读取）
    api_key: String,
    http: reqwest::Client,
}

impl DeepResearchTool {
    pub fn new(api_url: &str, api_key: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// 从环境变量构造，优先读 DEEP_RESEARCH_API_URL；Key 可选
    pub fn from_env() -> Self {
        let api_url = std::env::var("DEEP_RESEARCH_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:18200/api/research/start".to_string());
        let api_key = std::env::var("DEEP_RESEARCH_API_KEY").unwrap_or_default();
        Self::new(&api_url, &api_key)
    }
}

#[async_trait]
impl Tool for DeepResearchTool {
    fn name(&self) -> &str {
        "deep_research"
    }

    fn description(&self) -> &str {
        "【管理员专属】启动对指定公司的深度个股研究任务。系统将异步执行约 1-2 小时的全面研究，完成后可在「个股研究」页面查看报告。调用后返回 task_id，系统每分钟自动汇报进度（最多 15 分钟）。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "company_name".to_string(),
            param_type: "string".to_string(),
            description: "公司名称、英文名或股票代码，例如：\"英伟达\"、\"NVIDIA\"、\"NVDA\"、\"比亚迪\"、\"AAPL\"".to_string(),
            required: true,
            r#enum: None,
            items: None,
        }]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let company_name = args
            .get("company_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if company_name.is_empty() {
            return Ok(serde_json::json!({
                "success": false,
                "error": "company_name 不能为空，请提供公司名、英文名或股票代码"
            }));
        }

        let safe_api_url = sanitize_deep_research_error_detail(&self.api_url);
        tracing::info!(
            "[DeepResearchTool] 启动深度研究 company={} api_url={}",
            company_name,
            safe_api_url
        );

        let body = serde_json::json!({
            "company_name": company_name
        });

        let mut req = self
            .http
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30));

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = match req.send().await {
            Ok(response) => response,
            Err(e) => {
                let safe_error = sanitize_deep_research_error_detail(&e.to_string());
                tracing::error!("[DeepResearchTool] API 请求失败: {}", safe_error);
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("研究 API 请求失败: {safe_error}。请确认 DEEP_RESEARCH_API_URL 已正确配置（当前: {safe_api_url}）")
                }));
            }
        };

        let status = response.status();
        let response_json: Value = match response.json().await {
            Ok(value) => value,
            Err(e) => {
                let safe_error = sanitize_deep_research_error_detail(&e.to_string());
                tracing::error!("[DeepResearchTool] 响应解析失败: {}", safe_error);
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("研究 API 响应解析失败: {safe_error}")
                }));
            }
        };

        if !status.is_success() {
            let err_msg = deep_research_error_message(&response_json);
            let raw_preview = deep_research_payload_preview(&response_json);
            tracing::error!(
                "[DeepResearchTool] API 返回错误 status={} error={} response_preview={}",
                status,
                err_msg,
                raw_preview
            );
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("研究 API 返回错误 (HTTP {}): {}", status, err_msg)
            }));
        }

        // 提取 task_id（兼容多种字段名）
        let task_id = response_json
            .get("task_id")
            .or_else(|| response_json.get("taskId"))
            .or_else(|| response_json.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        tracing::info!(
            "[DeepResearchTool] 研究任务已启动 company={} task_id={}",
            company_name,
            task_id
        );

        Ok(serde_json::json!({
            "success": true,
            "task_id": task_id,
            "company_name": company_name,
            "message": format!("已成功启动 {} 的深度研究任务，系统将每分钟汇报一次进度，最多监控 15 分钟。完整报告约需 1-2 小时，完成后可在「个股研究」页面查阅。", company_name),
            "raw": response_json
        }))
    }
}

fn sanitize_deep_research_error_detail(text: &str) -> String {
    let redacted = redact_query_secrets(&redact_bearer_secret(&redact_url_userinfo(text)));
    if redacted.chars().count() <= MAX_DEEP_RESEARCH_ERROR_CHARS {
        return redacted;
    }
    redacted
        .chars()
        .take(MAX_DEEP_RESEARCH_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

fn deep_research_error_message(raw: &Value) -> String {
    raw.get("error")
        .or_else(|| raw.get("message"))
        .and_then(|v| v.as_str())
        .map(sanitize_deep_research_error_detail)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "未知错误".to_string())
}

fn deep_research_payload_preview(raw: &Value) -> String {
    let redacted = redact_json_secrets(raw);
    let encoded = serde_json::to_string(&redacted).unwrap_or_else(|_| redacted.to_string());
    sanitize_deep_research_error_detail(&encoded)
}

fn redact_json_secrets(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let lower = key.to_ascii_lowercase();
                    let redacted = matches!(
                        lower.as_str(),
                        "access_token"
                            | "apikey"
                            | "api_keys"
                            | "api_key"
                            | "apikeys"
                            | "authorization"
                            | "password"
                            | "secret"
                            | "token"
                    );
                    let sanitized = if redacted {
                        Value::String("<redacted>".to_string())
                    } else {
                        redact_json_secrets(value)
                    };
                    (key.clone(), sanitized)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_json_secrets).collect()),
        Value::String(text) => Value::String(sanitize_deep_research_error_detail(text)),
        _ => value.clone(),
    }
}

fn redact_bearer_secret(text: &str) -> String {
    let marker = "Bearer ";
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(marker) {
        let value_start = index + marker.len();
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch == '&' || ch == ')' || ch == ',' || ch == '"' || ch.is_whitespace())
                    .then_some(idx)
            })
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_url_userinfo(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;
    while let Some(scheme_index) = remaining.find("://") {
        let after_scheme = scheme_index + 3;
        output.push_str(&remaining[..after_scheme]);
        let tail = &remaining[after_scheme..];
        let auth_end = tail
            .char_indices()
            .find_map(|(idx, ch)| (ch == '/' || ch == '?' || ch == ')' || ch == ' ').then_some(idx))
            .unwrap_or(tail.len());
        let authority = &tail[..auth_end];
        if let Some(at_index) = authority.rfind('@') {
            output.push_str("<redacted>@");
            output.push_str(&authority[at_index + 1..]);
        } else {
            output.push_str(authority);
        }
        remaining = &tail[auth_end..];
    }
    output.push_str(remaining);
    output
}

fn redact_query_secrets(text: &str) -> String {
    let mut output = text.to_string();
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
        output = redact_delimited_secret_value(&output, &format!("{key}="));
        output = redact_delimited_secret_value(&output, &format!("{key}:"));
    }
    output
}

fn redact_delimited_secret_value(text: &str, needle: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(needle) {
        let value_start = index + needle.len();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_name_and_description() {
        let tool = DeepResearchTool::new("http://127.0.0.1:18200/api/research/start", "");
        assert_eq!(tool.name(), "deep_research");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn parameters_have_company_name() {
        let tool = DeepResearchTool::new("http://127.0.0.1:18200/api/research/start", "");
        let params = tool.parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "company_name");
        assert!(params[0].required);
    }

    #[tokio::test]
    async fn execute_empty_company_name_returns_error() {
        let tool = DeepResearchTool::new("http://127.0.0.1:18200/api/research/start", "");
        let result = tool
            .execute(serde_json::json!({"company_name": ""}))
            .await
            .expect("execute should not panic");
        assert_eq!(result["success"].as_bool(), Some(false));
        assert!(
            result["error"]
                .as_str()
                .unwrap_or("")
                .contains("company_name")
        );
    }

    #[tokio::test]
    async fn execute_network_failure_returns_structured_error() {
        // 使用一个必然失败的端口
        let tool = DeepResearchTool::new(
            "http://user:pass@127.0.0.1:19/api/research/start?token=secret&ok=1",
            "",
        );
        let result = tool
            .execute(serde_json::json!({"company_name": "NVIDIA"}))
            .await
            .expect("execute should not panic");
        assert_eq!(result["success"].as_bool(), Some(false));
        let err = result["error"].as_str().unwrap_or_default();
        assert!(!err.is_empty(), "error should have message");
        assert!(!err.contains("secret"));
        assert!(!err.contains("user:pass"));
        assert!(err.contains("token=<redacted>"));
        assert!(err.contains("<redacted>@127.0.0.1"));
    }

    #[test]
    fn deep_research_error_detail_redacts_url_credentials() {
        let detail = sanitize_deep_research_error_detail(
            "request failed for https://user:pass@example.test/path?api_key=abc&ok=1",
        );
        assert_eq!(
            detail,
            "request failed for https://<redacted>@example.test/path?api_key=<redacted>&ok=1"
        );
    }

    #[test]
    fn deep_research_error_detail_redacts_bearer_credentials() {
        let detail =
            sanitize_deep_research_error_detail("request failed with Authorization: Bearer abc123");
        assert_eq!(
            detail,
            "request failed with Authorization: Bearer <redacted>"
        );
    }

    #[test]
    fn deep_research_error_detail_redacts_colon_credentials() {
        let detail = sanitize_deep_research_error_detail(
            "backend rejected request with apiKey: header-secret and token=token-secret",
        );

        assert!(detail.contains("apiKey: <redacted>"));
        assert!(detail.contains("token=<redacted>"));
        assert!(!detail.contains("header-secret"));
        assert!(!detail.contains("token-secret"));
    }

    #[test]
    fn deep_research_payload_preview_redacts_secret_fields() {
        let detail = deep_research_payload_preview(&serde_json::json!({
            "error": "backend rejected token=abc and Bearer xyz",
            "debug": {
                "api_key": "key",
                "token": "tok",
                "safe": "kept"
            }
        }));

        assert!(detail.contains("backend rejected token=<redacted>"));
        assert!(detail.contains("Bearer <redacted>"));
        assert!(detail.contains("\"api_key\":\"<redacted>\""));
        assert!(detail.contains("\"token\":\"<redacted>\""));
        assert!(detail.contains("\"safe\":\"kept\""));
        assert!(!detail.contains("abc"));
        assert!(!detail.contains("xyz"));
        assert!(!detail.contains(":\"key\""));
        assert!(!detail.contains(":\"tok\""));
    }

    #[tokio::test]
    async fn execute_http_error_hides_raw_payload() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("read local addr");
        tokio::spawn(async move {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let mut buf = [0_u8; 4096];
            let _ = socket.read(&mut buf).await;
            let body = r#"{"error":"backend failed token=abc with Bearer xyz","debug":{"api_key":"key","trace_id":"trace-1"}}"#;
            let response = format!(
                "HTTP/1.1 502 Bad Gateway\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        });

        let tool = DeepResearchTool::new(&format!("http://{addr}/api/research/start"), "");
        let result = tool
            .execute(serde_json::json!({"company_name": "NVIDIA"}))
            .await
            .expect("execute should return structured error");

        assert_eq!(result["success"].as_bool(), Some(false));
        assert!(result.get("raw").is_none());
        let error = result["error"].as_str().expect("error message");
        assert!(error.contains("HTTP 502"), "{error}");
        assert!(error.contains("token=<redacted>"), "{error}");
        assert!(error.contains("Bearer <redacted>"), "{error}");
        assert!(!error.contains("abc"), "{error}");
        assert!(!error.contains("xyz"), "{error}");
        assert!(!error.contains("trace-1"), "{error}");
    }
}
