//! SkillTool — Claude Code 风格技能执行入口。

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

use crate::base::{Tool, ToolParameter};
use crate::skill_runtime::{SkillRuntime, SkillStageConstraints};

const INVOKED_SKILLS_METADATA_KEY: &str = "skill_runtime.invoked_skills";
const SUPPORTED_IMAGE_ARTIFACT_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];
const SUPPORTED_DOCUMENT_ARTIFACT_EXTENSIONS: &[&str] = &["pdf"];
const SKILL_SCRIPT_STDERR_CHARS: usize = 1000;
const MAX_SKILL_SCRIPT_ARGUMENTS: usize = 64;
const MAX_SKILL_SCRIPT_ARGUMENT_BYTES: usize = 256 * 1024;
const SKILL_SCRIPT_TIMEOUT: Duration = Duration::from_secs(120);

enum SkillScriptExecutionError {
    NotStarted(String),
    StateUncertain(String),
}

impl SkillScriptExecutionError {
    fn message(&self) -> &str {
        match self {
            Self::NotStarted(message) | Self::StateUncertain(message) => message,
        }
    }

    fn side_effect_status(&self) -> &'static str {
        match self {
            Self::NotStarted(_) => "not_started",
            Self::StateUncertain(_) => "uncertain",
        }
    }
}

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

    async fn persist_invoked_skill(&self, payload: &Value) -> hone_core::HoneResult<()> {
        let session_id = std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
        if session_id.trim().is_empty() {
            return Ok(());
        }
        let sessions_dir = resolve_sessions_dir()?;
        let storage = hone_memory::SessionStorage::new(sessions_dir).await;
        let session = match storage.load_session(&session_id).await? {
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
        let _ = storage.update_metadata(&session_id, metadata).await?;
        Ok(())
    }

    async fn maybe_execute_script(
        &self,
        runtime: &SkillRuntime,
        skill: &crate::skill_runtime::SkillDefinition,
        args: &Value,
    ) -> Result<Option<Value>, SkillScriptExecutionError> {
        let should_execute = args
            .get("execute_script")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if !should_execute {
            return Ok(None);
        }

        let script_path = runtime
            .resolve_script_path(skill, args.get("script").and_then(|value| value.as_str()))
            .map_err(SkillScriptExecutionError::NotStarted)?;
        let script_arguments = match args.get("script_payload") {
            Some(payload) => {
                if args.get("script_arguments").is_some() || args.get("args").is_some() {
                    return Err(SkillScriptExecutionError::NotStarted(
                        "script_payload 不能与 script_arguments 或 args 同时使用".to_string(),
                    ));
                }
                if !payload.is_object() {
                    return Err(SkillScriptExecutionError::NotStarted(
                        "script_payload 必须是结构化 JSON 对象".to_string(),
                    ));
                }
                vec![serde_json::to_string(payload).map_err(|error| {
                    SkillScriptExecutionError::NotStarted(format!(
                        "序列化 skill 脚本 payload 失败: {error}"
                    ))
                })?]
            }
            None => runtime
                .map_script_arguments(
                    skill,
                    args.get("script_arguments"),
                    args.get("args").and_then(|value| value.as_str()),
                )
                .map_err(SkillScriptExecutionError::NotStarted)?,
        };
        validate_script_argument_budget(&script_arguments)
            .map_err(SkillScriptExecutionError::NotStarted)?;

        let mut command = if let Some(shell) = skill.shell.as_deref() {
            let mut command = Command::new(shell);
            command.arg(&script_path);
            command
        } else {
            Command::new(&script_path)
        };

        // Skill scripts are trusted repository code, but they must not inherit database,
        // object-store, channel, or LLM credentials from the long-running server.
        command.env_clear();
        for name in ["PATH", "HOME", "LANG", "LC_ALL", "TMPDIR"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        command
            .args(&script_arguments)
            .current_dir(&skill.skill_dir)
            .env("HONE_SKILL_DIR", &skill.skill_dir)
            .env(
                "HONE_SESSION_ID",
                std::env::var("HONE_MCP_SESSION_ID").unwrap_or_default(),
            );
        if let Ok(working_directory) = std::env::var("HONE_MCP_WORKING_DIRECTORY") {
            command.env("HONE_SKILL_OUTPUT_DIR", working_directory);
        }
        if let Ok(gen_images_dir) = resolve_gen_images_dir() {
            command.env("HONE_GEN_IMAGES_DIR", gen_images_dir);
        }
        command.kill_on_drop(true);

        let output = tokio::time::timeout(SKILL_SCRIPT_TIMEOUT, command.output())
            .await
            .map_err(|_| {
                SkillScriptExecutionError::StateUncertain(format!(
                    "skill script 执行超时（>{} 秒），子进程已终止",
                    SKILL_SCRIPT_TIMEOUT.as_secs()
                ))
            })?
            .map_err(|err| {
                let message = format!("执行 skill script 失败: {err}");
                match err.kind() {
                    ErrorKind::NotFound | ErrorKind::PermissionDenied | ErrorKind::InvalidInput => {
                        SkillScriptExecutionError::NotStarted(message)
                    }
                    _ => SkillScriptExecutionError::StateUncertain(message),
                }
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stderr_preview = sanitize_skill_script_stderr(&stderr);
        if !output.status.success() {
            return Err(SkillScriptExecutionError::StateUncertain(format!(
                "skill script 退出失败: exit_code={:?}, stderr={}",
                output.status.code(),
                stderr_preview
            )));
        }

        let structured_output = parse_structured_script_stdout(&stdout)
            .map_err(SkillScriptExecutionError::StateUncertain)?;
        let render_success = structured_output
            .get("success")
            .and_then(|value| value.as_bool())
            .ok_or_else(|| {
                SkillScriptExecutionError::StateUncertain(
                    "skill script stdout JSON 必须包含布尔字段 success".to_string(),
                )
            })?;
        let artifacts = validate_script_artifacts(&structured_output, skill)
            .map_err(SkillScriptExecutionError::StateUncertain)?;
        if render_success && artifacts.is_empty() {
            return Err(SkillScriptExecutionError::StateUncertain(
                "skill script success=true 时必须返回至少一个有效 artifact".to_string(),
            ));
        }

        Ok(Some(serde_json::json!({
            "script": script_path
                .strip_prefix(&skill.skill_dir)
                .unwrap_or(&script_path)
                .to_string_lossy()
                .replace('\\', "/"),
            "cwd": skill.skill_dir.to_string_lossy().to_string(),
            "shell": skill.shell.clone(),
            "arguments": script_arguments,
            "process_success": true,
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr_preview,
            "render_success": render_success,
            "structured_output": structured_output,
            "artifacts": artifacts,
            "summary": structured_output.get("summary").cloned().unwrap_or(Value::Null),
            "warnings": structured_output
                .get("warnings")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
            "error": structured_output.get("error").cloned().unwrap_or(Value::Null),
            "fallback_message": structured_output
                .get("fallback_message")
                .cloned()
                .unwrap_or(Value::Null),
        })))
    }
}

fn validate_script_argument_budget(arguments: &[String]) -> Result<(), String> {
    if arguments.len() > MAX_SKILL_SCRIPT_ARGUMENTS {
        return Err(format!(
            "skill script 参数数量超过限制（>{MAX_SKILL_SCRIPT_ARGUMENTS}）"
        ));
    }
    let total_bytes = arguments
        .iter()
        .try_fold(0usize, |total, argument| {
            total.checked_add(argument.len()).ok_or(())
        })
        .unwrap_or(usize::MAX);
    if total_bytes > MAX_SKILL_SCRIPT_ARGUMENT_BYTES {
        return Err(format!(
            "skill script 参数总大小超过限制（>{} KiB）",
            MAX_SKILL_SCRIPT_ARGUMENT_BYTES / 1024
        ));
    }
    Ok(())
}

fn sanitize_skill_script_stderr(stderr: &str) -> String {
    let redacted = redact_skill_script_stderr_secrets(stderr.trim());
    if redacted.chars().count() <= SKILL_SCRIPT_STDERR_CHARS {
        return redacted;
    }
    redacted
        .chars()
        .take(SKILL_SCRIPT_STDERR_CHARS)
        .collect::<String>()
        + "..."
}

fn redact_skill_script_stderr_secrets(text: &str) -> String {
    let mut output = redact_url_userinfo(text);
    for marker in ["Bearer ", "bearer ", "Basic ", "basic "] {
        output = redact_skill_script_marker_value(&output, marker);
    }
    for key in SENSITIVE_SKILL_SCRIPT_STDERR_KEYS {
        output = redact_skill_script_marker_value(&output, &format!("{key}="));
        output = redact_skill_script_marker_value(&output, &format!("{key}:"));
        output = redact_skill_script_json_string_field(&output, key);
    }
    for key in ["authorization", "Authorization"] {
        output = redact_skill_script_json_string_field(&output, key);
    }
    output
}

const SENSITIVE_SKILL_SCRIPT_STDERR_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "api_key",
    "apiKey",
    "apikey",
    "client_secret",
    "clientSecret",
    "refresh_token",
    "refreshToken",
    "id_token",
    "idToken",
    "session_token",
    "sessionToken",
    "bot_token",
    "botToken",
    "OPENROUTER_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "TAVILY_API_KEY",
    "FMP_API_KEY",
    "HONE_CLOUD_API_KEY",
    "token",
    "secret",
    "password",
    "X-API-Key",
    "x-api-key",
];

fn redact_url_userinfo(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find("://") {
        let authority_start = index + 3;
        let authority = &remaining[authority_start..];
        let authority_end = authority
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch.is_whitespace() || matches!(ch, '/' | '?' | '#' | ')')).then_some(idx)
            })
            .unwrap_or(authority.len());
        let authority_slice = &authority[..authority_end];
        if let Some(at_index) = authority_slice.rfind('@') {
            output.push_str(&remaining[..authority_start]);
            output.push_str("<redacted>@");
            remaining = &remaining[authority_start + at_index + 1..];
        } else {
            output.push_str(&remaining[..authority_start]);
            remaining = &remaining[authority_start..];
        }
    }
    output.push_str(remaining);
    output
}

fn redact_skill_script_marker_value(text: &str, marker: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(marker) {
        let value_start = index + marker.len();
        output.push_str(&remaining[..value_start]);
        let leading_whitespace = remaining[value_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        output.push_str(&remaining[value_start..value_start + leading_whitespace]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start + leading_whitespace..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch == '&'
                    || ch == ')'
                    || ch == ','
                    || ch == ';'
                    || ch == '"'
                    || ch == '\''
                    || ch == '}'
                    || ch == ']'
                    || ch.is_whitespace())
                .then_some(idx)
            })
            .unwrap_or(remaining[value_start + leading_whitespace..].len());
        remaining = &remaining[value_start + leading_whitespace + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_skill_script_json_string_field(text: &str, key: &str) -> String {
    let key_marker = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&key_marker) {
        let after_key = index + key_marker.len();
        let tail = &remaining[after_key..];
        let Some((colon_offset, _)) = tail.char_indices().find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !tail[colon_offset..].starts_with(':') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let after_colon = &tail[colon_offset + 1..];
        let Some((quote_offset, _)) = after_colon
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !after_colon[quote_offset..].starts_with('"') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let value_start = after_key + colon_offset + 1 + quote_offset + 1;
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| (ch == '"').then_some(idx))
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Tool;
    use crate::test_support::{assert_text_contains_all, assert_text_contains_none};
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
            std::env::remove_var("HONE_MCP_WORKING_DIRECTORY");
            std::env::remove_var("HONE_DATA_DIR");
            std::env::remove_var("HONE_AGENT_SANDBOX_DIR");
            std::env::remove_var("HONE_CONFIG_PATH");
            std::env::remove_var("HONE_GEN_IMAGES_DIR");
        }
    }

    #[tokio::test]
    async fn execute_accepts_pdf_document_artifact_in_actor_working_directory() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_pdf_artifact");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("earnings-research");
        let scripts_dir = skill_dir.join("scripts");
        let working_directory = root.join("actor-workspace");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::create_dir_all(&working_directory).expect("working dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: earnings-research\ndescription: renders earnings PDF\nshell: bash\n---\n\nbody",
        )
        .expect("skill");
        fs::write(
            scripts_dir.join("render.sh"),
            concat!(
                "printf '%s' '%PDF-fake' > \"$HONE_SKILL_OUTPUT_DIR/report.pdf\"\n",
                "printf '{\"success\":true,\"summary\":\"ok\",\"artifacts\":[{\"kind\":\"document\",\"path\":\"%s/report.pdf\",\"mime\":\"application/pdf\"}],\"warnings\":[]}' \"$HONE_SKILL_OUTPUT_DIR\"\n"
            ),
        )
        .expect("script");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        unsafe {
            std::env::set_var("HONE_MCP_WORKING_DIRECTORY", &working_directory);
        }
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "earnings-research",
                "execute_script": true,
                "script": "scripts/render.sh"
            }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(true));
        assert!(result.get("prompt").is_none());
        assert!(result["script_execution"].get("arguments").is_none());
        assert!(result["script_execution"].get("stdout").is_none());
        assert_eq!(result["artifacts"][0]["kind"], "document");
        assert_eq!(result["artifacts"][0]["mime"], "application/pdf");
        assert!(working_directory.join("report.pdf").is_file());
        clear_test_env();
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn skill_script_stderr_preview_redacts_common_credentials() {
        let stderr = r#"failed https://user:password@api.test/path?api_key=abc&token=tok auth=Bearer xyz apiKey: header-secret; OPENROUTER_API_KEY=env-secret X-API-Key: gateway-secret Authorization: Basic basic-secret authorization: bearer lower-secret {"secret": "json-secret","client_secret":"json-client","authorization":"Basic json-basic"}"#;
        let detail = sanitize_skill_script_stderr(stderr);

        assert_text_contains_all(
            &detail,
            &[
                "https://<redacted>@api.test/path",
                "api_key=<redacted>",
                "token=<redacted>",
                "Bearer <redacted>",
                "apiKey: <redacted>;",
                "OPENROUTER_API_KEY=<redacted>",
                "X-API-Key: <redacted>",
                "Basic <redacted>",
                "bearer <redacted>",
                "\"secret\": \"<redacted>\"",
                "\"client_secret\":\"<redacted>\"",
                "\"authorization\":\"<redacted>\"",
            ],
        );
        assert_text_contains_none(
            &detail,
            &[
                "abc",
                "password",
                "=tok",
                "xyz",
                "header-secret",
                "json-secret",
                "env-secret",
                "gateway-secret",
                "basic-secret",
                "json-client",
                "json-basic",
            ],
        );
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
                "printf '{\"success\":true,\"summary\":\"ok\",\"artifacts\":[{\"kind\":\"image\",\"path\":\"%s/test.png\",\"mime\":\"image/png\"}],\"warnings\":[],\"debug\":{\"cwd\":\"%s\",\"dir\":\"%s\",\"session\":\"%s\",\"argv\":[\"%s\",\"%s\"]}}' \\\n",
                "  \"$HONE_SKILL_DIR\" \"$PWD\" \"$HONE_SKILL_DIR\" \"$HONE_SESSION_ID\" \"$1\" \"$2\"\n"
            ),
        )
        .expect("script");
        fs::write(skill_dir.join("test.png"), b"png").expect("test png");

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
        assert_eq!(result["render_success"], Value::Bool(true));
        assert_eq!(
            result["script_execution"]["process_success"],
            Value::Bool(true)
        );
        let canonical_skill_dir = skill_dir.canonicalize().expect("canonical skill dir");
        assert_eq!(
            result["artifacts"][0]["path"],
            Value::String(
                canonical_skill_dir
                    .join("test.png")
                    .to_string_lossy()
                    .to_string()
            )
        );
        let debug = &result["script_execution"]["structured_output"]["debug"];
        assert_eq!(
            debug["cwd"],
            Value::String(canonical_skill_dir.to_string_lossy().to_string())
        );
        assert_eq!(
            debug["dir"],
            Value::String(skill_dir.to_string_lossy().to_string())
        );
        assert_eq!(
            debug["session"],
            Value::String("session-script-test".to_string())
        );
        assert_eq!(
            debug["argv"],
            Value::Array(vec![
                Value::String("AAPL".to_string()),
                Value::String("5".to_string()),
            ])
        );
        clear_test_env();
    }

    #[tokio::test]
    async fn execute_returns_compact_actionable_validation_failure() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_render_rejection");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("alpha");
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                "---\nname: Alpha\ndescription: validates report\nscript: scripts/run.sh\nshell: bash\n---\n\n{}",
                "long prompt body ".repeat(4_000)
            ),
        )
        .expect("skill");
        fs::write(
            scripts_dir.join("run.sh"),
            "printf '%s' '{\"success\":false,\"error\":\"fix field A and field B\",\"fallback_message\":\"retry\",\"artifacts\":[],\"warnings\":[]}'\n",
        )
        .expect("script");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true
            }))
            .await
            .expect("validation failure is a determinate tool result");

        assert_eq!(result["success"], Value::Bool(false));
        assert_eq!(result["side_effect_status"], "not_started");
        assert_eq!(result["render_success"], Value::Bool(false));
        assert_eq!(result["render_error"], "fix field A and field B");
        assert!(result.get("prompt").is_none());
        assert!(result["script_execution"].get("arguments").is_none());
        assert!(result["script_execution"].get("stdout").is_none());
        assert!(serde_json::to_vec(&result).expect("serialize").len() < 4_000);
        clear_test_env();
    }

    #[tokio::test]
    async fn execute_does_not_inherit_server_secrets() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_clean_env");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("clean-env");
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            concat!(
                "---\n",
                "name: Clean Env\n",
                "description: verifies child environment\n",
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
                "printf '{\"success\":true,\"summary\":\"%s\",\"artifacts\":[{\"kind\":\"image\",\"path\":\"%s/test.png\",\"mime\":\"image/png\"}],\"warnings\":[]}' ",
                "\"${DATABASE_URL:-absent}\" \"$HONE_SKILL_DIR\"\n"
            ),
        )
        .expect("script");
        fs::write(skill_dir.join("test.png"), b"png").expect("artifact");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://must-not-leak");
        }
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "clean-env",
                "execute_script": true
            }))
            .await
            .expect("execute");

        assert_eq!(result["script_execution"]["summary"], "absent");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
        std::fs::remove_dir_all(root).ok();
    }

    #[tokio::test]
    async fn execute_serializes_structured_script_payload_without_model_escaping() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_structured_payload");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("alpha");
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");

        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Alpha\ndescription: structured payload\nscript: scripts/run.sh\nshell: bash\n---\n\nbody",
        )
        .expect("skill");
        fs::write(
            scripts_dir.join("run.sh"),
            concat!(
                "printf '%s' \"$1\" > \"$HONE_SKILL_DIR/payload.json\"\n",
                "printf '{\"success\":true,\"summary\":\"ok\",\"artifacts\":[{\"kind\":\"image\",\"path\":\"%s/test.png\",\"mime\":\"image/png\"}],\"warnings\":[]}' \"$HONE_SKILL_DIR\"\n"
            ),
        )
        .expect("script");
        fs::write(skill_dir.join("test.png"), b"png").expect("test png");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        let report_spec = serde_json::json!({
            "company": "AAOI",
            "report_markdown": "机构称\"买入\"，目标价 220 美元。",
            "preview_audit": {
                "institution_views": [{"institution": "Rosenblatt Securities"}]
            }
        });
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true,
                "script_payload": report_spec.clone()
            }))
            .await
            .expect("execute structured payload");

        assert_eq!(result["success"], Value::Bool(true));
        let persisted_payload: Value = serde_json::from_str(
            &fs::read_to_string(skill_dir.join("payload.json")).expect("payload file"),
        )
        .expect("runtime-generated JSON payload");
        assert_eq!(persisted_payload, report_spec);
        assert_eq!(
            result["validated_report_markdown"],
            report_spec["report_markdown"]
        );
        clear_test_env();
        std::fs::remove_dir_all(root).ok();
    }

    #[tokio::test]
    async fn execute_failed_skill_script_redacts_stderr() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_script_failure");
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
                "description: fails script\n",
                "script: scripts/run.sh\n",
                "shell: bash\n",
                "---\n\n",
                "body"
            ),
        )
        .expect("skill");
        fs::write(
            scripts_dir.join("run.sh"),
            "printf 'token=tok api_key=abc auth=Bearer xyz' >&2\nexit 2\n",
        )
        .expect("script");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true
            }))
            .await
            .expect("script failure should return structured tool error");

        assert_eq!(result["success"], Value::Bool(false));
        assert_eq!(result["side_effect_status"], "uncertain");
        let error = result["error"].as_str().expect("error message");
        assert_text_contains_all(
            error,
            &[
                "exit_code=Some(2)",
                "token=<redacted>",
                "api_key=<redacted>",
                "Bearer <redacted>",
            ],
        );
        assert_text_contains_none(error, &["token=tok", "api_key=abc", "xyz"]);
        clear_test_env();
    }

    #[tokio::test]
    async fn execute_marks_argument_validation_failure_as_not_started() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_preflight_failure");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("alpha");
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Alpha\ndescription: validates arguments\nscript: scripts/run.sh\nshell: bash\n---\n\nbody",
        )
        .expect("skill");
        fs::write(scripts_dir.join("run.sh"), "exit 99\n").expect("script");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true,
                "script_arguments": {"report": "draft"}
            }))
            .await
            .expect("preflight failure should be structured");

        assert_eq!(result["success"], Value::Bool(false));
        assert_eq!(result["side_effect_status"], "not_started");
        assert!(
            result["error"]
                .as_str()
                .is_some_and(|error| error.contains("arguments 顺序"))
        );
        clear_test_env();
        std::fs::remove_dir_all(root).ok();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn execute_marks_non_executable_skill_script_as_not_started() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_non_executable");
        let system = root.join("system");
        let custom = root.join("custom");
        let skill_dir = system.join("alpha");
        let scripts_dir = skill_dir.join("scripts");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: Alpha\ndescription: validates executable mode\nscript: scripts/run.sh\n---\n\nbody",
        )
        .expect("skill");
        let script = scripts_dir.join("run.sh");
        fs::write(&script, "#!/usr/bin/env bash\nexit 0\n").expect("script");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o644)).expect("permissions");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true
            }))
            .await
            .expect("spawn failure should be structured");

        assert_eq!(result["success"], Value::Bool(false));
        assert_eq!(result["side_effect_status"], "not_started");
        assert!(
            result["error"]
                .as_str()
                .is_some_and(|error| error.contains("Permission denied"))
        );
        clear_test_env();
        std::fs::remove_dir_all(root).ok();
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

        let storage = SessionStorage::new(&sessions_dir).await;
        let session_id = storage
            .create_session(Some("session-persist"), None, None)
            .await
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
            .await
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

    #[tokio::test]
    async fn execute_rejects_artifacts_outside_allowed_roots() {
        let _guard = env_lock();
        clear_test_env();
        let root = make_temp_dir("hone_skill_tool_artifact_roots");
        let system = root.join("system");
        let custom = root.join("custom");
        let data_dir = root.join("data");
        let skill_dir = system.join("alpha");
        let scripts_dir = skill_dir.join("scripts");
        let outside_dir = root.join("outside");
        fs::create_dir_all(&scripts_dir).expect("scripts dir");
        fs::create_dir_all(&custom).expect("custom dir");
        fs::create_dir_all(&outside_dir).expect("outside dir");
        let outside_png = outside_dir.join("outside.png");
        fs::write(&outside_png, b"png").expect("outside png");

        fs::write(
            skill_dir.join("SKILL.md"),
            concat!(
                "---\n",
                "name: Alpha\n",
                "description: rejects outside artifacts\n",
                "script: scripts/run.sh\n",
                "shell: bash\n",
                "---\n\n",
                "body"
            ),
        )
        .expect("skill");
        fs::write(
            scripts_dir.join("run.sh"),
            format!(
                "printf '%s' '{{\"success\":true,\"summary\":\"oops\",\"artifacts\":[{{\"kind\":\"image\",\"path\":\"{}\",\"mime\":\"image/png\"}}],\"warnings\":[]}}'\n",
                outside_png.to_string_lossy()
            ),
        )
        .expect("script");

        let tool = SkillTool::new(
            system,
            custom,
            root.join("runtime").join("skill_registry.json"),
        );
        unsafe {
            std::env::set_var("HONE_DATA_DIR", &data_dir);
            std::env::set_var("HONE_MCP_SESSION_ID", "session-outside-artifact");
        }

        let result = tool
            .execute(serde_json::json!({
                "skill_name": "alpha",
                "execute_script": true,
            }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(false));
        assert!(
            result["error"]
                .as_str()
                .is_some_and(|value| value.contains("artifact.path 不在允许目录内"))
        );
        clear_test_env();
    }

    #[test]
    fn skill_script_argument_budget_is_bounded() {
        assert!(
            validate_script_argument_budget(&vec!["arg".to_string(); MAX_SKILL_SCRIPT_ARGUMENTS])
                .is_ok()
        );
        assert!(
            validate_script_argument_budget(&vec![
                "arg".to_string();
                MAX_SKILL_SCRIPT_ARGUMENTS + 1
            ])
            .is_err()
        );
        assert!(
            validate_script_argument_budget(&[String::from_utf8(vec![
                b'x';
                MAX_SKILL_SCRIPT_ARGUMENT_BYTES
                    + 1
            ])
            .expect("ascii")])
            .is_err()
        );
    }

    #[test]
    fn chart_visualization_rejects_oversized_bins_before_rendering() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let script = repo_root.join("skills/chart_visualization/scripts/render_chart.py");
        let output = std::process::Command::new("python3")
            .arg(script)
            .arg(
                serde_json::json!({
                    "chart_type": "histogram",
                    "title": "bounded histogram",
                    "bins": 1_000_000,
                    "series": [{"name": "values", "values": [1, 2, 3]}]
                })
                .to_string(),
            )
            .output()
            .expect("run chart renderer");
        assert!(output.status.success(), "{:?}", output.status);
        let payload: Value =
            serde_json::from_slice(&output.stdout).expect("structured renderer output");
        assert_eq!(payload["success"], Value::Bool(false));
        assert!(
            payload["error"]
                .as_str()
                .is_some_and(|error| error.contains("between 1 and 200"))
        );
    }

    #[tokio::test]
    async fn chart_visualization_renderer_smoke_writes_png_when_matplotlib_is_available() {
        let _guard = env_lock();
        clear_test_env();

        let probe = std::process::Command::new("python3")
            .arg("-c")
            .arg("import matplotlib")
            .status()
            .expect("probe matplotlib");
        if !probe.success() {
            eprintln!("skip chart_visualization smoke test because matplotlib is unavailable");
            return;
        }

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let system = repo_root.join("skills");
        let custom = make_temp_dir("hone_skill_tool_chart_custom");
        let data_dir = make_temp_dir("hone_skill_tool_chart_data");

        let tool = SkillTool::new(
            system,
            custom,
            data_dir.join("runtime").join("skill_registry.json"),
        );
        unsafe {
            std::env::set_var("HONE_DATA_DIR", &data_dir);
            std::env::set_var("HONE_MCP_SESSION_ID", "chart-render-smoke");
        }

        let result = tool
            .execute(serde_json::json!({
                "skill_name": "chart_visualization",
                "execute_script": true,
                "script_arguments": {
                    "spec_json": serde_json::json!({
                        "chart_type": "line",
                        "title": "Revenue Trend",
                        "x_values": ["2023Q1", "2023Q2", "2023Q3"],
                        "series": [
                            {
                                "name": "Revenue",
                                "values": [100, 120, 135]
                            }
                        ],
                        "output_name": "revenue-trend"
                    }).to_string()
                }
            }))
            .await
            .expect("execute");

        assert_eq!(result["success"], Value::Bool(true));
        assert_eq!(result["render_success"], Value::Bool(true));
        let path = result["artifacts"][0]["path"]
            .as_str()
            .expect("artifact path");
        assert!(path.ends_with(".png"));
        assert!(PathBuf::from(path).exists());
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

fn resolve_gen_images_dir() -> hone_core::HoneResult<PathBuf> {
    if let Ok(root) = std::env::var("HONE_DATA_DIR") {
        return Ok(PathBuf::from(root).join("gen_images"));
    }

    let config_path =
        std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let config = hone_core::config::HoneConfig::from_file(&config_path)?;
    Ok(PathBuf::from(config.storage.gen_images_dir))
}

fn parse_structured_script_stdout(stdout: &str) -> Result<Value, String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err("skill script stdout 为空，必须输出 JSON".to_string());
    }

    let parsed: Value = serde_json::from_str(trimmed)
        .map_err(|err| format!("skill script stdout JSON 解析失败: {err}"))?;
    if !parsed.is_object() {
        return Err("skill script stdout 必须是 JSON 对象".to_string());
    }
    Ok(parsed)
}

fn validate_script_artifacts(
    structured_output: &Value,
    skill: &crate::skill_runtime::SkillDefinition,
) -> Result<Vec<Value>, String> {
    let artifacts = structured_output
        .get("artifacts")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    if artifacts.is_empty() {
        return Ok(Vec::new());
    }

    let allowed_roots = artifact_allowed_roots(&skill.skill_dir)?;
    let mut validated = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let kind = artifact
            .get("kind")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if kind != "image" && kind != "document" {
            return Err(format!("仅支持 image/document artifact，收到 kind={kind}"));
        }

        let path = artifact
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "artifact.path 缺失".to_string())?
            .trim();
        let artifact_path = PathBuf::from(path);
        if !artifact_path.is_absolute() {
            return Err(format!("artifact.path 必须是绝对路径: {path}"));
        }
        let canonical_path = std::fs::canonicalize(&artifact_path)
            .map_err(|err| format!("artifact.path 无法解析或文件不存在: {path} ({err})"))?;

        let ext = canonical_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let extension_supported = if kind == "image" {
            SUPPORTED_IMAGE_ARTIFACT_EXTENSIONS.contains(&ext.as_str())
        } else {
            SUPPORTED_DOCUMENT_ARTIFACT_EXTENSIONS.contains(&ext.as_str())
        };
        if !extension_supported {
            return Err(format!(
                "artifact.path 扩展名与 kind={kind} 不匹配: {}",
                canonical_path.display()
            ));
        }

        if !allowed_roots
            .iter()
            .any(|root| canonical_path.starts_with(root))
        {
            return Err(format!(
                "artifact.path 不在允许目录内: {}",
                canonical_path.display()
            ));
        }

        let mime = artifact
            .get("mime")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| match ext.as_str() {
                "png" => "image/png".to_string(),
                "jpg" | "jpeg" => "image/jpeg".to_string(),
                "webp" => "image/webp".to_string(),
                "gif" => "image/gif".to_string(),
                "pdf" => "application/pdf".to_string(),
                _ => "application/octet-stream".to_string(),
            });

        validated.push(serde_json::json!({
            "kind": kind,
            "path": canonical_path.to_string_lossy().to_string(),
            "mime": mime,
        }));
    }

    Ok(validated)
}

fn artifact_allowed_roots(skill_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();

    roots.push(
        std::fs::canonicalize(skill_dir)
            .map_err(|err| format!("skill 目录无法解析: {} ({err})", skill_dir.display()))?,
    );

    if let Ok(gen_images_dir) = resolve_gen_images_dir() {
        if let Ok(canonical) = std::fs::canonicalize(&gen_images_dir) {
            roots.push(canonical);
        } else if gen_images_dir.is_absolute() {
            roots.push(gen_images_dir);
        }
    }

    if let Ok(sandbox_root) = std::env::var("HONE_AGENT_SANDBOX_DIR") {
        let sandbox_root = PathBuf::from(sandbox_root);
        if let Ok(canonical) = std::fs::canonicalize(&sandbox_root) {
            roots.push(canonical);
        } else if sandbox_root.is_absolute() {
            roots.push(sandbox_root);
        }
    }

    if let Ok(working_directory) = std::env::var("HONE_MCP_WORKING_DIRECTORY") {
        let working_directory = PathBuf::from(working_directory);
        if let Ok(canonical) = std::fs::canonicalize(&working_directory) {
            roots.push(canonical);
        } else if working_directory.is_absolute() {
            roots.push(working_directory);
        }
    }

    roots.sort();
    roots.dedup();
    Ok(roots)
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
                name: "script_payload".to_string(),
                param_type: "object".to_string(),
                description: "可选。由运行时序列化为单个 JSON 脚本参数的结构化对象；不能与 script_arguments 或 args 同时使用。".to_string(),
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
                "error": "skill_name 不能为空",
                "side_effect_status": "not_started"
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
                                "error": error.message(),
                                "side_effect_status": error.side_effect_status(),
                                "skill_name": skill.id,
                                "script": skill.script,
                            }));
                        }
                    };
                let artifacts = script_execution
                    .as_ref()
                    .and_then(|value| value.get("artifacts"))
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new()));
                let render_success = script_execution
                    .as_ref()
                    .and_then(|value| value.get("render_success"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let render_summary = script_execution
                    .as_ref()
                    .and_then(|value| value.get("summary"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let render_warnings = script_execution
                    .as_ref()
                    .and_then(|value| value.get("warnings"))
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new()));
                let render_error = script_execution
                    .as_ref()
                    .and_then(|value| value.get("error"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let render_fallback_message = script_execution
                    .as_ref()
                    .and_then(|value| value.get("fallback_message"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let script_requested = args
                    .get("execute_script")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let compact_script_execution = script_execution.as_ref().map(|execution| {
                    let mut compact = execution.clone();
                    if let Some(object) = compact.as_object_mut() {
                        object.remove("arguments");
                        object.remove("stdout");
                        if object
                            .get("stderr")
                            .and_then(Value::as_str)
                            .is_some_and(str::is_empty)
                        {
                            object.remove("stderr");
                        }
                    }
                    compact
                });
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
                    "updated_at": hone_core::local_now_rfc3339(),
                });
                let _ = self.persist_invoked_skill(&payload).await;
                if script_requested {
                    let execution_succeeded = render_success.as_bool().unwrap_or(false);
                    let validated_report_markdown = execution_succeeded
                        .then(|| {
                            args.get("script_payload")
                                .and_then(|value| value.get("report_markdown"))
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .map(str::to_string)
                        })
                        .flatten();
                    return Ok(serde_json::json!({
                        "success": execution_succeeded,
                        "side_effect_status": if execution_succeeded { "completed" } else { "not_started" },
                        "skill_name": skill.id,
                        "skill_display_name": skill.display_name,
                        "script": payload["script"],
                        "skill_context_persisted": true,
                        "script_execution": compact_script_execution,
                        "artifacts": artifacts,
                        "render_success": render_success,
                        "render_summary": render_summary,
                        "render_warnings": render_warnings,
                        "render_error": render_error,
                        "render_fallback_message": render_fallback_message,
                        "validated_report_markdown": validated_report_markdown,
                        "reminder": if execution_succeeded {
                            "技能脚本执行成功；请确认 artifact 并完成用户原始任务。"
                        } else {
                            "技能脚本未通过校验；请一次修正 render_error 中列出的全部问题后重试。"
                        }
                    }));
                }
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
                    "script_execution": compact_script_execution,
                    "artifacts": artifacts,
                    "render_success": render_success,
                    "render_summary": render_summary,
                    "render_warnings": render_warnings,
                    "render_error": render_error,
                    "render_fallback_message": render_fallback_message,
                    "reminder": "技能已完整展开。请继续围绕用户原始任务执行，不要忘记真正要解决的问题。"
                }))
            }
            Err(error) => Ok(serde_json::json!({
                "success": false,
                "error": error,
                "side_effect_status": "not_started",
                "available_skills": runtime
                    .list_summaries_for_stage(&stage_constraints)
                    .into_iter()
                    .map(|skill| skill.id)
                    .collect::<Vec<_>>()
            })),
        }
    }
}
