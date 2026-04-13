use async_trait::async_trait;
use hone_agent_gemini_cli::{GeminiCliAgent, GeminiStreamEvent, parse_stream_event};
use hone_core::agent::ToolCallMade;
use hone_tools::ToolRegistry;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncBufReadExt;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind, GeminiStreamOptions};
use crate::runtime::{get_tool_status_message, resolve_tool_reasoning};

use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

pub struct GeminiCliRunner {
    system_prompt: String,
    tool_registry: Arc<ToolRegistry>,
    timeouts: RunnerTimeouts,
}

impl GeminiCliRunner {
    pub fn new(
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
        let mut full_reply = String::new();
        let mut iteration = 0u32;
        let mut hit_max_iterations = false;
        let mut total_raw_lines_seen = 0u32;
        let mut last_iter_buf = String::new();
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

            let iter_buf = match stream_gemini_prompt(
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
                Ok(buf) => buf,
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
                        terminal_error_emitted: true,
                        session_metadata_updates: std::collections::HashMap::new(),
                    };
                }
            };

            let (_visible_text, maybe_tool_call) = GeminiCliAgent::parse_tool_call(&iter_buf);

            if let Some((tool_name, tool_args, tool_reasoning)) = maybe_tool_call {
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "tool.execute",
                        detail: Some(tool_name.clone()),
                    })
                    .await;

                let tool_status = get_tool_status_message(&tool_name, "start");
                let reasoning = resolve_tool_reasoning(&tool_name, tool_reasoning);
                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool: tool_name.clone(),
                        status: "start".to_string(),
                        message: if tool_status.is_empty() {
                            None
                        } else {
                            Some(tool_status)
                        },
                        reasoning,
                    })
                    .await;

                let tool_result_val = self
                    .tool_registry
                    .execute_tool(&tool_name, tool_args.clone())
                    .await
                    .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }));
                let tool_result_str = tool_result_val.to_string();

                let tool_status = get_tool_status_message(&tool_name, "done");
                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool: tool_name.clone(),
                        status: "done".to_string(),
                        message: if tool_status.is_empty() {
                            None
                        } else {
                            Some(tool_status)
                        },
                        reasoning: None,
                    })
                    .await;

                tool_calls_made.push(ToolCallMade {
                    name: tool_name.clone(),
                    arguments: tool_args,
                    result: tool_result_val,
                    tool_call_id: None,
                });

                pending_tool_results.push((String::new(), tool_name, tool_result_str));
                last_iter_buf = iter_buf;
                continue;
            }

            last_iter_buf = iter_buf;
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
                    terminal_error_emitted: true,
                    session_metadata_updates: std::collections::HashMap::new(),
                };
            }
        }

        if full_reply.trim().is_empty() {
            tracing::warn!(
                "[AgentRunner/gemini] empty stream response (raw_lines_seen={}, last_buf_preview={})",
                total_raw_lines_seen,
                last_iter_buf.chars().take(200).collect::<String>()
            );
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
            terminal_error_emitted: false,
            session_metadata_updates: std::collections::HashMap::new(),
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
    let mut iter_buf = String::new();
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
                    let preview: String = line.chars().take(200).collect();
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
                        iter_buf.push_str(&chunk);
                        let visible_prefix = match iter_buf.find("<tool_call") {
                            Some(idx) => &iter_buf[..idx],
                            None => iter_buf.as_str(),
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
            tracing::warn!("[AgentRunner/gemini] stderr: {}", stderr_trimmed);
        }
        if !out.status.success() && iter_buf.is_empty() {
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::ExitFailure,
                message: format!(
                    "gemini exited with error (code={:?}): {}",
                    out.status.code(),
                    stderr_trimmed
                ),
            });
        }
    }

    Ok(iter_buf)
}

fn gemini_command() -> tokio::process::Command {
    let bin = std::env::var("HONE_GEMINI_BIN").unwrap_or_else(|_| "gemini".to_string());
    tokio::process::Command::new(bin)
}
