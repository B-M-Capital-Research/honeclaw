//! Hone Agent — Codex CLI Agent 核心
//!
//! 通过 `codex exec` 调用本地 Codex CLI，
//! 实现 `Agent` trait 以接入系统。
//!
//! ## 工具调用机制（Text-Based Tool Dispatch）
//!
//! Codex CLI 以 `codex exec` 模式运行，无法使用原生 Function Calling API。
//! 因此采用与 GeminiCliAgent 相同的文本协议：在系统 prompt 中注入调用规范，
//! 要求 LLM 以 `<tool_call>{"name":"...","arguments":{...},"reasoning":"正在..."}</tool_call>` 格式标记工具调用。
//! Rust 层解析该标签，执行 ToolRegistry 中的工具，将结果注入对话，循环直到无工具调用。

use async_trait::async_trait;
use hone_core::agent::{Agent, AgentContext, AgentResponse, ToolCallMade};
use hone_core::{LlmAuditRecord, LlmAuditSink, ToolExecutionObserver};
use hone_tools::registry::ToolRegistry;
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;

/// Codex CLI Agent Wrapper
pub struct CodexCliAgent {
    pub debug_log: bool,
    pub system_prompt: String,
    pub codex_model: Option<String>,
    pub working_directory: Option<String>,
    pub tools: Arc<ToolRegistry>,
    /// 最大工具调用循环次数，防止无限循环
    pub max_tool_iterations: u32,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub tool_observer: Option<Arc<dyn ToolExecutionObserver>>,
}

impl CodexCliAgent {
    pub fn new(
        system_prompt: String,
        codex_model: Option<String>,
        working_directory: Option<String>,
        tools: Arc<ToolRegistry>,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        let debug_log = std::env::var("HONE_AGENT_DEBUG")
            .map(|v| matches!(v.trim(), "1" | "true" | "True"))
            .unwrap_or(false);

        Self {
            debug_log,
            system_prompt,
            codex_model: codex_model.and_then(|m| {
                let trimmed = m.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }),
            working_directory: working_directory.and_then(|dir| {
                let trimmed = dir.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }),
            tools,
            max_tool_iterations: 5,
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

    fn record_audit(
        &self,
        context: &AgentContext,
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
            "agent.codex_cli",
            "cli_exec",
            "codex_cli",
            self.codex_model.clone(),
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
            tracing::warn!("[LlmAudit] failed to persist codex_cli audit: {}", err);
        }
    }

    /// 构建发送给 Codex CLI 的完整 prompt（与 GeminiCliAgent 保持一致的协议）
    fn build_prompt(
        &self,
        context: &AgentContext,
        tool_results: &[(String, String, String)], // (tool_call_id, tool_name, result)
    ) -> String {
        let mut prompt = String::new();

        if !self.system_prompt.is_empty() {
            prompt.push_str("### System Instructions ###\n");
            prompt.push_str(&self.system_prompt);
            prompt.push_str("\n\n");
        }

        if !self.tools.is_empty() {
            let tools_schema = self.tools.get_tools_schema();
            let tools_str = serde_json::to_string_pretty(&tools_schema).unwrap_or_default();

            prompt.push_str("### Available Tools ###\n");
            prompt.push_str(
                "You have access to the following tools. When you need to call a tool, \
                output EXACTLY this format (on its own line, nothing else before the closing tag):\n\
                <tool_call>{\"name\": \"tool_name\", \"arguments\": {\"param\": \"value\"}, \"reasoning\": \"正在...\"}</tool_call>\n\
                IMPORTANT RULES:\n\
                - Output ONLY ONE tool_call block per response if you need a tool.\n\
                - After the tool_call block, stop generating. Do NOT write anything after </tool_call>.\n\
                - Only use tools listed below. Do NOT invent tool names.\n\
                - Provide a short Chinese reasoning string starting with \"正在...\".\n\
                - After receiving a tool result, continue answering the user in Chinese.\n\n",
            );
            prompt.push_str(&tools_str);
            prompt.push_str("\n\n");
        }

        prompt.push_str("### Conversation History ###\n");
        for msg in &context.messages {
            let content = msg.content.as_deref().unwrap_or("");
            if !content.is_empty() {
                prompt.push_str(&format!("{}: {}\n\n", msg.role.to_uppercase(), content));
            }
        }

        // 注入工具调用结果（多轮循环时）
        if !tool_results.is_empty() {
            prompt.push_str("### Tool Results ###\n");
            for (_call_id, tool_name, result) in tool_results {
                prompt.push_str(&format!("TOOL[{}]: {}\n\n", tool_name, result));
            }
        }

        prompt.push_str("### Output Requirements ###\n");
        prompt.push_str(
            "Respond in Chinese. If you need a tool, output the <tool_call> block as instructed above.\n",
        );

        prompt
    }

    /// 解析 LLM 输出中的工具调用标签（与 GeminiCliAgent 保持相同逻辑）
    pub fn parse_tool_call(text: &str) -> (String, Option<(String, Value, Option<String>)>) {
        const OPEN: &str = "<tool_call>";
        const CLOSE: &str = "</tool_call>";

        if let Some(start) = text.find(OPEN) {
            if let Some(end) = text[start..].find(CLOSE) {
                let json_str = &text[start + OPEN.len()..start + end];
                let before = text[..start].trim().to_string();

                if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                    let name = parsed
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let arguments = parsed
                        .get("arguments")
                        .cloned()
                        .unwrap_or(Value::Object(serde_json::Map::new()));
                    let reasoning = parsed
                        .get("reasoning")
                        .and_then(|v| v.as_str())
                        .map(|value| value.to_string());

                    if !name.is_empty() {
                        return (before, Some((name, arguments, reasoning)));
                    }
                }
            }
        }

        (text.to_string(), None)
    }

    /// 调用 Codex CLI 进程，返回输出字符串
    async fn call_codex(&self, prompt: &str) -> Result<String, String> {
        let now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let output_path = std::env::temp_dir().join(format!(
            "hone_codex_cli_{}_{}.txt",
            std::process::id(),
            now_nanos
        ));

        let mut command = tokio::process::Command::new("codex");
        command
            .arg("exec")
            .arg("--skip-git-repo-check")
            .arg("--sandbox")
            .arg("workspace-write")
            .arg("--ask-for-approval")
            .arg("never")
            .arg("-o")
            .arg(&output_path);
        if let Some(model) = &self.codex_model {
            command.arg("-m").arg(model);
        }
        if let Some(working_directory) = &self.working_directory {
            command.arg("--cd").arg(working_directory);
        }
        let child = command
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                return Err(format!("failed to execute codex process: {}", e));
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(prompt.as_bytes()).await {
                let _ = tokio::fs::remove_file(&output_path).await;
                return Err(format!("failed to write prompt to codex stdin: {}", e));
            }
        }

        let result = child.wait_with_output().await;

        match result {
            Ok(output) => {
                if output.status.success() {
                    let content = match tokio::fs::read_to_string(&output_path).await {
                        Ok(file_content) if !file_content.trim().is_empty() => file_content,
                        _ => String::from_utf8_lossy(&output.stdout).to_string(),
                    };
                    let _ = tokio::fs::remove_file(&output_path).await;
                    Ok(content)
                } else {
                    let _ = tokio::fs::remove_file(&output_path).await;
                    let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(format!("codex cli exited with error: {}", err_msg))
                }
            }
            Err(e) => {
                let _ = tokio::fs::remove_file(&output_path).await;
                Err(format!("failed while waiting codex process: {}", e))
            }
        }
    }
}

#[async_trait]
impl Agent for CodexCliAgent {
    /// 运行单次交互，支持 Text-Based Tool Dispatch 多轮循环
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse {
        context.add_user_message(user_input);

        self.dbg(&format!("[CodexCliAgent] run with input: {user_input}"));

        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        let mut pending_tool_results: Vec<(String, String, String)> = Vec::new();
        let mut iteration = 0u32;

        loop {
            if iteration >= self.max_tool_iterations {
                self.dbg(&format!(
                    "[CodexCliAgent] 已达最大工具调用迭代次数 {}",
                    self.max_tool_iterations
                ));
                break;
            }
            iteration += 1;
            self.dbg(&format!("[CodexCliAgent] tool_dispatch iter={iteration}"));

            let prompt = self.build_prompt(context, &pending_tool_results);
            let request_payload = serde_json::json!({
                "prompt": prompt.clone(),
                "model": self.codex_model.clone()
            });
            let call_started = std::time::Instant::now();

            let content = match self.call_codex(&prompt).await {
                Ok(raw) => raw,
                Err(e) => {
                    self.record_audit(
                        context,
                        request_payload,
                        None,
                        Some(e.clone()),
                        call_started.elapsed().as_millis(),
                        serde_json::json!({ "iteration": iteration }),
                        None,
                    );
                    self.dbg(&format!("[CodexCliAgent] {e}"));
                    return AgentResponse {
                        content: String::new(),
                        tool_calls_made,
                        iterations: iteration,
                        success: false,
                        error: Some(e),
                    };
                }
            };

            self.record_audit(
                context,
                request_payload,
                Some(serde_json::json!({ "content": content.clone() })),
                None,
                call_started.elapsed().as_millis(),
                serde_json::json!({ "iteration": iteration }),
                None,
            );

            self.dbg(&format!(
                "[CodexCliAgent] output chars={}",
                content.chars().count()
            ));

            let (visible_text, maybe_tool_call) = Self::parse_tool_call(&content);

            if let Some((tool_name, tool_args, tool_reasoning)) = maybe_tool_call {
                tracing::info!(
                    "[Agent/codex_cli] tool_dispatch name={} iter={}",
                    tool_name,
                    iteration
                );
                self.dbg(&format!(
                    "[CodexCliAgent] tool_call detected name={tool_name}"
                ));

                if !visible_text.is_empty() {
                    context.add_assistant_message(&visible_text, None);
                }

                let call_id = format!("cli_tc_{}_{}", iteration, tool_name);
                if let Some(observer) = &self.tool_observer {
                    observer.on_tool_start(&tool_name, tool_reasoning).await;
                }

                match self.tools.execute_tool(&tool_name, tool_args.clone()).await {
                    Ok(tool_result) => {
                        let result_str = serde_json::to_string(&tool_result).unwrap_or_default();
                        tracing::info!(
                            "[Agent/codex_cli] tool_result name={} success=true",
                            tool_name
                        );
                        self.dbg(&format!("[CodexCliAgent] tool_result name={tool_name}"));

                        tool_calls_made.push(ToolCallMade {
                            name: tool_name.clone(),
                            arguments: tool_args,
                            result: tool_result,
                            tool_call_id: Some(call_id.clone()),
                        });

                        if let Some(observer) = &self.tool_observer {
                            observer.on_tool_finish(&tool_name, true).await;
                        }
                        context.add_tool_result(&call_id, &tool_name, &result_str);
                        pending_tool_results.push((call_id, tool_name, result_str));
                    }
                    Err(e) => {
                        tracing::error!(
                            "[Agent/codex_cli] tool_dispatch_error name={} error={}",
                            tool_name,
                            e
                        );
                        self.dbg(&format!(
                            "[CodexCliAgent] tool_error name={tool_name} error={e}"
                        ));
                        let err_val = serde_json::json!({"error": e.to_string()});
                        let result_str = serde_json::to_string(&err_val).unwrap_or_default();
                        if let Some(observer) = &self.tool_observer {
                            observer.on_tool_finish(&tool_name, false).await;
                        }
                        context.add_tool_result(&call_id, &tool_name, &result_str);
                        pending_tool_results.push((call_id, tool_name, result_str));
                    }
                }

                continue;
            }

            // 没有工具调用 — 最终回复
            self.dbg("[CodexCliAgent] done (no tool_call detected)");
            context.add_assistant_message(&content, None);

            return AgentResponse {
                content,
                tool_calls_made,
                iterations: iteration,
                success: true,
                error: None,
            };
        }

        // 超过最大迭代次数后，再调用一次拿最终回复
        self.dbg("[CodexCliAgent] max iterations reached, fetching final response");
        let prompt = self.build_prompt(context, &pending_tool_results);
        let request_payload = serde_json::json!({
            "prompt": prompt.clone(),
            "model": self.codex_model.clone()
        });
        let call_started = std::time::Instant::now();
        let content = match self.call_codex(&prompt).await {
            Ok(raw) => raw,
            Err(e) => {
                self.record_audit(
                    context,
                    request_payload,
                    None,
                    Some(e.clone()),
                    call_started.elapsed().as_millis(),
                    serde_json::json!({ "iteration": iteration, "final_fetch": true }),
                    None,
                );
                return AgentResponse {
                    content: String::new(),
                    tool_calls_made,
                    iterations: iteration,
                    success: false,
                    error: Some(e),
                };
            }
        };

        self.record_audit(
            context,
            request_payload,
            Some(serde_json::json!({ "content": content.clone() })),
            None,
            call_started.elapsed().as_millis(),
            serde_json::json!({ "iteration": iteration, "final_fetch": true }),
            None,
        );

        context.add_assistant_message(&content, None);
        AgentResponse {
            content,
            tool_calls_made,
            iterations: iteration,
            success: true,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CodexCliAgent;
    use hone_core::agent::AgentContext;
    use hone_tools::registry::ToolRegistry;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn parse_tool_call_detects_valid_call() {
        let text = r#"让我查一下这个。<tool_call>{"name": "data_fetch", "arguments": {"data_type": "quote", "symbol": "NVDA"}, "reasoning": "正在获取英伟达行情..."}</tool_call>"#;
        let (visible, maybe_call) = CodexCliAgent::parse_tool_call(text);
        assert_eq!(visible.trim(), "让我查一下这个。");
        let (name, args, reasoning) = maybe_call.expect("should have tool call");
        assert_eq!(name, "data_fetch");
        assert_eq!(args["data_type"], "quote");
        assert_eq!(reasoning.as_deref(), Some("正在获取英伟达行情..."));
    }

    #[test]
    fn parse_tool_call_no_tag() {
        let text = "这是普通回复。";
        let (visible, maybe_call) = CodexCliAgent::parse_tool_call(text);
        assert_eq!(visible, text);
        assert!(maybe_call.is_none());
    }

    #[test]
    fn parse_tool_call_skill_tool() {
        let text =
            r#"<tool_call>{"name": "skill_tool", "arguments": {"action": "list"}}</tool_call>"#;
        let (_visible, maybe_call) = CodexCliAgent::parse_tool_call(text);
        let (name, args, reasoning) = maybe_call.expect("should detect skill_tool");
        assert_eq!(name, "skill_tool");
        assert_eq!(args["action"], json!("list"));
        assert!(reasoning.is_none());
    }

    #[test]
    fn build_prompt_keeps_full_context_window() {
        let agent = CodexCliAgent::new(
            "system".to_string(),
            None,
            None,
            Arc::new(ToolRegistry::new()),
            None,
        );
        let mut context = AgentContext::new("session-1".to_string());
        for idx in 0..24 {
            context.add_user_message(&format!("u{idx}"));
        }

        let prompt = agent.build_prompt(&context, &[]);
        assert!(prompt.contains("USER: u0"));
        assert!(prompt.contains("USER: u23"));
    }
}
