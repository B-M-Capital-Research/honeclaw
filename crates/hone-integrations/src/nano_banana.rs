//! NanoBanana 图片生成客户端
//!
//! 通过 OpenRouter chat/completions API 生成图片，
//! 支持 data:image URI 和 HTTP URL 两种图片格式的下载。

use base64::Engine;
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use hone_core::{ActorIdentity, HoneResult};

/// NanoBanana 图片生成客户端
pub struct NanoBananaClient {
    pub base_url: String,
    pub model: String,
    pub default_image_count: u32,
    pub timeout_seconds: u64,
    pub api_key: String,
    pub max_tokens: u32,
    pub output_dir: PathBuf,
    http: reqwest::Client,
}

impl NanoBananaClient {
    pub fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let download_dir = if config.nano_banana.download_dir.is_empty() {
            "gen_images"
        } else {
            &config.nano_banana.download_dir
        };
        let output_dir = PathBuf::from(&config.storage.sessions_dir)
            .parent()
            .unwrap_or(Path::new("./data"))
            .join(download_dir);

        Self {
            base_url: config
                .nano_banana
                .base_url
                .trim_end_matches('/')
                .to_string(),
            model: config.nano_banana.model.clone(),
            default_image_count: config.nano_banana.default_image_count,
            timeout_seconds: 90,
            api_key: config.llm.openrouter.api_key.trim().to_string(),
            max_tokens: 2048,
            output_dir,
            http: reqwest::Client::new(),
        }
    }

    fn get_api_key(&self) -> String {
        self.api_key.clone()
    }

    /// 从 API 响应中提取图片 URL（HTTP 或 data:image）
    fn extract_image_urls(payload: &Value) -> Vec<String> {
        let mut urls = Vec::new();
        Self::walk_for_urls(payload, &mut urls);

        // 保序去重
        let mut deduped = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for u in urls {
            if seen.insert(u.clone()) {
                deduped.push(u);
            }
        }
        deduped
    }

    fn walk_for_urls(node: &Value, urls: &mut Vec<String>) {
        match node {
            Value::Object(map) => {
                // image_url field
                if let Some(iu) = map.get("image_url") {
                    match iu {
                        Value::Object(obj) => {
                            if let Some(Value::String(u)) = obj.get("url") {
                                if u.starts_with("http") || u.starts_with("data:image/") {
                                    urls.push(u.clone());
                                }
                            }
                        }
                        Value::String(u) => {
                            if u.starts_with("http") || u.starts_with("data:image/") {
                                urls.push(u.clone());
                            }
                        }
                        _ => {}
                    }
                }
                for (key, value) in map {
                    let lk = key.to_lowercase();
                    if lk == "image_url" || lk == "image" || lk == "url" {
                        match value {
                            Value::Object(obj) => {
                                if let Some(Value::String(u)) = obj.get("url") {
                                    if u.starts_with("http") || u.starts_with("data:image/") {
                                        urls.push(u.clone());
                                    }
                                }
                            }
                            Value::String(u) => {
                                if u.starts_with("http") || u.starts_with("data:image/") {
                                    urls.push(u.clone());
                                }
                            }
                            _ => Self::walk_for_urls(value, urls),
                        }
                    } else if lk == "image_urls" || lk == "images" || lk == "urls" {
                        if let Value::Array(arr) = value {
                            for item in arr {
                                Self::walk_for_urls(item, urls);
                            }
                        }
                    } else {
                        Self::walk_for_urls(value, urls);
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::walk_for_urls(item, urls);
                }
            }
            _ => {}
        }
    }

    /// 生成图片
    pub async fn generate_images(&self, prompt: &str, image_count: Option<u32>) -> Value {
        let api_key = self.get_api_key();
        if api_key.is_empty() {
            return serde_json::json!({
                "success": false,
                "error": "未配置 OpenRouter API Key，请在 config.yaml 中设置 llm.openrouter.api_key"
            });
        }

        let url = format!("{}/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "modalities": ["image", "text"],
            "max_tokens": self.max_tokens,
            "temperature": 0.7
        });

        let resp = match self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("HTTP-Referer", "https://openrouter.ai")
            .header("X-Title", "Honeclaw")
            .json(&body)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return serde_json::json!({
                    "success": false,
                    "error": format!("OpenRouter 出图调用失败: {e}")
                });
            }
        };

        let raw: Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                return serde_json::json!({
                    "success": false,
                    "error": format!("响应解析失败: {e}")
                });
            }
        };

        let mut image_urls = Self::extract_image_urls(&raw);
        if let Some(count) = image_count {
            image_urls.truncate(count as usize);
        }

        if image_urls.is_empty() {
            return serde_json::json!({
                "success": false,
                "error": "OpenRouter 返回成功但未提取到图片 URL",
                "raw": raw
            });
        }

        serde_json::json!({
            "success": true,
            "task_id": raw.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "status": "completed",
            "image_urls": image_urls,
            "raw": raw
        })
    }

    /// 下载图片到本地
    pub async fn download_images(
        &self,
        image_urls: &[String],
        actor: &ActorIdentity,
        draft_id: &str,
    ) -> HoneResult<Vec<String>> {
        let base = self.output_dir.join(actor.storage_key());
        std::fs::create_dir_all(&base).map_err(|e| hone_core::HoneError::Storage(e.to_string()))?;

        let data_uri_re = Regex::new(r"^data:image/([a-zA-Z0-9.+-]+);base64,(.*)$").unwrap();
        let mut local_paths = Vec::new();

        for (idx, url) in image_urls.iter().enumerate() {
            let idx_1 = idx + 1;

            if url.starts_with("data:image/") {
                if let Some(caps) = data_uri_re.captures(url) {
                    let ext = caps[1].to_lowercase();
                    let suffix = if ext == "jpeg" || ext == "jpg" {
                        ".jpg".to_string()
                    } else {
                        format!(".{ext}")
                    };
                    let content = base64::engine::general_purpose::STANDARD
                        .decode(&caps[2])
                        .map_err(|e| {
                            hone_core::HoneError::Storage(format!("data URI 解码失败: {e}"))
                        })?;

                    let filename = format!(
                        "{draft_id}_{idx_1}_{}{suffix}",
                        &Uuid::new_v4().to_string()[..6]
                    );
                    let path = base.join(&filename);
                    std::fs::write(&path, content)
                        .map_err(|e| hone_core::HoneError::Storage(e.to_string()))?;
                    local_paths.push(path.to_string_lossy().to_string());
                }
                continue;
            }

            // HTTP download
            let resp = self
                .http
                .get(url)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_lowercase();

            let suffix = if content_type.contains("png") {
                ".png"
            } else if content_type.contains("webp") {
                ".webp"
            } else {
                ".jpg"
            };

            let bytes = resp
                .bytes()
                .await
                .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

            let filename = format!(
                "{draft_id}_{idx_1}_{}{suffix}",
                &Uuid::new_v4().to_string()[..6]
            );
            let path = base.join(&filename);
            std::fs::write(&path, &bytes)
                .map_err(|e| hone_core::HoneError::Storage(e.to_string()))?;
            local_paths.push(path.to_string_lossy().to_string());
        }

        Ok(local_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn make_test_client(output_dir: PathBuf, api_key: &str) -> NanoBananaClient {
        NanoBananaClient {
            base_url: "http://127.0.0.1:9".to_string(),
            model: "test-model".to_string(),
            default_image_count: 2,
            timeout_seconds: 3,
            api_key: api_key.to_string(),
            max_tokens: 64,
            output_dir,
            http: reqwest::Client::new(),
        }
    }

    #[test]
    fn extract_image_urls_handles_nested_and_dedup() {
        let payload = serde_json::json!({
            "choices": [{
                "message": {
                    "content": [
                        {"type":"output_image","image_url":{"url":"http://a/img1.jpg"}},
                        {"type":"output_image","image_url":{"url":"http://a/img1.jpg"}},
                        {"type":"output_image","image_url":{"url":"data:image/png;base64,AAA"}}
                    ]
                }
            }]
        });
        let urls = NanoBananaClient::extract_image_urls(&payload);
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().any(|u| u == "http://a/img1.jpg"));
        assert!(urls.iter().any(|u| u == "data:image/png;base64,AAA"));
    }

    #[tokio::test]
    async fn download_images_supports_data_uri() {
        let output_dir = make_temp_dir("hone_banana_download");
        let client = make_test_client(output_dir.clone(), "");
        let actor = ActorIdentity::new("imessage", "user-1", None::<String>).expect("actor");

        let data = base64::engine::general_purpose::STANDARD.encode("hello-image");
        let url = format!("data:image/png;base64,{data}");
        let paths = client
            .download_images(&[url], &actor, "draft-1")
            .await
            .expect("download data uri");

        assert_eq!(paths.len(), 1);
        let path = PathBuf::from(&paths[0]);
        assert!(path.exists());
        assert!(path.starts_with(output_dir));
        assert!(path.to_string_lossy().contains("imessage__direct__user-1"));
    }

    #[tokio::test]
    async fn generate_images_failure_path_returns_error_json() {
        let output_dir = make_temp_dir("hone_banana_no_key");
        let client = make_test_client(output_dir, "");

        let result = client.generate_images("test prompt", Some(1)).await;
        assert_eq!(result["success"], false);
        let err = result["error"].as_str().unwrap_or_default();
        assert!(!err.is_empty());
    }
}
