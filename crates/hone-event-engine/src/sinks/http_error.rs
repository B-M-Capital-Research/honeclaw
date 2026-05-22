use reqwest::StatusCode;
use serde_json::Value;

const MAX_UPSTREAM_ERROR_CHARS: usize = 500;
const MAX_TRANSPORT_ERROR_CHARS: usize = 300;

pub(super) fn format_upstream_http_error(
    service: &str,
    operation: &str,
    status: StatusCode,
    body: &str,
) -> String {
    let detail = extract_upstream_error_detail(body);
    if detail.is_empty() {
        format!("{service} {operation} HTTP {status} (empty response body)")
    } else {
        format!("{service} {operation} HTTP {status}: {detail}")
    }
}

pub(super) fn format_transport_error(
    service: &str,
    operation: &str,
    error: &reqwest::Error,
) -> String {
    let detail = sanitize_transport_error_detail(&error.to_string());
    if detail.is_empty() {
        format!("{service} {operation} transport error")
    } else {
        format!("{service} {operation} transport error: {detail}")
    }
}

pub(super) fn format_provider_api_error(
    service: &str,
    operation: &str,
    code: i64,
    message: &str,
) -> String {
    let detail = sanitize_upstream_error_detail(message.trim());
    if detail.is_empty() {
        format!("{service} {operation} error code={code} (empty error message)")
    } else {
        format!("{service} {operation} error code={code} msg={detail}")
    }
}

fn extract_upstream_error_detail(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return sanitize_upstream_error_detail(trimmed);
    };
    let error = value.get("error").unwrap_or(&value);
    let message = error
        .get("message")
        .or_else(|| error.get("msg"))
        .or_else(|| error.get("detail"))
        .or_else(|| error.get("error_description"))
        .or_else(|| error.get("description"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(redact_sensitive_error_detail)
        .unwrap_or_else(|| sanitize_upstream_error_detail(trimmed));
    let code = error.get("code").or_else(|| value.get("code"));
    match code {
        Some(Value::String(code)) if !code.is_empty() => format!("{message} (code: {code})"),
        Some(Value::Number(code)) => format!("{message} (code: {code})"),
        _ => message,
    }
}

fn sanitize_transport_error_detail(text: &str) -> String {
    truncate_transport_error(&redact_sensitive_error_detail(text))
}

fn sanitize_upstream_error_detail(text: &str) -> String {
    truncate_error_body(&redact_sensitive_error_detail(text))
}

fn redact_sensitive_error_detail(text: &str) -> String {
    let redacted = redact_url_userinfo(text);
    let redacted = redact_telegram_bot_tokens(&redacted);
    let redacted = redact_bearer_tokens(&redacted);
    let redacted = redact_discord_bot_tokens(&redacted);
    let redacted = redact_query_secrets(&redacted);
    redact_json_string_fields(&redacted)
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

fn redact_telegram_bot_tokens(text: &str) -> String {
    const MARKER: &str = "api.telegram.org/bot";
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(MARKER) {
        let (prefix, after_prefix) = remaining.split_at(index + MARKER.len());
        output.push_str(prefix);
        output.push_str("<redacted>");
        let token_tail = after_prefix
            .char_indices()
            .find_map(|(idx, ch)| (ch == '/' || ch == '?' || ch == ')').then_some(idx))
            .unwrap_or(after_prefix.len());
        remaining = &after_prefix[token_tail..];
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
        "app_secret",
        "appSecret",
        "password",
    ] {
        output = redact_query_value(&output, key);
        output = redact_marker_value(&output, &format!("{key}:"));
    }
    output
}

fn redact_bearer_tokens(text: &str) -> String {
    const MARKER: &str = "Bearer ";
    redact_marker_value(text, MARKER)
}

fn redact_discord_bot_tokens(text: &str) -> String {
    const MARKER: &str = "Authorization: Bot ";
    redact_marker_value(text, MARKER)
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

fn redact_json_string_fields(text: &str) -> String {
    let mut output = text.to_string();
    for key in [
        "access_token",
        "accessToken",
        "api_key",
        "apiKey",
        "apikey",
        "token",
        "app_secret",
        "appSecret",
        "password",
    ] {
        output = redact_json_string_field(&output, key);
    }
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

fn truncate_transport_error(text: &str) -> String {
    if text.chars().count() <= MAX_TRANSPORT_ERROR_CHARS {
        return text.to_string();
    }
    text.chars()
        .take(MAX_TRANSPORT_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

fn truncate_error_body(text: &str) -> String {
    if text.chars().count() <= MAX_UPSTREAM_ERROR_CHARS {
        return text.to_string();
    }
    text.chars()
        .take(MAX_UPSTREAM_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_empty_body_as_explicit_empty_response() {
        let detail = format_upstream_http_error("discord", "send", StatusCode::BAD_GATEWAY, "  \n");
        assert_eq!(
            detail,
            "discord send HTTP 502 Bad Gateway (empty response body)"
        );
    }

    #[test]
    fn extracts_nested_message_and_code() {
        let body = r#"{"error":{"message":"invalid payload","code":50035}}"#;
        let detail = format_upstream_http_error("discord", "send", StatusCode::BAD_REQUEST, body);
        assert_eq!(
            detail,
            "discord send HTTP 400 Bad Request: invalid payload (code: 50035)"
        );
    }

    #[test]
    fn extracts_top_level_msg_and_code() {
        let body = r#"{"code":99991663,"msg":"receive_id invalid"}"#;
        let detail = format_upstream_http_error("feishu", "send", StatusCode::BAD_REQUEST, body);
        assert_eq!(
            detail,
            "feishu send HTTP 400 Bad Request: receive_id invalid (code: 99991663)"
        );
    }

    #[test]
    fn truncates_large_unstructured_body() {
        let body = "x".repeat(MAX_UPSTREAM_ERROR_CHARS + 10);
        let detail =
            format_upstream_http_error("telegram", "sendMessage", StatusCode::BAD_REQUEST, &body);
        assert_eq!(
            detail,
            format!(
                "telegram sendMessage HTTP 400 Bad Request: {}...",
                "x".repeat(MAX_UPSTREAM_ERROR_CHARS)
            )
        );
    }

    #[test]
    fn redacts_telegram_bot_token_from_transport_error_detail() {
        let detail = sanitize_transport_error_detail(
            "error sending request for url (https://api.telegram.org/bot123456:SECRET/sendMessage)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://api.telegram.org/bot<redacted>/sendMessage)"
        );
    }

    #[test]
    fn redacts_common_query_secrets_from_transport_error_detail() {
        let detail = sanitize_transport_error_detail(
            "error sending request for url (https://example.test/callback?access_token=abc&appSecret=def&apiKey=ghi&ok=1)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://example.test/callback?access_token=<redacted>&appSecret=<redacted>&apiKey=<redacted>&ok=1)"
        );
    }

    #[test]
    fn redacts_bearer_token_from_transport_error_detail() {
        let detail =
            sanitize_transport_error_detail("upstream rejected Authorization: Bearer secret-token");
        assert_eq!(detail, "upstream rejected Authorization: Bearer <redacted>");
    }

    #[test]
    fn redacts_url_userinfo_from_transport_error_detail() {
        let detail = sanitize_transport_error_detail(
            "error sending request for url (https://user:secret@example.test/a)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://<redacted>@example.test/a)"
        );
    }

    #[test]
    fn redacts_discord_bot_token_from_transport_error_detail() {
        let detail =
            sanitize_transport_error_detail("upstream rejected Authorization: Bot secret-token");
        assert_eq!(detail, "upstream rejected Authorization: Bot <redacted>");
    }

    #[test]
    fn redacts_query_secrets_from_upstream_body_detail() {
        let body = r#"{"error":{"message":"callback failed: https://x.test/a?token=abc&ok=1"}}"#;
        let detail = format_upstream_http_error("discord", "send", StatusCode::BAD_REQUEST, body);
        assert_eq!(
            detail,
            "discord send HTTP 400 Bad Request: callback failed: https://x.test/a?token=<redacted>&ok=1"
        );
    }

    #[test]
    fn redacts_json_secret_fields_from_unstructured_body_detail() {
        let body =
            r#"{"debug":{"app_secret":"feishu-secret","token":"raw-token"},"reason":"denied"}"#;
        let detail = format_upstream_http_error("feishu", "token", StatusCode::BAD_REQUEST, body);
        assert_eq!(
            detail,
            r#"feishu token HTTP 400 Bad Request: {"debug":{"app_secret":"<redacted>","token":"<redacted>"},"reason":"denied"}"#
        );
    }

    #[test]
    fn formats_provider_api_errors_with_redacted_message() {
        let detail = format_provider_api_error(
            "feishu",
            "send",
            99991663,
            r#"bad request app_secret=secret token: raw {"api_key":"json-secret"}"#,
        );
        assert_eq!(
            detail,
            r#"feishu send error code=99991663 msg=bad request app_secret=<redacted> token: <redacted> {"api_key":"<redacted>"}"#
        );
    }

    #[test]
    fn formats_provider_api_errors_with_empty_message() {
        let detail = format_provider_api_error("feishu", "token", 99991663, "  ");
        assert_eq!(
            detail,
            "feishu token error code=99991663 (empty error message)"
        );
    }
}
