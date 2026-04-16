use hone_core::ActorIdentity;
use hone_tools::ToolRegistry;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::HoneBotCore;
use crate::runners::AgentRunnerRequest;

pub fn hone_mcp_servers(request: &AgentRunnerRequest) -> Result<Value, String> {
    let command = hone_mcp_command_path()?;
    let mut env_entries = vec![
        json!({
            "name": "HONE_CONFIG_PATH",
            "value": request.config_path,
        }),
        json!({
            "name": "HONE_MCP_ACTOR_CHANNEL",
            "value": request.actor.channel,
        }),
        json!({
            "name": "HONE_MCP_ACTOR_USER_ID",
            "value": request.actor.user_id,
        }),
        json!({
            "name": "HONE_MCP_CHANNEL_TARGET",
            "value": request.channel_target,
        }),
        json!({
            "name": "HONE_MCP_SESSION_ID",
            "value": request.session_id,
        }),
        json!({
            "name": "HONE_MCP_ALLOW_CRON",
            "value": if request.allow_cron { "1" } else { "0" },
        }),
    ];
    if let Some(scope) = &request.actor.channel_scope {
        env_entries.push(json!({
            "name": "HONE_MCP_ACTOR_SCOPE",
            "value": scope,
        }));
    }
    if let Ok(data_dir) = env::var("HONE_DATA_DIR") {
        env_entries.push(json!({
            "name": "HONE_DATA_DIR",
            "value": data_dir,
        }));
    }
    if let Ok(skills_dir) = env::var("HONE_SKILLS_DIR") {
        env_entries.push(json!({
            "name": "HONE_SKILLS_DIR",
            "value": skills_dir,
        }));
    }
    if let Ok(sandbox_dir) = env::var("HONE_AGENT_SANDBOX_DIR") {
        env_entries.push(json!({
            "name": "HONE_AGENT_SANDBOX_DIR",
            "value": sandbox_dir,
        }));
    }
    if let Some(allowed_tools) = &request.allowed_tools {
        env_entries.push(json!({
            "name": "HONE_MCP_ALLOWED_TOOLS",
            "value": allowed_tools.join(","),
        }));
    }
    if let Some(max_tool_calls) = request.max_tool_calls {
        env_entries.push(json!({
            "name": "HONE_MCP_MAX_TOOL_CALLS",
            "value": max_tool_calls.to_string(),
        }));
    }

    Ok(json!([
        {
            "name": "hone",
            "command": command,
            "args": [],
            "env": env_entries,
        }
    ]))
}

fn hone_mcp_command_path() -> Result<String, String> {
    if let Ok(path) = env::var("HONE_MCP_BIN") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let current_exe =
        env::current_exe().map_err(|e| format!("failed to resolve current exe: {e}"))?;
    let parent = current_exe
        .parent()
        .ok_or_else(|| format!("failed to resolve parent dir for {}", current_exe.display()))?;
    let mut candidates = bundled_binary_candidates(parent, "hone-mcp");
    if parent.file_name().and_then(|value| value.to_str()) == Some("deps") {
        if let Some(grandparent) = parent.parent() {
            candidates.extend(bundled_binary_candidates(grandparent, "hone-mcp"));
        }
    }

    if let Some(found) = candidates.iter().find(|candidate| candidate.exists()) {
        Ok(found.to_string_lossy().to_string())
    } else {
        let tried = candidates
            .iter()
            .map(|candidate| candidate.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!(
            "hone-mcp binary not found near current executable; tried: {tried} (set HONE_MCP_BIN to override)"
        ))
    }
}

fn bundled_binary_candidates(base_dir: &Path, binary_stem: &str) -> Vec<PathBuf> {
    let mut dirs = vec![base_dir.to_path_buf()];

    if let Some(resources_dir) = macos_resources_dir(base_dir) {
        dirs.push(resources_dir.clone());
        dirs.push(resources_dir.join("binaries"));
    }

    let mut candidates = Vec::new();
    for dir in dirs {
        for name in bundled_binary_names(binary_stem) {
            candidates.push(dir.join(&name));
        }
    }
    candidates
}

fn bundled_binary_names(binary_stem: &str) -> Vec<String> {
    let mut names = Vec::new();
    let base = if cfg!(windows) {
        format!("{binary_stem}.exe")
    } else {
        binary_stem.to_string()
    };
    names.push(base);

    if let Some(triple) = current_target_triple() {
        let suffixed = if cfg!(windows) {
            format!("{binary_stem}-{triple}.exe")
        } else {
            format!("{binary_stem}-{triple}")
        };
        names.push(suffixed);
    }

    names
}

fn current_target_triple() -> Option<String> {
    let arch = match env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        "x86" => "i686",
        other => other,
    };
    let os = match env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        _ => return None,
    };
    Some(format!("{arch}-{os}"))
}

fn macos_resources_dir(base_dir: &Path) -> Option<PathBuf> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let macos_dir = base_dir.file_name()?.to_str()?;
    if macos_dir != "MacOS" {
        return None;
    }
    base_dir.parent().map(|contents| contents.join("Resources"))
}

pub async fn run_hone_mcp_stdio() -> Result<(), String> {
    let (config, config_path) = crate::load_runtime_config().map_err(|e| e.to_string())?;
    let core = HoneBotCore::new(config);
    let actor = actor_from_env()?;
    let channel_target = env::var("HONE_MCP_CHANNEL_TARGET").unwrap_or_else(|_| "mcp".to_string());
    let allow_cron = env_bool("HONE_MCP_ALLOW_CRON");
    let registry = core.create_tool_registry(actor.as_ref(), &channel_target, allow_cron);

    tracing::info!(
        "[hone-mcp] started config_path={} actor={} channel_target={} allow_cron={} tools={}",
        config_path,
        actor
            .as_ref()
            .map(|a| a.session_id())
            .unwrap_or_else(|| "none".to_string()),
        channel_target,
        allow_cron,
        registry.len()
    );

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = tokio::io::BufWriter::new(stdout);

    while let Some(line) = reader
        .next_line()
        .await
        .map_err(|e| format!("failed to read MCP stdin: {e}"))?
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let payload: Value = match serde_json::from_str(trimmed) {
            Ok(value) => value,
            Err(err) => {
                write_response(
                    &mut writer,
                    None,
                    None,
                    Some(jsonrpc_error(-32700, &format!("parse error: {err}"))),
                )
                .await?;
                continue;
            }
        };

        let id = payload.get("id").cloned();
        let method = payload.get("method").and_then(|v| v.as_str());
        let params = payload.get("params").cloned().unwrap_or(Value::Null);

        let Some(method) = method else {
            if id.is_some() {
                write_response(
                    &mut writer,
                    id,
                    None,
                    Some(jsonrpc_error(-32600, "invalid request: missing method")),
                )
                .await?;
            }
            continue;
        };

        let result = match method {
            "initialize" => Some(handle_initialize(&params)),
            "notifications/initialized" => None,
            "ping" => Some(json!({})),
            "tools/list" => Some(handle_tools_list(&registry)),
            "tools/call" => Some(handle_tools_call(&registry, &params).await),
            "resources/list" => Some(json!({ "resources": [] })),
            "prompts/list" => Some(json!({ "prompts": [] })),
            _ => {
                if id.is_some() {
                    write_response(
                        &mut writer,
                        id,
                        None,
                        Some(jsonrpc_error(
                            -32601,
                            &format!("method not found: {method}"),
                        )),
                    )
                    .await?;
                }
                continue;
            }
        };

        if id.is_some() {
            if let Some(result) = result {
                write_response(&mut writer, id, Some(result), None).await?;
            }
        }
    }

    Ok(())
}

fn actor_from_env() -> Result<Option<ActorIdentity>, String> {
    let channel = env::var("HONE_MCP_ACTOR_CHANNEL").unwrap_or_default();
    let user_id = env::var("HONE_MCP_ACTOR_USER_ID").unwrap_or_default();
    if channel.trim().is_empty() || user_id.trim().is_empty() {
        return Ok(None);
    }
    let scope = env::var("HONE_MCP_ACTOR_SCOPE").ok();
    ActorIdentity::new(channel, user_id, scope)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn env_bool(name: &str) -> bool {
    matches!(
        env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}

fn allowed_tools_from_env() -> Option<HashSet<String>> {
    let raw = env::var("HONE_MCP_ALLOWED_TOOLS").ok()?;
    let set: HashSet<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect();
    if set.is_empty() { None } else { Some(set) }
}

fn max_tool_calls_from_env() -> Option<u32> {
    env::var("HONE_MCP_MAX_TOOL_CALLS")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
}

fn tool_call_counters() -> &'static Mutex<HashMap<String, u32>> {
    static COUNTERS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
    COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn truncate_for_log(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let truncated = text.chars().take(keep).collect::<String>();
    format!("{truncated}…")
}

fn redact_value_for_log(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let lower = key.to_ascii_lowercase();
                    let redacted = matches!(
                        lower.as_str(),
                        "api_key"
                            | "apikey"
                            | "token"
                            | "access_token"
                            | "authorization"
                            | "password"
                            | "secret"
                    );
                    let sanitized = if redacted {
                        Value::String("<redacted>".to_string())
                    } else {
                        redact_value_for_log(value)
                    };
                    (key.clone(), sanitized)
                })
                .collect(),
        ),
        Value::Array(values) => {
            Value::Array(values.iter().map(redact_value_for_log).collect::<Vec<_>>())
        }
        _ => value.clone(),
    }
}

fn value_excerpt_for_log(value: &Value, max_chars: usize) -> String {
    let redacted = redact_value_for_log(value);
    let encoded = serde_json::to_string(&redacted).unwrap_or_else(|_| redacted.to_string());
    truncate_for_log(&encoded, max_chars)
}

fn mcp_actor_label_for_log() -> String {
    let channel = env::var("HONE_MCP_ACTOR_CHANNEL").unwrap_or_default();
    let user_id = env::var("HONE_MCP_ACTOR_USER_ID").unwrap_or_default();
    let scope = env::var("HONE_MCP_ACTOR_SCOPE").unwrap_or_default();
    if scope.trim().is_empty() {
        format!("{channel}/{user_id}")
    } else {
        format!("{channel}/{user_id}@{scope}")
    }
}

fn handle_initialize(params: &Value) -> Value {
    let protocol_version = params
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("2025-06-18");

    json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "hone-mcp",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn handle_tools_list(registry: &ToolRegistry) -> Value {
    let mut tools: Vec<Value> = registry
        .get_tools_schema()
        .into_iter()
        .filter(|schema| {
            let allowed_tools = allowed_tools_from_env();
            if let Some(allowed) = allowed_tools {
                schema
                    .get("function")
                    .and_then(|value| value.get("name"))
                    .and_then(|value| value.as_str())
                    .is_some_and(|name| allowed.contains(name))
            } else {
                true
            }
        })
        .filter_map(openai_tool_schema_to_mcp)
        .collect();
    tools.sort_by(|a, b| {
        a.get("name")
            .and_then(|v| v.as_str())
            .cmp(&b.get("name").and_then(|v| v.as_str()))
    });
    json!({ "tools": tools })
}

async fn handle_tools_call(registry: &ToolRegistry, params: &Value) -> Value {
    let Some(name) = params.get("name").and_then(|v| v.as_str()) else {
        return json!({
            "content": [{ "type": "text", "text": "missing tool name" }],
            "isError": true
        });
    };

    if let Some(allowed_tools) = allowed_tools_from_env() {
        if !allowed_tools.contains(name) {
            return json!({
                "content": [{ "type": "text", "text": format!("tool `{name}` is not allowed in this stage") }],
                "isError": true
            });
        }
    }

    if let Some(limit) = max_tool_calls_from_env() {
        let session_id = env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
        if !session_id.trim().is_empty() {
            let counters = tool_call_counters();
            let mut guard = counters.lock().expect("tool_call_counters lock");
            let entry = guard.entry(session_id).or_insert(0);
            if *entry >= limit {
                return json!({
                    "content": [{ "type": "text", "text": format!("tool call limit reached ({limit})") }],
                    "isError": true
                });
            }
            *entry += 1;
        }
    }

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let session_id = env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
    let actor = mcp_actor_label_for_log();
    let args_excerpt = value_excerpt_for_log(&arguments, 240);
    tracing::info!(
        "[hone-mcp] tool.start session={} actor={} name={} args={}",
        session_id,
        actor,
        name,
        args_excerpt
    );
    let started_at = Instant::now();
    match registry.execute_tool(name, arguments).await {
        Ok(value) => {
            let is_error = value.get("error").is_some();
            if is_error {
                tracing::warn!(
                    "[hone-mcp] tool.done session={} actor={} name={} duration_ms={} is_error={} result={}",
                    session_id,
                    actor,
                    name,
                    started_at.elapsed().as_millis(),
                    is_error,
                    value_excerpt_for_log(&value, 320)
                );
            } else {
                tracing::info!(
                    "[hone-mcp] tool.done session={} actor={} name={} duration_ms={} is_error={} result={}",
                    session_id,
                    actor,
                    name,
                    started_at.elapsed().as_millis(),
                    is_error,
                    value_excerpt_for_log(&value, 320)
                );
            }
            json!({
                "content": [{ "type": "text", "text": value.to_string() }],
                "structuredContent": value,
                "isError": is_error
            })
        }
        Err(err) => {
            tracing::warn!(
                "[hone-mcp] tool.error session={} actor={} name={} duration_ms={} error={}",
                session_id,
                actor,
                name,
                started_at.elapsed().as_millis(),
                truncate_for_log(&err.to_string(), 320)
            );
            json!({
                "content": [{ "type": "text", "text": err.to_string() }],
                "isError": true
            })
        }
    }
}

fn openai_tool_schema_to_mcp(schema: Value) -> Option<Value> {
    let function = schema.get("function")?;
    let name = function.get("name")?.as_str()?;
    let description = function
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let input_schema = function
        .get("parameters")
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
    Some(json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    }))
}

async fn write_response(
    writer: &mut tokio::io::BufWriter<tokio::io::Stdout>,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<Value>,
) -> Result<(), String> {
    let mut payload = serde_json::Map::new();
    payload.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    if let Some(id) = id {
        payload.insert("id".to_string(), id);
    }
    if let Some(result) = result {
        payload.insert("result".to_string(), result);
    }
    if let Some(error) = error {
        payload.insert("error".to_string(), error);
    }

    let encoded = serde_json::to_string(&Value::Object(payload))
        .map_err(|e| format!("failed to encode MCP response: {e}"))?;
    writer
        .write_all(encoded.as_bytes())
        .await
        .map_err(|e| format!("failed to write MCP response: {e}"))?;
    writer
        .write_all(b"\n")
        .await
        .map_err(|e| format!("failed to write MCP newline: {e}"))?;
    writer
        .flush()
        .await
        .map_err(|e| format!("failed to flush MCP response: {e}"))?;
    Ok(())
}

fn jsonrpc_error(code: i64, message: &str) -> Value {
    json!({
        "code": code,
        "message": message,
    })
}

pub fn hone_mcp_command_candidate() -> Option<PathBuf> {
    hone_mcp_command_path().ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GeminiStreamOptions;
    use crate::HoneBotCore;
    use hone_core::agent::AgentContext;
    use hone_core::{ActorIdentity, HoneConfig};
    use serde_json::json;
    use std::sync::MutexGuard;
    use std::time::Duration;

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    fn clear_test_env() {
        for key in [
            "HONE_MCP_BIN",
            "HONE_MCP_ALLOWED_TOOLS",
            "HONE_MCP_MAX_TOOL_CALLS",
            "HONE_MCP_ACTOR_CHANNEL",
            "HONE_MCP_ACTOR_USER_ID",
            "HONE_MCP_ACTOR_SCOPE",
            "HONE_MCP_SESSION_ID",
            "HONE_DATA_DIR",
            "HONE_SKILLS_DIR",
            "HONE_AGENT_SANDBOX_DIR",
        ] {
            unsafe { env::remove_var(key) };
        }
    }

    fn make_request() -> AgentRunnerRequest {
        AgentRunnerRequest {
            session_id: "session-1".to_string(),
            actor_label: "feishu:alice".to_string(),
            actor: ActorIdentity::new("feishu", "alice", Some("group-1")).expect("actor"),
            channel_target: "feishu".to_string(),
            allow_cron: true,
            config_path: "/tmp/config.yaml".to_string(),
            system_prompt: "system".to_string(),
            runtime_input: "input".to_string(),
            context: AgentContext::new("session-1".to_string()),
            timeout: Some(Duration::from_secs(30)),
            gemini_stream: GeminiStreamOptions::default(),
            session_metadata: HashMap::new(),
            working_directory: ".".to_string(),
            allowed_tools: Some(vec![
                "discover_skills".to_string(),
                "skill_tool".to_string(),
            ]),
            max_tool_calls: Some(3),
        }
    }

    #[test]
    fn hone_mcp_servers_prefers_explicit_binary_and_exports_request_env() {
        let _guard = env_lock();
        clear_test_env();
        unsafe {
            env::set_var("HONE_MCP_BIN", "/tmp/hone-mcp-custom");
            env::set_var("HONE_DATA_DIR", "/tmp/hone-data");
            env::set_var("HONE_SKILLS_DIR", "/tmp/hone-skills");
            env::set_var("HONE_AGENT_SANDBOX_DIR", "/tmp/hone-sandboxes");
        }

        let payload = hone_mcp_servers(&make_request()).expect("payload");
        let server = payload
            .as_array()
            .and_then(|items| items.first())
            .expect("server entry");
        let env_entries = server
            .get("env")
            .and_then(|value| value.as_array())
            .expect("env entries");

        assert_eq!(
            server.get("command").and_then(|value| value.as_str()),
            Some("/tmp/hone-mcp-custom")
        );
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_MCP_ALLOWED_TOOLS")
                && entry.get("value").and_then(|v| v.as_str()) == Some("discover_skills,skill_tool")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_MCP_MAX_TOOL_CALLS")
                && entry.get("value").and_then(|v| v.as_str()) == Some("3")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_MCP_ACTOR_SCOPE")
                && entry.get("value").and_then(|v| v.as_str()) == Some("group-1")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_DATA_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp/hone-data")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_SKILLS_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp/hone-skills")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_AGENT_SANDBOX_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp/hone-sandboxes")
        }));
    }

    #[test]
    fn actor_and_tool_limits_can_be_read_from_env() {
        let _guard = env_lock();
        clear_test_env();
        unsafe {
            env::set_var("HONE_MCP_ACTOR_CHANNEL", "discord");
            env::set_var("HONE_MCP_ACTOR_USER_ID", "bob");
            env::set_var("HONE_MCP_ACTOR_SCOPE", "room-9");
            env::set_var("HONE_MCP_ALLOWED_TOOLS", "web_search, skill_tool ,, ");
            env::set_var("HONE_MCP_MAX_TOOL_CALLS", "7");
        }

        let actor = actor_from_env().expect("actor parse").expect("actor");
        let allowed = allowed_tools_from_env().expect("allowed tools");

        assert_eq!(actor.channel, "discord");
        assert_eq!(actor.user_id, "bob");
        assert_eq!(actor.channel_scope.as_deref(), Some("room-9"));
        assert!(allowed.contains("web_search"));
        assert!(allowed.contains("skill_tool"));
        assert_eq!(max_tool_calls_from_env(), Some(7));
    }

    #[test]
    fn env_bool_accepts_common_truthy_values() {
        let _guard = env_lock();
        clear_test_env();
        unsafe { env::set_var("HONE_MCP_ALLOW_CRON", "YES") };
        assert!(env_bool("HONE_MCP_ALLOW_CRON"));
        unsafe { env::set_var("HONE_MCP_ALLOW_CRON", "0") };
        assert!(!env_bool("HONE_MCP_ALLOW_CRON"));
    }

    #[test]
    fn handle_tools_list_respects_allowed_tools_for_local_file_tools() {
        let _guard = env_lock();
        clear_test_env();
        unsafe {
            env::set_var("HONE_MCP_ALLOWED_TOOLS", "local_list_files");
        }

        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("telegram", "8039067465", None::<String>).expect("actor");
        let registry = core.create_tool_registry(Some(&actor), "telegram", false);
        let payload = handle_tools_list(&registry);
        let tools = payload["tools"].as_array().expect("tools");

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "local_list_files");
    }

    #[test]
    fn handle_tools_list_exposes_cron_job_only_when_allow_cron_is_enabled() {
        let _guard = env_lock();
        clear_test_env();

        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("telegram", "8039067465", None::<String>).expect("actor");

        let disabled_registry = core.create_tool_registry(Some(&actor), "telegram", false);
        let disabled_payload = handle_tools_list(&disabled_registry);
        let disabled_tools = disabled_payload["tools"].as_array().expect("tools");
        assert!(
            !disabled_tools
                .iter()
                .any(|tool| tool["name"].as_str() == Some("cron_job"))
        );

        let enabled_registry = core.create_tool_registry(Some(&actor), "telegram", true);
        let enabled_payload = handle_tools_list(&enabled_registry);
        let enabled_tools = enabled_payload["tools"].as_array().expect("tools");
        assert!(
            enabled_tools
                .iter()
                .any(|tool| tool["name"].as_str() == Some("cron_job"))
        );
    }

    #[test]
    fn handle_tools_call_rejects_cron_job_when_stage_allowed_tools_excludes_it() {
        let _guard = env_lock();
        clear_test_env();
        unsafe {
            env::set_var("HONE_MCP_ALLOWED_TOOLS", "discover_skills,skill_tool");
        }

        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("telegram", "8039067465", None::<String>).expect("actor");
        let registry = core.create_tool_registry(Some(&actor), "telegram", true);

        let list_payload = handle_tools_list(&registry);
        let tools = list_payload["tools"].as_array().expect("tools");
        assert!(
            !tools
                .iter()
                .any(|tool| tool["name"].as_str() == Some("cron_job"))
        );

        let call_payload = futures::executor::block_on(handle_tools_call(
            &registry,
            &json!({
                "name": "cron_job",
                "arguments": { "action": "list" }
            }),
        ));
        assert_eq!(call_payload["isError"], Value::Bool(true));
        assert_eq!(
            call_payload["content"][0]["text"],
            Value::String("tool `cron_job` is not allowed in this stage".to_string())
        );
    }

    #[test]
    fn openai_tool_schema_to_mcp_preserves_name_description_and_schema() {
        let converted = openai_tool_schema_to_mcp(json!({
            "type": "function",
            "function": {
                "name": "skill_tool",
                "description": "run a skill",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "skill_name": { "type": "string" }
                    }
                }
            }
        }))
        .expect("converted");

        assert_eq!(
            converted.get("name").and_then(|v| v.as_str()),
            Some("skill_tool")
        );
        assert_eq!(
            converted.get("description").and_then(|v| v.as_str()),
            Some("run a skill")
        );
        assert_eq!(
            converted
                .get("inputSchema")
                .and_then(|v| v.get("properties"))
                .and_then(|v| v.get("skill_name"))
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str()),
            Some("string")
        );
    }
}
