//! SkillTool — Claude Code 风格技能执行入口。

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

use crate::base::{Tool, ToolParameter};
use crate::skill_runtime::{SkillRuntime, SkillStageConstraints};

const INVOKED_SKILLS_METADATA_KEY: &str = "skill_runtime.invoked_skills";

pub struct SkillTool {
    system_dir: PathBuf,
    custom_dir: PathBuf,
    registry_path: PathBuf,
}

impl SkillTool {
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

    async fn maybe_execute_script(
        &self,
        runtime: &SkillRuntime,
        skill: &crate::skill_runtime::SkillDefinition,
        args: &Value,
    ) -> Result<Option<Value>, String> {
        let should_execute = args
            .get("execute_script")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if !should_execute {
            return Ok(None);
        }

        let script_path = runtime
            .resolve_script_path(skill, args.get("script").and_then(|value| value.as_str()))?;
        let script_arguments = runtime.map_script_arguments(
            skill,
            args.get("script_arguments"),
            args.get("args").and_then(|value| value.as_str()),
        )?;

        let mut command = if let Some(shell) = skill.shell.as_deref() {
            let mut command = Command::new(shell);
            command.arg(&script_path);
            command
        } else {
            Command::new(&script_path)
        };

        command
            .args(&script_arguments)
            .current_dir(&skill.skill_dir)
            .env("HONE_SKILL_DIR", &skill.skill_dir)
            .env(
                "HONE_SESSION_ID",
                std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default(),
            );

        let output = command
            .output()
            .await
            .map_err(|err| format!("执行 skill script 失败: {err}"))?;
        Ok(Some(serde_json::json!({
            "script": script_path
                .strip_prefix(&skill.skill_dir)
                .unwrap_or(&script_path)
                .to_string_lossy()
                .replace('\\', "/"),
            "cwd": skill.skill_dir.to_string_lossy().to_string(),
            "shell": skill.shell.clone(),
            "arguments": script_arguments,
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Tool;
    use hone_memory::SessionStorage;
    use serde_json::Value;
    use std::fs;
    use std::sync::{Mutex, MutexGuard, OnceLock};
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

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    fn clear_test_env() {
        unsafe {
            std::env::remove_var("HONE_MCP_SESSION_ID");
            std::env::remove_var("HONE_DATA_DIR");
        }
    }

    #[tokio::test]
    async fn execute_runs_declared_skill_script() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_script");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("alpha");
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");

        fs::write(
            skill_dir.join("SKILL.md"),
            concat!(
                "---\n",
                "name: Alpha\n",
                "description: executes script\n",
                "arguments:\n",
                "  - ticker\n",
                "  - days\n",
                "script: scripts/run.sh\n",
                "shell: bash\n",
                "---\n\n",
                "body"
            ),
        )
        .expect("skill");
        fs::write(
            scripts_dir.join("run.sh"),
            concat!(
                "printf 'cwd=%s\\n' \"$PWD\"\n",
                "printf 'dir=%s\\n' \"$HONE_SKILL_DIR\"\n",
                "printf 'session=%s\\n' \"$HONE_SESSION_ID\"\n",
                "printf 'argv=%s,%s\\n' \"$1\" \"$2\"\n"
            ),
        )
        .expect("script");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        unsafe {
            std::env::set_var("HONE_MCP_SESSION_ID", "session-script-test");
        }
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true,
                "script_arguments": {
                    "days": 5,
                    "ticker": "AAPL"
                }
            }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(true));
        assert_eq!(
            result["script"],
            Value::String("scripts/run.sh".to_string())
        );
        assert_eq!(result["script_execution"]["success"], Value::Bool(true));
        let stdout = result["script_execution"]["stdout"]
            .as_str()
            .expect("stdout");
        let canonical_skill_dir = skill_dir.canonicalize().expect("canonical skill dir");
        assert!(stdout.contains(&format!("cwd={}", canonical_skill_dir.to_string_lossy())));
        assert!(stdout.contains(&format!("dir={}", skill_dir.to_string_lossy())));
        assert!(stdout.contains("session=session-script-test"));
        assert!(stdout.contains("argv=AAPL,5"));
        clear_test_env();
    }

    #[tokio::test]
    async fn execute_persists_invoked_skill_into_real_session_storage() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_persist");
        let system = root.join("system");
        let custom = root.join("custom");
        let data_dir = root.join("data");
        let sessions_dir = data_dir.join("sessions");
        let skill_dir = system.join("alpha");
        fs::create_dir_all(&skill_dir).expect("skill dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::create_dir_all(&sessions_dir).expect("sessions dir");

        fs::write(
            skill_dir.join("SKILL.md"),
            concat!(
                "---\n",
                "name: Alpha\n",
                "description: persist invoked skill\n",
                "allowed-tools:\n",
                "  - discover_skills\n",
                "  - data_fetch\n",
                "---\n\n",
                "Prompt body for ${HONE_SESSION_ID}"
            ),
        )
        .expect("skill");

        let storage = SessionStorage::new(&sessions_dir);
        let session_id = storage
            .create_session(Some("session-persist"), None, None)
            .expect("create session");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        unsafe {
            std::env::set_var("HONE_DATA_DIR", &data_dir);
            std::env::set_var("HONE_MCP_SESSION_ID", &session_id);
        }
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "args": "AAPL"
            }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(true));
        assert_eq!(result["skill_name"], Value::String("alpha".to_string()));
        assert_eq!(
            result["allowed_tools"],
            Value::Array(vec![
                Value::String("discover_skills".to_string()),
                Value::String("data_fetch".to_string()),
            ])
        );

        let session = storage
            .load_session(&session_id)
            .expect("load session")
            .expect("session exists");
        let invoked = session
            .metadata
            .get(INVOKED_SKILLS_METADATA_KEY)
            .and_then(|value| value.as_array())
            .expect("invoked skills array");
        assert_eq!(invoked.len(), 1);
        assert_eq!(
            invoked[0]
                .get("skill_name")
                .and_then(|value| value.as_str()),
            Some("alpha")
        );
        assert_eq!(
            invoked[0]
                .get("allowed_tools")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|value| value.as_str()),
            Some("discover_skills")
        );
        assert!(
            invoked[0]
                .get("prompt")
                .and_then(|value| value.as_str())
                .is_some_and(|value| value.contains("Prompt body for session-persist"))
        );

        clear_test_env();
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
                description: "可选。传递给 skill 的附加参数文本；若 execute_script=true 且未提供 script_arguments，会作为单个脚本参数传入。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "execute_script".to_string(),
                param_type: "boolean".to_string(),
                description: "可选。为 true 时执行 skill frontmatter 声明的 script。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "script".to_string(),
                param_type: "string".to_string(),
                description: "可选。覆盖 skill 默认 script，必须是 skill 目录内的相对路径。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "script_arguments".to_string(),
                param_type: "object".to_string(),
                description: "可选。脚本参数。可传对象（按 SKILL.md arguments 顺序映射）、数组或标量。".to_string(),
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
        let stage_constraints = SkillStageConstraints::from_mcp_env();
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
        match runtime.load_skill_for_stage(skill_name, &file_paths, &stage_constraints) {
            Ok(skill) => {
                let session_id = std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
                let prompt = runtime.render_invocation_prompt(
                    &skill,
                    &session_id,
                    args.get("args").and_then(|value| value.as_str()),
                );
                let script_execution =
                    match self.maybe_execute_script(&runtime, &skill, &args).await {
                        Ok(result) => result,
                        Err(error) => {
                            return Ok(serde_json::json!({
                                "success": false,
                                "error": error,
                                "skill_name": skill.id,
                                "script": skill.script,
                            }));
                        }
                    };
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
                    "script": skill.script,
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
                    "script": payload["script"],
                    "execution_context": payload["execution_context"],
                    "loaded_from": payload["loaded_from"],
                    "paths": payload["paths"],
                    "user_invocable": skill.user_invocable,
                    "hooks": skill.hooks,
                    "prompt": payload["prompt"],
                    "script_execution": script_execution,
                    "reminder": "技能已完整展开。请继续围绕用户原始任务执行，不要忘记真正要解决的问题。"
                }))
            }
            Err(error) => Ok(serde_json::json!({
                "success": false,
                "error": error,
                "available_skills": runtime
                    .list_summaries_for_stage(&stage_constraints)
                    .into_iter()
                    .map(|skill| skill.id)
                    .collect::<Vec<_>>()
            })),
        }
    }
}
