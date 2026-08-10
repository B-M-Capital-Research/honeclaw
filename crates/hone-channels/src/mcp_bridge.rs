use hone_core::ActorIdentity;
use hone_core::config::HoneConfig;
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
use crate::runners::REQUIRE_EARNINGS_PDF_COMPLETION_METADATA_KEY;
#[cfg(test)]
use crate::runners::RunnerConversationInput;

pub(crate) const EMPTY_MCP_TOOL_ALLOWLIST_SENTINEL: &str = "__hone_no_mcp_tools__";
const HONE_MCP_REQUIRE_EARNINGS_EVIDENCE: &str = "HONE_MCP_REQUIRE_EARNINGS_EVIDENCE";
const MAX_EARNINGS_EVIDENCE_DOCUMENTS: usize = 256;
const MAX_EARNINGS_EVIDENCE_MANIFEST_ENTRIES: usize = 120;
const MAX_EARNINGS_EVIDENCE_EXCERPT_CHARS: usize = 700;

#[derive(Debug, Default)]
struct EarningsEvidenceLedger {
    documents: Vec<EarningsEvidenceDocument>,
}

#[derive(Debug)]
struct EarningsEvidenceDocument {
    urls: HashSet<String>,
    normalized_text: String,
}

pub fn hone_mcp_servers(request: &AgentRunnerRequest) -> Result<Value, String> {
    if matches!(
        request.allowed_tools.as_deref(),
        Some([only]) if only == EMPTY_MCP_TOOL_ALLOWLIST_SENTINEL
    ) {
        return Ok(json!([]));
    }
    let command = hone_mcp_command_path()?;
    let mut env_entries = vec![
        mcp_env_entry("HONE_CONFIG_PATH", request.config_path.as_str()),
        mcp_env_entry("HONE_MCP_ACTOR_CHANNEL", request.actor.channel.as_str()),
        mcp_env_entry("HONE_MCP_ACTOR_USER_ID", request.actor.user_id.as_str()),
        mcp_env_entry("HONE_MCP_CHANNEL_TARGET", request.channel_target.as_str()),
        mcp_env_entry("HONE_MCP_SESSION_ID", request.session_id.as_str()),
        mcp_env_entry(
            "HONE_MCP_WORKING_DIRECTORY",
            request.working_directory.as_str(),
        ),
        mcp_env_entry(
            "HONE_MCP_ALLOW_CRON",
            if request.allow_cron { "1" } else { "0" },
        ),
    ];
    if let Some(scope) = &request.actor.channel_scope {
        env_entries.push(mcp_env_entry("HONE_MCP_ACTOR_SCOPE", scope.as_str()));
    }
    push_data_dir_env_or_derived(&mut env_entries, || {
        absolute_parent_dir(&request.runtime_dir)
    });
    push_skills_dir_env_or_derived(&mut env_entries, &request.config_path);
    push_env_var_if_present(&mut env_entries, "HONE_AGENT_SANDBOX_DIR");
    push_runtime_env_vars_from_config(&mut env_entries, &request.config_path);
    if let Some(allowed_tools) = &request.allowed_tools {
        env_entries.push(mcp_env_entry(
            "HONE_MCP_ALLOWED_TOOLS",
            allowed_tools.join(","),
        ));
    }
    if let Some(max_tool_calls) = request.max_tool_calls {
        env_entries.push(mcp_env_entry(
            "HONE_MCP_MAX_TOOL_CALLS",
            max_tool_calls.to_string(),
        ));
    }
    if request
        .session_metadata
        .get(REQUIRE_EARNINGS_PDF_COMPLETION_METADATA_KEY)
        .and_then(Value::as_bool)
        == Some(true)
    {
        env_entries.push(mcp_env_entry(HONE_MCP_REQUIRE_EARNINGS_EVIDENCE, "1"));
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

fn mcp_env_entry(name: &str, value: impl Into<String>) -> Value {
    json!({
        "name": name,
        "value": value.into(),
    })
}

fn push_env_var_if_present(env_entries: &mut Vec<Value>, name: &str) {
    if let Ok(value) = env::var(name) {
        env_entries.push(mcp_env_entry(name, value));
    }
}

fn push_data_dir_env_or_derived(
    env_entries: &mut Vec<Value>,
    derived: impl FnOnce() -> Option<String>,
) {
    if let Some(value) = normalized_env_dir("HONE_DATA_DIR") {
        env_entries.push(mcp_env_entry("HONE_DATA_DIR", value));
    } else if let Some(value) = derived().filter(|value| !value.trim().is_empty()) {
        env_entries.push(mcp_env_entry("HONE_DATA_DIR", value));
    }
}

fn push_skills_dir_env_or_derived(env_entries: &mut Vec<Value>, config_path: &str) {
    if let Some(value) = normalized_env_dir("HONE_SKILLS_DIR") {
        env_entries.push(mcp_env_entry("HONE_SKILLS_DIR", value));
        return;
    }

    let config_file = PathBuf::from(config_path);
    let config_root = config_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let configured = HoneConfig::from_file(config_path)
        .ok()
        .and_then(|config| {
            config
                .extra
                .get("skills_dir")
                .and_then(|value| value.as_str())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("skills"));
    let absolute = if configured.is_absolute() {
        configured
    } else {
        config_root.join(configured)
    };
    env_entries.push(mcp_env_entry(
        "HONE_SKILLS_DIR",
        absolute.to_string_lossy().to_string(),
    ));
}

fn normalized_env_dir(name: &str) -> Option<String> {
    let value = env::var(name).ok()?;
    if value.trim().is_empty() {
        return None;
    }
    let candidate = PathBuf::from(value);
    let absolute = if candidate.is_absolute() {
        candidate
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(candidate)
    };
    Some(absolute.to_string_lossy().to_string())
}

fn absolute_parent_dir(path: &str) -> Option<String> {
    let candidate = PathBuf::from(path);
    let absolute = if candidate.is_absolute() {
        candidate
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(candidate)
    };
    absolute
        .parent()
        .map(|path| path.to_string_lossy().to_string())
}

fn push_runtime_env_vars_from_config(env_entries: &mut Vec<Value>, config_path: &str) {
    let mut names = vec![
        "HONE_CLOUD_MODE".to_string(),
        "HONE_CLOUD_ENABLED".to_string(),
        "HONE_CLOUD_STRICT_NO_LOCAL_STORAGE".to_string(),
        "DATABASE_URL".to_string(),
        "HONE_POSTGRES_HOST".to_string(),
        "HONE_POSTGRES_PORT".to_string(),
        "HONE_POSTGRES_USER".to_string(),
        "HONE_POSTGRES_PASSWORD".to_string(),
        "HONE_POSTGRES_DATABASE".to_string(),
        "HONE_POSTGRES_PROXY".to_string(),
        "HONE_POSTGRES_NO_PROXY".to_string(),
        "HONE_OSS_PROVIDER".to_string(),
        "HONE_OSS_ACCESS_KEY_ID".to_string(),
        "HONE_OSS_ACCESS_KEY_SECRET".to_string(),
        "HONE_OSS_BUCKET".to_string(),
        "HONE_OSS_ENDPOINT".to_string(),
        "HONE_OSS_REGION".to_string(),
        "HONE_OSS_PROXY".to_string(),
    ];

    if let Ok(config) = HoneConfig::from_file(config_path) {
        let pg = &config.cloud.postgres;
        names.extend([
            pg.database_url_env.clone(),
            pg.host_env.clone(),
            pg.port_env.clone(),
            pg.user_env.clone(),
            pg.password_env.clone(),
            pg.database_env.clone(),
            pg.proxy_env.clone(),
            pg.no_proxy_env.clone(),
        ]);

        let oss = &config.cloud.oss;
        names.extend([
            oss.provider_env.clone(),
            oss.access_key_id_env.clone(),
            oss.access_key_secret_env.clone(),
            oss.bucket_env.clone(),
            oss.endpoint_env.clone(),
            oss.region_env.clone(),
            oss.proxy_env.clone(),
        ]);
    }

    let mut seen: HashSet<String> = env_entries
        .iter()
        .filter_map(|entry| entry.get("name").and_then(|value| value.as_str()))
        .map(|name| name.to_string())
        .collect();
    for name in names {
        let trimmed = name.trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_string()) {
            continue;
        }
        push_env_var_if_present(env_entries, trimmed);
    }
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
    if parent.file_name().and_then(|value| value.to_str()) == Some("deps")
        && let Some(grandparent) = parent.parent()
    {
        candidates.extend(bundled_binary_candidates(grandparent, "hone-mcp"));
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
    let require_earnings_evidence = env_bool(HONE_MCP_REQUIRE_EARNINGS_EVIDENCE);
    let registry = core.create_tool_registry(actor.as_ref(), &channel_target, allow_cron);
    let mut earnings_evidence = EarningsEvidenceLedger::default();

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
            "tools/call" => Some(
                handle_tools_call(
                    &registry,
                    &params,
                    require_earnings_evidence,
                    &mut earnings_evidence,
                )
                .await,
            ),
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

        if id.is_some()
            && let Some(result) = result
        {
            write_response(&mut writer, id, Some(result), None).await?;
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
                    let sanitized = if is_sensitive_log_key(key) {
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

fn is_sensitive_log_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "api_key"
            | "apikey"
            | "x-api-key"
            | "token"
            | "access_token"
            | "refresh_token"
            | "id_token"
            | "session_token"
            | "bot_token"
            | "authorization"
            | "password"
            | "secret"
            | "app_secret"
            | "client_secret"
            | "openrouter_api_key"
            | "anthropic_api_key"
            | "gemini_api_key"
            | "google_api_key"
            | "tavily_api_key"
            | "fmp_api_key"
            | "hone_cloud_api_key"
    )
}

fn value_excerpt_for_log(value: &Value, max_chars: usize) -> String {
    let redacted = redact_value_for_log(value);
    let encoded = serde_json::to_string(&redacted).unwrap_or_else(|_| redacted.to_string());
    truncate_for_log(&encoded, max_chars)
}

fn text_excerpt_for_log(text: &str, max_chars: usize) -> String {
    truncate_for_log(&redact_text_for_log(text), max_chars)
}

fn redact_text_for_log(text: &str) -> String {
    let mut output = redact_marker_value(text, "Bearer ");
    output = redact_marker_value(&output, "Basic ");
    for key in SENSITIVE_TEXT_MARKER_KEYS {
        output = redact_marker_value(&output, &format!("{key}="));
        output = redact_marker_value(&output, &format!("{key}:"));
    }
    output
}

const SENSITIVE_TEXT_MARKER_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "api_key",
    "apiKey",
    "apikey",
    "app_secret",
    "appSecret",
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

fn redact_marker_value(text: &str, marker: &str) -> String {
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
    let allowed_tools = allowed_tools_from_env();
    let mut tools: Vec<Value> = registry
        .get_tools_schema()
        .into_iter()
        .filter(|schema| schema_tool_is_allowed(schema, allowed_tools.as_ref()))
        .filter_map(openai_tool_schema_to_mcp)
        .collect();
    tools.sort_by(|a, b| {
        a.get("name")
            .and_then(|v| v.as_str())
            .cmp(&b.get("name").and_then(|v| v.as_str()))
    });
    json!({ "tools": tools })
}

impl EarningsEvidenceLedger {
    fn record_tool_result(&mut self, tool_name: &str, value: &Value) {
        if !matches!(tool_name, "data_fetch" | "web_search") {
            return;
        }
        collect_earnings_evidence_documents(value, &mut self.documents);
        if self.documents.len() > MAX_EARNINGS_EVIDENCE_DOCUMENTS {
            let overflow = self.documents.len() - MAX_EARNINGS_EVIDENCE_DOCUMENTS;
            self.documents.drain(..overflow);
        }
    }

    fn contains_url(&self, url: &str) -> bool {
        self.documents
            .iter()
            .any(|document| document.urls.contains(url))
    }

    fn contains_excerpt_for_url(&self, url: &str, excerpt: &str) -> bool {
        let normalized_excerpt = normalize_evidence_text(excerpt);
        !normalized_excerpt.is_empty()
            && self.documents.iter().any(|document| {
                document.urls.contains(url)
                    && document.normalized_text.contains(&normalized_excerpt)
            })
    }
}

fn collect_earnings_evidence_documents(
    value: &Value,
    documents: &mut Vec<EarningsEvidenceDocument>,
) {
    match value {
        Value::Object(map) => {
            let urls = map
                .iter()
                .filter(|(key, _)| {
                    matches!(
                        key.to_ascii_lowercase().as_str(),
                        "url" | "source_url" | "link" | "href"
                    )
                })
                .filter_map(|(_, value)| value.as_str())
                .filter(|value| is_real_http_source_url(value))
                .map(str::to_string)
                .collect::<HashSet<_>>();
            if !urls.is_empty() {
                let mut strings = Vec::new();
                collect_evidence_strings(value, &mut strings);
                documents.push(EarningsEvidenceDocument {
                    urls,
                    normalized_text: normalize_evidence_text(&strings.join(" ")),
                });
            }
            for child in map.values() {
                collect_earnings_evidence_documents(child, documents);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_earnings_evidence_documents(child, documents);
            }
        }
        _ => {}
    }
}

fn collect_evidence_strings(value: &Value, strings: &mut Vec<String>) {
    match value {
        Value::String(text) => strings.push(text.clone()),
        Value::Number(number) => strings.push(number.to_string()),
        Value::Object(map) => {
            for child in map.values() {
                collect_evidence_strings(child, strings);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_evidence_strings(child, strings);
            }
        }
        Value::Bool(_) | Value::Null => {}
    }
}

fn normalize_evidence_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_real_http_source_url(value: &str) -> bool {
    let trimmed = value.trim();
    let Some(rest) = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .split('@')
        .next_back()
        .unwrap_or_default()
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    !host.is_empty()
        && !matches!(
            host.as_str(),
            "example.com" | "example.net" | "example.org" | "localhost" | "0.0.0.0"
        )
}

fn report_http_urls(text: &str) -> HashSet<String> {
    let mut urls = HashSet::new();
    let mut remaining = text;
    while let Some(index) = remaining
        .find("https://")
        .or_else(|| remaining.find("http://"))
    {
        let candidate = &remaining[index..];
        let end = candidate
            .char_indices()
            .find_map(|(offset, ch)| {
                (ch.is_whitespace() || matches!(ch, ')' | ']' | '>' | '"' | '\'')).then_some(offset)
            })
            .unwrap_or(candidate.len());
        let url = candidate[..end]
            .trim_end_matches([',', '.', ';', ':', '。', '，', '；'])
            .to_string();
        if is_real_http_source_url(&url) {
            urls.insert(url);
        }
        remaining = &candidate[end.max(1)..];
    }
    urls
}

fn strip_http_urls(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;
    while let Some(index) = remaining
        .find("https://")
        .or_else(|| remaining.find("http://"))
    {
        output.push_str(&remaining[..index]);
        let candidate = &remaining[index..];
        let end = candidate
            .char_indices()
            .find_map(|(offset, ch)| {
                (ch.is_whitespace() || matches!(ch, ')' | ']' | '>' | '"' | '\'')).then_some(offset)
            })
            .unwrap_or(candidate.len());
        remaining = &candidate[end.max(1)..];
    }
    output.push_str(remaining);
    output
}

fn material_report_units(report: &str) -> Vec<String> {
    report
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .filter(|line| !line.chars().all(|ch| matches!(ch, '-' | '_' | '*' | ' ')))
        .filter(|line| !is_markdown_table_separator(line))
        .filter(|line| !is_source_note_only(line))
        .filter(|line| is_material_earnings_claim(line))
        .map(str::to_string)
        .collect()
}

fn is_source_note_only(line: &str) -> bool {
    let trimmed = line.trim_start_matches(['-', '*', '>', ' ']);
    let lowered = trimmed.to_ascii_lowercase();
    report_http_urls(line).len() > 0
        && (trimmed.starts_with("来源")
            || trimmed.starts_with("参考")
            || trimmed.starts_with("资料")
            || lowered.starts_with("source")
            || lowered.starts_with("references"))
}

fn is_markdown_table_separator(line: &str) -> bool {
    line.contains('|') && line.chars().all(|ch| matches!(ch, '|' | '-' | ':' | ' '))
}

fn is_material_earnings_claim(line: &str) -> bool {
    let without_urls = strip_http_urls(line);
    if without_urls.chars().any(|ch| ch.is_ascii_digit()) {
        return true;
    }
    let lowered = without_urls.to_ascii_lowercase();
    const MARKERS: &[&str] = &[
        "评级",
        "目标价",
        "上调",
        "下调",
        "重申",
        "宣布",
        "公布",
        "表示",
        "积压",
        "订单",
        "营收",
        "利润",
        "亏损",
        "资本开支",
        "做空",
        "减持",
        "管理层",
        "分析师",
        "华尔街",
        "客户",
        "供应",
        "容量",
        "电力",
        "同比",
        "环比",
        "预期",
        "指引",
        "因果",
        "驱动",
        "导致",
        "带动",
        "受益",
        "源于",
        "由于",
        "合作",
        "协议",
        "投资",
        "收购",
        "融资",
        "任命",
        "离职",
        "backlog",
        "revenue",
        "profit",
        "loss",
        "rating",
        "target price",
        "guidance",
        "analyst",
        "management",
        "capex",
        "short interest",
        "insider",
        "customer",
        "capacity",
        "contract",
        "because",
        "driven by",
        "benefit from",
        "caused by",
        "lead to",
        "partnership",
        "agreement",
        "investment",
        "acquisition",
        "financing",
        "appointed",
        "resigned",
        "eps",
        "arr",
        "ceo",
        "cfo",
    ];
    MARKERS.iter().any(|marker| lowered.contains(marker))
}

fn numeric_evidence_tokens(text: &str) -> HashSet<String> {
    let without_urls = strip_http_urls(text);
    let chars = without_urls.char_indices().collect::<Vec<_>>();
    let mut tokens = HashSet::new();
    let mut index = 0usize;
    while index < chars.len() {
        let (_, ch) = chars[index];
        if !ch.is_ascii_digit() {
            index += 1;
            continue;
        }
        let start_byte = chars[index].0;
        let mut end_index = index + 1;
        let mut saw_dot = false;
        while end_index < chars.len() {
            let next = chars[end_index].1;
            if next.is_ascii_digit() || next == ',' {
                end_index += 1;
            } else if next == '.' && !saw_dot {
                saw_dot = true;
                end_index += 1;
            } else {
                break;
            }
        }
        let end_byte = chars
            .get(end_index)
            .map(|(offset, _)| *offset)
            .unwrap_or(without_urls.len());
        if numeric_token_is_citation_marker(&without_urls, start_byte, end_byte) {
            index = end_index.max(index + 1);
            continue;
        }
        let raw = without_urls[start_byte..end_byte]
            .trim_end_matches('.')
            .replace(',', "");
        if let Ok(number) = raw.parse::<f64>() {
            let suffix = &without_urls[end_byte..];
            tokens.insert(normalized_number(number * evidence_number_scale(suffix)));
        }
        index = end_index.max(index + 1);
    }
    tokens
}

fn numeric_token_is_citation_marker(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    matches!(
        (before, after),
        (Some('['), Some(']')) | (Some('【'), Some('】'))
    )
}

fn evidence_number_scale(suffix: &str) -> f64 {
    let trimmed = suffix.trim_start();
    if trimmed.starts_with('亿') {
        return 0.1;
    }
    let lowered = trimmed.to_ascii_lowercase();
    if has_number_unit_prefix(&lowered, "billion")
        || has_number_unit_prefix(&lowered, "bn")
        || has_number_unit_prefix(&lowered, "b")
    {
        return 1.0;
    }
    if has_number_unit_prefix(&lowered, "million")
        || has_number_unit_prefix(&lowered, "mm")
        || has_number_unit_prefix(&lowered, "m")
    {
        return 0.001;
    }
    1.0
}

fn has_number_unit_prefix(value: &str, unit: &str) -> bool {
    value.strip_prefix(unit).is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphabetic())
    })
}

fn normalized_number(number: f64) -> String {
    let rendered = format!("{number:.6}");
    rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn validate_earnings_renderer_evidence(
    arguments: &Value,
    ledger: &EarningsEvidenceLedger,
) -> Result<(), String> {
    if arguments.get("skill_name").and_then(Value::as_str) != Some("earnings-research")
        || arguments.get("execute_script").and_then(Value::as_bool) != Some(true)
    {
        return Ok(());
    }
    let payload = arguments
        .get("script_payload")
        .and_then(Value::as_object)
        .ok_or_else(|| "earnings renderer 缺少结构化 script_payload".to_string())?;
    let report = payload
        .get("report_markdown")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "earnings renderer 缺少 report_markdown".to_string())?;
    let manifest = payload
        .get("evidence_manifest")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "缺少 evidence_manifest；每个重大断言必须映射到本轮工具结果中的真实 URL 与原文摘录"
                .to_string()
        })?;
    if manifest.is_empty() {
        return Err("evidence_manifest 不能为空".to_string());
    }
    if manifest.len() > MAX_EARNINGS_EVIDENCE_MANIFEST_ENTRIES {
        return Err(format!(
            "evidence_manifest 条目过多（>{MAX_EARNINGS_EVIDENCE_MANIFEST_ENTRIES}）"
        ));
    }

    let units = material_report_units(report);
    if units.is_empty() {
        return Err("报告没有可核验的重大断言".to_string());
    }
    let report_urls = report_http_urls(report);
    for url in &report_urls {
        if !ledger.contains_url(url) {
            return Err(format!("报告来源 URL 不在本轮工具结果中：{url}"));
        }
    }

    struct ManifestEntry {
        claim_text: String,
        source_url: String,
        source_excerpt: String,
    }
    let mut entries = Vec::with_capacity(manifest.len());
    for (index, value) in manifest.iter().enumerate() {
        let object = value
            .as_object()
            .ok_or_else(|| format!("evidence_manifest[{index}] 必须是对象"))?;
        let claim_text = object
            .get("claim_text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("evidence_manifest[{index}] 缺少 claim_text"))?;
        let source_url = object
            .get("source_url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| is_real_http_source_url(value))
            .ok_or_else(|| format!("evidence_manifest[{index}] source_url 不是有效真实 URL"))?;
        let source_excerpt = object
            .get("source_excerpt")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("evidence_manifest[{index}] 缺少 source_excerpt"))?;
        let excerpt_chars = source_excerpt.chars().count();
        if !(12..=MAX_EARNINGS_EVIDENCE_EXCERPT_CHARS).contains(&excerpt_chars) {
            return Err(format!(
                "evidence_manifest[{index}] source_excerpt 长度必须为 12–{MAX_EARNINGS_EVIDENCE_EXCERPT_CHARS} 字符"
            ));
        }
        if !units.iter().any(|unit| unit == claim_text) {
            return Err(format!(
                "evidence_manifest[{index}] claim_text 必须逐字等于报告中的一条重大断言"
            ));
        }
        if !report_urls.contains(source_url) {
            return Err(format!(
                "evidence_manifest[{index}] 的 source_url 未出现在可见报告中：{source_url}"
            ));
        }
        if !ledger.contains_excerpt_for_url(source_url, source_excerpt) {
            return Err(format!(
                "evidence_manifest[{index}] 的 URL/摘录组合不属于本轮同一条工具结果：{source_url}"
            ));
        }
        entries.push(ManifestEntry {
            claim_text: claim_text.to_string(),
            source_url: source_url.to_string(),
            source_excerpt: source_excerpt.to_string(),
        });
    }

    let mut errors = Vec::new();
    for unit in &units {
        let matching = entries
            .iter()
            .filter(|entry| entry.claim_text == *unit)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            errors.push(format!("缺少证据映射：{}", truncate_for_log(unit, 120)));
            continue;
        }
        let claim_numbers = numeric_evidence_tokens(unit);
        if !claim_numbers.is_empty() {
            let evidence_numbers = matching
                .iter()
                .flat_map(|entry| numeric_evidence_tokens(&entry.source_excerpt))
                .collect::<HashSet<_>>();
            let missing = claim_numbers
                .difference(&evidence_numbers)
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                errors.push(format!(
                    "证据摘录未覆盖数字 {}：{}",
                    missing.join(","),
                    truncate_for_log(unit, 100)
                ));
            }
        }
        let mapped_urls = matching
            .iter()
            .map(|entry| entry.source_url.as_str())
            .collect::<HashSet<_>>();
        if mapped_urls.is_empty() {
            errors.push(format!(
                "重大断言没有来源 URL：{}",
                truncate_for_log(unit, 120)
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        let total = errors.len();
        Err(format!(
            "证据门禁未通过（PDF 尚未执行，共 {total} 项）：{}。请对缺口继续本轮搜索，或删除/改写断言，再提交 evidence_manifest",
            errors.into_iter().take(8).collect::<Vec<_>>().join("；")
        ))
    }
}

fn earnings_evidence_failure(error: String) -> Value {
    json!({
        "success": false,
        "render_success": false,
        "render_error": error,
        "side_effect_status": "not_started",
        "artifacts": [],
        "skill_name": "earnings-research",
    })
}

fn mcp_tool_result(value: Value) -> Value {
    let is_error = value.get("error").is_some();
    json!({
        "content": [{ "type": "text", "text": value.to_string() }],
        "structuredContent": value,
        "isError": is_error
    })
}

async fn handle_tools_call(
    registry: &ToolRegistry,
    params: &Value,
    require_earnings_evidence: bool,
    earnings_evidence: &mut EarningsEvidenceLedger,
) -> Value {
    let Some(name) = params.get("name").and_then(|v| v.as_str()) else {
        return mcp_text_error("missing tool name");
    };

    if let Some(allowed_tools) = allowed_tools_from_env()
        && !allowed_tools.contains(name)
    {
        return mcp_text_error(format!("tool `{name}` is not allowed in this stage"));
    }

    if let Some(limit_error) = consume_tool_call_budget() {
        return limit_error;
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
    if require_earnings_evidence
        && name == "skill_tool"
        && let Err(error) = validate_earnings_renderer_evidence(&arguments, earnings_evidence)
    {
        let value = earnings_evidence_failure(error);
        log_tool_done(&session_id, &actor, name, started_at, true, &value);
        return mcp_tool_result(value);
    }
    match registry.execute_tool(name, arguments).await {
        Ok(value) => {
            let is_error = value.get("error").is_some();
            if !is_error {
                earnings_evidence.record_tool_result(name, &value);
            }
            log_tool_done(&session_id, &actor, name, started_at, is_error, &value);
            mcp_tool_result(value)
        }
        Err(err) => {
            let err_text = err.to_string();
            tracing::warn!(
                "[hone-mcp] tool.error session={} actor={} name={} duration_ms={} error={}",
                session_id,
                actor,
                name,
                started_at.elapsed().as_millis(),
                text_excerpt_for_log(&err_text, 320)
            );
            mcp_text_error(err_text)
        }
    }
}

fn schema_tool_is_allowed(schema: &Value, allowed_tools: Option<&HashSet<String>>) -> bool {
    allowed_tools
        .map(|allowed| schema_tool_name(schema).is_some_and(|name| allowed.contains(name)))
        .unwrap_or(true)
}

fn schema_tool_name(schema: &Value) -> Option<&str> {
    schema
        .get("function")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
}

fn consume_tool_call_budget() -> Option<Value> {
    let limit = max_tool_calls_from_env()?;
    let session_id = env::var("HONE_MCP_SESSION_ID").unwrap_or_default();
    if session_id.trim().is_empty() {
        return None;
    }

    let counters = tool_call_counters();
    let mut guard = counters.lock().expect("tool_call_counters lock");
    let entry = guard.entry(session_id).or_insert(0);
    if *entry >= limit {
        return Some(mcp_text_error(format!("tool call limit reached ({limit})")));
    }
    *entry += 1;
    None
}

fn mcp_text_error(text: impl Into<String>) -> Value {
    json!({
        "content": [{ "type": "text", "text": text.into() }],
        "isError": true
    })
}

fn log_tool_done(
    session_id: &str,
    actor: &str,
    name: &str,
    started_at: Instant,
    is_error: bool,
    value: &Value,
) {
    let duration_ms = started_at.elapsed().as_millis();
    let result_excerpt = value_excerpt_for_log(value, 320);
    if is_error {
        tracing::warn!(
            "[hone-mcp] tool.done session={} actor={} name={} duration_ms={} is_error={} result={}",
            session_id,
            actor,
            name,
            duration_ms,
            is_error,
            result_excerpt
        );
    } else {
        tracing::info!(
            "[hone-mcp] tool.done session={} actor={} name={} duration_ms={} is_error={} result={}",
            session_id,
            actor,
            name,
            duration_ms,
            is_error,
            result_excerpt
        );
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

    fn set_test_mcp_binary_override() {
        unsafe {
            env::set_var(
                "HONE_MCP_BIN",
                std::env::temp_dir().join("hone-mcp-test-stub"),
            );
        }
    }

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{}_{}_{}",
            name,
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ))
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
            "HONE_MCP_WORKING_DIRECTORY",
            "HONE_MCP_REQUIRE_EARNINGS_EVIDENCE",
            "HONE_DATA_DIR",
            "HONE_SKILLS_DIR",
            "HONE_AGENT_SANDBOX_DIR",
            "HONE_CLOUD_MODE",
            "HONE_CLOUD_ENABLED",
            "HONE_CLOUD_STRICT_NO_LOCAL_STORAGE",
            "DATABASE_URL",
            "HONE_POSTGRES_HOST",
            "HONE_POSTGRES_PORT",
            "HONE_POSTGRES_USER",
            "HONE_POSTGRES_PASSWORD",
            "HONE_POSTGRES_DATABASE",
            "HONE_POSTGRES_PROXY",
            "HONE_POSTGRES_NO_PROXY",
            "HONE_OSS_PROVIDER",
            "HONE_OSS_ACCESS_KEY_ID",
            "HONE_OSS_ACCESS_KEY_SECRET",
            "HONE_OSS_BUCKET",
            "HONE_OSS_ENDPOINT",
            "HONE_OSS_REGION",
            "HONE_OSS_PROXY",
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
            runtime_dir: "/tmp/runtime".to_string(),
            conversation: RunnerConversationInput::StructuredReplay {
                system_prompt: "system".to_string(),
                current_user_turn: "input".to_string(),
                context: AgentContext::new("session-1".to_string()),
            },
            timeout: Some(Duration::from_secs(30)),
            gemini_stream: GeminiStreamOptions::default(),
            session_metadata: HashMap::new(),
            session_metadata_checkpoint: None,
            working_directory: ".".to_string(),
            allowed_tools: Some(vec![
                "discover_skills".to_string(),
                "skill_tool".to_string(),
            ]),
            max_tool_calls: Some(3),
            tool_call_limits: None,
            agent_owned_finance_loop: false,
            preloaded_evidence_calls: 0,
            service_owned_initial_prefix: None,
            terminal_stream_policy: Default::default(),
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
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_MCP_WORKING_DIRECTORY")
                && entry.get("value").and_then(|v| v.as_str()) == Some(".")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_SKILLS_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp/hone-skills")
        }));
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_AGENT_SANDBOX_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp/hone-sandboxes")
        }));
        assert!(!env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_MCP_REQUIRE_EARNINGS_EVIDENCE")
        }));
    }

    #[test]
    fn hone_mcp_servers_enables_evidence_gate_only_for_verified_earnings_turns() {
        let _guard = env_lock();
        clear_test_env();
        set_test_mcp_binary_override();
        let mut request = make_request();
        request.session_metadata.insert(
            REQUIRE_EARNINGS_PDF_COMPLETION_METADATA_KEY.to_string(),
            Value::Bool(true),
        );

        let payload = hone_mcp_servers(&request).expect("payload");
        let env_entries = payload[0]["env"].as_array().expect("env entries");
        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(Value::as_str) == Some("HONE_MCP_REQUIRE_EARNINGS_EVIDENCE")
                && entry.get("value").and_then(Value::as_str) == Some("1")
        }));
    }

    #[test]
    fn hone_mcp_servers_derives_data_dir_from_runtime_dir_when_env_missing() {
        let _guard = env_lock();
        clear_test_env();
        set_test_mcp_binary_override();

        let payload = hone_mcp_servers(&make_request()).expect("payload");
        let server = payload
            .as_array()
            .and_then(|items| items.first())
            .expect("server entry");
        let env_entries = server
            .get("env")
            .and_then(|value| value.as_array())
            .expect("env entries");

        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_DATA_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp")
        }));
    }

    #[test]
    fn hone_mcp_servers_derives_skills_dir_from_config_location() {
        let _guard = env_lock();
        clear_test_env();
        set_test_mcp_binary_override();
        let root = temp_root("hone_mcp_derived_skills_dir");
        std::fs::create_dir_all(&root).expect("create config root");
        let config_path = root.join("config.yaml");
        std::fs::write(&config_path, "{}\n").expect("write config");
        let mut request = make_request();
        request.config_path = config_path.to_string_lossy().to_string();

        let payload = hone_mcp_servers(&request).expect("payload");
        let env_entries = payload[0]["env"].as_array().expect("env entries");
        let expected = root.join("skills").to_string_lossy().to_string();

        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(Value::as_str) == Some("HONE_SKILLS_DIR")
                && entry.get("value").and_then(Value::as_str) == Some(expected.as_str())
        }));
        clear_test_env();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn hone_mcp_servers_absolutizes_relative_hone_data_dir_env() {
        let _guard = env_lock();
        clear_test_env();
        set_test_mcp_binary_override();
        let previous_dir = env::current_dir().expect("cwd");
        let temp = tempfile::tempdir().expect("tempdir");
        env::set_current_dir(temp.path()).expect("chdir");
        unsafe {
            env::set_var("HONE_DATA_DIR", "data");
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
        let actual = env_entries
            .iter()
            .find(|entry| entry.get("name").and_then(|v| v.as_str()) == Some("HONE_DATA_DIR"))
            .and_then(|entry| entry.get("value").and_then(|v| v.as_str()))
            .map(str::to_string)
            .expect("HONE_DATA_DIR");
        let expected = temp
            .path()
            .canonicalize()
            .expect("canonical temp path")
            .join("data");

        env::set_current_dir(previous_dir).expect("restore cwd");
        assert_eq!(actual, expected.to_string_lossy());
    }

    #[test]
    fn hone_mcp_servers_ignores_empty_hone_data_dir_env_and_uses_runtime_dir() {
        let _guard = env_lock();
        clear_test_env();
        set_test_mcp_binary_override();
        unsafe {
            env::set_var("HONE_DATA_DIR", "");
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

        assert!(env_entries.iter().any(|entry| {
            entry.get("name").and_then(|v| v.as_str()) == Some("HONE_DATA_DIR")
                && entry.get("value").and_then(|v| v.as_str()) == Some("/tmp")
        }));
    }

    #[test]
    fn hone_mcp_servers_absolutizes_relative_runtime_dir_before_deriving_data_dir() {
        let _guard = env_lock();
        clear_test_env();
        set_test_mcp_binary_override();
        let previous_dir = env::current_dir().expect("cwd");
        let temp = tempfile::tempdir().expect("tempdir");
        env::set_current_dir(temp.path()).expect("chdir");

        let mut request = make_request();
        request.runtime_dir = "data/runtime".to_string();

        let payload = hone_mcp_servers(&request).expect("payload");
        let server = payload
            .as_array()
            .and_then(|items| items.first())
            .expect("server entry");
        let env_entries = server
            .get("env")
            .and_then(|value| value.as_array())
            .expect("env entries");
        let actual = env_entries
            .iter()
            .find(|entry| entry.get("name").and_then(|v| v.as_str()) == Some("HONE_DATA_DIR"))
            .and_then(|entry| entry.get("value").and_then(|v| v.as_str()))
            .map(str::to_string)
            .expect("HONE_DATA_DIR");
        let expected = temp
            .path()
            .canonicalize()
            .expect("canonical temp path")
            .join("data");

        env::set_current_dir(previous_dir).expect("restore cwd");
        assert_eq!(actual, expected.to_string_lossy());
    }

    #[test]
    fn hone_mcp_servers_exports_configured_cloud_runtime_env() {
        let _guard = env_lock();
        clear_test_env();
        let unique = format!(
            "{}_{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        );
        let pg_url_env = format!("HONE_TEST_MCP_DATABASE_URL_{unique}");
        let oss_key_env = format!("HONE_TEST_MCP_OSS_KEY_{unique}");
        let oss_secret_env = format!("HONE_TEST_MCP_OSS_SECRET_{unique}");
        let oss_bucket_env = format!("HONE_TEST_MCP_OSS_BUCKET_{unique}");
        let oss_endpoint_env = format!("HONE_TEST_MCP_OSS_ENDPOINT_{unique}");
        let temp_dir = temp_root("hone_mcp_cloud_env");
        std::fs::create_dir_all(&temp_dir).expect("temp dir");
        let config_path = temp_dir.join("config.yaml");
        std::fs::write(
            &config_path,
            format!(
                r#"
cloud:
  mode: cloud
  postgres:
    database_url_env: "{pg_url_env}"
  oss:
    access_key_id_env: "{oss_key_env}"
    access_key_secret_env: "{oss_secret_env}"
    bucket_env: "{oss_bucket_env}"
    endpoint_env: "{oss_endpoint_env}"
"#
            ),
        )
        .expect("write config");
        unsafe {
            env::set_var("HONE_MCP_BIN", "/tmp/hone-mcp-custom");
            env::set_var("HONE_CLOUD_MODE", "cloud");
            env::set_var(
                &pg_url_env,
                "postgres://user:pass@example.invalid:5432/hone",
            );
            env::set_var(&oss_key_env, "oss-key");
            env::set_var(&oss_secret_env, "oss-secret");
            env::set_var(&oss_bucket_env, "oss-bucket");
            env::set_var(&oss_endpoint_env, "https://oss.example.invalid");
        }

        let mut request = make_request();
        request.config_path = config_path.to_string_lossy().to_string();

        let payload = hone_mcp_servers(&request).expect("payload");
        let env_entries = payload[0]["env"].as_array().expect("env entries");
        let env_value = |name: &str| {
            env_entries
                .iter()
                .find(|entry| entry.get("name").and_then(|v| v.as_str()) == Some(name))
                .and_then(|entry| entry.get("value").and_then(|v| v.as_str()))
                .map(|value| value.to_string())
        };

        assert_eq!(env_value("HONE_CLOUD_MODE").as_deref(), Some("cloud"));
        assert_eq!(
            env_value(&pg_url_env).as_deref(),
            Some("postgres://user:pass@example.invalid:5432/hone")
        );
        assert_eq!(env_value(&oss_key_env).as_deref(), Some("oss-key"));
        assert_eq!(env_value(&oss_secret_env).as_deref(), Some("oss-secret"));
        assert_eq!(env_value(&oss_bucket_env).as_deref(), Some("oss-bucket"));
        assert_eq!(
            env_value(&oss_endpoint_env).as_deref(),
            Some("https://oss.example.invalid")
        );

        unsafe {
            env::remove_var(pg_url_env);
            env::remove_var(oss_key_env);
            env::remove_var(oss_secret_env);
            env::remove_var(oss_bucket_env);
            env::remove_var(oss_endpoint_env);
        }
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
    fn tool_call_budget_rejects_calls_after_session_limit() {
        let _guard = env_lock();
        clear_test_env();
        let session_id = "mcp-budget-test-session";
        tool_call_counters()
            .lock()
            .expect("tool_call_counters lock")
            .remove(session_id);
        unsafe {
            env::set_var("HONE_MCP_SESSION_ID", session_id);
            env::set_var("HONE_MCP_MAX_TOOL_CALLS", "1");
        }

        assert!(consume_tool_call_budget().is_none());

        let rejected = consume_tool_call_budget().expect("limit error");
        assert_eq!(rejected["isError"], Value::Bool(true));
        assert_eq!(
            rejected["content"][0]["text"],
            Value::String("tool call limit reached (1)".to_string())
        );
    }

    #[test]
    fn text_excerpt_for_log_redacts_common_secrets() {
        let excerpt = text_excerpt_for_log(
            "request failed https://api.test/path?api_key=abc&token=def auth=Bearer bearer-secret apiKey: header-secret OPENROUTER_API_KEY=env-secret Authorization: Basic basic-secret",
            320,
        );
        assert_eq!(
            excerpt,
            "request failed https://api.test/path?api_key=<redacted>&token=<redacted> auth=Bearer <redacted> apiKey: <redacted> OPENROUTER_API_KEY=<redacted> Authorization: Basic <redacted>"
        );
    }

    #[test]
    fn value_excerpt_for_log_redacts_extended_secret_keys() {
        let excerpt = value_excerpt_for_log(
            &json!({
                "client_secret": "json-client",
                "refresh_token": "json-refresh",
                "authorization": "Basic json-basic",
                "nested": {
                    "bot_token": "json-bot",
                    "X-API-Key": "json-header",
                    "safe": "kept",
                },
            }),
            500,
        );

        assert!(excerpt.contains("\"client_secret\":\"<redacted>\""));
        assert!(excerpt.contains("\"refresh_token\":\"<redacted>\""));
        assert!(excerpt.contains("\"authorization\":\"<redacted>\""));
        assert!(excerpt.contains("\"bot_token\":\"<redacted>\""));
        assert!(excerpt.contains("\"X-API-Key\":\"<redacted>\""));
        assert!(excerpt.contains("\"safe\":\"kept\""));
        assert!(!excerpt.contains("json-client"));
        assert!(!excerpt.contains("json-refresh"));
        assert!(!excerpt.contains("json-basic"));
        assert!(!excerpt.contains("json-bot"));
        assert!(!excerpt.contains("json-header"));
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
            false,
            &mut EarningsEvidenceLedger::default(),
        ));
        assert_eq!(call_payload["isError"], Value::Bool(true));
        assert_eq!(
            call_payload["content"][0]["text"],
            Value::String("tool `cron_job` is not allowed in this stage".to_string())
        );
    }

    #[test]
    fn real_mcp_skill_tool_name_is_blocked_before_renderer_without_manifest() {
        let _guard = env_lock();
        clear_test_env();
        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("web", "earnings-user", None::<String>).expect("actor");
        let registry = core.create_tool_registry(Some(&actor), "earnings-user", false);
        let payload = futures::executor::block_on(handle_tools_call(
            &registry,
            &json!({
                "name": "skill_tool",
                "arguments": {
                    "skill_name": "earnings-research",
                    "execute_script": true,
                    "script": "scripts/render_report_pdf.py",
                    "script_payload": {
                        "company": "CRWV",
                        "mode": "preview",
                        "report_markdown": "# CRWV\n\n营收为10亿美元。\n\n来源：[IR](https://ir.example.test/results)"
                    }
                }
            }),
            true,
            &mut EarningsEvidenceLedger::default(),
        ));

        assert_eq!(payload["isError"], Value::Bool(false));
        assert_eq!(payload["structuredContent"]["success"], Value::Bool(false));
        assert_eq!(
            payload["structuredContent"]["side_effect_status"],
            Value::String("not_started".to_string())
        );
        assert_eq!(
            payload["structuredContent"]["render_success"],
            Value::Bool(false)
        );
        assert!(
            payload["structuredContent"]["render_error"]
                .as_str()
                .is_some_and(|error| error.contains("evidence_manifest"))
        );
    }

    fn evidence_ledger() -> EarningsEvidenceLedger {
        let mut ledger = EarningsEvidenceLedger::default();
        ledger.record_tool_result(
            "web_search",
            &json!({
                "results": [
                    {
                        "url": "https://ir.example.test/q1-2026",
                        "title": "Q1 2026 results",
                        "content": "Revenue backlog reached nearly $100 billion and active power surpassed 1 GW. Full-year revenue guidance is $12 billion to $13 billion."
                    },
                    {
                        "url": "https://analyst.example.test/consensus",
                        "title": "Analyst consensus",
                        "content": "The average price target is $124 and the rating is Moderate Buy."
                    }
                ]
            }),
        );
        ledger
    }

    #[test]
    fn earnings_evidence_gate_accepts_visible_current_turn_claim_mapping() {
        let report = "# CRWV 财报前瞻\n\n积压订单接近1000亿美元，活跃电力超过1GW。\n\n来源：[Q1 2026 results](https://ir.example.test/q1-2026)";
        let arguments = json!({
            "skill_name": "earnings-research",
            "execute_script": true,
            "script_payload": {
                "report_markdown": report,
                "evidence_manifest": [{
                    "claim_text": "积压订单接近1000亿美元，活跃电力超过1GW。",
                    "source_url": "https://ir.example.test/q1-2026",
                    "source_excerpt": "Revenue backlog reached nearly $100 billion and active power surpassed 1 GW."
                }]
            }
        });

        if let Err(error) = validate_earnings_renderer_evidence(&arguments, &evidence_ledger()) {
            panic!("expected valid evidence manifest: {error}");
        }
    }

    #[test]
    fn earnings_evidence_gate_rejects_numeric_claim_missing_from_exact_source_excerpt() {
        let report = "# CRWV 财报前瞻\n\n管理层将2026年底ARR预期下限上调至180亿美元。\n\n来源：[Q1 2026 results](https://ir.example.test/q1-2026)";
        let arguments = json!({
            "skill_name": "earnings-research",
            "execute_script": true,
            "script_payload": {
                "report_markdown": report,
                "evidence_manifest": [{
                    "claim_text": "管理层将2026年底ARR预期下限上调至180亿美元。",
                    "source_url": "https://ir.example.test/q1-2026",
                    "source_excerpt": "Full-year revenue guidance is $12 billion to $13 billion."
                }]
            }
        });

        let error = validate_earnings_renderer_evidence(&arguments, &evidence_ledger())
            .expect_err("unsupported ARR must fail");
        assert!(error.contains("证据摘录未覆盖数字"));
        assert!(error.contains("18"));
    }

    #[test]
    fn earnings_evidence_gate_normalizes_million_billion_and_chinese_yi_units() {
        let source_url = "https://ir.example.test/q4-guidance";
        let source_excerpt =
            "Revenue guidance is $960 million to $1.01 billion and EPS is $2.85 to $3.05.";
        let mut ledger = EarningsEvidenceLedger::default();
        ledger.record_tool_result(
            "web_search",
            &json!({
                "results": [{
                    "url": source_url,
                    "content": source_excerpt
                }]
            }),
        );
        let valid_claim = "公司指引为营收9.6亿至10.1亿美元，EPS为2.85至3.05美元。";
        let arguments = json!({
            "skill_name": "earnings-research",
            "execute_script": true,
            "script_payload": {
                "report_markdown": format!("# LITE 财报前瞻\n\n{valid_claim}\n\n来源：[{source_url}]({source_url})"),
                "evidence_manifest": [{
                    "claim_text": valid_claim,
                    "source_url": source_url,
                    "source_excerpt": source_excerpt
                }]
            }
        });
        if let Err(error) = validate_earnings_renderer_evidence(&arguments, &ledger) {
            panic!("equivalent million/billion/亿 values should pass: {error}");
        }

        let bad_claim = "公司指引为营收9.6亿至1.01亿美元，EPS为2.85至3.05美元。";
        let mut bad_arguments = arguments;
        bad_arguments["script_payload"]["report_markdown"] = Value::String(format!(
            "# LITE 财报前瞻\n\n{bad_claim}\n\n来源：[{source_url}]({source_url})"
        ));
        bad_arguments["script_payload"]["evidence_manifest"][0]["claim_text"] =
            Value::String(bad_claim.to_string());
        let error = validate_earnings_renderer_evidence(&bad_arguments, &ledger)
            .expect_err("the observed LITE billion-to-亿 magnitude error must fail");
        assert!(error.contains("证据摘录未覆盖数字"));
        assert!(error.contains("0.101"));
    }

    #[test]
    fn earnings_evidence_gate_rejects_unmapped_material_line_and_unseen_url() {
        let report = "# CRWV 财报前瞻\n\n积压订单接近1000亿美元。\n\n分析师将评级下调至中性。\n\n来源：[Q1](https://ir.example.test/q1-2026) [伪来源](https://fake.example.test/story)";
        let arguments = json!({
            "skill_name": "earnings-research",
            "execute_script": true,
            "script_payload": {
                "report_markdown": report,
                "evidence_manifest": [{
                    "claim_text": "积压订单接近1000亿美元。",
                    "source_url": "https://ir.example.test/q1-2026",
                    "source_excerpt": "Revenue backlog reached nearly $100 billion"
                }]
            }
        });

        let error = validate_earnings_renderer_evidence(&arguments, &evidence_ledger())
            .expect_err("unseen report URL must fail first");
        assert!(error.contains("报告来源 URL 不在本轮工具结果中"));

        let clean_report = report.replace(" [伪来源](https://fake.example.test/story)", "");
        let mut clean_arguments = arguments;
        clean_arguments["script_payload"]["report_markdown"] = Value::String(clean_report);
        let error = validate_earnings_renderer_evidence(&clean_arguments, &evidence_ledger())
            .expect_err("unmapped rating line must fail");
        assert!(error.contains("缺少证据映射"));
        assert!(error.contains("评级下调"));
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
