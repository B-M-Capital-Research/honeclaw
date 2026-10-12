//! NanoBanana 图片生成客户端
//!
//! 通过 OpenRouter chat/completions 或 Atlas Cloud 异步图片 API 生成图片，
//! 支持 data:image URI 和 HTTP URL 两种图片格式的下载。

use base64::Engine;
use regex::Regex;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

use hone_core::{ActorIdentity, HoneResult};

/// NanoBanana 图片生成客户端
pub struct NanoBananaClient {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub default_image_count: u32,
    pub timeout_seconds: u64,
    pub api_key: String,
    pub max_tokens: u32,
    pub output_dir: PathBuf,
    prediction_poll_interval: Duration,
    max_prediction_polls: u32,
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

        let provider = match config.nano_banana.provider.trim() {
            "" => "openrouter".to_string(),
            value => value.to_ascii_lowercase(),
        };
        let api_key = if provider == "openrouter" {
            config
                .llm
                .openrouter_key_pool()
                .first()
                .unwrap_or_default()
                .to_string()
        } else {
            config
                .llm
                .providers
                .get(&provider)
                .and_then(|entry| entry.effective_key_pool().first().map(str::to_string))
                .unwrap_or_default()
        };

        Self {
            provider,
            base_url: config
                .nano_banana
                .base_url
                .trim_end_matches('/')
                .to_string(),
            model: config.nano_banana.model.clone(),
            default_image_count: config.nano_banana.default_image_count,
            timeout_seconds: 90,
            api_key,
            max_tokens: 2048,
            output_dir,
            prediction_poll_interval: Duration::from_secs(1),
            max_prediction_polls: 30,
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
        if !matches!(self.provider.as_str(), "openrouter" | "atlascloud") {
            return serde_json::json!({
                "success": false,
                "error": format!("nano_banana.provider 不支持: {}", self.provider)
            });
        }
        let api_key = self.get_api_key();
        if api_key.is_empty() {
            return serde_json::json!({
                "success": false,
                "error": format!(
                    "未配置 {} API Key，请在 config.yaml 中设置 llm.providers.{}.api_key/api_keys",
                    self.provider, self.provider
                )
            });
        }

        match self.provider.as_str() {
            "openrouter" => self.generate_openrouter_images(prompt, image_count).await,
            "atlascloud" => self.generate_atlascloud_images(prompt, image_count).await,
            _ => unreachable!("provider validated above"),
        }
    }

    async fn generate_openrouter_images(&self, prompt: &str, image_count: Option<u32>) -> Value {
        let api_key = self.get_api_key();

        let url = format!("{}/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "modalities": ["image", "text"],
            "max_tokens": self.max_tokens,
            "temperature": 0.7
        });

        let response = match self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("HTTP-Referer", "https://openrouter.ai")
            .header("X-Title", "Hone-Financial")
            .json(&body)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                return serde_json::json!({
                    "success": false,
                    "error": format!("OpenRouter 出图调用失败: {e}")
                });
            }
        };

        let response_json: Value = match response.json().await {
            Ok(value) => value,
            Err(e) => {
                return serde_json::json!({
                    "success": false,
                    "error": format!("响应解析失败: {e}")
                });
            }
        };

        let mut image_urls = Self::extract_image_urls(&response_json);
        if let Some(count) = image_count {
            image_urls.truncate(count as usize);
        }

        if image_urls.is_empty() {
            return serde_json::json!({
                "success": false,
                "error": "OpenRouter 返回成功但未提取到图片 URL",
                "raw": response_json
            });
        }

        serde_json::json!({
            "success": true,
            "task_id": response_json.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "status": "completed",
            "image_urls": image_urls,
            "raw": response_json
        })
    }

    async fn generate_atlascloud_images(&self, prompt: &str, image_count: Option<u32>) -> Value {
        let url = format!("{}/model/generateImage", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "enable_sync_mode": false
        });

        // Submission is intentionally attempted once. Retrying a paid generation POST could
        // create duplicate tasks and duplicate charges.
        let response = match self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.get_api_key()))
            .json(&body)
            .timeout(Duration::from_secs(self.timeout_seconds))
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return serde_json::json!({
                    "success": false,
                    "error": format!("Atlas Cloud 出图提交失败: {error}")
                });
            }
        };

        let response_json = match Self::json_response(response, "Atlas Cloud 出图提交").await {
            Ok(value) => value,
            Err(error) => return serde_json::json!({"success": false, "error": error}),
        };
        let prediction = Self::atlas_prediction(&response_json);
        let request_id = prediction
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if request_id.is_empty() {
            return serde_json::json!({
                "success": false,
                "error": "Atlas Cloud 出图提交成功但未返回 prediction id"
            });
        }

        if prediction.get("status").and_then(Value::as_str) == Some("completed") {
            return Self::atlas_success(prediction, request_id, image_count);
        }

        let prediction_url = format!("{}/model/prediction/{request_id}", self.base_url);
        let mut last_poll_error = None;
        for attempt in 0..self.max_prediction_polls {
            if attempt > 0 {
                let backoff = attempt.min(5);
                tokio::time::sleep(self.prediction_poll_interval.saturating_mul(backoff)).await;
            }

            let response = match self
                .http
                .get(&prediction_url)
                .header("Authorization", format!("Bearer {}", self.get_api_key()))
                .timeout(Duration::from_secs(self.timeout_seconds))
                .send()
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    last_poll_error = Some(error.to_string());
                    continue;
                }
            };
            let status = response.status();
            let response_json = match Self::json_response(response, "Atlas Cloud 出图查询").await
            {
                Ok(value) => value,
                Err(error) => {
                    if status.as_u16() != 429 && !status.is_server_error() {
                        return serde_json::json!({
                            "success": false,
                            "task_id": request_id,
                            "error": error
                        });
                    }
                    last_poll_error = Some(error);
                    continue;
                }
            };
            let prediction = Self::atlas_prediction(&response_json);
            match prediction
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "completed" => return Self::atlas_success(prediction, request_id, image_count),
                "failed" => {
                    let message = prediction
                        .get("error")
                        .or_else(|| prediction.get("message"))
                        .and_then(Value::as_str)
                        .unwrap_or("上游任务失败");
                    return serde_json::json!({
                        "success": false,
                        "task_id": request_id,
                        "status": "failed",
                        "error": format!("Atlas Cloud 出图失败: {message}")
                    });
                }
                _ => {}
            }
        }

        serde_json::json!({
            "success": false,
            "task_id": request_id,
            "status": "processing",
            "error": last_poll_error
                .map(|error| format!("Atlas Cloud 出图查询超时: {error}"))
                .unwrap_or_else(|| "Atlas Cloud 出图查询超时".to_string())
        })
    }

    async fn json_response(response: reqwest::Response, operation: &str) -> Result<Value, String> {
        let status = response.status();
        let value = response
            .json::<Value>()
            .await
            .map_err(|error| format!("{operation}响应解析失败: {error}"))?;
        if !status.is_success() {
            let message = value
                .pointer("/error/message")
                .or_else(|| value.pointer("/data/message"))
                .or_else(|| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("上游请求失败");
            return Err(format!("{operation}失败 (HTTP {status}): {message}"));
        }
        if value
            .get("code")
            .and_then(Value::as_i64)
            .is_some_and(|code| code != 200)
        {
            let message = value
                .pointer("/error/message")
                .or_else(|| value.pointer("/data/message"))
                .or_else(|| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("上游请求失败");
            return Err(format!("{operation}失败: {message}"));
        }
        Ok(value)
    }

    fn atlas_prediction(response: &Value) -> &Value {
        response.get("data").unwrap_or(response)
    }

    fn atlas_success(prediction: &Value, request_id: &str, image_count: Option<u32>) -> Value {
        let mut image_urls = prediction
            .get("outputs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|url| url.starts_with("http") || url.starts_with("data:image/"))
            .map(str::to_string)
            .collect::<Vec<_>>();
        if let Some(count) = image_count {
            image_urls.truncate(count as usize);
        }
        if image_urls.is_empty() {
            return serde_json::json!({
                "success": false,
                "task_id": request_id,
                "status": "completed",
                "error": "Atlas Cloud 出图已完成但未返回图片 URL"
            });
        }
        serde_json::json!({
            "success": true,
            "task_id": request_id,
            "status": "completed",
            "image_urls": image_urls,
            "raw": prediction
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
            let response = self
                .http
                .get(url)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| hone_core::HoneError::Integration(e.to_string()))?;

            let content_type = response
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

            let bytes = response
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
    use hone_core::config::HoneConfig;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;
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
            provider: "openrouter".to_string(),
            base_url: "http://127.0.0.1:9".to_string(),
            model: "test-model".to_string(),
            default_image_count: 2,
            timeout_seconds: 3,
            api_key: api_key.to_string(),
            max_tokens: 64,
            output_dir,
            prediction_poll_interval: Duration::from_millis(1),
            max_prediction_polls: 3,
            http: reqwest::Client::new(),
        }
    }

    fn spawn_sequence_server(bodies: Vec<&'static str>) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        thread::spawn(move || {
            for body in bodies {
                let (mut stream, _) = listener.accept().expect("accept");
                let request = read_http_request(&mut stream);
                captured.lock().expect("request lock").push(request);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write response");
            }
        });
        (format!("http://{addr}"), requests)
    }

    fn read_http_request(stream: &mut TcpStream) -> String {
        let mut request = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let size = stream.read(&mut buffer).expect("read request");
            if size == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..size]);
            let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
            else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= header_end + 4 + content_length {
                break;
            }
        }
        String::from_utf8_lossy(&request).to_string()
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
        assert_eq!(result["success"].as_bool(), Some(false));
        let err = result["error"].as_str().unwrap_or_default();
        assert!(!err.is_empty());
    }

    #[test]
    fn from_config_uses_selected_provider_key_pool() {
        let config: HoneConfig = serde_yaml::from_str(
            r#"
nano_banana:
  provider: atlascloud
  base_url: https://api.atlascloud.ai/api/v1
  model: google/nano-banana-2/text-to-image
llm:
  providers:
    atlascloud:
      kind: openai_compatible
      base_url: https://api.atlascloud.ai/v1
      api_key: atlas-test-key
"#,
        )
        .expect("parse config");

        let client = NanoBananaClient::from_config(&config);
        assert_eq!(client.provider, "atlascloud");
        assert_eq!(client.api_key, "atlas-test-key");
        assert_eq!(client.model, "google/nano-banana-2/text-to-image");
    }

    #[test]
    fn from_config_keeps_openrouter_as_the_legacy_default() {
        let config: HoneConfig = serde_yaml::from_str(
            r#"
nano_banana: {}
llm:
  providers:
    openrouter:
      kind: openrouter
      api_key: openrouter-test-key
"#,
        )
        .expect("parse config");

        let client = NanoBananaClient::from_config(&config);
        assert_eq!(client.provider, "openrouter");
        assert_eq!(client.api_key, "openrouter-test-key");
        assert_eq!(client.base_url, "https://openrouter.ai/api/v1");
    }

    #[tokio::test]
    async fn atlascloud_submits_once_then_polls_until_completed() {
        let (base_url, requests) = spawn_sequence_server(vec![
            r#"{"code":200,"data":{"id":"prediction-1","status":"starting"}}"#,
            r#"{"code":200,"data":{"id":"prediction-1","status":"processing","outputs":[]}}"#,
            r#"{"code":200,"data":{"id":"prediction-1","status":"completed","outputs":["https://cdn.example/image.png"]}}"#,
        ]);
        let mut client = make_test_client(make_temp_dir("hone_atlascloud_generation"), "key");
        client.provider = "atlascloud".to_string();
        client.base_url = base_url;
        client.model = "google/nano-banana-2/text-to-image".to_string();
        client.http = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("test client");

        let result = client
            .generate_images("portfolio infographic", Some(1))
            .await;

        if result["success"].as_bool() != Some(true) {
            panic!(
                "generation failed: {result}; requests={:?}",
                requests.lock().expect("request lock")
            );
        }
        assert_eq!(result["task_id"], "prediction-1");
        assert_eq!(result["image_urls"][0], "https://cdn.example/image.png");
        let requests = requests.lock().expect("request lock");
        assert_eq!(requests.len(), 3);
        assert!(requests[0].starts_with("POST /model/generateImage "));
        assert!(requests[0].contains("google/nano-banana-2/text-to-image"));
        assert!(requests[1].starts_with("GET /model/prediction/prediction-1 "));
        assert!(requests[2].starts_with("GET /model/prediction/prediction-1 "));
    }
}
