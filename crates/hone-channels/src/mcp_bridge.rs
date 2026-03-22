use hone_core::ActorIdentity;
use hone_tools::ToolRegistry;
use serde_json::{Value, json};
use std::env;
use std::path::PathBuf;
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
    let binary_name = if cfg!(windows) {
        "hone-mcp.exe"
    } else {
        "hone-mcp"
    };

    let mut candidates = vec![parent.join(binary_name)];
    if parent.file_name().and_then(|value| value.to_str()) == Some("deps") {
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.join(binary_name));
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

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match registry.execute_tool(name, arguments).await {
        Ok(value) => {
            let is_error = value.get("error").is_some();
            json!({
                "content": [{ "type": "text", "text": value.to_string() }],
                "structuredContent": value,
                "isError": is_error
            })
        }
        Err(err) => json!({
            "content": [{ "type": "text", "text": err.to_string() }],
            "isError": true
        }),
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
