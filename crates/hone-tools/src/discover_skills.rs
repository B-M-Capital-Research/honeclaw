use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

use crate::base::{Tool, ToolParameter};
use crate::skill_runtime::{SkillRuntime, SkillStageConstraints};

pub struct DiscoverSkillsTool {
    system_dir: PathBuf,
    custom_dir: PathBuf,
    registry_path: PathBuf,
}

impl DiscoverSkillsTool {
    pub fn new(system_dir: PathBuf, custom_dir: PathBuf, registry_path: PathBuf) -> Self {
        Self {
            system_dir,
            custom_dir,
            registry_path,
        }
    }

    fn runtime(&self) -> SkillRuntime {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        SkillRuntime::new(self.system_dir.clone(), self.custom_dir.clone(), cwd)
            .with_registry_path(self.registry_path.clone())
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
        let stage_constraints = SkillStageConstraints::from_mcp_env();

        let skills = self
            .runtime()
            .search_for_stage(query, &file_paths, limit, &stage_constraints)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Tool;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), ts));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[tokio::test]
    async fn execute_returns_error_for_empty_query() {
        let root = make_temp_dir("hone_discover_skills_empty");
        let tool = DiscoverSkillsTool::new(
            root.join("system"),
            root.join("custom"),
            root.join("runtime").join("skill_registry.json"),
        );

        let result = tool
            .execute(serde_json::json!({ "query": "   " }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(false));
        assert_eq!(result["error"], Value::String("query 不能为空".to_string()));
    }

    #[tokio::test]
    async fn execute_searches_real_skill_files_with_path_filter_and_limit() {
        let root = make_temp_dir("hone_discover_skills_e2e");
        let system = root.join("system");
        let custom = root.join("custom");
        fs::create_dir_all(system.join("stock_alpha")).expect("alpha dir");
        fs::create_dir_all(system.join("macro_beta")).expect("beta dir");
        fs::create_dir_all(custom.join("portfolio_gamma")).expect("gamma dir");

        fs::write(
            system.join("stock_alpha/SKILL.md"),
            concat!(
                "---\n",
                "name: Stock Alpha\n",
                "description: analyze stock momentum and setup\n",
                "aliases:\n",
                "  - alpha stock\n",
                "allowed-tools:\n",
                "  - data_fetch\n",
                "paths:\n",
                "  - src/**/*.rs\n",
                "---\n\n",
                "Use this skill for stock analysis."
            ),
        )
        .expect("write alpha");
        fs::write(
            system.join("macro_beta/SKILL.md"),
            concat!(
                "---\n",
                "name: Macro Beta\n",
                "description: track macro events\n",
                "aliases:\n",
                "  - macro\n",
                "allowed-tools:\n",
                "  - web_search\n",
                "---\n\n",
                "Use this skill for macro monitoring."
            ),
        )
        .expect("write beta");
        fs::write(
            custom.join("portfolio_gamma/SKILL.md"),
            concat!(
                "---\n",
                "name: Portfolio Gamma\n",
                "description: review portfolio concentration risk\n",
                "aliases:\n",
                "  - risk review\n",
                "allowed-tools:\n",
                "  - portfolio_tool\n",
                "---\n\n",
                "Use this skill for portfolio review."
            ),
        )
        .expect("write gamma");

        let tool = DiscoverSkillsTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );

        let result = tool
            .execute(serde_json::json!({
                "query": "stock",
                "file_paths": ["src/main.rs", "   "],
                "limit": 1
            }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(true));
        assert_eq!(result["count"], Value::Number(1.into()));
        assert_eq!(result["query"], Value::String("stock".to_string()));
        assert_eq!(
            result["skills"][0]["id"],
            Value::String("stock_alpha".to_string())
        );
        assert_eq!(
            result["skills"][0]["loaded_from"],
            Value::String("system".to_string())
        );
        assert_eq!(
            result["skills"][0]["allowed_tools"][0],
            Value::String("data_fetch".to_string())
        );
    }
}
