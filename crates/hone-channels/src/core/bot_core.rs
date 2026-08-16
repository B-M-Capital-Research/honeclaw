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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::config::{AgentConversationStrategy, AgentRunnerKind, HoneConfig};
use hone_core::{ActorIdentity, LlmAuditSink};
use hone_event_engine::EventStore;
use hone_llm::{LlmProvider, LlmResolver};
use hone_memory::{
    CompanyProfileStorage, ConversationQuotaStorage, CronJobStorage, LlmAuditStorage,
    SessionStorage, configure_cloud_company_profile_storage, configure_cloud_portfolio_storage,
};
use hone_scheduler::{HoneScheduler, SchedulerEvent};
use hone_tools::{
    CronJobTool, DeepResearchTool, DiscoverSkillsTool, LoadSkillTool, ToolExecutionGuard,
    ToolRegistry, configure_cloud_notification_prefs, configure_cloud_skill_registry,
};
use tokio::sync::mpsc;

use crate::runners::{
    AgentRunner, CodexAcpRunner, CodexCliReasoningRunner, FunctionCallingReasoningRunner,
    GeminiCliRunner, HoneCloudRunner, OpencodeAcpRunner, RunnerTimeouts,
};
use crate::sandbox::sandbox_base_dir;
use crate::session_compactor::SessionCompactor;

use super::logging::printable_or_default;

#[cfg(test)]
async fn isolated_test_postgres(postgres: CloudPgRuntime) -> CloudPgRuntime {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_NAMESPACE: AtomicU64 = AtomicU64::new(1);
    let namespace = format!(
        "hone_memory_channels_{}_{}",
        std::process::id(),
        NEXT_NAMESPACE.fetch_add(1, Ordering::Relaxed)
    );
    let postgres = postgres
        .with_isolated_test_connection(namespace)
        .expect("PostgreSQL test namespace must be valid");
    postgres
        .ensure_schema()
        .await
        .expect("failed to initialize isolated PostgreSQL test schema");
    postgres
}

pub(super) const STRICT_ACTOR_MAX_ITERATIONS: u32 = 18;

#[derive(Debug, Clone)]
pub struct CompactSessionOutcome {
    pub compacted: bool,
    pub summary: Option<String>,
}

/// Bot 核心 — 持有所有共享依赖。
///
/// `pub(super)` 字段(`workflow_runner_http`, `runtime_admin_overrides`)
/// 留给 `super::intercept` 访问 —— 它们本质是「core 状态」但方法已经搬到
/// sibling module,所以可见性收在 `core` module 内部。
pub struct HoneBotCore {
    pub config: HoneConfig,
    pub(crate) cloud_pg_runtime: CloudPgRuntime,
    pub llm: Option<Arc<dyn LlmProvider>>,
    pub auxiliary_llm: Option<Arc<dyn LlmProvider>>,
    pub llm_audit: Option<Arc<dyn LlmAuditSink>>,
    pub session_storage: SessionStorage,
    pub conversation_quota_storage: ConversationQuotaStorage,
    pub(crate) delivered_push_context_store: Option<EventStore>,
    pub(super) workflow_runner_http: reqwest::Client,
    pub company_profile_storage: CompanyProfileStorage,
    pub(super) runtime_admin_overrides: RwLock<HashSet<ActorIdentity>>,
    #[cfg(test)]
    pub(crate) test_runner_factory: Option<Arc<dyn Fn() -> Box<dyn AgentRunner> + Send + Sync>>,
}

#[cfg(test)]
impl Drop for HoneBotCore {
    fn drop(&mut self) {
        let postgres = self.cloud_pg_runtime.clone();
        // `Drop` 不能 async；仅测试实例在独立线程的临时 runtime 中同步回收
        // 隔离 schema。生产存储调用链不经过这里，也不再共享 bridge runtime。
        let _ = std::thread::Builder::new()
            .name("hone-channels-test-pg-cleanup".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| error.to_string())?;
                runtime
                    .block_on(postgres.drop_isolated_memory_test_schema())
                    .map_err(|error| error.to_string())
            })
            .and_then(|thread| {
                thread
                    .join()
                    .map_err(|_| std::io::Error::other("channels PostgreSQL cleanup panicked"))?
                    .map_err(std::io::Error::other)
            });
        self.cloud_pg_runtime.evict_cached_test_client();
    }
}

impl HoneBotCore {
    /// 从配置创建
    pub async fn new(config: HoneConfig) -> Self {
        hone_core::configure_runtime_timezone(config.timezone.as_deref())
            .expect("runtime timezone must be a validated IANA name");
        let cloud_pg_runtime = CloudPgRuntime::from_cloud_config(&config.cloud)
            .expect("PostgreSQL must be configured for the runtime");
        #[cfg(test)]
        let cloud_pg_runtime = isolated_test_postgres(cloud_pg_runtime).await;
        let session_storage =
            SessionStorage::new_cloud(&config.storage.sessions_dir, cloud_pg_runtime.clone())
                .await
                .expect("failed to initialize PostgreSQL session storage");
        let conversation_quota_storage =
            ConversationQuotaStorage::new_cloud(cloud_pg_runtime.clone())
                .await
                .expect("failed to initialize PostgreSQL conversation quota storage");
        #[cfg(test)]
        let delivered_push_context_store = Some(
            EventStore::new(cloud_pg_runtime.clone())
                .await
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to open isolated PostgreSQL delivered push context store: {err:#}"
                    )
                }),
        );
        #[cfg(not(test))]
        let delivered_push_context_store = match EventStore::new(cloud_pg_runtime.clone()).await {
            Ok(store) => Some(store),
            Err(err) => {
                tracing::warn!("failed to open PostgreSQL delivered push context store: {err:#}");
                None
            }
        };
        #[cfg(not(test))]
        {
            configure_cloud_skill_registry(Some(cloud_pg_runtime.clone()));
            configure_cloud_notification_prefs(Some(cloud_pg_runtime.clone()));
            configure_cloud_portfolio_storage(Some(cloud_pg_runtime.clone()));
            hone_memory::configure_cloud_survey_storage(Some(cloud_pg_runtime.clone()));
            configure_cloud_company_profile_storage(Some(cloud_pg_runtime.clone()));
        }
        #[cfg(test)]
        {
            configure_cloud_skill_registry(None);
            configure_cloud_notification_prefs(None);
            configure_cloud_portfolio_storage(None);
            hone_memory::configure_cloud_survey_storage(None);
            configure_cloud_company_profile_storage(None);
        }
        let company_profile_storage = CompanyProfileStorage::new(sandbox_base_dir());
        let llm = Self::create_llm_provider(&config);
        let auxiliary_llm = Self::create_auxiliary_llm_provider(&config);
        let llm_audit = Self::create_llm_audit_sink(&config, cloud_pg_runtime.clone()).await;
        let workflow_runner_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|err| {
                tracing::warn!("failed to create workflow runner HTTP client: {}", err);
                reqwest::Client::new()
            });

        Self {
            config,
            cloud_pg_runtime,
            llm,
            auxiliary_llm,
            llm_audit,
            session_storage,
            conversation_quota_storage,
            delivered_push_context_store,
            workflow_runner_http,
            company_profile_storage,
            runtime_admin_overrides: RwLock::new(HashSet::new()),
            #[cfg(test)]
            test_runner_factory: None,
        }
    }

    /// 从配置文件创建
    pub async fn from_config_file(path: &str) -> hone_core::HoneResult<Self> {
        let config = HoneConfig::from_file(path)?;
        Ok(Self::new(config).await)
    }

    /// 创建 LLM Provider
    fn create_llm_provider(config: &HoneConfig) -> Option<Arc<dyn LlmProvider>> {
        if !config.llm.default_profile.trim().is_empty() {
            return match LlmResolver::new(config)
                .provider_for_profile(&config.llm.default_profile, None)
            {
                Ok(created) => Some(created.provider),
                Err(e) => {
                    tracing::warn!("Failed to create default LLM profile provider: {}", e);
                    None
                }
            };
        }

        match LlmResolver::new(config).provider_for_profile_or_openrouter_model(
            None,
            &config.llm.openrouter.model,
            &config.llm.openrouter.model,
            None,
        ) {
            Ok(created) => Some(created.provider),
            Err(e) => {
                tracing::warn!("Failed to create OpenRouter provider: {}", e);
                None
            }
        }
    }

    fn create_auxiliary_llm_provider(config: &HoneConfig) -> Option<Arc<dyn LlmProvider>> {
        match LlmResolver::new(config).auxiliary_provider(Some(&config.llm.auxiliary_profile), None)
        {
            Ok(created) => Some(created.provider),
            Err(err) => {
                tracing::warn!("Failed to create auxiliary provider: {}", err);
                Self::create_llm_provider(config)
            }
        }
    }

    pub fn auxiliary_model_name(&self) -> String {
        let auxiliary_profile = self.config.llm.auxiliary_profile.trim();
        if !auxiliary_profile.is_empty() {
            if let Some(profile) = self.config.llm.profiles.get(auxiliary_profile) {
                if !profile.model.trim().is_empty() {
                    return profile.model.trim().to_string();
                }
            }
        }
        let configured = self.config.llm.auxiliary.model.trim();
        if !configured.is_empty() {
            configured.to_string()
        } else {
            self.config.llm.openrouter.auxiliary_model().to_string()
        }
    }

    pub fn auxiliary_provider_hint(&self) -> (String, String) {
        let auxiliary_profile = self.config.llm.auxiliary_profile.trim();
        if !auxiliary_profile.is_empty() {
            if let Some(profile) = self.config.llm.profiles.get(auxiliary_profile) {
                return (profile.provider.clone(), self.auxiliary_model_name());
            }
        }
        if self.config.llm.auxiliary.is_configured() {
            ("openai-compatible".to_string(), self.auxiliary_model_name())
        } else {
            (
                self.config.llm.provider.clone(),
                self.auxiliary_model_name(),
            )
        }
    }

    async fn create_llm_audit_sink(
        config: &HoneConfig,
        postgres: CloudPgRuntime,
    ) -> Option<Arc<dyn LlmAuditSink>> {
        if !config.storage.llm_audit_enabled {
            return None;
        }

        match LlmAuditStorage::new_cloud(postgres, config.storage.llm_audit_retention_days).await {
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
            // Deleting every cron job does not stop the daily digest. The tool
            // needs the digest schedule to say so instead of reporting an empty
            // job list as "nothing will push you any more".
            let default_digest_slot_times = self
                .config
                .event_engine
                .digest
                .default_slots
                .iter()
                .map(|slot| slot.time.clone())
                .collect::<Vec<_>>();
            let postgres = self.cloud_pg_runtime.clone();
            let cron_tool: Box<dyn hone_tools::Tool> = Box::new(
                CronJobTool::new_cloud(
                    &self.config.storage.cron_jobs_dir,
                    actor.cloned(),
                    channel_target,
                    admin_bypass,
                    postgres,
                )
                .with_push_context(
                    &self.config.storage.notif_prefs_dir,
                    default_digest_slot_times.clone(),
                ),
            );
            registry.register(cron_tool);
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
        let overview_defaults = hone_tools::schedule_view::NotificationOverviewDefaults {
            slots: self
                .config
                .event_engine
                .digest
                .default_slots
                .iter()
                .map(|s| hone_tools::schedule_view::DigestDefaultSlot {
                    time: s.time.clone(),
                    label: s.label.clone(),
                })
                .collect(),
            price_alert: hone_tools::schedule_view::PriceAlertPolicyDefaults::from(
                &self.config.event_engine.thresholds,
            ),
            event_engine_enabled: self.config.event_engine.enabled,
            globally_disabled_kinds: self.config.event_engine.disabled_kinds.clone(),
            high_severity_daily_cap: self.config.event_engine.thresholds.high_severity_daily_cap,
            same_symbol_cooldown_minutes: self
                .config
                .event_engine
                .thresholds
                .same_symbol_cooldown_minutes,
        };
        let postgres = self.cloud_pg_runtime.clone();
        registry.register(Box::new(hone_tools::NotificationPrefsTool::new_cloud(
            &self.config.storage.notif_prefs_dir,
            actor.cloned(),
            &self.config.storage.cron_jobs_dir,
            overview_defaults,
            postgres,
        )));

        // 让用户通过 `/missed` 或自然语言查回 digest/router 主动筛掉的事件。
        // 与 event-engine 共用 PostgreSQL 权威表；actor 强制绑定调用方。
        registry.register(Box::new(hone_tools::MissedEventsTool::new(
            self.cloud_pg_runtime.clone(),
            actor.cloned(),
        )));

        if let Some(actor) = actor.cloned() {
            let sandbox_base = sandbox_base_dir();
            if let Some(oss) =
                hone_core::cloud_runtime::OssObjectStore::from_config(&self.config.cloud.oss)
            {
                registry.register(Box::new(hone_tools::LocalListFilesTool::new_cloud(
                    sandbox_base.clone(),
                    actor.clone(),
                    oss.clone(),
                )));
                registry.register(Box::new(hone_tools::LocalSearchFilesTool::new_cloud(
                    sandbox_base.clone(),
                    actor.clone(),
                    oss.clone(),
                )));
                registry.register(Box::new(hone_tools::LocalReadFileTool::new_cloud(
                    sandbox_base,
                    actor,
                    oss,
                )));
            } else {
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
        let path = std::env::var_os("HONE_SKILLS_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                self.config
                    .extra
                    .get("skills_dir")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("./skills"));
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
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

    pub fn configured_event_store_path(&self) -> PathBuf {
        configured_event_store_path(&self.config)
    }

    pub fn configured_runtime_dir(&self) -> PathBuf {
        hone_core::runtime_heartbeat_dir(&self.config)
    }

    pub fn configured_skill_registry_path(&self) -> PathBuf {
        self.configured_runtime_dir().join("skill_registry.json")
    }

    pub(crate) fn enabled_skill_directories(&self, workspace: &Path) -> Vec<(String, PathBuf)> {
        hone_tools::SkillRuntime::new(
            self.configured_system_skills_dir(),
            self.configured_custom_skills_dir(),
            workspace.to_path_buf(),
        )
        .with_registry_path(self.configured_skill_registry_path())
        .list_summaries()
        .into_iter()
        .filter_map(|skill| {
            let skill_path = PathBuf::from(skill.detail_path);
            skill_path
                .parent()
                .map(Path::to_path_buf)
                .map(|skill_dir| (skill.id, skill_dir))
        })
        .collect()
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
    pub async fn create_scheduler(
        &self,
        channels: Vec<String>,
    ) -> (HoneScheduler, mpsc::Receiver<SchedulerEvent>) {
        let storage = Arc::new(self.cron_job_storage().await);
        let (tx, rx) = mpsc::channel(64);
        (HoneScheduler::new(storage, tx, channels), rx)
    }

    pub async fn cron_job_storage(&self) -> CronJobStorage {
        let task_runs_dir = Some(self.configured_runtime_dir());
        CronJobStorage::new_cloud(self.cloud_pg_runtime.clone())
            .await
            .unwrap_or_else(|error| panic!("{}", cloud_cron_storage_failure(&error)))
            .with_task_runs_dir(task_runs_dir)
    }

    /// 创建 Agent runner 实例。
    ///
    /// `AgentSession` 应通过 runner，而不是直接感知底层 provider/CLI 分支。
    ///
    /// 返回 `Err(message)` 表示配置不满足要求或 runner 已被移除。
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
        #[cfg(test)]
        if let Some(factory) = &self.test_runner_factory {
            return Ok(factory());
        }

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
                 请在 config.yaml 的 agent.runner 切换到 codex_acp / opencode_acp / hone_cloud。"
                    .to_string(),
            ),
            AgentRunnerKind::CodexCli => Ok(Box::new(CodexCliReasoningRunner::new(
                system_prompt.to_string(),
                Some(self.config.agent.codex_model.clone()),
                Arc::new(tool_registry),
                self.llm_audit.clone(),
            ))),
            AgentRunnerKind::CodexAcp => {
                let mut codex_config = self.config.agent.codex_acp.clone();
                if let Some(model_override) =
                    model_override.filter(|value| !value.trim().is_empty())
                {
                    codex_config.model = model_override.trim().to_string();
                }
                Ok(Box::new(CodexAcpRunner::new(codex_config, runner_timeouts)))
            }
            AgentRunnerKind::OpencodeAcp => {
                self.create_opencode_runner_with_model_override(tool_registry, model_override)
            }
            AgentRunnerKind::HoneCloud => Ok(Box::new(HoneCloudRunner::new(
                self.config.agent.hone_cloud.clone(),
                runner_timeouts,
            ))),
            AgentRunnerKind::Unknown => {
                let configured = printable_or_default(runner, "<empty>");
                if matches!(runner, "function_calling" | "multi-agent") {
                    Err(format!(
                        "agent runner `{configured}` has been removed; set agent.runner to codex_acp (recommended), opencode_acp, hone_cloud, codex_cli, or gemini_cli"
                    ))
                } else {
                    Err(format!(
                        "unknown agent runner `{configured}`; supported runners: codex_acp, opencode_acp, hone_cloud, codex_cli, gemini_cli"
                    ))
                }
            }
        }
    }

    pub(crate) fn create_opencode_runner_with_model_override(
        &self,
        _tool_registry: ToolRegistry,
        model_override: Option<&str>,
    ) -> Result<Box<dyn AgentRunner>, String> {
        #[cfg(test)]
        if let Some(factory) = &self.test_runner_factory {
            return Ok(factory());
        }

        let mut opencode_config = self.config.agent.opencode.clone();
        if let Some(model_override) = model_override.filter(|value| !value.trim().is_empty()) {
            opencode_config.model = model_override.trim().to_string();
            opencode_config.variant = String::new();
        }
        let hone_manages_opencode_route = !opencode_config.model.trim().is_empty()
            || !opencode_config.variant.trim().is_empty()
            || !opencode_config.api_base_url.trim().is_empty()
            || !opencode_config.api_key.trim().is_empty();
        if hone_manages_opencode_route && opencode_config.api_key.trim().is_empty() {
            let pool = self.config.llm.openrouter_key_pool();
            if let Some(key) = pool.first() {
                opencode_config.openrouter_api_key = Some(key.to_string());
            }
        }
        Ok(Box::new(OpencodeAcpRunner::new(
            opencode_config,
            RunnerTimeouts {
                step: self.config.agent.step_timeout(),
                overall: self.config.agent.overall_timeout(),
            },
        )))
    }

    pub(crate) fn create_openrouter_opencode_runner_with_model_override(
        &self,
        _tool_registry: ToolRegistry,
        model_override: Option<&str>,
    ) -> Result<Box<dyn AgentRunner>, String> {
        #[cfg(test)]
        if let Some(factory) = &self.test_runner_factory {
            return Ok(factory());
        }

        let model = model_override
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "earnings workflow OpenRouter model is missing".to_string())?;
        let pool = self.config.llm.openrouter_key_pool();
        let key = pool.first().ok_or_else(|| {
            "earnings workflow OpenRouter credential is missing from config".to_string()
        })?;
        let mut opencode_config = self.config.agent.opencode.clone();
        opencode_config.model = model.to_string();
        opencode_config.variant = String::new();
        opencode_config.api_base_url = "https://openrouter.ai/api/v1".to_string();
        opencode_config.api_key.clear();
        opencode_config.openrouter_api_key = Some(key.to_string());
        Ok(Box::new(OpencodeAcpRunner::new(
            opencode_config,
            RunnerTimeouts {
                step: self.config.agent.step_timeout(),
                overall: self.config.agent.overall_timeout(),
            },
        )))
    }

    /// Native CLI/ACP runners execute outside Hone's actor-bound tool registry and may
    /// inspect host files or process state. They remain an explicit administrator boundary.
    pub(crate) fn configured_runner_requires_trusted_host_access(&self) -> bool {
        matches!(
            self.config.agent.runner_kind(),
            AgentRunnerKind::GeminiCli
                | AgentRunnerKind::CodexCli
                | AgentRunnerKind::CodexAcp
                | AgentRunnerKind::OpencodeAcp
        )
    }

    /// Whether a configured host-capable runner is replaced by the actor-safe
    /// function-calling runner for this request.  Context ownership must use
    /// this effective route rather than the static configured runner name.
    pub(crate) fn actor_uses_strict_runner_fallback(&self, actor: &ActorIdentity) -> bool {
        !self.is_admin_actor(actor) && self.configured_runner_requires_trusted_host_access()
    }

    pub(crate) fn effective_runner_conversation_strategy(
        &self,
        actor: &ActorIdentity,
    ) -> AgentConversationStrategy {
        if self.actor_uses_strict_runner_fallback(actor) {
            AgentConversationStrategy::StructuredReplay
        } else {
            self.config.agent.runner_kind().conversation_strategy()
        }
    }

    /// A trusted native Codex ACP thread accepts one new user turn at a time
    /// and owns its retained history, tool loop, and compaction. OpenCode ACP
    /// shares the protocol family but still uses a fresh Hone replay and does
    /// not inherit this Codex-specific prompt lifecycle contract.
    pub(crate) fn effective_runner_uses_native_codex_turns(&self, actor: &ActorIdentity) -> bool {
        !self.actor_uses_strict_runner_fallback(actor)
            && self.config.agent.runner_kind() == AgentRunnerKind::CodexAcp
    }

    /// Whether the configured administrator runner owns a persistent Codex
    /// conversation. Web authentication can establish administrator trust
    /// after `ActorIdentity` is created, so turn preparation cannot infer this
    /// solely from the actor whitelist in the static runtime config.
    pub(crate) fn configured_runner_uses_native_codex_turns(&self) -> bool {
        self.config.agent.runner_kind() == AgentRunnerKind::CodexAcp
    }

    pub(crate) fn create_strict_actor_runner(
        &self,
        system_prompt: &str,
        tool_registry: ToolRegistry,
    ) -> Result<Box<dyn AgentRunner>, String> {
        let llm = self.llm.clone().ok_or_else(|| {
            "安全执行器不可用：普通用户不能使用具备宿主机访问能力的 CLI/ACP，且严格 function-calling LLM 未配置。"
                .to_string()
        })?;
        Ok(Box::new(FunctionCallingReasoningRunner::new(
            llm,
            Arc::new(tool_registry),
            system_prompt.to_string(),
            STRICT_ACTOR_MAX_ITERATIONS,
            self.llm_audit.clone(),
            RunnerTimeouts {
                step: self.config.agent.step_timeout(),
                overall: self.config.agent.overall_timeout(),
            },
        )))
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
    /// 对于本轮实际使用且持有原生会话 / 内置 compact 的 runner
    ///（当前为 codex_acp），直接短路返回。普通 Web actor 即使全局
    /// 配置了 ACP，也会安全降级到 function-calling，因此仍必须由 Hone
    /// 管理上下文。
    /// 见 `docs/bugs/session_compact_summary_report_hallucination.md` 2026-04-23 决策。
    pub async fn maybe_compress_session(
        &self,
        session_id: &str,
        actor: &ActorIdentity,
    ) -> hone_core::HoneResult<()> {
        if self
            .effective_runner_conversation_strategy(actor)
            .retains_native_history()
        {
            tracing::debug!(
                "[Compact] session={} runner={} 自管上下文，跳过 honeclaw 自动 compact",
                session_id,
                self.config.agent.runner
            );
            return Ok(());
        }
        let _ = SessionCompactor::new(self)
            .compact_session(session_id, "auto", false, None, true)
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
            .compact_session(session_id, trigger, force, user_instructions, false)
            .await
    }
}

fn cloud_cron_storage_failure(error: &impl std::fmt::Display) -> String {
    format!("PostgreSQL cron 存储初始化失败: {error}")
}

fn configured_event_store_path(config: &HoneConfig) -> PathBuf {
    if let Ok(root) = std::env::var("HONE_DATA_DIR") {
        return PathBuf::from(root).join("event-store");
    }
    PathBuf::from(&config.storage.sessions_dir)
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./data"))
        .join("event-store")
}

#[cfg(test)]
mod cron_storage_tests {
    use super::cloud_cron_storage_failure;

    #[test]
    fn postgres_failure_includes_original_error_without_claiming_a_fallback() {
        let warning = cloud_cron_storage_failure(&"PG schema unavailable");

        assert_eq!(
            warning,
            "PostgreSQL cron 存储初始化失败: PG schema unavailable"
        );
        assert!(!warning.contains("降级"));
    }
}
