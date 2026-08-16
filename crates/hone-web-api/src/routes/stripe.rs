use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{Months, SecondsFormat, TimeZone, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tracing::{info, warn};
use url::Url;

use hone_memory::{
    BILLING_ACCESS_ACTIVE, BILLING_ACCESS_GRACE, BILLING_ACCESS_INACTIVE, BILLING_ACCESS_PENDING,
    BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE, BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION,
    BILLING_EVENT_RECEIVED, BILLING_PROVIDER_STRIPE, BillingEntitlement,
    BillingEntitlementUpsertOutcome, BillingStorage, BillingWebhookEvent,
    WEB_IDENTITY_INTERNATIONAL_EMAIL,
};

use crate::state::AppState;

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";
const WEBHOOK_MAX_BODY_BYTES: usize = 1024 * 1024;
const WEBHOOK_TIMESTAMP_TOLERANCE_SECS: i64 = 5 * 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StripeMode {
    Test,
    Live,
}

impl StripeMode {
    fn from_env() -> Result<Self, String> {
        match env_or("HONE_STRIPE_MODE", "test")
            .to_ascii_lowercase()
            .as_str()
        {
            "test" => Ok(Self::Test),
            "live" => Ok(Self::Live),
            _ => Err("HONE_STRIPE_MODE 必须为 test 或 live".to_string()),
        }
    }

    fn expects_livemode(self) -> bool {
        matches!(self, Self::Live)
    }

    fn secret_key_prefixes(self) -> [&'static str; 2] {
        match self {
            Self::Test => ["sk_test_", "rk_test_"],
            Self::Live => ["sk_live_", "rk_live_"],
        }
    }

    fn accepts_secret_key(self, value: &str) -> bool {
        self.secret_key_prefixes()
            .iter()
            .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
    }
}

#[derive(Debug, Clone)]
struct StripeCatalogConfig {
    mode: StripeMode,
    product_id: String,
    subscription_price_id: String,
    fixed_term_price_id: String,
}

impl StripeCatalogConfig {
    fn from_env() -> Result<Self, String> {
        let mode = StripeMode::from_env()?;
        let product_id = required_env("HONE_STRIPE_PRODUCT_ID")?;
        let subscription_price_id = required_env("HONE_STRIPE_SUBSCRIPTION_PRICE_ID")?;
        let fixed_term_price_id = required_env("HONE_STRIPE_FIXED_TERM_PRICE_ID")?;
        validate_stripe_id(&product_id, "prod_", "HONE_STRIPE_PRODUCT_ID")?;
        validate_stripe_id(
            &subscription_price_id,
            "price_",
            "HONE_STRIPE_SUBSCRIPTION_PRICE_ID",
        )?;
        validate_stripe_id(
            &fixed_term_price_id,
            "price_",
            "HONE_STRIPE_FIXED_TERM_PRICE_ID",
        )?;
        if subscription_price_id == fixed_term_price_id {
            return Err("Stripe 订阅与单次年费必须使用不同 Price".to_string());
        }
        Ok(Self {
            mode,
            product_id,
            subscription_price_id,
            fixed_term_price_id,
        })
    }

    fn price_id(&self, offer: StripeOffer) -> &str {
        match offer {
            StripeOffer::Subscription => &self.subscription_price_id,
            StripeOffer::FixedTerm => &self.fixed_term_price_id,
        }
    }

    fn price_for_kind(&self, entitlement_kind: &str) -> Option<&str> {
        match entitlement_kind {
            BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION => Some(&self.subscription_price_id),
            BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE => Some(&self.fixed_term_price_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum StripeOffer {
    Subscription,
    FixedTerm,
}

impl StripeOffer {
    fn entitlement_kind(self) -> &'static str {
        match self {
            Self::Subscription => BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION,
            Self::FixedTerm => BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE,
        }
    }

    fn checkout_mode(self) -> &'static str {
        match self {
            Self::Subscription => "subscription",
            Self::FixedTerm => "payment",
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct StripeCheckoutRequest {
    offer: StripeOffer,
}

#[derive(Debug, Clone)]
struct StripeWebhookConfig {
    catalog: StripeCatalogConfig,
    webhook_secret: String,
}

impl StripeWebhookConfig {
    fn from_env() -> Result<Self, String> {
        let catalog = StripeCatalogConfig::from_env()?;
        let webhook_secret = required_env("HONE_STRIPE_WEBHOOK_SECRET")?;
        if !webhook_secret.starts_with("whsec_") || webhook_secret.len() <= "whsec_".len() {
            return Err("HONE_STRIPE_WEBHOOK_SECRET 格式不合法".to_string());
        }
        Ok(Self {
            catalog,
            webhook_secret,
        })
    }
}

#[derive(Debug, Clone)]
struct StripeApiConfig {
    catalog: StripeCatalogConfig,
    secret_key: String,
    public_base_url: Url,
}

impl StripeApiConfig {
    fn from_env() -> Result<Self, String> {
        let catalog = StripeCatalogConfig::from_env()?;
        let secret_key = required_env("HONE_STRIPE_SECRET_KEY")?;
        if !catalog.mode.accepts_secret_key(&secret_key) {
            let [standard_prefix, restricted_prefix] = catalog.mode.secret_key_prefixes();
            return Err(format!(
                "HONE_STRIPE_SECRET_KEY 与 HONE_STRIPE_MODE 不匹配，必须使用 {standard_prefix} 或 {restricted_prefix} 开头的密钥"
            ));
        }
        let public_base_url = Url::parse(&env_or(
            "HONE_STRIPE_PUBLIC_BASE_URL",
            "https://hone-claw.com/",
        ))
        .map_err(|_| "HONE_STRIPE_PUBLIC_BASE_URL 不合法".to_string())?;
        validate_public_base_url(&public_base_url)?;
        Ok(Self {
            catalog,
            secret_key,
            public_base_url,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StripeEntitlementEvent {
    user_id: String,
    entitlement_kind: String,
    provider_reference_id: String,
    customer_id: Option<String>,
    email_address: Option<String>,
    product_id: String,
    price_id: String,
    raw_status: String,
    access_signal: StripeAccessSignal,
    current_period_start: Option<String>,
    current_period_end: Option<String>,
    cancel_at_period_end: bool,
    event_id: String,
    event_type: String,
    event_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum StripeAccessSignal {
    Pending,
    Paid,
    Status,
    PaymentFailed,
    Inactive,
}

#[derive(Debug)]
enum StripeNormalization {
    Relevant(StripeEntitlementEvent),
    Ignored(&'static str),
}

pub(crate) fn checkout_available() -> bool {
    env_flag("HONE_STRIPE_CHECKOUT_ENABLED", false) && StripeApiConfig::from_env().is_ok()
}

pub(crate) async fn handle_stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let config = match StripeWebhookConfig::from_env() {
        Ok(config) => config,
        Err(error) => return crate::routes::json_error(StatusCode::SERVICE_UNAVAILABLE, error),
    };
    if body.len() > WEBHOOK_MAX_BODY_BYTES {
        return crate::routes::json_error(StatusCode::PAYLOAD_TOO_LARGE, "Stripe webhook 过大");
    }
    if let Err(error) = verify_stripe_signature(&headers, &body, &config.webhook_secret) {
        return crate::routes::json_error(StatusCode::UNAUTHORIZED, error);
    }
    let envelope: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            return crate::routes::json_error(
                StatusCode::BAD_REQUEST,
                "Stripe webhook JSON 格式不合法",
            );
        }
    };
    let event_id = match json_string(&envelope, &["id"]) {
        Some(value) if valid_stripe_id(&value, "evt_") => value,
        _ => {
            return crate::routes::json_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Stripe event ID 不合法",
            );
        }
    };
    let event_type = json_string(&envelope, &["type"]).unwrap_or_default();
    if !is_supported_event_type(&event_type) {
        return Json(json!({ "ok": true, "ignored": true })).into_response();
    }
    let Some(livemode) = json_bool(&envelope, &["livemode"]) else {
        return crate::routes::json_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Stripe event livemode 缺失或格式不合法",
        );
    };
    if livemode != config.catalog.mode.expects_livemode() {
        return crate::routes::json_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Stripe event livemode 与服务端模式不匹配",
        );
    }
    let normalized = match normalize_stripe_event(&envelope, &config.catalog) {
        Ok(StripeNormalization::Relevant(value)) => value,
        Ok(StripeNormalization::Ignored(reason)) => {
            return Json(json!({ "ok": true, "ignored": true, "reason": reason })).into_response();
        }
        Err(error) => {
            return crate::routes::json_error(StatusCode::UNPROCESSABLE_ENTITY, error);
        }
    };
    let webhook = BillingWebhookEvent {
        provider: BILLING_PROVIDER_STRIPE.to_string(),
        event_id: event_id.clone(),
        event_type,
        object_id: Some(normalized.provider_reference_id.clone()),
        payload_sha256: sha256_hex(&body),
        provider_created_at: normalized.event_at.clone(),
        processing_state: BILLING_EVENT_RECEIVED.to_string(),
        attempt_count: 0,
        last_error: None,
        received_at: hone_core::local_now_rfc3339(),
        processing_started_at: None,
        processed_at: None,
        normalized_payload: match serde_json::to_value(&normalized) {
            Ok(value) => value,
            Err(error) => {
                return crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Stripe 标准事件序列化失败: {error}"),
                );
            }
        },
    };
    if let Err(error) = state.billing.record_webhook_event(webhook).await {
        return crate::routes::json_error(
            StatusCode::CONFLICT,
            format!("Stripe webhook 收件失败: {error}"),
        );
    }
    spawn_stripe_processing(state, event_id);
    (
        StatusCode::ACCEPTED,
        Json(json!({ "ok": true, "queued": true })),
    )
        .into_response()
}

pub(crate) async fn handle_create_checkout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<StripeCheckoutRequest>,
) -> Response {
    if !env_flag("HONE_STRIPE_CHECKOUT_ENABLED", false) {
        return crate::routes::json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Stripe Checkout 未开放",
        );
    }
    let config = match StripeApiConfig::from_env() {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::SERVICE_UNAVAILABLE, error),
    };
    if let Err(response) = require_browser_mutation(&headers, &config.public_base_url) {
        return response;
    }
    let user = match crate::routes::public::require_public_session_user(&state, &headers) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let profile = match verified_checkout_profile(&state, &user.user_id) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let entitlements = match state.billing.list_user_entitlements(&user.user_id).await {
        Ok(value) => value,
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取现有权益失败: {error}"),
            );
        }
    };
    if entitlements
        .iter()
        .any(BillingEntitlement::grants_paid_access)
    {
        return crate::routes::json_error(
            StatusCode::CONFLICT,
            "账号已有有效权益，请先在账户页查看到期或续费状态",
        );
    }
    let customer_ids = entitlements
        .iter()
        .filter(|value| value.provider == BILLING_PROVIDER_STRIPE)
        .filter(|value| value.entitlement_kind == BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION)
        .filter_map(|value| value.provider_customer_id.clone())
        .collect::<BTreeSet<_>>();
    if customer_ids.len() > 1 {
        return crate::routes::json_error(
            StatusCode::CONFLICT,
            "账号关联了多个 Stripe Customer，请联系支持后再购买",
        );
    }
    let Some(email_address) = profile.email_address else {
        return crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "请先完成邮箱验证，再创建 Stripe Checkout",
        );
    };
    let offer = request.offer;
    let price_id = config.catalog.price_id(offer).to_string();
    let success_url = format!(
        "{}me?checkout=processing&offer={}&session_id={{CHECKOUT_SESSION_ID}}",
        normalized_base_url(&config.public_base_url),
        match offer {
            StripeOffer::Subscription => "subscription",
            StripeOffer::FixedTerm => "fixed_term",
        },
    );
    let cancel_url = format!(
        "{}activate?checkout=canceled",
        normalized_base_url(&config.public_base_url)
    );
    let mut form = vec![
        ("mode".to_string(), offer.checkout_mode().to_string()),
        ("line_items[0][price]".to_string(), price_id.clone()),
        ("line_items[0][quantity]".to_string(), "1".to_string()),
        ("success_url".to_string(), success_url),
        ("cancel_url".to_string(), cancel_url),
        ("client_reference_id".to_string(), user.user_id.clone()),
        ("metadata[hone_user_id]".to_string(), user.user_id.clone()),
        (
            "metadata[hone_product_id]".to_string(),
            config.catalog.product_id.clone(),
        ),
        ("metadata[hone_price_id]".to_string(), price_id.clone()),
        (
            "metadata[hone_entitlement_kind]".to_string(),
            offer.entitlement_kind().to_string(),
        ),
        ("billing_address_collection".to_string(), "auto".to_string()),
        ("automatic_tax[enabled]".to_string(), "false".to_string()),
    ];
    match offer {
        StripeOffer::Subscription => {
            form.extend([
                (
                    "subscription_data[metadata][hone_user_id]".to_string(),
                    user.user_id.clone(),
                ),
                (
                    "subscription_data[metadata][hone_product_id]".to_string(),
                    config.catalog.product_id.clone(),
                ),
                (
                    "subscription_data[metadata][hone_price_id]".to_string(),
                    price_id.clone(),
                ),
                (
                    "subscription_data[metadata][hone_entitlement_kind]".to_string(),
                    offer.entitlement_kind().to_string(),
                ),
            ]);
        }
        StripeOffer::FixedTerm => {
            form.extend([
                ("metadata[hone_term_months]".to_string(), "12".to_string()),
                (
                    "payment_intent_data[metadata][hone_user_id]".to_string(),
                    user.user_id.clone(),
                ),
                (
                    "payment_intent_data[metadata][hone_product_id]".to_string(),
                    config.catalog.product_id.clone(),
                ),
                (
                    "payment_intent_data[metadata][hone_price_id]".to_string(),
                    price_id.clone(),
                ),
                (
                    "payment_intent_data[metadata][hone_entitlement_kind]".to_string(),
                    offer.entitlement_kind().to_string(),
                ),
                (
                    "payment_intent_data[metadata][hone_term_months]".to_string(),
                    "12".to_string(),
                ),
            ]);
        }
    }
    if let Some(customer_id) = customer_ids.into_iter().next() {
        form.push(("customer".to_string(), customer_id));
    } else {
        if offer == StripeOffer::FixedTerm {
            form.push(("customer_creation".to_string(), "always".to_string()));
        }
        form.push(("customer_email".to_string(), email_address));
    }
    let checkout_idempotency_key = checkout_idempotency_key(
        &user.user_id,
        &price_id,
        &entitlements,
        &Utc::now().format("%Y%m%d").to_string(),
    );
    let response = match stripe_form_post(
        &config.secret_key,
        "checkout/sessions",
        &form,
        &checkout_idempotency_key,
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return crate::routes::json_error(StatusCode::BAD_GATEWAY, error);
        }
    };
    let Some(checkout_url) = json_string(&response, &["url"]) else {
        return crate::routes::json_error(StatusCode::BAD_GATEWAY, "Stripe 未返回 Checkout URL");
    };
    if !trusted_stripe_redirect(&checkout_url, "checkout.stripe.com") {
        return crate::routes::json_error(
            StatusCode::BAD_GATEWAY,
            "Stripe 返回了不可信的 Checkout URL",
        );
    }
    Json(json!({ "checkout_url": checkout_url })).into_response()
}

pub(crate) async fn handle_create_portal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let config = match StripeApiConfig::from_env() {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::SERVICE_UNAVAILABLE, error),
    };
    if let Err(response) = require_browser_mutation(&headers, &config.public_base_url) {
        return response;
    }
    let user = match crate::routes::public::require_public_session_user(&state, &headers) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let entitlements = match state.billing.list_user_entitlements(&user.user_id).await {
        Ok(value) => value,
        Err(error) => {
            return crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("读取 Stripe Customer 失败: {error}"),
            );
        }
    };
    let customer_ids = entitlements
        .iter()
        .filter(|value| value.provider == BILLING_PROVIDER_STRIPE)
        .filter_map(|value| value.provider_customer_id.clone())
        .collect::<BTreeSet<_>>();
    let customer_id = match customer_ids.len() {
        0 => {
            return crate::routes::json_error(
                StatusCode::NOT_FOUND,
                "账号尚未关联 Stripe Customer",
            );
        }
        1 => match customer_ids.into_iter().next() {
            Some(value) => value,
            None => {
                return crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Stripe Customer 状态不一致",
                );
            }
        },
        _ => {
            return crate::routes::json_error(
                StatusCode::CONFLICT,
                "账号关联了多个 Stripe Customer，请联系支持",
            );
        }
    };
    let return_url = format!("{}me", normalized_base_url(&config.public_base_url));
    let response = match stripe_form_post(
        &config.secret_key,
        "billing_portal/sessions",
        &[
            ("customer".to_string(), customer_id),
            ("return_url".to_string(), return_url),
        ],
        &format!("hone-portal-{}", uuid::Uuid::new_v4()),
    )
    .await
    {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_GATEWAY, error),
    };
    let Some(portal_url) = json_string(&response, &["url"]) else {
        return crate::routes::json_error(
            StatusCode::BAD_GATEWAY,
            "Stripe 未返回 Customer Portal URL",
        );
    };
    if !trusted_stripe_redirect(&portal_url, "billing.stripe.com") {
        return crate::routes::json_error(
            StatusCode::BAD_GATEWAY,
            "Stripe 返回了不可信的 Customer Portal URL",
        );
    }
    Json(json!({ "portal_url": portal_url })).into_response()
}

fn normalize_stripe_event(
    envelope: &Value,
    config: &StripeCatalogConfig,
) -> Result<StripeNormalization, String> {
    let event_id =
        json_string(envelope, &["id"]).ok_or_else(|| "Stripe event ID 缺失".to_string())?;
    let event_type =
        json_string(envelope, &["type"]).ok_or_else(|| "Stripe event type 缺失".to_string())?;
    let created =
        json_i64(envelope, &["created"]).ok_or_else(|| "Stripe event created 缺失".to_string())?;
    let object = json_value_at(envelope, &["data", "object"])
        .ok_or_else(|| "Stripe event data.object 缺失".to_string())?;
    match event_type.as_str() {
        "checkout.session.completed"
        | "checkout.session.async_payment_succeeded"
        | "checkout.session.async_payment_failed" => {
            normalize_checkout_event(object, config, &event_id, &event_type, created)
        }
        "checkout.session.expired" => Ok(StripeNormalization::Ignored("checkout_expired")),
        "invoice.paid" | "invoice.payment_failed" => {
            normalize_invoice_event(object, config, &event_id, &event_type, created)
        }
        "customer.subscription.created"
        | "customer.subscription.updated"
        | "customer.subscription.deleted" => {
            normalize_subscription_event(object, config, &event_id, &event_type, created)
        }
        "charge.refunded" => {
            normalize_charge_refund_event(object, config, &event_id, &event_type, created)
        }
        _ => Ok(StripeNormalization::Ignored("unsupported_event")),
    }
}

fn normalize_checkout_event(
    object: &Value,
    config: &StripeCatalogConfig,
    event_id: &str,
    event_type: &str,
    created: i64,
) -> Result<StripeNormalization, String> {
    let product_id = metadata_string(object, "hone_product_id").unwrap_or_default();
    let price_id = metadata_string(object, "hone_price_id").unwrap_or_default();
    let entitlement_kind = metadata_string(object, "hone_entitlement_kind").unwrap_or_default();
    let Some(expected_price_id) = config.price_for_kind(&entitlement_kind) else {
        return Ok(StripeNormalization::Ignored("catalog_mismatch"));
    };
    if product_id != config.product_id || price_id != expected_price_id {
        return Ok(StripeNormalization::Ignored("catalog_mismatch"));
    }
    let checkout_mode =
        json_string(object, &["mode"]).ok_or_else(|| "Stripe Checkout mode 缺失".to_string())?;
    let expected_mode = if entitlement_kind == BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION {
        "subscription"
    } else {
        "payment"
    };
    if checkout_mode != expected_mode {
        return Err("Stripe Checkout mode 与权益类型不匹配".to_string());
    }
    if entitlement_kind == BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
        && metadata_string(object, "hone_term_months").as_deref() != Some("12")
    {
        return Err("Stripe 单次年费期限配置不合法".to_string());
    }
    let metadata_user_id = metadata_string(object, "hone_user_id")
        .ok_or_else(|| "Stripe Checkout hone_user_id 缺失".to_string())?;
    let client_reference_id = json_string(object, &["client_reference_id"])
        .ok_or_else(|| "Stripe Checkout client_reference_id 缺失".to_string())?;
    if metadata_user_id != client_reference_id {
        return Err("Stripe Checkout 用户绑定字段不一致".to_string());
    }
    let provider_reference_id = if entitlement_kind == BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION {
        let subscription_id = json_id_at(object, &["subscription"])
            .ok_or_else(|| "Stripe Checkout subscription 缺失".to_string())?;
        validate_stripe_id(&subscription_id, "sub_", "Stripe subscription")?;
        subscription_id
    } else {
        let payment_intent_id = json_id_at(object, &["payment_intent"])
            .ok_or_else(|| "Stripe Checkout payment_intent 缺失".to_string())?;
        validate_stripe_id(&payment_intent_id, "pi_", "Stripe PaymentIntent")?;
        payment_intent_id
    };
    let payment_status = json_string(object, &["payment_status"]).unwrap_or_default();
    let (raw_status, access_signal) = match event_type {
        "checkout.session.async_payment_succeeded" => {
            ("paid".to_string(), StripeAccessSignal::Paid)
        }
        "checkout.session.async_payment_failed" => (
            "payment_failed".to_string(),
            StripeAccessSignal::PaymentFailed,
        ),
        "checkout.session.completed"
            if entitlement_kind == BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
                && payment_status == "paid" =>
        {
            ("paid".to_string(), StripeAccessSignal::Paid)
        }
        _ => (
            "checkout_completed".to_string(),
            StripeAccessSignal::Pending,
        ),
    };
    // Stripe can create the invoice/subscription events immediately before it
    // creates checkout.session.completed. The completed event is only a
    // provisional "pending" marker, so ordering it by the later webhook
    // envelope timestamp can incorrectly make the authoritative paid events
    // look stale. Anchor only this provisional transition to the Checkout
    // Session creation time; the webhook inbox still preserves the real event
    // creation time for audit and every authoritative transition keeps using
    // its own event timestamp.
    let ordering_created = if event_type == "checkout.session.completed"
        && access_signal == StripeAccessSignal::Pending
    {
        json_i64(object, &["created"]).ok_or_else(|| "Stripe Checkout created 缺失".to_string())?
    } else {
        created
    };
    let event_at = stripe_event_time(ordering_created, access_signal)?;
    let (current_period_start, current_period_end) = if entitlement_kind
        == BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
        && access_signal == StripeAccessSignal::Paid
    {
        let (start, end) = fixed_term_window(created)?;
        (Some(start), Some(end))
    } else {
        (None, None)
    };
    Ok(StripeNormalization::Relevant(StripeEntitlementEvent {
        user_id: metadata_user_id,
        entitlement_kind,
        provider_reference_id,
        customer_id: json_id_at(object, &["customer"]),
        email_address: json_string(object, &["customer_details", "email"])
            .or_else(|| json_string(object, &["customer_email"])),
        product_id,
        price_id,
        raw_status,
        access_signal,
        current_period_start,
        current_period_end,
        cancel_at_period_end: false,
        event_id: event_id.to_string(),
        event_type: event_type.to_string(),
        event_at,
    }))
}

fn normalize_invoice_event(
    object: &Value,
    config: &StripeCatalogConfig,
    event_id: &str,
    event_type: &str,
    created: i64,
) -> Result<StripeNormalization, String> {
    let matching_line = matching_catalog_item(
        object,
        &["lines", "data"],
        &config.product_id,
        &config.subscription_price_id,
    );
    let Some(line) = matching_line else {
        return Ok(StripeNormalization::Ignored("catalog_mismatch"));
    };
    let user_id = metadata_string_from_paths(
        object,
        "hone_user_id",
        &[
            &["parent", "subscription_details", "metadata"],
            &["metadata"],
            &["subscription_details", "metadata"],
        ],
    )
    .or_else(|| metadata_string(line, "hone_user_id"))
    .ok_or_else(|| "Stripe Invoice hone_user_id 缺失".to_string())?;
    let subscription_id = json_id_at(object, &["parent", "subscription_details", "subscription"])
        .or_else(|| json_id_at(object, &["subscription"]))
        .or_else(|| json_id_at(line, &["subscription"]));
    let subscription_id =
        subscription_id.ok_or_else(|| "Stripe Invoice subscription 缺失".to_string())?;
    validate_stripe_id(&subscription_id, "sub_", "Stripe subscription")?;
    let (raw_status, access_signal) = if event_type == "invoice.paid" {
        ("active".to_string(), StripeAccessSignal::Paid)
    } else {
        ("past_due".to_string(), StripeAccessSignal::PaymentFailed)
    };
    let event_at = stripe_event_time(created, access_signal)?;
    Ok(StripeNormalization::Relevant(StripeEntitlementEvent {
        user_id,
        entitlement_kind: BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION.to_string(),
        provider_reference_id: subscription_id,
        customer_id: json_id_at(object, &["customer"]),
        email_address: json_string(object, &["customer_email"]),
        product_id: config.product_id.clone(),
        price_id: config.subscription_price_id.clone(),
        raw_status,
        access_signal,
        current_period_start: unix_timestamp_at(line, &["period", "start"]),
        current_period_end: unix_timestamp_at(line, &["period", "end"]),
        cancel_at_period_end: false,
        event_id: event_id.to_string(),
        event_type: event_type.to_string(),
        event_at,
    }))
}

fn normalize_subscription_event(
    object: &Value,
    config: &StripeCatalogConfig,
    event_id: &str,
    event_type: &str,
    created: i64,
) -> Result<StripeNormalization, String> {
    let matching_item = matching_catalog_item(
        object,
        &["items", "data"],
        &config.product_id,
        &config.subscription_price_id,
    );
    let Some(item) = matching_item else {
        return Ok(StripeNormalization::Ignored("catalog_mismatch"));
    };
    let user_id = metadata_string(object, "hone_user_id")
        .ok_or_else(|| "Stripe Subscription hone_user_id 缺失".to_string())?;
    let subscription_id =
        json_string(object, &["id"]).ok_or_else(|| "Stripe Subscription ID 缺失".to_string())?;
    validate_stripe_id(&subscription_id, "sub_", "Stripe subscription")?;
    let raw_status = json_string(object, &["status"])
        .ok_or_else(|| "Stripe Subscription status 缺失".to_string())?;
    let access_signal = if event_type == "customer.subscription.deleted"
        || matches!(
            raw_status.as_str(),
            "unpaid" | "canceled" | "paused" | "incomplete_expired"
        ) {
        StripeAccessSignal::Inactive
    } else if raw_status == "incomplete" {
        StripeAccessSignal::Pending
    } else {
        StripeAccessSignal::Status
    };
    if !matches!(
        raw_status.as_str(),
        "active"
            | "trialing"
            | "past_due"
            | "incomplete"
            | "unpaid"
            | "canceled"
            | "paused"
            | "incomplete_expired"
    ) {
        return Err("Stripe Subscription status 不受支持".to_string());
    }
    let event_at = stripe_event_time(created, access_signal)?;
    Ok(StripeNormalization::Relevant(StripeEntitlementEvent {
        user_id,
        entitlement_kind: BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION.to_string(),
        provider_reference_id: subscription_id,
        customer_id: json_id_at(object, &["customer"]),
        email_address: None,
        product_id: config.product_id.clone(),
        price_id: config.subscription_price_id.clone(),
        raw_status,
        access_signal,
        current_period_start: unix_timestamp_at(object, &["current_period_start"])
            .or_else(|| unix_timestamp_at(item, &["current_period_start"]))
            .or_else(|| unix_timestamp_at(item, &["current_period", "start"])),
        current_period_end: unix_timestamp_at(object, &["current_period_end"])
            .or_else(|| unix_timestamp_at(item, &["current_period_end"]))
            .or_else(|| unix_timestamp_at(item, &["current_period", "end"])),
        cancel_at_period_end: json_bool(object, &["cancel_at_period_end"]).unwrap_or(false),
        event_id: event_id.to_string(),
        event_type: event_type.to_string(),
        event_at,
    }))
}

fn normalize_charge_refund_event(
    object: &Value,
    config: &StripeCatalogConfig,
    event_id: &str,
    event_type: &str,
    created: i64,
) -> Result<StripeNormalization, String> {
    let entitlement_kind = metadata_string(object, "hone_entitlement_kind").unwrap_or_default();
    let product_id = metadata_string(object, "hone_product_id").unwrap_or_default();
    let price_id = metadata_string(object, "hone_price_id").unwrap_or_default();
    if entitlement_kind != BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
        || product_id != config.product_id
        || price_id != config.fixed_term_price_id
    {
        return Ok(StripeNormalization::Ignored("catalog_mismatch"));
    }
    if metadata_string(object, "hone_term_months").as_deref() != Some("12") {
        return Err("Stripe 单次年费退款期限配置不合法".to_string());
    }
    let amount =
        json_i64(object, &["amount"]).ok_or_else(|| "Stripe Charge amount 缺失".to_string())?;
    let amount_refunded = json_i64(object, &["amount_refunded"])
        .ok_or_else(|| "Stripe Charge amount_refunded 缺失".to_string())?;
    let refunded = json_bool(object, &["refunded"]).unwrap_or(false);
    if amount <= 0 || !refunded || amount_refunded < amount {
        return Ok(StripeNormalization::Ignored("partial_refund"));
    }
    let user_id = metadata_string(object, "hone_user_id")
        .ok_or_else(|| "Stripe Charge hone_user_id 缺失".to_string())?;
    let provider_reference_id = json_id_at(object, &["payment_intent"])
        .ok_or_else(|| "Stripe Charge payment_intent 缺失".to_string())?;
    validate_stripe_id(&provider_reference_id, "pi_", "Stripe Charge PaymentIntent")?;
    Ok(StripeNormalization::Relevant(StripeEntitlementEvent {
        user_id,
        entitlement_kind,
        provider_reference_id,
        customer_id: json_id_at(object, &["customer"]),
        email_address: json_string(object, &["billing_details", "email"]),
        product_id,
        price_id,
        raw_status: "refunded".to_string(),
        access_signal: StripeAccessSignal::Inactive,
        current_period_start: None,
        current_period_end: None,
        cancel_at_period_end: false,
        event_id: event_id.to_string(),
        event_type: event_type.to_string(),
        event_at: stripe_event_time(created, StripeAccessSignal::Inactive)?,
    }))
}

pub(crate) fn spawn_stripe_processing(state: Arc<AppState>, event_id: String) {
    tokio::spawn(async move {
        for attempt in 1..=3u32 {
            let claimed = match state
                .billing
                .claim_webhook_event(BILLING_PROVIDER_STRIPE, &event_id)
                .await
            {
                Ok(value) => value,
                Err(error) => {
                    warn!(%event_id, %error, "Stripe billing event claim failed");
                    return;
                }
            };
            let Some(claimed) = claimed else {
                return;
            };
            let claim_attempt = claimed.attempt_count;
            let event: StripeEntitlementEvent =
                match serde_json::from_value(claimed.normalized_payload) {
                    Ok(value) => value,
                    Err(error) => {
                        let message = format!("Stripe 标准事件反序列化失败: {error}");
                        let _ = state
                            .billing
                            .finish_webhook_event(
                                BILLING_PROVIDER_STRIPE,
                                &event_id,
                                claim_attempt,
                                Err(&message),
                            )
                            .await;
                        return;
                    }
                };
            match apply_stripe_entitlement(&state, &event).await {
                Ok(outcome) => {
                    match state
                        .billing
                        .finish_webhook_event(
                            BILLING_PROVIDER_STRIPE,
                            &event_id,
                            claim_attempt,
                            Ok(()),
                        )
                        .await
                    {
                        Err(error) => {
                            warn!(%event_id, %error, "Stripe billing event completion failed");
                        }
                        Ok(false) => {
                            warn!(%event_id, claim_attempt, "Stripe billing event completion lost its lease");
                        }
                        Ok(true) => {
                            info!(%event_id, outcome = ?outcome, "Stripe billing entitlement processed");
                        }
                    }
                    return;
                }
                Err(error) => {
                    let _ = state
                        .billing
                        .finish_webhook_event(
                            BILLING_PROVIDER_STRIPE,
                            &event_id,
                            claim_attempt,
                            Err(&error),
                        )
                        .await;
                    if attempt == 3 {
                        warn!(%event_id, %error, "Stripe billing event exhausted retries");
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(250 * u64::from(attempt))).await;
                }
            }
        }
    });
}

async fn apply_stripe_entitlement(
    state: &AppState,
    event: &StripeEntitlementEvent,
) -> Result<BillingEntitlementUpsertOutcome, String> {
    let user = state
        .web_auth
        .find_invite_user(&event.user_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Stripe webhook 对应的 HONE 用户不存在".to_string())?;
    if user.revoked_at.is_some() {
        return Err("Stripe webhook 对应的 HONE 用户已禁用".to_string());
    }
    let profile = state
        .web_auth
        .external_profile(&user.user_id)
        .map_err(|error| error.to_string())?;
    if profile.identity_kind != WEB_IDENTITY_INTERNATIONAL_EMAIL {
        return Err("Stripe webhook 只能绑定国际邮箱身份".to_string());
    }
    let profile_email = profile
        .email_address
        .clone()
        .ok_or_else(|| "Stripe webhook 对应账号缺少邮箱".to_string())?;
    if profile.email_verified_at.is_none() {
        return Err("Stripe webhook 对应账号邮箱尚未验证".to_string());
    }
    if event
        .email_address
        .as_deref()
        .is_some_and(|value| value.trim().to_ascii_lowercase() != profile_email)
    {
        return Err("Stripe 付款邮箱与已验证 HONE 邮箱不一致".to_string());
    }
    let existing = state
        .billing
        .find_entitlement(BILLING_PROVIDER_STRIPE, &event.provider_reference_id)
        .await
        .map_err(|error| error.to_string())?;
    if existing
        .as_ref()
        .is_some_and(|current| current.user_id != user.user_id)
    {
        return Err("Stripe 权益引用已绑定到另一个 HONE 用户".to_string());
    }
    if existing
        .as_ref()
        .is_some_and(|current| current.entitlement_kind != event.entitlement_kind)
    {
        return Err("Stripe 权益引用的类型发生冲突".to_string());
    }
    if let (Some(current), Some(incoming)) = (
        existing
            .as_ref()
            .and_then(|value| value.provider_customer_id.as_deref()),
        event.customer_id.as_deref(),
    ) && current != incoming
    {
        return Err("Stripe 权益的 Customer 发生冲突".to_string());
    }
    let access_state = stripe_access_state(event, existing.as_ref());
    let grace_expires_at = if access_state == BILLING_ACCESS_GRACE {
        existing
            .as_ref()
            .filter(|value| value.access_state == BILLING_ACCESS_GRACE)
            .and_then(|value| value.grace_expires_at.clone())
            .or(Some(grace_deadline(&event.event_at)?))
    } else {
        None
    };
    let now = hone_core::local_now_rfc3339();
    state
        .billing
        .upsert_entitlement(BillingEntitlement {
            entitlement_id: BillingStorage::entitlement_id(
                BILLING_PROVIDER_STRIPE,
                &event.provider_reference_id,
            ),
            user_id: user.user_id,
            provider: BILLING_PROVIDER_STRIPE.to_string(),
            entitlement_kind: event.entitlement_kind.clone(),
            provider_customer_id: event.customer_id.clone().or_else(|| {
                existing
                    .as_ref()
                    .and_then(|value| value.provider_customer_id.clone())
            }),
            provider_reference_id: event.provider_reference_id.clone(),
            provider_product_id: Some(event.product_id.clone()),
            provider_price_id: Some(event.price_id.clone()),
            purchase_email_normalized: Some(profile_email),
            raw_status: event.raw_status.clone(),
            access_state: access_state.to_string(),
            current_period_start: event.current_period_start.clone().or_else(|| {
                existing
                    .as_ref()
                    .and_then(|value| value.current_period_start.clone())
            }),
            current_period_end: event.current_period_end.clone().or_else(|| {
                existing
                    .as_ref()
                    .and_then(|value| value.current_period_end.clone())
            }),
            cancel_at_period_end: event.cancel_at_period_end,
            manage_url: None,
            grace_expires_at,
            last_event_id: event.event_id.clone(),
            last_event_created_at: event.event_at.clone(),
            created_at: existing
                .as_ref()
                .map(|value| value.created_at.clone())
                .unwrap_or_else(|| event.event_at.clone()),
            updated_at: now,
        })
        .await
        .map_err(|error| error.to_string())
}

fn stripe_access_state(
    event: &StripeEntitlementEvent,
    existing: Option<&BillingEntitlement>,
) -> &'static str {
    if event.entitlement_kind == BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE {
        return match event.access_signal {
            StripeAccessSignal::Paid => BILLING_ACCESS_ACTIVE,
            StripeAccessSignal::Inactive => BILLING_ACCESS_INACTIVE,
            StripeAccessSignal::Pending | StripeAccessSignal::PaymentFailed => {
                BILLING_ACCESS_PENDING
            }
            StripeAccessSignal::Status => BILLING_ACCESS_PENDING,
        };
    }
    match event.access_signal {
        StripeAccessSignal::Paid => BILLING_ACCESS_ACTIVE,
        StripeAccessSignal::Pending => BILLING_ACCESS_PENDING,
        StripeAccessSignal::PaymentFailed => {
            if existing.is_some_and(BillingEntitlement::grants_paid_access) {
                BILLING_ACCESS_GRACE
            } else {
                BILLING_ACCESS_PENDING
            }
        }
        StripeAccessSignal::Inactive => BILLING_ACCESS_INACTIVE,
        StripeAccessSignal::Status => match event.raw_status.as_str() {
            "active" | "trialing"
                if existing.is_some_and(BillingEntitlement::grants_paid_access) =>
            {
                BILLING_ACCESS_ACTIVE
            }
            "past_due" if existing.is_some_and(BillingEntitlement::grants_paid_access) => {
                BILLING_ACCESS_GRACE
            }
            "unpaid" | "canceled" | "paused" | "incomplete_expired" => BILLING_ACCESS_INACTIVE,
            _ => BILLING_ACCESS_PENDING,
        },
    }
}

fn verified_checkout_profile(
    state: &AppState,
    user_id: &str,
) -> Result<hone_memory::WebUserExternalProfile, Response> {
    let profile = state.web_auth.external_profile(user_id).map_err(|error| {
        crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("读取邮箱身份失败: {error}"),
        )
    })?;
    if profile.identity_kind != WEB_IDENTITY_INTERNATIONAL_EMAIL
        || profile.email_address.is_none()
        || profile.email_verified_at.is_none()
    {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "请先完成邮箱验证，再创建 Stripe Checkout",
        ));
    }
    Ok(profile)
}

fn require_browser_mutation(headers: &HeaderMap, public_base_url: &Url) -> Result<(), Response> {
    if crate::routes::billing::is_hone_ios(headers) {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "iOS App 内不提供外部购买或订阅管理跳转",
        ));
    }
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| crate::routes::json_error(StatusCode::FORBIDDEN, "缺少 Origin"))?;
    let expected_origin = public_base_url.origin().ascii_serialization();
    let configured = std::env::var("HONE_PUBLIC_ALLOWED_ORIGINS").unwrap_or_default();
    let origin_header = axum::http::HeaderValue::from_str(origin)
        .map_err(|_| crate::routes::json_error(StatusCode::FORBIDDEN, "Origin 不合法"))?;
    if origin != expected_origin
        && !crate::routes::public_origin_allowed(&origin_header, &configured)
    {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "Origin 不受信任",
        ));
    }
    if headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value != "same-origin")
    {
        return Err(crate::routes::json_error(
            StatusCode::FORBIDDEN,
            "跨站浏览器请求已拒绝",
        ));
    }
    Ok(())
}

async fn stripe_form_post(
    secret_key: &str,
    path: &str,
    form: &[(String, String)],
    idempotency_key: &str,
) -> Result<Value, String> {
    let url = format!("{STRIPE_API_BASE}/{path}");
    let response = reqwest::Client::new()
        .post(url)
        .bearer_auth(secret_key)
        .header("Idempotency-Key", idempotency_key)
        .form(form)
        .send()
        .await
        .map_err(|error| format!("连接 Stripe API 失败: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("解析 Stripe API 响应失败: {error}"))?;
    if !status.is_success() {
        let message =
            json_string(&body, &["error", "message"]).unwrap_or_else(|| format!("HTTP {status}"));
        return Err(format!("Stripe API 请求失败: {}", truncate(&message, 300)));
    }
    Ok(body)
}

fn verify_stripe_signature(headers: &HeaderMap, body: &[u8], secret: &str) -> Result<(), String> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| "缺少 Stripe-Signature".to_string())?;
    let mut timestamp = None;
    let mut signatures = Vec::new();
    for item in signature.split(',').map(str::trim) {
        if let Some(value) = item.strip_prefix("t=") {
            timestamp = value.parse::<i64>().ok();
        } else if let Some(value) = item.strip_prefix("v1=") {
            if let Some(decoded) = decode_hex(value) {
                signatures.push(decoded);
            }
        }
    }
    let timestamp = timestamp.ok_or_else(|| "Stripe-Signature timestamp 不合法".to_string())?;
    if Utc::now().timestamp().abs_diff(timestamp) > WEBHOOK_TIMESTAMP_TOLERANCE_SECS as u64 {
        return Err("Stripe webhook timestamp 已过期".to_string());
    }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| "Stripe webhook secret 长度不合法".to_string())?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    let expected = mac.finalize().into_bytes();
    let matched = signatures.iter().any(|candidate| {
        candidate.len() == expected.len()
            && bool::from(candidate.as_slice().ct_eq(expected.as_slice()))
    });
    if !matched {
        return Err("Stripe webhook 签名无效".to_string());
    }
    Ok(())
}

fn matching_catalog_item<'a>(
    root: &'a Value,
    list_path: &[&str],
    product_id: &str,
    price_id: &str,
) -> Option<&'a Value> {
    json_value_at(root, list_path)?
        .as_array()?
        .iter()
        .find(|item| {
            catalog_ids(item)
                .is_some_and(|(product, price)| product == product_id && price == price_id)
        })
}

fn catalog_ids(value: &Value) -> Option<(String, String)> {
    let price = json_id_at(value, &["pricing", "price_details", "price"])
        .or_else(|| json_id_at(value, &["price"]));
    let product = json_id_at(value, &["pricing", "price_details", "product"])
        .or_else(|| json_id_at(value, &["price", "product"]));
    Some((product?, price?))
}

fn metadata_string(root: &Value, key: &str) -> Option<String> {
    json_string(root, &["metadata", key])
}

fn metadata_string_from_paths(root: &Value, key: &str, paths: &[&[&str]]) -> Option<String> {
    paths.iter().find_map(|path| {
        let metadata = json_value_at(root, path)?;
        json_string(metadata, &[key])
    })
}

fn stripe_event_time(created: i64, signal: StripeAccessSignal) -> Result<String, String> {
    let millis = match signal {
        StripeAccessSignal::Pending => 100,
        StripeAccessSignal::Status => 200,
        StripeAccessSignal::Paid => 400,
        StripeAccessSignal::PaymentFailed => 500,
        StripeAccessSignal::Inactive => 700,
    };
    Utc.timestamp_opt(created, millis * 1_000_000)
        .single()
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Millis, true))
        .ok_or_else(|| "Stripe event created 超出支持范围".to_string())
}

fn fixed_term_window(created: i64) -> Result<(String, String), String> {
    let start = Utc
        .timestamp_opt(created, 0)
        .single()
        .ok_or_else(|| "Stripe fixed-term paid timestamp 超出支持范围".to_string())?;
    let end = start
        .checked_add_months(Months::new(12))
        .ok_or_else(|| "Stripe fixed-term 到期时间超出支持范围".to_string())?;
    Ok((
        start.to_rfc3339_opts(SecondsFormat::Secs, true),
        end.to_rfc3339_opts(SecondsFormat::Secs, true),
    ))
}

fn unix_timestamp_at(root: &Value, path: &[&str]) -> Option<String> {
    let timestamp = json_i64(root, path)?;
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn trusted_stripe_redirect(value: &str, expected_host: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some_and(|host| {
                host == expected_host || host.ends_with(&format!(".{expected_host}"))
            })
    })
}

fn validate_public_base_url(value: &Url) -> Result<(), String> {
    let host = value
        .host_str()
        .ok_or_else(|| "HONE_STRIPE_PUBLIC_BASE_URL 缺少 host".to_string())?;
    let local = matches!(host, "localhost" | "127.0.0.1" | "::1");
    if value.scheme() != "https" && !(local && value.scheme() == "http") {
        return Err("HONE_STRIPE_PUBLIC_BASE_URL 必须为 HTTPS（本机开发除外）".to_string());
    }
    if value.query().is_some() || value.fragment().is_some() {
        return Err("HONE_STRIPE_PUBLIC_BASE_URL 不能包含 query 或 fragment".to_string());
    }
    Ok(())
}

fn normalized_base_url(value: &Url) -> String {
    let mut value = value.clone();
    value.set_path("/");
    value.set_query(None);
    value.set_fragment(None);
    value.to_string()
}

fn checkout_idempotency_key(
    user_id: &str,
    price_id: &str,
    entitlements: &[BillingEntitlement],
    utc_day: &str,
) -> String {
    let stripe_state = entitlements
        .iter()
        .filter(|value| value.provider == BILLING_PROVIDER_STRIPE)
        .map(|value| format!("{}:{}", value.provider_reference_id, value.last_event_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("|");
    let seed = format!("{user_id}:{price_id}:{utc_day}:{stripe_state}");
    format!("hone-checkout-{}", sha256_hex(seed.as_bytes()))
}

fn is_supported_event_type(value: &str) -> bool {
    matches!(
        value,
        "checkout.session.completed"
            | "checkout.session.async_payment_succeeded"
            | "checkout.session.async_payment_failed"
            | "checkout.session.expired"
            | "invoice.paid"
            | "invoice.payment_failed"
            | "customer.subscription.created"
            | "customer.subscription.updated"
            | "customer.subscription.deleted"
            | "charge.refunded"
    )
}

fn json_value_at<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(root, |value, key| value.get(*key))
}

fn json_string(root: &Value, path: &[&str]) -> Option<String> {
    json_value_at(root, path)?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn json_bool(root: &Value, path: &[&str]) -> Option<bool> {
    json_value_at(root, path)?.as_bool()
}

fn json_i64(root: &Value, path: &[&str]) -> Option<i64> {
    json_value_at(root, path)?.as_i64()
}

fn json_id_at(root: &Value, path: &[&str]) -> Option<String> {
    let value = json_value_at(root, path)?;
    if let Some(value) = value.as_str() {
        let value = value.trim();
        return (!value.is_empty()).then(|| value.to_string());
    }
    value
        .get("id")?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn valid_stripe_id(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() > prefix.len()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn validate_stripe_id(value: &str, prefix: &str, label: &str) -> Result<(), String> {
    if valid_stripe_id(value, prefix) {
        Ok(())
    } else {
        Err(format!("{label} 格式不合法"))
    }
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let high = (chunk[0] as char).to_digit(16)?;
            let low = (chunk[1] as char).to_digit(16)?;
            Some(((high << 4) | low) as u8)
        })
        .collect()
}

fn sha256_hex(body: &[u8]) -> String {
    use sha2::Digest as _;
    sha2::Sha256::digest(body)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn required_env(key: &str) -> Result<String, String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{key} 未配置"))
}

fn grace_deadline(event_at: &str) -> Result<String, String> {
    let event_at = chrono::DateTime::parse_from_rfc3339(event_at)
        .map_err(|_| "Stripe event timestamp 格式不合法".to_string())?;
    Ok((event_at + chrono::Duration::days(billing_grace_days())).to_rfc3339())
}

fn billing_grace_days() -> i64 {
    std::env::var("HONE_BILLING_GRACE_DAYS")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .filter(|value| (1..=30).contains(value))
        .unwrap_or(7)
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

pub(crate) fn env_flag(key: &str, fallback: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" | "" => false,
            _ => fallback,
        },
        Err(_) => fallback,
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        HmacSha256, StripeAccessSignal, StripeCatalogConfig, StripeEntitlementEvent, StripeMode,
        StripeNormalization, checkout_idempotency_key, fixed_term_window, normalize_stripe_event,
        stripe_access_state, verify_stripe_signature,
    };
    use axum::http::{HeaderMap, HeaderValue};
    use hmac::Mac;
    use hone_memory::{
        BILLING_ACCESS_ACTIVE, BILLING_ACCESS_GRACE, BILLING_ACCESS_INACTIVE,
        BILLING_ACCESS_PENDING, BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE,
        BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION, BILLING_PROVIDER_STRIPE, BillingEntitlement,
        BillingStorage,
    };
    use serde_json::json;

    fn config() -> StripeCatalogConfig {
        StripeCatalogConfig {
            mode: StripeMode::Test,
            product_id: "prod_test123".to_string(),
            subscription_price_id: "price_test123".to_string(),
            fixed_term_price_id: "price_fixed123".to_string(),
        }
    }

    #[test]
    fn stripe_mode_accepts_standard_and_restricted_keys_only_in_its_own_mode() {
        assert!(StripeMode::Test.accepts_secret_key("sk_test_example"));
        assert!(StripeMode::Test.accepts_secret_key("rk_test_example"));
        assert!(!StripeMode::Test.accepts_secret_key("sk_live_example"));
        assert!(!StripeMode::Test.accepts_secret_key("rk_live_example"));
        assert!(StripeMode::Live.accepts_secret_key("sk_live_example"));
        assert!(StripeMode::Live.accepts_secret_key("rk_live_example"));
        assert!(!StripeMode::Live.accepts_secret_key("sk_test_example"));
        assert!(!StripeMode::Live.accepts_secret_key("rk_test_example"));
        assert!(!StripeMode::Live.accepts_secret_key("rk_live_"));
    }

    fn signed_headers(body: &[u8], secret: &str) -> HeaderMap {
        let timestamp = chrono::Utc::now().timestamp();
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac");
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(body);
        let signature = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let mut headers = HeaderMap::new();
        headers.insert(
            "stripe-signature",
            HeaderValue::from_str(&format!("t={timestamp},v1={signature}")).expect("signature"),
        );
        headers
    }

    fn entitlement(access_state: &str) -> BillingEntitlement {
        BillingEntitlement {
            entitlement_id: BillingStorage::entitlement_id(BILLING_PROVIDER_STRIPE, "sub_test123"),
            user_id: "web_test123".to_string(),
            provider: BILLING_PROVIDER_STRIPE.to_string(),
            entitlement_kind: BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION.to_string(),
            provider_customer_id: Some("cus_test123".to_string()),
            provider_reference_id: "sub_test123".to_string(),
            provider_product_id: Some("prod_test123".to_string()),
            provider_price_id: Some("price_test123".to_string()),
            purchase_email_normalized: Some("buyer@example.com".to_string()),
            raw_status: "active".to_string(),
            access_state: access_state.to_string(),
            current_period_start: None,
            current_period_end: None,
            cancel_at_period_end: false,
            manage_url: None,
            grace_expires_at: (access_state == BILLING_ACCESS_GRACE)
                .then(|| "2099-08-10T03:00:00+00:00".to_string()),
            last_event_id: "evt_previous".to_string(),
            last_event_created_at: "2026-08-03T00:00:00Z".to_string(),
            created_at: "2026-08-03T00:00:00Z".to_string(),
            updated_at: "2026-08-03T00:00:00Z".to_string(),
        }
    }

    fn event(signal: StripeAccessSignal, status: &str) -> StripeEntitlementEvent {
        StripeEntitlementEvent {
            user_id: "web_test123".to_string(),
            entitlement_kind: BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION.to_string(),
            provider_reference_id: "sub_test123".to_string(),
            customer_id: Some("cus_test123".to_string()),
            email_address: Some("buyer@example.com".to_string()),
            product_id: "prod_test123".to_string(),
            price_id: "price_test123".to_string(),
            raw_status: status.to_string(),
            access_signal: signal,
            current_period_start: None,
            current_period_end: None,
            cancel_at_period_end: false,
            event_id: "evt_test123".to_string(),
            event_type: "test".to_string(),
            event_at: "2026-08-03T01:00:00Z".to_string(),
        }
    }

    #[test]
    fn stripe_signature_uses_raw_body_and_fresh_timestamp() {
        let secret = "whsec_test_only_not_a_secret";
        let body = br#"{"id":"evt_test123"}"#;
        let headers = signed_headers(body, secret);
        verify_stripe_signature(&headers, body, secret).expect("valid");
        assert!(verify_stripe_signature(&headers, b"{}", secret).is_err());
    }

    #[test]
    fn subscription_event_normalizes_exact_catalog_and_never_grants_first_payment() {
        let envelope = json!({
            "id": "evt_test123",
            "type": "customer.subscription.updated",
            "created": 1785686400,
            "livemode": false,
            "data": { "object": {
                "id": "sub_test123",
                "customer": "cus_test123",
                "status": "active",
                "metadata": { "hone_user_id": "web_test123" },
                "items": { "data": [{
                    "price": { "id": "price_test123", "product": "prod_test123" },
                    "current_period_start": 1785686400,
                    "current_period_end": 1817222400
                }] },
                "cancel_at_period_end": false
            } }
        });
        let StripeNormalization::Relevant(event) =
            normalize_stripe_event(&envelope, &config()).expect("event")
        else {
            panic!("expected relevant event")
        };
        assert_eq!(event.provider_reference_id, "sub_test123");
        assert_eq!(event.access_signal, StripeAccessSignal::Status);
        assert_eq!(
            event.current_period_end.as_deref(),
            Some("2027-08-02T16:00:00Z")
        );
        assert_eq!(stripe_access_state(&event, None), BILLING_ACCESS_PENDING);
    }

    #[test]
    fn access_state_requires_payment_and_uses_bounded_post_payment_grace() {
        assert_eq!(
            stripe_access_state(&event(StripeAccessSignal::Paid, "paid"), None),
            BILLING_ACCESS_ACTIVE
        );
        assert_eq!(
            stripe_access_state(&event(StripeAccessSignal::PaymentFailed, "past_due"), None),
            BILLING_ACCESS_PENDING
        );
        let active = entitlement(BILLING_ACCESS_ACTIVE);
        assert_eq!(
            stripe_access_state(
                &event(StripeAccessSignal::PaymentFailed, "past_due"),
                Some(&active)
            ),
            BILLING_ACCESS_GRACE
        );
        assert_eq!(
            stripe_access_state(
                &event(StripeAccessSignal::Inactive, "canceled"),
                Some(&active)
            ),
            BILLING_ACCESS_INACTIVE
        );
    }

    #[test]
    fn checkout_idempotency_is_stable_until_stripe_state_or_day_changes() {
        let initial = checkout_idempotency_key("web_test123", "price_test123", &[], "20260803");
        assert_eq!(
            initial,
            checkout_idempotency_key("web_test123", "price_test123", &[], "20260803",)
        );

        let mut canceled = entitlement(BILLING_ACCESS_INACTIVE);
        canceled.last_event_id = "evt_canceled".to_string();
        assert_ne!(
            initial,
            checkout_idempotency_key("web_test123", "price_test123", &[canceled], "20260803",)
        );
        assert_ne!(
            initial,
            checkout_idempotency_key("web_test123", "price_test123", &[], "20260804",)
        );
    }

    #[test]
    fn invoice_event_supports_current_parent_and_pricing_shape() {
        let envelope = json!({
            "id": "evt_invoice123",
            "type": "invoice.paid",
            "created": 1785686400,
            "livemode": false,
            "data": { "object": {
                "id": "in_test123",
                "customer": "cus_test123",
                "customer_email": "buyer@example.com",
                "parent": { "subscription_details": {
                    "subscription": "sub_test123",
                    "metadata": { "hone_user_id": "web_test123" }
                } },
                "lines": { "data": [{
                    "pricing": { "price_details": {
                        "price": "price_test123",
                        "product": "prod_test123"
                    } },
                    "period": { "start": 1785686400, "end": 1817222400 }
                }] }
            } }
        });
        let StripeNormalization::Relevant(event) =
            normalize_stripe_event(&envelope, &config()).expect("event")
        else {
            panic!("expected relevant event")
        };
        assert_eq!(event.user_id, "web_test123");
        assert_eq!(event.access_signal, StripeAccessSignal::Paid);
        assert_eq!(event.price_id, "price_test123");
    }

    #[test]
    fn checkout_pending_uses_session_creation_as_its_ordering_baseline() {
        let envelope = json!({
            "id": "evt_checkout123",
            "type": "checkout.session.completed",
            "created": 1785686700,
            "livemode": false,
            "data": { "object": {
                "id": "cs_test123",
                "created": 1785686400,
                "mode": "subscription",
                "subscription": "sub_test123",
                "customer": "cus_test123",
                "client_reference_id": "web_test123",
                "metadata": {
                    "hone_user_id": "web_test123",
                    "hone_product_id": "prod_test123",
                    "hone_price_id": "price_test123",
                    "hone_entitlement_kind": "recurring_subscription"
                },
                "customer_details": { "email": "buyer@example.com" }
            } }
        });
        let StripeNormalization::Relevant(event) =
            normalize_stripe_event(&envelope, &config()).expect("event")
        else {
            panic!("expected relevant event")
        };
        assert_eq!(event.access_signal, StripeAccessSignal::Pending);
        assert_eq!(event.event_at, "2026-08-02T16:00:00.100Z");
    }

    #[test]
    fn fixed_term_paid_checkout_grants_exact_twelve_calendar_months() {
        let envelope = json!({
            "id": "evt_fixed123",
            "type": "checkout.session.completed",
            "created": 1709208000,
            "livemode": false,
            "data": { "object": {
                "id": "cs_fixed123",
                "created": 1709207900,
                "mode": "payment",
                "payment_status": "paid",
                "payment_intent": "pi_fixed123",
                "customer": "cus_test123",
                "client_reference_id": "web_test123",
                "metadata": {
                    "hone_user_id": "web_test123",
                    "hone_product_id": "prod_test123",
                    "hone_price_id": "price_fixed123",
                    "hone_entitlement_kind": "fixed_term_purchase",
                    "hone_term_months": "12"
                },
                "customer_details": { "email": "buyer@example.com" }
            } }
        });
        let StripeNormalization::Relevant(event) =
            normalize_stripe_event(&envelope, &config()).expect("fixed checkout")
        else {
            panic!("expected relevant event")
        };
        assert_eq!(
            event.entitlement_kind,
            BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE
        );
        assert_eq!(event.provider_reference_id, "pi_fixed123");
        assert_eq!(event.access_signal, StripeAccessSignal::Paid);
        assert_eq!(
            event.current_period_start.as_deref(),
            Some("2024-02-29T12:00:00Z")
        );
        assert_eq!(
            event.current_period_end.as_deref(),
            Some("2025-02-28T12:00:00Z")
        );
        assert_eq!(stripe_access_state(&event, None), BILLING_ACCESS_ACTIVE);
        assert_eq!(
            fixed_term_window(1709208000).expect("window").1,
            "2025-02-28T12:00:00Z"
        );
    }

    #[test]
    fn unpaid_fixed_term_checkout_never_grants_access() {
        let envelope = json!({
            "id": "evt_fixed_pending",
            "type": "checkout.session.completed",
            "created": 1785686700,
            "livemode": false,
            "data": { "object": {
                "id": "cs_fixed_pending",
                "created": 1785686400,
                "mode": "payment",
                "payment_status": "unpaid",
                "payment_intent": "pi_fixed_pending",
                "client_reference_id": "web_test123",
                "metadata": {
                    "hone_user_id": "web_test123",
                    "hone_product_id": "prod_test123",
                    "hone_price_id": "price_fixed123",
                    "hone_entitlement_kind": "fixed_term_purchase",
                    "hone_term_months": "12"
                }
            } }
        });
        let StripeNormalization::Relevant(event) =
            normalize_stripe_event(&envelope, &config()).expect("pending checkout")
        else {
            panic!("expected relevant event")
        };
        assert_eq!(event.access_signal, StripeAccessSignal::Pending);
        assert_eq!(stripe_access_state(&event, None), BILLING_ACCESS_PENDING);
        assert!(event.current_period_end.is_none());
    }

    #[test]
    fn full_refund_revokes_only_the_matching_fixed_term_reference() {
        let envelope = json!({
            "id": "evt_refund123",
            "type": "charge.refunded",
            "created": 1785686800,
            "livemode": false,
            "data": { "object": {
                "id": "ch_fixed123",
                "payment_intent": "pi_fixed123",
                "customer": "cus_test123",
                "amount": 22999,
                "amount_refunded": 22999,
                "refunded": true,
                "metadata": {
                    "hone_user_id": "web_test123",
                    "hone_product_id": "prod_test123",
                    "hone_price_id": "price_fixed123",
                    "hone_entitlement_kind": "fixed_term_purchase",
                    "hone_term_months": "12"
                },
                "billing_details": { "email": "buyer@example.com" }
            } }
        });
        let StripeNormalization::Relevant(event) =
            normalize_stripe_event(&envelope, &config()).expect("refund")
        else {
            panic!("expected relevant event")
        };
        assert_eq!(event.provider_reference_id, "pi_fixed123");
        assert_eq!(event.access_signal, StripeAccessSignal::Inactive);
        assert_eq!(stripe_access_state(&event, None), BILLING_ACCESS_INACTIVE);
    }

    #[test]
    fn wrong_catalog_is_ignored() {
        let envelope = json!({
            "id": "evt_test123",
            "type": "customer.subscription.updated",
            "created": 1785686400,
            "livemode": false,
            "data": { "object": {
                "id": "sub_test123",
                "status": "active",
                "metadata": { "hone_user_id": "web_test123" },
                "items": { "data": [{
                    "price": { "id": "price_other", "product": "prod_other" }
                }] }
            } }
        });
        assert!(matches!(
            normalize_stripe_event(&envelope, &config()).expect("ignore"),
            StripeNormalization::Ignored("catalog_mismatch")
        ));
    }
}
