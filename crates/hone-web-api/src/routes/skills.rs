use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use hone_tools::{SkillRuntime, reset_skill_registry, set_skill_enabled};

use crate::routes::json_error;
use crate::state::AppState;
use crate::types::{SkillDetailInfo, SkillInfo, SkillStateUpdateRequest};

pub(crate) async fn handle_skills(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(
        collect_skill_infos(&state.core)
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>(),
    )
}

pub(crate) async fn handle_skill_detail(
    State(state): State<Arc<AppState>>,
    Path(skill_id): Path<String>,
) -> impl IntoResponse {
    match collect_skill_detail(&state.core, &skill_id) {
        Some(skill) => Json(serde_json::to_value(skill).unwrap_or_default()).into_response(),
        None => json_error(axum::http::StatusCode::NOT_FOUND, "skill not found").into_response(),
    }
}

pub(crate) async fn handle_skill_state_update(
    State(state): State<Arc<AppState>>,
    Path(skill_id): Path<String>,
    Json(payload): Json<SkillStateUpdateRequest>,
) -> impl IntoResponse {
    let enabled = payload.enabled.unwrap_or(true);
    if runtime(&state.core)
        .load_registered_skill(&skill_id)
        .is_err()
    {
        return json_error(axum::http::StatusCode::NOT_FOUND, "skill not found").into_response();
    }

    match set_skill_enabled(
        &state.core.configured_skill_registry_path(),
        &skill_id,
        enabled,
    ) {
        Ok(_) => match collect_skill_info(&state.core, &skill_id) {
            Some(skill) => Json(serde_json::to_value(skill).unwrap_or_default()).into_response(),
            None => {
                json_error(axum::http::StatusCode::NOT_FOUND, "skill not found").into_response()
            }
        },
        Err(error) => {
            json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, &error).into_response()
        }
    }
}

pub(crate) async fn handle_skill_registry_reset(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match reset_skill_registry(&state.core.configured_skill_registry_path()) {
        Ok(_) => Json(
            serde_json::to_value(collect_skill_infos(&state.core).unwrap_or_default())
                .unwrap_or_default(),
        )
        .into_response(),
        Err(error) => {
            json_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, &error).into_response()
        }
    }
}

fn runtime(core: &hone_channels::HoneBotCore) -> SkillRuntime {
    SkillRuntime::new(
        core.configured_system_skills_dir(),
        core.configured_custom_skills_dir(),
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    )
    .with_registry_path(core.configured_skill_registry_path())
}

fn collect_skill_infos(core: &hone_channels::HoneBotCore) -> Option<Vec<SkillInfo>> {
    let mut skills = runtime(core)
        .list_registered_summaries()
        .into_iter()
        .map(skill_summary_to_info)
        .collect::<Vec<_>>();

    skills.sort_by(|a, b| {
        a.display_name
            .cmp(&b.display_name)
            .then_with(|| a.id.cmp(&b.id))
    });
    Some(skills)
}

fn collect_skill_info(core: &hone_channels::HoneBotCore, skill_id: &str) -> Option<SkillInfo> {
    runtime(core)
        .list_registered_summaries()
        .into_iter()
        .find(|skill| skill.id == skill_id)
        .map(skill_summary_to_info)
}

fn collect_skill_detail(
    core: &hone_channels::HoneBotCore,
    skill_id: &str,
) -> Option<SkillDetailInfo> {
    let skill = runtime(core).load_registered_skill(skill_id).ok()?;
    Some(SkillDetailInfo {
        summary: skill_summary_to_info((&skill).into()),
        markdown: skill.body,
        detail_path: skill.skill_path.to_string_lossy().to_string(),
    })
}

fn skill_summary_to_info(skill: hone_tools::SkillSummary) -> SkillInfo {
    let disabled_reason = (!skill.enabled).then(|| "该技能已被管理员禁用，当前不会出现在 discover/search/list 中，也不能被 slash 或 skill_tool 调用。".to_string());
    SkillInfo {
        id: skill.id,
        display_name: skill.display_name,
        description: skill.description,
        when_to_use: skill.when_to_use,
        aliases: skill.aliases,
        allowed_tools: skill.allowed_tools,
        user_invocable: skill.user_invocable,
        context: skill.context.as_str().to_string(),
        loaded_from: skill.loaded_from,
        enabled: skill.enabled,
        disabled_reason,
        has_script: skill.script.is_some(),
        has_path_gate: !skill.paths.is_empty(),
        paths: skill.paths,
    }
}
