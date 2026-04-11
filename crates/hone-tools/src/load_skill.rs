//! LoadSkillTool — 兼容层
//!
//! 旧协议保留为 shim，底层统一走新的 SkillRuntime。

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};
use crate::skill_runtime::{SkillRuntime, SkillSummary};

#[derive(Debug, Clone)]
pub struct SkillMeta {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub when_to_use: Option<String>,
    pub aliases: Vec<String>,
    pub tools: Vec<String>,
    pub user_invocable: bool,
    pub context: String,
    pub script: Option<String>,
    pub loaded_from: String,
    pub paths: Vec<String>,
}

pub struct LoadSkillTool {
    skills_dirs: Vec<PathBuf>,
    registry_path: Option<PathBuf>,
}

impl LoadSkillTool {
    pub fn new(skills_dirs: Vec<PathBuf>) -> Self {
        Self {
            skills_dirs,
            registry_path: None,
        }
    }

    pub fn with_registry_path(mut self, registry_path: PathBuf) -> Self {
        self.registry_path = Some(registry_path);
        self
    }

    fn runtime(&self) -> SkillRuntime {
        let system_dir = self
            .skills_dirs
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("./skills"));
        let custom_dir = self
            .skills_dirs
            .get(1)
            .cloned()
            .unwrap_or_else(|| PathBuf::from("./data/custom_skills"));
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let runtime = SkillRuntime::new(system_dir, custom_dir, cwd);
        if let Some(path) = &self.registry_path {
            runtime.with_registry_path(path.clone())
        } else {
            runtime
        }
    }

    fn list_skills(&self) -> Vec<String> {
        self.runtime()
            .list_summaries()
            .into_iter()
            .map(|skill| skill.id)
            .collect()
    }

    pub fn list_skills_with_meta(&self) -> Vec<SkillMeta> {
        self.runtime()
            .list_summaries()
            .into_iter()
            .map(skill_summary_to_meta)
            .collect()
    }

    pub fn search_skills_with_meta(&self, query: &str, limit: usize) -> Vec<SkillMeta> {
        let runtime = self.runtime();
        let skills = if query.trim().is_empty() {
            runtime.list_summaries()
        } else {
            runtime.search(query, &[], limit)
        };
        let mut metas = skills
            .into_iter()
            .map(skill_summary_to_meta)
            .collect::<Vec<_>>();
        if limit > 0 && metas.len() > limit {
            metas.truncate(limit);
        }
        metas
    }
}

fn skill_summary_to_meta(skill: SkillSummary) -> SkillMeta {
    SkillMeta {
        name: skill.id,
        display_name: skill.display_name,
        description: skill.description,
        when_to_use: skill.when_to_use,
        aliases: skill.aliases,
        tools: skill.allowed_tools,
        user_invocable: skill.user_invocable,
        context: skill.context.as_str().to_string(),
        script: skill.script,
        loaded_from: skill.loaded_from,
        paths: skill.paths,
    }
}

#[async_trait]
impl Tool for LoadSkillTool {
    fn name(&self) -> &str {
        "load_skill"
    }

    fn description(&self) -> &str {
        "兼容旧协议：读取一个技能并返回完整 prompt。新的主路径请改用 skill_tool。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "skill_name".to_string(),
            param_type: "string".to_string(),
            description: "技能名称，对应 skills/<name>/SKILL.md 的目录名。".to_string(),
            required: true,
            r#enum: None,
            items: None,
        }]
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
                "error": "skill_name 不能为空",
                "available_skills": self.list_skills()
            }));
        }

        let runtime = self.runtime();
        match runtime.load_skill(skill_name, &[]) {
            Ok(skill) => {
                let session_id = std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
                let prompt = runtime.render_invocation_prompt(&skill, &session_id, None);
                Ok(serde_json::json!({
                    "success": true,
                    "skill_name": skill.id,
                    "skill_display_name": skill.display_name,
                    "skill_description": skill.description,
                    "when_to_use": skill.when_to_use,
                    "available_tools": skill.allowed_tools,
                    "execution_context": skill.context.as_str(),
                    "user_invocable": skill.user_invocable,
                    "script": skill.script,
                    "loaded_from": skill.source.as_str(),
                    "paths": skill.paths,
                    "prompt": prompt,
                    "reminder": "这是兼容层结果。新的技能执行请优先使用 skill_tool。"
                }))
            }
            Err(error) => Ok(serde_json::json!({
                "success": false,
                "error": error,
                "available_skills": self.list_skills()
            })),
        }
    }
}
