use async_trait::async_trait;
use hone_agent::FunctionCallingAgent;
use hone_core::agent::{Agent, AgentContext, AgentResponse, ToolCallMade};
use hone_core::config::{MultiAgentSearchConfig, OpencodeAcpConfig};
use hone_core::{LlmAuditRecord, LlmAuditSink};
use hone_llm::{LlmProvider, OpenAiCompatibleProvider};
use hone_tools::ToolRegistry;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::mcp_bridge::hone_mcp_servers;

use super::opencode_acp::OpencodeAcpRunner;
use super::tool_reasoning::RunnerToolObserver;
use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
};

pub struct MultiAgentRunner {
    system_prompt: String,
    search_config: MultiAgentSearchConfig,
    answer_config: OpencodeAcpConfig,
    answer_max_tool_calls: u32,
    tools: Arc<ToolRegistry>,
    llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl MultiAgentRunner {
    const REQUIRED_SEARCH_TOOLS: [&'static str; 2] = ["web_search", "data_fetch"];

    pub fn new(
        system_prompt: String,
        search_config: MultiAgentSearchConfig,
        answer_config: OpencodeAcpConfig,
        answer_max_tool_calls: u32,
        tools: Arc<ToolRegistry>,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        Self {
            system_prompt,
            search_config,
            answer_config,
            answer_max_tool_calls,
            tools,
            llm_audit,
        }
    }

    fn build_search_provider(&self) -> Result<Arc<dyn LlmProvider>, String> {
        let api_key = self.search_config.api_key.trim();
        if api_key.is_empty() {
            return Err("multi-agent search agent API key 为空".to_string());
        }
        let provider = OpenAiCompatibleProvider::new(
            api_key,
            &self.search_config.base_url,
            &self.search_config.model,
            120,
            4096,
        )
        .map_err(|err| err.to_string())?;
        Ok(Arc::new(provider))
    }

    fn stage_handoff_text(
        &self,
        runtime_input: &str,
        search_response: &AgentResponse,
        tool_calls_made: &[ToolCallMade],
    ) -> String {
        let tool_results: Vec<Value> = tool_calls_made
            .iter()
            .map(|call| {
                json!({
                    "tool": call.name,
                    "arguments": call.arguments,
                    "result": call.result,
                })
            })
            .collect();

        format!(
            "You are the answer agent in a two-stage workflow.\n\
Use the verified search results below to answer the original user request.\n\
You may make at most one supplemental tool call only if the answer would otherwise be materially incomplete.\n\
Do not mention internal workflow, search agent, or hidden reasoning.\n\
\n\
Original user request:\n{runtime_input}\n\
\n\
Search agent final note:\n{}\n\
\n\
Verified search tool transcript (JSON):\n{}",
            search_response.content.trim(),
            serde_json::to_string_pretty(&tool_results).unwrap_or_else(|_| "[]".to_string())
        )
    }

    fn record_stage_audit(
        &self,
        request: &AgentRunnerRequest,
        source: &str,
        provider: &str,
        model: Option<String>,
        started: Instant,
        success: bool,
        request_payload: Value,
        response_payload: Option<Value>,
        error: Option<String>,
        metadata: Value,
    ) {
        let Some(sink) = &self.llm_audit else {
            return;
        };
        let mut record = LlmAuditRecord::new(
            request.session_id.clone(),
            Some(request.actor.clone()),
            source.to_string(),
            "run".to_string(),
            provider.to_string(),
            model,
            request_payload,
        );
        record.success = success;
        record.latency_ms = Some(started.elapsed().as_millis());
        record.response = response_payload;
        record.error = error;
        record.metadata = metadata;
        if let Err(err) = sink.record(record) {
            tracing::warn!(
                "[LlmAudit] failed to persist multi-agent stage audit: {}",
                err
            );
        }
    }

    fn sanitize_search_context(&self, mut context: AgentContext) -> (AgentContext, usize) {
        let original_len = context.messages.len();
        context.messages.retain(|message| message.role != "tool");
        let removed = original_len.saturating_sub(context.messages.len());
        (context, removed)
    }

    fn has_required_search_tool_call(&self, tool_calls: &[ToolCallMade]) -> bool {
        tool_calls.iter().any(|call| {
            Self::REQUIRED_SEARCH_TOOLS
                .iter()
                .any(|required| call.name == *required)
        })
    }

    fn build_forced_search_input(&self, runtime_input: &str) -> String {
        format!(
            "{runtime_input}\n\n[SEARCH STAGE REQUIREMENT]\nBefore you finish this search stage, you MUST call at least one of these tools: `web_search` or `data_fetch`.\nUse the tool result to ground your answer. Do not answer from memory alone."
        )
    }
}

#[async_trait]
impl AgentRunner for MultiAgentRunner {
    fn name(&self) -> &'static str {
        "multi-agent"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        tracing::info!(
            "[MultiAgent] session={} actor={} search.provider=openai-compatible search.base_url={} search.model={} answer.runner=opencode_acp answer.base_url={} answer.model={} answer.variant={} answer.max_tool_calls={}",
            request.session_id,
            request.actor.session_id(),
            self.search_config.base_url,
            self.search_config.model,
            self.answer_config.api_base_url,
            self.answer_config.model,
            self.answer_config.variant,
            self.answer_max_tool_calls,
        );
        let search_started = Instant::now();
        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "multi_agent.search.start",
                detail: Some(format!(
                    "provider=openai-compatible base_url={} model={} max_iterations={}",
                    self.search_config.base_url,
                    self.search_config.model,
                    self.search_config.max_iterations
                )),
            })
            .await;

        let search_provider = match self.build_search_provider() {
            Ok(provider) => provider,
            Err(message) => {
                let error = hone_core::agent::AgentResponse {
                    content: String::new(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: false,
                    error: Some(message.clone()),
                };
                return AgentRunnerResult {
                    response: error,
                    streamed_output: false,
                    terminal_error_emitted: false,
                    session_metadata_updates: HashMap::new(),
                };
            }
        };

        let observer = Arc::new(RunnerToolObserver {
            emitter: emitter.clone(),
        });
        let search_agent = FunctionCallingAgent::new(
            search_provider,
            self.tools.clone(),
            self.system_prompt.clone(),
            self.search_config.max_iterations,
            self.llm_audit.clone(),
        )
        .with_tool_observer(Some(observer));

        let (mut search_context, removed_tool_messages) =
            self.sanitize_search_context(request.context.clone());
        if removed_tool_messages > 0 {
            tracing::info!(
                "[MultiAgent] session={} stage=search.context_sanitized removed_tool_messages={}",
                request.session_id,
                removed_tool_messages,
            );
            emitter
                .emit(AgentRunnerEvent::Progress {
                    stage: "multi_agent.search.context_sanitized",
                    detail: Some(format!(
                        "removed_tool_messages={}",
                        removed_tool_messages
                    )),
                })
                .await;
        }
        let mut search_runtime_input = request.runtime_input.clone();
        let mut search_response = search_agent
            .run(&search_runtime_input, &mut search_context)
            .await;
        let mut forced_retry = false;
        if search_response.success
            && !self.has_required_search_tool_call(&search_response.tool_calls_made)
        {
            forced_retry = true;
            tracing::warn!(
                "[MultiAgent] session={} stage=search.missing_required_tool tools_seen={} required_tools={:?}",
                request.session_id,
                search_response
                    .tool_calls_made
                    .iter()
                    .map(|call| call.name.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                Self::REQUIRED_SEARCH_TOOLS,
            );
            emitter
                .emit(AgentRunnerEvent::Progress {
                    stage: "multi_agent.search.retry_required_tool",
                    detail: Some("missing required search tool; retrying with forced web_search/data_fetch requirement".to_string()),
                })
                .await;
            let (mut retry_context, _) = self.sanitize_search_context(request.context.clone());
            search_runtime_input = self.build_forced_search_input(&request.runtime_input);
            search_response = search_agent
                .run(&search_runtime_input, &mut retry_context)
                .await;
        }
        let search_elapsed_ms = search_started.elapsed().as_millis();
        tracing::info!(
            "[MultiAgent] session={} stage=search.done success={} iterations={} tool_calls={} elapsed_ms={}",
            request.session_id,
            search_response.success,
            search_response.iterations,
            search_response.tool_calls_made.len(),
            search_elapsed_ms,
        );
        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "multi_agent.search.done",
                detail: Some(format!(
                    "success={} iterations={} tool_calls={} elapsed_ms={}",
                    search_response.success,
                    search_response.iterations,
                    search_response.tool_calls_made.len(),
                    search_elapsed_ms
                )),
            })
            .await;
        self.record_stage_audit(
            &request,
            "agent.multi_agent.search",
            "openai-compatible",
            Some(self.search_config.model.clone()),
            search_started,
            search_response.success,
            json!({
                "model": self.search_config.model.as_str(),
                "base_url": self.search_config.base_url.as_str(),
                "runtime_input": search_runtime_input.as_str(),
            }),
            Some(json!({
                "content": search_response.content.as_str(),
                "tool_calls_made": search_response.tool_calls_made,
                "iterations": search_response.iterations,
            })),
            search_response.error.clone(),
            json!({
                "kind": "multi_agent_search",
                "removed_tool_messages": removed_tool_messages,
                "forced_retry": forced_retry,
            }),
        );

        if !search_response.success {
            return AgentRunnerResult {
                response: search_response,
                streamed_output: false,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
            };
        }

        if !self.has_required_search_tool_call(&search_response.tool_calls_made) {
            let error_message =
                "multi-agent search stage must call web_search or data_fetch before answering"
                    .to_string();
            tracing::error!(
                "[MultiAgent] session={} stage=search.required_tool_missing_after_retry error=\"{}\"",
                request.session_id,
                error_message,
            );
            emitter
                .emit(AgentRunnerEvent::Error {
                    error: crate::agent_session::AgentSessionError {
                        kind: crate::agent_session::AgentSessionErrorKind::AgentFailed,
                        message: error_message.clone(),
                    },
                })
                .await;
            return AgentRunnerResult {
                response: AgentResponse {
                    content: String::new(),
                    tool_calls_made: search_response.tool_calls_made,
                    iterations: search_response.iterations,
                    success: false,
                    error: Some(error_message),
                },
                streamed_output: false,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
            };
        }

        let answer_prompt = format!(
            "{}\n\n{}",
            self.system_prompt,
            format!(
                "You are in the final answer stage. Prefer the provided verified search results. If absolutely necessary, you may use at most {} extra tool call(s).",
                self.answer_max_tool_calls
            )
        );
        let answer_runtime_input = self.stage_handoff_text(
            &request.runtime_input,
            &search_response,
            &search_response.tool_calls_made,
        );
        let mut answer_request = request.clone();
        answer_request.system_prompt = answer_prompt;
        answer_request.runtime_input = answer_runtime_input.clone();
        answer_request.max_tool_calls = Some(self.answer_max_tool_calls);

        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "multi_agent.answer.start",
                detail: Some(format!(
                    "runner=opencode_acp base_url={} model={} variant={} max_tool_calls={}",
                    self.answer_config.api_base_url,
                    self.answer_config.model,
                    self.answer_config.variant,
                    self.answer_max_tool_calls
                )),
            })
            .await;
        let answer_started = Instant::now();
        let answer_runner = OpencodeAcpRunner::new(self.answer_config.clone());
        let answer_result = answer_runner.run(answer_request, emitter.clone()).await;
        let answer_elapsed_ms = answer_started.elapsed().as_millis();
        tracing::info!(
            "[MultiAgent] session={} stage=answer.done success={} iterations={} tool_calls={} elapsed_ms={} streamed_output={} terminal_error_emitted={}",
            request.session_id,
            answer_result.response.success,
            answer_result.response.iterations,
            answer_result.response.tool_calls_made.len(),
            answer_elapsed_ms,
            answer_result.streamed_output,
            answer_result.terminal_error_emitted,
        );
        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "multi_agent.answer.done",
                detail: Some(format!(
                    "success={} iterations={} tool_calls={} elapsed_ms={}",
                    answer_result.response.success,
                    answer_result.response.iterations,
                    answer_result.response.tool_calls_made.len(),
                    answer_elapsed_ms
                )),
            })
            .await;
        self.record_stage_audit(
            &request,
            "agent.multi_agent.answer",
            "opencode_acp",
            Some(self.answer_config.model.clone()),
            answer_started,
            answer_result.response.success,
            json!({
                "model": self.answer_config.model.as_str(),
                "api_base_url": self.answer_config.api_base_url.as_str(),
                "runtime_input": answer_runtime_input,
                "max_tool_calls": self.answer_max_tool_calls,
                "mcp_servers": hone_mcp_servers(&request).ok(),
            }),
            Some(json!({
                "content": answer_result.response.content.as_str(),
                "tool_calls_made": answer_result.response.tool_calls_made,
                "iterations": answer_result.response.iterations,
            })),
            answer_result.response.error.clone(),
            json!({
                "kind": "multi_agent_answer"
            }),
        );

        let mut combined_tool_calls = search_response.tool_calls_made.clone();
        combined_tool_calls.extend(answer_result.response.tool_calls_made.clone());
        tracing::info!(
            "[MultiAgent] session={} stage=complete success={} search_tool_calls={} answer_tool_calls={} combined_tool_calls={}",
            request.session_id,
            answer_result.response.success,
            search_response.tool_calls_made.len(),
            answer_result.response.tool_calls_made.len(),
            combined_tool_calls.len(),
        );

        AgentRunnerResult {
            response: AgentResponse {
                content: answer_result.response.content,
                tool_calls_made: combined_tool_calls,
                iterations: search_response.iterations + answer_result.response.iterations,
                success: answer_result.response.success,
                error: answer_result.response.error,
            },
            streamed_output: answer_result.streamed_output,
            terminal_error_emitted: answer_result.terminal_error_emitted,
            session_metadata_updates: answer_result.session_metadata_updates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiAgentRunner;
    use hone_core::agent::{AgentContext, AgentMessage, ToolCallMade};
    use hone_core::config::{MultiAgentSearchConfig, OpencodeAcpConfig};
    use hone_tools::ToolRegistry;
    use serde_json::json;
    use std::sync::Arc;

    fn make_runner() -> MultiAgentRunner {
        MultiAgentRunner::new(
            "system".to_string(),
            MultiAgentSearchConfig {
                base_url: "https://api.minimaxi.com/v1".to_string(),
                api_key: "test-key".to_string(),
                model: "MiniMax-M2.7-highspeed".to_string(),
                max_iterations: 8,
            },
            OpencodeAcpConfig::default(),
            1,
            Arc::new(ToolRegistry::new()),
            None,
        )
    }

    #[test]
    fn sanitize_search_context_drops_historical_tool_messages() {
        let runner = make_runner();
        let mut context = AgentContext::new("session".to_string());
        context.messages.push(AgentMessage {
            role: "user".to_string(),
            content: Some("hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        context.messages.push(AgentMessage {
            role: "tool".to_string(),
            content: Some("{\"price\":123}".to_string()),
            tool_calls: None,
            tool_call_id: Some("call_legacy".to_string()),
            name: Some("data_fetch".to_string()),
        });
        context.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: Some("later answer".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        let (sanitized, removed) = runner.sanitize_search_context(context);
        assert_eq!(removed, 1);
        assert_eq!(sanitized.messages.len(), 2);
        assert_eq!(sanitized.messages[0].role, "user");
        assert_eq!(sanitized.messages[1].role, "assistant");
    }

    #[test]
    fn required_search_tool_detection_only_accepts_web_search_or_data_fetch() {
        let runner = make_runner();
        assert!(!runner.has_required_search_tool_call(&[ToolCallMade {
            name: "kb_search".to_string(),
            arguments: json!({}),
            result: json!({}),
            tool_call_id: None,
        }]));
        assert!(runner.has_required_search_tool_call(&[ToolCallMade {
            name: "web_search".to_string(),
            arguments: json!({}),
            result: json!({}),
            tool_call_id: None,
        }]));
        assert!(runner.has_required_search_tool_call(&[ToolCallMade {
            name: "data_fetch".to_string(),
            arguments: json!({}),
            result: json!({}),
            tool_call_id: None,
        }]));
    }
}
