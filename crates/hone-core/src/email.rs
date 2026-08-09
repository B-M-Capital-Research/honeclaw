//! Outbound transactional email through Cloudflare Email Sending.
//!
//! Email is a side channel for scheduled pushes the user explicitly asked to
//! receive by mail. It is therefore best-effort by construction: a send that
//! fails, or a deployment with no credential configured, must never turn a
//! push that otherwise succeeded into a failure. Every entry point here
//! returns a typed outcome the caller can log and move on from.

use std::time::Duration;

use serde::Deserialize;
use serde_json::json;

use crate::config::EmailConfig;

/// What happened to one send attempt. `Skipped` is not an error: it is the
/// expected state of a deployment that has not configured a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailOutcome {
    Sent { message_id: Option<String> },
    Skipped(EmailSkipReason),
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailSkipReason {
    /// No account, sender or token — the feature is simply off here.
    NotConfigured,
    /// The recipient has no usable address on file.
    NoRecipient,
    /// Hone's own daily ceiling, separate from the provider's quota.
    DailyLimitReached,
}

impl EmailOutcome {
    pub fn was_sent(&self) -> bool {
        matches!(self, EmailOutcome::Sent { .. })
    }
}

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    /// Plain text is required: some clients never render the HTML part, and a
    /// push whose body only exists in HTML would arrive empty for them.
    pub text: String,
    pub html: Option<String>,
}

pub struct EmailSender {
    config: EmailConfig,
    client: reqwest::Client,
}

impl EmailSender {
    pub fn new(config: EmailConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout.max(1)))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    pub fn is_configured(&self) -> bool {
        self.config.is_configured()
    }

    pub fn daily_limit(&self) -> u32 {
        self.config.daily_limit
    }

    /// Sends one message. `sent_today` is supplied by the caller because the
    /// counter belongs to whatever store already tracks per-actor delivery;
    /// duplicating it here would let the two disagree.
    pub async fn send(&self, message: &EmailMessage, sent_today: u32) -> EmailOutcome {
        if !self.config.is_configured() {
            return EmailOutcome::Skipped(EmailSkipReason::NotConfigured);
        }
        if !recipient_is_plausible(&message.to) {
            return EmailOutcome::Skipped(EmailSkipReason::NoRecipient);
        }
        if self.config.daily_limit > 0 && sent_today >= self.config.daily_limit {
            return EmailOutcome::Skipped(EmailSkipReason::DailyLimitReached);
        }

        let from = self.config.from_address.trim();
        let from_name = self.config.from_name.trim();
        let from_header = if from_name.is_empty() {
            from.to_string()
        } else {
            format!("{from_name} <{from}>")
        };

        let mut payload = json!({
            "to": message.to.trim(),
            "from": from_header,
            "subject": message.subject.trim(),
            "text": message.text,
        });
        if let Some(html) = message.html.as_ref().filter(|html| !html.trim().is_empty()) {
            payload["html"] = json!(html);
        }

        let response = self
            .client
            .post(self.config.send_endpoint())
            .bearer_auth(self.config.resolved_api_token())
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(response) => {
                let status = response.status();
                let envelope = response.json::<CloudflareEmailEnvelope>().await;
                classify_response(status, envelope)
            }
            Err(error) => EmailOutcome::Failed(format!(
                "cloudflare email send transport error: {}",
                request_error_kind(&error)
            )),
        }
    }
}

/// Deliberately permissive: address validity is the provider's judgement, not
/// a regex's. This only rejects what obviously cannot be delivered, so a valid
/// but unusual address is never silently dropped.
pub fn recipient_is_plausible(address: &str) -> bool {
    let address = address.trim();
    if address.len() < 3 || address.len() > 254 {
        return false;
    }
    if address.chars().any(char::is_whitespace) {
        return false;
    }
    let mut parts = address.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

#[derive(Debug, Deserialize)]
struct CloudflareEmailEnvelope {
    success: bool,
    #[serde(default)]
    errors: Vec<CloudflareEmailError>,
    result: Option<CloudflareEmailResult>,
}

#[derive(Debug, Deserialize)]
struct CloudflareEmailError {
    code: u64,
}

#[derive(Debug, Deserialize)]
struct CloudflareEmailResult {
    #[serde(default)]
    delivered: Vec<String>,
    #[serde(default, alias = "messageId")]
    message_id: Option<String>,
    #[serde(default)]
    permanent_bounces: Vec<String>,
    #[serde(default)]
    queued: Vec<String>,
}

fn classify_response(
    status: reqwest::StatusCode,
    envelope: Result<CloudflareEmailEnvelope, reqwest::Error>,
) -> EmailOutcome {
    let Ok(envelope) = envelope else {
        return EmailOutcome::Failed(format!(
            "cloudflare email send returned an invalid response: {status}"
        ));
    };
    if !status.is_success() || !envelope.success {
        let code = envelope
            .errors
            .first()
            .map(|error| error.code.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return EmailOutcome::Failed(format!(
            "cloudflare email send failed: {status} code={code}"
        ));
    }
    let Some(result) = envelope.result else {
        return EmailOutcome::Failed("cloudflare email send response missing result".to_string());
    };
    if !result.permanent_bounces.is_empty() {
        return EmailOutcome::Failed("cloudflare email permanently bounced".to_string());
    }
    let message_id = result
        .message_id
        .filter(|message_id| !message_id.trim().is_empty());
    if result.delivered.is_empty() && result.queued.is_empty() && message_id.is_none() {
        return EmailOutcome::Failed("cloudflare email did not accept the message".to_string());
    }
    EmailOutcome::Sent { message_id }
}

fn request_error_kind(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else if error.is_request() {
        "request"
    } else {
        "transport"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configured() -> EmailConfig {
        EmailConfig {
            account_id: "acct".to_string(),
            api_token: "token".to_string(),
            from_address: "push@hone-claw.com".to_string(),
            ..EmailConfig::default()
        }
    }

    #[test]
    fn a_deployment_without_a_credential_is_off_rather_than_broken() {
        // A missing token must not fail a scheduled push that otherwise
        // succeeded; the push is the product, the email is an extra.
        let config = EmailConfig::default();
        assert!(!config.is_configured());

        let with_account = EmailConfig {
            account_id: "acct".to_string(),
            ..EmailConfig::default()
        };
        // Partially configured is off, not half-on.
        assert!(!with_account.is_configured());
        assert!(configured().is_configured());
    }

    #[test]
    fn the_token_comes_from_the_environment_when_the_config_omits_it() {
        // Secrets belong outside the repository; the config file is a fallback
        // for deployments that manage them elsewhere.
        let config = EmailConfig {
            api_token: String::new(),
            api_token_env: "HONE_TEST_EMAIL_TOKEN".to_string(),
            ..configured()
        };
        assert_eq!(config.resolved_api_token(), "");
        unsafe { std::env::set_var("HONE_TEST_EMAIL_TOKEN", " from-env ") };
        assert_eq!(config.resolved_api_token(), "from-env");
        unsafe { std::env::remove_var("HONE_TEST_EMAIL_TOKEN") };
    }

    #[test]
    fn the_send_endpoint_follows_the_account() {
        assert_eq!(
            configured().send_endpoint(),
            "https://api.cloudflare.com/client/v4/accounts/acct/email/sending/send"
        );
    }

    #[tokio::test]
    async fn an_unconfigured_sender_skips_without_calling_out() {
        let sender = EmailSender::new(EmailConfig::default());
        assert_eq!(
            sender
                .send(
                    &EmailMessage {
                        to: "someone@example.com".to_string(),
                        subject: "s".to_string(),
                        text: "t".to_string(),
                        html: None,
                    },
                    0
                )
                .await,
            EmailOutcome::Skipped(EmailSkipReason::NotConfigured)
        );
    }

    #[tokio::test]
    async fn hones_own_daily_ceiling_stops_a_runaway_scheduler() {
        let sender = EmailSender::new(EmailConfig {
            daily_limit: 2,
            ..configured()
        });
        let message = EmailMessage {
            to: "someone@example.com".to_string(),
            subject: "s".to_string(),
            text: "t".to_string(),
            html: None,
        };
        assert_eq!(
            sender.send(&message, 2).await,
            EmailOutcome::Skipped(EmailSkipReason::DailyLimitReached)
        );
    }

    #[test]
    fn obviously_undeliverable_addresses_are_rejected_and_unusual_ones_are_not() {
        for good in ["a@b.co", "first.last+tag@sub.example.com", "用户@例子.中国"] {
            assert!(recipient_is_plausible(good), "{good}");
        }
        for bad in [
            "",
            "no-at-sign",
            "a@b",
            "two@at@signs.com",
            "has space@x.com",
        ] {
            assert!(!recipient_is_plausible(bad), "{bad}");
        }
    }

    #[test]
    fn provider_success_requires_an_accepted_delivery() {
        let accepted: CloudflareEmailEnvelope = serde_json::from_value(json!({
            "success": true,
            "errors": [],
            "result": {
                "delivered": [],
                "permanent_bounces": [],
                "queued": ["recipient@example.com"],
                "message_id": "message-1"
            }
        }))
        .expect("valid envelope");
        assert_eq!(
            classify_response(reqwest::StatusCode::OK, Ok(accepted)),
            EmailOutcome::Sent {
                message_id: Some("message-1".to_string())
            }
        );

        let empty: CloudflareEmailEnvelope = serde_json::from_value(json!({
            "success": true,
            "errors": [],
            "result": { "delivered": [], "permanent_bounces": [], "queued": [] }
        }))
        .expect("valid envelope");
        assert!(matches!(
            classify_response(reqwest::StatusCode::OK, Ok(empty)),
            EmailOutcome::Failed(_)
        ));
    }

    #[test]
    fn provider_failures_expose_only_status_and_numeric_code() {
        let failed: CloudflareEmailEnvelope = serde_json::from_value(json!({
            "success": false,
            "errors": [{ "code": 10102, "message": "recipient@example.com rejected" }],
            "result": null
        }))
        .expect("valid envelope");
        let outcome = classify_response(reqwest::StatusCode::FORBIDDEN, Ok(failed));
        let EmailOutcome::Failed(detail) = outcome else {
            panic!("expected failure");
        };
        assert!(detail.contains("403"));
        assert!(detail.contains("10102"));
        assert!(!detail.contains("recipient@example.com"));
    }

    #[test]
    fn permanent_bounce_is_not_reported_as_sent() {
        let bounced: CloudflareEmailEnvelope = serde_json::from_value(json!({
            "success": true,
            "errors": [],
            "result": {
                "delivered": [],
                "permanent_bounces": ["recipient@example.com"],
                "queued": []
            }
        }))
        .expect("valid envelope");
        assert!(matches!(
            classify_response(reqwest::StatusCode::OK, Ok(bounced)),
            EmailOutcome::Failed(_)
        ));
    }
}
