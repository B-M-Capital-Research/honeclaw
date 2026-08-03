use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use tracing::warn;

use hone_memory::{
    BILLING_PROVIDER_STRIPE, BILLING_PROVIDER_WHOP, WEB_IDENTITY_DOMESTIC_INVITE,
    WEB_IDENTITY_INTERNATIONAL_EMAIL, WebInviteUser,
};

use crate::state::AppState;
use crate::types::{PublicBillingEntitlement, PublicBillingSummary};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublicBillingConfig {
    pub primary_provider: String,
    pub stripe_checkout_enabled: bool,
    pub whop_new_purchases_enabled: bool,
    pub purchases_allowed_on_this_client: bool,
    pub management_allowed_on_this_client: bool,
}

impl PublicBillingConfig {
    fn from_env(headers: &HeaderMap) -> Self {
        let primary_provider = match std::env::var("HONE_BILLING_PRIMARY_PROVIDER")
            .unwrap_or_else(|_| "whop".to_string())
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "stripe" => "stripe".to_string(),
            _ => "whop".to_string(),
        };
        let external_billing_allowed = !is_hone_ios(headers);
        Self {
            primary_provider,
            stripe_checkout_enabled: crate::routes::stripe::checkout_available(),
            whop_new_purchases_enabled: env_flag("HONE_WHOP_NEW_PURCHASES_ENABLED", true),
            purchases_allowed_on_this_client: external_billing_allowed,
            management_allowed_on_this_client: external_billing_allowed,
        }
    }
}

pub(crate) async fn handle_billing_config(headers: HeaderMap) -> Response {
    Json(PublicBillingConfig::from_env(&headers)).into_response()
}

pub(crate) async fn handle_billing_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_session_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    match public_billing_summary(&state, &user) {
        Ok(summary) => Json(json!({
            "billing": summary,
            "config": PublicBillingConfig::from_env(&headers),
        }))
        .into_response(),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取 Billing 状态失败: {error}"),
        ),
    }
}

pub(crate) async fn handle_billing_entitlements(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_session_user(&state, &headers) {
        Ok(user) => user,
        Err(response) => return response,
    };
    match public_billing_summary(&state, &user) {
        Ok(summary) => Json(json!({ "entitlements": summary.entitlements })).into_response(),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取 Billing 权益失败: {error}"),
        ),
    }
}

pub(crate) fn user_has_product_access(
    state: &AppState,
    user: &WebInviteUser,
) -> Result<bool, String> {
    let profile = state
        .web_auth
        .external_profile(&user.user_id)
        .map_err(|error| error.to_string())?;
    match profile.identity_kind.as_str() {
        WEB_IDENTITY_DOMESTIC_INVITE => Ok(true),
        WEB_IDENTITY_INTERNATIONAL_EMAIL => state
            .billing
            .user_has_paid_access(&user.user_id)
            .map_err(|error| error.to_string()),
        _ => Ok(false),
    }
}

pub(crate) fn public_billing_summary(
    state: &AppState,
    user: &WebInviteUser,
) -> Result<PublicBillingSummary, String> {
    let entitlements = state
        .billing
        .list_user_entitlements(&user.user_id)
        .map_err(|error| error.to_string())?;
    let active_count = entitlements
        .iter()
        .filter(|value| value.grants_paid_access())
        .count();
    let access_granted = user_has_product_access(state, user)?;
    Ok(PublicBillingSummary {
        access_granted,
        has_duplicate_active_subscriptions: active_count > 1,
        entitlements: entitlements
            .into_iter()
            .map(|value| PublicBillingEntitlement {
                entitlement_id: value.entitlement_id,
                provider: value.provider,
                raw_status: value.raw_status,
                access_state: value.access_state,
                current_period_start: value.current_period_start,
                current_period_end: value.current_period_end,
                cancel_at_period_end: value.cancel_at_period_end,
                manage_url: value.manage_url,
                grace_expires_at: value.grace_expires_at,
            })
            .collect(),
    })
}

pub(crate) fn stripe_activation_enabled() -> bool {
    crate::routes::stripe::checkout_available()
}

pub(crate) fn spawn_billing_recovery_worker(state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            for provider in [BILLING_PROVIDER_STRIPE, BILLING_PROVIDER_WHOP] {
                let event_ids = match state.billing.claimable_webhook_event_ids(provider, 100) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(%provider, %error, "Billing webhook recovery scan failed");
                        continue;
                    }
                };
                for event_id in event_ids {
                    match provider {
                        BILLING_PROVIDER_STRIPE => {
                            crate::routes::stripe::spawn_stripe_processing(state.clone(), event_id);
                        }
                        BILLING_PROVIDER_WHOP => {
                            crate::routes::whop::spawn_whop_processing(state.clone(), event_id);
                        }
                        _ => unreachable!("provider list is static"),
                    }
                }
            }
        }
    })
}

pub(crate) fn is_hone_ios(headers: &HeaderMap) -> bool {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("HONE-iOS"))
}

fn env_flag(key: &str, fallback: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" | "" => false,
            _ => fallback,
        },
        Err(_) => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::{PublicBillingConfig, is_hone_ios};
    use axum::http::{HeaderMap, HeaderValue, header};

    #[test]
    fn ios_user_agent_disables_external_purchase_surface() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("HONE-iOS/1.0 WKWebView"),
        );
        assert!(is_hone_ios(&headers));
        let config = PublicBillingConfig::from_env(&headers);
        assert!(!config.purchases_allowed_on_this_client);
        assert!(!config.management_allowed_on_this_client);
    }
}
