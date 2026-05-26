use reqwest::{Client, Response, StatusCode, multipart};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::sleep;

const FEISHU_REQUEST_MAX_ATTEMPTS: usize = 3;
const FEISHU_RETRY_DELAYS: [Duration; FEISHU_REQUEST_MAX_ATTEMPTS - 1] =
    [Duration::from_millis(500), Duration::from_millis(1500)];
const FEISHU_INVALID_TOKEN_REFRESH_ATTEMPTS: usize = 2;
const FEISHU_CONTACT_LOOKUP_MAX_ATTEMPTS: usize = FEISHU_REQUEST_MAX_ATTEMPTS;
const FEISHU_ERROR_BODY_MAX_CHARS: usize = 500;

#[derive(Clone)]
pub(crate) struct FeishuApiClient {
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
pub(crate) struct FeishuResolvedUser {
    #[serde(default)]
    pub(crate) email: String,
    #[serde(default)]
    pub(crate) mobile: String,
    pub(crate) open_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FeishuSendResult {
    pub(crate) message_id: String,
}

impl FeishuApiClient {
    pub(crate) fn new(app_id: String, app_secret: String) -> Self {
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

        let resp = send_feishu_request_with_retry(
            self.http
                .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal")
                .json(&TokenRequest {
                    app_id: &self.app_id,
                    app_secret: &self.app_secret,
                }),
            "Feishu token request",
        )
        .await
        .map_err(|e| format!("Feishu token request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format_feishu_http_error(
                "Feishu token auth failed",
                status,
                &body,
            ));
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

        let token = token_resp
            .tenant_access_token
            .ok_or_else(feishu_token_missing_message)?;
        let expire_secs = token_resp.expire.unwrap_or(7200);

        let mut cache = self.token_cache.write().await;
        // Expire 5 minutes early
        let valid_duration = Duration::from_secs(expire_secs.max(300) - 300);
        *cache = Some((token.clone(), Instant::now() + valid_duration));

        Ok(token)
    }

    async fn clear_token_cache(&self) {
        *self.token_cache.write().await = None;
    }

    pub(crate) async fn send_message(
        &self,
        receive_id: &str,
        msg_type: &str,
        content: &str,
        uuid: Option<&str>,
    ) -> Result<FeishuSendResult, String> {
        self.send_message_with_receive_id_type("open_id", receive_id, msg_type, content, uuid)
            .await
    }

    pub(crate) async fn send_chat_message(
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

        #[derive(Deserialize)]
        struct SendResp {
            code: i64,
            msg: String,
            data: Option<FeishuSendResult>,
        }

        for attempt in 1..=FEISHU_INVALID_TOKEN_REFRESH_ATTEMPTS {
            let token = self.get_token().await?;
            let resp = send_feishu_request_with_retry(
                self.http.post(&url).bearer_auth(&token).json(&body),
                "Feishu send message request",
            )
            .await
            .map_err(|e| format!("Feishu send message request failed: {e}"))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let error_body = resp.text().await.unwrap_or_default();
                if should_refresh_feishu_token_for_http_error(status, &error_body)
                    && should_retry_invalid_token_refresh(attempt)
                {
                    self.clear_token_cache().await;
                    continue;
                }
                return Err(format_feishu_http_error(
                    "Feishu send message failed",
                    status,
                    error_body,
                ));
            }

            let send_resp: SendResp = resp
                .json()
                .await
                .map_err(|e| format!("Feishu send message json err: {e}"))?;
            if send_resp.code != 0 {
                if is_feishu_invalid_access_token_error(send_resp.code, &send_resp.msg)
                    && should_retry_invalid_token_refresh(attempt)
                {
                    self.clear_token_cache().await;
                    continue;
                }
                return Err(format_feishu_api_error(
                    "Feishu send message",
                    send_resp.code,
                    &send_resp.msg,
                ));
            }

            return send_resp
                .data
                .ok_or_else(|| feishu_missing_response_field("Feishu send message", "data"));
        }

        Err("Feishu send message invalid token refresh exhausted".to_string())
    }

    pub(crate) async fn reply_message(
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

        let resp = send_feishu_request_with_retry(
            self.http.post(&url).bearer_auth(token).json(&body),
            "Feishu reply message request",
        )
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
            return Err(format_feishu_http_error(
                "Feishu reply message failed",
                status,
                error_body,
            ));
        }

        let reply_resp: ReplyResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu reply message json err: {e}"))?;
        if reply_resp.code != 0 {
            return Err(format_feishu_api_error(
                "Feishu reply message",
                reply_resp.code,
                &reply_resp.msg,
            ));
        }

        reply_resp
            .data
            .ok_or_else(|| feishu_missing_response_field("Feishu reply message", "data"))
    }

    pub(crate) async fn update_message(
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

        let resp = send_feishu_request_with_retry(
            self.http.patch(&url).bearer_auth(token).json(&body),
            "Feishu update message request",
        )
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
            return Err(format_feishu_http_error(
                "Feishu update message failed",
                status,
                error_body,
            ));
        }

        // Feishu update message API 的响应体结构不稳定（有时 data 缺失，有时格式略有差异）。
        // HTTP 2xx 已代表更新成功，JSON 解析失败只做 warn 不报错，直接返回 Ok。
        match resp.json::<UpdateResp>().await {
            Ok(update_resp) if update_resp.code != 0 => {
                return Err(format_feishu_api_error(
                    "Feishu update message",
                    update_resp.code,
                    &update_resp.msg,
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

    pub(crate) async fn upload_image(&self, path: &str) -> Result<String, String> {
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

        let resp = send_feishu_request_with_retry(
            self.http
                .post("https://open.feishu.cn/open-apis/im/v1/images")
                .bearer_auth(token)
                .multipart(form),
            "Feishu upload image request",
        )
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
            return Err(format_feishu_http_error(
                "Feishu upload image failed",
                status,
                error_body,
            ));
        }

        let upload_resp: UploadImageResp = resp
            .json()
            .await
            .map_err(|e| format!("Feishu upload image json err: {e}"))?;
        if upload_resp.code != 0 {
            return Err(format_feishu_api_error(
                "Feishu upload image",
                upload_resp.code,
                &upload_resp.msg,
            ));
        }

        upload_resp
            .data
            .and_then(|data| data.image_key)
            .ok_or_else(|| feishu_missing_response_field("Feishu upload image", "data.image_key"))
    }

    pub(crate) async fn resolve_email(&self, email: &str) -> Result<FeishuResolvedUser, String> {
        let url =
            "https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id";

        let body = serde_json::json!({
            "emails": [email]
        });

        #[derive(Deserialize)]
        struct BatchGetIdResp {
            code: i64,
            msg: String,
            data: Option<serde_json::Value>,
        }

        for attempt in 1..=FEISHU_INVALID_TOKEN_REFRESH_ATTEMPTS {
            let token = self.get_token().await?;
            let resp = send_feishu_request_with_retry(
                self.http.post(url).bearer_auth(&token).json(&body),
                "Feishu resolve email request",
            )
            .await
            .map_err(|e| format!("Feishu resolve email request failed: {e}"))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                if should_refresh_feishu_token_for_http_error(status, &body)
                    && should_retry_invalid_token_refresh(attempt)
                {
                    self.clear_token_cache().await;
                    continue;
                }
                return Err(format_feishu_http_error(
                    "Feishu resolve email failed",
                    status,
                    body,
                ));
            }

            let batch_resp: BatchGetIdResp = resp
                .json()
                .await
                .map_err(|e| format!("Feishu resolve email json err: {e}"))?;
            if batch_resp.code != 0 {
                if is_feishu_invalid_access_token_error(batch_resp.code, &batch_resp.msg)
                    && should_retry_invalid_token_refresh(attempt)
                {
                    self.clear_token_cache().await;
                    continue;
                }
                if should_retry_feishu_contact_lookup_api_error(batch_resp.code, &batch_resp.msg)
                    && should_retry_attempt(attempt)
                {
                    tracing::warn!(
                        "Feishu resolve email API returned retryable error code={} message={}; retrying attempt {}/{}",
                        batch_resp.code,
                        batch_resp.msg,
                        attempt + 1,
                        FEISHU_CONTACT_LOOKUP_MAX_ATTEMPTS
                    );
                    sleep(feishu_retry_delay(attempt)).await;
                    continue;
                }
                return Err(format_feishu_api_error(
                    "Feishu resolve email",
                    batch_resp.code,
                    &batch_resp.msg,
                ));
            }

            if let Some(user_id) = first_batch_get_open_id(batch_resp.data) {
                return Ok(FeishuResolvedUser {
                    email: email.to_string(),
                    mobile: String::new(),
                    open_id: user_id,
                });
            }

            return Err(feishu_contact_not_found_message("email", email));
        }

        Err("Feishu resolve email retry attempts exhausted".to_string())
    }

    pub(crate) async fn resolve_mobile(&self, mobile: &str) -> Result<FeishuResolvedUser, String> {
        let url =
            "https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id";

        let body = serde_json::json!({
            "mobiles": [mobile]
        });

        #[derive(Deserialize)]
        struct BatchGetIdResp {
            code: i64,
            msg: String,
            data: Option<serde_json::Value>,
        }

        for attempt in 1..=FEISHU_CONTACT_LOOKUP_MAX_ATTEMPTS {
            let token = self.get_token().await?;
            let resp = send_feishu_request_with_retry(
                self.http.post(url).bearer_auth(&token).json(&body),
                "Feishu resolve mobile request",
            )
            .await
            .map_err(|e| format!("Feishu resolve mobile request failed: {e}"))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                if should_refresh_feishu_token_for_http_error(status, &body)
                    && should_retry_invalid_token_refresh(attempt)
                {
                    self.clear_token_cache().await;
                    continue;
                }
                return Err(format_feishu_http_error(
                    "Feishu resolve mobile failed",
                    status,
                    body,
                ));
            }

            let batch_resp: BatchGetIdResp = resp
                .json()
                .await
                .map_err(|e| format!("Feishu resolve mobile json err: {e}"))?;
            if batch_resp.code != 0 {
                if is_feishu_invalid_access_token_error(batch_resp.code, &batch_resp.msg)
                    && should_retry_invalid_token_refresh(attempt)
                {
                    self.clear_token_cache().await;
                    continue;
                }
                if should_retry_feishu_contact_lookup_api_error(batch_resp.code, &batch_resp.msg)
                    && should_retry_attempt(attempt)
                {
                    tracing::warn!(
                        "Feishu resolve mobile API returned retryable error code={} message={}; retrying attempt {}/{}",
                        batch_resp.code,
                        batch_resp.msg,
                        attempt + 1,
                        FEISHU_CONTACT_LOOKUP_MAX_ATTEMPTS
                    );
                    sleep(feishu_retry_delay(attempt)).await;
                    continue;
                }
                return Err(format_feishu_api_error(
                    "Feishu resolve mobile",
                    batch_resp.code,
                    &batch_resp.msg,
                ));
            }

            if let Some(user_id) = first_batch_get_open_id(batch_resp.data) {
                return Ok(FeishuResolvedUser {
                    mobile: mobile.to_string(),
                    email: String::new(),
                    open_id: user_id,
                });
            }

            return Err(feishu_contact_not_found_message("mobile", mobile));
        }

        Err("Feishu resolve mobile retry attempts exhausted".to_string())
    }

    pub(crate) async fn download_resource(
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
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format_feishu_http_error(
                "Feishu download resource failed",
                status,
                body,
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
            .map_err(|e| format!("Feishu download resource body read failed: {e}"))?;
        Ok((bytes.to_vec(), content_type))
    }

    // ── CardKit API ──────────────────────────────────────────────────────────
    // 飞书 CardKit 是独立的流式卡片 API，与普通消息更新 API 完全分离：
    //   PUT    /cardkit/v1/cards/{id}/elements/{eid}/content — 更新单个元素内容
    //   PATCH  /cardkit/v1/cards/{id}/settings            — 修改卡片配置（关闭流式）

    /// 更新 CardKit 卡片中指定元素（`element_id`）的 `content` 字段。
    /// `sequence` 必须严格递增；`uuid` 用于幂等去重。
    pub(crate) async fn update_card_element(
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
            return Err(format_feishu_http_error(
                "CardKit update element failed",
                status,
                body_text,
            ));
        }

        // Best-effort JSON 解析；HTTP 2xx 本身已代表成功
        #[derive(Deserialize)]
        struct ApiResp {
            code: i64,
            msg: String,
        }
        match resp.json::<ApiResp>().await {
            Ok(r) if r.code != 0 => Err(format_feishu_api_error(
                "CardKit update element",
                r.code,
                &r.msg,
            )),
            _ => Ok(()),
        }
    }

    /// 关闭 CardKit 卡片的流式模式，并设置 summary（用于折叠预览）。
    pub(crate) async fn close_card_streaming(
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
            return Err(format_feishu_http_error(
                "CardKit close streaming failed",
                status,
                body_text,
            ));
        }

        #[derive(Deserialize)]
        struct ApiResp {
            code: i64,
            msg: String,
        }
        match resp.json::<ApiResp>().await {
            Ok(r) if r.code != 0 => Err(format_feishu_api_error(
                "CardKit close streaming",
                r.code,
                &r.msg,
            )),
            _ => Ok(()),
        }
    }

    pub(crate) async fn get_user_by_open_id(
        &self,
        open_id: &str,
    ) -> Result<FeishuResolvedUser, String> {
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
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format_feishu_http_error(
                "Feishu get user failed",
                status,
                body,
            ));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Feishu get user json err: {e}"))?;
        if json["code"].as_i64() != Some(0) {
            let code = json["code"].as_i64().unwrap_or_default();
            let msg = json["msg"].as_str().unwrap_or("unknown");
            return Err(format_feishu_api_error("Feishu get user", code, msg));
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

async fn send_feishu_request_with_retry(
    request: reqwest::RequestBuilder,
    label: &str,
) -> Result<Response, String> {
    if request.try_clone().is_none() {
        tracing::debug!("{label} is not cloneable; sending without retry");
        return request.send().await.map_err(|err| err.to_string());
    }

    for attempt in 1..=FEISHU_REQUEST_MAX_ATTEMPTS {
        let next_request = request
            .try_clone()
            .expect("cloneability checked before retry loop");

        match next_request.send().await {
            Ok(resp)
                if should_retry_feishu_status(resp.status()) && should_retry_attempt(attempt) =>
            {
                let status = resp.status();
                tracing::warn!(
                    "{} returned retryable status {}; retrying attempt {}/{}",
                    label,
                    status,
                    attempt + 1,
                    FEISHU_REQUEST_MAX_ATTEMPTS
                );
                sleep(feishu_retry_delay(attempt)).await;
            }
            Ok(resp) => return Ok(resp),
            Err(err) if should_retry_attempt(attempt) => {
                tracing::warn!(
                    "{} transport error on attempt {}/{}: {}; retrying",
                    label,
                    attempt,
                    FEISHU_REQUEST_MAX_ATTEMPTS,
                    err
                );
                sleep(feishu_retry_delay(attempt)).await;
            }
            Err(err) => return Err(err.to_string()),
        }
    }

    Err(format!(
        "{label} failed after {FEISHU_REQUEST_MAX_ATTEMPTS} attempts"
    ))
}

fn should_retry_attempt(attempt: usize) -> bool {
    attempt < FEISHU_REQUEST_MAX_ATTEMPTS
}

fn feishu_retry_delay(attempt: usize) -> Duration {
    FEISHU_RETRY_DELAYS[attempt.saturating_sub(1).min(FEISHU_RETRY_DELAYS.len() - 1)]
}

fn should_retry_feishu_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn should_retry_invalid_token_refresh(attempt: usize) -> bool {
    attempt < FEISHU_INVALID_TOKEN_REFRESH_ATTEMPTS
}

fn should_refresh_feishu_token_for_http_error(status: StatusCode, body: &str) -> bool {
    status == StatusCode::UNAUTHORIZED || contains_invalid_access_token_text(body)
}

fn is_feishu_invalid_access_token_error(code: i64, msg: &str) -> bool {
    matches!(code, 99991663 | 99991668) || contains_invalid_access_token_text(msg)
}

fn contains_invalid_access_token_text(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("invalid access token")
        || normalized.contains("access token invalid")
        || normalized.contains("tenant_access_token invalid")
}

fn should_retry_feishu_contact_lookup_api_error(code: i64, msg: &str) -> bool {
    code == 1663 || msg.trim().eq_ignore_ascii_case("internal error")
}

fn format_feishu_http_error(action: &str, status: StatusCode, body: impl AsRef<str>) -> String {
    let detail = extract_feishu_error_detail(body.as_ref());
    let mut message = if detail.is_empty() {
        format!("{action}: HTTP {status} (empty response body)")
    } else {
        format!("{action}: HTTP {status} - {detail}")
    };
    if let Some(hint) = feishu_error_hint(None, body.as_ref(), Some(status)) {
        message.push_str("; ");
        message.push_str(hint);
    }
    message
}

fn format_feishu_api_error(action: &str, code: i64, msg: &str) -> String {
    let mut message = format!("{action} api error {code}: {msg}");
    if let Some(hint) = feishu_error_hint(Some(code), msg, None) {
        message.push_str("; ");
        message.push_str(hint);
    }
    message
}

fn feishu_error_hint(
    code: Option<i64>,
    message: &str,
    status: Option<StatusCode>,
) -> Option<&'static str> {
    if status == Some(StatusCode::UNAUTHORIZED)
        || code.is_some_and(|code| is_feishu_invalid_access_token_error(code, message))
        || contains_invalid_access_token_text(message)
    {
        return Some(
            "请检查 feishu.app_id / feishu.app_secret 是否属于同一个应用，并确认 tenant_access_token 权限未失效。",
        );
    }

    let normalized = message.to_ascii_lowercase();
    if code == Some(99992361) || normalized.contains("open_id cross app") {
        return Some(
            "请用当前飞书应用重新解析 allowlist 用户 open_id，避免跨应用或跨租户的用户 ID。",
        );
    }
    if normalized.contains("permission")
        || normalized.contains("no authority")
        || normalized.contains("unauthorized scope")
    {
        return Some("请确认飞书应用已开通消息发送和通讯录读取权限，并覆盖目标用户或群组。");
    }

    None
}

fn extract_feishu_error_detail(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return sanitize_feishu_error_detail(trimmed);
    };
    let error = value.get("error").unwrap_or(&value);
    let message = error
        .get("message")
        .or_else(|| error.get("msg"))
        .or_else(|| error.get("detail"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(sanitize_feishu_error_detail)
        .unwrap_or_else(|| sanitize_feishu_error_detail(trimmed));
    let code = error.get("code").or_else(|| value.get("code"));
    match code {
        Some(serde_json::Value::String(code)) if !code.is_empty() => {
            format!("{message} (code: {code})")
        }
        Some(serde_json::Value::Number(code)) => format!("{message} (code: {code})"),
        _ => message,
    }
}

fn sanitize_feishu_error_detail(text: &str) -> String {
    truncate_feishu_error_body(&redact_common_feishu_error_secrets(text))
}

fn redact_common_feishu_error_secrets(text: &str) -> String {
    let mut output = redact_feishu_marker_value(&redact_feishu_url_userinfo(text), "Bearer ");
    output = redact_feishu_marker_value(&output, "Basic ");
    for key in SENSITIVE_FEISHU_ERROR_KEYS {
        output = redact_feishu_marker_value(&output, &format!("{key}="));
        output = redact_feishu_marker_value(&output, &format!("{key}:"));
        output = redact_feishu_json_string_field(&output, key);
    }
    for key in ["authorization", "Authorization"] {
        output = redact_feishu_json_string_field(&output, key);
    }
    output
}

const SENSITIVE_FEISHU_ERROR_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "api_key",
    "apiKey",
    "apikey",
    "app_secret",
    "appSecret",
    "client_secret",
    "clientSecret",
    "refresh_token",
    "refreshToken",
    "id_token",
    "idToken",
    "session_token",
    "sessionToken",
    "bot_token",
    "botToken",
    "FEISHU_APP_SECRET",
    "OPENROUTER_API_KEY",
    "token",
    "secret",
    "password",
    "X-API-Key",
    "x-api-key",
];

fn redact_feishu_url_userinfo(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find("://") {
        let authority_start = index + 3;
        let authority = &remaining[authority_start..];
        let authority_end = authority
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch.is_whitespace() || matches!(ch, '/' | '?' | '#' | ')')).then_some(idx)
            })
            .unwrap_or(authority.len());
        let authority_slice = &authority[..authority_end];
        if let Some(at_index) = authority_slice.rfind('@') {
            output.push_str(&remaining[..authority_start]);
            output.push_str("<redacted>@");
            remaining = &remaining[authority_start + at_index + 1..];
        } else {
            output.push_str(&remaining[..authority_start]);
            remaining = &remaining[authority_start..];
        }
    }
    output.push_str(remaining);
    output
}

fn redact_feishu_marker_value(text: &str, marker: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(marker) {
        let value_start = index + marker.len();
        output.push_str(&remaining[..value_start]);
        let leading_whitespace = remaining[value_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        output.push_str(&remaining[value_start..value_start + leading_whitespace]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start + leading_whitespace..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch.is_whitespace() || matches!(ch, ')' | ',' | '"' | '&')).then_some(idx)
            })
            .unwrap_or(remaining[value_start + leading_whitespace..].len());
        remaining = &remaining[value_start + leading_whitespace + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_feishu_json_string_field(text: &str, key: &str) -> String {
    let needle = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&needle) {
        let after_key = index + needle.len();
        let Some((value_quote_offset, _)) = remaining[after_key..]
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace() && *ch != ':')
            .filter(|(_, ch)| *ch == '"')
        else {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        };
        let value_start = after_key + value_quote_offset + 1;
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let mut escaped = false;
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| {
                if escaped {
                    escaped = false;
                    return None;
                }
                if ch == '\\' {
                    escaped = true;
                    return None;
                }
                (ch == '"').then_some(idx)
            })
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn truncate_feishu_error_body(text: &str) -> String {
    if text.chars().count() <= FEISHU_ERROR_BODY_MAX_CHARS {
        return text.to_string();
    }
    text.chars()
        .take(FEISHU_ERROR_BODY_MAX_CHARS)
        .collect::<String>()
        + "..."
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

fn feishu_token_missing_message() -> String {
    "Feishu token 响应缺少 tenant_access_token；请检查 feishu.app_id / feishu.app_secret 是否属于同一个应用，并确认应用权限已启用。"
        .to_string()
}

fn feishu_contact_not_found_message(kind: &str, value: &str) -> String {
    format!(
        "Feishu contact lookup 未找到用户（{kind}={value}）；请检查 allowlist 配置值是否准确，以及应用通讯录权限是否覆盖该用户。"
    )
}

fn feishu_missing_response_field(action: &str, field: &str) -> String {
    format!("{action} response missing `{field}`; 请检查飞书 API 返回结构或应用权限。")
}

#[cfg(test)]
mod tests {
    use super::{
        FEISHU_ERROR_BODY_MAX_CHARS, feishu_contact_not_found_message,
        feishu_missing_response_field, feishu_retry_delay, feishu_token_missing_message,
        first_batch_get_open_id, format_feishu_api_error, format_feishu_http_error,
        is_feishu_invalid_access_token_error, should_refresh_feishu_token_for_http_error,
        should_retry_feishu_contact_lookup_api_error, should_retry_feishu_status,
        should_retry_invalid_token_refresh,
    };
    use reqwest::StatusCode;
    use serde_json::json;
    use std::time::Duration;

    fn assert_text_contains_none(text: &str, needles: &[&str]) {
        for needle in needles {
            assert!(
                !text.contains(needle),
                "expected text not to contain `{needle}`: {text}"
            );
        }
    }

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

    #[test]
    fn feishu_token_and_contact_errors_are_actionable() {
        let token_message = feishu_token_missing_message();
        assert!(token_message.contains("tenant_access_token"));
        assert!(token_message.contains("feishu.app_id"));
        assert!(token_message.contains("权限"));

        let contact_message = feishu_contact_not_found_message("email", "alice@example.com");
        assert!(contact_message.contains("email=alice@example.com"));
        assert!(contact_message.contains("allowlist"));
        assert!(contact_message.contains("通讯录权限"));

        let missing_field = feishu_missing_response_field("Feishu upload image", "data.image_key");
        assert!(missing_field.contains("Feishu upload image"));
        assert!(missing_field.contains("data.image_key"));
        assert!(missing_field.contains("返回结构"));
    }

    #[test]
    fn retry_status_only_matches_transient_feishu_failures() {
        assert!(should_retry_feishu_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(should_retry_feishu_status(
            StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(should_retry_feishu_status(StatusCode::BAD_GATEWAY));
        assert!(!should_retry_feishu_status(StatusCode::BAD_REQUEST));
        assert!(!should_retry_feishu_status(StatusCode::UNAUTHORIZED));
        assert!(!should_retry_feishu_status(StatusCode::OK));
    }

    #[test]
    fn retry_delay_is_bounded_for_all_attempt_values() {
        assert_eq!(feishu_retry_delay(1), Duration::from_millis(500));
        assert_eq!(feishu_retry_delay(2), Duration::from_millis(1500));
        assert_eq!(feishu_retry_delay(99), Duration::from_millis(1500));
    }

    #[test]
    fn contact_lookup_json_request_is_cloneable_for_retry() {
        let client = reqwest::Client::new();
        let request = client
            .post("https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id")
            .bearer_auth("token")
            .json(&json!({ "mobiles": ["+8613800138000"] }));

        assert!(request.try_clone().is_some());
    }

    #[test]
    fn invalid_access_token_errors_trigger_one_cache_refresh() {
        assert!(is_feishu_invalid_access_token_error(
            99991663,
            "Invalid access token"
        ));
        assert!(is_feishu_invalid_access_token_error(
            0,
            "tenant_access_token invalid"
        ));
        assert!(should_refresh_feishu_token_for_http_error(
            StatusCode::UNAUTHORIZED,
            ""
        ));
        assert!(should_refresh_feishu_token_for_http_error(
            StatusCode::BAD_REQUEST,
            r#"{"msg":"Invalid access token"}"#
        ));
        assert!(should_retry_invalid_token_refresh(1));
        assert!(!should_retry_invalid_token_refresh(2));
        assert!(!is_feishu_invalid_access_token_error(
            99992361,
            "open_id cross app"
        ));
    }

    #[test]
    fn contact_lookup_internal_errors_are_retryable() {
        assert!(should_retry_feishu_contact_lookup_api_error(
            1663,
            "internal error"
        ));
        assert!(should_retry_feishu_contact_lookup_api_error(
            0,
            "Internal Error"
        ));
        assert!(!should_retry_feishu_contact_lookup_api_error(
            40001,
            "invalid parameter"
        ));
        assert!(!should_retry_feishu_contact_lookup_api_error(
            99992361,
            "open_id cross app"
        ));
    }

    #[test]
    fn contact_lookup_retry_budget_matches_request_retry_budget() {
        assert_eq!(
            super::FEISHU_CONTACT_LOOKUP_MAX_ATTEMPTS,
            super::FEISHU_REQUEST_MAX_ATTEMPTS
        );
    }

    #[test]
    fn feishu_api_errors_add_actionable_hints() {
        let invalid_token =
            format_feishu_api_error("Feishu send message", 99991663, "Invalid access token");
        assert!(invalid_token.contains("tenant_access_token"));
        assert!(invalid_token.contains("feishu.app_id"));

        let cross_app = format_feishu_api_error("Feishu get user", 99992361, "open_id cross app");
        assert!(cross_app.contains("allowlist"));
        assert!(cross_app.contains("跨应用"));

        let permission = format_feishu_api_error("Feishu resolve email", 40001, "no permission");
        assert!(permission.contains("消息发送"));
        assert!(permission.contains("通讯录读取"));
    }

    #[test]
    fn feishu_http_error_extracts_message_and_code() {
        let message = format_feishu_http_error(
            "Feishu send message failed",
            StatusCode::BAD_REQUEST,
            r#"{"code":99991663,"msg":"Invalid access token","debug":"ignored"}"#,
        );
        assert!(message.starts_with(
            "Feishu send message failed: HTTP 400 Bad Request - Invalid access token (code: 99991663)"
        ));
        assert!(message.contains("tenant_access_token"));
    }

    #[test]
    fn feishu_http_error_marks_empty_body() {
        let message =
            format_feishu_http_error("CardKit create card failed", StatusCode::BAD_GATEWAY, " ");
        assert_eq!(
            message,
            "CardKit create card failed: HTTP 502 Bad Gateway (empty response body)"
        );
    }

    #[test]
    fn feishu_http_error_truncates_unstructured_body() {
        let body = "x".repeat(FEISHU_ERROR_BODY_MAX_CHARS + 10);
        let message =
            format_feishu_http_error("Feishu upload image failed", StatusCode::BAD_REQUEST, body);
        assert_eq!(
            message,
            format!(
                "Feishu upload image failed: HTTP 400 Bad Request - {}...",
                "x".repeat(FEISHU_ERROR_BODY_MAX_CHARS)
            )
        );
    }

    #[test]
    fn feishu_http_error_redacts_secret_detail() {
        let message = format_feishu_http_error(
            "Feishu send message failed",
            StatusCode::BAD_REQUEST,
            r#"{"msg":"callback failed https://user:pass@example.test/a?token=query-secret Authorization: Bearer bearer-secret app_secret: app-secret","debug":{"client_secret":"json-client"}}"#,
        );

        assert!(message.contains("token=<redacted>"));
        assert!(message.contains("Bearer <redacted>"));
        assert!(message.contains("app_secret: <redacted>"));
        assert_text_contains_none(
            &message,
            &[
                "user:pass",
                "query-secret",
                "bearer-secret",
                "app-secret",
                "json-client",
            ],
        );
    }
}
