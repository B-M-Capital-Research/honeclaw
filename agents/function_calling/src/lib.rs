//! Hone Agent — Function Calling Agent 核心
//!
//! 基于 `OpenAI` Function Calling 模式的 legacy Agent 适配器。
//! 这里负责多轮工具调用循环，并把最终结果聚合成 `AgentResponse`；
//! 渠道级流式输出由 `hone-channels` 的 runner 层处理。

use async_trait::async_trait;
use futures::StreamExt;
use hone_core::agent::{Agent, AgentContext, AgentResponse, ToolCallMade};
use hone_core::{LlmAuditRecord, LlmAuditSink, ToolExecutionObserver};
use hone_llm::provider::ChatStreamFinishReason;
use hone_llm::{
    ChatResponse, ChatStreamEvent, FunctionCall, LlmProvider, Message, ToolCall, ToolChoiceMode,
};
use hone_tools::ToolRegistry;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;

const REASONING_CONTENT_METADATA_KEY: &str = "reasoning_content";
#[cfg(not(test))]
const ACTIVE_BUSINESS_TIMEOUT: Duration = Duration::from_secs(20);
#[cfg(test)]
const ACTIVE_BUSINESS_TIMEOUT: Duration = Duration::from_millis(25);
const FINISH_RESEARCH_TOOL_NAME: &str = "finish_research";
const ACTIVE_BUSINESS_FAILURE_RETRY_LIMIT: u32 = 1;
const POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION: &str = "【内部研究取证轮】当前已通过 DataFetch 进入金融数据工具链，但本轮只有实体发现证据，`finish_research` 尚不可用。请继续使用真实业务工具完成至少一次实体发现之后的证据尝试：按用户原始问题核验 exact-symbol quote/profile，并继续取得所需财务、新闻、网页、公告、持仓或其它业务证据。不得把 data_fetch(search) 或 profile 当成公司关系、事件或因果证据，也不要在本轮输出最终答案。";
const ACTIVE_RESEARCH_SYSTEM_INSTRUCTION: &str = "【内部研究工具轮】当前已进入金融数据工具链。本轮同时提供真实业务工具和 `finish_research`。请由同一 Agent 继续阅读用户原始问题与本轮真实工具结果：证据不足时调用当前最需要的一个或多个业务工具；只有合理的研究尝试已经完成，或必要来源已明确不可得并可如实披露时，才单独调用 `finish_research`。不要把完成信号与业务工具混用。若当前模型无法调用完成信号而以完整正文自然结束，只能在所需证据已经完成后输出一次符合最终回答契约的终稿；服务端会原样采用，不会另行审查或重写。实体 search/profile 只证明身份，不证明公司关系；关系、事件和因果结论必须先取得本轮 web/news/公告证据。";
const FINISH_RESEARCH_SYSTEM_INSTRUCTION: &str = "【显式完成后的终稿阶段】Agent 已在同一业务工具循环中显式确认本轮合理的研究与工具尝试完成，现由同一 Agent 和同一上下文进入无工具终稿阶段。这是证据整理而不是新的研究规划：直接组织终稿，不要重新展开工具决策或冗长隐藏推演。只能使用用户请求与此前已成功返回的业务工具结果；`reasoning_content`、隐藏思考、未采用草稿和内部状态文本都不是事实证据。缺失证据应如实披露但不构成拒答。";
const TERMINAL_SYNTHESIS_PROMPT: &str = concat!(
    "当前阶段不再提供任何工具；请只基于同一轮对话中已有的用户请求和此前已成功返回的业务工具结果，直接生成一次完整、可见的最终回答。",
    "`reasoning_content`、隐藏思考、未采用草稿、内部状态文本以及模型记忆都不是事实证据，不得从中提取或补齐关系、日期、行情、财务或估值事实。",
    "第一个可见行必须严格使用“数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：…”，且时间与口径只能来自已有工具上下文；没有行情证据时仍使用“行情口径”字段名并说明数据范围，不得伪造报价时间或盘前/盘后时段。",
    "逐项复核所有公司关系、新闻因果、日期、行情、财务与估值数字：实体 search/profile 只证明标的身份，不证明公司关系；关系、事件与因果结论必须有当前 web/news/公告或工具原文明确支持，搜索摘要只能按其明确表述的有限范围使用，其他事实必须标为未核验。未找到证据不等于事实不存在；‘无股权关系’、‘不是客户/供应商’、‘没有合同/合作’等否定结论同样必须有明确来源，否则只能说本轮未找到支持该关系的证据。",
    "年度数据不得写成 TTM；单季数据必须标明季度与报告期，年化时必须显示是“单季×4”还是“最近四季求和”及算术、分子分母口径，并披露季节性限制。",
    "未取得净债务或企业价值时不得使用 EV 或 EV/EBITDA 标签，也不得把市值/EBITDA 写成 EV/EBITDA。quote 返回的 PE 未明确标注 forward 时不得称为 Forward PE；已核验期间 EBITDA 为正时不得声称公司需到未来才转正。",
    "没有直接证据与完整输入时，不得给出目标价、概率、仓位比例、止损位或精确支撑位；第三方分析师目标价必须标注为第三方聚合口径与对应时间，不得直接作为交易锚点。",
    "某项证据不可得时，披露缺项并继续完成能够被当前证据支持的分析，不得因此拒绝整个问题。不要提及 finish_research、内部协议、工具循环、终态原因或这条提示。"
);

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

    /// Returns an exact user-visible prefix that has already crossed an
    /// irreversible channel boundary. Most observers buffer/reset all output
    /// and therefore return `None`; canonical terminal observers use this to
    /// permit a terminal-only transport recovery without rerunning tools.
    fn committed_visible_prefix(&self) -> Option<String> {
        None
    }

    async fn on_content_reset(&self);
}

#[derive(Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Debug, Default)]
struct ResearchEvidenceLedger {
    identity_only_attempts: u32,
    post_identity_attempts: u32,
}

impl ResearchEvidenceLedger {
    fn observe_business_call(&mut self, tool_call: &ToolCall, post_identity_round: bool) {
        // A malformed function payload never counts as an evidence attempt.
        // The normal execution path will return its parse error to the Agent,
        // which can then issue a corrected business call.
        if serde_json::from_str::<Value>(&tool_call.function.arguments).is_err() {
            return;
        }
        if is_identity_only_search_call(tool_call) {
            self.identity_only_attempts = self.identity_only_attempts.saturating_add(1);
        } else if post_identity_round {
            self.post_identity_attempts = self.post_identity_attempts.saturating_add(1);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StreamToolChoiceTelemetry {
    requested: ToolChoiceMode,
    effective: Option<ToolChoiceMode>,
    fallback: Option<bool>,
}

impl StreamToolChoiceTelemetry {
    fn new(requested: ToolChoiceMode) -> Self {
        Self {
            requested,
            effective: None,
            fallback: None,
        }
    }

    fn observe(
        &mut self,
        requested: ToolChoiceMode,
        effective: ToolChoiceMode,
        fallback: bool,
    ) -> hone_core::HoneResult<()> {
        if self.effective.is_some() {
            return Err(hone_core::HoneError::Llm(
                "stream returned duplicate tool choice metadata".to_string(),
            ));
        }
        if requested != self.requested {
            return Err(hone_core::HoneError::Llm(format!(
                "stream tool choice metadata mismatch: requested {}, expected {}",
                tool_choice_mode_name(requested),
                tool_choice_mode_name(self.requested),
            )));
        }
        self.effective = Some(effective);
        self.fallback = Some(fallback);
        Ok(())
    }
}

fn tool_choice_mode_name(mode: ToolChoiceMode) -> &'static str {
    match mode {
        ToolChoiceMode::Auto => "auto",
        ToolChoiceMode::Required => "required",
    }
}

fn observe_stream_finish(
    finish: &mut Option<ChatStreamFinishReason>,
    reason: ChatStreamFinishReason,
) -> hone_core::HoneResult<()> {
    if finish.is_some() {
        return Err(hone_core::HoneError::Llm(
            "stream returned duplicate finish reason".to_string(),
        ));
    }
    match reason {
        ChatStreamFinishReason::Stop | ChatStreamFinishReason::ToolCalls => {
            *finish = Some(reason);
            Ok(())
        }
        ChatStreamFinishReason::Length => Err(hone_core::HoneError::Llm(
            "stream completion was truncated (finish reason: length)".to_string(),
        )),
        ChatStreamFinishReason::ContentFilter => Err(hone_core::HoneError::Llm(
            "stream completion was blocked (finish reason: content_filter)".to_string(),
        )),
        ChatStreamFinishReason::Error => Err(hone_core::HoneError::Llm(
            "stream completion failed (finish reason: error)".to_string(),
        )),
        ChatStreamFinishReason::Other(reason) => Err(hone_core::HoneError::Llm(format!(
            "stream completion ended with unsupported finish reason: {reason}"
        ))),
    }
}

fn require_complete_stream(
    telemetry: &StreamToolChoiceTelemetry,
    finish: Option<ChatStreamFinishReason>,
    done: bool,
    expected_finish: ChatStreamFinishReason,
    operation: &str,
) -> hone_core::HoneResult<()> {
    if telemetry.effective.is_none() {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream ended without tool choice metadata"
        )));
    }
    if !done {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream ended before Done"
        )));
    }
    let Some(actual_finish) = finish else {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream reached Done without a finish reason"
        )));
    };
    if actual_finish != expected_finish {
        return Err(hone_core::HoneError::Llm(format!(
            "{operation} stream finish mismatch: expected {expected_finish:?}, got {actual_finish:?}"
        )));
    }
    Ok(())
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

    /// Enable the Agent-owned research terminal protocol. Once the Agent has
    /// actually attempted DataFetch in an eligible turn, the same business
    /// loop first requires a post-identity evidence attempt, then exposes the
    /// real actor-bound tools together with a sole `finish_research` signal.
    /// DataFetch is the structural finance-evidence boundary already required
    /// by the investment prompt; using it avoids a question-phrase classifier
    /// and does not force unrelated Web/file/skill tool turns into the
    /// canonical investment answer format. A sole finish signal performs one
    /// final tool-free streamed completion using the same in-memory context.
    /// Direct answers before finance research remain exact one-shot answers.
    pub fn with_finish_research_terminal_synthesis(mut self, enabled: bool) -> Self {
        self.finish_research_terminal_synthesis = enabled;
        self
    }

    fn dbg(&self, msg: &str) {
        if self.debug_log {
            tracing::debug!("{msg}");
        }
    }

    async fn reset_emitted_content(&self, emitted: bool) {
        if emitted && let Some(observer) = &self.stream_observer {
            // A committed canonical prefix is irreversible. Resetting after it
            // would make a successful buffered recovery impossible to append
            // byte-for-byte and can cause visible flicker in non-deferred
            // adapters.
            if observer.committed_visible_prefix().is_none() {
                observer.on_content_reset().await;
            }
        }
    }

    /// 构建完整消息列表（system prompt + context messages）
    fn build_messages(
        &self,
        context: &AgentContext,
        additional_system_instruction: Option<&str>,
    ) -> Vec<Message> {
        self.build_messages_from_index(context, additional_system_instruction, 0)
    }

    fn build_messages_from_index(
        &self,
        context: &AgentContext,
        additional_system_instruction: Option<&str>,
        message_start: usize,
    ) -> Vec<Message> {
        let message_start = message_start.min(context.messages.len());
        let mut messages =
            Vec::with_capacity(context.messages.len().saturating_sub(message_start) + 1);

        if !self.system_prompt.is_empty() || additional_system_instruction.is_some() {
            let system_prompt = match (self.system_prompt.is_empty(), additional_system_instruction)
            {
                (false, Some(instruction)) => {
                    format!("{}\n\n{}", self.system_prompt, instruction)
                }
                (true, Some(instruction)) => instruction.to_string(),
                (_, None) => self.system_prompt.clone(),
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

        for msg in &context.messages[message_start..] {
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
        emit_speculative_content: bool,
        telemetry: &mut StreamToolChoiceTelemetry,
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
        let mut finish = None;
        let mut done = false;

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(event) => event,
                Err(error) => {
                    self.reset_emitted_content(emitted_visible_content).await;
                    return Err(error);
                }
            };
            if !matches!(event, ChatStreamEvent::ToolChoiceMetadata { .. })
                && telemetry.effective.is_none()
            {
                return Err(hone_core::HoneError::Llm(
                    "chat_with_tools stream emitted payload before tool choice metadata"
                        .to_string(),
                ));
            }
            match event {
                ChatStreamEvent::ToolChoiceMetadata {
                    requested,
                    effective,
                    fallback,
                } => {
                    if let Err(error) = telemetry.observe(requested, effective, fallback) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::ContentDelta(delta) => {
                    content.push_str(&delta);
                    if emit_speculative_content {
                        let visible = formatter.push(&delta);
                        if !visible.is_empty() && tool_calls.is_empty() {
                            if let Some(observer) = &self.stream_observer {
                                observer.on_content_delta(&visible).await;
                                emitted_visible_content = true;
                            }
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
                    if emit_speculative_content && tool_calls.is_empty() && emitted_visible_content
                    {
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
                ChatStreamEvent::Finish(reason) => {
                    if let Err(error) = observe_stream_finish(&mut finish, reason) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::Done => {
                    done = true;
                    break;
                }
            }
        }

        let has_tool_calls = !tool_calls.is_empty();
        if let Err(error) = require_complete_stream(
            telemetry,
            finish,
            done,
            if has_tool_calls {
                ChatStreamFinishReason::ToolCalls
            } else {
                ChatStreamFinishReason::Stop
            },
            "chat_with_tools",
        ) {
            self.reset_emitted_content(emitted_visible_content).await;
            return Err(error);
        }

        if emit_speculative_content && !has_tool_calls {
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

    async fn chat_active_business_tools(
        &self,
        messages: &[Message],
        tools: &[Value],
        telemetry: &mut StreamToolChoiceTelemetry,
    ) -> hone_core::HoneResult<ActiveBusinessStreamOutcome> {
        let mut stream =
            self.llm
                .chat_with_tools_stream(messages, tools, None, ToolChoiceMode::Required);
        let mut reasoning_content = String::new();
        let mut tool_calls = BTreeMap::<u32, PendingToolCall>::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut visible_content = String::new();
        let mut finish = None;
        let mut done = false;

        while let Some(event) = stream.next().await {
            let event = event?;
            if !matches!(event, ChatStreamEvent::ToolChoiceMetadata { .. })
                && telemetry.effective.is_none()
            {
                return Err(hone_core::HoneError::Llm(
                    "active business stream emitted payload before tool choice metadata"
                        .to_string(),
                ));
            }
            match event {
                ChatStreamEvent::ToolChoiceMetadata {
                    requested,
                    effective,
                    fallback,
                } => telemetry.observe(requested, effective, fallback)?,
                // Some supported providers can still emit a short preamble
                // before a timely tool call, including after Required falls
                // back from a provider capability error. Keep it silent and
                // out of context, but continue polling for the tool call. The
                // outer ACTIVE_BUSINESS_TIMEOUT bounds a long/hung bypass.
                ChatStreamEvent::ContentDelta(delta) => {
                    visible_content.push_str(&formatter.push(&delta));
                }
                ChatStreamEvent::ReasoningDelta(delta) => reasoning_content.push_str(&delta),
                ChatStreamEvent::ToolCallDelta {
                    index,
                    id,
                    name,
                    arguments,
                } => {
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
                ChatStreamEvent::Finish(reason) => observe_stream_finish(&mut finish, reason)?,
                ChatStreamEvent::Done => {
                    done = true;
                    break;
                }
            }
        }
        visible_content.push_str(&formatter.finish());

        if tool_calls.is_empty() {
            require_complete_stream(
                telemetry,
                finish,
                done,
                ChatStreamFinishReason::Stop,
                "active business",
            )?;
            return Ok(if visible_content.trim().is_empty() {
                ActiveBusinessStreamOutcome::Empty
            } else {
                ActiveBusinessStreamOutcome::DirectFinal(ChatResponse {
                    content: visible_content,
                    reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
                    tool_calls: None,
                    usage,
                })
            });
        }

        require_complete_stream(
            telemetry,
            finish,
            done,
            ChatStreamFinishReason::ToolCalls,
            "active business",
        )?;

        let tool_calls = tool_calls
            .into_values()
            .map(|pending| ToolCall {
                id: pending.id,
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: pending.name,
                    arguments: pending.arguments,
                },
            })
            .collect::<Vec<_>>();
        Ok(ActiveBusinessStreamOutcome::Tools(ChatResponse {
            content: String::new(),
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls: Some(tool_calls),
            usage,
        }))
    }

    async fn chat_terminal_streaming(
        &self,
        messages: &[Message],
        telemetry: &mut StreamToolChoiceTelemetry,
        emit_to_observer: bool,
    ) -> hone_core::HoneResult<ChatResponse> {
        let empty_tools = Vec::<Value>::new();
        let mut stream =
            self.llm
                .chat_with_tools_stream(messages, &empty_tools, None, ToolChoiceMode::Auto);
        let mut visible_content = String::new();
        let mut reasoning_content = String::new();
        let mut usage = None;
        let mut formatter = hone_channels_compat::HiddenStreamFormatter::default();
        let mut unexpected_tool_call = false;
        let mut emitted_visible_content = false;
        let mut finish = None;
        let mut done = false;

        while let Some(event) = stream.next().await {
            let event = match event {
                Ok(event) => event,
                Err(error) => {
                    self.reset_emitted_content(emitted_visible_content).await;
                    return Err(error);
                }
            };
            if !matches!(event, ChatStreamEvent::ToolChoiceMetadata { .. })
                && telemetry.effective.is_none()
            {
                return Err(hone_core::HoneError::Llm(
                    "terminal stream emitted payload before tool choice metadata".to_string(),
                ));
            }
            match event {
                ChatStreamEvent::ToolChoiceMetadata {
                    requested,
                    effective,
                    fallback,
                } => {
                    if let Err(error) = telemetry.observe(requested, effective, fallback) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::ContentDelta(delta) => {
                    let visible = formatter.push(&delta);
                    visible_content.push_str(&visible);
                    if emit_to_observer
                        && !visible.is_empty()
                        && let Some(observer) = &self.stream_observer
                    {
                        observer.on_final_content_delta(&visible).await;
                        emitted_visible_content = true;
                    }
                }
                ChatStreamEvent::ReasoningDelta(delta) => reasoning_content.push_str(&delta),
                ChatStreamEvent::ToolCallDelta { .. } => unexpected_tool_call = true,
                ChatStreamEvent::Usage(value) => usage = Some(value),
                ChatStreamEvent::Finish(reason) => {
                    if let Err(error) = observe_stream_finish(&mut finish, reason) {
                        self.reset_emitted_content(emitted_visible_content).await;
                        return Err(error);
                    }
                }
                ChatStreamEvent::Done => {
                    done = true;
                    break;
                }
            }
        }

        if unexpected_tool_call {
            self.reset_emitted_content(emitted_visible_content).await;
            return Err(hone_core::HoneError::Llm(
                "tool-free terminal synthesis returned a tool call".to_string(),
            ));
        }

        if let Err(error) = require_complete_stream(
            telemetry,
            finish,
            done,
            ChatStreamFinishReason::Stop,
            "terminal synthesis",
        ) {
            self.reset_emitted_content(emitted_visible_content).await;
            return Err(error);
        }

        let visible = formatter.finish();
        visible_content.push_str(&visible);
        if emit_to_observer
            && !visible.is_empty()
            && let Some(observer) = &self.stream_observer
        {
            observer.on_final_content_delta(&visible).await;
        }

        if emit_to_observer
            && let Some(committed_prefix) = self
                .stream_observer
                .as_ref()
                .and_then(|observer| observer.committed_visible_prefix())
        {
            // A header-only terminal is not a complete answer. Treat this as
            // an interrupted terminal transport so run_terminal_synthesis can
            // use its one buffered, empty-tools recovery rather than publish a
            // bare timestamp line as success.
            validate_terminal_recovery_content(&visible_content, &committed_prefix)?;
        }

        Ok(ChatResponse {
            // Some compatible providers encode hidden reasoning inside the
            // content stream as <think> blocks. Return the same formatter-
            // reduced bytes that the observer sees so prefix validation,
            // persistence, and terminal recovery operate on one canonical
            // user-visible representation.
            content: visible_content,
            reasoning_content: (!reasoning_content.is_empty()).then_some(reasoning_content),
            tool_calls: None,
            usage,
        })
    }

    async fn run_terminal_synthesis(
        &self,
        context: &mut AgentContext,
        tool_calls_made: Vec<ToolCallMade>,
        completed_iterations: u32,
        turn_message_start: usize,
    ) -> AgentResponse {
        let iterations = completed_iterations.saturating_add(1);
        // Initial and business rounds retain bounded conversation history
        // so the Agent can understand follow-ups. Final factual synthesis must
        // not let an older ticker, quote, or assistant draft masquerade as
        // current-turn evidence, so its transcript begins at this run's user
        // message and contains only this turn's tool calls/results.
        let mut terminal_messages = self.build_messages_from_index(
            context,
            Some(FINISH_RESEARCH_SYSTEM_INSTRUCTION),
            turn_message_start,
        );
        // Provider reasoning and assistant prose emitted alongside tool calls
        // are useful at most as an intermediate draft; neither is fact
        // evidence. Scrub both at the terminal boundary so synthesis can rely
        // only on the user's request and actual tool results. This also covers
        // an early Web/search round that precedes the DataFetch activation
        // boundary for a finance turn.
        for message in &mut terminal_messages {
            message.reasoning_content = None;
            if message.role == "assistant"
                && message
                    .tool_calls
                    .as_ref()
                    .is_some_and(|tool_calls| !tool_calls.is_empty())
            {
                message.content = Some(String::new());
            }
        }
        let terminal_prompt = terminal_synthesis_prompt();
        terminal_messages.push(Message {
            role: "user".to_string(),
            content: Some(terminal_prompt),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        let terminal_request_payload = serde_json::json!({
            "messages": terminal_messages.clone(),
            "tools": Vec::<Value>::new(),
        });
        let terminal_started = std::time::Instant::now();
        let mut terminal_tool_choice = StreamToolChoiceTelemetry::new(ToolChoiceMode::Auto);
        let terminal_result = match self
            .chat_terminal_streaming(&terminal_messages, &mut terminal_tool_choice, true)
            .await
        {
            Ok(response) => {
                self.record_audit(
                    context,
                    "chat_terminal_without_tools",
                    terminal_request_payload,
                    Some(serde_json::json!({
                        "content": response.content.clone(),
                        "tool_calls": response.tool_calls.clone(),
                    })),
                    None,
                    terminal_started.elapsed().as_millis(),
                    serde_json::json!({
                        "iteration": iterations,
                        "has_tools": false,
                        "finish_research": true,
                        "terminal_reason": "explicit_finish",
                        "terminal_recovery_eligible": false,
                        "requested_tool_choice": tool_choice_mode_name(terminal_tool_choice.requested),
                        "effective_tool_choice": terminal_tool_choice.effective.map(tool_choice_mode_name),
                        "tool_choice_fallback": terminal_tool_choice.fallback,
                    }),
                    response.usage.clone(),
                );
                response
            }
            Err(error) => {
                let committed_prefix = self
                    .stream_observer
                    .as_ref()
                    .and_then(|observer| observer.committed_visible_prefix());
                self.record_audit(
                    context,
                    "chat_terminal_without_tools",
                    terminal_request_payload.clone(),
                    None,
                    Some(error.to_string()),
                    terminal_started.elapsed().as_millis(),
                    serde_json::json!({
                        "iteration": iterations,
                        "has_tools": false,
                        "finish_research": true,
                        "terminal_reason": "explicit_finish",
                        "terminal_recovery_eligible": committed_prefix.is_some(),
                        "requested_tool_choice": tool_choice_mode_name(terminal_tool_choice.requested),
                        "effective_tool_choice": terminal_tool_choice.effective.map(tool_choice_mode_name),
                        "tool_choice_fallback": terminal_tool_choice.fallback,
                    }),
                    None,
                );
                let Some(committed_prefix) = committed_prefix else {
                    return AgentResponse {
                        content: String::new(),
                        tool_calls_made,
                        iterations,
                        success: false,
                        error: Some(error.to_string()),
                    };
                };

                // The canonical header has already reached the user, so an
                // outer Agent/runner retry would either duplicate it or rerun
                // business tools. Retry this terminal transport exactly once,
                // buffered, against the same evidence and with tools disabled.
                let recovery_messages =
                    terminal_recovery_messages(&terminal_messages, &committed_prefix);
                let recovery_request_payload = serde_json::json!({
                    "messages": recovery_messages.clone(),
                    "tools": Vec::<Value>::new(),
                });
                let recovery_started = std::time::Instant::now();
                let mut recovery_tool_choice = StreamToolChoiceTelemetry::new(ToolChoiceMode::Auto);
                let recovery_result = self
                    .chat_terminal_streaming(&recovery_messages, &mut recovery_tool_choice, false)
                    .await
                    .and_then(|response| {
                        validate_terminal_recovery_content(&response.content, &committed_prefix)?;
                        Ok(response)
                    });

                match recovery_result {
                    Ok(response) => {
                        self.record_audit(
                            context,
                            "chat_terminal_recovery_without_tools",
                            recovery_request_payload,
                            Some(serde_json::json!({
                                "content": response.content.clone(),
                                "tool_calls": response.tool_calls.clone(),
                            })),
                            None,
                            recovery_started.elapsed().as_millis(),
                            serde_json::json!({
                                "iteration": iterations,
                                "has_tools": false,
                                "finish_research": true,
                                "terminal_reason": "explicit_finish",
                                "terminal_recovery": true,
                                "recovery_attempt": 1,
                                "committed_prefix_bytes": committed_prefix.len(),
                                "requested_tool_choice": tool_choice_mode_name(recovery_tool_choice.requested),
                                "effective_tool_choice": recovery_tool_choice.effective.map(tool_choice_mode_name),
                                "tool_choice_fallback": recovery_tool_choice.fallback,
                            }),
                            response.usage.clone(),
                        );
                        response
                    }
                    Err(recovery_error) => {
                        self.record_audit(
                            context,
                            "chat_terminal_recovery_without_tools",
                            recovery_request_payload,
                            None,
                            Some(recovery_error.to_string()),
                            recovery_started.elapsed().as_millis(),
                            serde_json::json!({
                                "iteration": iterations,
                                "has_tools": false,
                                "finish_research": true,
                                "terminal_reason": "explicit_finish",
                                "terminal_recovery": true,
                                "recovery_attempt": 1,
                                "committed_prefix_bytes": committed_prefix.len(),
                                "initial_terminal_error": error.to_string(),
                                "requested_tool_choice": tool_choice_mode_name(recovery_tool_choice.requested),
                                "effective_tool_choice": recovery_tool_choice.effective.map(tool_choice_mode_name),
                                "tool_choice_fallback": recovery_tool_choice.fallback,
                            }),
                            None,
                        );
                        return AgentResponse {
                            content: String::new(),
                            tool_calls_made,
                            iterations,
                            success: false,
                            error: Some(format!(
                                "terminal synthesis recovery failed: {recovery_error}"
                            )),
                        };
                    }
                }
            }
        };

        // Terminal reasoning is neither user-visible output nor fact evidence.
        // Do not persist it into context, where a later turn could replay it.
        context.add_assistant_message_with_metadata(&terminal_result.content, None, None);
        AgentResponse {
            content: terminal_result.content,
            tool_calls_made,
            iterations,
            success: true,
            error: None,
        }
    }
}

fn terminal_recovery_messages(messages: &[Message], committed_prefix: &str) -> Vec<Message> {
    let mut recovery_messages = messages.to_vec();
    let encoded_prefix = Value::String(committed_prefix.to_string()).to_string();
    let recovery_constraint = format!(
        "\n【终稿传输恢复】上一次终稿流在已提交首行后中断。请基于完全相同的事实证据重新生成完整终稿；第一个字节起必须逐字输出以下 JSON 字符串解码后的已提交前缀，前面不得有任何字符：{encoded_prefix}。前缀后必须继续输出非空正文，其余事实边界与格式要求不变。不要提及本次传输恢复。"
    );
    if let Some(prompt) = recovery_messages
        .last_mut()
        .and_then(|message| message.content.as_mut())
    {
        prompt.push_str(&recovery_constraint);
    } else {
        recovery_messages.push(Message {
            role: "user".to_string(),
            content: Some(recovery_constraint),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }
    recovery_messages
}

fn validate_terminal_recovery_content(
    content: &str,
    committed_prefix: &str,
) -> hone_core::HoneResult<()> {
    let Some(tail) = content.strip_prefix(committed_prefix) else {
        return Err(hone_core::HoneError::Llm(
            "terminal recovery content does not start with the committed visible prefix"
                .to_string(),
        ));
    };
    if tail.trim().is_empty() {
        return Err(hone_core::HoneError::Llm(
            "terminal recovery content contains no body after the committed visible prefix"
                .to_string(),
        ));
    }
    if tail.trim_start().starts_with(committed_prefix) {
        return Err(hone_core::HoneError::Llm(
            "terminal recovery content repeats the committed visible prefix".to_string(),
        ));
    }
    Ok(())
}

fn finish_research_tool_schema() -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": FINISH_RESEARCH_TOOL_NAME,
            "description": "Agent-owned terminal signal available in the same business-tool loop. Call it by itself only after all reasonable research attempts for the user's original question are complete, including when a required source is explicitly unavailable and that gap must be disclosed. Identity search/profile is never enough for company-relationship, event, or causal claims: those require current web/news/filing evidence. Valuation labels require the actual denominator period and inputs. Hone will then ask the same Agent for the final answer with tools disabled. Never call it together with another function.",
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

fn is_valid_finish_research_call(tool_call: &ToolCall) -> bool {
    is_finish_research_call(tool_call)
        && serde_json::from_str::<Value>(&tool_call.function.arguments)
            .ok()
            .is_some_and(|arguments| {
                arguments
                    .as_object()
                    .is_some_and(|arguments| arguments.is_empty())
            })
}

fn is_identity_only_search_call(tool_call: &ToolCall) -> bool {
    if !tool_call.function.name.eq_ignore_ascii_case("data_fetch") {
        return false;
    }
    serde_json::from_str::<Value>(&tool_call.function.arguments)
        .ok()
        .and_then(|arguments| {
            arguments
                .get("data_type")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|data_type| data_type.trim().eq_ignore_ascii_case("search"))
}

fn starts_investment_research_protocol(tool_call: &ToolCall) -> bool {
    tool_call.function.name.eq_ignore_ascii_case("data_fetch")
}

fn terminal_synthesis_prompt() -> String {
    format!(
        "【终局回答阶段】\n{}\n{}",
        "Agent 已通过显式完成信号确认：本轮合理的研究与工具尝试已经完成。",
        TERMINAL_SYNTHESIS_PROMPT
    )
}

enum ActiveBusinessStreamOutcome {
    Tools(ChatResponse),
    DirectFinal(ChatResponse),
    Empty,
}

fn consume_active_business_retry(failures: &mut u32) -> bool {
    if *failures >= ACTIVE_BUSINESS_FAILURE_RETRY_LIMIT {
        return false;
    }
    *failures = failures.saturating_add(1);
    true
}

fn failed_agent_response(
    tool_calls_made: Vec<ToolCallMade>,
    iterations: u32,
    error: impl Into<String>,
) -> AgentResponse {
    AgentResponse {
        content: String::new(),
        tool_calls_made,
        iterations,
        success: false,
        error: Some(error.into()),
    }
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
        let turn_message_start = context.messages.len();
        context.add_user_message(user_input);

        let business_tools: Vec<Value> = self.tools.get_tools_schema();
        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        let mut tool_call_counts: HashMap<String, u32> = HashMap::new();
        let mut total_tool_calls = 0u32;
        let mut iterations: u32 = 0;
        let mut investment_research_started = false;
        let mut research_evidence = ResearchEvidenceLedger::default();
        let mut active_business_failures = 0u32;

        self.dbg(&format!(
            "[Agent] start tools={:?}",
            self.tools.list_tool_names()
        ));

        loop {
            let finance_protocol_active =
                self.finish_research_terminal_synthesis && investment_research_started;

            if iterations >= self.max_iterations {
                // The iteration bound is a normal failed run, never implicit
                // finish authority. Only the Agent's eligible sole finish call
                // can enter the tool-free terminal completion.
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

            let active_business_round = finance_protocol_active;
            let finish_research_available =
                active_business_round && research_evidence.post_identity_attempts > 0;
            let mut round_tools = business_tools.clone();
            if finish_research_available {
                round_tools.push(finish_research_tool_schema());
            }
            let has_tools = !round_tools.is_empty();
            let tool_choice_mode = if active_business_round {
                ToolChoiceMode::Required
            } else {
                ToolChoiceMode::Auto
            };
            let messages = self.build_messages(
                context,
                active_business_round.then_some(if finish_research_available {
                    ACTIVE_RESEARCH_SYSTEM_INSTRUCTION
                } else {
                    POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION
                }),
            );
            let request_payload = serde_json::json!({
                "messages": messages.clone(),
                "tools": if has_tools { Some(round_tools.clone()) } else { None },
                "tool_choice_mode": format!("{tool_choice_mode:?}"),
            });
            let call_started = std::time::Instant::now();
            let mut stream_tool_choice = StreamToolChoiceTelemetry::new(tool_choice_mode);
            let mut active_business_outcome = active_business_round.then_some("tools");

            // 如果有工具，使用 chat_with_tools；否则使用 chat
            let result: ChatResponse = if has_tools {
                if active_business_round {
                    match tokio::time::timeout(
                        ACTIVE_BUSINESS_TIMEOUT,
                        self.chat_active_business_tools(
                            &messages,
                            &round_tools,
                            &mut stream_tool_choice,
                        ),
                    )
                    .await
                    {
                        Ok(Ok(ActiveBusinessStreamOutcome::Tools(response))) => {
                            active_business_failures = 0;
                            response
                        }
                        Ok(Ok(ActiveBusinessStreamOutcome::DirectFinal(response))) => {
                            if finish_research_available {
                                // A complete Stop + Done response after the
                                // structural evidence floor is the same Agent's
                                // natural final answer. In particular, a
                                // provider's Required -> Auto compatibility
                                // fallback must not turn that answer into a
                                // service-authored refusal or a second model run.
                                active_business_failures = 0;
                                active_business_outcome = Some("direct_final");
                                response
                            } else {
                                let error = "active business stream returned final content before post-identity evidence";
                                let retrying =
                                    consume_active_business_retry(&mut active_business_failures);
                                self.record_audit(
                                    context,
                                    "chat_with_tools",
                                    request_payload,
                                    None,
                                    Some(error.to_string()),
                                    call_started.elapsed().as_millis(),
                                    serde_json::json!({
                                        "iteration": iterations,
                                        "has_tools": true,
                                        "active_business_outcome": "premature_direct_final",
                                        "terminal_authorized": false,
                                        "retrying": retrying,
                                        "tool_choice_mode": "required",
                                        "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                        "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                        "tool_choice_fallback": stream_tool_choice.fallback,
                                    }),
                                    None,
                                );
                                if retrying {
                                    continue;
                                }
                                return failed_agent_response(tool_calls_made, iterations, error);
                            }
                        }
                        Ok(Ok(ActiveBusinessStreamOutcome::Empty)) => {
                            let error = "active business stream returned no tool call";
                            let retrying =
                                consume_active_business_retry(&mut active_business_failures);
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.to_string()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "active_business_outcome": "empty",
                                    "terminal_authorized": false,
                                    "retrying": retrying,
                                    "tool_choice_mode": "required",
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            );
                            if retrying {
                                continue;
                            }
                            return failed_agent_response(tool_calls_made, iterations, error);
                        }
                        Ok(Err(error)) => {
                            let error = error.to_string();
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.clone()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "active_business_outcome": "error",
                                    "terminal_authorized": false,
                                    "retrying": false,
                                    "tool_choice_mode": "required",
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            );
                            self.dbg(&format!(
                                "[Agent] active business stream failed without terminal authorization: {error}"
                            ));
                            return failed_agent_response(tool_calls_made, iterations, error);
                        }
                        Err(_) => {
                            let error = "active business stream timed out";
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.to_string()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "active_business_timeout": true,
                                    "active_business_outcome": "timeout",
                                    "terminal_authorized": false,
                                    "retrying": false,
                                    "tool_choice_mode": "required",
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
                                }),
                                None,
                            );
                            return failed_agent_response(tool_calls_made, iterations, error);
                        }
                    }
                } else {
                    match self
                        .chat_with_tools_streaming(
                            &messages,
                            &round_tools,
                            tool_choice_mode,
                            true,
                            &mut stream_tool_choice,
                        )
                        .await
                    {
                        Ok(response) => response,
                        Err(error) => {
                            self.record_audit(
                                context,
                                "chat_with_tools",
                                request_payload,
                                None,
                                Some(error.to_string()),
                                call_started.elapsed().as_millis(),
                                serde_json::json!({
                                    "iteration": iterations,
                                    "has_tools": true,
                                    "requested_tool_choice": tool_choice_mode_name(stream_tool_choice.requested),
                                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                                    "tool_choice_fallback": stream_tool_choice.fallback,
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
                serde_json::json!({
                    "iteration": iterations,
                    "has_tools": has_tools,
                    "active_business_outcome": active_business_outcome,
                    "finish_research_available": finish_research_available,
                    "identity_only_attempts": research_evidence.identity_only_attempts,
                    "post_identity_attempts": research_evidence.post_identity_attempts,
                    "requested_tool_choice": has_tools.then_some(tool_choice_mode_name(stream_tool_choice.requested)),
                    "effective_tool_choice": stream_tool_choice.effective.map(tool_choice_mode_name),
                    "tool_choice_fallback": stream_tool_choice.fallback,
                }),
                result.usage.clone(),
            );

            // 检查是否有工具调用
            if let Some(ref tcs) = result.tool_calls {
                let tcs: &Vec<hone_llm::ToolCall> = tcs;
                if !tcs.is_empty() {
                    self.dbg(&format!("[Agent] tool_calls n={}", tcs.len()));

                    let sole_finish_research = finish_research_available
                        && tcs.len() == 1
                        && tcs.first().is_some_and(is_valid_finish_research_call);
                    if sole_finish_research {
                        return self
                            .run_terminal_synthesis(
                                context,
                                tool_calls_made,
                                iterations,
                                turn_message_start,
                            )
                            .await;
                    }

                    // A mixed, duplicate, or not-yet-available finish signal
                    // never substitutes for research. Ignore it while keeping
                    // every real business call in the same Agent loop; the
                    // internal signal must not consume budget, reach the
                    // registry, persist in ToolCallMade, or notify observers.
                    let actionable_tool_calls = tcs
                        .iter()
                        .filter(|tool_call| !is_finish_research_call(tool_call))
                        .collect::<Vec<_>>();

                    // Every nonempty Interactive turn enters the open Agent
                    // discovery path, including non-finance questions that may
                    // use Web/file/skill tools. Activate the canonical finance
                    // protocol only at the structural DataFetch boundary that
                    // the investment prompt requires for every security turn;
                    // do not infer it from a closed question vocabulary.
                    investment_research_started |= actionable_tool_calls
                        .iter()
                        .any(|tool_call| starts_investment_research_protocol(tool_call));
                    if investment_research_started {
                        for tool_call in &actionable_tool_calls {
                            // Calls selected in the discovery completion have
                            // not seen the identity results yet. They remain in
                            // context as usable evidence, but only a subsequent
                            // Agent business round can satisfy the structural
                            // post-identity attempt gate.
                            research_evidence
                                .observe_business_call(tool_call, active_business_round);
                        }
                    }

                    if actionable_tool_calls.is_empty() {
                        self.dbg("[Agent] ignored malformed or unavailable finish signal");
                        if active_business_round {
                            continue;
                        }
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
                        let assistant_tool_content = if self.finish_research_terminal_synthesis
                            && investment_research_started
                        {
                            ""
                        } else {
                            &result.content
                        };
                        context.add_assistant_message_with_metadata(
                            assistant_tool_content,
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

            // Before finance research starts, preserve ordinary direct answers.
            // After the structural evidence floor, a complete Stop + Done body
            // is likewise the same Agent's natural final answer and is not sent
            // through another terminal generation or a service semantic gate.
            self.dbg("[Agent] done (no more tool_calls)");
            let metadata = if active_business_round {
                None
            } else {
                result.reasoning_content.as_ref().map(|reasoning| {
                    std::collections::HashMap::from([(
                        REASONING_CONTENT_METADATA_KEY.to_string(),
                        Value::String(reasoning.clone()),
                    )])
                })
            };
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
        seen_tool_names: Arc<Mutex<Vec<Vec<String>>>>,
        seen_tool_choice_modes: Arc<Mutex<Vec<ToolChoiceMode>>>,
        seen_messages: Arc<Mutex<Vec<Vec<Message>>>>,
        delivered_events: Arc<AtomicUsize>,
        stream_calls: Arc<AtomicUsize>,
        failed_stream_calls: Arc<Mutex<Vec<usize>>>,
        pending_stream_calls: Arc<Mutex<Vec<usize>>>,
        hang_after_first_event_stream_calls: Arc<Mutex<Vec<usize>>>,
    }

    impl StreamingMockLlmProvider {
        fn with_rounds(rounds: Vec<Vec<ChatStreamEvent>>) -> Self {
            Self {
                rounds: Arc::new(Mutex::new(rounds.into())),
                seen_tool_counts: Arc::new(Mutex::new(Vec::new())),
                seen_tool_names: Arc::new(Mutex::new(Vec::new())),
                seen_tool_choice_modes: Arc::new(Mutex::new(Vec::new())),
                seen_messages: Arc::new(Mutex::new(Vec::new())),
                delivered_events: Arc::new(AtomicUsize::new(0)),
                stream_calls: Arc::new(AtomicUsize::new(0)),
                failed_stream_calls: Arc::new(Mutex::new(Vec::new())),
                pending_stream_calls: Arc::new(Mutex::new(Vec::new())),
                hang_after_first_event_stream_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn failing_on_stream_calls(self, calls: &[usize]) -> Self {
            self.failed_stream_calls
                .lock()
                .expect("failed stream calls lock")
                .extend_from_slice(calls);
            self
        }

        fn pending_on_stream_calls(self, calls: &[usize]) -> Self {
            self.pending_stream_calls
                .lock()
                .expect("pending stream calls lock")
                .extend_from_slice(calls);
            self
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
            self.seen_tool_names
                .lock()
                .expect("stream tool names lock")
                .push(
                    tools
                        .iter()
                        .filter_map(|tool| {
                            tool.get("function")
                                .and_then(|function| function.get("name"))
                                .and_then(Value::as_str)
                                .map(ToString::to_string)
                        })
                        .collect(),
                );
            self.seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .push(tool_choice_mode);
            self.seen_messages
                .lock()
                .expect("stream messages lock")
                .push(messages.to_vec());
            let stream_call = self.stream_calls.fetch_add(1, Ordering::SeqCst) + 1;
            let mut events = self
                .rounds
                .lock()
                .expect("stream rounds lock")
                .pop_front()
                .expect("stream round");
            let should_fail = self
                .failed_stream_calls
                .lock()
                .expect("failed stream calls lock")
                .contains(&stream_call);
            let should_pending = self
                .pending_stream_calls
                .lock()
                .expect("pending stream calls lock")
                .contains(&stream_call);
            if should_pending {
                return Box::pin(stream::pending());
            }
            // Most tests describe only payload deltas. Mirror the provider
            // contract by adding the lifecycle envelope automatically. A
            // round that contains any lifecycle event is intentionally kept
            // raw so protocol-negative tests can model missing/mismatched
            // Finish or Done boundaries precisely.
            let explicit_lifecycle = events.iter().any(|event| {
                matches!(
                    event,
                    ChatStreamEvent::ToolChoiceMetadata { .. }
                        | ChatStreamEvent::Finish(_)
                        | ChatStreamEvent::Done
                )
            });
            if !explicit_lifecycle {
                let finish_reason = if events
                    .iter()
                    .any(|event| matches!(event, ChatStreamEvent::ToolCallDelta { .. }))
                {
                    ChatStreamFinishReason::ToolCalls
                } else {
                    ChatStreamFinishReason::Stop
                };
                events.insert(
                    0,
                    ChatStreamEvent::ToolChoiceMetadata {
                        requested: tool_choice_mode,
                        effective: tool_choice_mode,
                        fallback: false,
                    },
                );
                events.push(ChatStreamEvent::Finish(finish_reason));
                events.push(ChatStreamEvent::Done);
            }
            let hang_take = if matches!(
                events.first(),
                Some(ChatStreamEvent::ToolChoiceMetadata { .. })
            ) {
                2
            } else {
                1
            };
            let items: Vec<hone_core::HoneResult<ChatStreamEvent>> = if should_fail {
                vec![Err(hone_core::HoneError::Llm(format!(
                    "mock stream failure {stream_call}"
                )))]
            } else {
                events.into_iter().map(Ok).collect()
            };
            let delivered_events = self.delivered_events.clone();
            let should_hang_after_first = self
                .hang_after_first_event_stream_calls
                .lock()
                .expect("hang after first event calls lock")
                .contains(&stream_call);
            if should_hang_after_first {
                return Box::pin(
                    stream::iter(items.into_iter().take(hang_take))
                        .inspect(move |_| {
                            delivered_events.fetch_add(1, Ordering::SeqCst);
                        })
                        .chain(stream::pending()),
                );
            }
            Box::pin(stream::iter(items).inspect(move |_| {
                delivered_events.fetch_add(1, Ordering::SeqCst);
            }))
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

    struct CommittedPrefixStreamObserver {
        prefix: String,
        accumulated: Mutex<String>,
        events: Mutex<Vec<String>>,
    }

    impl CommittedPrefixStreamObserver {
        fn new(prefix: impl Into<String>) -> Self {
            Self {
                prefix: prefix.into(),
                accumulated: Mutex::new(String::new()),
                events: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl FunctionCallingStreamObserver for CommittedPrefixStreamObserver {
        async fn on_content_delta(&self, content: &str) {
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("delta:{content}"));
        }

        async fn on_final_content_delta(&self, content: &str) {
            self.accumulated
                .lock()
                .expect("accumulated stream content")
                .push_str(content);
            self.events
                .lock()
                .expect("stream events lock")
                .push(format!("final:{content}"));
        }

        fn committed_visible_prefix(&self) -> Option<String> {
            self.accumulated
                .lock()
                .expect("accumulated stream content")
                .starts_with(&self.prefix)
                .then(|| self.prefix.clone())
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

    struct WebSearchEvidenceTool;

    #[async_trait]
    impl Tool for WebSearchEvidenceTool {
        fn name(&self) -> &str {
            "web_search"
        }

        fn description(&self) -> &str {
            "relationship evidence"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            vec![]
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Ok(json!({ "evidence": "relationship" }))
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

    #[derive(Default)]
    struct RecordingAuditSink {
        operations: Mutex<Vec<String>>,
        records: Mutex<Vec<LlmAuditRecord>>,
    }

    impl LlmAuditSink for RecordingAuditSink {
        fn record(&self, record: LlmAuditRecord) -> hone_core::HoneResult<()> {
            self.operations
                .lock()
                .expect("audit operations lock")
                .push(record.operation.clone());
            self.records
                .lock()
                .expect("audit records lock")
                .push(record);
            Ok(())
        }
    }

    fn assert_explicit_terminal_messages(seen_messages: &Arc<Mutex<Vec<Vec<Message>>>>) {
        let terminal_messages = seen_messages
            .lock()
            .expect("stream messages lock")
            .last()
            .cloned()
            .expect("terminal messages");
        let system = terminal_messages
            .first()
            .and_then(|message| message.content.as_deref())
            .expect("terminal system instruction");
        let prompt = terminal_messages
            .last()
            .and_then(|message| message.content.as_deref())
            .expect("terminal user prompt");

        assert!(
            terminal_messages
                .iter()
                .all(|message| message.reasoning_content.is_none()),
            "hidden reasoning must be stripped at the terminal evidence boundary"
        );
        assert_eq!(prompt, terminal_synthesis_prompt());
        assert!(prompt.contains("Agent 已通过显式完成信号确认"));
        assert!(prompt.contains("`reasoning_content`、隐藏思考、未采用草稿"));
        assert!(system.contains(FINISH_RESEARCH_SYSTEM_INSTRUCTION));
        assert!(!prompt.contains("上一内部步骤未产出可用的新事实证据"));
    }

    #[test]
    fn terminal_prompt_is_authorized_only_by_explicit_finish() {
        let explicit = terminal_synthesis_prompt();
        assert!(explicit.contains("Agent 已通过显式完成信号确认"));
        assert!(!explicit.contains("上一内部步骤未产出可用的新事实证据"));
        assert!(explicit.contains("此前已成功返回的业务工具结果"));
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
            seen_tool_names: Arc::new(Mutex::new(Vec::new())),
            seen_tool_choice_modes: Arc::new(Mutex::new(Vec::new())),
            seen_messages: Arc::new(Mutex::new(Vec::new())),
            delivered_events: Arc::new(AtomicUsize::new(0)),
            stream_calls: Arc::new(AtomicUsize::new(0)),
            failed_stream_calls: Arc::new(Mutex::new(Vec::new())),
            pending_stream_calls: Arc::new(Mutex::new(Vec::new())),
            hang_after_first_event_stream_calls: Arc::new(Mutex::new(Vec::new())),
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
            vec![
                ChatStreamEvent::ReasoningDelta(
                    "hidden draft must not become terminal evidence".to_string(),
                ),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_data_fetch".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV,NVIDIA"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
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
        registry.register(Box::new(WebSearchEvidenceTool));
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
        context.add_user_message("旧问题：NBIS 估值");
        context.add_assistant_message(
            "旧草稿：NBIS 价格 15 USD；不要把它当成本轮事实。",
            Some(vec![
                serde_json::to_value(ToolCall {
                    id: "tc_stale_nbis".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"ticker":"NBIS"}"#.to_string(),
                    },
                })
                .expect("stale tool call"),
            ]),
        );
        context.add_tool_result(
            "tc_stale_nbis",
            "data_fetch",
            r#"{"symbol":"NBIS","price":15,"stale":true}"#,
        );

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "最终答案");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 5);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].arguments["data_type"], "quote");
        assert_eq!(
            response.tool_calls_made[2].arguments["data_type"],
            "profile"
        );
        assert_eq!(
            response.tool_calls_made[3].arguments["data_type"],
            "profile"
        );
        assert_eq!(response.tool_calls_made[4].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 0],
            "search-only evidence must force one post-identity business round before the same Agent can select finish and enter the empty-tools terminal"
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ],
            "the first turn is open, post-identity research and Agent-owned finish are required-tool rounds, and terminal synthesis has no tools"
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
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:data_fetch",
                "done:data_fetch:true",
                "start:web_search",
                "done:web_search:true",
            ]
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
        let terminal_assistant = context.messages.last().expect("terminal assistant message");
        assert_eq!(terminal_assistant.role, "assistant");
        assert_eq!(terminal_assistant.content.as_deref(), Some("最终答案"));
        assert!(
            terminal_assistant.metadata.is_none(),
            "terminal reasoning must not persist into cross-turn context"
        );
        assert_explicit_terminal_messages(&seen_messages);
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let terminal_messages = seen_messages.last().expect("terminal messages");
        assert!(terminal_messages.iter().any(|message| {
            message
                .content
                .as_deref()
                .is_some_and(|content| content.contains("relationship"))
        }));
        assert!(
            terminal_messages.iter().all(|message| {
                message.content.as_deref().is_none_or(|content| {
                    !content.contains("NBIS")
                        && !content.contains("15 USD")
                        && !content.contains("\"price\":15")
                })
            }),
            "stale prior-turn ticker/price evidence reached terminal synthesis"
        );
    }

    #[tokio::test]
    async fn mixed_finish_keeps_business_tools_in_the_same_agent_loop() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_data_fetch".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV,NVIDIA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_hallucinated_finish_with_data".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_relationship_evidence".to_string()),
                name: Some("echo_tool".to_string()),
                arguments: r#"{"text":"relationship evidence"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("不应发布的业务轮草稿".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_echo".to_string()),
                    name: Some("echo_tool".to_string()),
                    arguments: r#"{"text":"relationship evidence"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_hallucinated_finish_with_echo".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta("最终研究答案".to_string())],
        ]);
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let stream_observer = Arc::new(RecordingStreamObserver::default());
        let tool_observer = Arc::new(MockToolObserver::default());
        let audit = Arc::new(RecordingAuditSink::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(EchoTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            "system".to_string(),
            5,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_tool_call_budget(Some(2), HashMap::new())
        .with_stream_observer(Some(stream_observer.clone()))
        .with_tool_observer(Some(tool_observer.clone()));
        let mut context = AgentContext::new("same-agent-business-finish".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "最终研究答案");
        assert_eq!(response.iterations, 5);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].name, "echo_tool");
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ]
        );
        let tool_names = seen_tool_names.lock().expect("stream tool names lock");
        assert!(
            tool_names[0]
                .iter()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(
            tool_names[1]
                .iter()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(tool_names[2].iter().any(|name| name == "data_fetch"));
        assert!(tool_names[2].iter().any(|name| name == "echo_tool"));
        assert!(
            tool_names[2]
                .iter()
                .any(|name| name == FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(
            tool_names[3]
                .iter()
                .any(|name| name == FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(tool_names[4].is_empty());
        drop(tool_names);
        assert_eq!(
            stream_observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:最终研究答案"],
            "active business drafts and internal finish signals must remain invisible"
        );
        assert_eq!(
            tool_observer
                .events
                .lock()
                .expect("tool observer lock")
                .as_slice(),
            [
                "start:data_fetch",
                "done:data_fetch:true",
                "start:echo_tool",
                "done:echo_tool:true",
            ],
            "finish signals and budget-rejected mixed calls must not enter the business tool trace"
        );
        assert_eq!(
            audit
                .operations
                .lock()
                .expect("audit operations lock")
                .as_slice(),
            [
                "chat_with_tools",
                "chat_with_tools",
                "chat_with_tools",
                "chat_with_tools",
                "chat_terminal_without_tools",
            ],
            "business calls and Agent-owned finish decisions stay in one audited loop"
        );
        assert!(context.messages.iter().all(|message| {
            message.content.as_deref() != Some("不应发布的业务轮草稿")
                && message.tool_calls.as_ref().is_none_or(|tool_calls| {
                    tool_calls.iter().all(|tool_call| {
                        let name = tool_call
                            .get("function")
                            .and_then(|function| function.get("name"))
                            .and_then(Value::as_str);
                        name != Some(FINISH_RESEARCH_TOOL_NAME)
                    })
                })
        }));
    }

    #[tokio::test]
    async fn premature_direct_final_retries_once_then_fails_without_terminal_or_visible_draft() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "provider bypass draft".to_string(),
            )],
            vec![ChatStreamEvent::ContentDelta(
                "second provider bypass draft".to_string(),
            )],
        ]);
        let delivered_events = llm.delivered_events.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-content-bypass".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(response.error.as_deref().is_some_and(|error| {
            error.contains("final content before post-identity evidence")
        }));
        assert_eq!(
            delivered_events.load(Ordering::SeqCst),
            12,
            "both complete active content streams must be consumed through Finish + Done"
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 1],
            "a failed active stream must never trigger an empty-tools terminal call"
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert!(context.messages.iter().all(|message| {
            !matches!(
                message.content.as_deref(),
                Some("provider bypass draft" | "second provider bypass draft")
            )
        }));
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        let records = audit.records.lock().expect("audit records lock");
        let premature_finals = records
            .iter()
            .filter(|record| {
                record.metadata["active_business_outcome"].as_str()
                    == Some("premature_direct_final")
            })
            .collect::<Vec<_>>();
        assert_eq!(premature_finals.len(), 2);
        assert_eq!(
            premature_finals[0].metadata["retrying"].as_bool(),
            Some(true)
        );
        assert_eq!(
            premature_finals[1].metadata["retrying"].as_bool(),
            Some(false)
        );
        assert!(
            premature_finals
                .iter()
                .all(|record| { record.metadata["terminal_authorized"].as_bool() == Some(false) })
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn eligible_direct_final_is_preserved_without_terminal_or_second_generation() {
        let answer = "数据时间：北京时间 2026-07-18 21:08；行情口径：本轮网页关系证据\n\nCoreWeave 与 NVIDIA 的关系以本轮网页来源为准。";
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV,NVIDIA"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Auto,
                    fallback: true,
                },
                ChatStreamEvent::ReasoningDelta("未采用的隐藏关系推演不能进入后续会话".to_string()),
                ChatStreamEvent::ContentDelta(answer.to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("eligible-direct-final".to_string());

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, answer);
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 5);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3],
            "a natural final must not trigger an empty-tools terminal or second generation"
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty(),
            "the complete direct final stays deferred until AgentSession publishes it once"
        );
        assert_eq!(
            context
                .messages
                .last()
                .and_then(|message| message.content.as_deref()),
            Some(answer)
        );
        assert!(
            context
                .messages
                .last()
                .and_then(|message| message.metadata.as_ref())
                .is_none(),
            "finance direct-final reasoning must not persist into a later turn"
        );
        let records = audit.records.lock().expect("audit records lock");
        let direct_final = records.last().expect("direct final audit");
        assert!(direct_final.success);
        assert_eq!(
            direct_final.metadata["active_business_outcome"].as_str(),
            Some("direct_final")
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn relationship_search_does_not_offer_finish_until_post_identity_evidence() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch_1".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV,NVIDIA"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("discarded unavailable-finish preamble".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_unavailable_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_quote".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV,NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 1,
                    id: Some("tc_crwv_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 2,
                    id: Some("tc_nvda_profile".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 3,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "continue preamble terminal".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("relationship-stage-gate".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "continue preamble terminal");
        assert_eq!(response.iterations, 5);
        assert_eq!(response.tool_calls_made.len(), 5);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].arguments["data_type"], "quote");
        assert_eq!(
            response.tool_calls_made[2].arguments["data_type"],
            "profile"
        );
        assert_eq!(
            response.tool_calls_made[3].arguments["data_type"],
            "profile"
        );
        assert_eq!(response.tool_calls_made[4].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 2, 3, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ]
        );
        let tool_names = seen_tool_names.lock().expect("stream tool names lock");
        assert!(
            tool_names[..3]
                .iter()
                .flatten()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(
            tool_names[3]
                .iter()
                .any(|name| name == FINISH_RESEARCH_TOOL_NAME)
        );
        assert!(tool_names[4].is_empty());
        drop(tool_names);
        assert_explicit_terminal_messages(&seen_messages);
        let terminal_transcript = serde_json::to_string(
            seen_messages
                .lock()
                .expect("stream messages lock")
                .last()
                .expect("terminal transcript"),
        )
        .expect("serialize terminal transcript");
        for required in [
            r#"\"data_type\":\"quote\""#,
            r#"\"data_type\":\"profile\""#,
            "CoreWeave NVIDIA relationship filing",
        ] {
            assert!(
                terminal_transcript.contains(required),
                "missing {required}: {terminal_transcript}"
            );
        }
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:continue preamble terminal"]
        );
        assert!(context.messages.iter().all(|message| {
            message.content.as_deref() != Some("discarded unavailable-finish preamble")
        }));
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
    async fn sole_finish_preamble_is_hidden_before_terminal_synthesis() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("discarded finish preamble".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![ChatStreamEvent::ContentDelta(
                "finish preamble terminal".to_string(),
            )],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("finish-visible-preamble".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "finish preamble terminal");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_explicit_terminal_messages(&seen_messages);
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:finish preamble terminal"]
        );
        assert!(
            context
                .messages
                .iter()
                .all(|message| { message.content.as_deref() != Some("discarded finish preamble") })
        );
    }

    #[tokio::test]
    async fn malformed_finish_is_ignored_until_a_later_valid_sole_finish() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_malformed_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{bad".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_valid_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "malformed finish 后的唯一终稿".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("malformed-finish".to_string());

        let response = agent.run("CRWV research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "malformed finish 后的唯一终稿");
        assert_eq!(response.iterations, 5);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 2, 0],
            "the malformed signal must not enter terminal synthesis"
        );
        assert_explicit_terminal_messages(&seen_messages);
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:malformed finish 后的唯一终稿"]
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
    async fn fragmented_hidden_thinking_stays_internal_during_business_evidence_round() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch_1".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ContentDelta("<thi".to_string()),
                ChatStreamEvent::ContentDelta("nk>private business thought</think>".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_web_relationship".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: r#"{"query":"relationship evidence"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "隐藏思考后的终稿".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("hidden-business-thinking".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "隐藏思考后的终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 0],
            "fragmented hidden thinking must not replace the business evidence call or the later sole finish"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:隐藏思考后的终稿"]
        );
        assert!(context.messages.iter().all(|message| {
            message
                .content
                .as_deref()
                .is_none_or(|content| !content.contains("private business thought"))
        }));
    }

    #[tokio::test]
    async fn active_timeout_fails_immediately_without_terminal_or_visible_draft() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![],
        ])
        .pending_on_stream_calls(&[2]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-timeout".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            response.error.as_deref(),
            Some("active business stream timed out")
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 2);
        let records = audit.records.lock().expect("audit records lock");
        let timeout = records.last().expect("timeout audit");
        assert_eq!(timeout.metadata["active_business_outcome"], "timeout");
        assert_eq!(timeout.metadata["retrying"].as_bool(), Some(false));
        assert_eq!(
            timeout.metadata["terminal_authorized"].as_bool(),
            Some(false)
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn successful_tools_reset_the_consecutive_active_failure_counter() {
        let first_business_empty = vec![ChatStreamEvent::ReasoningDelta(
            "first hidden-only business thought".to_string(),
        )];
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            first_business_empty,
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_relationship".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"relationship evidence"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ReasoningDelta(
                "second hidden-only business thought".to_string(),
            )],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta("唯一可见终稿".to_string())],
        ]);
        let delivered_events = llm.delivered_events.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            5,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-failure-counter-reset".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "唯一可见终稿");
        assert_eq!(response.iterations, 6);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(response.tool_calls_made[1].name, "web_search");
        assert_eq!(
            delivered_events.load(Ordering::SeqCst),
            24,
            "all six completed streams must be consumed through their lifecycle boundaries"
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 2, 3, 3, 0]
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:唯一可见终稿"]
        );
        assert!(context.messages.iter().all(|message| {
            message
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(REASONING_CONTENT_METADATA_KEY))
                .and_then(Value::as_str)
                .is_none_or(|reasoning| !reasoning.contains("hidden-only business thought"))
        }));
        assert_explicit_terminal_messages(&seen_messages);
        let records = audit.records.lock().expect("audit records lock");
        let empties = records
            .iter()
            .filter(|record| record.metadata["active_business_outcome"].as_str() == Some("empty"))
            .collect::<Vec<_>>();
        assert_eq!(empties.len(), 2);
        assert!(
            empties
                .iter()
                .all(|record| record.metadata["retrying"].as_bool() == Some(true))
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.operation == "chat_terminal_without_tools")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn premature_fallback_direct_final_never_authorizes_terminal_synthesis() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Auto,
                    fallback: true,
                },
                ChatStreamEvent::ContentDelta("finite active draft".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
                ChatStreamEvent::Done,
            ],
            vec![ChatStreamEvent::ContentDelta(
                "基于已有数据正常回答".to_string(),
            )],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("finite-active-content-bypass".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert!(
            context
                .messages
                .iter()
                .all(|message| { message.content.as_deref() != Some("finite active draft") })
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);

        let records = audit.records.lock().expect("audit records lock");
        let premature_finals = records
            .iter()
            .filter(|record| {
                record.metadata["active_business_outcome"].as_str()
                    == Some("premature_direct_final")
            })
            .collect::<Vec<_>>();
        assert_eq!(premature_finals.len(), 2);
        let premature_final = premature_finals[0];
        assert!(!premature_final.success);
        assert_eq!(
            premature_final.error.as_deref(),
            Some("active business stream returned final content before post-identity evidence")
        );
        assert_eq!(
            premature_final.metadata["terminal_authorized"].as_bool(),
            Some(false)
        );
        assert_eq!(premature_final.metadata["retrying"].as_bool(), Some(true));
        assert_eq!(
            premature_finals[1].metadata["retrying"].as_bool(),
            Some(false)
        );
        assert_eq!(
            premature_final.metadata["requested_tool_choice"].as_str(),
            Some("required")
        );
        assert_eq!(
            premature_final.metadata["effective_tool_choice"].as_str(),
            Some("auto")
        );
        assert_eq!(
            premature_final.metadata["tool_choice_fallback"].as_bool(),
            Some(true)
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn active_empty_retries_once_then_fails_without_terminal() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ReasoningDelta(
                "hidden-only active thought".to_string(),
            )],
            vec![ChatStreamEvent::ReasoningDelta(
                "second hidden-only active thought".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-empty".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(
            response.error.as_deref(),
            Some("active business stream returned no tool call")
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 1]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        assert!(context.messages.iter().all(|message| {
            message
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(REASONING_CONTENT_METADATA_KEY))
                .and_then(Value::as_str)
                .is_none_or(|reasoning| !reasoning.contains("hidden-only active thought"))
        }));
        let records = audit.records.lock().expect("audit records lock");
        let empties = records
            .iter()
            .filter(|record| record.metadata["active_business_outcome"].as_str() == Some("empty"))
            .collect::<Vec<_>>();
        assert_eq!(empties.len(), 2);
        assert_eq!(empties[0].metadata["retrying"].as_bool(), Some(true));
        assert_eq!(empties[1].metadata["retrying"].as_bool(), Some(false));
        assert!(
            empties
                .iter()
                .all(|record| { record.metadata["terminal_authorized"].as_bool() == Some(false) })
        );
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn active_provider_error_fails_immediately_without_terminal() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![],
        ])
        .failing_on_stream_calls(&[2]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-business-provider-error".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("mock stream failure 2"))
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1]
        );
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 2);
        let records = audit.records.lock().expect("audit records lock");
        let error = records.last().expect("active error audit");
        assert_eq!(error.metadata["active_business_outcome"], "error");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(false));
        assert_eq!(error.metadata["terminal_authorized"].as_bool(), Some(false));
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn data_fetch_starts_same_agent_research_before_finish_is_available() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ContentDelta("首轮隐藏工具草稿".to_string()),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_data_fetch".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta("关系分析终稿".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("data-fetch-starts-agent-research".to_string());

        let response = agent.run("relationship research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "关系分析终稿");
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(response.tool_calls_made[0].name, "data_fetch");
        assert_eq!(response.tool_calls_made[1].name, "data_fetch");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 0],
            "the same Agent must complete a post-identity business round before finish becomes available"
        );
        assert!(
            context
                .messages
                .iter()
                .all(|message| { message.content.as_deref() != Some("首轮隐藏工具草稿") })
        );
    }

    #[tokio::test]
    async fn terminal_scrubs_tool_round_drafts_that_precede_data_fetch_activation() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![
                ChatStreamEvent::ContentDelta(
                    "未经证据支持的早期关系草稿：CRWV 是 NVIDIA 子公司。".to_string(),
                ),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_web_search".to_string()),
                    name: Some("web_search".to_string()),
                    arguments: "{}".to_string(),
                },
            ],
            vec![
                ChatStreamEvent::ContentDelta(
                    "未经采用的行情草稿：CRWV 市值已经核验。".to_string(),
                ),
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_data_fetch".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"text":"CRWV current quote"}"#.to_string(),
                },
            ],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_web".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"CoreWeave NVIDIA relationship filing"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "基于两项工具证据的终稿".to_string(),
            )],
        ]);
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(WebSearchEvidenceTool));
        registry.register(Box::new(FinanceEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("pre-data-fetch-draft-scrub".to_string());

        let response = agent.run("crwv和英伟达有什么关系", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "基于两项工具证据的终稿");
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_explicit_terminal_messages(&seen_messages);
        let seen_messages = seen_messages.lock().expect("stream messages lock");
        let terminal_messages = seen_messages.last().expect("terminal messages");
        assert!(terminal_messages.iter().any(|message| {
            message
                .content
                .as_deref()
                .is_some_and(|content| content.contains("relationship"))
        }));
        assert!(terminal_messages.iter().any(|message| {
            message
                .content
                .as_deref()
                .is_some_and(|content| content.contains("CRWV current quote"))
        }));
        assert!(terminal_messages.iter().all(|message| {
            message.content.as_deref().is_none_or(|content| {
                !content.contains("CRWV 是 NVIDIA 子公司") && !content.contains("CRWV 市值已经核验")
            })
        }));
    }

    #[tokio::test]
    async fn non_finance_web_search_does_not_activate_the_investment_terminal_protocol() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_search".to_string()),
                name: Some("web_search".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta(
                "这是普通网页检索后的直接回答。".to_string(),
            )],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_names = llm.seen_tool_names.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("non-finance-web-search".to_string());

        let response = agent.run("查一下普通网页资料", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "这是普通网页检索后的直接回答。");
        assert_eq!(response.iterations, 2);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.tool_calls_made[0].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1],
            "non-finance tools must keep the ordinary Agent loop without an internal finish signal or terminal completion"
        );
        assert!(
            seen_tool_names
                .lock()
                .expect("stream tool names lock")
                .iter()
                .flatten()
                .all(|name| name != FINISH_RESEARCH_TOOL_NAME)
        );
    }

    #[tokio::test]
    async fn iteration_limit_fails_without_terminal_call() {
        let llm =
            StreamingMockLlmProvider::with_rounds(vec![vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }]]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            1,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("iteration-limit-failure".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 1);
        assert_eq!(response.tool_calls_made.len(), 1);
        assert_eq!(response.error.as_deref(), Some("max_iterations_exceeded:1"));
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [ToolChoiceMode::Auto]
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 1);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        assert!(
            audit
                .operations
                .lock()
                .expect("audit operations lock")
                .iter()
                .all(|operation| operation != "chat_terminal_without_tools")
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
    async fn direct_stream_requires_stop_and_done() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![
            ChatStreamEvent::ToolChoiceMetadata {
                requested: ToolChoiceMode::Auto,
                effective: ToolChoiceMode::Auto,
                fallback: false,
            },
            ChatStreamEvent::ContentDelta("partial direct answer".to_string()),
            ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
        ]]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("direct-missing-done".to_string());

        let response = agent.run("answer", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("ended before Done"))
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["delta:partial direct answer", "reset"]
        );
    }

    #[tokio::test]
    async fn tool_stream_requires_tool_calls_finish_reason() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![vec![
            ChatStreamEvent::ToolChoiceMetadata {
                requested: ToolChoiceMode::Auto,
                effective: ToolChoiceMode::Auto,
                fallback: false,
            },
            ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_wrong_finish".to_string()),
                name: Some("echo_tool".to_string()),
                arguments: r#"{"text":"never execute"}"#.to_string(),
            },
            ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ChatStreamEvent::Done,
        ]]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 3, None);
        let mut context = AgentContext::new("tool-wrong-finish".to_string());

        let response = agent.run("tool", &mut context).await;

        assert!(!response.success);
        assert!(response.tool_calls_made.is_empty());
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("expected ToolCalls, got Stop"))
        );
    }

    #[tokio::test]
    async fn active_finish_stream_missing_done_fails_immediately_without_terminal() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Required,
                    fallback: false,
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_incomplete_finish".to_string()),
                    name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                    arguments: "{}".to_string(),
                },
                ChatStreamEvent::Finish(ChatStreamFinishReason::ToolCalls),
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-finish-missing-done".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("ended before Done"))
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2]
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        let records = audit.records.lock().expect("audit records lock");
        let error = records.last().expect("active finish error audit");
        assert_eq!(error.metadata["active_business_outcome"], "error");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(false));
        assert_eq!(error.metadata["terminal_authorized"].as_bool(), Some(false));
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn active_stream_missing_done_fails_immediately_without_terminal() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Required,
                    effective: ToolChoiceMode::Required,
                    fallback: false,
                },
                ChatStreamEvent::ToolCallDelta {
                    index: 0,
                    id: Some("tc_incomplete_data".to_string()),
                    name: Some("data_fetch".to_string()),
                    arguments: r#"{"text":"incomplete"}"#.to_string(),
                },
                ChatStreamEvent::Finish(ChatStreamFinishReason::ToolCalls),
            ],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(RecordingStreamObserver::default());
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("active-missing-done".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(response.content.is_empty());
        assert_eq!(response.iterations, 3);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("ended before Done"))
        );
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2]
        );
        assert_eq!(seen_messages.lock().expect("stream messages lock").len(), 3);
        assert!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .is_empty()
        );
        let records = audit.records.lock().expect("audit records lock");
        let error = records.last().expect("active business error audit");
        assert_eq!(error.metadata["active_business_outcome"], "error");
        assert_eq!(error.metadata["retrying"].as_bool(), Some(false));
        assert_eq!(error.metadata["terminal_authorized"].as_bool(), Some(false));
        assert!(
            records
                .iter()
                .all(|record| record.operation != "chat_terminal_without_tools")
        );
    }

    #[tokio::test]
    async fn terminal_stream_requires_stop_and_done() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta("incomplete terminal".to_string()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ],
        ]);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let observer = Arc::new(RecordingStreamObserver::default());
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 4, None)
                .with_finish_research_terminal_synthesis(true)
                .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("terminal-missing-done".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains("terminal synthesis stream ended before Done"))
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            ["final:incomplete terminal", "reset"]
        );
    }

    #[tokio::test]
    async fn committed_terminal_prefix_recovers_once_without_restreaming_or_rerunning_tools() {
        let prefix = concat!(
            "数据时间：北京时间 2026-07-18 21:05；行情口径：",
            "报价源最新可得、非逐笔\n"
        );
        let incomplete = format!("{prefix}未完成的正文");
        let recovered = format!("{prefix}\n## 结论\n基于本轮工具证据完成回答。");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta(incomplete.clone()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ],
            vec![ChatStreamEvent::ContentDelta(format!(
                "<think>recovery reasoning is not visible evidence</think>{recovered}"
            ))],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_messages = llm.seen_messages.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(CommittedPrefixStreamObserver::new(prefix));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("terminal-recovery-success".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, recovered);
        assert_eq!(response.iterations, 4);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(stream_calls.load(Ordering::SeqCst), 5);
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [1, 1, 2, 0, 0],
            "recovery must stay in the same terminal phase with tools disabled"
        );
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            [format!("final:{incomplete}")],
            "the recovery response must remain buffered and must not reset the committed prefix"
        );
        let messages = seen_messages.lock().expect("stream messages lock");
        assert_eq!(messages.len(), 5);
        assert!(
            messages[4]
                .last()
                .and_then(|message| message.content.as_deref())
                .is_some_and(|prompt| {
                    prompt.contains("【终稿传输恢复】")
                        && prompt.contains("前缀后必须继续输出非空正文")
                })
        );
        assert!(
            messages[4]
                .iter()
                .all(|message| message.reasoning_content.is_none())
        );
        drop(messages);

        let records = audit.records.lock().expect("audit records lock");
        let initial = records
            .iter()
            .find(|record| record.operation == "chat_terminal_without_tools")
            .expect("initial terminal audit");
        assert!(!initial.success);
        assert_eq!(
            initial.metadata["terminal_recovery_eligible"],
            Value::Bool(true)
        );
        let recovery = records
            .iter()
            .find(|record| record.operation == "chat_terminal_recovery_without_tools")
            .expect("terminal recovery audit");
        assert!(recovery.success, "{:?}", recovery.error);
        assert_eq!(recovery.metadata["recovery_attempt"], 1);
        assert_eq!(recovery.metadata["has_tools"], Value::Bool(false));
        assert_eq!(
            recovery.metadata["effective_tool_choice"],
            Value::String("auto".to_string())
        );
        assert_eq!(
            context
                .messages
                .last()
                .and_then(|message| message.content.as_deref()),
            Some(response.content.as_str())
        );
        assert!(
            context
                .messages
                .last()
                .expect("terminal message")
                .metadata
                .is_none()
        );
    }

    #[tokio::test]
    async fn committed_terminal_prefix_recovery_mismatch_fails_after_exactly_one_attempt() {
        let prefix = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";
        let incomplete = format!("{prefix}未完成的正文");
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![
                ChatStreamEvent::ToolChoiceMetadata {
                    requested: ToolChoiceMode::Auto,
                    effective: ToolChoiceMode::Auto,
                    fallback: false,
                },
                ChatStreamEvent::ContentDelta(incomplete.clone()),
                ChatStreamEvent::Finish(ChatStreamFinishReason::Stop),
            ],
            vec![ChatStreamEvent::ContentDelta(
                "数据时间：北京时间 2026-07-18 21:06；行情口径：不同前缀\n正文".to_string(),
            )],
        ]);
        let stream_calls = llm.stream_calls.clone();
        let audit = Arc::new(RecordingAuditSink::default());
        let observer = Arc::new(CommittedPrefixStreamObserver::new(prefix));
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        let agent = FunctionCallingAgent::new(
            Arc::new(llm),
            Arc::new(registry),
            String::new(),
            4,
            Some(audit.clone()),
        )
        .with_finish_research_terminal_synthesis(true)
        .with_stream_observer(Some(observer.clone()));
        let mut context = AgentContext::new("terminal-recovery-mismatch".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(!response.success);
        assert!(
            response
                .error
                .as_deref()
                .is_some_and(|error| error.contains(
                    "terminal recovery content does not start with the committed visible prefix"
                )),
            "{:?}",
            response.error
        );
        assert_eq!(stream_calls.load(Ordering::SeqCst), 5);
        assert_eq!(response.tool_calls_made.len(), 2);
        assert_eq!(
            observer
                .events
                .lock()
                .expect("stream events lock")
                .as_slice(),
            [format!("final:{incomplete}")]
        );
        let records = audit.records.lock().expect("audit records lock");
        let recovery_records = records
            .iter()
            .filter(|record| record.operation == "chat_terminal_recovery_without_tools")
            .collect::<Vec<_>>();
        assert_eq!(recovery_records.len(), 1);
        assert!(!recovery_records[0].success);
    }

    #[test]
    fn terminal_content_rejects_header_only_and_duplicate_committed_prefix() {
        let prefix = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";
        let header_only = validate_terminal_recovery_content(prefix, prefix)
            .expect_err("a canonical header without a body is incomplete");
        assert!(header_only.to_string().contains("contains no body"));

        let duplicated = format!("{prefix}\n{prefix}正文");
        let duplicate_error = validate_terminal_recovery_content(&duplicated, prefix)
            .expect_err("replaying the committed header would duplicate visible output");
        assert!(
            duplicate_error
                .to_string()
                .contains("repeats the committed visible prefix")
        );
    }

    #[test]
    fn non_success_stream_finish_reasons_are_errors() {
        for reason in [
            ChatStreamFinishReason::Length,
            ChatStreamFinishReason::ContentFilter,
            ChatStreamFinishReason::Error,
            ChatStreamFinishReason::Other("provider_specific".to_string()),
        ] {
            let mut finish = None;
            assert!(
                observe_stream_finish(&mut finish, reason).is_err(),
                "non-success finish reason must fail"
            );
            assert!(finish.is_none());
        }
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
                id: Some("tc_quote_failed".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
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
            [1, 1, 2, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ]
        );
    }

    #[tokio::test]
    async fn duplicate_finish_calls_are_ignored_until_a_later_sole_finish() {
        let llm = StreamingMockLlmProvider::with_rounds(vec![
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_data_fetch".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_post_identity_quote".to_string()),
                name: Some("data_fetch".to_string()),
                arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
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
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_web_after_duplicate".to_string()),
                name: Some("web_search".to_string()),
                arguments: r#"{"query":"relationship evidence"}"#.to_string(),
            }],
            vec![ChatStreamEvent::ToolCallDelta {
                index: 0,
                id: Some("tc_sole_finish".to_string()),
                name: Some(FINISH_RESEARCH_TOOL_NAME.to_string()),
                arguments: "{}".to_string(),
            }],
            vec![ChatStreamEvent::ContentDelta("唯一终稿".to_string())],
        ]);
        let seen_tool_counts = llm.seen_tool_counts.clone();
        let seen_tool_choice_modes = llm.seen_tool_choice_modes.clone();
        let seen_messages = llm.seen_messages.clone();
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FinanceEvidenceTool));
        registry.register(Box::new(WebSearchEvidenceTool));
        let agent =
            FunctionCallingAgent::new(Arc::new(llm), Arc::new(registry), String::new(), 5, None)
                .with_finish_research_terminal_synthesis(true);
        let mut context = AgentContext::new("duplicate-finish".to_string());

        let response = agent.run("research", &mut context).await;

        assert!(response.success, "{:?}", response.error);
        assert_eq!(response.content, "唯一终稿");
        assert_eq!(response.iterations, 6);
        assert_eq!(response.tool_calls_made.len(), 3);
        assert_eq!(response.tool_calls_made[2].name, "web_search");
        assert_eq!(
            seen_tool_counts
                .lock()
                .expect("stream tool counts lock")
                .as_slice(),
            [2, 2, 3, 3, 3, 0]
        );
        assert_eq!(
            seen_tool_choice_modes
                .lock()
                .expect("stream tool choice modes lock")
                .as_slice(),
            [
                ToolChoiceMode::Auto,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Required,
                ToolChoiceMode::Auto,
            ]
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
        assert_explicit_terminal_messages(&seen_messages);
    }

    #[tokio::test]
    async fn hallucinated_finish_is_ignored_when_terminal_policy_is_disabled() {
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
            "a hallucinated internal finish signal must be ignored when the terminal policy is disabled"
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
