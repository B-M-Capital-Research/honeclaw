use reqwest::StatusCode;
use serde_json::Value;

const MAX_UPSTREAM_ERROR_CHARS: usize = 500;

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

fn extract_upstream_error_detail(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return truncate_error_body(trimmed);
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
        .map(str::to_string)
        .unwrap_or_else(|| truncate_error_body(trimmed));
    let code = error.get("code").or_else(|| value.get("code"));
    match code {
        Some(Value::String(code)) if !code.is_empty() => format!("{message} (code: {code})"),
        Some(Value::Number(code)) => format!("{message} (code: {code})"),
        _ => message,
    }
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
}
