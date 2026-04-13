use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde_json::json;

use crate::routes::{json_error, require_actor};
use crate::state::AppState;
use crate::types::UserIdQuery;

pub(crate) async fn handle_company_profile_spaces(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let spaces = state.core.company_profile_storage.list_profile_spaces_raw();
    Json(json!({ "actors": spaces }))
}

pub(crate) async fn handle_company_profiles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    let profiles = state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .list_profiles_raw();
    Json(json!({ "profiles": profiles })).into_response()
}

pub(crate) async fn handle_company_profile_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .get_profile_raw(&id)
    {
        Ok(Some(profile)) => Json(json!({ "profile": profile })).into_response(),
        Ok(None) => json_error(
            axum::http::StatusCode::NOT_FOUND,
            "company profile not found",
        ),
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_delete_company_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<UserIdQuery>,
) -> impl IntoResponse {
    let actor = match require_actor(params.channel, params.user_id, params.channel_scope) {
        Ok(actor) => actor,
        Err(error) => return error,
    };
    match state
        .core
        .company_profile_storage
        .for_actor(&actor)
        .delete_profile(&id)
    {
        Ok(true) => Json(json!({ "ok": true })).into_response(),
        Ok(false) => json_error(
            axum::http::StatusCode::NOT_FOUND,
            "company profile not found",
        ),
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}
