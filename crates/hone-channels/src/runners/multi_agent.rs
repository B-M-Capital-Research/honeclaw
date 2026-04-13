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
    RunnerTimeouts,
};

pub struct MultiAgentRunner {
    system_prompt: String,
    search_config: MultiAgentSearchConfig,
    answer_config: OpencodeAcpConfig,
    timeouts: RunnerTimeouts,
    answer_max_tool_calls: u32,
    tools: Arc<ToolRegistry>,
    llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl MultiAgentRunner {
    pub fn new(
        system_prompt: String,
        search_config: MultiAgentSearchConfig,
        answer_config: OpencodeAcpConfig,
        timeouts: RunnerTimeouts,
        answer_max_tool_calls: u32,
        tools: Arc<ToolRegistry>,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        Self {
            system_prompt,
            search_config,
            answer_config,
            timeouts,
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
            self.timeouts.step.as_secs(),
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

    fn has_live_search_tool_call(&self, tool_calls: &[ToolCallMade]) -> bool {
        tool_calls
            .iter()
            .any(|call| matches!(call.name.as_str(), "web_search" | "data_fetch"))
    }

    fn build_search_input(&self, runtime_input: &str) -> String {
        format!(
            "{runtime_input}\n\n[SEARCH STAGE GUIDANCE]\nDecide whether tool use is actually needed for this turn.\nUse `web_search` or `data_fetch` when the user asks for fresh external facts, live market data, recent news, or other time-sensitive information.\nDo not call tools just to satisfy workflow.\nGreetings, short meta-chat, and other low-cost turns may be answered directly without tools."
        )
    }

    fn should_return_search_response_directly(&self, search_response: &AgentResponse) -> bool {
        search_response.success && search_response.tool_calls_made.is_empty()
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
                    detail: Some(format!("removed_tool_messages={}", removed_tool_messages)),
                })
                .await;
        }
        let search_runtime_input = self.build_search_input(&request.runtime_input);
        let search_response = search_agent
            .run(&search_runtime_input, &mut search_context)
            .await;
        let search_elapsed_ms = search_started.elapsed().as_millis();
        let search_tool_calls = search_response.tool_calls_made.len();
        let used_live_search_tool =
            self.has_live_search_tool_call(&search_response.tool_calls_made);
        tracing::info!(
            "[MultiAgent] session={} stage=search.done success={} iterations={} tool_calls={} live_search_tool={} elapsed_ms={}",
            request.session_id,
            search_response.success,
            search_response.iterations,
            search_tool_calls,
            used_live_search_tool,
            search_elapsed_ms,
        );
        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "multi_agent.search.done",
                detail: Some(format!(
                    "success={} iterations={} tool_calls={} live_search_tool={} elapsed_ms={}",
                    search_response.success,
                    search_response.iterations,
                    search_tool_calls,
                    used_live_search_tool,
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
                "tool_calls": search_tool_calls,
                "used_live_search_tool": used_live_search_tool,
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

        if self.should_return_search_response_directly(&search_response) {
            tracing::info!(
                "[MultiAgent] session={} stage=search.direct_return content_len={} elapsed_ms={}",
                request.session_id,
                search_response.content.len(),
                search_elapsed_ms,
            );
            emitter
                .emit(AgentRunnerEvent::Progress {
                    stage: "multi_agent.search.direct_return",
                    detail: Some(format!(
                        "tool_calls=0 content_len={} elapsed_ms={}",
                        search_response.content.len(),
                        search_elapsed_ms
                    )),
                })
                .await;
            return AgentRunnerResult {
                response: search_response,
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
        let answer_runner = OpencodeAcpRunner::new(self.answer_config.clone(), self.timeouts);
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
    use hone_core::agent::{AgentContext, AgentMessage, AgentResponse, ToolCallMade};
    use hone_core::config::{MultiAgentSearchConfig, OpencodeAcpConfig};
    use hone_tools::ToolRegistry;
    use serde_json::json;
    use std::sync::Arc;

    use crate::runners::RunnerTimeouts;

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
            RunnerTimeouts {
                step: std::time::Duration::from_secs(180),
                overall: std::time::Duration::from_secs(1200),
            },
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
    fn live_search_tool_detection_is_telemetry_only_for_web_search_and_data_fetch() {
        let runner = make_runner();
        assert!(!runner.has_live_search_tool_call(&[ToolCallMade {
            name: "kb_search".to_string(),
            arguments: json!({}),
            result: json!({}),
            tool_call_id: None,
        }]));
        assert!(runner.has_live_search_tool_call(&[ToolCallMade {
            name: "web_search".to_string(),
            arguments: json!({}),
            result: json!({}),
            tool_call_id: None,
        }]));
        assert!(runner.has_live_search_tool_call(&[ToolCallMade {
            name: "data_fetch".to_string(),
            arguments: json!({}),
            result: json!({}),
            tool_call_id: None,
        }]));
    }

    #[test]
    fn search_input_guidance_allows_direct_replies_for_greetings() {
        let runner = make_runner();
        let input = runner.build_search_input("hi");
        assert!(input.contains("Greetings, short meta-chat"));
        assert!(input.contains("may be answered directly without tools"));
        assert!(input.contains("Use `web_search` or `data_fetch`"));
    }

    #[test]
    fn zero_tool_successful_search_response_can_return_directly() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "你好".to_string(),
            tool_calls_made: Vec::new(),
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(runner.should_return_search_response_directly(&response));
        assert!(!runner.has_live_search_tool_call(&response.tool_calls_made));
    }

    #[test]
    fn tool_backed_search_response_does_not_skip_answer_stage() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "这是检索摘要".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "web_search".to_string(),
                arguments: json!({"query": "Rocket Lab stock"}),
                result: json!({"results": []}),
                tool_call_id: None,
            }],
            iterations: 2,
            success: true,
            error: None,
        };

        assert!(!runner.should_return_search_response_directly(&response));
        assert!(runner.has_live_search_tool_call(&response.tool_calls_made));
    }
}
