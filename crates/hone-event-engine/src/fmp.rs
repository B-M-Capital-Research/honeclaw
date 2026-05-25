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
        let http = reqwest::Client::builder()
            .gzip(true)
            .build()
            .expect("reqwest client init");
        Self {
            keys: pool.keys().to_vec(),
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
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

        let url_base = format!("{}{}", self.base_url, path_with_query);
        let mut last_err: Option<anyhow::Error> = None;

        for key in &self.keys {
            let sep = if url_base.contains('?') { '&' } else { '?' };
            let full_url = format!("{url_base}{sep}apikey={key}");
            match self.fetch_once(&full_url).await {
                Ok(v) => return Ok(v),
                Err(e) => last_err = Some(e),
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("FMP: 无可用 Key")))
    }

    async fn fetch_once(&self, url: &str) -> anyhow::Result<Value> {
        let response = self
            .http
            .get(url)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|err| format_fmp_transport_error("请求", &err))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| format_fmp_transport_error("读取响应", &err))?;
        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let prefix = sanitize_fmp_error_detail(&body)
                .chars()
                .take(200)
                .collect::<String>();
            anyhow::anyhow!("FMP JSON 解析失败: {e}; body_prefix={prefix}")
        })?;

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            anyhow::bail!("FMP Key 无效（HTTP {status}）");
        }
        if let Some(err_msg) = data.get("Error Message").and_then(|v| v.as_str()) {
            let lower = err_msg.to_lowercase();
            if lower.contains("invalid api key")
                || lower.contains("api key")
                || lower.contains("limit reach")
                || lower.contains("upgrade")
            {
                anyhow::bail!("FMP Key 被拒绝: {}", sanitize_fmp_error_detail(err_msg));
            }
        }
        Ok(data)
    }
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
            .find_map(|(idx, ch)| (ch == '&' || ch == ')' || ch == ' ').then_some(idx))
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
                (ch.is_whitespace() || matches!(ch, ')' | ',' | '"' | '&')).then_some(idx)
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
}
