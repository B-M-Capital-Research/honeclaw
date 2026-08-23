//! FMP 最小 HTTP 客户端。
//!
//! 实现 multi-key fallback + 401/403 自动切换下一把 Key（与 `hone-tools/data_fetch.rs`
//! 一致的语义），供 pollers 复用。不做任何参数校验或 endpoint 封装，只负责把
//! path+query 变成带 apikey 的完整 URL，并返回解析后的 JSON。

use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;

use hone_core::config::FmpConfig;

const MAX_FMP_TRANSPORT_ERROR_CHARS: usize = 300;
const FMP_TRANSPORT_RETRY_DELAY_MS: u64 = 200;

enum FmpFetchError {
    KeyRejected(anyhow::Error),
    NonRetryable(anyhow::Error),
    RetryableTransport(anyhow::Error),
}

#[derive(Clone)]
pub struct FmpClient {
    keys: Vec<String>,
    base_url: String,
    timeout: Duration,
    http: reqwest::Client,
}

impl FmpClient {
    pub fn from_config(cfg: &FmpConfig) -> Self {
        let pool = cfg.effective_key_pool();
        // 显式启用 gzip:earning_calendar / stock_dividend_calendar 未压缩响应
        // 体可达数 MB,在 30s timeout 内拉不完(参考 v0.4.x 修复记录)。
        let base_url = cfg.base_url.trim_end_matches('/').to_string();
        let http = fmp_http_client(&base_url);
        Self {
            keys: pool.keys().to_vec(),
            base_url,
            timeout: Duration::from_secs(cfg.timeout),
            http,
        }
    }

    /// 是否有可用的 Key。
    pub fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }

    /// `path_with_query` 形如 `"/v3/earning_calendar?from=2026-04-21&to=2026-05-05"`
    /// （以 `/` 开头）。函数拼接 base_url + apikey 后 GET。
    pub async fn get_json(&self, path_with_query: &str) -> anyhow::Result<Value> {
        if self.keys.is_empty() {
            anyhow::bail!("FMP API Key 未配置");
        }

        let mut last_err: Option<anyhow::Error> = None;
        for key in &self.keys {
            match self.fetch_with_key(path_with_query, key).await {
                Ok(v) => return Ok(v),
                Err(FmpFetchError::KeyRejected(err)) => last_err = Some(err),
                Err(FmpFetchError::NonRetryable(err))
                | Err(FmpFetchError::RetryableTransport(err)) => return Err(err),
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("FMP: 无可用 Key")))
    }

    async fn fetch_with_key(
        &self,
        path_with_query: &str,
        key: &str,
    ) -> Result<Value, FmpFetchError> {
        let url_base = format!("{}{}", self.base_url, path_with_query);
        let sep = if url_base.contains('?') { '&' } else { '?' };
        let full_url = format!("{url_base}{sep}apikey={key}");

        for attempt in 0..=1 {
            match self.fetch_once(&full_url).await {
                Ok(value) => return Ok(value),
                Err(FmpFetchError::RetryableTransport(err)) if attempt == 0 => {
                    tracing::warn!("FMP transport error, retrying once on the same key: {err:#}");
                    tokio::time::sleep(Duration::from_millis(FMP_TRANSPORT_RETRY_DELAY_MS)).await;
                }
                Err(err) => return Err(err),
            }
        }

        Err(FmpFetchError::NonRetryable(anyhow::anyhow!(
            "FMP transport retry unexpectedly exhausted"
        )))
    }

    async fn fetch_once(&self, url: &str) -> Result<Value, FmpFetchError> {
        let response = self
            .http
            .get(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(classify_fmp_transport_error("请求"))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(classify_fmp_transport_error("读取响应"))?;

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(FmpFetchError::KeyRejected(anyhow::anyhow!(
                "FMP Key 无效（HTTP {status}）"
            )));
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(FmpFetchError::KeyRejected(anyhow::anyhow!(
                "FMP Key 配额受限（HTTP {status}）"
            )));
        }
        if !status.is_success() {
            return Err(FmpFetchError::NonRetryable(anyhow::anyhow!(
                "FMP provider error (HTTP {status}): {}",
                sanitize_fmp_error_detail(&body)
            )));
        }

        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let prefix = sanitize_fmp_error_detail(&body)
                .chars()
                .take(200)
                .collect::<String>();
            FmpFetchError::NonRetryable(anyhow::anyhow!(
                "FMP JSON 解析失败: {e}; body_prefix={prefix}"
            ))
        })?;

        if let Some(err_msg) = data.get("Error Message").and_then(|v| v.as_str()) {
            let lower = err_msg.to_lowercase();
            if lower.contains("invalid api key")
                || lower.contains("api key")
                || lower.contains("limit reach")
                || lower.contains("quota")
                || lower.contains("too many requests")
                || lower.contains("upgrade")
            {
                return Err(FmpFetchError::KeyRejected(anyhow::anyhow!(
                    "FMP Key 被拒绝: {}",
                    sanitize_fmp_error_detail(err_msg)
                )));
            }
            return Err(FmpFetchError::NonRetryable(anyhow::anyhow!(
                "FMP provider error: {}",
                sanitize_fmp_error_detail(err_msg)
            )));
        }
        Ok(data)
    }
}

fn fmp_base_url_is_loopback(base_url: &str) -> bool {
    let Some(host) = reqwest::Url::parse(base_url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
    else {
        return false;
    };
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(&host);
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn fmp_http_client(base_url: &str) -> reqwest::Client {
    let builder = reqwest::Client::builder().gzip(true);
    let builder = if fmp_base_url_is_loopback(base_url) {
        builder.no_proxy()
    } else {
        builder
    };
    builder.build().expect("reqwest client init")
}

fn classify_fmp_transport_error(
    operation: &'static str,
) -> impl Fn(reqwest::Error) -> FmpFetchError + Clone {
    move |error| {
        let err = format_fmp_transport_error(operation, &error);
        if is_retryable_fmp_transport_error(&error) {
            FmpFetchError::RetryableTransport(err)
        } else {
            FmpFetchError::NonRetryable(err)
        }
    }
}

fn is_retryable_fmp_transport_error(error: &reqwest::Error) -> bool {
    if error.is_timeout() || error.is_connect() {
        return true;
    }
    let lower = error.to_string().to_lowercase();
    lower.contains("error sending request")
        || lower.contains("error decoding response body")
        || lower.contains("connection reset")
        || lower.contains("connection closed before message completed")
        || lower.contains("operation timed out")
        || lower.contains("tcp connect error")
}

fn format_fmp_transport_error(operation: &str, error: &reqwest::Error) -> anyhow::Error {
    let detail = sanitize_fmp_error_detail(&error.to_string());
    if detail.is_empty() {
        anyhow::anyhow!("FMP {operation}失败")
    } else {
        anyhow::anyhow!("FMP {operation}失败: {detail}")
    }
}

fn sanitize_fmp_error_detail(text: &str) -> String {
    let redacted = redact_fmp_secrets(text);
    if redacted.chars().count() <= MAX_FMP_TRANSPORT_ERROR_CHARS {
        return redacted;
    }
    redacted
        .chars()
        .take(MAX_FMP_TRANSPORT_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

fn redact_fmp_secrets(text: &str) -> String {
    let mut output = redact_url_userinfo(text);
    for key in ["apikey", "api_key", "apiKey"] {
        output = redact_query_value(&output, key);
        output = redact_marker_value(&output, &format!("{key}:"));
        output = redact_json_string_field(&output, key);
    }
    output
}

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

fn redact_query_value(text: &str, key: &str) -> String {
    let needle = format!("{key}=");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&needle) {
        let value_start = index + needle.len();
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch == '&'
                    || ch == ')'
                    || ch == ','
                    || ch == ';'
                    || ch == '"'
                    || ch == '\''
                    || ch == '}'
                    || ch == ']'
                    || ch.is_whitespace())
                .then_some(idx)
            })
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_marker_value(text: &str, marker: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(marker) {
        let marker_end = index + marker.len();
        let after_marker = &remaining[marker_end..];
        let value_offset = after_marker
            .char_indices()
            .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
            .unwrap_or(after_marker.len());
        let value_start = marker_end + value_offset;
        output.push_str(&remaining[..value_start]);
        if value_start == remaining.len() {
            remaining = "";
            break;
        }
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch.is_whitespace() || matches!(ch, ')' | ',' | ';' | '"' | '\'' | '&' | '}' | ']'))
                    .then_some(idx)
            })
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_json_string_field(text: &str, key: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::config::FmpConfig;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn test_client(base_url: &str, keys: Vec<&str>) -> FmpClient {
        let cfg = FmpConfig {
            api_key: String::new(),
            api_keys: keys.into_iter().map(str::to_string).collect(),
            base_url: base_url.to_string(),
            timeout: 2,
        };
        FmpClient::from_config(&cfg)
    }

    #[test]
    fn fmp_transport_error_detail_redacts_apikey_query_param() {
        let detail = sanitize_fmp_error_detail(
            "error sending request for url (https://fmp.test/v3/quote/AAPL?limit=1&apikey=secret)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://fmp.test/v3/quote/AAPL?limit=1&apikey=<redacted>)"
        );
    }

    #[test]
    fn fmp_error_detail_redacts_api_key_aliases() {
        let detail = sanitize_fmp_error_detail(
            "https://fmp.test/v3/quote/AAPL?api_key=one&apiKey=two&apikey=three",
        );
        assert_eq!(
            detail,
            "https://fmp.test/v3/quote/AAPL?api_key=<redacted>&apiKey=<redacted>&apikey=<redacted>"
        );
    }

    #[test]
    fn fmp_error_detail_redacts_api_key_aliases_before_extra_delimiters() {
        let detail = sanitize_fmp_error_detail(
            r#"https://fmp.test/v3/quote/AAPL?api_key=one;apiKey=two" apikey: three]"#,
        );
        assert_eq!(
            detail,
            r#"https://fmp.test/v3/quote/AAPL?api_key=<redacted>;apiKey=<redacted>" apikey: <redacted>]"#
        );
    }

    #[test]
    fn fmp_error_detail_redacts_marker_and_json_key_aliases() {
        let detail = sanitize_fmp_error_detail(
            r#"upstream said api_key: one {"apikey":"two","apiKey":"three"}"#,
        );
        assert_eq!(
            detail,
            r#"upstream said api_key: <redacted> {"apikey":"<redacted>","apiKey":"<redacted>"}"#
        );
    }

    #[test]
    fn fmp_error_detail_redacts_url_userinfo() {
        let detail = sanitize_fmp_error_detail(
            "error sending request for url (https://user:secret@fmp.test/v3/quote/AAPL)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://<redacted>@fmp.test/v3/quote/AAPL)"
        );
    }

    #[test]
    fn loopback_fmp_adapters_bypass_workstation_proxies() {
        assert!(fmp_base_url_is_loopback("http://127.0.0.1:8080/api"));
        assert!(fmp_base_url_is_loopback("http://[::1]:8080/api"));
        assert!(fmp_base_url_is_loopback("http://localhost:8080/api"));
        assert!(!fmp_base_url_is_loopback(
            "https://financialmodelingprep.com/api"
        ));
    }

    #[tokio::test]
    async fn transport_error_retries_same_key_once_before_success() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("server addr");
        let requests = Arc::new(AtomicUsize::new(0));
        let requests_for_task = Arc::clone(&requests);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let request_index = requests_for_task.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buf = vec![0_u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    if request_index == 0 {
                        let body = r#"{"incomplete":true"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                            body.len() + 20,
                            body
                        );
                        let _ = socket.write_all(response.as_bytes()).await;
                        let _ = socket.shutdown().await;
                    } else {
                        let body = r#"{"ok":true}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = socket.write_all(response.as_bytes()).await;
                        let _ = socket.shutdown().await;
                    }
                });
            }
        });

        let client = test_client(&format!("http://{addr}"), vec!["key1", "key2"]);
        let value = client
            .get_json("/v3/stock_news?limit=1")
            .await
            .expect("same-key retry succeeds");

        assert_eq!(value["ok"], true);
        assert_eq!(requests.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn auth_error_still_falls_back_to_next_key() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("server addr");
        let requests = Arc::new(AtomicUsize::new(0));
        let requests_for_task = Arc::clone(&requests);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let request_index = requests_for_task.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buf = vec![0_u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let (status, body) = if request_index == 0 {
                        ("401 Unauthorized", r#"{"Error Message":"invalid api key"}"#)
                    } else {
                        ("200 OK", r#"{"ok":true}"#)
                    };
                    let response = format!(
                        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });

        let client = test_client(&format!("http://{addr}"), vec!["bad", "good"]);
        let value = client
            .get_json("/v3/stock_news?limit=1")
            .await
            .expect("later key should succeed");

        assert_eq!(value["ok"], true);
        assert_eq!(requests.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn provider_errors_do_not_fan_out_to_later_keys() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("server addr");
        let requests = Arc::new(AtomicUsize::new(0));
        let requests_for_task = Arc::clone(&requests);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                requests_for_task.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buf = vec![0_u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let body = r#"{"Error Message":"upstream internal failure"}"#;
                    let response = format!(
                        "HTTP/1.1 500 Internal Server Error\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });

        let client = test_client(&format!("http://{addr}"), vec!["first", "second"]);
        let err = client
            .get_json("/v3/stock_news?limit=1")
            .await
            .expect_err("provider error should stop immediately");

        assert!(err.to_string().contains("HTTP 500"));
        assert_eq!(requests.load(Ordering::SeqCst), 1);
    }
}
