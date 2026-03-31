use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};
use crate::skill_runtime::SkillRuntime;

pub struct DiscoverSkillsTool {
    system_dir: PathBuf,
    custom_dir: PathBuf,
}

impl DiscoverSkillsTool {
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
}

#[async_trait]
impl Tool for DiscoverSkillsTool {
    fn name(&self) -> &str {
        "discover_skills"
    }

    fn description(&self) -> &str {
        "根据当前任务检索相关技能，返回精简 skill 索引。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "当前任务描述、目标或用户问题。".to_string(),
                required: true,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "file_paths".to_string(),
                param_type: "array".to_string(),
                description: "可选。与当前任务相关的文件路径，用于激活 paths 条件技能。"
                    .to_string(),
                required: false,
                r#enum: None,
                items: Some(serde_json::json!({ "type": "string" })),
            },
            ToolParameter {
                name: "limit".to_string(),
                param_type: "number".to_string(),
                description: "最多返回多少条技能，默认 6。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let query = args
            .get("query")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if query.is_empty() {
            return Ok(serde_json::json!({
                "success": false,
                "error": "query 不能为空"
            }));
        }

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
        let limit = args
            .get("limit")
            .and_then(|value| value.as_u64())
            .unwrap_or(6) as usize;

        let skills = self
            .runtime()
            .search(query, &file_paths, limit)
            .into_iter()
            .map(|skill| {
                serde_json::json!({
                    "id": skill.id,
                    "display_name": skill.display_name,
                    "description": skill.description,
                    "when_to_use": skill.when_to_use,
                    "allowed_tools": skill.allowed_tools,
                    "aliases": skill.aliases,
                    "user_invocable": skill.user_invocable,
                    "execution_context": skill.context.as_str(),
                    "loaded_from": skill.loaded_from,
                    "paths": skill.paths,
                })
            })
            .collect::<Vec<_>>();

        Ok(serde_json::json!({
            "success": true,
            "query": query,
            "skills": skills,
            "count": skills.len()
        }))
    }
}
