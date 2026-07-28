use std::env;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};

const CLOUDFLARE_EMAIL_API_BASE_URL: &str = "https://api.cloudflare.com/client/v4/";
const CLOUDFLARE_ACCOUNT_ID_ENV: &str = "HONE_CLOUDFLARE_ACCOUNT_ID";
const CLOUDFLARE_EMAIL_API_TOKEN_ENV: &str = "HONE_CLOUDFLARE_EMAIL_API_TOKEN";
const EMAIL_FROM_ENV: &str = "HONE_EMAIL_FROM";

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

pub fn email_verification_sender_from_env(
    http_client: Client,
) -> Result<Arc<dyn EmailVerificationSender>, String> {
    let account_id = read_optional_env(CLOUDFLARE_ACCOUNT_ID_ENV)?;
    let api_token = read_optional_env(CLOUDFLARE_EMAIL_API_TOKEN_ENV)?;
    let from_address = read_optional_env(EMAIL_FROM_ENV)?;
    email_verification_sender_from_values(http_client, account_id, api_token, from_address)
}

fn email_verification_sender_from_values(
    http_client: Client,
    account_id: Option<String>,
    api_token: Option<String>,
    from_address: Option<String>,
) -> Result<Arc<dyn EmailVerificationSender>, String> {
    if account_id.is_none() && api_token.is_none() && from_address.is_none() {
        return Ok(Arc::new(UnconfiguredEmailVerificationSender));
    }

    let mut missing = Vec::new();
    if account_id.is_none() {
        missing.push(CLOUDFLARE_ACCOUNT_ID_ENV);
    }
    if api_token.is_none() {
        missing.push(CLOUDFLARE_EMAIL_API_TOKEN_ENV);
    }
    if from_address.is_none() {
        missing.push(EMAIL_FROM_ENV);
    }
    if !missing.is_empty() {
        return Err(format!(
            "Cloudflare 邮件配置不完整，缺少环境变量：{}",
            missing.join(", ")
        ));
    }

    Ok(Arc::new(CloudflareEmailVerificationSender::new(
        http_client,
        account_id.expect("account id checked above"),
        api_token.expect("API token checked above"),
        from_address.expect("from address checked above"),
    )?))
}

fn read_optional_env(name: &str) -> Result<Option<String>, String> {
    match env::var(name) {
        Ok(value) => {
            let value = value.trim();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_string()))
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(format!("环境变量 {name} 不是有效 UTF-8")),
    }
}

pub struct CloudflareEmailVerificationSender {
    http_client: Client,
    endpoint: Url,
    api_token: String,
    from_address: String,
}

impl CloudflareEmailVerificationSender {
    pub fn new(
        http_client: Client,
        account_id: String,
        api_token: String,
        from_address: String,
    ) -> Result<Self, String> {
        Self::new_with_api_base_url(
            http_client,
            account_id,
            api_token,
            from_address,
            CLOUDFLARE_EMAIL_API_BASE_URL,
        )
    }

    fn new_with_api_base_url(
        http_client: Client,
        account_id: String,
        api_token: String,
        from_address: String,
        api_base_url: &str,
    ) -> Result<Self, String> {
        let account_id = account_id.trim();
        if account_id.len() != 32 || !account_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!(
                "{CLOUDFLARE_ACCOUNT_ID_ENV} 必须是 32 位十六进制 Cloudflare account ID"
            ));
        }

        let api_token = api_token.trim();
        if api_token.is_empty() {
            return Err(format!("{CLOUDFLARE_EMAIL_API_TOKEN_ENV} 不能为空"));
        }

        let from_address = from_address.trim();
        if !is_valid_email_address(from_address) {
            return Err(format!("{EMAIL_FROM_ENV} 必须是有效的邮箱地址"));
        }

        let base_url = Url::parse(api_base_url)
            .map_err(|error| format!("Cloudflare Email API 基础地址无效: {error}"))?;
        if !matches!(base_url.scheme(), "http" | "https") {
            return Err("Cloudflare Email API 基础地址必须使用 HTTP(S)".to_string());
        }
        let endpoint = base_url
            .join(&format!("accounts/{account_id}/email/sending/send"))
            .map_err(|error| format!("Cloudflare Email API 地址构建失败: {error}"))?;

        Ok(Self {
            http_client,
            endpoint,
            api_token: api_token.to_string(),
            from_address: from_address.to_string(),
        })
    }
}

fn is_valid_email_address(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && !value.chars().any(char::is_whitespace)
        && !value.chars().any(char::is_control)
}

#[derive(Serialize)]
struct CloudflareEmailRequest<'a> {
    to: [&'a str; 1],
    from: CloudflareEmailAddress<'a>,
    subject: &'static str,
    text: String,
    html: String,
}

#[derive(Serialize)]
struct CloudflareEmailAddress<'a> {
    address: &'a str,
    name: &'static str,
}

#[derive(Deserialize)]
struct CloudflareEmailEnvelope {
    success: bool,
    #[serde(default)]
    errors: Vec<CloudflareEmailError>,
    result: Option<CloudflareEmailResult>,
}

#[derive(Deserialize)]
struct CloudflareEmailError {
    code: u64,
}

#[derive(Deserialize)]
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

#[async_trait]
impl EmailVerificationSender for CloudflareEmailVerificationSender {
    fn is_configured(&self) -> bool {
        true
    }

    async fn send_verification_code(
        &self,
        message: EmailVerificationMessage,
    ) -> Result<(), String> {
        if !is_valid_email_address(&message.to)
            || message.code.len() != 8
            || !message.code.bytes().all(|byte| byte.is_ascii_digit())
            || message.expires_in_minutes == 0
        {
            return Err("邮箱验证码消息无效".to_string());
        }
        let text = format!(
            "你的 HONE 登录验证码是：{}\n\n验证码将在 {} 分钟后失效。如果不是你本人操作，请忽略此邮件。",
            message.code, message.expires_in_minutes
        );
        let html = format!(
            "<p>你的 HONE 登录验证码是：</p>\
             <p style=\"font-size:28px;font-weight:700;letter-spacing:4px\">{}</p>\
             <p>验证码将在 {} 分钟后失效。如果不是你本人操作，请忽略此邮件。</p>",
            message.code, message.expires_in_minutes
        );
        let request = CloudflareEmailRequest {
            to: [&message.to],
            from: CloudflareEmailAddress {
                address: &self.from_address,
                name: "HONE",
            },
            subject: "你的 HONE 登录验证码",
            text,
            html,
        };

        let response = self
            .http_client
            .post(self.endpoint.clone())
            .bearer_auth(&self.api_token)
            .json(&request)
            .send()
            .await
            .map_err(|error| format!("Cloudflare 邮件请求失败: {}", request_error_kind(&error)))?;
        let status = response.status();
        let envelope = response
            .json::<CloudflareEmailEnvelope>()
            .await
            .map_err(|_| cloudflare_response_error(status, None))?;

        if !status.is_success() || !envelope.success {
            return Err(cloudflare_response_error(
                status,
                envelope.errors.first().map(|error| error.code),
            ));
        }

        let Some(result) = envelope.result else {
            return Err("Cloudflare 邮件发送响应缺少结果".to_string());
        };
        if !result.permanent_bounces.is_empty() {
            return Err("Cloudflare 拒绝了收件地址".to_string());
        }
        let has_message_id = result
            .message_id
            .as_deref()
            .is_some_and(|message_id| !message_id.trim().is_empty());
        if result.delivered.is_empty() && result.queued.is_empty() && !has_message_id {
            return Err("Cloudflare 未确认邮件已接受、投递或进入队列".to_string());
        }

        Ok(())
    }
}

fn request_error_kind(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "请求超时"
    } else if error.is_connect() {
        "连接失败"
    } else {
        "网络错误"
    }
}

fn cloudflare_response_error(status: StatusCode, code: Option<u64>) -> String {
    match code {
        Some(code) => format!(
            "Cloudflare 邮件发送失败（HTTP {}，错误码 {code}）",
            status.as_u16()
        ),
        None => format!("Cloudflare 邮件发送失败（HTTP {}）", status.as_u16()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::State;
    use axum::http::{HeaderMap, StatusCode};
    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::{Value, json};
    use tokio::sync::Mutex;

    use super::{
        CloudflareEmailVerificationSender, EmailVerificationMessage, EmailVerificationSender,
        UnconfiguredEmailVerificationSender, email_verification_sender_from_values,
    };

    const TEST_ACCOUNT_ID: &str = "0123456789abcdef0123456789abcdef";

    #[derive(Clone)]
    struct MockEmailApi {
        captured: Arc<Mutex<Option<(HeaderMap, Value)>>>,
        status: StatusCode,
        response: Value,
    }

    async fn capture_email_request(
        State(api): State<MockEmailApi>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        *api.captured.lock().await = Some((headers, body));
        (api.status, Json(api.response))
    }

    async fn mock_email_api(
        status: StatusCode,
        response: Value,
    ) -> (
        String,
        Arc<Mutex<Option<(HeaderMap, Value)>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let captured = Arc::new(Mutex::new(None));
        let api = MockEmailApi {
            captured: captured.clone(),
            status,
            response,
        };
        let app = Router::new()
            .route(
                &format!("/client/v4/accounts/{TEST_ACCOUNT_ID}/email/sending/send"),
                post(capture_email_request),
            )
            .with_state(api);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock email API");
        let address = listener.local_addr().expect("mock email API address");
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve mock email API");
        });
        (format!("http://{address}/client/v4/"), captured, handle)
    }

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

    #[tokio::test]
    async fn cloudflare_sender_sends_bounded_verification_message() {
        let (api_base_url, captured, server) = mock_email_api(
            StatusCode::OK,
            json!({
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "delivered": ["buyer@example.com"],
                    "permanent_bounces": [],
                    "queued": []
                }
            }),
        )
        .await;
        let sender = CloudflareEmailVerificationSender::new_with_api_base_url(
            reqwest::Client::new(),
            TEST_ACCOUNT_ID.to_string(),
            "test-api-token".to_string(),
            "verify@hone-claw.com".to_string(),
            &api_base_url,
        )
        .expect("valid sender");

        sender
            .send_verification_code(EmailVerificationMessage {
                to: "buyer@example.com".to_string(),
                code: "12345678".to_string(),
                expires_in_minutes: 10,
            })
            .await
            .expect("send verification email");

        let (headers, body) = captured
            .lock()
            .await
            .clone()
            .expect("captured email request");
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer test-api-token")
        );
        assert_eq!(body["to"], json!(["buyer@example.com"]));
        assert_eq!(
            body["from"],
            json!({"address": "verify@hone-claw.com", "name": "HONE"})
        );
        assert_eq!(body["subject"], "你的 HONE 登录验证码");
        assert!(
            body["text"]
                .as_str()
                .is_some_and(|text| text.contains("12345678") && text.contains("10 分钟"))
        );
        assert!(
            body["html"]
                .as_str()
                .is_some_and(|html| html.contains("12345678") && html.contains("10 分钟"))
        );
        server.abort();
    }

    #[tokio::test]
    async fn cloudflare_sender_does_not_leak_provider_body() {
        let (api_base_url, _captured, server) = mock_email_api(
            StatusCode::FORBIDDEN,
            json!({
                "success": false,
                "errors": [{
                    "code": 10000,
                    "message": "buyer@example.com 12345678 forbidden"
                }],
                "result": null
            }),
        )
        .await;
        let sender = CloudflareEmailVerificationSender::new_with_api_base_url(
            reqwest::Client::new(),
            TEST_ACCOUNT_ID.to_string(),
            "test-api-token".to_string(),
            "verify@hone-claw.com".to_string(),
            &api_base_url,
        )
        .expect("valid sender");

        let error = sender
            .send_verification_code(EmailVerificationMessage {
                to: "buyer@example.com".to_string(),
                code: "12345678".to_string(),
                expires_in_minutes: 10,
            })
            .await
            .expect_err("provider failure must be surfaced");

        assert!(error.contains("HTTP 403"));
        assert!(error.contains("10000"));
        assert!(!error.contains("buyer@example.com"));
        assert!(!error.contains("12345678"));
        server.abort();
    }

    #[tokio::test]
    async fn cloudflare_sender_accepts_message_id_only_response() {
        let (api_base_url, _captured, server) = mock_email_api(
            StatusCode::OK,
            json!({
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "message_id": "<accepted@example.com>"
                }
            }),
        )
        .await;
        let sender = CloudflareEmailVerificationSender::new_with_api_base_url(
            reqwest::Client::new(),
            TEST_ACCOUNT_ID.to_string(),
            "test-api-token".to_string(),
            "verify@hone-claw.com".to_string(),
            &api_base_url,
        )
        .expect("valid sender");

        sender
            .send_verification_code(EmailVerificationMessage {
                to: "buyer@example.com".to_string(),
                code: "12345678".to_string(),
                expires_in_minutes: 10,
            })
            .await
            .expect("message id proves provider acceptance");
        server.abort();
    }

    #[test]
    fn cloudflare_sender_rejects_unsafe_configuration() {
        assert!(
            CloudflareEmailVerificationSender::new(
                reqwest::Client::new(),
                "not-an-account-id".to_string(),
                "token".to_string(),
                "verify@hone-claw.com".to_string(),
            )
            .is_err()
        );
        assert!(
            CloudflareEmailVerificationSender::new(
                reqwest::Client::new(),
                TEST_ACCOUNT_ID.to_string(),
                "token".to_string(),
                "not-an-email".to_string(),
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_configuration_is_all_or_nothing_without_secret_leakage() {
        let unconfigured =
            email_verification_sender_from_values(reqwest::Client::new(), None, None, None)
                .expect("all absent keeps sender disabled");
        assert!(!unconfigured.is_configured());

        let error = email_verification_sender_from_values(
            reqwest::Client::new(),
            Some(TEST_ACCOUNT_ID.to_string()),
            Some("super-secret-token".to_string()),
            None,
        )
        .err()
        .expect("partial configuration must fail");
        assert!(error.contains("HONE_EMAIL_FROM"));
        assert!(!error.contains("super-secret-token"));
    }
}
