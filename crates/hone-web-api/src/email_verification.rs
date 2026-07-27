use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailVerificationMessage {
    pub to: String,
    pub code: String,
    pub expires_in_minutes: u32,
}

#[async_trait]
pub trait EmailVerificationSender: Send + Sync {
    fn is_configured(&self) -> bool;

    async fn send_verification_code(&self, message: EmailVerificationMessage)
    -> Result<(), String>;
}

#[derive(Debug, Default)]
pub struct UnconfiguredEmailVerificationSender;

#[async_trait]
impl EmailVerificationSender for UnconfiguredEmailVerificationSender {
    fn is_configured(&self) -> bool {
        false
    }

    async fn send_verification_code(
        &self,
        _message: EmailVerificationMessage,
    ) -> Result<(), String> {
        Err("HONE 邮箱验证码服务尚未配置".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EmailVerificationMessage, EmailVerificationSender, UnconfiguredEmailVerificationSender,
    };

    #[tokio::test]
    async fn unconfigured_sender_fails_closed() {
        let sender = UnconfiguredEmailVerificationSender;
        assert!(!sender.is_configured());
        let error = sender
            .send_verification_code(EmailVerificationMessage {
                to: "buyer@example.com".to_string(),
                code: "12345678".to_string(),
                expires_in_minutes: 10,
            })
            .await
            .expect_err("sender must stay disabled");
        assert!(error.contains("尚未配置"));
    }
}
