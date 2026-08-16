use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use tracing::warn;

use hone_memory::{
    BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION, BILLING_PROVIDER_STRIPE,
    WEB_IDENTITY_DOMESTIC_INVITE, WEB_IDENTITY_INTERNATIONAL_EMAIL, WebInviteUser,
};

use crate::state::AppState;
use crate::types::{PublicBillingEntitlement, PublicBillingSummary};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublicBillingConfig {
    pub stripe: PublicStripeBillingConfig,
    pub purchases_allowed_on_this_client: bool,
    pub management_allowed_on_this_client: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublicStripeBillingConfig {
    pub subscription: PublicBillingOfferConfig,
    pub fixed_term: PublicBillingOfferConfig,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublicBillingOfferConfig {
    pub enabled: bool,
    pub amount_minor: u32,
    pub currency: &'static str,
    pub term_months: u8,
    pub auto_renews: bool,
    pub advertised_payment_methods: PublicBillingPaymentMethods,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublicBillingPaymentMethods {
    pub card: bool,
    pub alipay: bool,
    pub wechat_pay: bool,
}

impl PublicBillingPaymentMethods {
    fn card_only() -> Self {
        Self::with_wallets(false, false)
    }

    fn with_wallets(alipay: bool, wechat_pay: bool) -> Self {
        Self {
            card: true,
            alipay,
            wechat_pay,
        }
    }

    fn fixed_term_from_env() -> Self {
        Self::with_wallets(
            crate::routes::stripe::env_flag("HONE_STRIPE_ADVERTISE_ALIPAY", false),
            crate::routes::stripe::env_flag("HONE_STRIPE_ADVERTISE_WECHAT_PAY", false),
        )
    }
}

impl PublicBillingConfig {
    fn from_env(headers: &HeaderMap) -> Self {
        let external_billing_allowed = !is_hone_ios(headers);
        let checkout_enabled = crate::routes::stripe::checkout_available();
        Self {
            stripe: PublicStripeBillingConfig {
                subscription: PublicBillingOfferConfig {
                    enabled: checkout_enabled,
                    amount_minor: 19_999,
                    currency: "usd",
                    term_months: 12,
                    auto_renews: true,
                    advertised_payment_methods: PublicBillingPaymentMethods::card_only(),
                },
                fixed_term: PublicBillingOfferConfig {
                    enabled: checkout_enabled,
                    amount_minor: 22_999,
                    currency: "usd",
                    term_months: 12,
                    auto_renews: false,
                    advertised_payment_methods: PublicBillingPaymentMethods::fixed_term_from_env(),
                },
            },
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
    match public_billing_summary(&state, &user).await {
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
    match public_billing_summary(&state, &user).await {
        Ok(summary) => Json(json!({ "entitlements": summary.entitlements })).into_response(),
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取 Billing 权益失败: {error}"),
        ),
    }
}

pub(crate) async fn user_has_product_access(
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
            .await
            .map_err(|error| error.to_string()),
        _ => Ok(false),
    }
}

pub(crate) async fn public_billing_summary(
    state: &AppState,
    user: &WebInviteUser,
) -> Result<PublicBillingSummary, String> {
    let entitlements = state
        .billing
        .list_user_entitlements(&user.user_id)
        .await
        .map_err(|error| error.to_string())?;
    let active_recurring_count = entitlements
        .iter()
        .filter(|value| {
            value.entitlement_kind == BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION
                && value.grants_paid_access()
        })
        .count();
    let access_granted = user_has_product_access(state, user).await?;
    Ok(PublicBillingSummary {
        access_granted,
        has_duplicate_active_subscriptions: active_recurring_count > 1,
        entitlements: entitlements
            .into_iter()
            .map(|value| {
                let grants_access = value.grants_paid_access();
                PublicBillingEntitlement {
                    entitlement_id: value.entitlement_id,
                    provider: value.provider,
                    entitlement_kind: value.entitlement_kind,
                    raw_status: value.raw_status,
                    access_state: value.access_state,
                    grants_access,
                    current_period_start: value.current_period_start,
                    current_period_end: value.current_period_end,
                    cancel_at_period_end: value.cancel_at_period_end,
                    manage_url: value.manage_url,
                    grace_expires_at: value.grace_expires_at,
                }
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
            let event_ids = match state
                .billing
                .claimable_webhook_event_ids(BILLING_PROVIDER_STRIPE, 100)
                .await
            {
                Ok(value) => value,
                Err(error) => {
                    warn!(provider = BILLING_PROVIDER_STRIPE, %error, "Billing webhook recovery scan failed");
                    continue;
                }
            };
            for event_id in event_ids {
                crate::routes::stripe::spawn_stripe_processing(state.clone(), event_id);
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

#[cfg(test)]
mod tests {
    use super::{PublicBillingConfig, PublicBillingPaymentMethods, is_hone_ios};
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

    #[test]
    fn card_only_payment_method_copy_is_the_fail_closed_default() {
        let methods = PublicBillingPaymentMethods::card_only();
        assert!(methods.card);
        assert!(!methods.alipay);
        assert!(!methods.wechat_pay);
    }

    #[test]
    fn fixed_term_payment_method_copy_advertises_only_proven_wallets() {
        let alipay_only = PublicBillingPaymentMethods::with_wallets(true, false);
        assert!(alipay_only.card);
        assert!(alipay_only.alipay);
        assert!(!alipay_only.wechat_pay);

        let both = PublicBillingPaymentMethods::with_wallets(true, true);
        assert!(both.card);
        assert!(both.alipay);
        assert!(both.wechat_pay);
    }
}
