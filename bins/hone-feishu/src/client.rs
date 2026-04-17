use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct FeishuApiClient {
    app_id: String,
    app_secret: String,
    http: Client,
    token_cache: Arc<RwLock<Option<(String, Instant)>>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    code: i64,
    msg: String,
    tenant_access_token: Option<String>,
    expire: Option<u64>,
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    app_id: &'a str,
    app_secret: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeishuResolvedUser {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub mobile: String,
    pub open_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeishuSendResult {
    pub message_id: String,
}

impl FeishuApiClient {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            http: Client::new(),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_token(&self) -> Result<String, String> {
        {
            let cache = self.token_cache.read().await;
            if let Some((token, expires_at)) = &*cache {
                if Instant::now() < *expires_at {
                    return Ok(token.clone());
                }
            }
        }

        let resp = self
            .http
            .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal")
            .json(&TokenRequest {
                app_id: &self.app_id,
                app_secret: &self.app_secret,
            })
            .send()
            .await
            .map_err(|e| format!("Feishu token request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Feishu token auth failed: HTTP {}", resp.status()));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Feishu token json err: {e}"))?;
        if token_resp.code != 0 {
            return Err(format!(
                "Feishu token error {}: {}",
                token_resp.code, token_resp.msg
            ));
        }

        let token = token_resp.tenant_access_token.ok_or("No token returned")?;
        let expire_secs = token_resp.expire.unwrap_or(7200);

        let mut cache = self.token_cache.write().await;
        // Expire 5 minutes early
        let valid_duration = Duration::from_secs(expire_secs.max(300) - 300);
        *cache = Some((token.clone(), Instant::now() + valid_duration));

        Ok(token)
    }

    pub async fn send_message(
        &self,
        receive_id: &str,
        msg_type: &str,
        content: &str,
        uuid: Option<&str>,
    ) -> Result<FeishuSendResult, String> {
        self.send_message_with_receive_id_type("open_id", receive_id, msg_type, content, uuid)
            .await
    }

    pub async fn send_chat_message(
        &self,
        chat_id: &str,
        msg_type: &str,
        content: &str,
        uuid: Option<&str>,
    ) -> Result<FeishuSendResult, String> {
        self.send_message_with_receive_id_type("chat_id", chat_id, msg_type, content, uuid)
            .await
    }

    async fn send_message_with_receive_id_type(
        &self,
        receive_id_type: &str,
        receive_id: &str,
        msg_type: &str,
        content: &str,
        uuid: Option<&str>,
    ) -> Result<FeishuSendResult, String> {
        let token = self.get_token().await?;
        let url = format!(
            "https://open.feishu.cn/open-apis/im/v1/messages?receive_id_type={receive_id_type}"
        );

        let mut body = serde_json::json!({
            "receive_id": receive_id,
            "msg_type": msg_type,
            "content": content,
        });

        if let Some(uid) = uuid {
            body.as_object_mut().unwrap().insert(
                "uuid".to_string(),
                serde_json::Value::String(uid.to_string()),
            );
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Feishu send message request failed: {e}"))?;

        #[derive(Deserialize)]
        struct SendResp {
            code: i64,
            msg: String,
            data: Option<FeishuSendResult>,
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Feishu send message failed: HTTP {} - {}",
                status, error_body
            ));
        }

        let send_resp: SendResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu send message json err: {e}"))?;
        if send_resp.code != 0 {
            return Err(format!(
                "Feishu send message api error {}: {}",
                send_resp.code, send_resp.msg
            ));
        }

        send_resp
            .data
            .ok_or_else(|| "No data in send message response".to_string())
    }

    pub async fn reply_message(
        &self,
        message_id: &str,
        msg_type: &str,
        content: &str,
        uuid: Option<&str>,
    ) -> Result<FeishuSendResult, String> {
        let token = self.get_token().await?;
        let url = format!("https://open.feishu.cn/open-apis/im/v1/messages/{message_id}/reply");

        let mut body = serde_json::json!({
            "msg_type": msg_type,
            "content": content,
        });
        if let Some(uid) = uuid {
            body.as_object_mut().unwrap().insert(
                "uuid".to_string(),
                serde_json::Value::String(uid.to_string()),
            );
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Feishu reply message request failed: {e}"))?;

        #[derive(Deserialize)]
        struct ReplyResp {
            code: i64,
            msg: String,
            data: Option<FeishuSendResult>,
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Feishu reply message failed: HTTP {} - {}",
                status, error_body
            ));
        }

        let reply_resp: ReplyResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu reply message json err: {e}"))?;
        if reply_resp.code != 0 {
            return Err(format!(
                "Feishu reply message api error {}: {}",
                reply_resp.code, reply_resp.msg
            ));
        }

        reply_resp
            .data
            .ok_or_else(|| "No data in reply message response".to_string())
    }

    pub async fn update_message(
        &self,
        message_id: &str,
        msg_type: &str,
        content: &str,
    ) -> Result<FeishuSendResult, String> {
        let token = self.get_token().await?;
        let url = format!("https://open.feishu.cn/open-apis/im/v1/messages/{message_id}");

        let body = serde_json::json!({
            "msg_type": msg_type,
            "content": content,
        });

        let resp = self
            .http
            .patch(&url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Feishu update message request failed: {e}"))?;

        #[derive(Deserialize)]
        struct UpdateResp {
            code: i64,
            msg: String,
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Feishu update message failed: HTTP {} - {}",
                status, error_body
            ));
        }

        // Feishu update message API 的响应体结构不稳定（有时 data 缺失，有时格式略有差异）。
        // HTTP 2xx 已代表更新成功，JSON 解析失败只做 warn 不报错，直接返回 Ok。
        match resp.json::<UpdateResp>().await {
            Ok(update_resp) if update_resp.code != 0 => {
                return Err(format!(
                    "Feishu update message api error {}: {}",
                    update_resp.code, update_resp.msg
                ));
            }
            Err(e) => {
                tracing::warn!(
                    "Feishu update message: HTTP 2xx but json decode failed (ignored): {e}"
                );
            }
            Ok(_) => {}
        }

        // Sometimes update message doesn't return `data.message_id`, we can just return what we have
        Ok(FeishuSendResult {
            message_id: message_id.to_string(),
        })
    }

    pub async fn upload_image(&self, path: &str) -> Result<String, String> {
        let token = self.get_token().await?;
        let filename = Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("Invalid Feishu image path: {path}"))?;
        let bytes = fs::read(path)
            .await
            .map_err(|err| format!("Feishu image read failed for {path}: {err}"))?;
        let part = multipart::Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str(image_mime_type(path))
            .map_err(|err| format!("Feishu image mime build failed for {path}: {err}"))?;
        let form = multipart::Form::new()
            .text("image_type", "message")
            .part("image", part);

        let resp = self
            .http
            .post("https://open.feishu.cn/open-apis/im/v1/images")
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Feishu upload image request failed: {e}"))?;

        #[derive(Deserialize)]
        struct UploadImageResp {
            code: i64,
            msg: String,
            data: Option<UploadImageData>,
        }

        #[derive(Deserialize)]
        struct UploadImageData {
            image_key: Option<String>,
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let error_body = resp.text().await.unwrap_or_default();
            return Err(format!(
                "Feishu upload image failed: HTTP {} - {}",
                status, error_body
            ));
        }

        let upload_resp: UploadImageResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu upload image json err: {e}"))?;
        if upload_resp.code != 0 {
            return Err(format!(
                "Feishu upload image api error {}: {}",
                upload_resp.code, upload_resp.msg
            ));
        }

        upload_resp
            .data
            .and_then(|data| data.image_key)
            .ok_or_else(|| "No image_key in Feishu upload image response".to_string())
    }

    pub async fn resolve_email(&self, email: &str) -> Result<FeishuResolvedUser, String> {
        let token = self.get_token().await?;
        let url =
            "https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id";

        let body = serde_json::json!({
            "emails": [email]
        });

        let resp = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Feishu resolve email request failed: {e}"))?;

        #[derive(Deserialize)]
        struct BatchGetIdResp {
            code: i64,
            msg: String,
            data: Option<serde_json::Value>,
        }

        let batch_resp: BatchGetIdResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu resolve email json err: {e}"))?;
        if batch_resp.code != 0 {
            return Err(format!(
                "Feishu resolve email api error {}: {}",
                batch_resp.code, batch_resp.msg
            ));
        }

        if let Some(user_id) = first_batch_get_open_id(batch_resp.data) {
            return Ok(FeishuResolvedUser {
                email: email.to_string(),
                mobile: String::new(),
                open_id: user_id,
            });
        }

        Err(format!("No user found for email {}", email))
    }

    pub async fn resolve_mobile(&self, mobile: &str) -> Result<FeishuResolvedUser, String> {
        let token = self.get_token().await?;
        let url =
            "https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id";

        let body = serde_json::json!({
            "mobiles": [mobile]
        });

        let resp = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Feishu resolve mobile request failed: {e}"))?;

        #[derive(Deserialize)]
        struct BatchGetIdResp {
            code: i64,
            msg: String,
            data: Option<serde_json::Value>,
        }

        let batch_resp: BatchGetIdResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu resolve mobile json err: {e}"))?;
        if batch_resp.code != 0 {
            return Err(format!(
                "Feishu resolve mobile api error {}: {}",
                batch_resp.code, batch_resp.msg
            ));
        }

        if let Some(user_id) = first_batch_get_open_id(batch_resp.data) {
            return Ok(FeishuResolvedUser {
                mobile: mobile.to_string(),
                email: String::new(),
                open_id: user_id,
            });
        }

        Err(format!("No user found for mobile {}", mobile))
    }

    pub async fn download_resource(
        &self,
        message_id: &str,
        file_key: &str,
        resource_type: &str,
    ) -> Result<(Vec<u8>, Option<String>), String> {
        let token = self.get_token().await?;
        let url = format!(
            "https://open.feishu.cn/open-apis/im/v1/messages/{message_id}/resources/{file_key}?type={resource_type}"
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Feishu download resource request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Feishu download resource failed: HTTP {}",
                resp.status()
            ));
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read body: {e}"))?;
        Ok((bytes.to_vec(), content_type))
    }

    // ── CardKit API ──────────────────────────────────────────────────────────
    // 飞书 CardKit 是独立的流式卡片 API，与普通消息更新 API 完全分离：
    //   POST   /cardkit/v1/cards                          — 创建卡片实体
    //   PUT    /cardkit/v1/cards/{id}/elements/{eid}/content — 更新单个元素内容
    //   PATCH  /cardkit/v1/cards/{id}/settings            — 修改卡片配置（关闭流式）

    /// 创建 CardKit 卡片实体，返回 `card_id`。
    /// `card_json` 为完整卡片 JSON 字符串（schema 2.0）。
    /// 当前流程已改为直接发普通卡片+ticker，此方法预留供后续 CardKit 流式迭代使用。
    #[allow(dead_code)]
    pub async fn create_card(&self, card_json: &str) -> Result<String, String> {
        let token = self.get_token().await?;
        let url = "https://open.feishu.cn/open-apis/cardkit/v1/cards";

        let body = serde_json::json!({
            "type": "card_json",
            "data": card_json,
        });

        let resp = self
            .http
            .post(url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("CardKit create card request failed: {e}"))?;

        #[derive(Deserialize)]
        struct CreateResp {
            code: i64,
            msg: String,
            data: Option<CardCreateData>,
        }
        #[derive(Deserialize)]
        struct CardCreateData {
            card_id: String,
        }

        if !resp.status().is_success() {
            return Err(format!("CardKit create card HTTP {}", resp.status()));
        }

        let create_resp: CreateResp = resp
            .json()
            .await
            .map_err(|e| format!("CardKit create card json err: {e}"))?;
        if create_resp.code != 0 {
            return Err(format!(
                "CardKit create card api error {}: {}",
                create_resp.code, create_resp.msg
            ));
        }

        create_resp
            .data
            .map(|d| d.card_id)
            .ok_or_else(|| "CardKit create card: no card_id in response".to_string())
    }

    /// 更新 CardKit 卡片中指定元素（`element_id`）的 `content` 字段。
    /// `sequence` 必须严格递增；`uuid` 用于幂等去重。
    pub async fn update_card_element(
        &self,
        card_id: &str,
        element_id: &str,
        content: &str,
        sequence: u64,
        uuid: &str,
    ) -> Result<(), String> {
        let token = self.get_token().await?;
        let url = format!(
            "https://open.feishu.cn/open-apis/cardkit/v1/cards/{card_id}/elements/{element_id}/content"
        );

        let body = serde_json::json!({
            "content": content,
            "sequence": sequence,
            "uuid": uuid,
        });

        let resp = self
            .http
            .put(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("CardKit update element request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!(
                "CardKit update element HTTP {} - {}",
                status, body_text
            ));
        }

        // Best-effort JSON 解析；HTTP 2xx 本身已代表成功
        #[derive(Deserialize)]
        struct ApiResp {
            code: i64,
            msg: String,
        }
        match resp.json::<ApiResp>().await {
            Ok(r) if r.code != 0 => Err(format!(
                "CardKit update element api error {}: {}",
                r.code, r.msg
            )),
            _ => Ok(()),
        }
    }

    /// 关闭 CardKit 卡片的流式模式，并设置 summary（用于折叠预览）。
    pub async fn close_card_streaming(
        &self,
        card_id: &str,
        summary: &str,
        sequence: u64,
        uuid: &str,
    ) -> Result<(), String> {
        let token = self.get_token().await?;
        let url = format!("https://open.feishu.cn/open-apis/cardkit/v1/cards/{card_id}/settings");

        // settings 字段本身是一个 JSON 字符串
        let settings_json = serde_json::json!({
            "config": {
                "streaming_mode": false,
                "summary": { "content": summary }
            }
        })
        .to_string();

        let body = serde_json::json!({
            "settings": settings_json,
            "sequence": sequence,
            "uuid": uuid,
        });

        let resp = self
            .http
            .patch(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("CardKit close streaming request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!(
                "CardKit close streaming HTTP {} - {}",
                status, body_text
            ));
        }

        #[derive(Deserialize)]
        struct ApiResp {
            code: i64,
            msg: String,
        }
        match resp.json::<ApiResp>().await {
            Ok(r) if r.code != 0 => Err(format!(
                "CardKit close streaming api error {}: {}",
                r.code, r.msg
            )),
            _ => Ok(()),
        }
    }

    pub async fn get_user_by_open_id(&self, open_id: &str) -> Result<FeishuResolvedUser, String> {
        let token = self.get_token().await?;
        let url = format!(
            "https://open.feishu.cn/open-apis/contact/v3/users/{open_id}?user_id_type=open_id"
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Feishu get user request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Feishu get user failed: HTTP {}", resp.status()));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Feishu get user json err: {e}"))?;
        if json["code"].as_i64() != Some(0) {
            return Err(format!(
                "Feishu get user error: {}",
                json["msg"].as_str().unwrap_or("unknown")
            ));
        }

        let user = &json["data"]["user"];
        let email = user["enterprise_email"]
            .as_str()
            .or_else(|| user["email"].as_str())
            .unwrap_or("")
            .to_string();
        let mobile = user["mobile"].as_str().unwrap_or("").to_string();

        Ok(FeishuResolvedUser {
            email,
            mobile,
            open_id: open_id.to_string(),
        })
    }
}

fn image_mime_type(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        _ => "image/png",
    }
}

fn first_batch_get_open_id(data: Option<serde_json::Value>) -> Option<String> {
    data.and_then(|data| data.get("user_list").cloned())
        .and_then(|value| value.as_array().cloned())
        .and_then(|list| {
            list.into_iter().next().and_then(|entry| {
                entry
                    .get("user_id")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
            })
        })
}

#[cfg(test)]
mod tests {
    use super::first_batch_get_open_id;
    use serde_json::json;

    #[test]
    fn first_batch_get_open_id_prefers_first_match() {
        let open_id = first_batch_get_open_id(Some(json!({
            "user_list": [
                { "user_id": "ou_first" },
                { "user_id": "ou_second" }
            ]
        })));
        assert_eq!(open_id.as_deref(), Some("ou_first"));
    }

    #[test]
    fn first_batch_get_open_id_returns_none_for_missing_user_id() {
        let open_id = first_batch_get_open_id(Some(json!({
            "user_list": [
                { "name": "alice" }
            ]
        })));
        assert!(open_id.is_none());
    }
}
