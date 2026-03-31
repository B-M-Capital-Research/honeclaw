//! HoneBotCore — 配置/工具/会话 runner 工厂
//!
//! 各渠道共享的 Bot 核心配置。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use hone_core::agent::AgentResponse;
use hone_core::config::HoneConfig;
use hone_core::{ActorIdentity, LlmAuditRecord, LlmAuditSink};
use hone_llm::{LlmProvider, OpenRouterProvider};
use hone_memory::{
    ConversationQuotaStorage, CronJobStorage, KbStorage, LlmAuditStorage, SessionStorage,
    StockTableStorage, build_compact_boundary_metadata, build_compact_skill_snapshot_metadata,
    build_compact_summary_metadata, invoked_skills_from_metadata,
    select_messages_after_compact_boundary, session::SessionSummary,
};
use hone_scheduler::{HoneScheduler, SchedulerEvent};
use hone_tools::{
    CronJobTool, DeepResearchTool, DiscoverSkillsTool, LoadSkillTool, ToolExecutionGuard,
    ToolRegistry,
};
use tokio::sync::mpsc;

use crate::runners::{
    AgentRunner, CodexAcpRunner, CodexCliReasoningRunner, FunctionCallingReasoningRunner,
    GeminiAcpRunner, GeminiCliRunner, OpencodeAcpRunner,
};

pub const REGISTER_ADMIN_INTERCEPT_TEXT: &str = "/register-admin AMM";
pub const REGISTER_ADMIN_INTERCEPT_ACK: &str = "已将当前 identity 升级为管理员。";
const POST_COMPACT_MAX_SKILL_SNAPSHOT_CHARS: usize = 12_000;
const POST_COMPACT_MAX_SKILL_SNAPSHOTS: usize = 4;

#[derive(Debug, Clone)]
pub struct CompactSessionOutcome {
    pub compacted: bool,
    pub summary: Option<String>,
}

/// Bot 核心 — 持有所有共享依赖
pub struct HoneBotCore {
    pub config: HoneConfig,
    pub llm: Option<Arc<dyn LlmProvider>>,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub session_storage: SessionStorage,
    pub conversation_quota_storage: ConversationQuotaStorage,
    pub kb_storage: KbStorage,
    pub stock_table: StockTableStorage,
    runtime_admin_overrides: RwLock<HashSet<ActorIdentity>>,
}

impl HoneBotCore {
    /// 从配置创建
    pub fn new(config: HoneConfig) -> Self {
        let session_storage = SessionStorage::from_storage_config(&config.storage);
        let conversation_quota_storage =
            ConversationQuotaStorage::new(&config.storage.conversation_quota_dir)
                .expect("failed to initialize conversation quota storage");
        let kb_storage = KbStorage::new(&config.storage.kb_dir);
        let stock_table = StockTableStorage::new(&config.storage.kb_dir);
        let llm = Self::create_llm_provider(&config);
        let llm_audit = Self::create_llm_audit_sink(&config);

        Self {
            config,
            llm,
            llm_audit,
            session_storage,
            conversation_quota_storage,
            kb_storage,
            stock_table,
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

        if self.llm.is_some() {
            tracing::info!(
                "[Startup/{channel}] session.compression.engine=llm provider={} model={} threshold=40 retain_recent=4",
                printable_or_default(llm_provider, "<empty>"),
                printable_or_default(self.config.llm.openrouter.auxiliary_model(), "<empty>")
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

    pub fn try_intercept_admin_registration(&self, actor: &ActorIdentity, input: &str) -> bool {
        if !matches_register_admin_intercept(input) {
            return false;
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
        true
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

        let dirs = vec![skills_dir.clone(), custom_skills_dir.clone()];

        registry.register(Box::new(LoadSkillTool::new(dirs)));
        registry.register(Box::new(DiscoverSkillsTool::new(
            skills_dir.clone(),
            custom_skills_dir.clone(),
        )));
        registry.register(Box::new(hone_tools::skill_tool::SkillTool::new(
            skills_dir,
            custom_skills_dir,
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

        // 知识记忆查询工具
        registry.register(Box::new(hone_tools::KbSearchTool::new(
            std::path::PathBuf::from(&self.config.storage.kb_dir),
            actor.cloned(),
            self.config.security.kb_actor_isolation,
        )));
        tracing::info!("[HoneBotCore] 已注册工具 kb_search");

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
        std::env::var("HONE_DATA_DIR")
            .map(|root| PathBuf::from(root).join("custom_skills"))
            .unwrap_or_else(|_| PathBuf::from("./data/custom_skills"))
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
        match runner {
            "gemini_cli" => Ok(Box::new(GeminiCliRunner::new(
                system_prompt.to_string(),
                Arc::new(tool_registry),
            ))),
            "gemini_acp" => Ok(Box::new(GeminiAcpRunner::new(
                self.config.agent.gemini_acp.clone(),
            ))),
            "codex_cli" => Ok(Box::new(CodexCliReasoningRunner::new(
                system_prompt.to_string(),
                Some(self.config.agent.codex_model.clone()),
                Arc::new(tool_registry),
                self.llm_audit.clone(),
            ))),
            "codex_acp" => Ok(Box::new(CodexAcpRunner::new(
                self.config.agent.codex_acp.clone(),
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
                // 优先使用 config_runtime.yaml 中配置的 OpenRouter API key（取第一个有效 key）
                let pool = self.config.llm.openrouter.effective_key_pool();
                if let Some(key) = pool.first() {
                    opencode_config.openrouter_api_key = Some(key.to_string());
                }
                Ok(Box::new(OpencodeAcpRunner::new(opencode_config)))
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
        let Some(session) = self.session_storage.load_session(session_id)? else {
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        };

        let active_messages = select_messages_after_compact_boundary(&session.messages, None);
        if active_messages.is_empty() {
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        }
        let is_group_session = session
            .session_identity
            .as_ref()
            .map(|identity| identity.is_group())
            .unwrap_or(false);

        let compress_threshold = if is_group_session {
            self.config.group_context.compress_threshold_messages.max(1)
        } else {
            20
        };
        let compress_byte_threshold = if is_group_session {
            self.config.group_context.compress_threshold_bytes.max(1024)
        } else {
            80_000
        };
        let retain_recent = if is_group_session {
            self.config
                .group_context
                .retain_recent_after_compress
                .max(1)
        } else {
            6
        };

        let total_content_bytes: usize = active_messages.iter().map(|m| m.content.len()).sum();

        let should_compress = force
            || active_messages.len() > compress_threshold
            || total_content_bytes > compress_byte_threshold;

        if !should_compress {
            return Ok(CompactSessionOutcome {
                compacted: false,
                summary: None,
            });
        }

        tracing::info!(
            "[HoneBotCore] Compressing session {} with {} messages (~{} bytes)...",
            session_id,
            active_messages.len(),
            total_content_bytes,
        );

        let llm = match &self.llm {
            Some(provider) => provider.as_ref(),
            None => {
                tracing::warn!(
                    "[HoneBotCore] No LLM provider available for compression. Please configure llm provider in config.yaml. Skipping compression."
                );
                return Ok(CompactSessionOutcome {
                    compacted: false,
                    summary: None,
                });
            }
        };

        // 构建供 LLM 总结的历史文本
        let mut history_text = String::new();
        for m in &active_messages {
            history_text.push_str(&format!("{}: {}\n\n", m.role, m.content));
        }

        let prompt = if is_group_session {
            format!(
                "你是一个群聊上下文整理员。由于群会话历史过长，需要把更早内容压缩成稳定、简洁、适合后续继续讨论的群摘要。\n\
                \n\
                只输出纯 Markdown，并严格使用以下四段标题：\n\
                \n\
                ## 进行中议题\n\
                - 列出群里当前仍在讨论的问题或主题\n\
                \n\
                ## 已形成结论\n\
                - 只记录群内已经达成的结论或明确共识\n\
                \n\
                ## 未决问题\n\
                - 列出仍待回答、待确认、待补充的信息\n\
                \n\
                ## 群约定 / 待办\n\
                - 记录群里明确提到的后续动作、分工、约定\n\
                \n\
                额外约束：\n\
                - 不要写成员画像、长期个人偏好或性格判断\n\
                - 不要固化个人金融隐私，如持仓、成本、成交价、交易单等\n\
                - 只保留对后续群讨论真正有帮助的信息\n\
                - 不要寒暄，不要输出其它标题\n\
                {}\n\
                \n\
                以下是待压缩的群历史：\n\
                {}",
                user_instructions
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| format!(" - 额外要求：{value}\n"))
                    .unwrap_or_default(),
                history_text
            )
        } else {
            format!(
                "你是一个金融分析助手的记忆整理员。由于用户的对话历史过长，我需要你将他们进行压缩和总结。\n\
                \n\
                请按照以下格式输出：\n\
                \n\
                1. **股票关注表**（只提取对话中提到的具体股票/公司）包含以下五列：\n\
                | 股票代码 | 公司名 | 公司一句话简介 | 助手的观点 | 用户的观点 |\n\
                | --- | --- | --- | --- | --- |\n\
                （如果没有提取到股票，输出一个空表即可，但必须包含表头）\n\
                \n\
                2. **【历史对话总结】**\n\
                在表下面，用1-2段话总结上面发生的核心交互和用户的偏好习惯信息。\n\
                \n\
                {}\n\
                请保持纯净的 Markdown 输出，不要有多余的寒喧。\n\
                \n\
                以下是对话历史：\n\
                {}",
                user_instructions
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| format!("额外要求：{value}\n"))
                    .unwrap_or_default(),
                history_text
            )
        };

        let msgs = vec![hone_llm::Message {
            role: "user".to_string(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];

        let started = std::time::Instant::now();
        let request_payload = serde_json::json!({ "messages": msgs.clone() });
        let auxiliary_model = self.config.llm.openrouter.auxiliary_model().to_string();
        let (new_summary_content, usage) = match llm.chat(&msgs, Some(&auxiliary_model)).await {
            Ok(result) => (result.content, result.usage),
            Err(e) => {
                self.record_llm_audit(LlmAuditRecord {
                    success: false,
                    latency_ms: Some(started.elapsed().as_millis()),
                    error: Some(e.to_string()),
                    metadata: serde_json::json!({
                        "kind": "session_compression",
                        "active_messages": active_messages.len(),
                        "is_group_session": is_group_session
                    }),
                    prompt_tokens: None,
                    completion_tokens: None,
                    total_tokens: None,
                    ..LlmAuditRecord::new(
                        session_id.to_string(),
                        session.actor.clone(),
                        "core.session_compression",
                        "chat",
                        self.config.llm.provider.clone(),
                        Some(auxiliary_model.clone()),
                        request_payload.clone(),
                    )
                });
                tracing::error!("[HoneBotCore] LLM summarization failed: {}", e);
                return Ok(CompactSessionOutcome {
                    compacted: false,
                    summary: None,
                });
            }
        };

        self.record_llm_audit(LlmAuditRecord {
            success: true,
            latency_ms: Some(started.elapsed().as_millis()),
            response: Some(serde_json::json!({ "content": new_summary_content.clone() })),
            metadata: serde_json::json!({
                "kind": "session_compression",
                "active_messages": active_messages.len(),
                "retained_recent": retain_recent,
                "is_group_session": is_group_session,
                "trigger": trigger,
                "forced": force,
                "custom_instructions": user_instructions
            }),
            prompt_tokens: usage.as_ref().and_then(|u| u.prompt_tokens),
            completion_tokens: usage.as_ref().and_then(|u| u.completion_tokens),
            total_tokens: usage.as_ref().and_then(|u| u.total_tokens),
            ..LlmAuditRecord::new(
                session_id.to_string(),
                session.actor.clone(),
                "core.session_compression",
                "chat",
                self.config.llm.provider.clone(),
                Some(auxiliary_model),
                request_payload,
            )
        });

        let mut new_messages = Vec::new();
        new_messages.push(hone_memory::session::SessionMessage {
            role: "system".to_string(),
            content: "Conversation compacted".to_string(),
            timestamp: hone_core::beijing_now_rfc3339(),
            metadata: Some(build_compact_boundary_metadata(
                trigger,
                active_messages.len().saturating_sub(retain_recent),
                active_messages.len(),
            )),
        });
        new_messages.push(hone_memory::session::SessionMessage {
            role: "user".to_string(),
            content: format!("【Compact Summary】\n{}", new_summary_content.trim()),
            timestamp: hone_core::beijing_now_rfc3339(),
            metadata: Some(build_compact_summary_metadata(trigger)),
        });
        for skill in invoked_skills_from_metadata(&session.metadata)
            .into_iter()
            .take(POST_COMPACT_MAX_SKILL_SNAPSHOTS)
        {
            let snapshot = truncate_chars(&skill.prompt, POST_COMPACT_MAX_SKILL_SNAPSHOT_CHARS);
            if snapshot.trim().is_empty() {
                continue;
            }
            new_messages.push(hone_memory::session::SessionMessage {
                role: "user".to_string(),
                content: snapshot,
                timestamp: hone_core::beijing_now_rfc3339(),
                metadata: Some(build_compact_skill_snapshot_metadata(&skill.skill_name)),
            });
        }
        // 保留最近的 N 条对话
        let retained: Vec<_> = active_messages
            .into_iter()
            .rev()
            .take(retain_recent)
            .collect();
        for m in retained.into_iter().rev() {
            new_messages.push(m.clone());
        }

        // 保存回 session
        self.session_storage.replace_messages_with_summary(
            session_id,
            new_messages,
            Some(SessionSummary::new(&new_summary_content)),
        )?;
        tracing::info!(
            "[HoneBotCore] Session {} compacted to boundary + summary + {} retained items.",
            session_id,
            retain_recent
        );

        Ok(CompactSessionOutcome {
            compacted: true,
            summary: Some(new_summary_content),
        })
    }

    fn record_llm_audit(&self, record: LlmAuditRecord) {
        if let Some(sink) = &self.llm_audit {
            if let Err(err) = sink.record(record) {
                tracing::warn!("[LlmAudit] failed to persist record: {}", err);
            }
        }
    }
}

fn truncate_chars(content: &str, max_chars: usize) -> String {
    if max_chars == 0 || content.chars().count() <= max_chars {
        return content.to_string();
    }
    content.chars().take(max_chars).collect::<String>()
}

pub fn runtime_config_path() -> String {
    std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string())
}

pub fn load_runtime_config() -> hone_core::HoneResult<(HoneConfig, String)> {
    let config_path = runtime_config_path();
    let mut config = HoneConfig::from_file(&config_path)?;
    apply_runtime_config_overrides(&mut config);
    ensure_runtime_dirs(&config);
    Ok((config, config_path))
}

fn apply_runtime_config_overrides(config: &mut HoneConfig) {
    if let Ok(data_dir) = std::env::var("HONE_DATA_DIR") {
        let root = PathBuf::from(data_dir);
        config.storage.sessions_dir = root.join("sessions").to_string_lossy().to_string();
        config.storage.conversation_quota_dir = root
            .join("conversation_quota")
            .to_string_lossy()
            .to_string();
        config.storage.llm_audit_db_path =
            root.join("llm_audit.sqlite3").to_string_lossy().to_string();
        config.storage.portfolio_dir = root.join("portfolio").to_string_lossy().to_string();
        config.storage.cron_jobs_dir = root.join("cron_jobs").to_string_lossy().to_string();
        config.storage.reports_dir = root.join("reports").to_string_lossy().to_string();
        config.storage.x_drafts_dir = root.join("x_drafts").to_string_lossy().to_string();
        config.storage.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();
        config.storage.kb_dir = root.join("kb").to_string_lossy().to_string();
    }

    if let Ok(skills_dir) = std::env::var("HONE_SKILLS_DIR") {
        config.extra.insert(
            "skills_dir".to_string(),
            serde_yaml::Value::String(skills_dir),
        );
    }
}

fn ensure_runtime_dirs(config: &HoneConfig) {
    let _ = std::fs::create_dir_all(&config.storage.sessions_dir);
    let _ = std::fs::create_dir_all(&config.storage.portfolio_dir);
    let _ = std::fs::create_dir_all(&config.storage.cron_jobs_dir);
    let _ = std::fs::create_dir_all(&config.storage.reports_dir);
    let _ = std::fs::create_dir_all(&config.storage.x_drafts_dir);
    let _ = std::fs::create_dir_all(&config.storage.gen_images_dir);
    let _ = std::fs::create_dir_all(&config.storage.kb_dir);
    let _ = std::fs::create_dir_all(&config.storage.conversation_quota_dir);
    if let Some(parent) = PathBuf::from(&config.storage.llm_audit_db_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
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

fn matches_register_admin_intercept(input: &str) -> bool {
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
    normalized == REGISTER_ADMIN_INTERCEPT_TEXT
}

#[cfg(test)]
mod tests {
    use super::{HoneBotCore, REGISTER_ADMIN_INTERCEPT_TEXT, matches_register_admin_intercept};
    use hone_core::{ActorIdentity, HoneConfig};

    #[test]
    fn register_admin_intercept_matches_plain_and_quoted_text() {
        assert!(matches_register_admin_intercept(
            REGISTER_ADMIN_INTERCEPT_TEXT
        ));
        assert!(matches_register_admin_intercept("' /register-admin AMM '"));
        assert!(matches_register_admin_intercept("\"/register-admin AMM\""));
        assert!(!matches_register_admin_intercept("/register-admin"));
    }

    #[test]
    fn runtime_admin_override_is_scoped_to_actor_identity() {
        let core = HoneBotCore::new(HoneConfig::default());
        let actor = ActorIdentity::new("discord", "alice", Some("g:1:c:2")).expect("actor");
        let other_scope =
            ActorIdentity::new("discord", "alice", Some("g:1:c:3")).expect("other scope");

        assert!(!core.is_admin_actor(&actor));
        assert!(core.try_intercept_admin_registration(&actor, REGISTER_ADMIN_INTERCEPT_TEXT));
        assert!(core.is_admin_actor(&actor));
        assert!(!core.is_admin_actor(&other_scope));
    }
}
