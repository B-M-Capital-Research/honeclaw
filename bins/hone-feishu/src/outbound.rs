use hone_scheduler::SchedulerEvent;
use serde_json::json;

use crate::client::{FeishuApiClient, FeishuSendResult};
use crate::markdown::{preprocess_markdown_for_feishu, render_outbound_messages};

#[derive(Clone)]
pub(crate) struct SendIdempotency {
    pub(crate) dedup_key: String,
    pub(crate) uuid_seed: String,
}

pub(crate) async fn send_plain_text(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    text: &str,
) -> hone_core::HoneResult<usize> {
    if receive_id_type == "chat_id" {
        facade
            .send_chat_message(
                receive_id,
                "text",
                &json!({ "text": text }).to_string(),
                None,
            )
            .await
            .map_err(hone_core::HoneError::Integration)?;
    } else {
        facade
            .send_message(
                receive_id,
                "text",
                &json!({ "text": text }).to_string(),
                None,
            )
            .await
            .map_err(hone_core::HoneError::Integration)?;
    }
    Ok(1)
}

pub(crate) fn feishu_user_mention(open_id: &str) -> String {
    format!("<at id=\"{open_id}\"></at>")
}

pub(crate) fn prepend_reply_prefix(prefix: Option<&str>, text: &str) -> String {
    let Some(prefix) = prefix.map(str::trim).filter(|value| !value.is_empty()) else {
        return text.to_string();
    };

    let body = text.trim_start();
    if body.starts_with(prefix) {
        text.to_string()
    } else if body.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {body}")
    }
}

pub(crate) async fn send_placeholder_message(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    text: &str,
) -> hone_core::HoneResult<(String, Option<String>)> {
    let request_uuid = uuid::Uuid::new_v4().to_string();
    let card_content = json!({
        "schema": "2.0",
        "config": {"wide_screen_mode": true},
        "body": {
            "elements": [
                {"tag": "markdown", "content": text, "text_size": "heading"}
            ]
        }
    })
    .to_string();
    let result = if receive_id_type == "chat_id" {
        facade
            .send_chat_message(
                receive_id,
                "interactive",
                &card_content,
                Some(&request_uuid),
            )
            .await
    } else {
        facade
            .send_message(
                receive_id,
                "interactive",
                &card_content,
                Some(&request_uuid),
            )
            .await
    }
    .map_err(hone_core::HoneError::Integration)?;
    Ok((result.message_id, None))
}

pub(crate) async fn update_or_send_plain_text(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    placeholder_message_id: Option<&str>,
    text: &str,
) -> hone_core::HoneResult<usize> {
    if let Some(message_id) = placeholder_message_id {
        let processed = preprocess_markdown_for_feishu(text, true);
        let card_content = json!({
            "schema": "2.0",
            "config": {"wide_screen_mode": true},
            "body": {
                "elements": [
                    {
                        "tag": "markdown",
                        "content": processed,
                        "text_size": "heading"
                    }
                ]
            }
        })
        .to_string();
        match facade
            .update_message(message_id, "interactive", &card_content)
            .await
        {
            Ok(_) => {}
            Err(err) if is_feishu_bad_request(&err) => {
                tracing::warn!(
                    "[Feishu/outbound] update plain-text placeholder failed with bad request, fallback to standalone send: message_id={} receive_id_type={} err={}",
                    message_id,
                    receive_id_type,
                    err
                );
                send_segment_direct(
                    facade,
                    receive_id,
                    receive_id_type,
                    "interactive",
                    &card_content,
                    None,
                )
                .await
                .map_err(hone_core::HoneError::Integration)?;
            }
            Err(err) => return Err(hone_core::HoneError::Integration(err)),
        }
        return Ok(1);
    }

    if receive_id_type == "chat_id" {
        facade
            .send_chat_message(
                receive_id,
                "text",
                &json!({ "text": text }).to_string(),
                None,
            )
            .await
            .map_err(hone_core::HoneError::Integration)?;
        Ok(1)
    } else {
        send_plain_text(facade, receive_id, receive_id_type, text).await
    }
}

pub(crate) async fn send_rendered_messages(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    markdown: &str,
    max_message_length: usize,
    placeholder_message_id: Option<&str>,
    uuid_seed: Option<&str>,
) -> hone_core::HoneResult<usize> {
    let messages = render_outbound_messages(markdown, max_message_length);
    if messages.is_empty() {
        return Ok(0);
    }

    let should_thread_followups = receive_id_type == "chat_id" || placeholder_message_id.is_some();
    let total = messages.len();
    let mut previous_message_id = None;
    for (index, message) in messages.into_iter().enumerate() {
        if index == 0 {
            if let Some(message_id) = placeholder_message_id {
                let card_content = if message.msg_type == "interactive" {
                    message.content.clone()
                } else if message.msg_type == "post" {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&message.content)
                    {
                        if let Some(zh_cn) = parsed.get("zh_cn") {
                            let mut text_lines = Vec::new();
                            if let Some(title) = zh_cn.get("title").and_then(|t| t.as_str()) {
                                if !title.is_empty() {
                                    text_lines.push(format!("**{}**", title));
                                }
                            }
                            if let Some(content) = zh_cn.get("content").and_then(|c| c.as_array()) {
                                for row in content {
                                    if let Some(elements) = row.as_array() {
                                        let mut line_text = String::new();
                                        for el in elements {
                                            if let Some(text) =
                                                el.get("text").and_then(|t| t.as_str())
                                            {
                                                line_text.push_str(text);
                                            }
                                        }
                                        text_lines.push(line_text);
                                    }
                                }
                            }
                            json!({
                                "schema": "2.0",
                                "config": {"wide_screen_mode": true},
                                "body": {
                                    "elements": [
                                        {
                                            "tag": "markdown",
                                            "content": preprocess_markdown_for_feishu(&text_lines.join("\n"), true),
                                            "text_size": "heading"
                                        }
                                    ]
                                }
                            })
                            .to_string()
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                match facade
                    .update_message(message_id, "interactive", &card_content)
                    .await
                {
                    Ok(_) => {
                        previous_message_id = Some(message_id.to_string());
                    }
                    Err(err) if is_feishu_bad_request(&err) => {
                        tracing::warn!(
                            "[Feishu/outbound] update rendered placeholder failed with bad request, fallback to standalone send: message_id={} receive_id_type={} err={}",
                            message_id,
                            receive_id_type,
                            err
                        );
                        let sent = send_segment_direct(
                            facade,
                            receive_id,
                            receive_id_type,
                            "interactive",
                            &card_content,
                            Some(&stable_message_uuid(
                                uuid_seed,
                                index,
                                "interactive",
                                &card_content,
                            )),
                        )
                        .await
                        .map_err(hone_core::HoneError::Integration)?;
                        previous_message_id = Some(sent.message_id);
                    }
                    Err(err) => return Err(hone_core::HoneError::Integration(err)),
                }
                continue;
            }
        }

        let request_uuid =
            stable_message_uuid(uuid_seed, index, message.msg_type, &message.content);
        let sent = if should_thread_followups {
            if let Some(parent_id) = previous_message_id.as_deref() {
                match facade
                    .reply_message(
                        parent_id,
                        message.msg_type,
                        &message.content,
                        Some(&request_uuid),
                    )
                    .await
                {
                    Ok(sent) => sent,
                    Err(err) if is_feishu_bad_request(&err) => {
                        tracing::warn!(
                            "[Feishu/outbound] reply_message failed with bad request, fallback to standalone send: parent_id={} receive_id_type={} err={}",
                            parent_id,
                            receive_id_type,
                            err
                        );
                        send_segment_direct(
                            facade,
                            receive_id,
                            receive_id_type,
                            message.msg_type,
                            &message.content,
                            Some(&request_uuid),
                        )
                        .await
                        .map_err(hone_core::HoneError::Integration)?
                    }
                    Err(err) => return Err(hone_core::HoneError::Integration(err)),
                }
            } else {
                send_segment_direct(
                    facade,
                    receive_id,
                    receive_id_type,
                    message.msg_type,
                    &message.content,
                    Some(&request_uuid),
                )
                .await
                .map_err(hone_core::HoneError::Integration)?
            }
        } else {
            send_segment_direct(
                facade,
                receive_id,
                receive_id_type,
                message.msg_type,
                &message.content,
                Some(&request_uuid),
            )
            .await
            .map_err(hone_core::HoneError::Integration)?
        };
        previous_message_id = Some(sent.message_id);
    }
    Ok(total)
}

fn is_feishu_bad_request(err: &str) -> bool {
    err.contains("HTTP 400")
}

async fn send_segment_direct(
    facade: &FeishuApiClient,
    receive_id: &str,
    receive_id_type: &str,
    msg_type: &str,
    content: &str,
    uuid: Option<&str>,
) -> Result<FeishuSendResult, String> {
    if receive_id_type == "chat_id" {
        facade
            .send_chat_message(receive_id, msg_type, content, uuid)
            .await
    } else {
        facade
            .send_message(receive_id, msg_type, content, uuid)
            .await
    }
}

pub(crate) fn scheduled_send_idempotency(
    event: &SchedulerEvent,
    receive_id: &str,
    markdown: &str,
    receive_id_type: &str,
) -> SendIdempotency {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    markdown.hash(&mut hasher);
    let content_hash = hasher.finish();
    let dedup_key = format!(
        "scheduled:{}:{}:{}:{}:{content_hash}",
        event.delivery_key, event.job_id, receive_id_type, receive_id
    );
    let uuid_seed = format!(
        "{}:{}:{}:{}:{content_hash}",
        event.delivery_key, event.job_id, receive_id_type, receive_id
    );
    SendIdempotency {
        dedup_key,
        uuid_seed,
    }
}

fn stable_message_uuid(
    uuid_seed: Option<&str>,
    index: usize,
    msg_type: &str,
    content: &str,
) -> String {
    if let Some(seed) = uuid_seed {
        use std::hash::{Hash, Hasher};

        let composed = format!("{seed}:{index}:{msg_type}:{content}");
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        composed.hash(&mut hasher);
        format!("sched_{:016x}", hasher.finish())
    } else {
        uuid::Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepend_reply_prefix_is_idempotent_for_existing_prefix() {
        assert_eq!(
            prepend_reply_prefix(Some("@alice"), "@alice hello"),
            "@alice hello"
        );
    }

    #[test]
    fn stable_message_uuid_is_deterministic_for_seeded_messages() {
        let first = stable_message_uuid(Some("delivery-1"), 0, "interactive", "hello");
        let second = stable_message_uuid(Some("delivery-1"), 0, "interactive", "hello");
        let third = stable_message_uuid(Some("delivery-1"), 1, "interactive", "hello");
        assert_eq!(first, second);
        assert_ne!(first, third);
    }

    #[test]
    fn feishu_bad_request_detection_matches_http_400() {
        assert!(is_feishu_bad_request(
            "Feishu reply message failed: HTTP 400 Bad Request - {\"code\":1}"
        ));
        assert!(!is_feishu_bad_request(
            "Feishu reply message failed: HTTP 500 Internal Server Error"
        ));
    }
}
