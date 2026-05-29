use serde::{Deserialize, Serialize};

use hone_core::{HoneError, HoneResult};
use reqwest::StatusCode;
use serde_json::Value;

const MAX_FACADE_ERROR_CHARS: usize = 500;

#[derive(Debug, Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: T,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ResolveEmailParams<'a> {
    email: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct ResolveMobileParams<'a> {
    mobile: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageParams<'a> {
    receive_id_type: &'a str,
    receive_id: &'a str,
    msg_type: &'a str,
    content: &'a str,
    uuid: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateMessageParams<'a> {
    message_id: &'a str,
    msg_type: &'a str,
    content: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuResolvedUser {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub mobile: String,
    pub open_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuSendResult {
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuFacadeHealth {
    pub connected: bool,
    #[serde(default)]
    pub conn_id: Option<String>,
    #[serde(default)]
    pub service_id: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Clone)]
pub struct FeishuFacadeClient {
    rpc_url: String,
    http: reqwest::Client,
}

impl FeishuFacadeClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            http: reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub async fn health(&self) -> HoneResult<FeishuFacadeHealth> {
        self.call("feishu.health", serde_json::json!({})).await
    }

    pub async fn resolve_email(&self, email: &str) -> HoneResult<FeishuResolvedUser> {
        self.call("feishu.resolve_email", ResolveEmailParams { email })
            .await
    }

    pub async fn resolve_mobile(&self, mobile: &str) -> HoneResult<FeishuResolvedUser> {
        self.call("feishu.resolve_mobile", ResolveMobileParams { mobile })
            .await
    }

    pub async fn send_message(
        &self,
        receive_id: &str,
        msg_type: &str,
        content: &str,
        uuid: Option<&str>,
    ) -> HoneResult<FeishuSendResult> {
        self.call(
            "feishu.send_message",
            SendMessageParams {
                receive_id_type: "open_id",
                receive_id,
                msg_type,
                content,
                uuid,
            },
        )
        .await
    }

    pub async fn update_message(
        &self,
        message_id: &str,
        msg_type: &str,
        content: &str,
    ) -> HoneResult<FeishuSendResult> {
        self.call(
            "feishu.update_message",
            UpdateMessageParams {
                message_id,
                msg_type,
                content,
            },
        )
        .await
    }

    async fn call<TParams, TResult>(
        &self,
        method: &'static str,
        params: TParams,
    ) -> HoneResult<TResult>
    where
        TParams: Serialize,
        TResult: for<'de> Deserialize<'de>,
    {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: chrono::Utc::now().timestamp_millis().unsigned_abs(),
            method,
            params,
        };

        let response = self
            .http
            .post(&self.rpc_url)
            .json(&rpc_request)
            .timeout(std::time::Duration::from_secs(20))
            .send()
            .await
            .map_err(|e| HoneError::Integration(format!("Feishu facade 请求失败: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(HoneError::Integration(format_facade_http_error(
                status, &body,
            )));
        }

        let rpc_response: JsonRpcResponse<TResult> = response
            .json()
            .await
            .map_err(|e| HoneError::Integration(format!("Feishu facade 响应解析失败: {e}")))?;

        if let Some(error) = rpc_response.error {
            return Err(HoneError::Integration(format!(
                "Feishu facade RPC 错误 (code={}): {}",
                error.code,
                sanitize_facade_error_detail(&error.message)
            )));
        }

        rpc_response
            .result
            .ok_or_else(|| HoneError::Integration("Feishu facade 返回空结果".to_string()))
    }
}

fn format_facade_http_error(status: StatusCode, body: &str) -> String {
    let detail = extract_facade_error_detail(body);
    if detail.is_empty() {
        format!("Feishu facade HTTP {status} (empty response body)")
    } else {
        format!("Feishu facade HTTP {status}: {detail}")
    }
}

fn extract_facade_error_detail(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return sanitize_facade_error_detail(trimmed);
    };
    let message = value
        .get("error")
        .unwrap_or(&value)
        .get("message")
        .or_else(|| value.get("message"))
        .or_else(|| value.get("msg"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(sanitize_facade_error_detail)
        .unwrap_or_else(|| sanitize_facade_error_detail(trimmed));
    let code = value
        .get("error")
        .and_then(|error| error.get("code"))
        .or_else(|| value.get("code"));
    match code {
        Some(Value::String(code)) if !code.is_empty() => format!("{message} (code: {code})"),
        Some(Value::Number(code)) => format!("{message} (code: {code})"),
        _ => message,
    }
}

fn sanitize_facade_error_detail(text: &str) -> String {
    truncate_facade_error_body(&redact_common_facade_error_secrets(text))
}

fn redact_common_facade_error_secrets(text: &str) -> String {
    let mut output = redact_facade_marker_value(&redact_facade_url_userinfo(text), "Bearer ");
    output = redact_facade_marker_value(&output, "Basic ");
    for key in SENSITIVE_FACADE_ERROR_KEYS {
        output = redact_facade_marker_value(&output, &format!("{key}="));
        output = redact_facade_marker_value(&output, &format!("{key}:"));
        output = redact_facade_json_string_field(&output, key);
    }
    for key in ["authorization", "Authorization"] {
        output = redact_facade_json_string_field(&output, key);
    }
    output
}

const SENSITIVE_FACADE_ERROR_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "api_key",
    "apiKey",
    "apikey",
    "app_secret",
    "appSecret",
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
    "FEISHU_APP_SECRET",
    "token",
    "secret",
    "password",
    "X-API-Key",
    "x-api-key",
];

fn redact_facade_url_userinfo(text: &str) -> String {
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

fn redact_facade_marker_value(text: &str, marker: &str) -> String {
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
                (ch.is_whitespace() || matches!(ch, ')' | ',' | '"' | '&')).then_some(idx)
            })
            .unwrap_or(remaining[value_start + leading_whitespace..].len());
        remaining = &remaining[value_start + leading_whitespace + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_facade_json_string_field(text: &str, key: &str) -> String {
    let needle = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&needle) {
        let after_key = index + needle.len();
        let Some((value_quote_offset, _)) = remaining[after_key..]
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace() && *ch != ':')
            .filter(|(_, ch)| *ch == '"')
        else {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        };
        let value_start = after_key + value_quote_offset + 1;
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let mut escaped = false;
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| {
                if escaped {
                    escaped = false;
                    return None;
                }
                if ch == '\\' {
                    escaped = true;
                    return None;
                }
                (ch == '"').then_some(idx)
            })
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn truncate_facade_error_body(text: &str) -> String {
    if text.chars().count() <= MAX_FACADE_ERROR_CHARS {
        return text.to_string();
    }
    text.chars()
        .take(MAX_FACADE_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn spawn_response_server(status: &str, body: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let status = status.to_string();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let http_response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(http_response.as_bytes());
            }
        });
        format!("http://{addr}")
    }

    fn spawn_json_server(body: String) -> String {
        spawn_response_server("200 OK", body)
    }

    fn assert_text_contains_none(text: &str, needles: &[&str]) {
        for needle in needles {
            assert!(
                !text.contains(needle),
                "expected text not to contain `{needle}`: {text}"
            );
        }
    }

    #[tokio::test]
    async fn resolve_email_parses_result() {
        let url = spawn_json_server(
            r#"{"result":{"email":"alice@example.com","open_id":"ou_123","user_id":"u_1"},"error":null}"#
                .to_string(),
        );
        let client = FeishuFacadeClient::new(url);
        let result = client
            .resolve_email("alice@example.com")
            .await
            .expect("resolve");
        assert_eq!(result.open_id, "ou_123");
        assert_eq!(result.user_id.as_deref(), Some("u_1"));
    }

    #[tokio::test]
    async fn resolve_mobile_parses_result() {
        let url = spawn_json_server(
            r#"{"result":{"mobile":"+8613800138000","open_id":"ou_456","user_id":"u_2"},"error":null}"#
                .to_string(),
        );
        let client = FeishuFacadeClient::new(url);
        let result = client
            .resolve_mobile("+8613800138000")
            .await
            .expect("resolve");
        assert_eq!(result.open_id, "ou_456");
        assert_eq!(result.mobile, "+8613800138000");
    }

    #[tokio::test]
    async fn rpc_error_is_returned() {
        let url = spawn_json_server(
            r#"{"result":null,"error":{"code":32001,"message":"not found"}}"#.to_string(),
        );
        let client = FeishuFacadeClient::new(url);
        let err = client
            .resolve_email("alice@example.com")
            .await
            .expect_err("rpc error");
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn rpc_error_redacts_secret_message() {
        let url = spawn_json_server(
            r#"{"result":null,"error":{"code":32001,"message":"callback failed https://user:pass@example.test/a?token=query-secret Authorization: Bearer bearer-secret app_secret: app-secret {\"client_secret\":\"json-client\"}"}}"#
                .to_string(),
        );
        let client = FeishuFacadeClient::new(url);
        let err = client
            .resolve_email("alice@example.com")
            .await
            .expect_err("rpc error");
        let message = err.to_string();
        assert!(message.contains("token=<redacted>"));
        assert!(message.contains("Bearer <redacted>"));
        assert!(message.contains("app_secret: <redacted>"));
        assert_text_contains_none(
            &message,
            &[
                "user:pass",
                "query-secret",
                "bearer-secret",
                "app-secret",
                "json-client",
            ],
        );
    }

    #[tokio::test]
    async fn http_error_extracts_message_and_code_without_full_body() {
        let url = spawn_response_server(
            "502 Bad Gateway",
            r#"{"error":{"code":32002,"message":"upstream offline"},"debug":"ignored"}"#
                .to_string(),
        );
        let client = FeishuFacadeClient::new(url);
        let err = client
            .resolve_email("alice@example.com")
            .await
            .expect_err("http error");
        assert_eq!(
            err.to_string(),
            "集成错误: Feishu facade HTTP 502 Bad Gateway: upstream offline (code: 32002)"
        );
    }

    #[tokio::test]
    async fn http_error_redacts_secret_detail() {
        let url = spawn_response_server(
            "502 Bad Gateway",
            r#"{"error":{"code":32002,"message":"upstream failed https://user:pass@example.test/a?api_key=query-secret Authorization: Basic basic-secret"},"debug":{"client_secret":"json-client"}}"#
                .to_string(),
        );
        let client = FeishuFacadeClient::new(url);
        let err = client
            .resolve_email("alice@example.com")
            .await
            .expect_err("http error");
        let message = err.to_string();
        assert!(message.contains("api_key=<redacted>"));
        assert!(message.contains("Basic <redacted>"));
        assert_text_contains_none(
            &message,
            &["user:pass", "query-secret", "basic-secret", "json-client"],
        );
    }
}
