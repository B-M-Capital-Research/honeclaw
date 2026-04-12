use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde_json::json;

use hone_memory::{AppendEventInput, CreateProfileInput, IndustryTemplate, TrackingConfig};

use crate::routes::json_error;
use crate::state::AppState;
use crate::types::{
    CompanyProfileCreateRequest, CompanyProfileEventCreateRequest,
    CompanyProfileSectionsUpdateRequest, CompanyProfileTrackingUpdateRequest,
};

pub(crate) async fn handle_company_profiles(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let profiles = state.core.company_profile_storage.list_profiles();
    Json(json!({ "profiles": profiles }))
}

pub(crate) async fn handle_company_profile_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.core.company_profile_storage.get_profile(&id) {
        Ok(Some(profile)) => Json(json!({ "profile": profile })).into_response(),
        Ok(None) => json_error(
            axum::http::StatusCode::NOT_FOUND,
            "company profile not found",
        ),
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_create_company_profile(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CompanyProfileCreateRequest>,
) -> impl IntoResponse {
    let company_name = req.company_name.trim();
    if company_name.is_empty() {
        return json_error(axum::http::StatusCode::BAD_REQUEST, "company_name 不能为空");
    }

    let sections = req.sections.unwrap_or_else(BTreeMap::new);
    match state
        .core
        .company_profile_storage
        .create_profile(CreateProfileInput {
            company_name: company_name.to_string(),
            stock_code: req.stock_code,
            sector: req.sector,
            aliases: req.aliases.unwrap_or_default(),
            industry_template: parse_template(req.industry_template.as_deref()),
            tracking: None,
            initial_sections: sections,
        }) {
        Ok((profile, created)) => {
            Json(json!({ "profile": profile, "created": created })).into_response()
        }
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_update_company_profile_sections(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CompanyProfileSectionsUpdateRequest>,
) -> impl IntoResponse {
    match state
        .core
        .company_profile_storage
        .rewrite_sections(&id, &req.sections)
    {
        Ok(Some(profile)) => Json(json!({ "profile": profile })).into_response(),
        Ok(None) => json_error(
            axum::http::StatusCode::NOT_FOUND,
            "company profile not found",
        ),
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_update_company_profile_tracking(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CompanyProfileTrackingUpdateRequest>,
) -> impl IntoResponse {
    match state.core.company_profile_storage.set_tracking(
        &id,
        TrackingConfig {
            enabled: req.enabled,
            cadence: req.cadence.unwrap_or_else(|| "weekly".to_string()),
            focus_metrics: req.focus_metrics.unwrap_or_default(),
        },
    ) {
        Ok(Some(profile)) => Json(json!({ "profile": profile })).into_response(),
        Ok(None) => json_error(
            axum::http::StatusCode::NOT_FOUND,
            "company profile not found",
        ),
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

pub(crate) async fn handle_append_company_profile_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CompanyProfileEventCreateRequest>,
) -> impl IntoResponse {
    let title = req.title.trim();
    let event_type = req.event_type.trim();
    let occurred_at = req.occurred_at.trim();
    if title.is_empty() || event_type.is_empty() || occurred_at.is_empty() {
        return json_error(
            axum::http::StatusCode::BAD_REQUEST,
            "title、event_type、occurred_at 不能为空",
        );
    }

    match state.core.company_profile_storage.append_event(
        &id,
        AppendEventInput {
            title: title.to_string(),
            event_type: event_type.to_string(),
            occurred_at: occurred_at.to_string(),
            thesis_impact: req.thesis_impact.unwrap_or_else(|| "unknown".to_string()),
            changed_sections: req.changed_sections.unwrap_or_default(),
            refs: req.refs.unwrap_or_default(),
            what_happened: req.what_happened.unwrap_or_default(),
            why_it_matters: req.why_it_matters.unwrap_or_default(),
            thesis_effect: req.thesis_effect.unwrap_or_default(),
            evidence: req.evidence.unwrap_or_default(),
            research_log: req.research_log.unwrap_or_default(),
            follow_up: req.follow_up.unwrap_or_default(),
        },
    ) {
        Ok(Some(event)) => Json(json!({ "event": event })).into_response(),
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
) -> impl IntoResponse {
    match state.core.company_profile_storage.delete_profile(&id) {
        Ok(true) => Json(json!({ "ok": true })).into_response(),
        Ok(false) => json_error(
            axum::http::StatusCode::NOT_FOUND,
            "company profile not found",
        ),
        Err(err) => json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

fn parse_template(value: Option<&str>) -> IndustryTemplate {
    match value.unwrap_or("general").trim() {
        "saas" => IndustryTemplate::Saas,
        "semiconductor_hardware" => IndustryTemplate::SemiconductorHardware,
        "consumer" => IndustryTemplate::Consumer,
        "industrial_defense" => IndustryTemplate::IndustrialDefense,
        "financials" => IndustryTemplate::Financials,
        _ => IndustryTemplate::General,
    }
}
