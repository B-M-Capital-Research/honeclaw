use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use hone_tools::SkillRuntime;

use crate::routes::json_error;
use crate::state::AppState;
use crate::types::{SkillDetailInfo, SkillInfo};

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

fn runtime(core: &hone_channels::HoneBotCore) -> SkillRuntime {
    SkillRuntime::new(
        core.configured_system_skills_dir(),
        core.configured_custom_skills_dir(),
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    )
}

fn collect_skill_infos(core: &hone_channels::HoneBotCore) -> Option<Vec<SkillInfo>> {
    let mut skills = runtime(core)
        .list_all_summaries()
        .into_iter()
        .map(|skill| SkillInfo {
            id: skill.id,
            display_name: skill.display_name,
            description: skill.description,
            when_to_use: skill.when_to_use,
            aliases: skill.aliases,
            allowed_tools: skill.allowed_tools,
            user_invocable: skill.user_invocable,
            context: skill.context.as_str().to_string(),
            loaded_from: skill.loaded_from,
            paths: skill.paths,
        })
        .collect::<Vec<_>>();

    skills.sort_by(|a, b| {
        a.display_name
            .cmp(&b.display_name)
            .then_with(|| a.id.cmp(&b.id))
    });
    Some(skills)
}

fn collect_skill_detail(
    core: &hone_channels::HoneBotCore,
    skill_id: &str,
) -> Option<SkillDetailInfo> {
    let skill = runtime(core).load_skill(skill_id, &[]).ok()?;
    Some(SkillDetailInfo {
        summary: SkillInfo {
            id: skill.id.clone(),
            display_name: skill.display_name.clone(),
            description: skill.description.clone(),
            when_to_use: skill.when_to_use.clone(),
            aliases: skill.aliases.clone(),
            allowed_tools: skill.allowed_tools.clone(),
            user_invocable: skill.user_invocable,
            context: skill.context.as_str().to_string(),
            loaded_from: skill.source.as_str().to_string(),
            paths: skill.paths.clone(),
        },
        markdown: skill.body,
        detail_path: skill.skill_path.to_string_lossy().to_string(),
    })
}
