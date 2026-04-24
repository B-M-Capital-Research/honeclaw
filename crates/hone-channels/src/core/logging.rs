//! 启动期 + 消息流的结构化日志。
//!
//! 这里集中放:
//! - **启动路由日志** (`log_startup_routing`):把 config 里哪个 runner 被选中、
//!   哪个 LLM provider 被用、session 后端怎么配的,一次性打到 stdout,
//!   运维一眼就能判断「这份进程的配置是对的吗」;
//! - **消息流日志** (`log_message_*`):把 `received → step → finished / failed`
//!   写成统一的 `[MsgFlow/<channel>]` 前缀 + tracing 字段,方便用
//!   `message_id=` / `state=` 做 grep & timeline;
//! - **pub(super) 的格式化 helper**:`truncate_for_log` / `printable_or_default` /
//!   `summarize_tools`,供 `intercept.rs` 和其它 sibling module 复用,避免每个
//!   文件都重写一份「把 120 字符以外截掉 + 换行转义」。

use hone_core::agent::AgentResponse;
use hone_core::config::AgentRunnerKind;

use super::bot_core::HoneBotCore;

impl HoneBotCore {
    pub fn runner_supports_strict_actor_sandbox(&self) -> bool {
        true
    }

    pub fn strict_actor_sandbox_guard_message(&self) -> Option<&'static str> {
        None
    }

    /// 打印启动期路由信息（配置来源、主对话执行器、压缩执行器）
    pub fn log_startup_routing(&self, channel: &str, config_path: &str) {
        let llm_provider = self.config.llm.provider.trim();
        let (llm_model, llm_timeout, llm_max_tokens) = match llm_provider {
            "kimi" => (
                printable_or_default(&self.config.llm.kimi.model, "<empty>"),
                self.config.llm.kimi.timeout,
                self.config.llm.kimi.max_tokens,
            ),
            _ => (
                printable_or_default(&self.config.llm.openrouter.model, "<empty>"),
                self.config.llm.openrouter.timeout,
                self.config.llm.openrouter.max_tokens,
            ),
        };

        let llm_api_key_source = if match llm_provider {
            "kimi" => !self.config.llm.kimi.api_key.trim().is_empty(),
            _ => !self.config.llm.openrouter.api_key.trim().is_empty(),
        } {
            "config.yaml"
        } else {
            "empty"
        };

        tracing::info!("[Startup/{channel}] config.path={config_path}");
        tracing::info!(
            "[Startup/{channel}] llm.provider={} llm.model={} timeout={}s max_tokens={} api_key.source={}",
            printable_or_default(llm_provider, "<empty>"),
            llm_model,
            llm_timeout,
            llm_max_tokens,
            llm_api_key_source
        );
        tracing::info!(
            "[Startup/{channel}] agent.step_timeout={}s agent.overall_timeout={}s",
            self.config.agent.step_timeout_seconds.max(1),
            self.config
                .agent
                .overall_timeout_seconds
                .max(self.config.agent.step_timeout_seconds.max(1))
        );

        match self.config.agent.runner_kind() {
            AgentRunnerKind::GeminiCli => tracing::info!(
                "[Startup/{channel}] dialog.engine=gemini_cli command=gemini model.source=gemini-cli(profile/default)"
            ),
            AgentRunnerKind::GeminiAcp => tracing::warn!(
                "[Startup/{channel}] dialog.engine=gemini_acp 已禁用：gemini ACP 未推 usage_update，\
                 honeclaw 无法识别其内置 compact 信号；并且 Gemini ToS 不建议在第三方 ACP 客户端中长期复用 session。\
                 请在 config 中切换到 codex_acp / opencode_acp / multi-agent。"
            ),
            AgentRunnerKind::CodexCli => tracing::info!(
                "[Startup/{channel}] dialog.engine=codex_cli command=codex exec model={}",
                printable_or_default(&self.config.agent.codex_model, "<codex-cli-default>")
            ),
            AgentRunnerKind::OpencodeAcp => tracing::info!(
                "[Startup/{channel}] dialog.engine=opencode_acp transport=stdio-jsonrpc command={} args={:?}",
                printable_or_default(&self.config.agent.opencode.command, "opencode"),
                self.config.agent.opencode.args
            ),
            AgentRunnerKind::MultiAgent => tracing::info!(
                "[Startup/{channel}] dialog.engine=multi-agent search.base_url={} search.model={} answer.base_url={} answer.model={} answer.variant={} max_iterations={} max_tool_calls={}",
                printable_or_default(&self.config.agent.multi_agent.search.base_url, "<empty>"),
                printable_or_default(&self.config.agent.multi_agent.search.model, "<empty>"),
                printable_or_default(
                    &self.config.agent.multi_agent.answer.api_base_url,
                    "<empty>"
                ),
                printable_or_default(&self.config.agent.multi_agent.answer.model, "<empty>"),
                printable_or_default(&self.config.agent.multi_agent.answer.variant, "<empty>"),
                self.config.agent.multi_agent.search.max_iterations,
                self.config.agent.multi_agent.answer.max_tool_calls,
            ),
            AgentRunnerKind::CodexAcp => tracing::info!(
                "[Startup/{channel}] dialog.engine=codex_acp transport=stdio-jsonrpc command={} args={:?} codex_command={} sandbox_mode={} approval_policy={} dangerous_bypass={} sandbox_permissions={:?} extra_config_overrides={:?}",
                printable_or_default(&self.config.agent.codex_acp.command, "codex-acp"),
                self.config.agent.codex_acp.args,
                printable_or_default(&self.config.agent.codex_acp.codex_command, "codex"),
                printable_or_default(&self.config.agent.codex_acp.sandbox_mode, "<default>"),
                printable_or_default(&self.config.agent.codex_acp.approval_policy, "<default>"),
                self.config
                    .agent
                    .codex_acp
                    .dangerously_bypass_approvals_and_sandbox,
                self.config.agent.codex_acp.sandbox_permissions,
                self.config.agent.codex_acp.extra_config_overrides,
            ),
            AgentRunnerKind::FunctionCalling => tracing::info!(
                "[Startup/{channel}] dialog.engine=function_calling llm.provider={} llm.model={} max_iterations={}",
                printable_or_default(llm_provider, "<empty>"),
                llm_model,
                self.config.agent.max_iterations
            ),
            AgentRunnerKind::Unknown => tracing::warn!(
                "[Startup/{channel}] dialog.engine=unknown(agent.runner={}) fallback=function_calling llm.provider={} llm.model={}",
                printable_or_default(self.config.agent.runner.trim(), "<empty>"),
                printable_or_default(llm_provider, "<empty>"),
                llm_model
            ),
        }

        if self.auxiliary_llm.is_some() {
            let (aux_provider, aux_model) = self.auxiliary_provider_hint();
            tracing::info!(
                "[Startup/{channel}] session.compression.engine=llm provider={} model={} threshold=40 retain_recent=4",
                printable_or_default(&aux_provider, "<empty>"),
                printable_or_default(&aux_model, "<empty>")
            );
        } else {
            tracing::warn!(
                "[Startup/{channel}] session.compression.engine=disabled reason=llm_provider_unavailable"
            );
        }

        if self.llm_audit.is_some() {
            tracing::info!(
                "[Startup/{channel}] llm.audit.path={} retention_days={}",
                self.config.storage.llm_audit_db_path,
                self.config.storage.llm_audit_retention_days
            );
        } else {
            tracing::warn!("[Startup/{channel}] llm.audit=disabled");
        }

        tracing::info!(
            "[Startup/{channel}] session.runtime_backend={} session.shadow_sqlite.enabled={} session.shadow_sqlite.path={}",
            self.config.storage.session_runtime_backend,
            self.config.storage.session_sqlite_shadow_write_enabled,
            self.config.storage.session_sqlite_db_path
        );
    }

    /// 记录“收到用户消息”事件（统一日志格式）
    pub fn log_message_received(
        &self,
        channel: &str,
        user_id: &str,
        channel_target: &str,
        session_id: &str,
        input: &str,
        extra: Option<&str>,
        message_id: Option<&str>,
    ) {
        let preview = truncate_for_log(input, 120);
        let extra = extra.unwrap_or("-");
        tracing::info!(
            message_id = %message_id.unwrap_or("-"),
            state = "received",
            "[MsgFlow/{channel}] recv user={} target={} session={} input.chars={} input.preview=\"{}\" extra={}",
            printable_or_default(user_id, "<empty>"),
            printable_or_default(channel_target, "<empty>"),
            printable_or_default(session_id, "<empty>"),
            input.chars().count(),
            preview,
            extra
        );
    }

    /// 记录“处理中某一步”事件
    pub fn log_message_step(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        step: &str,
        detail: &str,
        message_id: Option<&str>,
        state_override: Option<&str>,
    ) {
        let state = if let Some(s) = state_override {
            s
        } else if step.contains("agent_spawned") {
            "agent_spawned"
        } else if step.contains("agent_active") {
            "agent_active"
        } else if step.contains("agent_iterating") {
            "agent_iterating"
        } else {
            "step"
        };

        tracing::info!(
            message_id = %message_id.unwrap_or("-"),
            state = state,
            "[MsgFlow/{channel}] step={} user={} session={} detail={}",
            printable_or_default(step, "<unknown>"),
            printable_or_default(user_id, "<empty>"),
            printable_or_default(session_id, "<empty>"),
            printable_or_default(detail, "-")
        );
    }

    /// 记录“消息处理完成”事件
    pub fn log_message_finished(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        response: &AgentResponse,
        elapsed_ms: u128,
        message_id: Option<&str>,
    ) {
        let tool_summary = summarize_tools(&response.tool_calls_made);
        tracing::info!(
            message_id = %message_id.unwrap_or("-"),
            state = "finished",
            "[MsgFlow/{channel}] done user={} session={} success={} elapsed_ms={} iterations={} tools={} reply.chars={}",
            printable_or_default(user_id, "<empty>"),
            printable_or_default(session_id, "<empty>"),
            response.success,
            elapsed_ms,
            response.iterations,
            tool_summary,
            response.content.chars().count(),
        );
    }

    /// 记录“消息处理失败”事件
    pub fn log_message_failed(
        &self,
        channel: &str,
        user_id: &str,
        session_id: &str,
        error: &str,
        elapsed_ms: u128,
        message_id: Option<&str>,
    ) {
        tracing::error!(
            message_id = %message_id.unwrap_or("-"),
            state = "failed",
            "[MsgFlow/{channel}] failed user={} session={} elapsed_ms={} error=\"{}\"",
            printable_or_default(user_id, "<empty>"),
            printable_or_default(session_id, "<empty>"),
            elapsed_ms,
            truncate_for_log(error, 280)
        );
    }
}

/// 空串时退化成 `default`,否则 `trim()` 去掉两端空白后返回。
/// 日志里常见场景:config 里可能漏填 llm.model,避免打出 `llm.model= ` 这种歧义。
pub(super) fn printable_or_default<'a>(value: &'a str, default: &'a str) -> &'a str {
    let v = value.trim();
    if v.is_empty() { default } else { v }
}

/// 把字符串按 char 个数(不是 byte)截断成最多 `max_chars` 个,并把
/// `\n` 转成 `\\n`、丢掉 `\r`,方便塞进单行 tracing 字段。
/// 完全为空时返回 `"-"`,避免日志里出现 `detail=""`。
pub(super) fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in value.chars().enumerate() {
        if idx >= max_chars {
            out.push_str("...");
            break;
        }
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(ch),
        }
    }
    if out.is_empty() { "-".to_string() } else { out }
}

/// 一次 agent run 里调了哪些 tool、一共多少次,压成一个短字符串放到
/// `done` 日志里。`BTreeSet` 保证名字排序稳定,方便跨多条日志 diff。
pub(super) fn summarize_tools(tool_calls: &[hone_core::agent::ToolCallMade]) -> String {
    if tool_calls.is_empty() {
        return "none".to_string();
    }
    let mut names = std::collections::BTreeSet::new();
    for call in tool_calls {
        names.insert(call.name.as_str());
    }
    format!(
        "{}({})",
        tool_calls.len(),
        names.into_iter().collect::<Vec<_>>().join(",")
    )
}
