//! Hone's private trading notebook. This API stores records; it places no orders.
//! Ownership is derived exclusively from the authenticated public web session.
mod store;
mod validation;

use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use hone_core::{ActorIdentity, cloud_runtime::CloudPgRuntime};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tokio::sync::OnceCell;

use crate::state::AppState;
use store::{Store, StoreError};

const CONTEXT_HEADER: &str = "x-hone-trade-context";
static STORE: OnceCell<Store> = OnceCell::const_new();

pub(super) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/state", get(get_state).put(put_state))
        .route("/history", get(get_history))
        .route("/history/{id}", get(get_historical))
        .route("/history/{id}/restore", post(restore))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(middleware::from_fn(private_response))
}

async fn private_response(request: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response
        .headers_mut()
        .append(header::VARY, HeaderValue::from_static("Cookie"));
    response
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SaveRequest {
    state: serde_json::Value,
    revision: i64,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RestoreRequest {
    revision: i64,
    #[serde(default)]
    request_id: Option<String>,
}

struct Owner {
    key: String,
    context: String,
}

async fn owner(
    state: &AppState,
    headers: &HeaderMap,
    require_context: bool,
) -> Result<Owner, Response> {
    // Includes existing paid-product access, revoked-user and session expiry
    // checks. There is deliberately no client userId or actor extractor.
    let user = super::public::require_public_user(state, headers).await?;
    let actor = ActorIdentity::new("web", user.user_id, None::<String>)
        .map_err(|_| error(StatusCode::UNAUTHORIZED, "登录身份无效"))?;
    let key = actor.storage_key();
    let context =
        session_context(headers, &key).ok_or_else(|| error(StatusCode::UNAUTHORIZED, "未登录"))?;
    verify_context(headers, &context, require_context)?;
    Ok(Owner { key, context })
}

/// This is an anti-mixup/CSRF token, not an alternate login credential. It is
/// one-way derived from the high-entropy HttpOnly cookie plus server actor.
/// The cookie is still authenticated on every request, and never returned.
fn session_context(headers: &HeaderMap, actor: &str) -> Option<String> {
    // Keep cookie parsing identical to public::read_session_token. That helper
    // remains private to the authentication module.
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    let token = cookies.split(';').find_map(|item| {
        let (name, value) = item.trim().split_once('=')?;
        (name == "hone_web_session").then_some(value)
    })?;
    if token.is_empty() {
        return None;
    }
    let mut digest = Sha256::new();
    digest.update(b"hone-trade-manager/context/v1\0");
    digest.update(actor.as_bytes());
    digest.update(b"\0");
    digest.update(token.as_bytes());
    Some(format!("{:x}", digest.finalize()))
}

fn verify_context(headers: &HeaderMap, expected: &str, required: bool) -> Result<(), Response> {
    match headers.get(CONTEXT_HEADER) {
        None if !required => Ok(()),
        Some(value) if value.as_bytes().ct_eq(expected.as_bytes()).into() => Ok(()),
        _ => Err(error(
            StatusCode::FORBIDDEN,
            "登录会话已变化，请重新打开交易管理台；旧页面数据未写入",
        )),
    }
}

async fn storage(state: &AppState) -> Result<&'static Store, Response> {
    STORE
        .get_or_try_init(|| async {
            let postgres = CloudPgRuntime::from_cloud_config(&state.core.config.cloud)
                .ok_or_else(|| StoreError::Database("PostgreSQL 未配置".into()))?;
            let store = Store(postgres);
            // Once per process, retrying after failure. Ordinary requests never
            // issue DDL. Cross-process initialization is transaction-locked.
            store.initialize().await?;
            Ok::<_, StoreError>(store)
        })
        .await
        .map_err(store_error)
}

async fn get_state(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let owner = match owner(&state, &headers, false).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let store = match storage(&state).await {
        Ok(store) => store,
        Err(response) => return response,
    };
    match store.load(&owner.key).await {
        Ok(snapshot) => Json(
            json!({"state":snapshot.state,"revision":snapshot.revision,"context":owner.context}),
        )
        .into_response(),
        Err(e) => store_error(e),
    }
}

async fn put_state(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: Result<Json<SaveRequest>, JsonRejection>,
) -> Response {
    let owner = match owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let Json(body) = match payload {
        Ok(body) => body,
        Err(e) => return error(e.status(), "请求 JSON 无效或超过 2 MiB 限制"),
    };
    if let Err(message) = validation::validate(&body.state) {
        return error(StatusCode::UNPROCESSABLE_ENTITY, &message);
    }
    let label = body.label.as_deref().unwrap_or("保存").trim();
    if label.is_empty() || label.chars().count() > 100 {
        return error(StatusCode::BAD_REQUEST, "保存说明应为 1 至 100 个字符");
    }
    let request_id = match request_id(body.request_id) {
        Ok(id) => id,
        Err(response) => return response,
    };
    let store = match storage(&state).await {
        Ok(store) => store,
        Err(response) => return response,
    };
    match store
        .save(
            &owner.key,
            &body.state,
            body.revision,
            label,
            &request_id,
            None,
        )
        .await
    {
        Ok(snapshot) => Json(
            json!({"state":snapshot.state,"revision":snapshot.revision,"context":owner.context}),
        )
        .into_response(),
        Err(e) => store_error(e),
    }
}

async fn get_history(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let owner = match owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let store = match storage(&state).await {
        Ok(store) => store,
        Err(response) => return response,
    };
    match store.history(&owner.key).await {
        Ok(history) => Json(json!({"history":history,"context":owner.context})).into_response(),
        Err(e) => store_error(e),
    }
}

async fn get_historical(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let owner = match owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let store = match storage(&state).await {
        Ok(store) => store,
        Err(response) => return response,
    };
    match store.historical(&owner.key, &id).await {
        Ok(snapshot) => {
            let mut value = serde_json::to_value(snapshot).expect("serializable snapshot");
            value["context"] = owner.context.into();
            Json(value).into_response()
        }
        Err(e) => store_error(e),
    }
}

async fn restore(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    payload: Result<Json<RestoreRequest>, JsonRejection>,
) -> Response {
    let owner = match owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let Json(body) = match payload {
        Ok(body) => body,
        Err(e) => return error(e.status(), "恢复请求 JSON 无效"),
    };
    let request_id = match request_id(body.request_id) {
        Ok(id) => id,
        Err(response) => return response,
    };
    let store = match storage(&state).await {
        Ok(store) => store,
        Err(response) => return response,
    };
    match store
        .restore(&owner.key, &id, body.revision, &request_id)
        .await
    {
        Ok(snapshot) => Json(
            json!({"state":snapshot.state,"revision":snapshot.revision,"context":owner.context}),
        )
        .into_response(),
        Err(e) => store_error(e),
    }
}

fn request_id(value: Option<String>) -> Result<String, Response> {
    match value {
        Some(value) => uuid::Uuid::parse_str(&value)
            .map(|id| id.to_string())
            .map_err(|_| error(StatusCode::BAD_REQUEST, "requestId 必须为 UUID")),
        None => Ok(uuid::Uuid::new_v4().to_string()),
    }
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error":message}))).into_response()
}

fn store_error(error_value: StoreError) -> Response {
    match error_value {
        StoreError::Conflict => error(
            StatusCode::CONFLICT,
            "版本冲突；请重新加载后核对，当前提交未覆盖其他更改",
        ),
        StoreError::Missing => error(
            StatusCode::NOT_FOUND,
            "找不到该恢复点，可能已超出最近 8 个版本",
        ),
        StoreError::Database(message) => {
            tracing::error!(error=%message, "trade manager persistence failed");
            error(
                StatusCode::SERVICE_UNAVAILABLE,
                "交易记录服务暂时不可用；未确认保存，请保留当前页面并重试",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_is_actor_and_session_bound_and_never_the_cookie() {
        let mut a = HeaderMap::new();
        a.insert(
            header::COOKIE,
            HeaderValue::from_static("other=x; hone_web_session=session-a"),
        );
        let context_a = session_context(&a, "web__direct__a").unwrap();
        assert_eq!(context_a.len(), 64);
        assert!(!context_a.contains("session-a"));
        assert_ne!(context_a, session_context(&a, "web__direct__b").unwrap());
        assert!(verify_context(&a, &context_a, false).is_ok());
        assert_eq!(
            verify_context(&a, &context_a, true).unwrap_err().status(),
            StatusCode::FORBIDDEN
        );
        a.insert(CONTEXT_HEADER, HeaderValue::from_str(&context_a).unwrap());
        assert!(verify_context(&a, &context_a, true).is_ok());
        a.insert(
            header::COOKIE,
            HeaderValue::from_static("hone_web_session=session-b"),
        );
        let context_b = session_context(&a, "web__direct__b").unwrap();
        assert_eq!(
            verify_context(&a, &context_b, true).unwrap_err().status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            verify_context(&a, &context_b, false).unwrap_err().status(),
            StatusCode::FORBIDDEN
        );
        assert!(session_context(&HeaderMap::new(), "web__direct__a").is_none());
    }

    #[test]
    fn client_identity_is_not_part_of_the_write_contract() {
        assert!(
            serde_json::from_value::<SaveRequest>(
                json!({"state":{},"revision":0,"userId":"victim"})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<RestoreRequest>(json!({"revision":0,"actor":"victim"}))
                .is_err()
        );
        assert!(serde_json::from_value::<SaveRequest>(json!({"state":{},"revision":0.5})).is_err());
        assert!(request_id(Some("not-a-uuid".into())).is_err());
    }

    #[tokio::test]
    async fn private_middleware_marks_success_and_rejections_no_store() {
        let app = Router::new()
            .route("/success", get(|| async { Json(json!({"state":null})) }))
            .route(
                "/denied",
                get(|| async { error(StatusCode::FORBIDDEN, "会话不匹配") }),
            )
            .layer(middleware::from_fn(private_response));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap();
        for path in ["success", "denied"] {
            let response = client
                .get(format!("http://{address}/{path}"))
                .send()
                .await
                .unwrap();
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            assert_eq!(response.headers()[header::VARY], "Cookie");
        }
        server.abort();
    }
}
