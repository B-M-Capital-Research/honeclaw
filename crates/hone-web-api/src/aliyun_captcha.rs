use std::fmt;

use hmac::{Hmac, Mac};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const API_VERSION: &str = "2023-03-05";
const ACTION: &str = "VerifyIntelligentCaptcha";
const SCRIPT_URL: &str = "https://o.alicdn.com/captcha-frontend/aliyunCaptcha/AliyunCaptcha.js";
const DEFAULT_REGION: &str = "cn";
const DEFAULT_ENDPOINT_CN: &str = "captcha.cn-shanghai.aliyuncs.com";
const DEFAULT_ENDPOINT_SGP: &str = "captcha.ap-southeast-1.aliyuncs.com";

const FORM_ENCODE_SET: &AsciiSet = &CONTROLS
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
pub(crate) struct AliyunCaptchaConfig {
    access_key_id: String,
    access_key_secret: String,
    endpoint: String,
    scene_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublicCaptchaConfig {
    pub enabled: bool,
    pub region: String,
    pub prefix: String,
    pub scene_id: String,
    pub script_url: &'static str,
}

impl AliyunCaptchaConfig {
    pub(crate) fn public_config_from_env() -> PublicCaptchaConfig {
        let enabled = env_bool("HONE_ALIYUN_CAPTCHA_ENABLED").unwrap_or(true);
        let region = env_or("HONE_ALIYUN_CAPTCHA_REGION", DEFAULT_REGION);
        let prefix = env_or("HONE_ALIYUN_CAPTCHA_PREFIX", "");
        let scene_id = env_or("HONE_ALIYUN_CAPTCHA_SCENE_ID", "");
        PublicCaptchaConfig {
            enabled: enabled && !prefix.is_empty() && !scene_id.is_empty(),
            region,
            prefix,
            scene_id,
            script_url: SCRIPT_URL,
        }
    }

    pub(crate) fn from_env() -> Result<Self, AliyunCaptchaError> {
        let access_key_id = env_first(&[
            "ALIBABA_CLOUD_ACCESS_KEY_ID",
            "ALIYUN_ACCESS_KEY_ID",
            "HONE_ALIYUN_ACCESS_KEY_ID",
        ])
        .ok_or_else(|| {
            AliyunCaptchaError::config(
                "缺少阿里云验证码 AccessKeyId，请设置 ALIBABA_CLOUD_ACCESS_KEY_ID",
            )
        })?;
        let access_key_secret = env_first(&[
            "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
            "ALIYUN_ACCESS_KEY_SECRET",
            "HONE_ALIYUN_ACCESS_KEY_SECRET",
        ])
        .ok_or_else(|| {
            AliyunCaptchaError::config(
                "缺少阿里云验证码 AccessKeySecret，请设置 ALIBABA_CLOUD_ACCESS_KEY_SECRET",
            )
        })?;
        let region = env_or("HONE_ALIYUN_CAPTCHA_REGION", DEFAULT_REGION);
        let prefix = env_or("HONE_ALIYUN_CAPTCHA_PREFIX", "");
        if prefix.is_empty() {
            return Err(AliyunCaptchaError::config(
                "缺少阿里云验证码 prefix，请设置 HONE_ALIYUN_CAPTCHA_PREFIX",
            ));
        }
        let scene_id = env_or("HONE_ALIYUN_CAPTCHA_SCENE_ID", "");
        if scene_id.is_empty() {
            return Err(AliyunCaptchaError::config(
                "缺少阿里云验证码 SceneId，请设置 HONE_ALIYUN_CAPTCHA_SCENE_ID",
            ));
        }

        Ok(Self {
            access_key_id,
            access_key_secret,
            endpoint: env_or(
                "HONE_ALIYUN_CAPTCHA_ENDPOINT",
                default_endpoint_for_region(&region),
            ),
            scene_id,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AliyunCaptchaErrorKind {
    Config,
    Transport,
    Provider,
}

#[derive(Debug, Clone)]
pub(crate) struct AliyunCaptchaError {
    pub(crate) kind: AliyunCaptchaErrorKind,
    message: String,
}

impl AliyunCaptchaError {
    fn config(message: impl Into<String>) -> Self {
        Self {
            kind: AliyunCaptchaErrorKind::Config,
            message: message.into(),
        }
    }

    fn transport(message: impl Into<String>) -> Self {
        Self {
            kind: AliyunCaptchaErrorKind::Transport,
            message: message.into(),
        }
    }

    fn provider(message: impl Into<String>) -> Self {
        Self {
            kind: AliyunCaptchaErrorKind::Provider,
            message: message.into(),
        }
    }
}

impl fmt::Display for AliyunCaptchaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AliyunCaptchaError {}

#[derive(Debug, Deserialize)]
struct AliyunCaptchaResponse {
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "Success")]
    success: Option<bool>,
    #[serde(rename = "Result")]
    result: Option<AliyunCaptchaResult>,
}

#[derive(Debug, Deserialize)]
struct AliyunCaptchaResult {
    #[serde(rename = "VerifyResult")]
    verify_result: Option<bool>,
    #[serde(rename = "VerifyCode")]
    #[cfg(test)]
    verify_code: Option<String>,
}

pub(crate) async fn verify_captcha(
    http: &reqwest::Client,
    captcha_verify_param: &str,
) -> Result<bool, AliyunCaptchaError> {
    let config = AliyunCaptchaConfig::from_env()?;
    verify_captcha_with_config(http, &config, captcha_verify_param).await
}

#[cfg(test)]
async fn probe_service(http: &reqwest::Client) -> Result<String, AliyunCaptchaError> {
    let config = AliyunCaptchaConfig::from_env()?;
    let response = call_aliyun(http, &config, "probe").await?;
    Ok(response
        .result
        .and_then(|result| result.verify_code)
        .or(response.code)
        .unwrap_or_else(|| "UNKNOWN".to_string()))
}

async fn verify_captcha_with_config(
    http: &reqwest::Client,
    config: &AliyunCaptchaConfig,
    captcha_verify_param: &str,
) -> Result<bool, AliyunCaptchaError> {
    let token = captcha_verify_param.trim();
    if token.is_empty() {
        return Ok(false);
    }
    let response = call_aliyun(http, config, token).await?;
    Ok(response.success.unwrap_or(false)
        && response
            .result
            .and_then(|result| result.verify_result)
            .unwrap_or(false))
}

async fn call_aliyun(
    http: &reqwest::Client,
    config: &AliyunCaptchaConfig,
    captcha_verify_param: &str,
) -> Result<AliyunCaptchaResponse, AliyunCaptchaError> {
    let body = form_body(&[
        ("CaptchaVerifyParam", captcha_verify_param),
        ("SceneId", &config.scene_id),
    ]);
    let date = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let nonce = Uuid::new_v4().to_string();
    let endpoint = endpoint_base(&config.endpoint);
    let host = endpoint_host(&endpoint)?;
    let body_hash = hex_sha256(body.as_bytes());
    let authorization = authorization(config, &host, &body_hash, &date, &nonce)?;

    let response = http
        .post(&endpoint)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("host", &host)
        .header("x-acs-action", ACTION)
        .header("x-acs-version", API_VERSION)
        .header("x-acs-date", &date)
        .header("x-acs-signature-nonce", &nonce)
        .header("x-acs-content-sha256", &body_hash)
        .header("authorization", authorization)
        .body(body)
        .send()
        .await
        .map_err(|err| AliyunCaptchaError::transport(format!("调用阿里云验证码接口失败: {err}")))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| AliyunCaptchaError::transport(format!("读取阿里云验证码响应失败: {err}")))?;
    let parsed = serde_json::from_str::<AliyunCaptchaResponse>(&text).map_err(|err| {
        AliyunCaptchaError::provider(format!("阿里云验证码响应解析失败: {err}; status={status}"))
    })?;
    if !status.is_success() {
        return Err(provider_error("阿里云验证码接口返回错误", parsed));
    }
    if parsed.success == Some(false) {
        return Err(provider_error("阿里云验证码接口调用失败", parsed));
    }
    Ok(parsed)
}

fn authorization(
    config: &AliyunCaptchaConfig,
    host: &str,
    body_hash: &str,
    date: &str,
    nonce: &str,
) -> Result<String, AliyunCaptchaError> {
    let signed_headers =
        "host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version";
    let canonical_headers = format!(
        "host:{host}\nx-acs-action:{ACTION}\nx-acs-content-sha256:{body_hash}\nx-acs-date:{date}\nx-acs-signature-nonce:{nonce}\nx-acs-version:{API_VERSION}\n"
    );
    let canonical_request =
        format!("POST\n/\n\n{canonical_headers}\n{signed_headers}\n{body_hash}");
    let string_to_sign = format!(
        "ACS3-HMAC-SHA256\n{}",
        hex_sha256(canonical_request.as_bytes())
    );
    let mut mac = HmacSha256::new_from_slice(config.access_key_secret.as_bytes())
        .map_err(|err| AliyunCaptchaError::config(format!("阿里云验证码签名初始化失败: {err}")))?;
    mac.update(string_to_sign.as_bytes());
    let signature = hex_lower(&mac.finalize().into_bytes());
    Ok(format!(
        "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        config.access_key_id, signed_headers, signature
    ))
}

fn provider_error(prefix: &str, response: AliyunCaptchaResponse) -> AliyunCaptchaError {
    let code = response.code.unwrap_or_else(|| "UNKNOWN".to_string());
    let message = response.message.unwrap_or_else(|| "未知错误".to_string());
    AliyunCaptchaError::provider(format!("{prefix}: {code} {message}"))
}

fn form_body(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                form_percent_encode(key),
                form_percent_encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn endpoint_base(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

fn endpoint_host(endpoint: &str) -> Result<String, AliyunCaptchaError> {
    url::Url::parse(endpoint)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
        .ok_or_else(|| AliyunCaptchaError::config("阿里云验证码 endpoint 配置不正确"))
}

fn default_endpoint_for_region(region: &str) -> &'static str {
    if region.trim().eq_ignore_ascii_case("sgp") {
        DEFAULT_ENDPOINT_SGP
    } else {
        DEFAULT_ENDPOINT_CN
    }
}

fn form_percent_encode(value: &str) -> String {
    utf8_percent_encode(value, FORM_ENCODE_SET)
        .to_string()
        .replace("%20", "+")
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_lower(&hasher.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(TABLE[(byte >> 4) as usize] as char);
        out.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    out
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

fn env_bool(name: &str) -> Option<bool> {
    std::env::var(name).ok().map(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AliyunCaptchaConfig, authorization, endpoint_base, form_body, form_percent_encode,
        hex_sha256, probe_service,
    };

    fn test_config() -> AliyunCaptchaConfig {
        AliyunCaptchaConfig {
            access_key_id: "testid".to_string(),
            access_key_secret: "testsecret".to_string(),
            endpoint: "captcha.cn-shanghai.aliyuncs.com".to_string(),
            scene_id: "scene".to_string(),
        }
    }

    #[test]
    fn form_encoding_preserves_token_safely() {
        assert_eq!(form_percent_encode("AZaz09-_.~"), "AZaz09-_.~");
        assert_eq!(form_percent_encode("a b+c/="), "a+b%2Bc%2F%3D");
        assert_eq!(
            form_body(&[("CaptchaVerifyParam", "a b+c/="), ("SceneId", "scene")]),
            "CaptchaVerifyParam=a+b%2Bc%2F%3D&SceneId=scene"
        );
    }

    #[test]
    fn endpoint_defaults_to_https() {
        assert_eq!(
            endpoint_base("captcha.cn-shanghai.aliyuncs.com/"),
            "https://captcha.cn-shanghai.aliyuncs.com"
        );
        assert_eq!(
            endpoint_base("https://captcha.ap-southeast-1.aliyuncs.com/"),
            "https://captcha.ap-southeast-1.aliyuncs.com"
        );
    }

    #[test]
    fn sha256_hex_is_lowercase() {
        assert_eq!(
            hex_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn authorization_uses_acs3_hmac_sha256() {
        let auth = authorization(
            &test_config(),
            "captcha.cn-shanghai.aliyuncs.com",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "2026-05-13T00:00:00Z",
            "nonce-1",
        )
        .expect("auth");
        assert!(auth.starts_with("ACS3-HMAC-SHA256 Credential=testid,"));
        assert!(auth.contains("SignedHeaders=host;x-acs-action;x-acs-content-sha256;"));
        assert!(auth.contains("Signature="));
    }

    #[tokio::test]
    #[ignore = "requires Aliyun Captcha credentials and configured scene"]
    async fn live_probe_smoke() {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .expect("http client");

        let code = probe_service(&http).await.expect("probe captcha service");
        assert!(
            code.starts_with('F') || code == "Success",
            "unexpected probe result code: {code}"
        );
    }
}
