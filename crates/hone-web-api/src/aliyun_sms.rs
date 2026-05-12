use std::collections::BTreeMap;
use std::fmt;

use base64::Engine as _;
use hmac::{Hmac, Mac};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::Deserialize;
use sha1::Sha1;
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

const API_VERSION: &str = "2017-05-25";
const DEFAULT_ENDPOINT: &str = "dypnsapi.aliyuncs.com";
const DEFAULT_COUNTRY_CODE: &str = "86";
const DEFAULT_SIGN_NAME: &str = "速通互联验证码";
const DEFAULT_TEMPLATE_CODE: &str = "100001";
const DEFAULT_TEMPLATE_PARAM: &str = r####"{"code":"##code##","min":"5"}"####;
const DEFAULT_VALID_TIME_SECS: &str = "300";
const DEFAULT_INTERVAL_SECS: &str = "60";

const ALIYUN_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[derive(Debug, Clone)]
pub struct AliyunSmsConfig {
    access_key_id: String,
    access_key_secret: String,
    endpoint: String,
    country_code: String,
    sign_name: String,
    template_code: String,
    template_param: String,
}

impl AliyunSmsConfig {
    pub fn from_env() -> Result<Self, AliyunSmsError> {
        let access_key_id = env_first(&[
            "ALIBABA_CLOUD_ACCESS_KEY_ID",
            "ALIYUN_ACCESS_KEY_ID",
            "HONE_ALIYUN_ACCESS_KEY_ID",
        ])
        .ok_or_else(|| {
            AliyunSmsError::config("缺少阿里云短信 AccessKeyId，请设置 ALIBABA_CLOUD_ACCESS_KEY_ID")
        })?;
        let access_key_secret = env_first(&[
            "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
            "ALIYUN_ACCESS_KEY_SECRET",
            "HONE_ALIYUN_ACCESS_KEY_SECRET",
        ])
        .ok_or_else(|| {
            AliyunSmsError::config(
                "缺少阿里云短信 AccessKeySecret，请设置 ALIBABA_CLOUD_ACCESS_KEY_SECRET",
            )
        })?;

        Ok(Self {
            access_key_id,
            access_key_secret,
            endpoint: env_or("HONE_ALIYUN_SMS_ENDPOINT", DEFAULT_ENDPOINT),
            country_code: env_or("HONE_ALIYUN_SMS_COUNTRY_CODE", DEFAULT_COUNTRY_CODE),
            sign_name: env_or("HONE_ALIYUN_SMS_SIGN_NAME", DEFAULT_SIGN_NAME),
            template_code: env_or("HONE_ALIYUN_SMS_TEMPLATE_CODE", DEFAULT_TEMPLATE_CODE),
            template_param: env_or("HONE_ALIYUN_SMS_TEMPLATE_PARAM", DEFAULT_TEMPLATE_PARAM),
        })
    }
}

#[derive(Debug, Clone)]
pub enum AliyunSmsErrorKind {
    Config,
    Transport,
    Provider,
}

#[derive(Debug, Clone)]
pub struct AliyunSmsError {
    pub kind: AliyunSmsErrorKind,
    message: String,
}

impl AliyunSmsError {
    fn config(message: impl Into<String>) -> Self {
        Self {
            kind: AliyunSmsErrorKind::Config,
            message: message.into(),
        }
    }

    fn transport(message: impl Into<String>) -> Self {
        Self {
            kind: AliyunSmsErrorKind::Transport,
            message: message.into(),
        }
    }

    fn provider(message: impl Into<String>) -> Self {
        Self {
            kind: AliyunSmsErrorKind::Provider,
            message: message.into(),
        }
    }
}

impl fmt::Display for AliyunSmsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AliyunSmsError {}

#[derive(Debug, Deserialize)]
struct AliyunSmsResponse {
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "Success")]
    success: Option<bool>,
    #[serde(rename = "Model")]
    model: Option<AliyunSmsModel>,
}

#[derive(Debug, Deserialize)]
struct AliyunSmsModel {
    #[serde(rename = "VerifyResult")]
    verify_result: Option<String>,
}

pub async fn send_verify_code(
    http: &reqwest::Client,
    phone_number: &str,
) -> Result<(), AliyunSmsError> {
    let config = AliyunSmsConfig::from_env()?;
    send_verify_code_with_config(http, &config, phone_number).await
}

pub async fn check_verify_code(
    http: &reqwest::Client,
    phone_number: &str,
    verify_code: &str,
) -> Result<bool, AliyunSmsError> {
    let config = AliyunSmsConfig::from_env()?;
    check_verify_code_with_config(http, &config, phone_number, verify_code).await
}

async fn send_verify_code_with_config(
    http: &reqwest::Client,
    config: &AliyunSmsConfig,
    phone_number: &str,
) -> Result<(), AliyunSmsError> {
    let mut params = BTreeMap::new();
    params.insert("CountryCode".to_string(), config.country_code.clone());
    params.insert("PhoneNumber".to_string(), phone_number.to_string());
    params.insert("SignName".to_string(), config.sign_name.clone());
    params.insert("TemplateCode".to_string(), config.template_code.clone());
    params.insert("TemplateParam".to_string(), config.template_param.clone());
    params.insert("CodeType".to_string(), "1".to_string());
    params.insert("ValidTime".to_string(), DEFAULT_VALID_TIME_SECS.to_string());
    params.insert("Interval".to_string(), DEFAULT_INTERVAL_SECS.to_string());
    params.insert("DuplicatePolicy".to_string(), "1".to_string());
    params.insert("ReturnVerifyCode".to_string(), "false".to_string());

    let response = call_aliyun(http, config, "SendSmsVerifyCode", params).await?;
    if response.code.as_deref() == Some("OK") && response.success.unwrap_or(false) {
        Ok(())
    } else {
        Err(provider_error("发送短信验证码失败", response))
    }
}

async fn check_verify_code_with_config(
    http: &reqwest::Client,
    config: &AliyunSmsConfig,
    phone_number: &str,
    verify_code: &str,
) -> Result<bool, AliyunSmsError> {
    let mut params = BTreeMap::new();
    params.insert("CountryCode".to_string(), config.country_code.clone());
    params.insert("PhoneNumber".to_string(), phone_number.to_string());
    params.insert("VerifyCode".to_string(), verify_code.to_string());
    params.insert("CaseAuthPolicy".to_string(), "1".to_string());

    let response = call_aliyun(http, config, "CheckSmsVerifyCode", params).await?;
    if response.code.as_deref() != Some("OK") || !response.success.unwrap_or(false) {
        return Err(provider_error("核验短信验证码失败", response));
    }

    Ok(response
        .model
        .and_then(|model| model.verify_result)
        .is_some_and(|value| value == "PASS"))
}

async fn call_aliyun(
    http: &reqwest::Client,
    config: &AliyunSmsConfig,
    action: &str,
    params: BTreeMap<String, String>,
) -> Result<AliyunSmsResponse, AliyunSmsError> {
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let nonce = Uuid::new_v4().to_string();
    let url = signed_url(config, action, params, &timestamp, &nonce)?;
    let response = http
        .get(&url)
        .send()
        .await
        .map_err(|err| AliyunSmsError::transport(format!("调用阿里云短信接口失败: {err}")))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| AliyunSmsError::transport(format!("读取阿里云短信响应失败: {err}")))?;
    let parsed = serde_json::from_str::<AliyunSmsResponse>(&text).map_err(|err| {
        AliyunSmsError::provider(format!("阿里云短信响应解析失败: {err}; status={status}"))
    })?;
    if !status.is_success() {
        return Err(provider_error("阿里云短信接口返回错误", parsed));
    }
    Ok(parsed)
}

fn provider_error(prefix: &str, response: AliyunSmsResponse) -> AliyunSmsError {
    let code = response.code.unwrap_or_else(|| "UNKNOWN".to_string());
    let message = response.message.unwrap_or_else(|| "未知错误".to_string());
    AliyunSmsError::provider(format!("{prefix}: {code} {message}"))
}

fn signed_url(
    config: &AliyunSmsConfig,
    action: &str,
    operation_params: BTreeMap<String, String>,
    timestamp: &str,
    nonce: &str,
) -> Result<String, AliyunSmsError> {
    let mut params = BTreeMap::new();
    params.insert("AccessKeyId".to_string(), config.access_key_id.clone());
    params.insert("Action".to_string(), action.to_string());
    params.insert("Format".to_string(), "JSON".to_string());
    params.insert("SignatureMethod".to_string(), "HMAC-SHA1".to_string());
    params.insert("SignatureNonce".to_string(), nonce.to_string());
    params.insert("SignatureVersion".to_string(), "1.0".to_string());
    params.insert("Timestamp".to_string(), timestamp.to_string());
    params.insert("Version".to_string(), API_VERSION.to_string());
    params.extend(operation_params);

    let canonical = canonical_query(&params);
    let string_to_sign = format!("GET&%2F&{}", aliyun_percent_encode(&canonical));
    let mut mac = HmacSha1::new_from_slice(format!("{}&", config.access_key_secret).as_bytes())
        .map_err(|err| AliyunSmsError::config(format!("阿里云签名初始化失败: {err}")))?;
    mac.update(string_to_sign.as_bytes());
    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    params.insert("Signature".to_string(), signature);

    let endpoint = config.endpoint.trim();
    let base = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", endpoint.trim_end_matches('/'))
    };
    Ok(format!("{base}/?{}", canonical_query(&params)))
}

fn canonical_query(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                aliyun_percent_encode(key),
                aliyun_percent_encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn aliyun_percent_encode(value: &str) -> String {
    utf8_percent_encode(value, ALIYUN_ENCODE_SET).to_string()
}

fn env_first(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        AliyunSmsConfig, AliyunSmsModel, AliyunSmsResponse, aliyun_percent_encode,
        send_verify_code, signed_url,
    };
    use std::collections::BTreeMap;

    fn test_config() -> AliyunSmsConfig {
        AliyunSmsConfig {
            access_key_id: "testid".to_string(),
            access_key_secret: "testsecret".to_string(),
            endpoint: "dypnsapi.aliyuncs.com".to_string(),
            country_code: "86".to_string(),
            sign_name: "速通互联验证码".to_string(),
            template_code: "100001".to_string(),
            template_param: r####"{"code":"##code##","min":"5"}"####.to_string(),
        }
    }

    #[test]
    fn aliyun_encoding_keeps_rfc3986_safe_chars() {
        assert_eq!(aliyun_percent_encode("AZaz09-_.~"), "AZaz09-_.~");
        assert_eq!(
            aliyun_percent_encode(r####"{"code":"##code##","min":"5"}"####),
            "%7B%22code%22%3A%22%23%23code%23%23%22%2C%22min%22%3A%225%22%7D"
        );
    }

    #[test]
    fn signed_url_contains_sorted_encoded_params_and_signature() {
        let mut params = BTreeMap::new();
        params.insert("PhoneNumber".to_string(), "13800138000".to_string());
        params.insert(
            "TemplateParam".to_string(),
            r####"{"code":"##code##","min":"5"}"####.to_string(),
        );

        let url = signed_url(
            &test_config(),
            "SendSmsVerifyCode",
            params,
            "2026-05-12T00:00:00Z",
            "nonce-1",
        )
        .expect("url");

        assert!(url.starts_with("https://dypnsapi.aliyuncs.com/?"));
        assert!(url.contains("Action=SendSmsVerifyCode"));
        assert!(url.contains("AccessKeyId=testid"));
        assert!(url.contains("Signature="));
        assert!(url.contains(
            "TemplateParam=%7B%22code%22%3A%22%23%23code%23%23%22%2C%22min%22%3A%225%22%7D"
        ));
    }

    #[test]
    fn check_response_requires_verify_result_pass() {
        let pass = AliyunSmsResponse {
            code: Some("OK".to_string()),
            message: Some("成功".to_string()),
            success: Some(true),
            model: Some(AliyunSmsModel {
                verify_result: Some("PASS".to_string()),
            }),
        };
        assert_eq!(
            pass.model.and_then(|model| model.verify_result).as_deref(),
            Some("PASS")
        );
    }

    #[tokio::test]
    #[ignore = "requires Aliyun credentials and sends a real SMS"]
    async fn live_send_verify_code_smoke() {
        let phone =
            std::env::var("HONE_ALIYUN_SMS_LIVE_PHONE").expect("HONE_ALIYUN_SMS_LIVE_PHONE");
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .expect("http client");

        send_verify_code(&http, &phone).await.expect("send sms");
    }
}
