//! SkillTool — Claude Code 风格技能执行入口。

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};
use crate::skill_runtime::SkillRuntime;

const INVOKED_SKILLS_METADATA_KEY: &str = "skill_runtime.invoked_skills";

pub struct SkillTool {
    system_dir: PathBuf,
    custom_dir: PathBuf,
}

impl SkillTool {
    pub fn new(system_dir: PathBuf, custom_dir: PathBuf) -> Self {
        Self {
            system_dir,
            custom_dir,
        }
    }

    fn runtime(&self) -> SkillRuntime {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        SkillRuntime::new(self.system_dir.clone(), self.custom_dir.clone(), cwd)
    }

    fn persist_invoked_skill(&self, payload: &Value) -> hone_core::HoneResult<()> {
        let session_id = std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
        if session_id.trim().is_empty() {
            return Ok(());
        }
        let sessions_dir = resolve_sessions_dir()?;
        let storage = hone_memory::SessionStorage::new(sessions_dir);
        let session = match storage.load_session(&session_id)? {
            Some(session) => session,
            None => return Ok(()),
        };

        let mut skills = session
            .metadata
            .get(INVOKED_SKILLS_METADATA_KEY)
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        let skill_name = payload
            .get("skill_name")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        skills.retain(|entry| {
            entry.get("skill_name").and_then(|value| value.as_str()) != Some(skill_name.as_str())
        });
        skills.push(payload.clone());

        let mut metadata = HashMap::new();
        metadata.insert(
            INVOKED_SKILLS_METADATA_KEY.to_string(),
            Value::Array(skills),
        );
        let _ = storage.update_metadata(&session_id, metadata)?;
        Ok(())
    }
}

fn resolve_sessions_dir() -> hone_core::HoneResult<PathBuf> {
    if let Ok(root) = std::env::var("HONE_DATA_DIR") {
        return Ok(PathBuf::from(root).join("sessions"));
    }

    let config_path =
        std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let config = hone_core::config::HoneConfig::from_file(&config_path)?;
    Ok(PathBuf::from(config.storage.sessions_dir))
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill_tool"
    }

    fn description(&self) -> &str {
        "执行一个技能并返回完整的 skill prompt、可用工具和执行上下文。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "skill_name".to_string(),
                param_type: "string".to_string(),
                description: "要执行的技能 id。".to_string(),
                required: true,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "args".to_string(),
                param_type: "string".to_string(),
                description: "可选。传递给 skill 的附加参数文本。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "file_paths".to_string(),
                param_type: "array".to_string(),
                description: "可选。当前任务关联的文件路径，用于激活 paths 条件技能。".to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({ "type": "string" })),
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let skill_name = args
            .get("skill_name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if skill_name.is_empty() {
            return Ok(serde_json::json!({
                "success": false,
                "error": "skill_name 不能为空"
            }));
        }

        let runtime = self.runtime();
        let file_paths = args
            .get("file_paths")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        match runtime.load_skill(skill_name, &file_paths) {
            Ok(skill) => {
                let session_id = std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
                let prompt = runtime.render_prompt(
                    &skill,
                    &session_id,
                    args.get("args").and_then(|value| value.as_str()),
                );
                let payload = serde_json::json!({
                    "skill_name": skill.id,
                    "display_name": skill.display_name,
                    "path": skill.skill_path.to_string_lossy().to_string(),
                    "prompt": prompt,
                    "execution_context": skill.context.as_str(),
                    "allowed_tools": skill.allowed_tools,
                    "model": skill.model,
                    "effort": skill.effort,
                    "agent": skill.agent,
                    "loaded_from": skill.source.as_str(),
                    "paths": skill.paths,
                    "updated_at": hone_core::beijing_now_rfc3339(),
                });
                let _ = self.persist_invoked_skill(&payload);
                Ok(serde_json::json!({
                    "success": true,
                    "skill_name": skill.id,
                    "skill_display_name": skill.display_name,
                    "skill_description": skill.description,
                    "when_to_use": skill.when_to_use,
                    "allowed_tools": payload["allowed_tools"],
                    "model": payload["model"],
                    "effort": payload["effort"],
                    "agent": payload["agent"],
                    "execution_context": payload["execution_context"],
                    "loaded_from": payload["loaded_from"],
                    "paths": payload["paths"],
                    "user_invocable": skill.user_invocable,
                    "hooks": skill.hooks,
                    "prompt": payload["prompt"],
                    "reminder": "技能已完整展开。请继续围绕用户原始任务执行，不要忘记真正要解决的问题。"
                }))
            }
            Err(error) => Ok(serde_json::json!({
                "success": false,
                "error": error,
                "available_skills": runtime
                    .list_summaries()
                    .into_iter()
                    .map(|skill| skill.id)
                    .collect::<Vec<_>>()
            })),
        }
    }
}
