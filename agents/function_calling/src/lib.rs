//! Hone Agent — Function Calling Agent 核心
//!
//! 基于 Function Calling 模式的 Agent 实现，
//! 支持多轮工具调用循环和流式输出。

use async_trait::async_trait;
use hone_core::agent::{Agent, AgentContext, AgentResponse, ToolCallMade};
use hone_core::{LlmAuditRecord, LlmAuditSink, ToolExecutionObserver};
use hone_llm::{ChatResponse, LlmProvider, Message};
use hone_tools::ToolRegistry;
use serde_json::Value;
use std::sync::Arc;

/// Function Calling Agent
pub struct FunctionCallingAgent {
    pub llm: Arc<dyn LlmProvider>,
    pub tools: Arc<ToolRegistry>,
    pub system_prompt: String,
    pub max_iterations: u32,
    pub debug_log: bool,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub tool_observer: Option<Arc<dyn ToolExecutionObserver>>,
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
        }
    }

    pub fn with_tool_observer(mut self, observer: Option<Arc<dyn ToolExecutionObserver>>) -> Self {
        self.tool_observer = observer;
        self
    }

    fn dbg(&self, msg: &str) {
        if self.debug_log {
            tracing::debug!("{msg}");
        }
    }

    /// 构建完整消息列表（system prompt + context messages）
    fn build_messages(&self, context: &AgentContext) -> Vec<Message> {
        let mut messages = Vec::with_capacity(context.messages.len() + 1);

        if !self.system_prompt.is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: Some(self.system_prompt.clone()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        for msg in &context.messages {
            messages.push(Message {
                role: msg.role.clone(),
                content: msg.content.clone(),
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
}

#[async_trait]
impl Agent for FunctionCallingAgent {
    /// 运行 Agent — 多轮工具调用循环
    ///
    /// 1. 接收用户输入
    /// 2. 调用 LLM，传入可用工具列表
    /// 3. 如果 LLM 返回 tool_calls，执行对应工具
    /// 4. 将工具结果反馈给 LLM
    /// 5. 重复 2-4 直到 LLM 返回最终答案
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse {
        context.add_user_message(user_input);

        let tools: Vec<Value> = self.tools.get_tools_schema();
        let has_tools = !tools.is_empty();
        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        let mut iterations: u32 = 0;

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
                    error: Some(format!("已达最大迭代次数 {}", self.max_iterations)),
                };
            }
            iterations += 1;
            self.dbg(&format!("[Agent] iter={iterations}"));

            let messages = self.build_messages(context);
            let request_payload = serde_json::json!({
                "messages": messages.clone(),
                "tools": if has_tools { Some(tools.clone()) } else { None }
            });
            let call_started = std::time::Instant::now();

            // 如果有工具，使用 chat_with_tools；否则使用 chat
            let result: ChatResponse = if has_tools {
                match self.llm.chat_with_tools(&messages, &tools, None).await {
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

                    // 记录 assistant 消息（含 tool_calls）
                    let tc_values: Vec<Value> = tcs
                        .iter()
                        .filter_map(|tc| serde_json::to_value(tc).ok())
                        .collect();
                    context.add_assistant_message(&result.content, Some(tc_values));

                    // 逐个执行工具
                    for tc in tcs {
                        let tool_name = &tc.function.name;
                        let tool_call_id = &tc.id;
                        let tool_args_str = &tc.function.arguments;

                        match serde_json::from_str::<Value>(tool_args_str) {
                            Ok(tool_args) => {
                                self.dbg(&format!("[Agent] tool_call name={tool_name}"));
                                if let Some(observer) = &self.tool_observer {
                                    observer.on_tool_start(tool_name, &tool_args, None).await;
                                }

                                match self.tools.execute_tool(tool_name, tool_args.clone()).await {
                                    Ok(tool_result) => {
                                        self.dbg(&format!("[Agent] tool_result name={tool_name}"));

                                        let tr: Value = tool_result.clone();
                                        tool_calls_made.push(ToolCallMade {
                                            name: tool_name.clone(),
                                            arguments: tool_args.clone(),
                                            result: tr,
                                            tool_call_id: Some(tool_call_id.clone()),
                                        });

                                        let result_str =
                                            serde_json::to_string(&tool_result).unwrap_or_default();
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
                                self.dbg(&format!("[Agent] json parse error for {tool_name}: {e}"));
                                let err_str = format!("参数解析失败: {e}");
                                let error_result: Value = serde_json::json!({"error": err_str});
                                let result_str =
                                    serde_json::to_string(&error_result).unwrap_or_default();
                                context.add_tool_result(tool_call_id, tool_name, &result_str);
                            }
                        }
                    }
                    // 继续循环 — 把工具结果送回 LLM
                    continue;
                }
            }

            // 没有工具调用 — 最终回复
            self.dbg("[Agent] done (no more tool_calls)");
            context.add_assistant_message(&result.content, None);
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
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockLlmProvider {
        state: Arc<Mutex<MockState>>,
    }

    struct MockState {
        chat_calls: usize,
        chat_with_tools_calls: usize,
        next_chat_response: Option<String>,
        next_tool_responses: VecDeque<ChatResponse>,
    }

    impl MockLlmProvider {
        fn with_chat_response(content: &str) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    next_chat_response: Some(content.to_string()),
                    next_tool_responses: VecDeque::new(),
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
            _messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            let mut state = self.state.lock().expect("mock state lock");
            state.chat_with_tools_calls += 1;
            match state.next_tool_responses.pop_front() {
                Some(resp) => Ok(resp),
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
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
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
                tool_calls: Some(vec![invalid_tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "fallback final".to_string(),
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
                tool_calls: Some(vec![tool_call]),
                usage: None,
            },
            ChatResponse {
                content: "done".to_string(),
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
}
