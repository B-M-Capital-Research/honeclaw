use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use hone_memory::WhopMembershipEvent;

use crate::state::AppState;

const DEFAULT_WHOP_COMPANY_ID: &str = "biz_h0UKqlfUJI55Am";
const DEFAULT_WHOP_PRODUCT_ID: &str = "prod_9jQsUKaifh6ZA";
const DEFAULT_WHOP_PLAN_ID: &str = "plan_ZXfsAisr4UOaw";
const WEBHOOK_MAX_BODY_BYTES: usize = 1024 * 1024;
const WEBHOOK_TIMESTAMP_TOLERANCE_SECS: i64 = 5 * 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
struct WhopWebhookConfig {
    secret: String,
    company_id: String,
    product_id: String,
    plan_id: String,
}

impl WhopWebhookConfig {
    fn from_env() -> Result<Self, String> {
        let secret = std::env::var("HONE_WHOP_WEBHOOK_SECRET")
            .unwrap_or_default()
            .trim()
            .to_string();
        if secret.is_empty() {
            return Err("HONE_WHOP_WEBHOOK_SECRET 未配置".to_string());
        }
        Ok(Self {
            secret,
            company_id: env_or("HONE_WHOP_COMPANY_ID", DEFAULT_WHOP_COMPANY_ID),
            product_id: env_or("HONE_WHOP_PRODUCT_ID", DEFAULT_WHOP_PRODUCT_ID),
            plan_id: env_or("HONE_WHOP_PLAN_ID", DEFAULT_WHOP_PLAN_ID),
        })
    }
}

#[derive(Debug, Deserialize)]
struct WhopWebhookEnvelope {
    id: String,
    api_version: String,
    timestamp: String,
    #[serde(rename = "type")]
    event_type: String,
    data: Value,
    company_id: Option<String>,
}

pub(crate) async fn handle_whop_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let config = match WhopWebhookConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            return crate::routes::json_error(StatusCode::SERVICE_UNAVAILABLE, error);
        }
    };
    if body.len() > WEBHOOK_MAX_BODY_BYTES {
        return crate::routes::json_error(StatusCode::PAYLOAD_TOO_LARGE, "Whop webhook 过大");
    }
    if let Err(error) = verify_standard_webhook(&headers, &body, &config.secret) {
        return crate::routes::json_error(StatusCode::UNAUTHORIZED, error);
    }
    let envelope: WhopWebhookEnvelope = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            return crate::routes::json_error(
                StatusCode::BAD_REQUEST,
                "Whop webhook JSON 格式不合法",
            );
        }
    };
    if header_string(&headers, "webhook-id").as_deref() != Some(envelope.id.as_str()) {
        return crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "Whop webhook header/body ID 不一致",
        );
    }
    if !matches!(
        envelope.event_type.as_str(),
        "membership.activated"
            | "membership.deactivated"
            | "membership.cancel_at_period_end_changed"
    ) {
        return Json(json!({ "ok": true, "ignored": true })).into_response();
    }
    let event = match membership_event_from_envelope(&envelope, &config) {
        Ok(event) => event,
        Err(error) => return crate::routes::json_error(StatusCode::UNPROCESSABLE_ENTITY, error),
    };
    match state.web_auth.upsert_whop_membership(event) {
        Ok((user, outcome)) => Json(json!({
            "ok": true,
            "user_id": user.user_id,
            "outcome": format!("{outcome:?}").to_ascii_lowercase(),
        }))
        .into_response(),
        Err(error) => crate::routes::json_error(
            StatusCode::CONFLICT,
            format!("Whop membership 写入失败: {error}"),
        ),
    }
}

fn membership_event_from_envelope(
    envelope: &WhopWebhookEnvelope,
    config: &WhopWebhookConfig,
) -> Result<WhopMembershipEvent, String> {
    if envelope.api_version != "v1" {
        return Err("Whop webhook API version 必须为 v1".to_string());
    }
    let company_id = envelope
        .company_id
        .as_deref()
        .ok_or_else(|| "Whop company_id 缺失".to_string())?;
    if company_id != config.company_id {
        return Err("Whop business 不匹配".to_string());
    }
    let membership_id = json_string(&envelope.data, &["id"])?;
    let whop_user_id = json_string(&envelope.data, &["user", "id"])?;
    let email_address = json_string(&envelope.data, &["user", "email"])?;
    let product_id = json_string(&envelope.data, &["product", "id"])?;
    let plan_id = json_string(&envelope.data, &["plan", "id"])?;
    if product_id != config.product_id {
        return Err("Whop product 不匹配".to_string());
    }
    if plan_id != config.plan_id {
        return Err("Whop plan 不匹配".to_string());
    }
    chrono::DateTime::parse_from_rfc3339(&envelope.timestamp)
        .map_err(|_| "Whop event timestamp 格式不合法".to_string())?;
    let cancel_at_period_end =
        json_optional_bool(&envelope.data, &["cancel_at_period_end"]).unwrap_or(false);
    let status = json_optional_string(&envelope.data, &["status"]).unwrap_or_else(|| {
        match envelope.event_type.as_str() {
            "membership.activated" => "active",
            "membership.deactivated" => "canceled",
            "membership.cancel_at_period_end_changed" if cancel_at_period_end => "canceling",
            _ => "active",
        }
        .to_string()
    });
    Ok(WhopMembershipEvent {
        membership_id,
        whop_user_id,
        email_address,
        company_id: company_id.to_string(),
        product_id,
        plan_id,
        status,
        manage_url: json_optional_string(&envelope.data, &["manage_url"]),
        renewal_period_start: json_optional_string(&envelope.data, &["renewal_period_start"]),
        renewal_period_end: json_optional_string(&envelope.data, &["renewal_period_end"]),
        cancel_at_period_end,
        event_id: envelope.id.clone(),
        event_at: envelope.timestamp.clone(),
    })
}

fn verify_standard_webhook(headers: &HeaderMap, body: &[u8], secret: &str) -> Result<(), String> {
    let webhook_id =
        header_string(headers, "webhook-id").ok_or_else(|| "缺少 webhook-id".to_string())?;
    let timestamp = header_string(headers, "webhook-timestamp")
        .ok_or_else(|| "缺少 webhook-timestamp".to_string())?;
    let signature = header_string(headers, "webhook-signature")
        .ok_or_else(|| "缺少 webhook-signature".to_string())?;
    if webhook_id.contains('.') || timestamp.contains('.') {
        return Err("Whop webhook header 格式不合法".to_string());
    }
    let timestamp_secs = timestamp
        .parse::<i64>()
        .map_err(|_| "Whop webhook timestamp 格式不合法".to_string())?;
    let now = chrono::Utc::now().timestamp();
    if now.abs_diff(timestamp_secs) > WEBHOOK_TIMESTAMP_TOLERANCE_SECS as u64 {
        return Err("Whop webhook timestamp 已过期".to_string());
    }
    let secret = secret.trim();
    let secret_value = secret
        .strip_prefix("ws_")
        .ok_or_else(|| "Whop webhook secret 格式不合法".to_string())?;
    if secret_value.is_empty() {
        return Err("Whop webhook secret 格式不合法".to_string());
    }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| "Whop webhook secret 长度不合法".to_string())?;
    mac.update(webhook_id.as_bytes());
    mac.update(b".");
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(body);
    let expected = mac.finalize().into_bytes();
    let matched = signature.split_whitespace().any(|candidate| {
        let Some(encoded) = candidate.strip_prefix("v1,") else {
            return false;
        };
        decode_base64(encoded)
            .ok()
            .is_some_and(|received| received.as_slice().ct_eq(expected.as_slice()).into())
    });
    if !matched {
        return Err("Whop webhook 签名无效".to_string());
    }
    Ok(())
}

fn decode_base64(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD
        .decode(value)
        .or_else(|_| STANDARD_NO_PAD.decode(value))
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)?
        .to_str()
        .ok()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn json_value_at<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(root, |value, key| value.get(*key))
}

fn json_string(root: &Value, path: &[&str]) -> Result<String, String> {
    json_optional_string(root, path).ok_or_else(|| format!("Whop {} 缺失", path.join(".")))
}

fn json_optional_string(root: &Value, path: &[&str]) -> Option<String> {
    json_value_at(root, path)?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn json_optional_bool(root: &Value, path: &[&str]) -> Option<bool> {
    json_value_at(root, path)?.as_bool()
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_WHOP_COMPANY_ID, DEFAULT_WHOP_PLAN_ID, DEFAULT_WHOP_PRODUCT_ID, HmacSha256,
        WhopWebhookConfig, WhopWebhookEnvelope, membership_event_from_envelope,
        verify_standard_webhook,
    };
    use axum::http::{HeaderMap, HeaderValue};
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use hmac::Mac;
    use serde_json::json;

    fn signed_headers(body: &[u8], secret: &str) -> HeaderMap {
        let webhook_id = "msg_test123";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
        mac.update(webhook_id.as_bytes());
        mac.update(b".");
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(body);
        let signature = STANDARD.encode(mac.finalize().into_bytes());
        let mut headers = HeaderMap::new();
        headers.insert("webhook-id", HeaderValue::from_static(webhook_id));
        headers.insert(
            "webhook-timestamp",
            HeaderValue::from_str(&timestamp).expect("timestamp"),
        );
        headers.insert(
            "webhook-signature",
            HeaderValue::from_str(&format!("v1,{signature}")).expect("signature"),
        );
        headers
    }

    #[test]
    fn standard_webhook_signature_accepts_ws_secret_and_rejects_legacy_or_tampered_input() {
        let body = br#"{"type":"membership.activated"}"#;
        let secret = format!("ws_{}", "test-only-not-a-secret-".repeat(3));
        let headers = signed_headers(body, &secret);
        verify_standard_webhook(&headers, body, &secret).expect("valid current Whop signature");
        assert!(verify_standard_webhook(&headers, b"{}", &secret).is_err());
        assert!(
            verify_standard_webhook(
                &headers,
                body,
                "whsec_MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="
            )
            .is_err()
        );
    }

    #[test]
    fn membership_parser_requires_the_canonical_business_product_and_plan() {
        let envelope: WhopWebhookEnvelope = serde_json::from_value(json!({
            "id": "msg_test123",
            "api_version": "v1",
            "timestamp": "2026-07-26T12:00:00Z",
            "type": "membership.activated",
            "company_id": DEFAULT_WHOP_COMPANY_ID,
            "data": {
                "id": "mem_test123",
                "status": "active",
                "user": {
                    "id": "user_test123",
                    "email": "Buyer@Example.com"
                },
                "product": { "id": DEFAULT_WHOP_PRODUCT_ID },
                "plan": { "id": DEFAULT_WHOP_PLAN_ID },
                "manage_url": "https://whop.com/billing/manage/mem_test123",
                "renewal_period_end": "2027-07-26T12:00:00Z"
            }
        }))
        .expect("envelope");
        let config = WhopWebhookConfig {
            secret: "unused".to_string(),
            company_id: DEFAULT_WHOP_COMPANY_ID.to_string(),
            product_id: DEFAULT_WHOP_PRODUCT_ID.to_string(),
            plan_id: DEFAULT_WHOP_PLAN_ID.to_string(),
        };
        let event = membership_event_from_envelope(&envelope, &config).expect("event");
        assert_eq!(event.email_address, "Buyer@Example.com");
        assert_eq!(event.status, "active");

        let wrong = WhopWebhookConfig {
            product_id: "prod_wrong".to_string(),
            ..config
        };
        assert!(membership_event_from_envelope(&envelope, &wrong).is_err());
    }
}
