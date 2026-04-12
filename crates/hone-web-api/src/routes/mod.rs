pub(crate) mod auth;
pub(crate) mod chat;
pub(crate) mod company_profiles;
pub(crate) mod cron;
pub(crate) mod events;
pub(crate) mod files;
pub(crate) mod history;
pub(crate) mod imessage;
pub(crate) mod llm_audit;
pub(crate) mod logs;
pub(crate) mod meta;
pub(crate) mod portfolio;
pub(crate) mod research;
pub(crate) mod skills;
pub(crate) mod users;

mod common;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{get, patch, post, put};
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::runtime::web_dist_dir;
use crate::state::AppState;

pub fn build_app(state: Arc<AppState>) -> Router {
    let web_dist = web_dist_dir();
    let assets_dir = web_dist.join("assets");
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
            "/company-profiles/actors",
            get(company_profiles::handle_company_profile_spaces),
        )
        .route(
            "/company-profiles",
            get(company_profiles::handle_company_profiles),
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_auth,
        ))
        .layer(cors)
        .with_state(state.clone());

    Router::new()
        .route("/logo.svg", get(files::handle_logo))
        .nest("/api", api)
        .nest_service(
            "/assets",
            axum::routing::get_service(ServeDir::new(assets_dir)),
        )
        .fallback(get(files::handle_spa_index))
        .with_state(state)
}

pub(crate) use common::{
    json_error, normalize_optional_string, normalized_actor, normalized_query_actor, require_actor,
    require_string,
};
