//! Hone Agent — Gemini CLI Agent 核心
//!
//! 通过 `std::process::Command` 调用本地 `gemini` CLI，
//! 实现 `Agent` trait 以接入系统。
//!
//! ## 工具调用机制（Text-Based Tool Dispatch）
//!
//! Gemini CLI 以 `--prompt` 非交互模式运行，无法使用原生 Function Calling API。
//! 因此采用文本协议：在系统 prompt 中注入调用规范，要求 LLM 以
//! `<tool_call>{"name":"...","arguments":{...},"reasoning":"正在..."}</tool_call>` 格式标记工具调用。
//! Rust 层解析该标签，执行 ToolRegistry 中的工具，将结果注入对话，循环直到无工具调用。
//!
//! ## 流式事件解析（GeminiStreamEvent）
//!
//! Gemini CLI 以 `-o stream-json` 模式运行，每行输出一个 JSON 事件对象。
//! `parse_stream_event` 统一解析所有已知事件类型，同时兼容旧版 CLI 输出格式。
//!
//! 已知事件类型（对照 aioncli-core ServerGeminiEventType）：
//! - `content`          — 模型输出的文本增量
//! - `thought`          — 模型的思考过程（隐藏，不展示给用户）
//! - `tool_call_request`— 模型请求调用工具（当前使用文本协议替代）
//! - `error`            — 错误事件
//! - `finished`         — 流结束，含 token 统计
//! - `retry`            — 服务端要求重试
//! - `invalid_stream`   — 无效流（空响应等）
//! - `context_window_will_overflow` — 上下文窗口即将溢出
//! - 旧格式 `message`  — 兼容旧版 CLI（type=message, role=assistant）
//! - 旧格式 `response` — 兼容批量模式（{"response":"..."}）

use async_trait::async_trait;
use hone_core::agent::{Agent, AgentContext, AgentResponse, ToolCallMade};
use hone_core::{LlmAuditRecord, LlmAuditSink};
use hone_tools::registry::ToolRegistry;
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};

// ── 流式事件类型 ─────────────────────────────────────────────────────────────

/// Gemini CLI `-o stream-json` 每行输出的结构化事件。
///
/// 对照 AionUI / aioncli-core 的 `ServerGeminiEventType`，覆盖所有已知类型，
/// 并兼容旧版 Gemini CLI 输出格式。
#[derive(Debug, Clone)]
pub enum GeminiStreamEvent {
    /// 模型输出的文本内容增量（新格式 `type=content`，旧格式 `type=message/role=assistant`）
    Content(String),
    /// 模型思考过程，不展示给用户（`type=thought`）
    Thought(String),
    /// 模型请求调用工具，值为完整的工具调用信息（`type=tool_call_request`）
    ToolCallRequest(Value),
    /// 错误事件，携带可读错误信息（`type=error`）
    Error(String),
    /// 流正常结束，携带统计信息（`type=finished`）
    Finished(Value),
    /// 服务端要求重试（`type=retry`）
    Retry,
    /// 无效流，通常是空响应或无结束原因（`type=invalid_stream`）
    InvalidStream,
    /// 上下文窗口即将溢出，携带 token 估算数（`type=context_window_will_overflow`）
    ContextWindowOverflow { estimated: u64, remaining: u64 },
    /// 其他未识别类型，记录 type 值便于诊断
    Unknown(String),
}

impl GeminiStreamEvent {
    /// 若为 `Content` 变体，返回其文本内容；否则返回 `None`
    pub fn as_content(&self) -> Option<&str> {
        match self {
            GeminiStreamEvent::Content(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 若为 `Error` 变体，返回错误信息；否则返回 `None`
    pub fn as_error(&self) -> Option<&str> {
        match self {
            GeminiStreamEvent::Error(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 返回该事件是否代表流的终止（Error / Finished / InvalidStream）
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            GeminiStreamEvent::Error(_)
                | GeminiStreamEvent::Finished(_)
                | GeminiStreamEvent::InvalidStream
        )
    }
}

// ── 流式事件解析 ─────────────────────────────────────────────────────────────

/// 解析 Gemini CLI `-o stream-json` 输出的单行 JSON，返回结构化事件。
///
/// 支持格式：
///
/// **新格式（aioncli-core / gemini CLI ≥ v0.3x）**
/// - `{"type":"content","value":"文本"}`
/// - `{"type":"thought","value":"思考内容"}`
/// - `{"type":"tool_call_request","value":{...}}`
/// - `{"type":"error","value":{"error":"..."}}` 或 `{"type":"error","value":"..."}`
/// - `{"type":"finished","value":{...}}`
/// - `{"type":"retry"}`
/// - `{"type":"invalid_stream"}`
/// - `{"type":"context_window_will_overflow","value":{"estimatedRequestTokenCount":N,"remainingTokenCount":M}}`
///
/// **旧格式（gemini CLI 早期版本 / -o json 批量模式输出）**
/// - `{"type":"message","role":"assistant","content":"..."}` — 兼容旧版 stream-json
/// - `{"response":"..."}` 或 `{"session_id":"...","response":"..."}` — 兼容 -o json
///
/// **忽略的行**（返回 `None`）
/// - `{"type":"init",...}` / `{"type":"result",...}` / `{"type":"user",...}`
/// - 非 JSON 行（CLI 进度日志等）
pub fn parse_stream_event(line: &str) -> Option<GeminiStreamEvent> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }

    let Ok(json) = serde_json::from_str::<Value>(t) else {
        // 非 JSON 行（进度日志等），忽略
        return None;
    };

    let type_str = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match type_str {
        // ── 新格式事件 ────────────────────────────────────────────────────────
        "content" => {
            let value = json.get("value").cloned().unwrap_or(Value::Null);
            let text = match &value {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let s = text.trim();
            if s.is_empty() {
                None
            } else {
                Some(GeminiStreamEvent::Content(s.to_string()))
            }
        }

        "thought" => {
            let value = json
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if value.is_empty() {
                None
            } else {
                Some(GeminiStreamEvent::Thought(value))
            }
        }

        "tool_call_request" => {
            let value = json.get("value").cloned().unwrap_or(Value::Null);
            Some(GeminiStreamEvent::ToolCallRequest(value))
        }

        "error" => {
            let raw = json.get("value").cloned().unwrap_or(Value::Null);
            let msg = extract_error_message(&raw);
            Some(GeminiStreamEvent::Error(msg))
        }

        "finished" => {
            let value = json.get("value").cloned().unwrap_or(Value::Null);
            Some(GeminiStreamEvent::Finished(value))
        }

        "retry" => Some(GeminiStreamEvent::Retry),

        "invalid_stream" => Some(GeminiStreamEvent::InvalidStream),

        "context_window_will_overflow" => {
            let value = json.get("value");
            let estimated = value
                .and_then(|v| v.get("estimatedRequestTokenCount"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let remaining = value
                .and_then(|v| v.get("remainingTokenCount"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Some(GeminiStreamEvent::ContextWindowOverflow {
                estimated,
                remaining,
            })
        }

        // ── 旧格式兼容：type=message ──────────────────────────────────────────
        "message" => {
            let role = json.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if role == "assistant" {
                if let Some(content) = json.get("content").and_then(|v| v.as_str()) {
                    let s = content.trim();
                    if !s.is_empty() {
                        return Some(GeminiStreamEvent::Content(s.to_string()));
                    }
                }
            }
            // role=user 回显或其他 message 类型 → 忽略
            None
        }

        // ── 明确忽略的类型 ─────────────────────────────────────────────────────
        "init"
        | "result"
        | "user"
        | "chat_compressed"
        | "user_cancelled"
        | "tool_call_confirmation"
        | "tool_call_response"
        | "max_session_turns"
        | "loop_detected"
        | "model_info"
        | "agent_execution_stopped"
        | "agent_execution_blocked" => None,

        // ── 带 type 字段但未知类型：记录并返回 Unknown ────────────────────────
        other if !other.is_empty() => Some(GeminiStreamEvent::Unknown(other.to_string())),

        // ── 无 type 字段：尝试旧格式 {"response":"..."} ───────────────────────
        _ => {
            if let Some(resp) = json.get("response").and_then(|v| v.as_str()) {
                let s = resp.trim();
                if !s.is_empty() {
                    return Some(GeminiStreamEvent::Content(s.to_string()));
                }
            }
            None
        }
    }
}

/// 从 error 事件的 value 字段中提取可读错误信息
fn extract_error_message(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Object(map) => {
            // 尝试 .error, .error.message, .message 等常见字段
            if let Some(err) = map.get("error") {
                match err {
                    Value::String(s) => return s.clone(),
                    Value::Object(inner) => {
                        if let Some(msg) = inner.get("message").and_then(|v| v.as_str()) {
                            return msg.to_string();
                        }
                    }
                    _ => {}
                }
            }
            if let Some(msg) = map.get("message").and_then(|v| v.as_str()) {
                return msg.to_string();
            }
            value.to_string()
        }
        Value::Null => "Unknown error occurred".to_string(),
        other => other.to_string(),
    }
}

// ── Agent 实现 ────────────────────────────────────────────────────────────────

/// Gemini CLI Agent Wrapper
pub struct GeminiCliAgent {
    pub debug_log: bool,
    pub system_prompt: String,
    pub tools: Arc<ToolRegistry>,
    /// 最大工具调用循环次数，防止无限循环
    pub max_tool_iterations: u32,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl GeminiCliAgent {
    pub fn new(
        system_prompt: String,
        tools: Arc<ToolRegistry>,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        let debug_log = std::env::var("HONE_AGENT_DEBUG")
            .map(|v| matches!(v.trim(), "1" | "true" | "True"))
            .unwrap_or(false);

        Self {
            debug_log,
            system_prompt,
            tools,
            max_tool_iterations: 8,
            llm_audit,
        }
    }

    fn dbg(&self, msg: &str) {
        if self.debug_log {
            tracing::debug!("{msg}");
        }
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
            "agent.gemini_cli",
            operation.to_string(),
            "gemini_cli",
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
            tracing::warn!("[LlmAudit] failed to persist gemini_cli audit: {}", err);
        }
    }

    /// 将字符串截断到最多 `max_bytes` 字节（保持 UTF-8 合法），超出时附加截断提示。
    fn truncate_to_bytes(s: &str, max_bytes: usize) -> String {
        if s.len() <= max_bytes {
            return s.to_string();
        }
        // 找到最近的合法 UTF-8 字符边界
        let mut end = max_bytes;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…[内容过长已截断]", &s[..end])
    }

    /// 公有静态版 build_prompt，供 hone-imessage / hone-feishu 流式路径直接调用
    ///
    /// ## 内存安全策略
    ///
    /// 为防止 `--prompt` 参数超出 OS `ARG_MAX` 限制（E2BIG / os error 7），本函数
    /// 严格限制最终 prompt 的字节大小：
    ///
    /// - `MAX_PROMPT_BYTES = 150_000`：整体上限，远低于 macOS/Linux 典型 ARG_MAX
    /// - `MAX_TOOL_RESULT_BYTES = 6_000`：每条工具结果单独截断（财报 JSON 可能很大）
    /// - 对话历史从最新到最旧依次填充，超出剩余预算时停止（保留最近上下文）
    pub fn build_streaming_prompt(
        system_prompt: &str,
        context: &AgentContext,
        tool_registry: &ToolRegistry,
        tool_results: &[(String, String, String)],
    ) -> String {
        /// 整体 prompt 字节上限（~150KB，安全地低于 OS ARG_MAX）
        const MAX_PROMPT_BYTES: usize = 150_000;
        /// 每条工具结果的字节上限
        const MAX_TOOL_RESULT_BYTES: usize = 6_000;

        let mut prompt = String::new();

        if !system_prompt.is_empty() {
            prompt.push_str("### System Instructions ###\n");
            prompt.push_str(system_prompt);
            prompt.push_str("\n\n");
        }

        if !tool_registry.is_empty() {
            let tools_schema = tool_registry.get_tools_schema();
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

        // 预先构建尾部（工具结果 + 输出要求），以便计算对话历史的可用预算
        let mut tail = String::new();
        if !tool_results.is_empty() {
            tail.push_str("### Tool Results ###\n");
            for (_call_id, tool_name, result) in tool_results {
                let truncated = Self::truncate_to_bytes(result, MAX_TOOL_RESULT_BYTES);
                tail.push_str(&format!("TOOL[{}]: {}\n\n", tool_name, truncated));
            }
        }
        tail.push_str("### Output Requirements ###\n");
        tail.push_str(
            "Respond in Chinese. If you need a tool, output the <tool_call> block as instructed above.\n",
        );

        // 计算对话历史可用字节预算
        let history_header = "### Conversation History ###\n";
        let fixed_size = prompt.len() + tail.len() + history_header.len();
        let history_budget = MAX_PROMPT_BYTES.saturating_sub(fixed_size);

        // 从最新消息向最旧消息填充，超出预算时停止
        prompt.push_str(history_header);
        let mut history_bytes_used = 0usize;
        let mut history_lines: Vec<String> = Vec::new();
        for msg in context.messages.iter().rev() {
            let content = msg.content.as_deref().unwrap_or("");
            if content.is_empty() {
                continue;
            }
            let line = format!("{}: {}\n\n", msg.role.to_uppercase(), content);
            if history_bytes_used + line.len() > history_budget {
                break;
            }
            history_bytes_used += line.len();
            history_lines.push(line);
        }
        // 恢复时间顺序（最旧→最新）
        for line in history_lines.into_iter().rev() {
            prompt.push_str(&line);
        }

        prompt.push_str(&tail);
        prompt
    }

    /// 构建发送给 Gemini CLI 的完整 prompt
    ///
    /// 包含：System Instructions、工具调用协议说明、工具列表、对话历史、当前用户输入
    ///
    /// ## 内存安全策略
    ///
    /// 同 `build_streaming_prompt`，限制最终 prompt 字节大小以防止 E2BIG 错误。
    fn build_prompt(
        &self,
        system_prompt: &str,
        context: &AgentContext,
        tool_results: &[(String, String, String)], // (tool_call_id, tool_name, result)
    ) -> String {
        /// 整体 prompt 字节上限（~150KB，安全地低于 OS ARG_MAX）
        const MAX_PROMPT_BYTES: usize = 150_000;
        /// 每条工具结果的字节上限
        const MAX_TOOL_RESULT_BYTES: usize = 6_000;

        let mut prompt = String::new();

        if !system_prompt.is_empty() {
            prompt.push_str("### System Instructions ###\n");
            prompt.push_str(system_prompt);
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

        // 预先构建尾部（工具结果 + 输出要求），以便计算对话历史的可用预算
        let mut tail = String::new();
        if !tool_results.is_empty() {
            tail.push_str("### Tool Results ###\n");
            for (_call_id, tool_name, result) in tool_results {
                let truncated = Self::truncate_to_bytes(result, MAX_TOOL_RESULT_BYTES);
                tail.push_str(&format!("TOOL[{}]: {}\n\n", tool_name, truncated));
            }
        }
        tail.push_str("### Output Requirements ###\n");
        tail.push_str(
            "Respond in Chinese. If you need a tool, output the <tool_call> block as instructed above.\n",
        );

        // 计算对话历史可用字节预算
        let history_header = "### Conversation History ###\n";
        let fixed_size = prompt.len() + tail.len() + history_header.len();
        let history_budget = MAX_PROMPT_BYTES.saturating_sub(fixed_size);

        // 从最新消息向最旧消息填充，超出预算时停止
        prompt.push_str(history_header);
        let mut history_bytes_used = 0usize;
        let mut history_lines: Vec<String> = Vec::new();
        for msg in context.messages.iter().rev() {
            let content = msg.content.as_deref().unwrap_or("");
            if content.is_empty() {
                continue;
            }
            let line = format!("{}: {}\n\n", msg.role.to_uppercase(), content);
            if history_bytes_used + line.len() > history_budget {
                break;
            }
            history_bytes_used += line.len();
            history_lines.push(line);
        }
        // 恢复时间顺序（最旧→最新）
        for line in history_lines.into_iter().rev() {
            prompt.push_str(&line);
        }

        // 注入工具调用结果（多轮循环时）
        prompt.push_str(&tail);

        prompt
    }

    /// 解析 LLM 输出中的工具调用标签
    ///
    /// 返回 `(visible_text, Option<(name, arguments_value, reasoning)>)`：
    /// - `visible_text`：去掉 `<tool_call>` 标签后用户可见的文本部分
    /// - `Some((name, args))`：当检测到完整的工具调用标签时
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
                        .map(|s| s.to_string());

                    if !name.is_empty() {
                        return (before, Some((name, arguments, reasoning)));
                    }
                }
            }
        }

        // 没有有效的工具调用
        (text.to_string(), None)
    }

    /// 调用 Gemini CLI 进程（`-o stream-json` 流式模式），返回累积的完整文本内容。
    ///
    /// 内部逐行读取 stdout，使用 `parse_stream_event` 解析每一行：
    /// - `Content` 事件累积到输出缓冲区
    /// - `Error` 事件立即终止并返回 `Err`
    /// - `Thought` 事件记录调试日志，不累积
    /// - `Finished` 事件标记流结束
    /// - `Retry` / `InvalidStream` 事件记录警告
    /// - `ContextWindowOverflow` 事件返回错误
    ///
    /// 超时行为：每行最长等待 180 秒，整体最长等待 600 秒。
    async fn call_gemini(
        &self,
        prompt: &str,
    ) -> Result<(String, Option<hone_llm::provider::TokenUsage>), String> {
        const PER_LINE_TIMEOUT_SECS: u64 = 180;
        const OVERALL_TIMEOUT_SECS: u64 = 600;

        let mut child = tokio::process::Command::new("gemini")
            .arg("--prompt")
            .arg(prompt)
            .arg("--yolo")
            .arg("-o")
            .arg("stream-json")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to execute gemini process: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "gemini stdout unavailable".to_string())?;

        let mut reader = BufReader::new(stdout).lines();
        let mut content_buf = String::new();
        let mut usage = None;
        let overall_start = std::time::Instant::now();
        let per_line_timeout = std::time::Duration::from_secs(PER_LINE_TIMEOUT_SECS);
        let overall_timeout = std::time::Duration::from_secs(OVERALL_TIMEOUT_SECS);

        loop {
            // 整体超时检查
            if overall_start.elapsed() > overall_timeout {
                let _ = child.kill().await;
                return Err(format!(
                    "gemini cli timed out after {}s",
                    OVERALL_TIMEOUT_SECS
                ));
            }

            match tokio::time::timeout(per_line_timeout, reader.next_line()).await {
                Ok(Ok(Some(line))) => {
                    match parse_stream_event(&line) {
                        Some(GeminiStreamEvent::Content(text)) => {
                            content_buf.push_str(&text);
                        }
                        Some(GeminiStreamEvent::Thought(thought)) => {
                            self.dbg(&format!("[GeminiCliAgent] thought: {thought}"));
                        }
                        Some(GeminiStreamEvent::Error(msg)) => {
                            let _ = child.kill().await;
                            return Err(format!("gemini cli error event: {}", msg));
                        }
                        Some(GeminiStreamEvent::ContextWindowOverflow {
                            estimated,
                            remaining,
                        }) => {
                            let _ = child.kill().await;
                            return Err(format!(
                                "context window overflow: estimated={}K remaining={}K tokens. \
                                Start a new conversation or reduce context.",
                                estimated / 1000,
                                remaining / 1000
                            ));
                        }
                        Some(GeminiStreamEvent::Finished(val)) => {
                            self.dbg("[GeminiCliAgent] stream finished event received");
                            if let Some(meta) = val.get("usageMetadata") {
                                let p = meta
                                    .get("promptTokenCount")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);
                                let c = meta
                                    .get("candidatesTokenCount")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);
                                let t = meta
                                    .get("totalTokenCount")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32);
                                if p.is_some() || c.is_some() || t.is_some() {
                                    usage = Some(hone_llm::provider::TokenUsage {
                                        prompt_tokens: p,
                                        completion_tokens: c,
                                        total_tokens: t,
                                    });
                                }
                            }
                            break;
                        }
                        Some(GeminiStreamEvent::Retry) => {
                            tracing::warn!("[GeminiCliAgent] stream retry event received");
                        }
                        Some(GeminiStreamEvent::InvalidStream) => {
                            tracing::warn!("[GeminiCliAgent] invalid stream event received");
                            // 不立即中止，继续读取剩余行
                        }
                        Some(GeminiStreamEvent::ToolCallRequest(val)) => {
                            self.dbg(&format!(
                                "[GeminiCliAgent] native tool_call_request event (ignored, using text protocol): {}",
                                val
                            ));
                        }
                        Some(GeminiStreamEvent::Unknown(type_name)) => {
                            self.dbg(&format!(
                                "[GeminiCliAgent] unknown stream event type: {}",
                                type_name
                            ));
                        }
                        None => {
                            // 忽略行（init/result/非JSON/旧格式 user 消息等）
                        }
                    }
                }
                // EOF：进程正常退出
                Ok(Ok(None)) => break,
                // IO 错误
                Ok(Err(e)) => {
                    return Err(format!("gemini cli stdout read error: {}", e));
                }
                // 每行超时
                Err(_) => {
                    let _ = child.kill().await;
                    return Err(format!(
                        "gemini cli per-line timeout ({}s) exceeded",
                        PER_LINE_TIMEOUT_SECS
                    ));
                }
            }
        }

        // 等待进程退出，读取 stderr
        if let Ok(out) = child.wait_with_output().await {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stderr_trimmed = stderr.trim();
            if !stderr_trimmed.is_empty() {
                tracing::warn!("[GeminiCliAgent] stderr: {}", stderr_trimmed);
            }
            if !out.status.success() && content_buf.is_empty() {
                return Err(format!(
                    "gemini cli exited with error (code={:?}): {}",
                    out.status.code(),
                    stderr_trimmed
                ));
            }
        }

        if content_buf.is_empty() {
            Ok(("未提取到有效的回复。".to_string(), usage))
        } else {
            Ok((content_buf, usage))
        }
    }
}

#[async_trait]
impl Agent for GeminiCliAgent {
    /// 运行单次交互，支持 Text-Based Tool Dispatch 多轮循环
    ///
    /// 流程：
    /// 1. 构建 prompt（含工具调用协议说明）
    /// 2. 调用 Gemini CLI（stream-json 模式，逐行解析）
    /// 3. 解析输出，检测 `<tool_call>` 标签
    /// 4. 若有工具调用，执行工具，将结果注入，重复 1-4（最多 max_tool_iterations 次）
    /// 5. 无工具调用时返回最终答案
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse {
        context.add_user_message(user_input);

        self.dbg(&format!("[GeminiCliAgent] run with input: {user_input}"));

        let mut tool_calls_made: Vec<ToolCallMade> = Vec::new();
        // 累积的工具结果（用于注入到后续 prompt）
        let mut pending_tool_results: Vec<(String, String, String)> = Vec::new();
        let mut iteration = 0u32;

        loop {
            if iteration >= self.max_tool_iterations {
                self.dbg(&format!(
                    "[GeminiCliAgent] 已达最大工具调用迭代次数 {}",
                    self.max_tool_iterations
                ));
                break;
            }
            iteration += 1;
            self.dbg(&format!("[GeminiCliAgent] tool_dispatch iter={iteration}"));

            let prompt = self.build_prompt(&self.system_prompt, context, &pending_tool_results);
            let request_payload = serde_json::json!({ "prompt": prompt.clone() });
            let call_started = std::time::Instant::now();

            let (content, usage) = match self.call_gemini(&prompt).await {
                Ok(res) => res,
                Err(e) => {
                    self.record_audit(
                        context,
                        "cli_exec",
                        request_payload,
                        None,
                        Some(e.clone()),
                        call_started.elapsed().as_millis(),
                        serde_json::json!({ "iteration": iteration }),
                        None,
                    );
                    self.dbg(&format!("[GeminiCliAgent] {e}"));
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
                "cli_exec",
                request_payload,
                Some(serde_json::json!({ "content": content.clone() })),
                None,
                call_started.elapsed().as_millis(),
                serde_json::json!({ "iteration": iteration }),
                usage,
            );

            self.dbg(&format!(
                "[GeminiCliAgent] content chars={}",
                content.chars().count()
            ));

            // 解析工具调用
            let (visible_text, maybe_tool_call) = Self::parse_tool_call(&content);

            if let Some((tool_name, tool_args, _reasoning)) = maybe_tool_call {
                tracing::info!(
                    "[Agent/gemini_cli] tool_dispatch name={} iter={}",
                    tool_name,
                    iteration
                );
                self.dbg(&format!(
                    "[GeminiCliAgent] tool_call detected name={tool_name}"
                ));

                // 若有可见文本（工具调用前的说明），先追加到 context
                if !visible_text.is_empty() {
                    context.add_assistant_message(&visible_text, None);
                }

                // 生成唯一的 call_id
                let call_id = format!("cli_tc_{}_{}", iteration, tool_name);

                match self.tools.execute_tool(&tool_name, tool_args.clone()).await {
                    Ok(tool_result) => {
                        let result_str = serde_json::to_string(&tool_result).unwrap_or_default();
                        tracing::info!(
                            "[Agent/gemini_cli] tool_result name={} success=true",
                            tool_name
                        );
                        self.dbg(&format!(
                            "[GeminiCliAgent] tool_result name={tool_name} result_chars={}",
                            result_str.chars().count()
                        ));

                        tool_calls_made.push(ToolCallMade {
                            name: tool_name.clone(),
                            arguments: tool_args,
                            result: tool_result,
                            tool_call_id: Some(call_id.clone()),
                        });

                        // 将工具结果记录到 context（role=tool）
                        context.add_tool_result(&call_id, &tool_name, &result_str);
                        // 同时追加到 pending_tool_results 供下一轮 prompt 引用
                        pending_tool_results.push((call_id, tool_name, result_str));
                    }
                    Err(e) => {
                        tracing::error!(
                            "[Agent/gemini_cli] tool_dispatch_error name={} error={}",
                            tool_name,
                            e
                        );
                        self.dbg(&format!(
                            "[GeminiCliAgent] tool_error name={tool_name} error={e}"
                        ));
                        let err_val = serde_json::json!({"error": e.to_string()});
                        let result_str = serde_json::to_string(&err_val).unwrap_or_default();
                        context.add_tool_result(&call_id, &tool_name, &result_str);
                        pending_tool_results.push((call_id, tool_name, result_str));
                    }
                }

                // 继续循环，把工具结果送回 LLM
                continue;
            }

            // 没有工具调用 — 这是最终回复
            self.dbg("[GeminiCliAgent] done (no tool_call detected)");
            context.add_assistant_message(&content, None);

            return AgentResponse {
                content,
                tool_calls_made,
                iterations: iteration,
                success: true,
                error: None,
            };
        }

        // 超过最大迭代次数后，再调用一次拿最终回复（不再解析工具调用）
        self.dbg("[GeminiCliAgent] max iterations reached, fetching final response");
        let prompt = self.build_prompt(&self.system_prompt, context, &pending_tool_results);
        let request_payload = serde_json::json!({ "prompt": prompt.clone() });
        let call_started = std::time::Instant::now();
        let (content, usage) = match self.call_gemini(&prompt).await {
            Ok(res) => res,
            Err(e) => {
                self.record_audit(
                    context,
                    "cli_exec",
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
            "cli_exec",
            request_payload,
            Some(serde_json::json!({ "content": content.clone() })),
            None,
            call_started.elapsed().as_millis(),
            serde_json::json!({ "iteration": iteration, "final_fetch": true }),
            usage,
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
    use super::{GeminiCliAgent, GeminiStreamEvent, parse_stream_event};
    use serde_json::json;

    // ── parse_stream_event: 新格式 ────────────────────────────────────────────

    #[test]
    fn parse_event_content_new_format() {
        let line = r#"{"type":"content","value":"hello world"}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert_eq!(event.as_content(), Some("hello world"));
    }

    #[test]
    fn parse_event_thought() {
        let line = r#"{"type":"thought","value":"thinking..."}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert!(matches!(event, GeminiStreamEvent::Thought(s) if s == "thinking..."));
    }

    #[test]
    fn parse_event_error_string_value() {
        let line = r#"{"type":"error","value":"something went wrong"}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert_eq!(event.as_error(), Some("something went wrong"));
    }

    #[test]
    fn parse_event_error_object_value() {
        let line = r#"{"type":"error","value":{"error":{"message":"quota exceeded"}}}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert_eq!(event.as_error(), Some("quota exceeded"));
    }

    #[test]
    fn parse_event_finished() {
        let line = r#"{"type":"finished","value":{"tokenCount":123}}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert!(matches!(event, GeminiStreamEvent::Finished(_)));
        assert!(event.is_terminal());
    }

    #[test]
    fn parse_event_retry() {
        let line = r#"{"type":"retry"}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert!(matches!(event, GeminiStreamEvent::Retry));
    }

    #[test]
    fn parse_event_invalid_stream() {
        let line = r#"{"type":"invalid_stream"}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert!(matches!(event, GeminiStreamEvent::InvalidStream));
        assert!(event.is_terminal());
    }

    #[test]
    fn parse_event_context_window_overflow() {
        let line = r#"{"type":"context_window_will_overflow","value":{"estimatedRequestTokenCount":150000,"remainingTokenCount":8000}}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert!(matches!(
            event,
            GeminiStreamEvent::ContextWindowOverflow {
                estimated: 150000,
                remaining: 8000
            }
        ));
    }

    #[test]
    fn parse_event_tool_call_request() {
        let line =
            r#"{"type":"tool_call_request","value":{"name":"web_search","args":{"q":"test"}}}"#;
        let event = parse_stream_event(line).expect("should parse");
        assert!(matches!(event, GeminiStreamEvent::ToolCallRequest(_)));
    }

    // ── parse_stream_event: 旧格式兼容 ───────────────────────────────────────

    #[test]
    fn parse_event_legacy_message_assistant() {
        let line = r#"{"type":"message","role":"assistant","content":"legacy reply"}"#;
        let event = parse_stream_event(line).expect("should parse legacy format");
        assert_eq!(event.as_content(), Some("legacy reply"));
    }

    #[test]
    fn parse_event_legacy_message_user_ignored() {
        let line = r#"{"type":"message","role":"user","content":"user text"}"#;
        let event = parse_stream_event(line);
        assert!(event.is_none(), "user role message should be ignored");
    }

    #[test]
    fn parse_event_legacy_response_field() {
        let line = r#"{"session_id":"x","response":"hello world"}"#;
        let event = parse_stream_event(line).expect("should parse legacy response format");
        assert_eq!(event.as_content(), Some("hello world"));
    }

    // ── parse_stream_event: 忽略的行 ──────────────────────────────────────────

    #[test]
    fn parse_event_init_ignored() {
        let line = r#"{"type":"init","version":"1.0"}"#;
        assert!(parse_stream_event(line).is_none());
    }

    #[test]
    fn parse_event_result_ignored() {
        let line = r#"{"type":"result","data":{}}"#;
        assert!(parse_stream_event(line).is_none());
    }

    #[test]
    fn parse_event_non_json_ignored() {
        assert!(parse_stream_event("not json at all").is_none());
        assert!(parse_stream_event("").is_none());
        assert!(parse_stream_event("   ").is_none());
    }

    #[test]
    fn parse_event_empty_content_ignored() {
        let line = r#"{"type":"content","value":""}"#;
        assert!(
            parse_stream_event(line).is_none(),
            "empty content should be ignored"
        );
    }

    #[test]
    fn parse_event_unknown_type_returns_unknown() {
        let line = r#"{"type":"some_future_event","data":"x"}"#;
        let event =
            parse_stream_event(line).expect("unknown type should still produce Unknown variant");
        assert!(matches!(event, GeminiStreamEvent::Unknown(s) if s == "some_future_event"));
    }

    // ── GeminiStreamEvent helpers ─────────────────────────────────────────────

    #[test]
    fn stream_event_is_terminal_checks() {
        assert!(GeminiStreamEvent::Error("x".to_string()).is_terminal());
        assert!(GeminiStreamEvent::Finished(json!({})).is_terminal());
        assert!(GeminiStreamEvent::InvalidStream.is_terminal());
        assert!(!GeminiStreamEvent::Content("x".to_string()).is_terminal());
        assert!(!GeminiStreamEvent::Thought("x".to_string()).is_terminal());
        assert!(!GeminiStreamEvent::Retry.is_terminal());
    }

    // ── parse_tool_call（保持原有测试）────────────────────────────────────────

    #[test]
    fn parse_tool_call_detects_valid_call() {
        let text = r#"好的，我来查一下。<tool_call>{"name": "web_search", "arguments": {"query": "NVDA stock"}, "reasoning": "正在搜索信息..."}</tool_call>"#;
        let (visible, maybe_call) = GeminiCliAgent::parse_tool_call(text);
        assert_eq!(visible.trim(), "好的，我来查一下。");
        let (name, args, reasoning) = maybe_call.expect("should have tool call");
        assert_eq!(name, "web_search");
        assert_eq!(args["query"], "NVDA stock");
        assert_eq!(reasoning.as_deref(), Some("正在搜索信息..."));
    }

    #[test]
    fn parse_tool_call_no_tag_returns_full_text() {
        let text = "这是普通回复，不包含工具调用。";
        let (visible, maybe_call) = GeminiCliAgent::parse_tool_call(text);
        assert_eq!(visible, text);
        assert!(maybe_call.is_none());
    }

    #[test]
    fn parse_tool_call_invalid_json_returns_no_call() {
        let text = "<tool_call>not json at all</tool_call>";
        let (_visible, maybe_call) = GeminiCliAgent::parse_tool_call(text);
        assert!(maybe_call.is_none());
    }

    #[test]
    fn parse_tool_call_empty_name_returns_no_call() {
        let text = r#"<tool_call>{"name": "", "arguments": {}}</tool_call>"#;
        let (_visible, maybe_call) = GeminiCliAgent::parse_tool_call(text);
        assert!(maybe_call.is_none());
    }

    #[test]
    fn parse_tool_call_deep_research() {
        let text = r#"<tool_call>{"name": "deep_research", "arguments": {"company_name": "英伟达"}}</tool_call>"#;
        let (visible, maybe_call) = GeminiCliAgent::parse_tool_call(text);
        assert!(visible.trim().is_empty());
        let (name, args, reasoning) = maybe_call.expect("should detect deep_research");
        assert_eq!(name, "deep_research");
        assert_eq!(args["company_name"], json!("英伟达"));
        assert!(reasoning.is_none());
    }

    #[test]
    fn parse_tool_call_reasoning_missing_returns_none() {
        let text =
            r#"<tool_call>{"name": "web_search", "arguments": {"query": "NVDA"}}</tool_call>"#;
        let (_visible, maybe_call) = GeminiCliAgent::parse_tool_call(text);
        let (_name, _args, reasoning) = maybe_call.expect("should have tool call");
        assert!(reasoning.is_none());
    }
}
