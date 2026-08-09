//! Outbound transactional email through Cloudflare Email Sending.
//!
//! Email is a side channel for scheduled pushes the user explicitly asked to
//! receive by mail. It is therefore best-effort by construction: a send that
//! fails, or a deployment with no credential configured, must never turn a
//! push that otherwise succeeded into a failure. Every entry point here
//! returns a typed outcome the caller can log and move on from.

use std::time::Duration;

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
                let body = response.text().await.unwrap_or_default();
                if status.is_success() {
                    EmailOutcome::Sent {
                        message_id: extract_message_id(&body),
                    }
                } else {
                    // The body can echo request fields; the token is only ever
                    // a header, so it cannot appear here, but the text is still
                    // bounded before it reaches a log.
                    EmailOutcome::Failed(format!(
                        "cloudflare email send failed: {status} {}",
                        bounded_detail(&body)
                    ))
                }
            }
            Err(error) => EmailOutcome::Failed(format!(
                "cloudflare email send transport error: {}",
                bounded_detail(&error.to_string())
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

fn extract_message_id(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    for pointer in ["/result/message_id", "/result/messageId", "/messageId"] {
        if let Some(id) = value.pointer(pointer).and_then(serde_json::Value::as_str) {
            return Some(id.to_string());
        }
    }
    None
}

fn bounded_detail(detail: &str) -> String {
    let mut end = detail.len().min(300);
    while end > 0 && !detail.is_char_boundary(end) {
        end -= 1;
    }
    detail[..end].replace('\n', " ")
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
    fn failure_detail_is_bounded_and_single_line() {
        let detail = bounded_detail(&format!("{}\nsecond line", "x".repeat(500)));
        assert!(detail.chars().count() <= 300);
        assert!(!detail.contains('\n'));
    }
}
