//! POC smoke for the proposed `llm.providers` + `llm.profiles` config shape.
//!
//! Parse-only:
//!   cargo run -p hone-llm --example llm_profile_poc
//!
//! Live OpenRouter request:
//!   RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc

use hone_core::config::{HoneConfig, LlmProfileEntryConfig, LlmProviderEntryConfig};
use serde_json::{Map, Value, json};
use std::{env, error::Error, io, path::PathBuf};

const DEFAULT_POC_CONFIG: &str = "tests/fixtures/llm/profile_poc.yaml";
const DEFAULT_PROFILE: &str = "poc_reasoning_json";
const DEFAULT_OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_path = env::args()
        .nth(1)
        .or_else(|| env::var("HONE_LLM_PROFILE_POC_CONFIG").ok())
        .unwrap_or_else(|| DEFAULT_POC_CONFIG.to_string());
    let profile_name = env::args()
        .nth(2)
        .or_else(|| env::var("HONE_LLM_PROFILE_POC_PROFILE").ok())
        .unwrap_or_else(|| DEFAULT_PROFILE.to_string());

    let config = HoneConfig::from_file(&config_path)?;
    let profile = config
        .llm
        .profiles
        .get(&profile_name)
        .ok_or_else(|| boxed_err(format!("missing llm.profiles.{profile_name}")))?;
    if profile.provider.trim().is_empty() {
        return Err(boxed_err(format!(
            "llm.profiles.{profile_name}.provider is required"
        )));
    }
    if profile.model.trim().is_empty() {
        return Err(boxed_err(format!(
            "llm.profiles.{profile_name}.model is required"
        )));
    }

    let provider_name = profile.provider.trim();
    let provider = config
        .llm
        .providers
        .get(provider_name)
        .ok_or_else(|| boxed_err(format!("missing llm.providers.{provider_name}")))?;
    let mut body = build_chat_body(profile, provider_name);
    let request_keys = sorted_keys(&body);

    println!(
        "[PASS] parsed profile={profile_name} provider={provider_name} kind={} model={} request_keys={}",
        provider.kind,
        profile.model,
        request_keys.join(",")
    );

    if env::var("RUN_LLM_PROFILE_POC").ok().as_deref() != Some("1") {
        println!("[INFO] set RUN_LLM_PROFILE_POC=1 to send one live LLM request");
        return Ok(());
    }

    let secrets_config = load_secrets_config();
    let api_key = resolve_api_key(provider_name, provider, secrets_config.as_ref())?;
    let base_url = resolve_base_url(provider_name, provider)?;
    let timeout = provider.timeout.unwrap_or(60).max(1);
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://openrouter.ai")
        .header("X-Title", "Hone LLM profile POC")
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    let raw = response.text().await?;
    if !status.is_success() {
        return Err(boxed_err(format!(
            "upstream HTTP {}: {}",
            status.as_u16(),
            truncate(&raw, 600)
        )));
    }

    let value: Value = serde_json::from_str(&raw)?;
    let choice = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| boxed_err("missing choices[0]"))?;
    let message = choice
        .get("message")
        .and_then(Value::as_object)
        .ok_or_else(|| boxed_err("missing choices[0].message"))?;
    let content = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let parsed_content: Value = serde_json::from_str(content).map_err(|err| {
        boxed_err(format!(
            "content was not JSON: {err}; content={}",
            truncate(content, 300)
        ))
    })?;
    let status_text = parsed_content
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("");
    if status_text != "ok" {
        return Err(boxed_err(format!(
            "unexpected response status={status_text}; content={parsed_content}"
        )));
    }

    let finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let reasoning_present = message.get("reasoning").and_then(Value::as_str).is_some()
        || message.get("reasoning_details").is_some();
    let usage = value.get("usage").cloned().unwrap_or(Value::Null);

    println!(
        "[PASS] live profile request accepted finish_reason={finish_reason} reasoning_present={reasoning_present} usage={}",
        sanitized_usage(&usage)
    );

    body.clear();
    Ok(())
}

fn build_chat_body(profile: &LlmProfileEntryConfig, provider_name: &str) -> Map<String, Value> {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(profile.model.clone()));
    body.insert(
        "messages".to_string(),
        json!([
            {
                "role": "system",
                "content": "Return only a compact JSON object. Do not include markdown."
            },
            {
                "role": "user",
                "content": "Return {\"status\":\"ok\",\"profile\":\"poc_reasoning_json\"}."
            }
        ]),
    );

    if let Ok(Value::Object(params)) = serde_json::to_value(&profile.params) {
        for (key, value) in params {
            if key == "extra_body" {
                if let Value::Object(extra) = value {
                    merge_json_object(&mut body, extra);
                }
                continue;
            }
            if !is_empty_json_value(&value) {
                body.insert(key, value);
            }
        }
    }

    for (key, value) in &profile.params.extra_body {
        body.insert(key.clone(), value.clone());
    }
    if let Some(options) = profile.provider_options.get(provider_name) {
        for (key, value) in &options.extra_body {
            body.insert(key.clone(), value.clone());
        }
    }
    body
}

fn load_secrets_config() -> Option<HoneConfig> {
    let path = env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let path = PathBuf::from(path);
    if path.exists() {
        HoneConfig::from_file(path).ok()
    } else {
        None
    }
}

fn resolve_api_key(
    provider_name: &str,
    provider: &LlmProviderEntryConfig,
    secrets_config: Option<&HoneConfig>,
) -> Result<String, Box<dyn Error>> {
    let direct = provider.api_key.trim();
    if !direct.is_empty() {
        return Ok(direct.to_string());
    }
    if let Some(key) = provider.api_keys.iter().find(|key| !key.trim().is_empty()) {
        return Ok(key.trim().to_string());
    }
    let env_name = provider.api_key_env.trim();
    if !env_name.is_empty() {
        if let Ok(key) = env::var(env_name) {
            let key = key.trim().to_string();
            if !key.is_empty() {
                return Ok(key);
            }
        }
    }

    if provider_name == "openrouter" {
        if let Some(config) = secrets_config {
            let pool = config.llm.openrouter.effective_key_pool();
            if let Some(key) = pool.keys().iter().find(|key| !key.trim().is_empty()) {
                return Ok(key.trim().to_string());
            }
            let env_name = config.llm.openrouter.api_key_env.trim();
            if !env_name.is_empty() {
                if let Ok(key) = env::var(env_name) {
                    let key = key.trim().to_string();
                    if !key.is_empty() {
                        return Ok(key);
                    }
                }
            }
        }
    }

    Err(boxed_err(format!(
        "missing API key for provider {provider_name}"
    )))
}

fn resolve_base_url(
    provider_name: &str,
    provider: &LlmProviderEntryConfig,
) -> Result<String, Box<dyn Error>> {
    let base_url = provider.base_url.trim();
    if !base_url.is_empty() {
        return Ok(base_url.to_string());
    }
    if provider_name == "openrouter" {
        return Ok(DEFAULT_OPENROUTER_BASE_URL.to_string());
    }
    Err(boxed_err(format!(
        "llm.providers.{provider_name}.base_url is required"
    )))
}

fn merge_json_object(body: &mut Map<String, Value>, extra: Map<String, Value>) {
    for (key, value) in extra {
        body.insert(key, value);
    }
}

fn is_empty_json_value(value: &Value) -> bool {
    value.is_null()
        || matches!(value, Value::Array(items) if items.is_empty())
        || matches!(value, Value::Object(map) if map.is_empty())
}

fn sorted_keys(map: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn sanitized_usage(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for key in ["prompt_tokens", "completion_tokens", "total_tokens", "cost"] {
                if let Some(value) = map.get(key) {
                    out.insert(key.to_string(), value.clone());
                }
            }
            Value::Object(out).to_string()
        }
        _ => "{}".to_string(),
    }
}

fn boxed_err(message: impl Into<String>) -> Box<dyn Error> {
    io::Error::other(message.into()).into()
}
