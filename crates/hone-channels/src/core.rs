//! HoneBotCore — 配置/工具/会话 runner 工厂
//!
//! 各渠道共享的 Bot 核心配置。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use hone_core::agent::AgentResponse;
use hone_core::config::HoneConfig;
use hone_core::{ActorIdentity, LlmAuditSink};
use hone_llm::{LlmProvider, OpenAiCompatibleProvider, OpenRouterProvider};
use hone_memory::{
    CompanyProfileStorage, ConversationQuotaStorage, CronJobStorage, LlmAuditStorage,
    SessionStorage,
};
use hone_scheduler::{HoneScheduler, SchedulerEvent};
use hone_tools::{
    CronJobTool, DeepResearchTool, DiscoverSkillsTool, LoadSkillTool, ToolExecutionGuard,
    ToolRegistry,
};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde_json::json;
use subtle::ConstantTimeEq;
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::runners::{
    AgentRunner, CodexAcpRunner, CodexCliReasoningRunner, FunctionCallingReasoningRunner,
    GeminiAcpRunner, GeminiCliRunner, MultiAgentRunner, OpencodeAcpRunner, RunnerTimeouts,
};
use crate::sandbox::sandbox_base_dir;
use crate::session_compactor::SessionCompactor;

pub const REGISTER_ADMIN_INTERCEPT_PREFIX: &str = "/register-admin";
pub const REGISTER_ADMIN_INTERCEPT_ACK: &str = "已将当前 identity 升级为管理员。";
pub const REGISTER_ADMIN_INTERCEPT_DENY_ACK: &str =
    "管理员注册失败：当前 identity 不在 admins 白名单中。";
pub const REGISTER_ADMIN_INTERCEPT_DISABLED_ACK: &str =
    "管理员注册失败：当前未配置 runtime 管理员注册口令。";
pub const REGISTER_ADMIN_INTERCEPT_INVALID_ACK: &str = "管理员注册失败：口令无效。";
const REPORT_INTERCEPT_PREFIX: &str = "/report";
const REPORT_PROGRESS_COMMAND: &str = "进度";
const REPORT_PROGRESS_COMMAND_ALIAS: &str = "progress";
const REPORT_WORKFLOW_ID: &str = "company_report";
const REPORT_DEFAULT_MODE: &str = "完整跑完";
const REPORT_DEFAULT_RESEARCH_TOPIC: &str = "新闻";

#[derive(Debug, Clone)]
pub struct CompactSessionOutcome {
    pub compacted: bool,
    pub summary: Option<String>,
}

/// Bot 核心 — 持有所有共享依赖
pub struct HoneBotCore {
    pub config: HoneConfig,
    pub llm: Option<Arc<dyn LlmProvider>>,
    pub auxiliary_llm: Option<Arc<dyn LlmProvider>>,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub session_storage: SessionStorage,
    pub conversation_quota_storage: ConversationQuotaStorage,
    workflow_runner_http: reqwest::Client,
    pub company_profile_storage: CompanyProfileStorage,
    runtime_admin_overrides: RwLock<HashSet<ActorIdentity>>,
}

impl HoneBotCore {
    /// 从配置创建
    pub fn new(config: HoneConfig) -> Self {
        let session_storage = SessionStorage::from_storage_config(&config.storage);
        let conversation_quota_storage =
            ConversationQuotaStorage::new(&config.storage.conversation_quota_dir)
                .expect("failed to initialize conversation quota storage");
        let company_profile_storage = CompanyProfileStorage::new(sandbox_base_dir());
        let llm = Self::create_llm_provider(&config);
        let auxiliary_llm = Self::create_auxiliary_llm_provider(&config);
        let llm_audit = Self::create_llm_audit_sink(&config);
        let workflow_runner_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|err| {
                tracing::warn!("failed to create workflow runner HTTP client: {}", err);
                reqwest::Client::new()
            });

        Self {
            config,
            llm,
            auxiliary_llm,
            llm_audit,
            session_storage,
            conversation_quota_storage,
            workflow_runner_http,
            company_profile_storage,
            runtime_admin_overrides: RwLock::new(HashSet::new()),
        }
    }

    /// 从配置文件创建
    pub fn from_config_file(path: &str) -> hone_core::HoneResult<Self> {
        let config = HoneConfig::from_file(path)?;
        Ok(Self::new(config))
    }

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

        match self.config.agent.runner.trim() {
            "gemini_cli" => tracing::info!(
                "[Startup/{channel}] dialog.engine=gemini_cli command=gemini model.source=gemini-cli(profile/default)"
            ),
            "gemini_acp" => tracing::info!(
                "[Startup/{channel}] dialog.engine=gemini_acp transport=stdio-jsonrpc command={} args={:?} model={} api_key_env={}",
                printable_or_default(&self.config.agent.gemini_acp.command, "gemini"),
                self.config.agent.gemini_acp.args,
                printable_or_default(&self.config.agent.gemini_acp.model, "<gemini-default>"),
                printable_or_default(&self.config.agent.gemini_acp.api_key_env, "GEMINI_API_KEY"),
            ),
            "codex_cli" => tracing::info!(
                "[Startup/{channel}] dialog.engine=codex_cli command=codex exec model={}",
                printable_or_default(&self.config.agent.codex_model, "<codex-cli-default>")
            ),
            "opencode_acp" => tracing::info!(
                "[Startup/{channel}] dialog.engine=opencode_acp transport=stdio-jsonrpc command={} args={:?}",
                printable_or_default(&self.config.agent.opencode.command, "opencode"),
                self.config.agent.opencode.args
            ),
            "multi-agent" => tracing::info!(
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
            "codex_acp" => tracing::info!(
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
            "function_calling" => tracing::info!(
                "[Startup/{channel}] dialog.engine=function_calling llm.provider={} llm.model={} max_iterations={}",
                printable_or_default(llm_provider, "<empty>"),
                llm_model,
                self.config.agent.max_iterations
            ),
            other => tracing::warn!(
                "[Startup/{channel}] dialog.engine=unknown(agent.runner={}) fallback=function_calling llm.provider={} llm.model={}",
                printable_or_default(other, "<empty>"),
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

    /// 创建 LLM Provider
    fn create_llm_provider(config: &HoneConfig) -> Option<Arc<dyn LlmProvider>> {
        match config.llm.provider.as_str() {
            _ => {
                // Default to OpenRouter
                match OpenRouterProvider::from_config(config) {
                    Ok(provider) => Some(Arc::new(provider)),
                    Err(e) => {
                        tracing::warn!("Failed to create OpenRouter provider: {}", e);
                        None
                    }
                }
            }
        }
    }

    fn create_auxiliary_llm_provider(config: &HoneConfig) -> Option<Arc<dyn LlmProvider>> {
        if config.llm.auxiliary.is_configured() {
            let api_key = config.llm.auxiliary.resolved_api_key();
            if api_key.trim().is_empty() {
                tracing::warn!("Failed to create auxiliary provider: auxiliary API key is empty");
                return None;
            }

            return match OpenAiCompatibleProvider::new(
                &api_key,
                config.llm.auxiliary.base_url.trim(),
                config.llm.auxiliary.model.trim(),
                config.llm.auxiliary.timeout,
                config.llm.auxiliary.max_tokens as u16,
            ) {
                Ok(provider) => Some(Arc::new(provider)),
                Err(err) => {
                    tracing::warn!("Failed to create auxiliary provider: {}", err);
                    None
                }
            };
        }

        Self::create_llm_provider(config)
    }

    pub fn auxiliary_model_name(&self) -> String {
        let configured = self.config.llm.auxiliary.model.trim();
        if !configured.is_empty() {
            configured.to_string()
        } else {
            self.config.llm.openrouter.auxiliary_model().to_string()
        }
    }

    pub fn auxiliary_provider_hint(&self) -> (String, String) {
        if self.config.llm.auxiliary.is_configured() {
            ("openai-compatible".to_string(), self.auxiliary_model_name())
        } else {
            (
                self.config.llm.provider.clone(),
                self.auxiliary_model_name(),
            )
        }
    }

    fn effective_multi_agent_search_config(&self) -> hone_core::config::MultiAgentSearchConfig {
        let mut search_config = self.config.agent.multi_agent.search.clone();
        if search_config.api_key.trim().is_empty() {
            let fallback_key = self.config.llm.auxiliary.resolved_api_key();
            if !fallback_key.trim().is_empty() {
                search_config.api_key = fallback_key;
            }
        }
        search_config
    }

    fn effective_multi_agent_answer_max_tool_calls(&self) -> u32 {
        self.config.agent.multi_agent.answer.max_tool_calls
    }

    fn create_llm_audit_sink(config: &HoneConfig) -> Option<Arc<dyn LlmAuditSink>> {
        if !config.storage.llm_audit_enabled {
            return None;
        }

        match LlmAuditStorage::new(
            &config.storage.llm_audit_db_path,
            config.storage.llm_audit_retention_days,
        ) {
            Ok(storage) => Some(Arc::new(storage)),
            Err(err) => {
                tracing::warn!("Failed to create LLM audit storage: {}", err);
                None
            }
        }
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

    /// 检查某用户在指定渠道是否为管理员
    ///
    /// - channel 传 "imessage" 时与 admins.imessage_handles 匹配
    /// - channel 传 "feishu"   时与 admins.feishu_emails / feishu_mobiles / feishu_open_ids 匹配
    /// - channel 传 "discord"  时与 admins.discord_user_ids  匹配
    pub fn is_admin(&self, user_id: &str, channel: &str) -> bool {
        if user_id.is_empty() {
            return false;
        }
        let admin_cfg = &self.config.admins;
        match channel {
            "imessage" => admin_cfg
                .imessage_handles
                .iter()
                .any(|h| !h.is_empty() && h == user_id),
            "feishu" => {
                admin_cfg
                    .feishu_emails
                    .iter()
                    .any(|email| !email.is_empty() && email == user_id)
                    || admin_cfg
                        .feishu_mobiles
                        .iter()
                        .any(|mobile| !mobile.is_empty() && mobile == user_id)
                    || admin_cfg
                        .feishu_open_ids
                        .iter()
                        .any(|open_id| !open_id.is_empty() && open_id == user_id)
            }
            "discord" => admin_cfg
                .discord_user_ids
                .iter()
                .any(|id| !id.is_empty() && id == user_id),
            "cli" => true,
            _ => false,
        }
    }

    pub fn is_admin_actor(&self, actor: &ActorIdentity) -> bool {
        self.runtime_admin_overrides
            .read()
            .map(|overrides| overrides.contains(actor))
            .unwrap_or(false)
            || self.is_admin(&actor.user_id, &actor.channel)
    }

    pub fn try_intercept_admin_registration(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String> {
        let Some(passphrase) = parse_admin_registration_passphrase(input) else {
            return None;
        };

        if !self.is_admin(&actor.user_id, &actor.channel) {
            tracing::warn!(
                "[HoneBotCore] runtime_admin_override denied actor={} reason=not_whitelisted",
                actor.session_id()
            );
            return Some(REGISTER_ADMIN_INTERCEPT_DENY_ACK.to_string());
        }

        let expected = self
            .config
            .admins
            .resolved_runtime_admin_registration_passphrase();
        if expected.is_empty() {
            tracing::warn!(
                "[HoneBotCore] runtime_admin_override denied actor={} reason=passphrase_disabled",
                actor.session_id()
            );
            return Some(REGISTER_ADMIN_INTERCEPT_DISABLED_ACK.to_string());
        }

        if !constant_time_str_eq(&passphrase, &expected) {
            tracing::warn!(
                "[HoneBotCore] runtime_admin_override denied actor={} reason=invalid_passphrase",
                actor.session_id()
            );
            return Some(REGISTER_ADMIN_INTERCEPT_INVALID_ACK.to_string());
        }

        let inserted = self
            .runtime_admin_overrides
            .write()
            .map(|mut overrides| overrides.insert(actor.clone()))
            .unwrap_or(false);

        tracing::warn!(
            "[HoneBotCore] runtime_admin_override actor={} inserted={}",
            actor.session_id(),
            inserted
        );
        Some(REGISTER_ADMIN_INTERCEPT_ACK.to_string())
    }

    pub async fn try_handle_intercept_command(
        &self,
        actor: &ActorIdentity,
        input: &str,
    ) -> Option<String> {
        if let Some(reply) = self.try_intercept_admin_registration(actor, input) {
            return Some(reply);
        }

        match parse_report_intercept(input) {
            Some(ReportIntercept::Start { company_name }) => {
                Some(self.handle_report_start(actor, &company_name).await)
            }
            Some(ReportIntercept::Progress) => Some(self.handle_report_progress(actor).await),
            None => None,
        }
    }

    async fn handle_report_start(&self, actor: &ActorIdentity, company_name: &str) -> String {
        let Some(base_url) = self.workflow_runner_base_url() else {
            return "未配置本地 workflow runner 地址，暂时无法启动研报任务。".to_string();
        };

        let validate_code = self.config.web.resolved_local_workflow_validate_code();
        if validate_code.is_empty() {
            return "未配置本地 workflow runner validate code，暂时无法启动研报任务。".to_string();
        }

        let request_body = build_report_run_input(company_name, &validate_code);
        let url = format!("{base_url}/api/runs");
        let response = match self
            .workflow_runner_request(
                Method::POST,
                &url,
                Some(json!({
                    "workflowId": REPORT_WORKFLOW_ID,
                    "input": request_body,
                    "promptOverrides": {},
                })),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    "[HoneBotCore] report start request failed actor={} error={}",
                    actor.session_id(),
                    err
                );
                return format!("研报任务启动失败：无法连接本地 workflow runner（{err}）。");
            }
        };

        if response.status == StatusCode::CONFLICT {
            let conflict = serde_json::from_str::<WorkflowConflictResponse>(&response.body).ok();
            if let Some(active_run_id) = conflict
                .as_ref()
                .and_then(|value| value.active_run_id.as_deref())
            {
                if let Ok(progress) = self.fetch_report_progress_by_run_id(active_run_id).await {
                    return format!(
                        "已有研报任务正在运行中：{}",
                        format_progress_message(&progress)
                    );
                }
            }

            let detail = conflict
                .and_then(|value| value.error)
                .unwrap_or_else(|| "已有研报任务正在运行中。".to_string());
            return format!("研报任务未重复启动：{detail}");
        }

        if !response.status.is_success() {
            let status = response.status;
            let detail = truncate_for_log(response.body.trim(), 160);
            return format!("研报任务启动失败：{status} {detail}");
        }

        match serde_json::from_str::<WorkflowRunCreatedResponse>(&response.body) {
            Ok(payload) => format!(
                "已启动公司研报：{}。研究倾向默认使用“{}”，任务正在运行中（run_id={}）。可发送 `/report 进度` 查看进度。",
                company_name.trim(),
                REPORT_DEFAULT_RESEARCH_TOPIC,
                payload.id
            ),
            Err(err) => format!("研报任务已提交，但解析启动响应失败：{err}"),
        }
    }

    async fn handle_report_progress(&self, actor: &ActorIdentity) -> String {
        let Some(base_url) = self.workflow_runner_base_url() else {
            return "未配置本地 workflow runner 地址，暂时无法查询研报进度。".to_string();
        };

        let url = format!("{base_url}/api/runs?workflowId={REPORT_WORKFLOW_ID}&limit=1");
        let response = match self.workflow_runner_request(Method::GET, &url, None).await {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    "[HoneBotCore] report progress request failed actor={} error={}",
                    actor.session_id(),
                    err
                );
                return format!("查询研报进度失败：无法连接本地 workflow runner（{err}）。");
            }
        };

        if !response.status.is_success() {
            let status = response.status;
            let detail = truncate_for_log(response.body.trim(), 160);
            return format!("查询研报进度失败：{status} {detail}");
        }

        let payload = match serde_json::from_str::<WorkflowRunListResponse>(&response.body) {
            Ok(payload) => payload,
            Err(err) => return format!("查询研报进度失败：响应解析错误（{err}）。"),
        };

        let Some(run) = payload.runs.into_iter().next() else {
            return "当前还没有可查询的研报任务。可直接发送 `/report 公司名` 启动。".to_string();
        };

        if run.status == "running" {
            match self.fetch_report_progress_by_run_id(&run.id).await {
                Ok(progress) => format_progress_message(&progress),
                Err(err) => format!(
                    "研报任务正在运行中（run_id={}），但拉取实时进度失败：{}",
                    run.id, err
                ),
            }
        } else {
            format_recent_report_message(&run)
        }
    }

    fn workflow_runner_base_url(&self) -> Option<String> {
        let base = self.config.web.local_workflow_api_base.trim();
        if base.is_empty() {
            None
        } else {
            Some(base.trim_end_matches('/').to_string())
        }
    }

    async fn fetch_report_progress_by_run_id(
        &self,
        run_id: &str,
    ) -> Result<WorkflowProgressEnvelope, String> {
        let base_url = self
            .workflow_runner_base_url()
            .ok_or_else(|| "未配置本地 workflow runner 地址".to_string())?;
        let url = format!("{base_url}/api/runs/{run_id}/progress");
        let response = self
            .workflow_runner_request(Method::GET, &url, None)
            .await
            .map_err(|err| err.to_string())?;
        if !response.status.is_success() {
            let status = response.status;
            let detail = truncate_for_log(response.body.trim(), 160);
            return Err(format!("{status} {detail}"));
        }
        serde_json::from_str::<WorkflowProgressEnvelope>(&response.body)
            .map_err(|err| err.to_string())
    }

    async fn workflow_runner_request(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<WorkflowRunnerHttpResponse, String> {
        let mut request = self.workflow_runner_http.request(method.clone(), url);
        if let Some(payload) = body.as_ref() {
            request = request.json(payload);
        }

        match request.send().await {
            Ok(response) => WorkflowRunnerHttpResponse::from_reqwest(response).await,
            Err(err) => {
                if should_fallback_to_curl(url) {
                    tracing::warn!(
                        "[HoneBotCore] workflow runner reqwest failed, falling back to curl url={} error={:?}",
                        url,
                        err
                    );
                    self.workflow_runner_request_via_curl(method, url, body)
                        .await
                } else {
                    Err(err.to_string())
                }
            }
        }
    }

    async fn workflow_runner_request_via_curl(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<WorkflowRunnerHttpResponse, String> {
        let mut command = Command::new("curl");
        command.arg("-sS");
        command.arg("-X").arg(method.as_str());
        command.arg(url);
        command.arg("-H").arg("Accept: application/json");
        command.arg("-w").arg("\n__HONE_STATUS__:%{http_code}");

        if let Some(payload) = body {
            let payload_text = serde_json::to_string(&payload).map_err(|err| err.to_string())?;
            command.arg("-H").arg("Content-Type: application/json");
            command.arg("--data").arg(payload_text);
        }

        let output = command.output().await.map_err(|err| err.to_string())?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "curl exited with status {}: {}",
                output.status,
                truncate_for_log(stderr.trim(), 240)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let (body_text, status_text) = stdout
            .rsplit_once("\n__HONE_STATUS__:")
            .ok_or_else(|| "curl response missing HTTP status marker".to_string())?;
        let status_code = status_text
            .trim()
            .parse::<u16>()
            .map_err(|err| format!("invalid curl status code: {err}"))?;
        let status = StatusCode::from_u16(status_code)
            .map_err(|err| format!("unsupported HTTP status code {status_code}: {err}"))?;

        Ok(WorkflowRunnerHttpResponse {
            status,
            body: body_text.to_string(),
        })
    }

    /// 创建工具注册表
    pub fn create_tool_registry(
        &self,
        actor: Option<&ActorIdentity>,
        channel_target: &str,
        allow_cron: bool,
    ) -> ToolRegistry {
        let guard = ToolExecutionGuard::from_config(&self.config.security.tool_guard);
        let mut registry = ToolRegistry::new_with_guard(guard);

        let skills_dir = self.configured_system_skills_dir();
        let custom_skills_dir = self.configured_custom_skills_dir();
        let skill_registry_path = self.configured_skill_registry_path();

        let dirs = vec![skills_dir.clone(), custom_skills_dir.clone()];

        registry.register(Box::new(
            LoadSkillTool::new(dirs).with_registry_path(skill_registry_path.clone()),
        ));
        registry.register(Box::new(DiscoverSkillsTool::new(
            skills_dir.clone(),
            custom_skills_dir.clone(),
            skill_registry_path.clone(),
        )));
        registry.register(Box::new(hone_tools::skill_tool::SkillTool::new(
            skills_dir,
            custom_skills_dir,
            skill_registry_path,
        )));

        if allow_cron {
            let admin_bypass = actor
                .map(|actor| self.is_admin_actor(actor))
                .unwrap_or(false);
            registry.register(Box::new(CronJobTool::new(
                &self.config.storage.cron_jobs_dir,
                actor.cloned(),
                channel_target,
                admin_bypass,
            )));
        } else {
            tracing::info!(
                "[HoneBotCore] cron_job disabled for channel_target={}",
                printable_or_default(channel_target, "<empty>")
            );
        }

        // 注册持仓管理工具
        let portfolio_actor = actor.cloned().unwrap_or_else(|| {
            ActorIdentity::new("system", "system", None::<String>)
                .expect("failed to create system actor")
        });
        registry.register(Box::new(hone_tools::PortfolioTool::new(
            &self.config.storage.portfolio_dir,
            portfolio_actor,
        )));

        if let Some(actor) = actor.cloned() {
            let sandbox_base = sandbox_base_dir();
            registry.register(Box::new(hone_tools::LocalListFilesTool::new(
                sandbox_base.clone(),
                actor.clone(),
            )));
            registry.register(Box::new(hone_tools::LocalSearchFilesTool::new(
                sandbox_base.clone(),
                actor.clone(),
            )));
            registry.register(Box::new(hone_tools::LocalReadFileTool::new(
                sandbox_base,
                actor,
            )));
        }

        // 注册金融数据获取工具
        registry.register(Box::new(hone_tools::DataFetchTool::from_config(
            &self.config,
        )));

        // 注册网络搜索工具
        registry.register(Box::new(hone_tools::WebSearchTool::from_config(
            &self.config,
        )));

        // deep_research 是核心分析工具，对所有用户开放
        registry.register(Box::new(DeepResearchTool::from_env()));
        tracing::info!("[HoneBotCore] 已注册通用工具 deep_research");

        // 管理员专属工具（仅 restart_hone 需要管理员权限）
        if let Some(actor) = actor.filter(|actor| self.is_admin_actor(actor)) {
            let project_root =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            registry.register(Box::new(hone_tools::RestartHoneTool::new(project_root)));
            tracing::info!(
                "[HoneBotCore] 管理员 {} 已注册专属工具 (restart_hone)",
                actor.user_id
            );
        }

        registry
    }

    pub fn configured_system_skills_dir(&self) -> PathBuf {
        self.config
            .extra
            .get("skills_dir")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./skills"))
    }

    pub fn configured_custom_skills_dir(&self) -> PathBuf {
        self.configured_data_dir().join("custom_skills")
    }

    pub fn configured_data_dir(&self) -> PathBuf {
        if let Ok(root) = std::env::var("HONE_DATA_DIR") {
            return PathBuf::from(root);
        }

        PathBuf::from(&self.config.storage.sessions_dir)
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./data"))
    }

    pub fn configured_runtime_dir(&self) -> PathBuf {
        hone_core::runtime_heartbeat_dir(&self.config)
    }

    pub fn configured_skill_registry_path(&self) -> PathBuf {
        self.configured_runtime_dir().join("skill_registry.json")
    }

    /// 创建调度器及其事件接收端。
    ///
    /// `channels`：本调度器负责的渠道列表，只触发 `job.channel` 在列表中的任务。
    /// 传入空 Vec 则处理所有渠道（通常不使用）。
    ///
    /// 示例：
    /// - `hone-console-page`：`vec!["imessage", "web"]`
    /// - `hone-feishu`：`vec!["feishu"]`
    /// - `hone-telegram`：`vec!["telegram"]`
    pub fn create_scheduler(
        &self,
        channels: Vec<String>,
    ) -> (HoneScheduler, mpsc::Receiver<SchedulerEvent>) {
        let storage = Arc::new(self.cron_job_storage());
        let (tx, rx) = mpsc::channel(64);
        (HoneScheduler::new(storage, tx, channels), rx)
    }

    pub fn cron_job_storage(&self) -> CronJobStorage {
        CronJobStorage::with_sqlite(
            &self.config.storage.cron_jobs_dir,
            &self.config.storage.session_sqlite_db_path,
        )
    }

    /// 创建 Agent runner 实例。
    ///
    /// `AgentSession` 应通过 runner，而不是直接感知底层 provider/CLI 分支。
    ///
    /// 返回 `Err(message)` 表示配置不满足要求（例如 function_calling 引擎要求 LLM Provider 已配置）。
    pub fn create_runner(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
    ) -> Result<Box<dyn AgentRunner>, String> {
        self.create_runner_with_model_override(system_prompt, tool_registry, None)
    }

    pub fn create_runner_with_model_override(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
        model_override: Option<&str>,
    ) -> Result<Box<dyn AgentRunner>, String> {
        let runner = self.config.agent.runner.trim();
        let runner_timeouts = RunnerTimeouts {
            step: self.config.agent.step_timeout(),
            overall: self.config.agent.overall_timeout(),
        };
        match runner {
            "gemini_cli" => Ok(Box::new(GeminiCliRunner::new(
                system_prompt.to_string(),
                Arc::new(tool_registry),
                runner_timeouts,
            ))),
            "gemini_acp" => Ok(Box::new(GeminiAcpRunner::new(
                self.config.agent.gemini_acp.clone(),
                runner_timeouts,
            ))),
            "codex_cli" => Ok(Box::new(CodexCliReasoningRunner::new(
                system_prompt.to_string(),
                Some(self.config.agent.codex_model.clone()),
                Arc::new(tool_registry),
                self.llm_audit.clone(),
            ))),
            "codex_acp" => Ok(Box::new(CodexAcpRunner::new(
                self.config.agent.codex_acp.clone(),
                runner_timeouts,
            ))),
            "function_calling" => {
                let llm = self.llm.clone().ok_or_else(|| {
                    "AI 服务未配置（openrouter.api_key 为空），无法使用 function_calling 引擎。\
请在 config.yaml 中填写有效的 API Key 后重启服务。"
                        .to_string()
                })?;
                Ok(Box::new(FunctionCallingReasoningRunner::new(
                    llm,
                    Arc::new(tool_registry),
                    system_prompt.to_string(),
                    self.config.agent.max_iterations,
                    self.llm_audit.clone(),
                )))
            }
            "opencode_acp" => {
                let mut opencode_config = self.config.agent.opencode.clone();
                if let Some(model_override) =
                    model_override.filter(|value| !value.trim().is_empty())
                {
                    opencode_config.model = model_override.trim().to_string();
                    opencode_config.variant = String::new();
                }
                let hone_manages_opencode_route = !opencode_config.model.trim().is_empty()
                    || !opencode_config.variant.trim().is_empty()
                    || !opencode_config.api_base_url.trim().is_empty()
                    || !opencode_config.api_key.trim().is_empty();
                if hone_manages_opencode_route && opencode_config.api_key.trim().is_empty() {
                    let pool = self.config.llm.openrouter.effective_key_pool();
                    if let Some(key) = pool.first() {
                        opencode_config.openrouter_api_key = Some(key.to_string());
                    }
                }
                Ok(Box::new(OpencodeAcpRunner::new(
                    opencode_config,
                    runner_timeouts,
                )))
            }
            "multi-agent" => {
                let pool = self.config.llm.openrouter.effective_key_pool();
                let mut answer_config = self.config.agent.opencode.clone();
                let multi_answer = &self.config.agent.multi_agent.answer;
                if !multi_answer.api_base_url.trim().is_empty() {
                    answer_config.api_base_url = multi_answer.api_base_url.trim().to_string();
                }
                if !multi_answer.api_key.trim().is_empty() {
                    answer_config.api_key = multi_answer.api_key.trim().to_string();
                }
                if !multi_answer.model.trim().is_empty() {
                    answer_config.model = multi_answer.model.trim().to_string();
                }
                if !multi_answer.variant.trim().is_empty() {
                    answer_config.variant = multi_answer.variant.trim().to_string();
                }
                if let Some(model_override) =
                    model_override.filter(|value| !value.trim().is_empty())
                {
                    answer_config.model = model_override.trim().to_string();
                    answer_config.variant = String::new();
                }
                let hone_manages_answer_route = !answer_config.model.trim().is_empty()
                    || !answer_config.variant.trim().is_empty()
                    || !answer_config.api_base_url.trim().is_empty()
                    || !answer_config.api_key.trim().is_empty();
                answer_config.openrouter_api_key =
                    if hone_manages_answer_route && answer_config.api_key.trim().is_empty() {
                        pool.first().map(|value| value.to_string())
                    } else {
                        None
                    };

                Ok(Box::new(MultiAgentRunner::new(
                    system_prompt.to_string(),
                    self.effective_multi_agent_search_config(),
                    answer_config,
                    runner_timeouts,
                    self.effective_multi_agent_answer_max_tool_calls(),
                    Arc::new(tool_registry),
                    self.llm_audit.clone(),
                )))
            }
            other => {
                tracing::warn!(
                    "[HoneBotCore] unknown runner={}, fallback to function_calling",
                    printable_or_default(other, "<empty>")
                );
                let llm = self.llm.clone().ok_or_else(|| {
                    "AI 服务未配置（openrouter.api_key 为空），无法使用 function_calling 引擎。\
请在 config.yaml 中填写有效的 API Key 后重启服务。"
                        .to_string()
                })?;
                Ok(Box::new(FunctionCallingReasoningRunner::new(
                    llm,
                    Arc::new(tool_registry),
                    system_prompt.to_string(),
                    self.config.agent.max_iterations,
                    self.llm_audit.clone(),
                )))
            }
        }
    }

    pub fn create_actor(
        channel: &str,
        user_id: &str,
        channel_scope: Option<&str>,
    ) -> hone_core::HoneResult<ActorIdentity> {
        ActorIdentity::new(channel, user_id, channel_scope)
    }

    /// 检查并压缩会话历史
    pub async fn maybe_compress_session(&self, session_id: &str) -> hone_core::HoneResult<()> {
        let _ = self
            .compact_session(session_id, "auto", false, None)
            .await?;
        Ok(())
    }

    pub async fn compact_session(
        &self,
        session_id: &str,
        trigger: &str,
        force: bool,
        user_instructions: Option<&str>,
    ) -> hone_core::HoneResult<CompactSessionOutcome> {
        SessionCompactor::new(self)
            .compact_session(session_id, trigger, force, user_instructions)
            .await
    }
}

pub fn runtime_config_path() -> String {
    std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string())
}

pub fn load_runtime_config() -> hone_core::HoneResult<(HoneConfig, String)> {
    let config_path = runtime_config_path();
    let mut config = HoneConfig::from_file(&config_path)?;
    let data_dir = std::env::var_os("HONE_DATA_DIR").map(PathBuf::from);
    let skills_dir = std::env::var_os("HONE_SKILLS_DIR").map(PathBuf::from);
    config.apply_runtime_overrides(
        data_dir.as_deref(),
        skills_dir.as_deref(),
        Some(PathBuf::from(&config_path).as_path()),
    );
    config.ensure_runtime_dirs();
    Ok((config, config_path))
}

fn printable_or_default<'a>(value: &'a str, default: &'a str) -> &'a str {
    let v = value.trim();
    if v.is_empty() { default } else { v }
}

fn truncate_for_log(value: &str, max_chars: usize) -> String {
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

fn summarize_tools(tool_calls: &[hone_core::agent::ToolCallMade]) -> String {
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReportIntercept {
    Start { company_name: String },
    Progress,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowRunCreatedResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowConflictResponse {
    error: Option<String>,
    active_run_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowRunListResponse {
    runs: Vec<WorkflowRunSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowRunSummary {
    id: String,
    workflow_id: String,
    workflow_name: Option<String>,
    status: String,
    ended_at: Option<String>,
    error: Option<String>,
    progress: Option<WorkflowProgressSnapshot>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowProgressEnvelope {
    id: String,
    progress: WorkflowProgressSnapshot,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowProgressSnapshot {
    total_nodes: u32,
    terminal_nodes: u32,
    running_nodes: u32,
    pending_nodes: u32,
    percent: f64,
    #[serde(default)]
    active_nodes: Vec<WorkflowActiveNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowActiveNode {
    workflow_name: Option<String>,
    title: Option<String>,
    id: String,
}

#[derive(Debug, Clone)]
struct WorkflowRunnerHttpResponse {
    status: StatusCode,
    body: String,
}

impl WorkflowRunnerHttpResponse {
    async fn from_reqwest(response: reqwest::Response) -> Result<Self, String> {
        let status = response.status();
        let body = response.text().await.map_err(|err| err.to_string())?;
        Ok(Self { status, body })
    }
}

fn build_report_run_input(company_name: &str, validate_code: &str) -> serde_json::Value {
    json!({
        "companyName": company_name.trim(),
        "genPost": REPORT_DEFAULT_MODE,
        "validateCode": validate_code,
        "news": "",
        "task_id": "",
        "research_topic": REPORT_DEFAULT_RESEARCH_TOPIC,
    })
}

fn normalize_intercept_input(input: &str) -> String {
    let trimmed = input.trim();
    let normalized = trimmed
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            trimmed
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
        .unwrap_or(trimmed)
        .trim();
    normalized.to_string()
}

fn parse_admin_registration_passphrase(input: &str) -> Option<String> {
    let normalized = normalize_intercept_input(input);
    let remainder = normalized
        .strip_prefix(REGISTER_ADMIN_INTERCEPT_PREFIX)?
        .trim();
    if remainder.is_empty() {
        return None;
    }
    Some(remainder.to_string())
}

#[cfg(test)]
fn matches_register_admin_intercept(input: &str) -> bool {
    parse_admin_registration_passphrase(input).is_some()
}

fn constant_time_str_eq(left: &str, right: &str) -> bool {
    left.as_bytes().ct_eq(right.as_bytes()).into()
}

fn parse_report_intercept(input: &str) -> Option<ReportIntercept> {
    let normalized = normalize_intercept_input(input);
    let remainder = normalized.strip_prefix(REPORT_INTERCEPT_PREFIX)?.trim();
    if remainder.is_empty() {
        return None;
    }
    if remainder == REPORT_PROGRESS_COMMAND
        || remainder.eq_ignore_ascii_case(REPORT_PROGRESS_COMMAND_ALIAS)
    {
        return Some(ReportIntercept::Progress);
    }
    Some(ReportIntercept::Start {
        company_name: remainder.to_string(),
    })
}

fn format_progress_message(progress: &WorkflowProgressEnvelope) -> String {
    let active = summarize_active_nodes(&progress.progress.active_nodes);
    format!(
        "研报任务正在运行中：{:.1}%（{}/{} 节点已进入终态，{} 个节点运行中，{} 个节点待执行）。{} run_id={}",
        progress.progress.percent,
        progress.progress.terminal_nodes,
        progress.progress.total_nodes,
        progress.progress.running_nodes,
        progress.progress.pending_nodes,
        active,
        progress.id
    )
}

fn format_recent_report_message(run: &WorkflowRunSummary) -> String {
    let workflow_name = run
        .workflow_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&run.workflow_id);
    let progress = run.progress.as_ref();
    let percent = progress.map(|value| value.percent).unwrap_or(0.0);
    let terminal_nodes = progress.map(|value| value.terminal_nodes).unwrap_or(0);
    let total_nodes = progress.map(|value| value.total_nodes).unwrap_or(0);
    let status_label = match run.status.as_str() {
        "succeeded" => "已完成",
        "failed" => "已失败",
        "stopped" => "已停止",
        other => other,
    };
    let mut message = format!(
        "当前没有运行中的研报任务。最近一次任务：{}（{}，{:.1}% ，{}/{} 节点终态，run_id={}）。",
        workflow_name, status_label, percent, terminal_nodes, total_nodes, run.id
    );
    if let Some(ended_at) = run
        .ended_at
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        message.push_str(&format!(" 结束时间：{ended_at}。"));
    }
    if let Some(error) = run
        .error
        .as_deref()
        .map(first_non_empty_line)
        .filter(|value| !value.is_empty())
    {
        message.push_str(&format!(" 错误：{}。", truncate_for_log(&error, 120)));
    }
    message
}

fn summarize_active_nodes(nodes: &[WorkflowActiveNode]) -> String {
    if nodes.is_empty() {
        return "当前没有活跃节点。".to_string();
    }
    let labels = nodes
        .iter()
        .take(3)
        .map(|node| {
            let workflow_name = node
                .workflow_name
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("-");
            let title = node
                .title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(&node.id);
            format!("{workflow_name}/{title}")
        })
        .collect::<Vec<_>>();
    if nodes.len() > 3 {
        format!(
            "当前活跃节点：{} 等 {} 个。",
            labels.join("、"),
            nodes.len()
        )
    } else {
        format!("当前活跃节点：{}。", labels.join("、"))
    }
}

fn first_non_empty_line(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

fn should_fallback_to_curl(url: &str) -> bool {
    url.starts_with("http://127.0.0.1:")
        || url.starts_with("http://localhost:")
        || url.starts_with("http://[::1]:")
}

#[cfg(test)]
mod tests {
    use super::{
        HoneBotCore, REGISTER_ADMIN_INTERCEPT_DENY_ACK, REGISTER_ADMIN_INTERCEPT_DISABLED_ACK,
        REGISTER_ADMIN_INTERCEPT_INVALID_ACK, REGISTER_ADMIN_INTERCEPT_PREFIX, REPORT_DEFAULT_MODE,
        REPORT_DEFAULT_RESEARCH_TOPIC, ReportIntercept, build_report_run_input,
        matches_register_admin_intercept, parse_report_intercept,
    };
    use hone_core::{ActorIdentity, HoneConfig};
    use serde_json::json;

    const REGISTER_ADMIN_INTERCEPT_TEXT: &str = "/register-admin secret";

    #[test]
    fn register_admin_intercept_matches_plain_and_quoted_text() {
        assert!(matches_register_admin_intercept(
            REGISTER_ADMIN_INTERCEPT_TEXT
        ));
        assert!(matches_register_admin_intercept(
            "' /register-admin secret '"
        ));
        assert!(matches_register_admin_intercept(
            "\"/register-admin secret\""
        ));
        assert!(!matches_register_admin_intercept("/register-admin"));
    }

    #[test]
    fn runtime_admin_override_requires_whitelisted_actor_and_configured_passphrase() {
        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
        assert_eq!(
            core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
            Some(REGISTER_ADMIN_INTERCEPT_DENY_ACK.to_string())
        );
    }

    #[test]
    fn runtime_admin_override_rejects_when_passphrase_missing_or_invalid() {
        let mut config = HoneConfig::default();
        config.admins.discord_user_ids = vec!["alice".to_string()];
        let core = HoneBotCore::new(config.clone());
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");

        assert_eq!(
            core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
            Some(REGISTER_ADMIN_INTERCEPT_DISABLED_ACK.to_string())
        );

        config.admins.runtime_admin_registration_passphrase = "secret".to_string();
        let core = HoneBotCore::new(config);
        assert_eq!(
            core.try_intercept_admin_registration(
                &actor,
                &format!("{REGISTER_ADMIN_INTERCEPT_PREFIX} wrong")
            ),
            Some(REGISTER_ADMIN_INTERCEPT_INVALID_ACK.to_string())
        );
    }

    #[test]
    fn runtime_admin_override_is_scoped_to_actor_identity() {
        let mut config = HoneConfig::default();
        config.admins.discord_user_ids = vec!["alice".to_string()];
        config.admins.runtime_admin_registration_passphrase = "secret".to_string();
        let core = HoneBotCore::new(config);
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
        let other_scope =
            ActorIdentity::new("discord", "alice", Some("g:1:c:3")).expect("other scope");

        assert!(core.is_admin(&actor.user_id, &actor.channel));
        assert!(
            !core
                .runtime_admin_overrides
                .read()
                .unwrap()
                .contains(&actor)
        );
        assert_eq!(
            core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT),
            Some(super::REGISTER_ADMIN_INTERCEPT_ACK.to_string())
        );
        assert!(
            core.runtime_admin_overrides
                .read()
                .unwrap()
                .contains(&actor)
        );
        assert!(core.is_admin_actor(&actor));
        assert!(core.is_admin_actor(&other_scope));
    }

    #[test]
    fn actor_scoped_registry_includes_local_file_tools() {
        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");

        let with_actor = core.create_tool_registry(Some(&actor), "discord", false);
        let without_actor = core.create_tool_registry(None, "discord", false);

        let with_actor_tools = with_actor.list_tool_names();
        assert!(with_actor_tools.contains(&"local_list_files"));
        assert!(with_actor_tools.contains(&"local_search_files"));
        assert!(with_actor_tools.contains(&"local_read_file"));

        let without_actor_tools = without_actor.list_tool_names();
        assert!(!without_actor_tools.contains(&"local_list_files"));
        assert!(!without_actor_tools.contains(&"local_search_files"));
        assert!(!without_actor_tools.contains(&"local_read_file"));
    }

    #[test]
    fn report_intercept_parses_company_name_and_progress() {
        assert_eq!(
            parse_report_intercept("/report Tempus AI"),
            Some(ReportIntercept::Start {
                company_name: "Tempus AI".to_string()
            })
        );
        assert_eq!(
            parse_report_intercept("  '/report 进度'  "),
            Some(ReportIntercept::Progress)
        );
        assert_eq!(
            parse_report_intercept("/report progress"),
            Some(ReportIntercept::Progress)
        );
        assert_eq!(parse_report_intercept("/report"), None);
    }

    #[test]
    fn report_run_input_includes_required_defaults() {
        assert_eq!(
            build_report_run_input("Astera Labs", "validate-me"),
            json!({
                "companyName": "Astera Labs",
                "genPost": REPORT_DEFAULT_MODE,
                "validateCode": "validate-me",
                "news": "",
                "task_id": "",
                "research_topic": REPORT_DEFAULT_RESEARCH_TOPIC,
            })
        );
    }

    #[test]
    fn effective_multi_agent_search_config_falls_back_to_auxiliary_api_key() {
        let mut config = HoneConfig::default();
        config.agent.runner = "multi-agent".to_string();
        config.agent.multi_agent.search.base_url = "https://api.minimaxi.com/v1".to_string();
        config.agent.multi_agent.search.model = "MiniMax-M2.7-highspeed".to_string();
        config.agent.multi_agent.search.api_key = String::new();
        config.llm.auxiliary.base_url = "https://api.minimaxi.com/v1".to_string();
        config.llm.auxiliary.model = "MiniMax-M2.7-highspeed".to_string();
        config.llm.auxiliary.api_key = "sk-cp-aux".to_string();

        let core = HoneBotCore::new(config);
        let effective = core.effective_multi_agent_search_config();

        assert_eq!(effective.api_key, "sk-cp-aux");
        assert_eq!(effective.base_url, "https://api.minimaxi.com/v1");
        assert_eq!(effective.model, "MiniMax-M2.7-highspeed");
    }

    #[test]
    fn effective_multi_agent_search_config_preserves_explicit_search_api_key() {
        let mut config = HoneConfig::default();
        config.agent.runner = "multi-agent".to_string();
        config.agent.multi_agent.search.api_key = "sk-cp-search".to_string();
        config.llm.auxiliary.api_key = "sk-cp-aux".to_string();

        let core = HoneBotCore::new(config);
        let effective = core.effective_multi_agent_search_config();

        assert_eq!(effective.api_key, "sk-cp-search");
    }

    #[test]
    fn multi_agent_answer_zero_tool_limit_is_preserved() {
        let mut config = HoneConfig::default();
        config.agent.runner = "multi-agent".to_string();
        config.agent.multi_agent.answer.max_tool_calls = 0;

        let core = HoneBotCore::new(config);

        assert_eq!(core.effective_multi_agent_answer_max_tool_calls(), 0);
    }
}
