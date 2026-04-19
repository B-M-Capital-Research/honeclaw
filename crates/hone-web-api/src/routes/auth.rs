use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use subtle::ConstantTimeEq;

use crate::routes::json_error;
use crate::state::AppState;
use crate::types::SseTicketResponse;

const SSE_TICKET_TTL_SECS: u64 = 300;

pub(crate) async fn handle_sse_ticket(State(state): State<Arc<AppState>>) -> Response {
    if state.auth.bearer_token.is_none() {
        return json_error(StatusCode::BAD_REQUEST, "当前 backend 未启用 token 鉴权");
    }

    let ticket = uuid::Uuid::new_v4().to_string();
    let expires_at = Instant::now() + Duration::from_secs(SSE_TICKET_TTL_SECS);
    state
        .auth
        .sse_tickets
        .lock()
        .unwrap()
        .insert(ticket.clone(), expires_at);

    Json(json!(SseTicketResponse {
        ticket,
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(SSE_TICKET_TTL_SECS as i64))
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339(),
    }))
    .into_response()
}

pub(crate) async fn require_api_auth(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if request_has_valid_auth(&state, &request) {
        next.run(request).await
    } else {
        json_error(StatusCode::UNAUTHORIZED, "缺少或无效的 Bearer token")
    }
}

fn has_valid_ticket(state: &AppState, ticket: &str) -> bool {
    let now = Instant::now();
    let mut tickets = state.auth.sse_tickets.lock().unwrap();
    tickets.retain(|_, expires_at| *expires_at > now);
    tickets
        .get(ticket)
        .is_some_and(|expires_at| *expires_at > now)
}

fn query_param(uri: &axum::http::Uri, name: &str) -> Option<String> {
    let query = uri.query()?;
    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        if k == name {
            return Some(v.to_string());
        }
    }
    None
}

fn request_has_valid_auth(state: &AppState, request: &Request<Body>) -> bool {
    let Some(expected) = state.auth.bearer_token.as_deref() else {
        return state.deployment_mode == "local";
    };

    if let Some(value) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = value.strip_prefix("Bearer ") {
            if token.as_bytes().ct_eq(expected.as_bytes()).into() {
                return true;
            }
        }
    }

    if let Some(ticket) = query_param(request.uri(), "sse_ticket") {
        return has_valid_ticket(state, &ticket);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::request_has_valid_auth;
    use crate::logging::LogBuffer;
    use crate::state::{AppState, AuthState, HeartbeatRegistry};
    use axum::body::Body;
    use axum::http::{Request, header};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tokio::sync::broadcast;

    fn test_state(token: Option<&str>) -> AppState {
        let (push_tx, _) = broadcast::channel(8);
        AppState {
            core: Arc::new(hone_channels::HoneBotCore::new(
                hone_core::HoneConfig::default(),
            )),
            web_auth: Arc::new(
                hone_memory::WebAuthStorage::new(
                    std::env::temp_dir()
                        .join(format!("hone_web_auth_auth_test_{}", uuid::Uuid::new_v4()))
                        .join("sessions.sqlite3"),
                )
                .expect("web auth"),
            ),
            push_tx,
            http_client: reqwest::Client::new(),
            log_buffer: LogBuffer::new(),
            deployment_mode: "remote".to_string(),
            auth: AuthState {
                bearer_token: token.map(str::to_string),
                sse_tickets: Mutex::new(HashMap::new()),
            },
            heartbeat_registry: HeartbeatRegistry::default(),
        }
    }

    #[test]
    fn bearer_token_auth_accepts_exact_match() {
        let state = test_state(Some("secret-token"));
        let request = Request::builder()
            .uri("/api/skills")
            .header(header::AUTHORIZATION, "Bearer secret-token")
            .body(Body::empty())
            .expect("request");
        assert!(request_has_valid_auth(&state, &request));
    }

    #[test]
    fn bearer_token_auth_rejects_non_matching_token() {
        let state = test_state(Some("secret-token"));
        let request = Request::builder()
            .uri("/api/skills")
            .header(header::AUTHORIZATION, "Bearer secret-token-x")
            .body(Body::empty())
            .expect("request");
        assert!(!request_has_valid_auth(&state, &request));
    }
}
