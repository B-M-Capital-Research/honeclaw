//! `HoneBotCore` —— 各渠道共享的「Bot 内核」。
//!
//! 持有所有跨渠道共享的依赖(配置、LLM provider、session 存储、
//! 管理员白名单、workflow runner HTTP 客户端 …)以及从这些依赖派生出
//! 的**工厂方法**(`create_tool_registry` / `create_runner` / `create_scheduler`)。
//!
//! 拆分策略:
//! - **这个文件**:`HoneBotCore` struct 定义 + 构造 + capability factory + session 压缩;
//! - `super::logging`:启动路由日志 + 消息流日志 + 格式化 helper;
//! - `super::intercept`:`/register-admin` / `/report` 拦截层 + workflow bridge HTTP 调用。
//!
//! 注:同一个 `impl HoneBotCore` 可以分散在多个文件里(Rust 的模块系统允许),
//! sibling module 通过 `pub(super)` 字段可见性访问内部状态。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use hone_core::config::{AgentRunnerKind, HoneConfig};
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
use tokio::sync::mpsc;

use crate::runners::{
    AgentRunner, CodexAcpRunner, CodexCliReasoningRunner, FunctionCallingReasoningRunner,
    GeminiCliRunner, MultiAgentRunner, OpencodeAcpRunner, RunnerTimeouts,
};
use crate::sandbox::sandbox_base_dir;
use crate::session_compactor::SessionCompactor;

use super::logging::printable_or_default;

#[derive(Debug, Clone)]
pub struct CompactSessionOutcome {
    pub compacted: bool,
    pub summary: Option<String>,
}

/// Bot 核心 — 持有所有共享依赖。
///
/// `pub(super)` 字段(`workflow_runner_http`, `runtime_admin_overrides`)
/// 留给 `super::intercept` 访问 —— 它们本质是「core 状态」但方法已经搬到
/// sibling,visibility 卡在 agent_session module 内部。
pub struct HoneBotCore {
    pub config: HoneConfig,
    pub llm: Option<Arc<dyn LlmProvider>>,
    pub auxiliary_llm: Option<Arc<dyn LlmProvider>>,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub session_storage: SessionStorage,
    pub conversation_quota_storage: ConversationQuotaStorage,
    pub(super) workflow_runner_http: reqwest::Client,
    pub company_profile_storage: CompanyProfileStorage,
    pub(super) runtime_admin_overrides: RwLock<HashSet<ActorIdentity>>,
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

    pub(super) fn effective_multi_agent_search_config(
        &self,
    ) -> hone_core::config::MultiAgentSearchConfig {
        let mut search_config = self.config.agent.multi_agent.search.clone();
        if search_config.api_key.trim().is_empty() {
            let fallback_key = self.config.llm.auxiliary.resolved_api_key();
            if !fallback_key.trim().is_empty() {
                search_config.api_key = fallback_key;
            }
        }
        search_config
    }

    pub(super) fn effective_multi_agent_answer_max_tool_calls(&self) -> u32 {
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

    /// 检查某用户在指定渠道是否为管理员
    ///
    /// - channel 传 "imessage" 时与 admins.imessage_handles 匹配
    /// - channel 传 "telegram" 时与 admins.telegram_user_ids 匹配
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
            "telegram" => admin_cfg
                .telegram_user_ids
                .iter()
                .any(|id| !id.is_empty() && id == user_id),
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

        // 终端用户通过自然语言调推送偏好——构造时硬绑定 actor,只能改自己那份。
        // 目录必须与 event-engine `with_prefs_dir` 使用同一个,否则写进去 router 读不到。
        // 同时强制注入 overview 上下文(cron_jobs_dir + unified digest 默认槽位时刻),
        // 让 get_overview action 总能给出完整的「我的推送日程」拍平视图,无 partial 分支。
        let overview_digest_defaults = hone_tools::schedule_view::DigestDefaults {
            pre_market: self.config.event_engine.digest.pre_market.clone(),
            post_market: self.config.event_engine.digest.post_market.clone(),
        };
        registry.register(Box::new(hone_tools::NotificationPrefsTool::new(
            &self.config.storage.notif_prefs_dir,
            actor.cloned(),
            &self.config.storage.cron_jobs_dir,
            overview_digest_defaults,
        )));

        // 让用户通过 `/missed` 或自然语言查回 digest/router 主动筛掉的事件。
        // event store 路径与 web-api `bootstrap_event_engine` 约定一致:
        // `<data_dir>/events.sqlite3`。actor 强制绑定调用方 —— 工具层面也
        // 不允许查别人。
        registry.register(Box::new(hone_tools::MissedEventsTool::new(
            self.configured_data_dir().join("events.sqlite3"),
            actor.cloned(),
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
        match self.config.agent.runner_kind() {
            AgentRunnerKind::GeminiCli => Ok(Box::new(GeminiCliRunner::new(
                system_prompt.to_string(),
                Arc::new(tool_registry),
                runner_timeouts,
            ))),
            AgentRunnerKind::GeminiAcp => Err(
                "dialog.engine=gemini_acp 已被 honeclaw 全局禁用（gemini 不推 usage_update，\
                 无法可靠检测内置 compact 信号；Gemini ToS 也不建议这种长 session 复用模式）。\
                 请在 config.yaml 的 agent.runner 切换到 codex_acp / opencode_acp / multi-agent / function_calling。"
                    .to_string(),
            ),
            AgentRunnerKind::CodexCli => Ok(Box::new(CodexCliReasoningRunner::new(
                system_prompt.to_string(),
                Some(self.config.agent.codex_model.clone()),
                Arc::new(tool_registry),
                self.llm_audit.clone(),
            ))),
            AgentRunnerKind::CodexAcp => Ok(Box::new(CodexAcpRunner::new(
                self.config.agent.codex_acp.clone(),
                runner_timeouts,
            ))),
            AgentRunnerKind::FunctionCalling => {
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
            AgentRunnerKind::OpencodeAcp => {
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
            AgentRunnerKind::MultiAgent => {
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
            AgentRunnerKind::Unknown => {
                tracing::warn!(
                    "[HoneBotCore] unknown runner={}, fallback to function_calling",
                    printable_or_default(runner, "<empty>")
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

    /// 检查并压缩会话历史。
    ///
    /// 对于自带上下文管理 / 内置 compact 的 runner（codex_acp / opencode_acp），
    /// 直接短路返回 —— 不再触发 SessionCompactor，把 honeclaw 自己的
    /// `session_messages` 完整保留，依赖 ACP runner 内部 session 自压。
    /// 见 `docs/bugs/session_compact_summary_report_hallucination.md` 2026-04-23 决策。
    pub async fn maybe_compress_session(&self, session_id: &str) -> hone_core::HoneResult<()> {
        if self.config.agent.runner_kind().manages_own_context() {
            tracing::debug!(
                "[Compact] session={} runner={} 自管上下文，跳过 honeclaw 自动 compact",
                session_id,
                self.config.agent.runner
            );
            return Ok(());
        }
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
