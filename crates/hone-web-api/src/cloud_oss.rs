use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use hmac::{Hmac, Mac};
use hone_core::config::OssConfig;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, DATE, HeaderMap, HeaderValue};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone)]
pub(crate) struct OssClient {
    access_key_id: String,
    access_key_secret: String,
    bucket: String,
    endpoint: String,
    public_upload_prefix: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OssObject {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

impl OssClient {
    pub(crate) fn from_config(config: &OssConfig) -> Option<Self> {
        if !config.is_configured() {
            return None;
        }
        Some(Self {
            access_key_id: config.resolved_access_key_id(),
            access_key_secret: config.resolved_access_key_secret(),
            bucket: config.resolved_bucket(),
            endpoint: config.resolved_endpoint(),
            public_upload_prefix: sanitize_prefix(&config.public_upload_prefix),
        })
    }

    pub(crate) fn public_upload_key(&self, user_id: &str, day: &str, stored_name: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            self.public_upload_prefix,
            sanitize_key_component(user_id),
            sanitize_key_component(day),
            sanitize_key_component(stored_name)
        )
    }

    pub(crate) fn object_uri(&self, key: &str) -> String {
        format!("oss://{}/{}", self.bucket, key.trim_start_matches('/'))
    }

    pub(crate) fn is_public_upload_uri_for_user(&self, raw: &str, user_id: &str) -> bool {
        let Some((bucket, key)) = parse_oss_uri(raw) else {
            return false;
        };
        if bucket != self.bucket {
            return false;
        }
        let allowed_prefix = format!(
            "{}/{}/",
            self.public_upload_prefix,
            sanitize_key_component(user_id)
        );
        key.starts_with(&allowed_prefix)
    }

    pub(crate) fn parse_managed_uri<'a>(&self, raw: &'a str) -> Option<&'a str> {
        let (bucket, key) = parse_oss_uri(raw)?;
        (bucket == self.bucket).then_some(key)
    }

    pub(crate) async fn put_object(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        let date = oss_date();
        let authorization = self.authorization("PUT", content_type, &date, key)?;
        let url = self.object_url(key);
        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(AUTHORIZATION, header_value(&authorization)?);
        headers.insert(CONTENT_TYPE, header_value(content_type)?);

        let response = reqwest::Client::new()
            .put(url)
            .headers(headers)
            .body(bytes)
            .send()
            .await
            .map_err(|error| format!("OSS 上传请求失败: {error}"))?;
        if response.status().is_success() {
            return Ok(());
        }
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("OSS 上传失败: {status} {body}"))
    }

    pub(crate) async fn get_object(&self, key: &str) -> Result<OssObject, String> {
        let date = oss_date();
        let authorization = self.authorization("GET", "", &date, key)?;
        let url = self.object_url(key);
        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(AUTHORIZATION, header_value(&authorization)?);

        let response = reqwest::Client::new()
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| format!("OSS 读取请求失败: {error}"))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OSS 读取失败: {status} {body}"));
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("OSS 响应读取失败: {error}"))?
            .to_vec();
        Ok(OssObject {
            bytes,
            content_type,
        })
    }

    fn authorization(
        &self,
        method: &str,
        content_type: &str,
        date: &str,
        key: &str,
    ) -> Result<String, String> {
        let canonical_resource = format!("/{}/{}", self.bucket, key.trim_start_matches('/'));
        let string_to_sign = format!("{method}\n\n{content_type}\n{date}\n{canonical_resource}");
        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes())
            .map_err(|error| format!("OSS 签名初始化失败: {error}"))?;
        mac.update(string_to_sign.as_bytes());
        let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());
        Ok(format!("OSS {}:{signature}", self.access_key_id))
    }

    fn object_url(&self, key: &str) -> String {
        let endpoint = self.endpoint.trim_end_matches('/');
        let host = if let Some(rest) = endpoint.strip_prefix("https://") {
            format!("https://{}.{}", self.bucket, rest)
        } else if let Some(rest) = endpoint.strip_prefix("http://") {
            format!("http://{}.{}", self.bucket, rest)
        } else {
            format!("https://{}.{}", self.bucket, endpoint)
        };
        format!("{host}/{}", encode_key(key))
    }
}

pub(crate) fn parse_oss_uri(raw: &str) -> Option<(&str, &str)> {
    let value = raw.trim();
    let rest = value.strip_prefix("oss://")?;
    let (bucket, key) = rest.split_once('/')?;
    if bucket.is_empty() || key.trim_matches('/').is_empty() {
        return None;
    }
    Some((bucket, key.trim_start_matches('/')))
}

fn oss_date() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn header_value(value: &str) -> Result<HeaderValue, String> {
    HeaderValue::from_str(value).map_err(|error| format!("OSS header 无效: {error}"))
}

fn encode_key(key: &str) -> String {
    key.trim_start_matches('/')
        .split('/')
        .map(|segment| utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn sanitize_prefix(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('/');
    if trimmed.is_empty() {
        "public-uploads".to_string()
    } else {
        trimmed.to_string()
    }
}

fn sanitize_key_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}
