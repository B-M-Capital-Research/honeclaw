use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;

use hone_tools::LoadSkillTool;

use crate::state::AppState;
use crate::types::SkillInfo;

/// GET /api/skills — 扫描 skills/ 目录，返回所有技能的元数据和 guide
pub(crate) async fn handle_skills(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::to_value(collect_skill_infos(&state.core)).unwrap_or(serde_json::json!([])))
}

fn configured_skill_dirs(core: &hone_channels::HoneBotCore) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let system_dir = core
        .config
        .extra
        .get("skills_dir")
        .and_then(|v| v.as_str())
        .unwrap_or("./skills");
    dirs.push(PathBuf::from(system_dir));

    let custom_dir = std::env::var("HONE_DATA_DIR")
        .map(|root| PathBuf::from(root).join("custom_skills"))
        .unwrap_or_else(|_| PathBuf::from("./data/custom_skills"));
    if !dirs.iter().any(|dir| dir == &custom_dir) {
        dirs.push(custom_dir);
    }

    dirs
}

fn collect_skill_infos(core: &hone_channels::HoneBotCore) -> Vec<SkillInfo> {
    let tool = LoadSkillTool::new(configured_skill_dirs(core));
    let mut skills = Vec::new();

    for meta in tool.list_skills_with_meta() {
        skills.push(SkillInfo {
            guide: read_skill_guide(core, &meta.name).unwrap_or_default(),
            id: meta.name,
            display_name: meta.display_name,
            description: meta.description,
            aliases: meta.aliases,
            tools: meta.tools,
        });
    }

    skills.sort_by(|a, b| {
        a.display_name
            .cmp(&b.display_name)
            .then_with(|| a.id.cmp(&b.id))
    });
    skills
}

fn read_skill_guide(core: &hone_channels::HoneBotCore, skill_id: &str) -> Option<String> {
    for dir in configured_skill_dirs(core) {
        let Ok(content) = std::fs::read_to_string(dir.join(skill_id).join("SKILL.md")) else {
            continue;
        };
        let Some(rest) = content
            .strip_prefix("---\n")
            .or_else(|| content.strip_prefix("---\r\n"))
        else {
            continue;
        };
        let Some(end) = rest.find("\n---\n").or_else(|| rest.find("\n---\r\n")) else {
            continue;
        };
        let body_start = end + "\n---\n".len();
        return Some(rest.get(body_start..).unwrap_or("").trim().to_string());
    }
    None
}
