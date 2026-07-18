use async_trait::async_trait;
use hone_agent_gemini_cli::{GeminiCliAgent, GeminiStreamEvent, parse_stream_event};
use hone_core::agent::{AgentMessage, ToolCallMade};
use hone_tools::ToolRegistry;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncBufReadExt;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind, GeminiStreamOptions};

use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

const GEMINI_CLI_STDERR_DETAIL_CHARS: usize = 400;

pub(crate) struct GeminiCliRunner {
    system_prompt: String,
    tool_registry: Arc<ToolRegistry>,
    timeouts: RunnerTimeouts,
}

impl GeminiCliRunner {
    pub(crate) fn new(
        system_prompt: String,
        tool_registry: Arc<ToolRegistry>,
        timeouts: RunnerTimeouts,
    ) -> Self {
        Self {
            system_prompt,
            tool_registry,
            timeouts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeminiCliToolRenderPhase {
    Start,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeminiCliRenderedToolStatus {
    pub tool: String,
    pub message: Option<String>,
    pub reasoning: Option<String>,
}

pub(crate) fn render_gemini_cli_tool_status(
    tool_name: &str,
    tool_args: &Value,
    tool_reasoning: Option<String>,
    phase: GeminiCliToolRenderPhase,
) -> GeminiCliRenderedToolStatus {
    let label = render_gemini_cli_tool_label(tool_name, tool_args);
    match phase {
        GeminiCliToolRenderPhase::Start => GeminiCliRenderedToolStatus {
            tool: label.clone(),
            message: None,
            reasoning: Some(render_gemini_cli_reasoning(&label, tool_reasoning)),
        },
        GeminiCliToolRenderPhase::Done => GeminiCliRenderedToolStatus {
            tool: label.clone(),
            message: Some(format!("执行完成：{label}")),
            reasoning: None,
        },
    }
}

pub(crate) fn append_gemini_cli_tool_context_messages(
    messages: &mut Vec<AgentMessage>,
    call_id: &str,
    visible_text: &str,
    tool_name: &str,
    tool_args: &Value,
    tool_result: &str,
) {
    messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some(visible_text.trim().to_string()),
        tool_calls: Some(vec![build_gemini_cli_tool_call_value(
            call_id, tool_name, tool_args,
        )]),
        tool_call_id: None,
        name: None,
        metadata: None,
    });
    messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some(tool_result.to_string()),
        tool_calls: None,
        tool_call_id: Some(call_id.to_string()),
        name: Some(tool_name.to_string()),
        metadata: None,
    });
}

fn append_gemini_cli_final_message(messages: &mut Vec<AgentMessage>, content: &str) {
    if content.trim().is_empty() {
        return;
    }
    messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
        metadata: None,
    });
}

fn build_gemini_cli_tool_call_value(call_id: &str, tool_name: &str, tool_args: &Value) -> Value {
    json!({
        "id": call_id,
        "type": "function",
        "function": {
            "name": tool_name,
            "arguments": stringify_gemini_cli_tool_arguments(tool_args),
        }
    })
}

fn stringify_gemini_cli_tool_arguments(tool_args: &Value) -> String {
    serde_json::to_string(tool_args).unwrap_or_else(|_| "null".to_string())
}

fn render_gemini_cli_reasoning(label: &str, tool_reasoning: Option<String>) -> String {
    let base = format!("正在执行：{label}");
    let note = tool_reasoning
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_gemini_cli_detail(value, 120));
    match note {
        Some(note) if note != base => format!("{base}；说明：{note}"),
        _ => base,
    }
}

fn render_gemini_cli_tool_label(tool_name: &str, tool_args: &Value) -> String {
    let base = match tool_name {
        "web_search" => {
            if let Some(query) = tool_arg_string(tool_args, &["query", "q"]) {
                format!(
                    "web_search query=\"{}\"",
                    truncate_gemini_cli_detail(&query, 80)
                )
            } else {
                "web_search".to_string()
            }
        }
        "data_fetch" => {
            let data_type = tool_arg_string(tool_args, &["data_type"]);
            let symbol = tool_arg_string(tool_args, &["symbol", "ticker"]);
            match (data_type, symbol) {
                (Some(data_type), Some(symbol)) => format!(
                    "data_fetch {} {}",
                    truncate_gemini_cli_detail(&data_type, 32),
                    truncate_gemini_cli_detail(&symbol, 32)
                ),
                (Some(data_type), None) => {
                    format!("data_fetch {}", truncate_gemini_cli_detail(&data_type, 48))
                }
                (None, Some(symbol)) => {
                    format!("data_fetch {}", truncate_gemini_cli_detail(&symbol, 48))
                }
                (None, None) => render_generic_gemini_cli_tool_label(tool_name, tool_args),
            }
        }
        "deep_research" => {
            if let Some(company) = tool_arg_string(tool_args, &["company_name", "query"]) {
                format!("deep_research {}", truncate_gemini_cli_detail(&company, 72))
            } else {
                render_generic_gemini_cli_tool_label(tool_name, tool_args)
            }
        }
        "skill_tool" | "load_skill" => {
            let action = tool_arg_string(tool_args, &["action"]);
            let skill = tool_arg_string(tool_args, &["skill_name"]);
            match (action, skill) {
                (Some(action), Some(skill)) => format!(
                    "{tool_name} {} {}",
                    truncate_gemini_cli_detail(&action, 24),
                    truncate_gemini_cli_detail(&skill, 48)
                ),
                (Some(action), None) => {
                    format!("{tool_name} {}", truncate_gemini_cli_detail(&action, 48))
                }
                (None, Some(skill)) => {
                    format!("{tool_name} {}", truncate_gemini_cli_detail(&skill, 48))
                }
                (None, None) => render_generic_gemini_cli_tool_label(tool_name, tool_args),
            }
        }
        "portfolio" => {
            let action = tool_arg_string(tool_args, &["action"]);
            let symbol = tool_arg_string(tool_args, &["symbol", "ticker"]);
            match (action, symbol) {
                (Some(action), Some(symbol)) => format!(
                    "portfolio {} {}",
                    truncate_gemini_cli_detail(&action, 24),
                    truncate_gemini_cli_detail(&symbol, 24)
                ),
                _ => render_generic_gemini_cli_tool_label(tool_name, tool_args),
            }
        }
        _ => render_generic_gemini_cli_tool_label(tool_name, tool_args),
    };

    truncate_gemini_cli_detail(&base, 120)
}

fn render_generic_gemini_cli_tool_label(tool_name: &str, tool_args: &Value) -> String {
    let summary = summarize_gemini_cli_arguments(tool_args);
    if summary.is_empty() {
        tool_name.to_string()
    } else {
        format!("{tool_name} {summary}")
    }
}

fn summarize_gemini_cli_arguments(tool_args: &Value) -> String {
    let Value::Object(map) = tool_args else {
        return String::new();
    };
    let mut pairs = Vec::new();
    for key in [
        "query",
        "q",
        "symbol",
        "ticker",
        "company_name",
        "skill_name",
        "action",
        "data_type",
        "path",
        "file_path",
        "url",
    ] {
        if let Some(value) = map.get(key) {
            let rendered = summarize_gemini_cli_argument_value(value);
            if !rendered.is_empty() {
                pairs.push(format!("{key}={rendered}"));
            }
        }
        if pairs.len() >= 2 {
            break;
        }
    }
    pairs.join(" ")
}

fn summarize_gemini_cli_argument_value(value: &Value) -> String {
    match value {
        Value::String(text) => format!("\"{}\"", truncate_gemini_cli_detail(text, 48)),
        Value::Number(number) => number.to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Array(items) => {
            if items.is_empty() {
                "[]".to_string()
            } else {
                format!("[{} items]", items.len())
            }
        }
        Value::Object(map) => format!("{{{} keys}}", map.len()),
        Value::Null => "null".to_string(),
    }
}

fn tool_arg_string(tool_args: &Value, keys: &[&str]) -> Option<String> {
    let Value::Object(map) = tool_args else {
        return None;
    };
    for key in keys {
        if let Some(value) = map.get(*key).and_then(|value| value.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn truncate_gemini_cli_detail(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let total = trimmed.chars().count();
    if total <= max_chars {
        return trimmed.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let prefix = trimmed.chars().take(keep).collect::<String>();
    format!("{prefix}…")
}

fn gemini_cli_exit_error_message(code: Option<i32>, stderr: &str) -> String {
    match gemini_cli_stderr_detail(stderr) {
        Some(stderr_detail) => {
            format!("gemini exited with error (code={code:?}; stderr={stderr_detail})")
        }
        None => format!("gemini exited with error (code={code:?}; stderr=<empty>)"),
    }
}

fn gemini_cli_stderr_detail(stderr: &str) -> Option<String> {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(truncate_gemini_cli_detail(
        &redact_common_stderr_secrets(trimmed),
        GEMINI_CLI_STDERR_DETAIL_CHARS,
    ))
}

fn redact_common_stderr_secrets(text: &str) -> String {
    let mut output = redact_marker_value(text, "Bearer ");
    output = redact_marker_value(&output, "Basic ");
    for key in SENSITIVE_GEMINI_STDERR_KEYS {
        output = redact_marker_value(&output, &format!("{key}="));
        output = redact_marker_value(&output, &format!("{key}:"));
        output = redact_json_string_field(&output, key);
    }
    for key in ["authorization", "Authorization"] {
        output = redact_json_string_field(&output, key);
    }
    output
}

const SENSITIVE_GEMINI_STDERR_KEYS: &[&str] = &[
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

fn redact_json_string_field(text: &str, key: &str) -> String {
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

#[async_trait]
impl AgentRunner for GeminiCliRunner {
    fn name(&self) -> &'static str {
        "gemini_cli"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let mut context = request.context;
        let mut pending_tool_results: Vec<(String, String, String)> = Vec::new();
        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        let mut context_messages: Vec<AgentMessage> = Vec::new();
        let mut full_reply = String::new();
        let mut iteration = 0u32;
        let mut hit_max_iterations = false;
        let mut total_raw_lines_seen = 0u32;
        let mut last_iteration_output = String::new();
        let mut final_assistant_content: Option<String> = None;
        let mut stream_options = request.gemini_stream.clone();
        stream_options.overall_timeout = self.timeouts.overall;
        stream_options.per_line_timeout = self.timeouts.step;

        loop {
            if iteration >= stream_options.max_iterations {
                hit_max_iterations = true;
                break;
            }
            iteration += 1;

            if iteration == 1 {
                context.add_user_message(&request.runtime_input);
            }

            let prompt = GeminiCliAgent::build_streaming_prompt(
                &self.system_prompt,
                &context,
                &self.tool_registry,
                &pending_tool_results,
            );

            emitter
                .emit(AgentRunnerEvent::Progress {
                    stage: "gemini.spawn",
                    detail: Some(format!("iteration={iteration}")),
                })
                .await;

            let iteration_output = match stream_gemini_prompt(
                &prompt,
                &request.actor_label,
                &request.working_directory,
                iteration,
                &stream_options,
                &mut full_reply,
                &mut total_raw_lines_seen,
                emitter.clone(),
            )
            .await
            {
                Ok(output) => output,
                Err(error) => {
                    emitter
                        .emit(AgentRunnerEvent::Error {
                            error: error.clone(),
                        })
                        .await;
                    return AgentRunnerResult {
                        response: hone_core::agent::AgentResponse {
                            content: full_reply,
                            tool_calls_made,
                            iterations: iteration,
                            success: false,
                            error: Some(error.message),
                        },
                        streamed_output: true,
                        committed_visible_prefix: None,
                        terminal_error_emitted: true,
                        session_metadata_updates: std::collections::HashMap::new(),
                        context_messages: None,
                    };
                }
            };

            let (visible_text, maybe_tool_call) =
                GeminiCliAgent::parse_tool_call(&iteration_output);

            if let Some((tool_name, tool_args, tool_reasoning)) = maybe_tool_call {
                let call_id = format!("gemini_cli_call_{iteration}_{}", tool_calls_made.len() + 1);
                if !visible_text.trim().is_empty() {
                    context.add_assistant_message(&visible_text, None);
                }
                let rendered_start = render_gemini_cli_tool_status(
                    &tool_name,
                    &tool_args,
                    tool_reasoning.clone(),
                    GeminiCliToolRenderPhase::Start,
                );
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "tool.execute",
                        detail: Some(rendered_start.tool.clone()),
                    })
                    .await;

                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool: rendered_start.tool.clone(),
                        status: "start".to_string(),
                        message: rendered_start.message,
                        reasoning: rendered_start.reasoning,
                    })
                    .await;

                let tool_result_value = self
                    .tool_registry
                    .execute_tool(&tool_name, tool_args.clone())
                    .await
                    .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }));
                let tool_result_str = tool_result_value.to_string();
                append_gemini_cli_tool_context_messages(
                    &mut context_messages,
                    &call_id,
                    &visible_text,
                    &tool_name,
                    &tool_args,
                    &tool_result_str,
                );
                context.add_tool_result(&call_id, &tool_name, &tool_result_str);

                let rendered_done = render_gemini_cli_tool_status(
                    &tool_name,
                    &tool_args,
                    None,
                    GeminiCliToolRenderPhase::Done,
                );
                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool: rendered_done.tool,
                        status: "done".to_string(),
                        message: rendered_done.message,
                        reasoning: rendered_done.reasoning,
                    })
                    .await;

                tool_calls_made.push(ToolCallMade {
                    name: tool_name.clone(),
                    arguments: tool_args,
                    result: tool_result_value,
                    tool_call_id: Some(call_id.clone()),
                });

                pending_tool_results.push((call_id, tool_name, tool_result_str));
                last_iteration_output = iteration_output;
                continue;
            }

            final_assistant_content = Some(visible_text);
            last_iteration_output = iteration_output;
            break;
        }

        if full_reply.trim().is_empty() && (hit_max_iterations || !pending_tool_results.is_empty())
        {
            let mut final_prompt = GeminiCliAgent::build_streaming_prompt(
                &self.system_prompt,
                &context,
                &self.tool_registry,
                &pending_tool_results,
            );
            final_prompt.push_str(
                "\n### Final Answer Required ###\n\
                You have now used all available tool calls. \
                Based on ALL the tool results above, provide your FINAL answer to the user NOW. \
                Do NOT call any more tools. \
                Respond directly and completely in Chinese.\n",
            );

            emitter
                .emit(AgentRunnerEvent::Progress {
                    stage: "gemini.final_response",
                    detail: Some(format!("iteration={iteration}")),
                })
                .await;

            if let Err(error) = stream_gemini_prompt(
                &final_prompt,
                &request.actor_label,
                &request.working_directory,
                iteration,
                &request.gemini_stream,
                &mut full_reply,
                &mut total_raw_lines_seen,
                emitter.clone(),
            )
            .await
            {
                emitter
                    .emit(AgentRunnerEvent::Error {
                        error: error.clone(),
                    })
                    .await;
                return AgentRunnerResult {
                    response: hone_core::agent::AgentResponse {
                        content: full_reply,
                        tool_calls_made,
                        iterations: iteration,
                        success: false,
                        error: Some(error.message),
                    },
                    streamed_output: true,
                    committed_visible_prefix: None,
                    terminal_error_emitted: true,
                    session_metadata_updates: std::collections::HashMap::new(),
                    context_messages: None,
                };
            }
        }

        if full_reply.trim().is_empty() {
            tracing::warn!(
                "[AgentRunner/gemini] empty stream response (raw_lines_seen={}, last_buf_preview={})",
                total_raw_lines_seen,
                last_iteration_output.chars().take(200).collect::<String>()
            );
        }
        if final_assistant_content.is_none() && !full_reply.trim().is_empty() {
            final_assistant_content = Some(full_reply.trim().to_string());
        }
        if let Some(content) = final_assistant_content.as_deref() {
            append_gemini_cli_final_message(&mut context_messages, content);
        }

        AgentRunnerResult {
            response: hone_core::agent::AgentResponse {
                content: full_reply,
                tool_calls_made,
                iterations: iteration,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: std::collections::HashMap::new(),
            context_messages: Some(context_messages),
        }
    }
}

pub(crate) async fn stream_gemini_prompt(
    prompt: &str,
    actor_label: &str,
    working_directory: &str,
    iteration: u32,
    options: &GeminiStreamOptions,
    full_reply: &mut String,
    total_raw_lines_seen: &mut u32,
    emitter: Arc<dyn AgentRunnerEmitter>,
) -> Result<String, AgentSessionError> {
    let mut child = gemini_command()
        .current_dir(working_directory)
        .arg("--prompt")
        .arg(prompt)
        .arg("--sandbox")
        .arg("--approval-mode")
        .arg("plan")
        .arg("-o")
        .arg("stream-json")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::SpawnFailed,
            message: format!("failed to spawn gemini: {e}"),
        })?;

    let stdout = child.stdout.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::StdoutUnavailable,
        message: "gemini stdout unavailable".to_string(),
    })?;

    let mut reader = tokio::io::BufReader::new(stdout).lines();
    let mut iteration_output = String::new();
    let mut visible_emitted_len = 0usize;
    let mut raw_line_count = 0u32;
    let overall_start = Instant::now();

    loop {
        if overall_start.elapsed() > options.overall_timeout {
            let _ = child.kill().await;
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::TimeoutOverall,
                message: format!(
                    "gemini stream overall timeout ({}s)",
                    options.overall_timeout.as_secs()
                ),
            });
        }

        match tokio::time::timeout(options.per_line_timeout, reader.next_line()).await {
            Ok(Ok(Some(line))) => {
                raw_line_count += 1;
                *total_raw_lines_seen += 1;
                if raw_line_count <= 5 {
                    let preview = gemini_cli_log_preview(&line, 200);
                    tracing::debug!(
                        "[AgentRunner/gemini] [{}] raw_line[iter={} n={}]: {}",
                        actor_label,
                        iteration,
                        raw_line_count,
                        preview
                    );
                }
                match parse_stream_event(&line) {
                    Some(GeminiStreamEvent::Content(chunk)) => {
                        iteration_output.push_str(&chunk);
                        let visible_prefix = match iteration_output.find("<tool_call") {
                            Some(idx) => &iteration_output[..idx],
                            None => iteration_output.as_str(),
                        };
                        if visible_prefix.len() > visible_emitted_len {
                            let delta = &visible_prefix[visible_emitted_len..];
                            visible_emitted_len = visible_prefix.len();
                            if !delta.is_empty() {
                                emitter
                                    .emit(AgentRunnerEvent::StreamDelta {
                                        content: delta.to_string(),
                                    })
                                    .await;
                                full_reply.push_str(delta);
                            }
                        }
                    }
                    Some(GeminiStreamEvent::Thought(thought)) => {
                        emitter
                            .emit(AgentRunnerEvent::StreamThought { thought })
                            .await;
                    }
                    Some(GeminiStreamEvent::Error(msg)) => {
                        let _ = child.kill().await;
                        return Err(AgentSessionError {
                            kind: AgentSessionErrorKind::GeminiError,
                            message: msg,
                        });
                    }
                    Some(GeminiStreamEvent::ContextWindowOverflow {
                        estimated,
                        remaining,
                    }) => {
                        let _ = child.kill().await;
                        return Err(AgentSessionError {
                            kind: AgentSessionErrorKind::ContextWindowOverflow,
                            message: format!(
                                "context window overflow: estimated={} remaining={}",
                                estimated, remaining
                            ),
                        });
                    }
                    Some(GeminiStreamEvent::Finished(_)) => break,
                    Some(GeminiStreamEvent::Retry) => {
                        tracing::warn!(
                            "[AgentRunner/gemini] [{}] retry event (iter={})",
                            actor_label,
                            iteration
                        );
                    }
                    Some(GeminiStreamEvent::InvalidStream) => {
                        tracing::warn!(
                            "[AgentRunner/gemini] [{}] invalid stream (iter={})",
                            actor_label,
                            iteration
                        );
                    }
                    Some(GeminiStreamEvent::ToolCallRequest(_)) => {}
                    Some(GeminiStreamEvent::Unknown(type_name)) => {
                        tracing::debug!(
                            "[AgentRunner/gemini] [{}] unknown stream event: {}",
                            actor_label,
                            type_name
                        );
                    }
                    None => {}
                }
            }
            Ok(Ok(None)) => break,
            Ok(Err(e)) => {
                return Err(AgentSessionError {
                    kind: AgentSessionErrorKind::Io,
                    message: format!("gemini stdout read error: {e}"),
                });
            }
            Err(_) => {
                let _ = child.kill().await;
                return Err(AgentSessionError {
                    kind: AgentSessionErrorKind::TimeoutPerLine,
                    message: format!(
                        "gemini per-line timeout ({}s)",
                        options.per_line_timeout.as_secs()
                    ),
                });
            }
        }
    }

    if let Ok(out) = child.wait_with_output().await {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stderr_trimmed = stderr.trim();
        if !stderr_trimmed.is_empty() {
            tracing::warn!(
                stderr_chars = stderr_trimmed.chars().count(),
                stderr_preview = %gemini_cli_stderr_detail(stderr_trimmed).unwrap_or_default(),
                "[AgentRunner/gemini] stderr"
            );
        }
        if !out.status.success() && iteration_output.is_empty() {
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::ExitFailure,
                message: gemini_cli_exit_error_message(out.status.code(), stderr_trimmed),
            });
        }
    }

    Ok(iteration_output)
}

fn gemini_cli_log_preview(text: &str, max_chars: usize) -> String {
    truncate_gemini_cli_detail(&redact_common_stderr_secrets(text), max_chars)
}

fn gemini_command() -> tokio::process::Command {
    let bin = std::env::var("HONE_GEMINI_BIN").unwrap_or_else(|_| "gemini".to_string());
    tokio::process::Command::new(bin)
}

#[cfg(test)]
mod tests {
    use super::gemini_cli_exit_error_message;

    #[test]
    fn gemini_exit_error_redacts_common_stderr_secret_shapes() {
        let message = gemini_cli_exit_error_message(
            Some(2),
            r#"request failed token: header-secret auth=Bearer bearer-secret OPENROUTER_API_KEY=env-secret Authorization: Basic basic-secret {"api_key":"json-secret","client_secret":"json-client","authorization":"Basic json-basic"}"#,
        );

        assert!(message.contains("token: <redacted>"));
        assert!(message.contains("Bearer <redacted>"));
        assert!(message.contains("OPENROUTER_API_KEY=<redacted>"));
        assert!(message.contains("Basic <redacted>"));
        assert!(message.contains("\"api_key\":\"<redacted>\""));
        assert!(message.contains("\"client_secret\":\"<redacted>\""));
        assert!(message.contains("\"authorization\":\"<redacted>\""));
        assert!(!message.contains("header-secret"));
        assert!(!message.contains("bearer-secret"));
        assert!(!message.contains("env-secret"));
        assert!(!message.contains("basic-secret"));
        assert!(!message.contains("json-secret"));
        assert!(!message.contains("json-client"));
        assert!(!message.contains("json-basic"));
    }

    #[test]
    fn gemini_raw_line_preview_redacts_common_secret_shapes() {
        let preview = super::gemini_cli_log_preview(
            r#"{"message":"token: header-secret","api_key":"json-secret","refresh_token":"json-refresh","bot_token":"json-bot"}"#,
            200,
        );

        assert!(preview.contains("token: <redacted>"));
        assert!(preview.contains("\"api_key\":\"<redacted>\""));
        assert!(preview.contains("\"refresh_token\":\"<redacted>\""));
        assert!(preview.contains("\"bot_token\":\"<redacted>\""));
        assert!(!preview.contains("header-secret"));
        assert!(!preview.contains("json-secret"));
        assert!(!preview.contains("json-refresh"));
        assert!(!preview.contains("json-bot"));
    }
}
