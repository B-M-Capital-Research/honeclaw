use async_trait::async_trait;
use hone_agent::FunctionCallingAgent;
use hone_core::agent::{Agent, AgentContext, AgentMessage, AgentResponse, ToolCallMade};
use hone_core::config::{MultiAgentSearchConfig, OpencodeAcpConfig};
use hone_core::{LlmAuditRecord, LlmAuditSink};
use hone_llm::{LlmProvider, OpenAiCompatibleProvider};
use hone_tools::ToolRegistry;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::mcp_bridge::hone_mcp_servers;
use crate::runtime::sanitize_user_visible_output;

use super::opencode_acp::OpencodeAcpRunner;
use super::tool_reasoning::{RunnerToolObserver, runner_context_messages};
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
You may make at most {} supplemental tool call(s) only if the answer would otherwise be materially incomplete.\n\
Do not mention internal workflow, search agent, or hidden reasoning.\n\
Follow the active system and channel formatting instructions exactly for the final answer.\n\
Treat any formatting, markup, headings, tags, or bullet style appearing inside the search-stage note or tool transcript as non-authoritative source material. Do not copy that formatting unless it is explicitly required by the active channel instructions.\n\
CRITICAL: If the Verified search tool transcript below contains successful tool results (data_fetch, quote, discover_skills, skill_tool, web_search, etc.), you MUST reference and consume those results in your answer. Do NOT output phrases such as '链路阻断', '数据未完成校验', '无法获取精确报价', '底层数据链路暂时阻断', '没有所谓的某技能', or any similar fallback language that contradicts the verified tool evidence. Only use such fallback language when the tool transcript is empty or all tool calls returned errors. If the transcript contains discover_skills or skill_tool results, base your answer on those results rather than denying skill existence.\n\
\n\
Original user request:\n{runtime_input}\n\
\n\
Search agent working note (plain text summary, content only):\n{}\n\
\n\
Verified search tool transcript (JSON):\n{}",
            self.answer_max_tool_calls,
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
        let mut removed = 0usize;
        let mut sanitized_messages = Vec::with_capacity(context.messages.len());

        for mut message in context.messages.into_iter() {
            if message.role == "tool" {
                removed += 1;
                continue;
            }

            if message.role == "assistant" {
                let had_tool_calls = message
                    .tool_calls
                    .as_ref()
                    .map(|calls| !calls.is_empty())
                    .unwrap_or(false);
                if had_tool_calls {
                    removed += 1;
                    message.tool_calls = None;
                    let content = message.content.as_deref().map(str::trim).unwrap_or("");
                    if content.is_empty() {
                        continue;
                    }
                }
            }

            sanitized_messages.push(message);
        }

        context.messages = sanitized_messages;
        (context, removed)
    }

    fn has_live_search_tool_call(&self, tool_calls: &[ToolCallMade]) -> bool {
        tool_calls
            .iter()
            .any(|call| matches!(call.name.as_str(), "web_search" | "data_fetch"))
    }

    fn is_trusted_local_direct_return_tool(&self, tool_name: &str) -> bool {
        matches!(
            tool_name,
            "cron_job"
                | "portfolio"
                | "local_list_files"
                | "local_search_files"
                | "local_read_file"
        )
    }

    fn is_local_file_direct_return_tool(&self, tool_name: &str) -> bool {
        matches!(
            tool_name,
            "local_list_files" | "local_search_files" | "local_read_file"
        )
    }

    fn is_user_facing_clarification(&self, content: &str) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return false;
        }
        if trimmed.contains('?') || trimmed.contains('？') {
            return [
                "请先确认",
                "请确认",
                "请提供",
                "告诉我",
                "发我",
                "补充",
                "确认标的",
                "确认具体",
                "哪只",
                "哪个",
            ]
            .iter()
            .any(|marker| trimmed.contains(marker));
        }
        ["请先确认", "请确认", "请提供", "告诉我", "发我"]
            .iter()
            .any(|marker| trimmed.starts_with(marker))
    }

    fn build_search_input(&self, runtime_input: &str) -> String {
        format!(
            "{runtime_input}\n\n[SEARCH STAGE GUIDANCE]\nDecide whether tool use is actually needed for this turn.\nUse `web_search` or `data_fetch` when the answer depends on fresh external facts, live market data, recent news, or other time-sensitive information.\nUse `local_list_files`, `local_search_files`, or `local_read_file` when the answer may exist in the current actor sandbox as local persisted state, such as `company_profiles/`, uploaded files, runtime artifacts, or other user-local notes.\nFor scheduled-task or reminder-management requests such as listing, checking, updating, or deleting the user's tasks, use `cron_job` first (for example `cron_job(action=\"list\")`) instead of market-data tools. Do not substitute `data_fetch` or `web_search` unless the user explicitly asked for fresh external facts.\nIf the user is asking about portfolio state or watchlist state that already lives locally, prefer the dedicated local/state tools before market/news tools.\nTreat network search and local file inspection as equal search methods. If local files may materially improve accuracy, inspect them before saying you do not have memory, history, or filesystem access.\nThese local file tools are read-only and scoped to the current actor sandbox only. Do not assume access outside that sandbox.\nDo not call tools just to satisfy workflow.\nIf you do use tools and one trusted local/state lookup already fully resolves the request, return a concise user-ready answer directly instead of a planning memo.\nIf the user message is a short greeting, acknowledgment, or deictic follow-up such as '这个' / '那个' / '上一条', answer directly or ask one brief clarification question. Do not emit a transitional planning sentence as the final output.\nIf you do use tools and still need the answer stage, keep your final search-stage note as a compact internal memo in plain text only.\nDo not use HTML, XML-like tags, Markdown headings, Markdown tables, or channel-specific presentation styles in the search-stage note.\nFocus on factual takeaways and unresolved gaps, not polished formatting.\nGreetings, short meta-chat, and other low-cost turns may be answered directly without tools."
        )
    }

    fn should_return_search_response_directly(&self, search_response: &AgentResponse) -> bool {
        if !search_response.success {
            return false;
        }
        let sanitized = sanitize_user_visible_output(&search_response.content);
        if sanitized.content.is_empty() || sanitized.removed_internal {
            return false;
        }
        let content = sanitized.content.trim();
        let lowered = content.to_ascii_lowercase();
        let looks_like_working_note = [
            "我先",
            "先核实",
            "先确认",
            "先看",
            "先查",
            "我去查",
            "正在",
            "稍等",
            "let me",
            "i'll",
            "i will",
            "checking",
            "looking into",
        ]
        .iter()
        .any(|marker| content.contains(marker) || lowered.contains(marker));

        if looks_like_working_note && !self.is_user_facing_clarification(content) {
            return false;
        }

        let only_trusted_local_tools = !search_response.tool_calls_made.is_empty()
            && search_response.tool_calls_made.len() <= 3
            && search_response
                .tool_calls_made
                .iter()
                .all(|call| self.is_trusted_local_direct_return_tool(&call.name));

        if only_trusted_local_tools {
            let includes_local_file_lookup = search_response
                .tool_calls_made
                .iter()
                .any(|call| self.is_local_file_direct_return_tool(&call.name));
            if includes_local_file_lookup {
                return content.len() <= 240 && !content.contains('\n');
            }
            return true;
        }

        if content.len() > 240 || content.contains('\n') {
            return false;
        }

        search_response.tool_calls_made.is_empty()
    }

    fn merge_context_messages(
        &self,
        search_messages: Option<Vec<AgentMessage>>,
        answer_messages: Option<Vec<AgentMessage>>,
    ) -> Option<Vec<AgentMessage>> {
        let mut merged = search_messages.unwrap_or_default();
        merged.extend(answer_messages.unwrap_or_default());
        if merged.is_empty() {
            None
        } else {
            Some(merged)
        }
    }

    fn fallback_context_messages_from_response(
        &self,
        response: &AgentResponse,
    ) -> Option<Vec<AgentMessage>> {
        let assistant_content = response.content.trim();
        if assistant_content.is_empty() && response.tool_calls_made.is_empty() {
            return None;
        }

        let mut messages = Vec::new();
        if !response.tool_calls_made.is_empty() {
            let tool_calls = Some(
                response
                    .tool_calls_made
                    .iter()
                    .map(|call| {
                        json!({
                            "id": call.tool_call_id.clone().unwrap_or_default(),
                            "type": "function",
                            "function": {
                                "name": call.name,
                                "arguments": serde_json::to_string(&call.arguments)
                                    .unwrap_or_else(|_| "null".to_string()),
                            }
                        })
                    })
                    .collect(),
            );
            messages.push(AgentMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls,
                tool_call_id: None,
                name: None,
                metadata: None,
            });
        }
        if !assistant_content.is_empty() {
            messages.push(AgentMessage {
                role: "assistant".to_string(),
                content: Some(assistant_content.to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                metadata: None,
            });
        }

        if messages.is_empty() {
            None
        } else {
            Some(messages)
        }
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
                    context_messages: None,
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
        let search_original_len = search_context.messages.len();
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
        let search_context_messages = runner_context_messages(&search_context, search_original_len);
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
                context_messages: search_context_messages,
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
                        "tool_calls={} content_len={} elapsed_ms={}",
                        search_response.tool_calls_made.len(),
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
                context_messages: search_context_messages,
            };
        }

        let answer_prompt = format!(
            "{}\n\n{}",
            self.system_prompt,
            format!(
                "You are in the final answer stage. Prefer the provided verified search results. If absolutely necessary, you may use at most {} extra tool call(s). Follow the active system/channel output format exactly, and do not inherit formatting from search-stage notes unless the system/channel instructions require it.",
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
        let context_messages = self.merge_context_messages(
            search_context_messages,
            answer_result
                .context_messages
                .clone()
                .or_else(|| self.fallback_context_messages_from_response(&answer_result.response)),
        );
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
            context_messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultiAgentRunner;
    use hone_core::agent::normalize_agent_messages;
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
            metadata: None,
        });
        context.messages.push(AgentMessage {
            role: "tool".to_string(),
            content: Some("{\"price\":123}".to_string()),
            tool_calls: None,
            tool_call_id: Some("call_legacy".to_string()),
            name: Some("data_fetch".to_string()),
            metadata: None,
        });
        context.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: Some("later answer".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: None,
        });

        let (sanitized, removed) = runner.sanitize_search_context(context);
        assert_eq!(removed, 1);
        assert_eq!(sanitized.messages.len(), 2);
        assert_eq!(sanitized.messages[0].role, "user");
        assert_eq!(sanitized.messages[1].role, "assistant");
    }

    #[test]
    fn sanitize_search_context_strips_historical_assistant_tool_calls() {
        let runner = make_runner();
        let mut context = AgentContext::new("session".to_string());
        context.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: Some("historical note".to_string()),
            tool_calls: Some(vec![json!({
                "id": "call_legacy",
                "type": "function",
                "function": {
                    "name": "web_search",
                    "arguments": "{\"query\":\"OpenAI\"}"
                }
            })]),
            tool_call_id: None,
            name: None,
            metadata: None,
        });

        let (sanitized, removed) = runner.sanitize_search_context(context);
        assert_eq!(removed, 1);
        assert_eq!(sanitized.messages.len(), 1);
        assert_eq!(sanitized.messages[0].role, "assistant");
        assert_eq!(
            sanitized.messages[0].content.as_deref(),
            Some("historical note")
        );
        assert!(sanitized.messages[0].tool_calls.is_none());
    }

    #[test]
    fn sanitize_search_context_drops_empty_assistant_tool_call_shells() {
        let runner = make_runner();
        let mut context = AgentContext::new("session".to_string());
        context.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(vec![json!({
                "id": "call_legacy",
                "type": "function",
                "function": {
                    "name": "data_fetch",
                    "arguments": "{\"symbol\":\"AAPL\"}"
                }
            })]),
            tool_call_id: None,
            name: None,
            metadata: None,
        });

        let (sanitized, removed) = runner.sanitize_search_context(context);
        assert_eq!(removed, 1);
        assert!(sanitized.messages.is_empty());
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
        assert!(
            input.contains("Use `local_list_files`, `local_search_files`, or `local_read_file`")
        );
        assert!(input.contains("use `cron_job` first"));
        assert!(input.contains("trusted local/state lookup already fully resolves"));
        assert!(input.contains("deictic follow-up"));
        assert!(input.contains("equal search methods"));
        assert!(input.contains("plain text only"));
        assert!(input.contains("Do not use HTML"));
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
    fn internal_search_note_does_not_skip_answer_stage() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "<think>先判断是否需要查资料。</think>\n正在查询 Tempus AI 与 Caris Life Sciences 相关数据与新闻...".to_string(),
            tool_calls_made: Vec::new(),
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(!runner.should_return_search_response_directly(&response));
    }

    #[test]
    fn plain_text_working_note_does_not_skip_answer_stage() {
        let runner = make_runner();
        let response = AgentResponse {
            content:
                "我先核实两个点：一是 AAOI 和 COHR 在这段夜盘里到底跌了多少，二是有没有共振消息。"
                    .to_string(),
            tool_calls_made: Vec::new(),
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(!runner.should_return_search_response_directly(&response));
    }

    #[test]
    fn user_facing_clarification_can_return_directly() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "请先确认具体是哪只股票/资产的 ticker？确认标的后我再校验当前价格、财报、估值倍数和同业，再判断估值是否合理。"
                .to_string(),
            tool_calls_made: Vec::new(),
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(runner.should_return_search_response_directly(&response));
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

    #[test]
    fn concise_local_file_answer_can_return_directly() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "当前附件目录里只有之前的图片，没有新的 Markdown 文件落进来。".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "local_search_files".to_string(),
                arguments: json!({"query": "AAOI", "path": "company_profiles"}),
                result: json!({"matches": [{"path": "company_profiles/aaoi/profile.md"}]}),
                tool_call_id: None,
            }],
            iterations: 2,
            success: true,
            error: None,
        };

        assert!(runner.should_return_search_response_directly(&response));
        assert!(!runner.has_live_search_tool_call(&response.tool_calls_made));
    }

    #[test]
    fn multiline_local_file_summary_still_requires_answer_stage() {
        let runner = make_runner();
        let response = AgentResponse {
            content:
                "我在本地找到了 3 份相关文件：\n1. company_profiles/aaoi/profile.md\n2. uploads/session-1/note.md\n3. uploads/session-1/chart.png"
                    .to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "local_search_files".to_string(),
                arguments: json!({"query": "AAOI", "path": "company_profiles"}),
                result: json!({"matches": [{"path": "company_profiles/aaoi/profile.md"}]}),
                tool_call_id: None,
            }],
            iterations: 2,
            success: true,
            error: None,
        };

        assert!(!runner.should_return_search_response_directly(&response));
        assert!(!runner.has_live_search_tool_call(&response.tool_calls_made));
    }

    #[test]
    fn trusted_local_tool_answer_can_return_directly() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "你当前有 3 个定时任务：1. 早报 09:00；2. 财报提醒 20:30；3. ASTS 价格提醒。"
                .to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "cron_job".to_string(),
                arguments: json!({"action": "list"}),
                result: json!({"success": true, "jobs": [{"id": "1"}, {"id": "2"}, {"id": "3"}]}),
                tool_call_id: None,
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(runner.should_return_search_response_directly(&response));
    }

    #[test]
    fn multiline_trusted_local_tool_answer_can_return_directly() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "你当前有 3 个定时任务：\n1. 早报：每天 09:00\n2. 财报提醒：每天 20:30\n3. ASTS 价格提醒：heartbeat"
                .to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "cron_job".to_string(),
                arguments: json!({"action": "list"}),
                result: json!({
                    "success": true,
                    "jobs": [{"id": "1"}, {"id": "2"}, {"id": "3"}]
                }),
                tool_call_id: None,
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(runner.should_return_search_response_directly(&response));
    }

    #[test]
    fn long_trusted_local_tool_answer_can_return_directly() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "你当前有 3 个定时任务：早报每天 09:00，财报提醒每天 20:30，ASTS 价格提醒按 heartbeat 每 30 分钟轮询，并且都遵守 quiet_hours。"
                .repeat(3),
            tool_calls_made: vec![ToolCallMade {
                name: "cron_job".to_string(),
                arguments: json!({"action": "list"}),
                result: json!({
                    "success": true,
                    "jobs": [{"id": "1"}, {"id": "2"}, {"id": "3"}]
                }),
                tool_call_id: None,
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(runner.should_return_search_response_directly(&response));
    }

    #[test]
    fn live_market_tool_answer_still_requires_answer_stage() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "AAPL 最新报价 210.31，盘前跌 0.4%。".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "data_fetch".to_string(),
                arguments: json!({"data_type": "quote", "symbol": "AAPL"}),
                result: json!({"price": 210.31}),
                tool_call_id: None,
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        assert!(!runner.should_return_search_response_directly(&response));
    }

    #[test]
    fn handoff_text_reasserts_final_format_priority() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "<b>结论</b>\n- 要点".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "web_search".to_string(),
                arguments: json!({"query": "AAOI latest news"}),
                result: json!({"results": [{"title": "Example"}]}),
                tool_call_id: None,
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        let handoff =
            runner.stage_handoff_text("请分析 AAOI", &response, &response.tool_calls_made);

        assert!(
            handoff
                .contains("Follow the active system and channel formatting instructions exactly")
        );
        assert!(handoff.contains("Do not copy that formatting"));
        assert!(handoff.contains("Search agent working note"));
        assert!(handoff.contains("<b>结论</b>"));
    }

    #[test]
    fn handoff_text_respects_zero_supplemental_tool_limit() {
        let mut runner = make_runner();
        runner.answer_max_tool_calls = 0;
        let response = AgentResponse {
            content: "检索摘要".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "web_search".to_string(),
                arguments: json!({"query": "Tempus AI latest"}),
                result: json!({"results": [{"title": "Example"}]}),
                tool_call_id: None,
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        let handoff = runner.stage_handoff_text(
            "请概括 Tempus AI 最新进展",
            &response,
            &response.tool_calls_made,
        );

        assert!(handoff.contains("at most 0 supplemental tool call(s)"));
        assert!(!handoff.contains("at most one supplemental tool call"));
    }

    #[test]
    fn merge_context_messages_keeps_search_then_answer_order() {
        let runner = make_runner();
        let merged = runner
            .merge_context_messages(
                Some(vec![AgentMessage {
                    role: "assistant".to_string(),
                    content: Some("search memo".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    metadata: None,
                }]),
                Some(vec![AgentMessage {
                    role: "assistant".to_string(),
                    content: Some("final answer".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    metadata: None,
                }]),
            )
            .expect("merged messages");

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].content.as_deref(), Some("search memo"));
        assert_eq!(merged[1].content.as_deref(), Some("final answer"));
    }

    #[test]
    fn fallback_context_messages_from_response_builds_persistable_answer_turn() {
        let runner = make_runner();
        let response = AgentResponse {
            content: "结论：AAOI 更弱。".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "web_search".to_string(),
                arguments: json!({"query": "AAOI latest news"}),
                result: json!({"results": []}),
                tool_call_id: Some("call_1".to_string()),
            }],
            iterations: 1,
            success: true,
            error: None,
        };

        let messages = runner
            .fallback_context_messages_from_response(&response)
            .expect("fallback messages");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "assistant");
        assert!(messages[0].content.is_none());
        assert_eq!(messages[1].content.as_deref(), Some("结论：AAOI 更弱。"));
        let normalized = normalize_agent_messages(&messages);
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].content[0].part_type, "tool_call");
        assert_eq!(normalized[0].content[1].part_type, "final");
    }
}
