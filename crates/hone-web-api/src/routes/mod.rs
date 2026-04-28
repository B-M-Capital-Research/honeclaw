pub(crate) mod auth;
pub(crate) mod chat;
pub(crate) mod company_profiles;
pub(crate) mod cron;
pub(crate) mod event_engine_admin;
pub(crate) mod events;
pub(crate) mod files;
pub(crate) mod history;
pub(crate) mod imessage;
pub(crate) mod llm_audit;
pub(crate) mod logs;
pub(crate) mod meta;
pub(crate) mod notification_prefs;
pub(crate) mod notifications;
pub(crate) mod portfolio;
pub(crate) mod public;
pub(crate) mod public_digest;
pub(crate) mod research;
pub(crate) mod schedule;
pub(crate) mod skills;
pub(crate) mod task_runs;
pub(crate) mod users;
pub(crate) mod web_users;

mod common;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post, put};
use axum::{
    http::{Method, StatusCode},
    response::Response,
};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::runtime::{public_web_dist_dir, web_dist_dir};
use crate::state::AppState;

async fn handle_not_found() -> Response {
    StatusCode::NOT_FOUND.into_response()
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
        .route("/auth/sse-ticket", post(auth::handle_sse_ticket))
        .route("/runtime/heartbeat", post(meta::handle_runtime_heartbeat))
        .route("/channels", get(meta::handle_channels))
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
            "/event-engine/thesis-distill",
            post(event_engine_admin::handle_distill_thesis_now),
        )
        .route(
            "/event-engine/thesis-context",
            get(event_engine_admin::handle_get_thesis_context),
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
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods(AllowMethods::list([Method::GET, Method::POST]))
        .allow_headers(AllowHeaders::mirror_request())
        .allow_credentials(true);

    let public_api = Router::new()
        .route("/auth/invite-login", post(public::handle_invite_login))
        .route("/auth/password-login", post(public::handle_password_login))
        .route("/auth/set-password", post(public::handle_set_password))
        .route(
            "/auth/change-password",
            post(public::handle_change_password),
        )
        .route("/auth/logout", post(public::handle_logout))
        .route("/auth/me", get(public::handle_me))
        .route("/history", get(public::handle_history))
        .route("/chat", post(public::handle_chat))
        .route("/upload", post(public::handle_upload))
        .route("/image", get(public::handle_public_image))
        .route("/file", get(public::handle_public_file))
        .route("/events", get(public::handle_events))
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
