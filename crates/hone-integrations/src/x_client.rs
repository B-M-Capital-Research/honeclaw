//! X (Twitter) API Client
//!
//! OAuth 1.0a user-context signing (HMAC-SHA1)
//! 媒体上传 (v1.1) + 推文创建 (v2)

use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::multipart;
use sha1::Sha1;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use uuid::Uuid;

use hone_core::HoneResult;

type HmacSha1 = Hmac<Sha1>;

/// OAuth1 凭证
#[derive(Debug, Clone)]
pub struct OAuth1Credentials {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

/// X Client
pub struct XClient {
    api_base_url: String,
    upload_base_url: String,
    timeout_seconds: u64,
    oauth_config: hone_core::config::XOAuth1Config,
    http: reqwest::Client,
}

impl XClient {
    pub fn new(config: &hone_core::config::HoneConfig) -> Self {
        Self {
            api_base_url: "https://api.twitter.com".to_string(),
            upload_base_url: "https://upload.twitter.com".to_string(),
            timeout_seconds: config.x.timeout_seconds,
            oauth_config: config.x.oauth1.clone(),
            http: reqwest::Client::new(),
        }
    }

    /// 从配置加载 OAuth1 凭证
    fn load_credentials(&self) -> HoneResult<OAuth1Credentials> {
        let consumer_key = self.oauth_config.consumer_key.trim().to_string();
        let consumer_secret = self.oauth_config.consumer_secret.trim().to_string();
        let access_token = self.oauth_config.access_token.trim().to_string();
        let access_token_secret = self.oauth_config.access_token_secret.trim().to_string();

        let mut missing = Vec::new();
        if consumer_key.is_empty() {
            missing.push("x.oauth1.consumer_key".to_string());
        }
        if consumer_secret.is_empty() {
            missing.push("x.oauth1.consumer_secret".to_string());
        }
        if access_token.is_empty() {
            missing.push("x.oauth1.access_token".to_string());
        }
        if access_token_secret.is_empty() {
            missing.push("x.oauth1.access_token_secret".to_string());
        }

        if !missing.is_empty() {
            return Err(hone_core::HoneError::Integration(format!(
                "X OAuth1 凭证未配置，请在 config.yaml 中设置: {}",
                missing.join(", ")
            )));
        }

        Ok(OAuth1Credentials {
            consumer_key,
            consumer_secret,
            access_token,
            access_token_secret,
        })
    }

    /// 验证凭证
    pub async fn verify_credentials(&self) -> HoneResult<serde_json::Value> {
        let url = format!("{}/1.1/account/verify_credentials.json", self.api_base_url);
        let mut query = BTreeMap::new();
        query.insert("skip_status".to_string(), "true".to_string());
        query.insert("include_email".to_string(), "false".to_string());

        let resp = self.oauth_request("GET", &url, Some(&query), None).await?;
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;
        Ok(data)
    }

    /// 上传媒体文件
    pub async fn upload_media(&self, path: &str) -> HoneResult<String> {
        if !std::path::Path::new(path).exists() {
            return Err(hone_core::HoneError::Integration(format!(
                "媒体文件不存在: {path}"
            )));
        }

        let content = tokio::fs::read(path)
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let filename = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "media".to_string());

        let url = format!("{}/1.1/media/upload.json", self.upload_base_url);

        let creds = self.load_credentials()?;
        let auth_header = build_oauth1_header(
            "POST",
            &url,
            &creds.consumer_key,
            &creds.consumer_secret,
            &creds.access_token,
            &creds.access_token_secret,
            None,
            None,
        );

        let part = multipart::Part::bytes(content)
            .file_name(filename)
            .mime_str(&mime)
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

        let form = multipart::Form::new().part("media", part);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth_header)
            .multipart(form)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(hone_core::HoneError::Integration(format!(
                "媒体上传失败: HTTP {status}: {text}"
            )));
        }

        let payload: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

        let media_id = payload
            .get("media_id_string")
            .or_else(|| payload.get("media_id"))
            .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
            .map(|s| s.to_string())
            .unwrap_or_default();

        if media_id.is_empty() {
            return Err(hone_core::HoneError::Integration(
                "媒体上传返回缺少 media_id".to_string(),
            ));
        }

        Ok(media_id)
    }

    /// 创建推文
    pub async fn create_tweet(
        &self,
        text: &str,
        in_reply_to: Option<&str>,
        media_ids: Option<&[String]>,
    ) -> HoneResult<serde_json::Value> {
        let url = format!("{}/2/tweets", self.api_base_url);

        let mut payload = serde_json::json!({"text": text});

        if let Some(reply_id) = in_reply_to {
            if !reply_id.is_empty() {
                payload["reply"] = serde_json::json!({"in_reply_to_tweet_id": reply_id});
            }
        }

        if let Some(ids) = media_ids {
            let valid_ids: Vec<&String> = ids.iter().filter(|id| !id.trim().is_empty()).collect();
            if !valid_ids.is_empty() {
                payload["media"] = serde_json::json!({"media_ids": valid_ids});
            }
        }

        let creds = self.load_credentials()?;
        let auth_header = build_oauth1_header(
            "POST",
            &url,
            &creds.consumer_key,
            &creds.consumer_secret,
            &creds.access_token,
            &creds.access_token_secret,
            None,
            None,
        );

        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(hone_core::HoneError::Integration(format!(
                "推文创建失败: HTTP {status}: {text}"
            )));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;
        Ok(data)
    }

    /// 带 OAuth1 签名的 GET/POST 请求
    async fn oauth_request(
        &self,
        method: &str,
        url: &str,
        query: Option<&BTreeMap<String, String>>,
        json_body: Option<&serde_json::Value>,
    ) -> HoneResult<reqwest::Response> {
        let creds = self.load_credentials()?;
        let auth_header = build_oauth1_header(
            method,
            url,
            &creds.consumer_key,
            &creds.consumer_secret,
            &creds.access_token,
            &creds.access_token_secret,
            query,
            None,
        );

        let mut req = match method.to_uppercase().as_str() {
            "GET" => self.http.get(url),
            "POST" => self.http.post(url),
            "PUT" => self.http.put(url),
            "DELETE" => self.http.delete(url),
            _ => self.http.get(url),
        };

        req = req.header("Authorization", auth_header);
        req = req.timeout(std::time::Duration::from_secs(self.timeout_seconds));

        if let Some(q) = query {
            let pairs: Vec<(&str, &str)> =
                q.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            req = req.query(&pairs);
        }

        if let Some(body) = json_body {
            req = req.json(body);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

        if resp.status().as_u16() >= 400 {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(hone_core::HoneError::Integration(format!(
                "X API 请求失败: HTTP {status}: {text}"
            )));
        }

        Ok(resp)
    }
}

// ── OAuth 1.0a HMAC-SHA1 签名 ──

fn pct_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}

fn normalize_base_url(raw: &str) -> String {
    if let Ok(parsed) = Url::parse(raw) {
        let scheme = parsed.scheme().to_lowercase();
        let host = parsed.host_str().unwrap_or("").to_lowercase();
        let port = parsed.port();
        let default_port = if scheme == "https" { 443 } else { 80 };
        let netloc = if let Some(p) = port {
            if p != default_port {
                format!("{host}:{p}")
            } else {
                host
            }
        } else {
            host
        };
        let path = parsed.path();
        format!("{scheme}://{netloc}{path}")
    } else {
        raw.to_string()
    }
}

fn build_oauth1_header(
    method: &str,
    url: &str,
    consumer_key: &str,
    consumer_secret: &str,
    token: &str,
    token_secret: &str,
    query_params: Option<&BTreeMap<String, String>>,
    form_params: Option<&BTreeMap<String, String>>,
) -> String {
    let nonce = Uuid::new_v4().to_string().replace('-', "");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let mut oauth_params = BTreeMap::new();
    oauth_params.insert("oauth_consumer_key".to_string(), consumer_key.to_string());
    oauth_params.insert("oauth_nonce".to_string(), nonce);
    oauth_params.insert(
        "oauth_signature_method".to_string(),
        "HMAC-SHA1".to_string(),
    );
    oauth_params.insert("oauth_timestamp".to_string(), timestamp);
    oauth_params.insert("oauth_token".to_string(), token.to_string());
    oauth_params.insert("oauth_version".to_string(), "1.0".to_string());

    // Collect all params for signature base
    let mut sig_pairs: Vec<(String, String)> = Vec::new();

    // URL query params
    if let Ok(parsed) = Url::parse(url) {
        for (k, v) in parsed.query_pairs() {
            sig_pairs.push((k.to_string(), v.to_string()));
        }
    }

    if let Some(qp) = query_params {
        for (k, v) in qp {
            sig_pairs.push((k.clone(), v.clone()));
        }
    }
    if let Some(fp) = form_params {
        for (k, v) in fp {
            sig_pairs.push((k.clone(), v.clone()));
        }
    }
    for (k, v) in &oauth_params {
        sig_pairs.push((k.clone(), v.clone()));
    }

    // Sort by key then value
    sig_pairs.sort_by(|a, b| {
        let key_cmp = pct_encode(&a.0).cmp(&pct_encode(&b.0));
        if key_cmp == std::cmp::Ordering::Equal {
            pct_encode(&a.1).cmp(&pct_encode(&b.1))
        } else {
            key_cmp
        }
    });

    let params_string: String = sig_pairs
        .iter()
        .map(|(k, v)| format!("{}={}", pct_encode(k), pct_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let base_string = format!(
        "{}&{}&{}",
        pct_encode(&method.to_uppercase()),
        pct_encode(&normalize_base_url(url)),
        pct_encode(&params_string)
    );

    let signing_key = format!(
        "{}&{}",
        pct_encode(consumer_secret),
        pct_encode(token_secret)
    );

    let mut mac =
        HmacSha1::new_from_slice(signing_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(base_string.as_bytes());
    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    oauth_params.insert("oauth_signature".to_string(), signature);

    let header_parts: Vec<String> = oauth_params
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", pct_encode(k), pct_encode(v)))
        .collect();

    format!("OAuth {}", header_parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_encode_and_normalize_url_work_as_expected() {
        assert_eq!(pct_encode("a b+c"), "a%20b%2Bc");
        assert_eq!(
            normalize_base_url("https://api.twitter.com:443/2/tweets?x=1"),
            "https://api.twitter.com/2/tweets"
        );
        assert_eq!(
            normalize_base_url("http://example.com:8080/path?q=1"),
            "http://example.com:8080/path"
        );
    }

    #[test]
    fn build_oauth1_header_contains_required_fields() {
        let mut query = BTreeMap::new();
        query.insert("a".to_string(), "1".to_string());
        let mut form = BTreeMap::new();
        form.insert("b".to_string(), "2".to_string());

        let header = build_oauth1_header(
            "POST",
            "https://api.twitter.com/2/tweets?z=9",
            "ck",
            "cs",
            "at",
            "ats",
            Some(&query),
            Some(&form),
        );

        assert!(header.starts_with("OAuth "));
        assert!(header.contains("oauth_consumer_key=\"ck\""));
        assert!(header.contains("oauth_token=\"at\""));
        assert!(header.contains("oauth_signature_method=\"HMAC-SHA1\""));
        assert!(header.contains("oauth_signature=\""));
    }

    #[tokio::test]
    async fn verify_credentials_fails_fast_when_oauth_env_missing() {
        let mut config = hone_core::config::HoneConfig::default();
        config.x.oauth1.consumer_key_env = "__HONE_TEST_X_CK__".to_string();
        config.x.oauth1.consumer_secret_env = "__HONE_TEST_X_CS__".to_string();
        config.x.oauth1.access_token_env = "__HONE_TEST_X_AT__".to_string();
        config.x.oauth1.access_token_secret_env = "__HONE_TEST_X_ATS__".to_string();

        let client = XClient::new(&config);
        let err = client
            .verify_credentials()
            .await
            .expect_err("should fail when oauth env vars missing");
        let msg = err.to_string();
        assert!(msg.contains("x.oauth1.consumer_key"));
        assert!(msg.contains("x.oauth1.consumer_secret"));
        assert!(msg.contains("x.oauth1.access_token"));
        assert!(msg.contains("x.oauth1.access_token_secret"));
    }
}
