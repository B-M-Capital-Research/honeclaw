use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use hone_core::ActorIdentity;

use crate::types::UserIdQuery;

pub(crate) fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(json!({ "error": message.into() }))).into_response()
}

pub(crate) fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn require_string(value: Option<String>, field: &str) -> Result<String, Response> {
    normalize_optional_string(value)
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, format!("缺少 {field}")))
}

pub(crate) fn normalize_phone_number(value: &str) -> String {
    let mut normalized = String::new();
    for ch in value.trim().chars() {
        if ch.is_ascii_digit() {
            normalized.push(ch);
        } else if ch == '+' && normalized.is_empty() {
            normalized.push(ch);
        }
    }
    normalized
}

pub(crate) fn require_phone_number(value: Option<String>, field: &str) -> Result<String, Response> {
    let raw = require_string(value, field)?;
    let normalized = normalize_phone_number(&raw);
    let digit_count = normalized.chars().filter(|ch| ch.is_ascii_digit()).count();
    if (6..=20).contains(&digit_count) {
        Ok(normalized)
    } else {
        Err(json_error(
            StatusCode::BAD_REQUEST,
            format!("{field}格式不合法"),
        ))
    }
}

pub(crate) fn require_actor(
    channel: Option<String>,
    user_id: Option<String>,
    channel_scope: Option<String>,
) -> Result<ActorIdentity, Response> {
    let channel = require_string(channel, "channel")?;
    let user_id = require_string(user_id, "user_id")?;
    hone_channels::HoneBotCore::create_actor(&channel, &user_id, channel_scope.as_deref())
        .map_err(|error| json_error(StatusCode::BAD_REQUEST, error.to_string()))
}

pub(crate) fn normalized_query_actor(
    params: &UserIdQuery,
) -> Result<Option<ActorIdentity>, Response> {
    let channel = normalize_optional_string(params.channel.clone());
    let user_id = normalize_optional_string(params.user_id.clone());
    let channel_scope = normalize_optional_string(params.channel_scope.clone());

    normalized_actor(channel, user_id, channel_scope)
}

pub(crate) fn normalized_actor(
    channel: Option<String>,
    user_id: Option<String>,
    channel_scope: Option<String>,
) -> Result<Option<ActorIdentity>, Response> {
    match (channel, user_id) {
        (None, None) => Ok(None),
        (Some(channel), Some(user_id)) => {
            hone_channels::HoneBotCore::create_actor(&channel, &user_id, channel_scope.as_deref())
                .map(Some)
                .map_err(|error| json_error(StatusCode::BAD_REQUEST, error.to_string()))
        }
        _ => Err(json_error(
            StatusCode::BAD_REQUEST,
            "channel 和 user_id 需要同时提供",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_phone_number;

    #[test]
    fn phone_number_normalization_keeps_digits_and_leading_plus() {
        assert_eq!(
            normalize_phone_number(" +86 138-0013-8000 "),
            "+8613800138000"
        );
        assert_eq!(normalize_phone_number("(021) 1234 5678"), "02112345678");
    }
}
