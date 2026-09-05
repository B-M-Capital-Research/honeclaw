pub(crate) mod auth;
pub(crate) mod billing;
pub(crate) mod channel_settings;
pub(crate) mod chat;
pub(crate) mod community_forum;
pub(crate) mod company_facts;
pub(crate) mod company_profiles;
pub(crate) mod company_ratings;
pub(crate) mod cron;
pub(crate) mod daily_signals;
pub(crate) mod event_engine_admin;
pub(crate) mod events;
pub(crate) mod files;
pub(crate) mod history;
pub(crate) mod imessage;
pub(crate) mod industry_map;
pub(crate) mod influencer_digest;
pub(crate) mod key_event_chain;
pub(crate) mod llm_audit;
pub(crate) mod logs;
pub(crate) mod meta;
pub(crate) mod notification_prefs;
pub(crate) mod notifications;
pub(crate) mod portfolio;
pub(crate) mod portfolio_news;
pub(crate) mod position_management;
pub(crate) mod public;
pub(crate) mod public_admin;
pub(crate) mod public_community;
pub(crate) mod public_digest;
pub(crate) mod public_finance_calendar;
pub(crate) mod public_media;
pub(crate) mod public_portfolio;
pub(crate) mod public_pushes;
pub(crate) mod public_quotes;
pub(crate) mod public_subscriptions;
pub(crate) mod public_survey;
pub(crate) mod research;
pub(crate) mod research_library;
pub(crate) mod research_overview;
pub(crate) mod research_store;
pub(crate) mod schedule;
pub(crate) mod skills;
pub(crate) mod stripe;
pub(crate) mod task_runs;
pub(crate) mod unsubscribe;
pub(crate) mod users;
pub(crate) mod valuation_lab;
pub(crate) mod web_users;
pub(crate) mod weekly_brief;

mod common;

use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::{
    http::{HeaderValue, Method, StatusCode},
    response::Response,
};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::runtime::{public_web_dist_dir, web_dist_dir};
use crate::state::AppState;

async fn handle_not_found() -> Response {
    StatusCode::NOT_FOUND.into_response()
}

async fn handle_api_not_found() -> Response {
    common::json_error(StatusCode::NOT_FOUND, "api route not found")
}

pub(crate) fn public_origin_allowed(origin: &HeaderValue, configured_origins: &str) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    if matches!(
        origin,
        "https://hone-claw.com" | "https://www.hone-claw.com"
    ) {
        return true;
    }
    if origin == "http://localhost"
        || origin.starts_with("http://localhost:")
        || origin == "http://127.0.0.1"
        || origin.starts_with("http://127.0.0.1:")
        || origin == "http://[::1]"
        || origin.starts_with("http://[::1]:")
    {
        return true;
    }
    configured_origins
        .split(',')
        .map(str::trim)
        .filter(|allowed| !allowed.is_empty())
        .any(|allowed| allowed == origin)
}

pub fn build_admin_app(state: Arc<AppState>) -> Router {
    let web_dist = web_dist_dir();
    let index_path = web_dist.join("index.html");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/meta", get(meta::handle_meta))
        .route("/language", put(meta::handle_put_language))
        .route("/auth/sse-ticket", post(auth::handle_sse_ticket))
        .route("/runtime/heartbeat", post(meta::handle_runtime_heartbeat))
        .route(
            "/runtime/active-chat-runs",
            get(chat::handle_active_chat_runs),
        )
        .route("/channels", get(meta::handle_channels))
        .route(
            "/channel-settings",
            get(channel_settings::handle_get_channel_settings)
                .put(channel_settings::handle_put_channel_settings),
        )
        .route("/history", get(history::handle_history))
        .route("/events", get(events::handle_events))
        .route("/image", get(files::handle_image))
        .route("/file", get(files::handle_file))
        .route("/users", get(users::handle_users))
        .route(
            "/web-users/invites",
            get(web_users::handle_list_invites).post(web_users::handle_create_invite),
        )
        .route(
            "/web-users/invites/{user_id}/disable",
            post(web_users::handle_disable_invite),
        )
        .route(
            "/web-users/invites/{user_id}/enable",
            post(web_users::handle_enable_invite),
        )
        .route(
            "/web-users/invites/{user_id}/reset",
            post(web_users::handle_reset_invite),
        )
        .route(
            "/web-users/invites/{user_id}/api-key",
            post(web_users::handle_get_api_key),
        )
        .route(
            "/web-users/invites/{user_id}/api-key/reset",
            post(web_users::handle_reset_api_key),
        )
        .route("/skills", get(skills::handle_skills))
        .route("/skills/reset", post(skills::handle_skill_registry_reset))
        .route("/skills/{id}", get(skills::handle_skill_detail))
        .route(
            "/skills/{id}/state",
            patch(skills::handle_skill_state_update),
        )
        .route("/chat", post(chat::handle_chat))
        .route(
            "/cron-jobs",
            get(cron::handle_cron_jobs).post(cron::handle_create_cron_job),
        )
        .route(
            "/cron-jobs/{id}",
            get(cron::handle_cron_job)
                .put(cron::handle_update_cron_job)
                .delete(cron::handle_delete_cron_job),
        )
        .route("/cron-jobs/{id}/toggle", post(cron::handle_toggle_cron_job))
        .route("/portfolio/actors", get(portfolio::handle_portfolio_actors))
        .route("/portfolio", get(portfolio::handle_portfolio))
        .route(
            "/notification-prefs/batch",
            post(notification_prefs::handle_batch_get_prefs),
        )
        .route(
            "/notification-prefs",
            get(notification_prefs::handle_get_prefs).put(notification_prefs::handle_put_prefs),
        )
        .route(
            "/event-engine/global-digest",
            get(event_engine_admin::handle_get_global_digest)
                .put(event_engine_admin::handle_put_global_digest),
        )
        .route(
            "/event-engine/rss-feeds",
            get(event_engine_admin::handle_list_rss_feeds)
                .post(event_engine_admin::handle_create_rss_feed),
        )
        .route(
            "/event-engine/rss-feeds/{handle}",
            put(event_engine_admin::handle_update_rss_feed)
                .delete(event_engine_admin::handle_delete_rss_feed),
        )
        .route(
            "/event-engine/mainline-distill",
            post(event_engine_admin::handle_distill_mainline_now),
        )
        .route(
            "/event-engine/mainline-context",
            get(event_engine_admin::handle_get_mainline_context),
        )
        .route(
            "/event-engine/company-profile",
            get(event_engine_admin::handle_get_actor_company_profile),
        )
        .route(
            "/company-profiles/actors",
            get(company_profiles::handle_company_profile_spaces),
        )
        .route(
            "/company-profiles",
            get(company_profiles::handle_company_profiles),
        )
        .route(
            "/company-profiles/export",
            get(company_profiles::handle_export_company_profiles),
        )
        .route(
            "/company-profiles/import/preview",
            post(company_profiles::handle_preview_import_company_profiles),
        )
        .route(
            "/company-profiles/import/apply",
            post(company_profiles::handle_apply_import_company_profiles),
        )
        .route(
            "/company-profiles/{id}",
            get(company_profiles::handle_company_profile_detail)
                .delete(company_profiles::handle_delete_company_profile),
        )
        .route(
            "/portfolio/holdings",
            post(portfolio::handle_create_holding),
        )
        .route(
            "/portfolio/holdings/{symbol}",
            put(portfolio::handle_update_holding).delete(portfolio::handle_delete_holding),
        )
        .route("/research/start", post(research::handle_research_start))
        .route(
            "/research/status/{task_id}",
            get(research::handle_research_status),
        )
        .route(
            "/research/generate-pdf",
            post(research::handle_research_generate_pdf),
        )
        .route(
            "/research/download-pdf",
            get(research::handle_research_download_pdf),
        )
        .route("/imessage-event", post(imessage::handle_imessage_event))
        .route("/llm-audit", get(llm_audit::handle_llm_audit_list))
        .route("/llm-audit/{id}", get(llm_audit::handle_llm_audit_detail))
        .route("/logs", get(logs::handle_logs))
        .route("/logs/stream", get(logs::handle_logs_stream))
        .route("/admin/task-runs", get(task_runs::handle_task_runs))
        .route(
            "/admin/notifications",
            get(notifications::handle_notifications),
        )
        .route("/admin/schedule", get(schedule::handle_schedule))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_auth,
        ))
        .fallback(handle_api_not_found)
        .layer(cors.clone())
        .with_state(state.clone());

    let static_service = ServeDir::new(&web_dist).fallback(ServeFile::new(&index_path));

    Router::new()
        .route("/logo.svg", get(files::handle_logo))
        .route(
            "/api/public/{*path}",
            get(handle_not_found).post(handle_not_found),
        )
        .nest("/api", api)
        .fallback_service(static_service)
        .with_state(state)
}

pub fn build_public_app(state: Arc<AppState>) -> Router {
    let web_dist = public_web_dist_dir();
    let index_path = web_dist.join("index.html");
    let configured_origins = std::env::var("HONE_PUBLIC_ALLOWED_ORIGINS").unwrap_or_default();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin, _| {
            public_origin_allowed(origin, &configured_origins)
        }))
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            // The forum registers DELETE routes (post/comment removal); without
            // this the browser preflight rejects them cross-origin.
            Method::DELETE,
        ]))
        .allow_headers(AllowHeaders::mirror_request())
        .allow_credentials(true);

    let public_api = Router::new()
        .route("/auth/captcha/config", get(public::handle_captcha_config))
        .route(
            "/auth/dev-login/config",
            get(public::handle_dev_login_config),
        )
        .route("/auth/dev-login", post(public::handle_dev_login))
        .route("/auth/sms/send", post(public::handle_sms_send_code))
        .route("/auth/sms/login", post(public::handle_sms_login))
        .route("/auth/email/send", post(public::handle_email_send_code))
        .route("/auth/email/login", post(public::handle_email_login))
        .route("/auth/logout", post(public::handle_logout))
        .route("/auth/me", get(public::handle_me))
        .route("/billing/config", get(billing::handle_billing_config))
        .route("/billing/status", get(billing::handle_billing_status))
        .route(
            "/billing/entitlements",
            get(billing::handle_billing_entitlements),
        )
        .route(
            "/billing/checkout/stripe",
            post(stripe::handle_create_checkout),
        )
        .route("/billing/portal/stripe", post(stripe::handle_create_portal))
        .route(
            "/admin/invites",
            get(public_admin::handle_list_invites).post(public_admin::handle_create_invite),
        )
        .route(
            "/admin/invites/{user_id}/disable",
            post(public_admin::handle_disable_invite),
        )
        .route("/admin/usage", get(public_admin::handle_usage_report))
        .route(
            "/integrations/stripe/webhook",
            post(stripe::handle_stripe_webhook),
        )
        .route("/survey", post(public_survey::handle_submit_survey))
        .route(
            "/admin/survey",
            get(public_survey::handle_admin_survey_report),
        )
        .route("/bootstrap", get(public::handle_bootstrap))
        .route("/history", get(public::handle_history))
        .route("/community", get(public_community::handle_list_community))
        .route(
            "/community/state",
            get(public_community::handle_community_state),
        )
        .route(
            "/community/edge-session",
            post(public_community::handle_community_edge_session),
        )
        .route("/media/session", post(public_media::handle_media_session))
        .route(
            "/media/upload-grant",
            post(public_media::handle_media_upload_grant),
        )
        .route(
            "/community/seen",
            post(public_community::handle_mark_community_seen),
        )
        .route(
            "/community/resources/{resource_id}",
            get(public_community::handle_community_resource_preview),
        )
        .route("/community/forum", get(community_forum::handle_list))
        .route(
            "/community/forum/posts",
            post(community_forum::handle_create_post)
                .layer(DefaultBodyLimit::max(11 * 1024 * 1024)),
        )
        .route(
            "/community/forum/posts/{post_id}/like",
            post(community_forum::handle_toggle_like),
        )
        .route(
            "/community/forum/posts/{post_id}/comments",
            post(community_forum::handle_comment),
        )
        .route(
            "/community/forum/posts/{post_id}/report",
            post(community_forum::handle_report),
        )
        .route(
            "/community/forum/posts/{post_id}",
            delete(community_forum::handle_delete_post),
        )
        .route(
            "/community/forum/posts/{post_id}/comments/{comment_id}",
            delete(community_forum::handle_delete_comment),
        )
        .route(
            "/community/forum/posts/{post_id}/moderation",
            post(community_forum::handle_moderate),
        )
        .route(
            "/community/forum/posts/{post_id}/attachments/{attachment_id}",
            get(community_forum::handle_attachment),
        )
        .route("/chat", post(public::handle_chat))
        .route(
            "/v1/chat/completions",
            post(public::handle_openai_chat_completions),
        )
        .route("/upload", post(public::handle_upload))
        .route(
            "/research-library",
            get(research_library::handle_list)
                .post(research_library::handle_upload)
                // Uploads allow 20 MiB files (MAX_FILE_BYTES); axum's default
                // 2 MB body limit would reject them before that check runs.
                .layer(DefaultBodyLimit::max(21 * 1024 * 1024)),
        )
        .route(
            "/research-library/{id}",
            put(research_library::handle_update).delete(research_library::handle_delete),
        )
        .route(
            "/research-library/{id}/file",
            get(research_library::handle_download),
        )
        .route(
            "/research-library/{id}/submit",
            post(research_library::handle_submit_candidate),
        )
        .route(
            "/research-library/{id}/review",
            post(research_library::handle_review_candidate),
        )
        .route(
            "/finance-calendar",
            get(public_finance_calendar::handle_get_finance_calendar),
        )
        .route(
            "/finance-calendar/send",
            post(public_finance_calendar::handle_send_finance_calendar),
        )
        .route("/image", get(public::handle_public_image))
        .route("/file", get(public::handle_public_file))
        .route("/events", get(public::handle_events))
        .route("/pushes", get(public_pushes::handle_list_pushes))
        // Login-free: the signed token in the path is the authorisation. This
        // must live on the public API router because production only proxies
        // `/api/public/*` from Pages to the backend. GET renders a confirmation
        // and POST performs the mutation, so link previews stay harmless.
        .route(
            "/unsubscribe/{token}",
            get(unsubscribe::handle_unsubscribe_page).post(unsubscribe::handle_unsubscribe_submit),
        )
        // The caller's own schedules. Scoped to their actor: managing a push
        // used to be admin-only, which is why nobody could turn theirs off.
        .route(
            "/subscriptions",
            get(public_subscriptions::handle_list_subscriptions),
        )
        .route(
            "/subscriptions/{job_id}",
            axum::routing::put(public_subscriptions::handle_update_subscription),
        )
        .route(
            "/subscriptions/{job_id}/unsubscribe",
            post(public_subscriptions::handle_unsubscribe_subscription),
        )
        .route(
            "/pushes/{push_id}/open",
            post(public_pushes::handle_open_push),
        )
        .route("/quotes", get(public_quotes::handle_get_quotes))
        .route(
            "/company-ratings",
            get(company_ratings::handle_get_company_ratings),
        )
        .route("/industry-map", get(industry_map::handle_get_industry_map))
        .route(
            "/industry-map/edits",
            post(industry_map::handle_post_industry_edit),
        )
        .route(
            "/valuation-lab",
            get(valuation_lab::handle_get_valuation_lab),
        )
        .route(
            "/portfolio-news",
            get(portfolio_news::handle_get_portfolio_news),
        )
        .route(
            "/position-management",
            get(position_management::handle_get_position_management),
        )
        .route(
            "/influencer-digest",
            get(influencer_digest::handle_get_influencer_digest),
        )
        .route(
            "/key-event-chains",
            get(key_event_chain::handle_get_key_event_chains),
        )
        .route("/weekly-brief", get(weekly_brief::handle_get_weekly_brief))
        .route(
            "/research-overview",
            get(research_overview::handle_get_research_overview),
        )
        .route(
            "/daily-signals/{kind}",
            get(daily_signals::handle_get_daily_signal),
        )
        .route(
            "/daily-signals/{kind}/history",
            get(daily_signals::handle_get_daily_signal_history),
        )
        .route("/portfolio", get(public_portfolio::handle_get_portfolio))
        .route(
            "/portfolio/holdings",
            post(public_portfolio::handle_create_holding),
        )
        .route(
            "/portfolio/holdings/{symbol}",
            put(public_portfolio::handle_update_holding)
                .delete(public_portfolio::handle_delete_holding),
        )
        .route("/settings", get(public_digest::handle_get_settings))
        .route(
            "/settings/investor-style",
            put(public_digest::handle_put_investor_style),
        )
        .route(
            "/digest-context",
            get(public_digest::handle_get_digest_context),
        )
        .route(
            "/digest-context/refresh",
            post(public_digest::handle_refresh_digest_context),
        )
        .route(
            "/company-profile",
            get(public_digest::handle_get_company_profile),
        )
        .layer(cors)
        .with_state(state.clone());

    let static_service = ServeDir::new(&web_dist).fallback(ServeFile::new(&index_path));

    Router::new()
        .route("/logo.svg", get(files::handle_logo))
        .route("/api/{*path}", get(handle_not_found).post(handle_not_found))
        .nest("/api/public", public_api)
        .fallback_service(static_service)
        .with_state(state)
}

pub(crate) use common::{
    json_error, normalize_optional_string, normalized_actor, normalized_query_actor, require_actor,
    require_phone_number, require_string,
};

#[cfg(test)]
mod tests {
    use super::public_origin_allowed;
    use axum::http::HeaderValue;

    #[test]
    fn credentialed_public_cors_rejects_untrusted_origins() {
        assert!(public_origin_allowed(
            &HeaderValue::from_static("https://hone-claw.com"),
            ""
        ));
        assert!(public_origin_allowed(
            &HeaderValue::from_static("http://localhost:5173"),
            ""
        ));
        assert!(public_origin_allowed(
            &HeaderValue::from_static("https://app.example.test"),
            "https://app.example.test"
        ));
        assert!(!public_origin_allowed(
            &HeaderValue::from_static("https://evil.example"),
            ""
        ));
        assert!(!public_origin_allowed(
            &HeaderValue::from_static("https://hone-claw.com.evil.example"),
            ""
        ));
    }
}
