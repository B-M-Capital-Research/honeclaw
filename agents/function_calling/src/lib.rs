//! Hone Agent — Function Calling Agent 核心
//!
//! 基于 `OpenAI` Function Calling 模式的 legacy Agent 适配器。
//! 这里负责多轮工具调用循环，并把最终结果聚合成 `AgentResponse`；
//! 渠道级流式输出由 `hone-channels` 的 runner 层处理。

use async_trait::async_trait;
use futures::StreamExt;
use hone_core::agent::{Agent, AgentContext, AgentResponse, ToolCallMade};
use hone_core::{LlmAuditRecord, LlmAuditSink, ToolExecutionObserver};
use hone_llm::{
    ChatResponse, ChatStreamEvent, FunctionCall, LlmProvider, Message, ToolCall, ToolChoiceMode,
};
use hone_tools::ToolRegistry;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

const REASONING_CONTENT_METADATA_KEY: &str = "reasoning_content";
const FINISH_RESEARCH_TOOL_NAME: &str = "finish_research";
const FINISH_RESEARCH_SYSTEM_INSTRUCTION: &str = "【内部终态流控制】当前轮已实际进入金融数据工具链。如果当前请求所需的合理工具尝试已经完成（包括已明确确认某项数据暂不可得），请不要在仍可调用工具的轮次直接展开长篇最终回答；必须单独调用 `finish_research`，由同一 Agent 和同一上下文进入无工具终稿阶段。不得把它与任何其它工具一起调用。缺失证据应在终稿中如实披露，不构成拒答。它只是流式终态信号，不是审查、改写或阻止正常回答的授权。";
const TERMINAL_SYNTHESIS_PROMPT: &str = "【终局回答阶段】\n\
你已经明确表示本轮合理的研究与工具尝试已经完成。当前阶段不再提供任何工具；请只基于同一轮对话中已有的用户请求和工具结果，直接生成一次完整、可见的最终回答。逐项复核所有公司关系、新闻因果、日期、行情、财务与估值数字：工具结果没有明确支持的事实必须标为未核验，不得用模型记忆补齐；年度数据不得写成 TTM，未取得净债务或企业价值时不得把市值/EBITDA 写成 EV/EBITDA；报价时间优先原样采用工具返回的 Hone 规范化北京时间字段，普通 quote 不得自行推断盘前/盘后时段。某项证据不可得时，披露缺项并继续完成能够被当前证据支持的分析，不得因此拒绝整个问题。不要提及 finish_research、内部协议、工具循环或这条提示。";

#[async_trait]
pub trait FunctionCallingStreamObserver: Send + Sync {
    async fn on_content_delta(&self, content: &str);

    /// A delta from a tool-free terminal synthesis round. The default keeps
    /// existing observers source-compatible; channel adapters may override it
    /// when they need to distinguish draft-capable tool rounds from a final
    /// stream that can no longer be followed by another tool call.
    async fn on_final_content_delta(&self, content: &str) {
        self.on_content_delta(content).await;
    }

    async fn on_content_reset(&self);
}

#[derive(Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

/// Function Calling Agent
pub struct FunctionCallingAgent {
    pub llm: Arc<dyn LlmProvider>,
    pub tools: Arc<ToolRegistry>,
    pub system_prompt: String,
    pub max_iterations: u32,
    pub debug_log: bool,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub tool_observer: Option<Arc<dyn ToolExecutionObserver>>,
    pub stream_observer: Option<Arc<dyn FunctionCallingStreamObserver>>,
    pub max_tool_calls: Option<u32>,
    pub tool_call_limits: HashMap<String, u32>,
    pub finish_research_terminal_synthesis: bool,
}

impl FunctionCallingAgent {
    pub fn new(
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        system_prompt: String,
        max_iterations: u32,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        let debug_log = std::env::var("HONE_AGENT_DEBUG")
            .map(|v| matches!(v.trim(), "1" | "true" | "True"))
            .unwrap_or(false);

        Self {
            llm,
            tools,
            system_prompt,
            max_iterations,
            debug_log,
            llm_audit,
            tool_observer: None,
            stream_observer: None,
            max_tool_calls: None,
            tool_call_limits: HashMap::new(),
            finish_research_terminal_synthesis: false,
        }
    }

    pub fn with_tool_observer(mut self, observer: Option<Arc<dyn ToolExecutionObserver>>) -> Self {
        self.tool_observer = observer;
        self
    }

    pub fn with_stream_observer(
        mut self,
        observer: Option<Arc<dyn FunctionCallingStreamObserver>>,
    ) -> Self {
        self.stream_observer = observer;
        self
    }

    pub fn with_tool_call_budget(
        mut self,
        max_tool_calls: Option<u32>,
        tool_call_limits: HashMap<String, u32>,
    ) -> Self {
        self.max_tool_calls = max_tool_calls;
        self.tool_call_limits = tool_call_limits;
        self
    }

    /// Enable the internal `finish_research` control tool. When the model emits
    /// that tool as the sole call in a round, this Agent performs one final
    /// tool-free streamed completion using the same in-memory context. Direct
    /// answers remain valid and do not trigger an additional completion.
    pub fn with_finish_research_terminal_synthesis(mut self, enabled: bool) -> Self {
        self.finish_research_terminal_synthesis = enabled;
        self
    }

    fn dbg(&self, msg: &str) {
        if self.debug_log {
            tracing::debug!("{msg}");
        }
    }

    /// 构建完整消息列表（system prompt + context messages）
    fn build_messages(
        &self,
        context: &AgentContext,
        finish_research_available: bool,
    ) -> Vec<Message> {
        let mut messages = Vec::with_capacity(context.messages.len() + 1);

        if !self.system_prompt.is_empty() {
            let system_prompt = if finish_research_available {
                format!(
                    "{}\n\n{}",
                    self.system_prompt, FINISH_RESEARCH_SYSTEM_INSTRUCTION
                )
            } else {
                self.system_prompt.clone()
            };
            messages.push(Message {
                role: "system".to_string(),
                content: Some(system_prompt),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        for msg in &context.messages {
            messages.push(Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
                reasoning_content: msg
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get(REASONING_CONTENT_METADATA_KEY))
                    .and_then(|value| value.as_str())
                    .map(ToString::to_string),
                tool_calls: msg.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .filter_map(|tc| serde_json::from_value(tc.clone()).ok())
                        .collect()
                }),
                tool_call_id: msg.tool_call_id.clone(),
                name: msg.name.clone(),
            });
        }

        messages
    }

    fn record_audit(
        &self,
        context: &AgentContext,
        operation: &str,
        request: Value,
        response: Option<Value>,
        error: Option<String>,
        latency_ms: u128,
        metadata: Value,
        usage: Option<hone_llm::provider::TokenUsage>,
    ) {
        let Some(sink) = &self.llm_audit else {
            return;
        };
        let mut record = LlmAuditRecord::new(
            context.session_id.clone(),
            context.actor_identity(),
            "agent.function_calling",
            operation.to_string(),
            "openrouter",
            None,
            request,
        );
        record.success = error.is_none();
        record.response = response;
        record.error = error;
        record.latency_ms = Some(latency_ms);
        record.metadata = metadata;
        if let Some(u) = usage {
            record.prompt_tokens = u.prompt_tokens;
            record.completion_tokens = u.completion_tokens;
            record.total_tokens = u.total_tokens;
        }
        if let Err(err) = sink.record(record) {
            tracing::warn!(
                "[LlmAudit] failed to persist function_calling audit: {}",
                err
            );
        }
    }

    async fn chat_with_tools_streaming(
        &self,
        messages: &[Message],
        tools: &[Value],
        tool_choice_mode: ToolChoiceMode,
    ) -> hone_core::HoneResult<ChatResponse> {
        let mut stream = self
            .llm
            .chat_with_tools_stream(messages, tools, None, tool_choice_mode);
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut tool_calls = BTreeMap::<u32, PendingToolCall>::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut emitted_visible_content = false;

        while let Some(event) = stream.next().await {
            match event? {
                ChatStreamEvent::ContentDelta(delta) => {
                    content.push_str(&delta);
                    let visible = formatter.push(&delta);
                    if !visible.is_empty() && tool_calls.is_empty() {
                        if let Some(observer) = &self.stream_observer {
                            observer.on_content_delta(&visible).await;
                            emitted_visible_content = true;
                        }
                    }
                }
                ChatStreamEvent::ReasoningDelta(delta) => reasoning_content.push_str(&delta),
                ChatStreamEvent::ToolCallDelta {
                    index,
                    id,
                    name,
                    arguments,
                } => {
                    if tool_calls.is_empty() && emitted_visible_content {
                        if let Some(observer) = &self.stream_observer {
                            observer.on_content_reset().await;
                        }
                        emitted_visible_content = false;
                    }
                    let pending = tool_calls.entry(index).or_default();
                    if let Some(id) = id {
                        pending.id.push_str(&id);
                    }
                    if let Some(name) = name {
                        pending.name.push_str(&name);
                    }
                    pending.arguments.push_str(&arguments);
                }
                ChatStreamEvent::Usage(value) => usage = Some(value),
            }
        }

        if tool_calls.is_empty() {
            let visible = formatter.finish();
            if !visible.is_empty()
                && let Some(observer) = &self.stream_observer
            {
                observer.on_content_delta(&visible).await;
            }
        }

        let tool_calls = (!tool_calls.is_empty()).then(|| {
            tool_calls
                .into_values()
                .map(|pending| ToolCall {
                    id: pending.id,
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: pending.name,
                        arguments: pending.arguments,
                    },
                })
                .collect()
        });

        Ok(ChatResponse {
            content,
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls,
            usage,
        })
    }

    async fn chat_terminal_streaming(
        &self,
        messages: &[Message],
    ) -> hone_core::HoneResult<ChatResponse> {
        let empty_tools = Vec::<Value>::new();
        let mut stream =
            self.llm
                .chat_with_tools_stream(messages, &empty_tools, None, ToolChoiceMode::Auto);
        let mut content = String::new();
        let mut reasoning_content = String::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut unexpected_tool_call = false;

        while let Some(event) = stream.next().await {
            match event? {
                ChatStreamEvent::ContentDelta(delta) => {
                    content.push_str(&delta);
                    let visible = formatter.push(&delta);
                    if !visible.is_empty()
                        && let Some(observer) = &self.stream_observer
                    {
                        observer.on_final_content_delta(&visible).await;
                    }
                }
                ChatStreamEvent::ReasoningDelta(delta) => reasoning_content.push_str(&delta),
                ChatStreamEvent::ToolCallDelta { .. } => unexpected_tool_call = true,
                ChatStreamEvent::Usage(value) => usage = Some(value),
            }
        }

        if unexpected_tool_call {
            return Err(hone_core::HoneError::Llm(
                "tool-free terminal synthesis returned a tool call".to_string(),
            ));
        }

        let visible = formatter.finish();
        if !visible.is_empty()
            && let Some(observer) = &self.stream_observer
        {
            observer.on_final_content_delta(&visible).await;
        }

        Ok(ChatResponse {
            content,
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls: None,
            usage,
        })
    }
}

fn finish_research_tool_schema() -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": FINISH_RESEARCH_TOOL_NAME,
            "description": "Internal control signal. Call this function by itself after all reasonable research attempts for the user's question are complete, including when a required source is unavailable and that gap must be disclosed. Company relationships and current causal claims require current web/news evidence; valuation labels require their actual denominator period and inputs. Hone will then ask you for the final answer with tools disabled. Never call it together with another function.",
            "parameters": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }
    })
}

fn is_finish_research_call(tool_call: &ToolCall) -> bool {
    tool_call.function.name == FINISH_RESEARCH_TOOL_NAME
}

fn starts_investment_terminal_protocol(tool_call: &ToolCall) -> bool {
    tool_call.function.name == "data_fetch"
}

// Keep the agent crate independent from channel presentation code while using
// the same hidden-tag semantics for incremental model output.
mod hone_channels_compat {
    #[derive(Default)]
    pub(super) struct HiddenStreamFormatter {
        pending: String,
        hidden: Option<&'static str>,
    }

    impl HiddenStreamFormatter {
        pub(super) fn push(&mut self, chunk: &str) -> String {
            self.pending.push_str(chunk);
            let mut visible = String::new();
            loop {
                if let Some(close) = self.hidden {
                    let Some(end) = self.pending.find(close) else {
                        break;
                    };
                    self.pending.drain(..end + close.len());
                    self.hidden = None;
                    continue;
                }
                let markers = [
                    ("<think>", "</think>"),
                    ("<tool_code>", "</tool_code>"),
                    ("<tool_call>", "</tool_call>"),
                    ("<tool_result>", "</tool_result>"),
                    ("<tool_use>", "</tool_use>"),
                ];
                if let Some((start, open, close)) = markers
                    .iter()
                    .filter_map(|(open, close)| {
                        self.pending.find(open).map(|start| (start, *open, *close))
                    })
                    .min_by_key(|(start, _, _)| *start)
                {
                    visible.push_str(&self.pending[..start]);
                    self.pending.drain(..start + open.len());
                    self.hidden = Some(close);
                    continue;
                }
                let keep = markers
                    .iter()
                    .map(|(open, _)| trailing_prefix_len(&self.pending, open))
                    .max()
                    .unwrap_or(0);
                let emit_len = self.pending.len().saturating_sub(keep);
                visible.push_str(&self.pending[..emit_len]);
                self.pending.drain(..emit_len);
                break;
            }
            visible
        }

        pub(super) fn finish(&mut self) -> String {
            if self.hidden.is_some() {
                self.pending.clear();
                return String::new();
            }
            std::mem::take(&mut self.pending)
        }
    }

    fn trailing_prefix_len(text: &str, marker: &str) -> usize {
        (1..marker.len())
            .rev()
            .find(|length| text.ends_with(&marker[..*length]))
            .unwrap_or(0)
    }
}

fn tool_budget_error(
    tool_name: &str,
    max_tool_calls: Option<u32>,
    tool_call_limits: &HashMap<String, u32>,
    total_tool_calls: u32,
    tool_call_counts: &HashMap<String, u32>,
) -> Option<Value> {
    if let Some(limit) = max_tool_calls
        && total_tool_calls >= limit
    {
        tracing::warn!(
            tool = tool_name,
            limit,
            "function_calling tool call rejected by global budget"
        );
        return Some(serde_json::json!({
            "error": format!("tool call limit reached ({limit})")
        }));
    }

    let Some(limit) = tool_call_limits.get(tool_name).copied() else {
        return None;
    };
    let used = tool_call_counts.get(tool_name).copied().unwrap_or(0);
    if used >= limit {
        tracing::warn!(
            tool = tool_name,
            limit,
            used,
            "function_calling tool call rejected by per-tool budget"
        );
        return Some(serde_json::json!({
            "error": format!("tool `{tool_name}` call limit reached ({limit})")
        }));
    }
    None
}

#[async_trait]
impl Agent for FunctionCallingAgent {
    /// 运行一次非流式 Agent turn，直到没有新的工具调用或达到迭代上限。
    ///
    /// 1. 接收用户输入
    /// 2. 调用 LLM，传入可用工具列表
    /// 3. 如果 LLM 返回 `tool_calls`，执行对应工具
    /// 4. 将工具结果反馈给 LLM
    /// 5. 重复 2-4 直到 LLM 返回最终答案
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse {
        context.add_user_message(user_input);

        let business_tools: Vec<Value> = self.tools.get_tools_schema();
        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        let mut tool_call_counts: HashMap<String, u32> = HashMap::new();
        let mut total_tool_calls = 0u32;
        let mut iterations: u32 = 0;
        let mut investment_research_started = false;

        self.dbg(&format!(
            "[Agent] start tools={:?}",
            self.tools.list_tool_names()
        ));

        loop {
            if iterations >= self.max_iterations {
                return AgentResponse {
                    content: String::new(),
                    tool_calls_made,
                    iterations,
                    success: false,
                    error: Some(format!("max_iterations_exceeded:{}", self.max_iterations)),
                };
            }
            iterations += 1;
            self.dbg(&format!("[Agent] iter={iterations}"));

            let finish_research_available =
                self.finish_research_terminal_synthesis && investment_research_started;
            let mut round_tools = business_tools.clone();
            if finish_research_available {
                round_tools.push(finish_research_tool_schema());
            }
            let has_tools = !round_tools.is_empty();
            let tool_choice_mode = if finish_research_available {
                ToolChoiceMode::Required
            } else {
                ToolChoiceMode::Auto
            };
            let messages = self.build_messages(context, finish_research_available);
            let request_payload = serde_json::json!({
                "messages": messages.clone(),
                "tools": if has_tools { Some(round_tools.clone()) } else { None },
                "tool_choice_mode": format!("{tool_choice_mode:?}"),
            });
            let call_started = std::time::Instant::now();

            // 如果有工具，使用 chat_with_tools；否则使用 chat
            let result: ChatResponse = if has_tools {
                match self
                    .chat_with_tools_streaming(&messages, &round_tools, tool_choice_mode)
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        self.record_audit(
                            context,
                            "chat_with_tools",
                            request_payload,
                            None,
                            Some(e.to_string()),
                            call_started.elapsed().as_millis(),
                            serde_json::json!({ "iteration": iterations, "has_tools": true }),
                            None,
                        );
                        return AgentResponse {
                            content: String::new(),
                            tool_calls_made,
                            iterations,
                            success: false,
                            error: Some(e.to_string()),
                        };
                    }
                }
            } else {
                match self.llm.chat(&messages, None).await {
                    Ok(r) => ChatResponse {
                        content: r.content,
                        reasoning_content: None,
                        tool_calls: None,
                        usage: r.usage,
                    },
                    Err(e) => {
                        self.record_audit(
                            context,
                            "chat",
                            request_payload,
                            None,
                            Some(e.to_string()),
                            call_started.elapsed().as_millis(),
                            serde_json::json!({ "iteration": iterations, "has_tools": false }),
                            None,
                        );
                        return AgentResponse {
                            content: String::new(),
                            tool_calls_made,
                            iterations,
                            success: false,
                            error: Some(e.to_string()),
                        };
                    }
                }
            };

            self.record_audit(
                context,
                if has_tools { "chat_with_tools" } else { "chat" },
                request_payload,
                Some(serde_json::json!({
                    "content": result.content.clone(),
                    "tool_calls": result.tool_calls.clone()
                })),
                None,
                call_started.elapsed().as_millis(),
                serde_json::json!({ "iteration": iterations, "has_tools": has_tools }),
                result.usage.clone(),
            );

            // 检查是否有工具调用
            if let Some(ref tcs) = result.tool_calls {
                let tcs: &Vec<hone_llm::ToolCall> = tcs;
                if !tcs.is_empty() {
                    self.dbg(&format!("[Agent] tool_calls n={}", tcs.len()));

                    let sole_finish_research =
                        finish_research_available && tcs.iter().all(is_finish_research_call);
                    if sole_finish_research {
                        // Keep the control protocol ephemeral: it selects the
                        // terminal phase but is not persisted as a business
                        // tool call or exposed to tool observers/budgets.
                        let mut terminal_messages = messages.clone();
                        terminal_messages.push(Message {
                            role: "user".to_string(),
                            content: Some(TERMINAL_SYNTHESIS_PROMPT.to_string()),
                            reasoning_content: None,
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                        });
                        iterations = iterations.saturating_add(1);
                        let terminal_request_payload = serde_json::json!({
                            "messages": terminal_messages.clone(),
                            "tools": Vec::<Value>::new(),
                        });
                        let terminal_started = std::time::Instant::now();
                        let terminal_result =
                            match self.chat_terminal_streaming(&terminal_messages).await {
                                Ok(response) => response,
                                Err(error) => {
                                    self.record_audit(
                                        context,
                                        "chat_terminal_without_tools",
                                        terminal_request_payload,
                                        None,
                                        Some(error.to_string()),
                                        terminal_started.elapsed().as_millis(),
                                        serde_json::json!({
                                            "iteration": iterations,
                                            "has_tools": false,
                                            "finish_research": true,
                                        }),
                                        None,
                                    );
                                    return AgentResponse {
                                        content: String::new(),
                                        tool_calls_made,
                                        iterations,
                                        success: false,
                                        error: Some(error.to_string()),
                                    };
                                }
                            };
                        self.record_audit(
                            context,
                            "chat_terminal_without_tools",
                            terminal_request_payload,
                            Some(serde_json::json!({
                                "content": terminal_result.content.clone(),
                                "tool_calls": terminal_result.tool_calls.clone(),
                            })),
                            None,
                            terminal_started.elapsed().as_millis(),
                            serde_json::json!({
                                "iteration": iterations,
                                "has_tools": false,
                                "finish_research": true,
                            }),
                            terminal_result.usage.clone(),
                        );

                        let metadata =
                            terminal_result.reasoning_content.as_ref().map(|reasoning| {
                                std::collections::HashMap::from([(
                                    REASONING_CONTENT_METADATA_KEY.to_string(),
                                    Value::String(reasoning.clone()),
                                )])
                            });
                        context.add_assistant_message_with_metadata(
                            &terminal_result.content,
                            None,
                            metadata,
                        );
                        return AgentResponse {
                            content: terminal_result.content,
                            tool_calls_made,
                            iterations,
                            success: true,
                            error: None,
                        };
                    }

                    // A mixed finish_research call is only a malformed control
                    // signal. Ignore it and continue executing the real calls;
                    // it must never consume tool budget, reach ToolRegistry, be
                    // persisted as ToolCallMade, or notify the business observer.
                    let actionable_tool_calls = tcs
                        .iter()
                        .filter(|tool_call| {
                            !self.finish_research_terminal_synthesis
                                || !is_finish_research_call(tool_call)
                        })
                        .collect::<Vec<_>>();

                    investment_research_started |= actionable_tool_calls
                        .iter()
                        .any(|tool_call| starts_investment_terminal_protocol(tool_call));

                    if actionable_tool_calls.is_empty() {
                        // The control was hallucinated before the finance
                        // terminal phase exposed it. Ignore the unavailable
                        // control and preserve any direct content without
                        // starting a second generation.
                        self.dbg("[Agent] ignored unavailable finish_research call");
                    } else {
                        // 记录 assistant 消息（只含真实业务 tool_calls）
                        let tc_values: Vec<Value> = actionable_tool_calls
                            .iter()
                            .filter_map(|tc| serde_json::to_value(*tc).ok())
                            .collect();
                        let metadata = result.reasoning_content.as_ref().map(|reasoning| {
                            std::collections::HashMap::from([(
                                REASONING_CONTENT_METADATA_KEY.to_string(),
                                Value::String(reasoning.clone()),
                            )])
                        });
                        context.add_assistant_message_with_metadata(
                            &result.content,
                            Some(tc_values),
                            metadata,
                        );

                        // 逐个执行真实业务工具
                        for tc in actionable_tool_calls {
                            let tool_name = &tc.function.name;
                            let tool_call_id = &tc.id;
                            let tool_args_str = &tc.function.arguments;

                            match serde_json::from_str::<Value>(tool_args_str) {
                                Ok(tool_args) => {
                                    self.dbg(&format!("[Agent] tool_call name={tool_name}"));
                                    if let Some(error_result) = tool_budget_error(
                                        tool_name,
                                        self.max_tool_calls,
                                        &self.tool_call_limits,
                                        total_tool_calls,
                                        &tool_call_counts,
                                    ) {
                                        let result_str = serde_json::to_string(&error_result)
                                            .unwrap_or_default();
                                        context.add_tool_result(
                                            tool_call_id,
                                            tool_name,
                                            &result_str,
                                        );
                                        continue;
                                    }
                                    total_tool_calls += 1;
                                    *tool_call_counts.entry(tool_name.clone()).or_insert(0) += 1;
                                    if let Some(observer) = &self.tool_observer {
                                        observer.on_tool_start(tool_name, &tool_args, None).await;
                                    }

                                    match self
                                        .tools
                                        .execute_tool(tool_name, tool_args.clone())
                                        .await
                                    {
                                        Ok(tool_result) => {
                                            self.dbg(&format!(
                                                "[Agent] tool_result name={tool_name}"
                                            ));

                                            let tr: Value = tool_result.clone();
                                            tool_calls_made.push(ToolCallMade {
                                                name: tool_name.clone(),
                                                arguments: tool_args.clone(),
                                                result: tr,
                                                tool_call_id: Some(tool_call_id.clone()),
                                            });

                                            let result_str = serde_json::to_string(&tool_result)
                                                .unwrap_or_default();
                                            context.add_tool_result(
                                                tool_call_id,
                                                tool_name,
                                                &result_str,
                                            );
                                            if let Some(observer) = &self.tool_observer {
                                                observer
                                                    .on_tool_finish(tool_name, &tool_args, true)
                                                    .await;
                                            }
                                        }
                                        Err(e) => {
                                            self.dbg(&format!(
                                                "[Agent] tool_error name={tool_name} error={e}"
                                            ));
                                            let err_str = e.to_string();
                                            let error_result: Value =
                                                serde_json::json!({"error": err_str});
                                            let result_str = serde_json::to_string(&error_result)
                                                .unwrap_or_default();
                                            context.add_tool_result(
                                                tool_call_id,
                                                tool_name,
                                                &result_str,
                                            );
                                            if let Some(observer) = &self.tool_observer {
                                                observer
                                                    .on_tool_finish(tool_name, &tool_args, false)
                                                    .await;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.dbg(&format!(
                                        "[Agent] json parse error for {tool_name}: {e}"
                                    ));
                                    let err_str = format!("参数解析失败: {e}");
                                    let error_result: Value = serde_json::json!({"error": err_str});
                                    let result_str =
                                        serde_json::to_string(&error_result).unwrap_or_default();
                                    context.add_tool_result(tool_call_id, tool_name, &result_str);
                                }
                            }
                        }
                        // 继续循环 — 把真实工具结果送回 LLM
                        continue;
                    }
                }
            }

            // 没有工具调用 — 最终回复
            self.dbg("[Agent] done (no more tool_calls)");
            if finish_research_available {
                tracing::warn!(
                    iterations,
                    business_tool_calls = tool_calls_made.len(),
                    "function_calling provider bypassed required terminal signal; preserving the Agent's direct answer without rewrite"
                );
            }
            let metadata = result.reasoning_content.as_ref().map(|reasoning| {
                std::collections::HashMap::from([(
                    REASONING_CONTENT_METADATA_KEY.to_string(),
                    Value::String(reasoning.clone()),
                )])
            });
            context.add_assistant_message_with_metadata(&result.content, None, metadata);
            return AgentResponse {
                content: result.content,
                tool_calls_made,
                iterations,
                success: true,
                error: None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::stream::{self, BoxStream};
    use hone_core::ToolExecutionObserver;
    use hone_core::agent::AgentContext;
    use hone_tools::{Tool, ToolParameter};
    use serde_json::{Value, json};
    use std::collections::VecDeque;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Clone)]
    struct StreamingMockLlmProvider {
        rounds: Arc<Mutex<VecDeque<Vec<ChatStreamEvent>>>>,
        seen_tool_counts: Arc<Mutex<Vec<usize>>>,
        seen_tool_choice_modes: Arc<Mutex<Vec<ToolChoiceMode>>>,
        seen_messages: Arc<Mutex<Vec<Vec<Message>>>>,
    }

    impl StreamingMockLlmProvider {
        fn with_rounds(rounds: Vec<Vec<ChatStreamEvent>>) -> Self {
            Self {
                rounds: Arc::new(Mutex::new(rounds.into())),
                seen_tool_counts: Arc::new(Mutex::new(Vec::new())),
                seen_tool_choice_modes: Arc::new(Mutex::new(Vec::new())),
                seen_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for StreamingMockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            unreachable!("streaming test uses tools")
        }

        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            unreachable!("native streaming override should be used")
        }

        fn chat_with_tools_stream<'a>(
            &'a self,
            messages: &'a [Message],
            tools: &'a [Value],
            _model: Option<&'a str>,
            tool_choice_mode: ToolChoiceMode,
        ) -> BoxStream<'a, hone_core::HoneResult<ChatStreamEvent>> {
            self.seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .push(tools.len());
            self.seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .push(tool_choice_mode);
            self.seen_messages
                .lock()
                .expect("stream messages lock")
                .push(messages.to_vec());
            let events = self
                .rounds
                .lock()
                .expect("stream rounds lock")
                .pop_front()
                .expect("stream round");
            Box::pin(stream::iter(events.into_iter().map(Ok)))
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    #[derive(Default)]
    struct RecordingStreamObserver {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl FunctionCallingStreamObserver for RecordingStreamObserver {
        async fn on_content_delta(&self, content: &str) {
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("delta:{content}"));
        }

        async fn on_final_content_delta(&self, content: &str) {
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("final:{content}"));
        }

        async fn on_content_reset(&self) {
            self.events
                .lock()
                .expect("stream events lock")
                .push("reset".to_string());
        }
    }

    #[derive(Clone)]
    struct MockLlmProvider {
        state: Arc<Mutex<MockState>>,
    }

    struct MockState {
        chat_calls: usize,
        chat_with_tools_calls: usize,
        next_chat_response: Option<String>,
        next_tool_responses: VecDeque<ChatResponse>,
        seen_tool_messages: Vec<Vec<Message>>,
    }

    impl MockLlmProvider {
        fn with_chat_response(content: &str) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    next_chat_response: Some(content.to_string()),
                    next_tool_responses: VecDeque::new(),
                    seen_tool_messages: Vec::new(),
                })),
            }
        }

        fn with_tool_responses(responses: Vec<ChatResponse>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    next_chat_response: None,
                    next_tool_responses: responses.into(),
                    seen_tool_messages: Vec::new(),
                })),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            let mut state = self.state.lock().expect("mock state lock");
            state.chat_calls += 1;
            Ok(hone_llm::provider::ChatResult {
                content: state
                    .next_chat_response
                    .clone()
                    .unwrap_or_else(|| "mock chat".to_string()),
                usage: None,
            })
        }

        async fn chat_with_tools(
            &self,
            messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            let mut state = self.state.lock().expect("mock state lock");
            state.chat_with_tools_calls += 1;
            state.seen_tool_messages.push(messages.to_vec());
            match state.next_tool_responses.pop_front() {
                Some(mock_tool_response) => Ok(mock_tool_response),
                None => Err(hone_core::HoneError::Llm(
                    "no more mock tool responses".to_string(),
                )),
            }
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    struct EchoTool;

    #[async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo_tool"
        }

        fn description(&self) -> &str {
            "echo"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![ToolParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "text".to_string(),
                required: true,
                r#enum: None,
                items: None,
            }]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "echo": args.get("text").and_then(|v| v.as_str()).unwrap_or_default()
            }))
        }
    }

    struct FinanceEvidenceTool;

    #[async_trait]
    impl Tool for FinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "finance evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![ToolParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "text".to_string(),
                required: false,
                r#enum: None,
                items: None,
            }]
        }

        async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({
                "evidence": args.get("text").and_then(|v| v.as_str()).unwrap_or_default()
            }))
        }
    }

    struct FailingFinanceEvidenceTool;

    #[async_trait]
    impl Tool for FailingFinanceEvidenceTool {
        fn name(&self) -> &str {
            "data_fetch"
        }

        fn description(&self) -> &str {
            "unavailable finance evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Err(hone_core::HoneError::Tool(
                "finance provider unavailable".to_string(),
            ))
        }
    }

    struct CountingTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for CountingTool {
        fn name(&self) -> &str {
            "counting_tool"
        }

        fn description(&self) -> &str {
            "count"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            let calls = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(json!({ "calls": calls }))
        }
    }

    #[derive(Default)]
    struct MockToolObserver {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl ToolExecutionObserver for MockToolObserver {
        async fn on_tool_start(
            &self,
            tool_name: &str,
            _arguments: &Value,
            _reasoning: Option<String>,
        ) {
            self.events
                .lock()
                .expect("observer lock")
                .push(format!("start:{tool_name}"));
        }

        async fn on_tool_finish(&self, tool_name: &str, _arguments: &Value, success: bool) {
            self.events
                .lock()
                .expect("observer lock")
                .push(format!("done:{tool_name}:{success}"));
        }
    }

    #[tokio::test]
    async fn run_without_tools_uses_chat_once() {
        let llm = MockLlmProvider::with_chat_response("plain response");
        let tools = Arc::new(ToolRegistry::new());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm.clone()), tools, "system".to_string(), 3, None);
        let mut context = AgentContext::new("s1".to_string());

        let response = agent.run("hello", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "plain response");
        assert_eq!(response.iterations, 1);
        assert!(response.tool_calls_made.is_empty());

        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.chat_calls, 1);
        assert_eq!(state.chat_with_tools_calls, 0);
    }

    #[tokio::test]
    async fn run_with_tool_call_executes_tool_and_returns_final_answer() {
        let tool_call = hone_llm::ToolCall {
            id: "tc_1".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: r#"{"text":"abc"}"#.to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "let me call tool".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm.clone()),
            Arc::new(registry),
            "system".to_string(),
            4,
            None,
        );
        let mut context = AgentContext::new("s2".to_string());

        let response = agent.run("trigger tool", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "done");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "echo_tool");
        assert_eq!(response.tool_calls_made[0].result["echo"], "abc");

        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.chat_calls, 0);
        assert_eq!(state.chat_with_tools_calls, 2);
    }

    #[tokio::test]
    async fn native_stream_resets_tool_preamble_and_hides_reasoning_from_final_deltas() {
        let llm = StreamingMockLlmProvider {
            rounds: Arc::new(Mutex::new(VecDeque::from([
                vec![
                    ChatStreamEvent::ContentDelta("I will check".to_string()),
                    ChatStreamEvent::ToolCallDelta {
                        index: 0,
                        id: Some("tc_stream".to_string()),
                        name: Some("echo_tool".to_string()),
                        arguments: "{\"text\":".to_string(),
                    },
                    ChatStreamEvent::ToolCallDelta {
                        index: 0,
                        id: None,
                        name: None,
                        arguments: "\"abc\"}".to_string(),
                    },
                ],
                vec![
                    ChatStreamEvent::ContentDelta("<thi".to_string()),
                    ChatStreamEvent::ContentDelta("nk>secret</think>最终".to_string()),
                    ChatStreamEvent::ContentDelta("答案".to_string()),
                ],
            ]))),
            seen_tool_counts: Arc::new(Mutex::new(Vec::new())),
            seen_tool_choice_modes: Arc::new(Mutex::new(Vec::new())),
            seen_messages: Arc::new(Mutex::new(Vec::new())),
        };
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("native-stream".to_string());

        let response = agent.run("stream", &mut context).await;

        assert!(response.success);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].result["echo"], "abc");
        assert_eq!(response.content, "<think>secret</think>最终答案");
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:I will check", "reset", "delta:最终", "delta:答案"]
        );
    }

    #[tokio::test]
    async fn sole_finish_research_runs_one_tool_free_terminal_stream_in_the_same_agent_run() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"text":"abc"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ReasoningDelta("terminal reasoning".to_string()),
                ChatStreamEvent::ContentDelta("最终".to_string()),
                ChatStreamEvent::ContentDelta("答案".to_string()),
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let stream_observer = Arc::new(RecordingStreamObserver::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            4,
            None,
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(stream_observer.clone()))
        .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("finish-research-terminal".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "最终答案");
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 2, 0],
            "finish_research must appear only after finance research starts, then terminal synthesis must use an empty tool list"
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ],
            "the first turn must permit direct non-investment answers; only an active finance research loop requires a tool decision"
        );
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:最终", "final:答案"]
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            ["start:data_fetch", "done:data_fetch:true"]
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
        let terminal_messages = seen_messages
            .lock()
            .expect("stream messages lock")
            .last()
            .cloned()
            .expect("terminal messages");
        assert!(
            terminal_messages
                .first()
                .and_then(|message| message.content.as_deref())
                .is_some_and(|content| content.contains(FINISH_RESEARCH_SYSTEM_INSTRUCTION))
        );
        assert_eq!(
            terminal_messages
                .last()
                .and_then(|message| message.content.as_deref()),
            Some(TERMINAL_SYNTHESIS_PROMPT)
        );
    }

    #[tokio::test]
    async fn direct_answer_fallback_does_not_start_a_second_terminal_generation() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ContentDelta(
            "直接答案".to_string(),
        )]]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("finish-research-direct".to_string());

        let response = agent.run("answer directly", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "直接答案");
        assert_eq!(response.iterations, 1);
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1],
            "a direct answer must not see finish_research or be followed by an empty-tools rewrite"
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [ToolChoiceMode::Auto],
            "a turn that has not entered the finance tool chain must preserve ordinary direct answers"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:直接答案"]
        );
    }

    #[tokio::test]
    async fn unavailable_finance_evidence_can_finish_with_a_disclosed_gap() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch_failed".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish_after_gap".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "本轮财务源不可用；以下仅分析已核验部分。".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FailingFinanceEvidenceTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            4,
            None,
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer));
        let mut context = AgentContext::new("finish-research-after-gap".to_string());

        let response = agent
            .run("research with unavailable evidence", &mut context)
            .await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "本轮财务源不可用；以下仅分析已核验部分。");
        assert!(response.tool_calls_made.is_empty());
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 2, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ]
        );
    }

    #[tokio::test]
    async fn duplicate_finish_calls_are_normalized_to_one_terminal_generation() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_finish_1".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_finish_2".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta("唯一终稿".to_string())],
        ]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("duplicate-finish".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "唯一终稿");
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
    }

    #[tokio::test]
    async fn mixed_finish_research_is_ignored_while_the_business_tool_executes_normally() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_finish_mixed".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_echo_mixed".to_string()),
                    name: Some("echo_tool".to_string()),
                    arguments: r#"{"text":"mixed"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta("完成".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let stream_observer = Arc::new(RecordingStreamObserver::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_finish_research_terminal_synthesis(true)
                .with_tool_call_budget(Some(1), HashMap::new())
                .with_stream_observer(Some(stream_observer.clone()))
                .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("finish-research-mixed".to_string());

        let response = agent.run("mixed", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "完成");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "echo_tool");
        assert_eq!(response.tool_calls_made[0].result["echo"], "mixed");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1],
            "an unavailable hallucinated finish_research must be ignored and must not activate terminal synthesis for a non-finance tool"
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            ["start:echo_tool", "done:echo_tool:true"],
            "finish_research must not reach the business tool observer"
        );
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:完成"]
        );
        assert!(context.messages.iter().all(|message| {
            message.tool_calls.as_ref().is_none_or(|tool_calls| {
                tool_calls.iter().all(|tool_call| {
                    tool_call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        != Some(FINISH_RESEARCH_TOOL_NAME)
                })
            })
        }));
    }

    #[tokio::test]
    async fn run_handles_invalid_tool_arguments_and_continues() {
        let invalid_tool_call = hone_llm::ToolCall {
            id: "tc_bad".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: "{not json}".to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "try tool".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![invalid_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "fallback final".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None);
        let mut context = AgentContext::new("s3".to_string());

        let response = agent.run("bad args", &mut context).await;

        assert!(response.success);
        assert_eq!(response.content, "fallback final");
        assert!(response.tool_calls_made.is_empty());
        let tool_msgs: Vec<_> = context
            .messages
            .iter()
            .filter(|m| m.role == "tool")
            .collect();
        assert_eq!(tool_msgs.len(), 1);
        let tool_msg_content = tool_msgs[0].content.clone().unwrap_or_default();
        assert!(tool_msg_content.contains("参数解析失败"));
    }

    #[tokio::test]
    async fn run_rejects_tool_calls_after_per_tool_budget() {
        let first_tool_call = hone_llm::ToolCall {
            id: "tc_1".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "counting_tool".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let second_tool_call = hone_llm::ToolCall {
            id: "tc_2".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "counting_tool".to_string(),
                arguments: "{}".to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "call once".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![first_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "call twice".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![second_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CountingTool {
            calls: calls.clone(),
        }));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_tool_call_budget(None, HashMap::from([("counting_tool".to_string(), 1)]));
        let mut context = AgentContext::new("budget".to_string());

        let response = agent.run("budget", &mut context).await;

        assert!(response.success);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(response.tool_calls_made.len(), 1);
        let tool_messages = context
            .messages
            .iter()
            .filter(|message| message.role == "tool")
            .collect::<Vec<_>>();
        assert_eq!(tool_messages.len(), 2);
        assert!(
            tool_messages[1]
                .content
                .as_deref()
                .unwrap_or_default()
                .contains("call limit reached")
        );
    }

    #[tokio::test]
    async fn run_notifies_tool_observer_on_execution() {
        let tool_call = hone_llm::ToolCall {
            id: "call_1".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: r#"{"echo":"abc"}"#.to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: "let me call tool".to_string(),
                reasoning_content: None,
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(MockToolObserver::default());
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            3,
            None,
        )
        .with_tool_observer(Some(observer.clone()));

        let actor = hone_core::ActorIdentity::new("web", "u1", None::<String>).expect("actor");
        let mut context = AgentContext::new("s1".to_string());
        context.set_actor_identity(&actor);
        let response = agent.run("trigger tool", &mut context).await;

        assert!(response.success);
        let events = observer.events.lock().expect("observer lock").clone();
        assert_eq!(events, vec!["start:echo_tool", "done:echo_tool:true"]);
    }

    #[tokio::test]
    async fn run_replays_reasoning_content_into_followup_tool_round() {
        let tool_call = hone_llm::ToolCall {
            id: "tc_reason".to_string(),
            call_type: "function".to_string(),
            function: hone_llm::FunctionCall {
                name: "echo_tool".to_string(),
                arguments: r#"{"text":"abc"}"#.to_string(),
            },
        };
        let llm = MockLlmProvider::with_tool_responses(vec![
            ChatResponse {
                content: String::new(),
                reasoning_content: Some("need tool lookup first".to_string()),
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            },
        ]);

        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm.clone()),
            Arc::new(registry),
            String::new(),
            4,
            None,
        );
        let mut context = AgentContext::new("s_reason".to_string());

        let response = agent.run("trigger tool", &mut context).await;

        assert!(response.success);
        let state = llm.state.lock().expect("mock state lock");
        assert_eq!(state.seen_tool_messages.len(), 2);
        let assistant = state.seen_tool_messages[1]
            .iter()
            .find(|message| message.role == "assistant")
            .expect("assistant followup message");
        assert_eq!(
            assistant.reasoning_content.as_deref(),
            Some("need tool lookup first")
        );
    }
}
