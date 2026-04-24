//! `AgentSession` struct + 所有实例方法 + per-session 运行锁。
//!
//! 这个文件是 module 里的「大脑」:把 types / helpers / emitter / guard /
//! restore / progress 这些零件组合成「一次完整对话」的 pipeline。
//! 详细的 pipeline 步骤见 [`AgentSession::run`] 顶部的分节注释。

use hone_core::agent::{AgentContext, AgentMessage, AgentResponse};
use hone_core::{ActorIdentity, SessionIdentity};
use hone_memory::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, session_message_from_normalized,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::HoneBotCore;
use crate::execution::{
    ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService, PreparedExecution,
};
use crate::prompt::PromptOptions;
use crate::prompt_audit::PromptAuditMetadata;
use crate::response_finalizer::{EMPTY_SUCCESS_FALLBACK_MESSAGE, finalize_agent_response};
use crate::runners::{AgentRunnerEmitter, AgentRunnerRequest, AgentRunnerResult};
use crate::session_compactor::SessionCompactor;
use crate::turn_builder::{PromptTurnBuilder, SlashSkillExpansion};

use super::emitter::SessionEventEmitter;
use super::guard::QuotaReservationGuard;
use super::helpers::{
    CONTEXT_OVERFLOW_FALLBACK_MESSAGE, CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT,
    CONTEXT_OVERFLOW_RECOVERY_LIMIT, CompactCommand, EMPTY_SUCCESS_RETRY_LIMIT,
    is_context_overflow_error_text, merge_message_metadata, persistable_turn_from_response,
    restore_limit_before_compaction, should_return_runner_result,
};
use super::progress::{progress_watchdog_tick, run_with_progress_ticks};
use super::restore::restore_context;
use super::types::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionError, AgentSessionErrorKind,
    AgentSessionEvent, AgentSessionListener, AgentSessionResult, GeminiStreamOptions,
    MessageMetadata, session_error_event, session_progress_event,
};

pub struct AgentSession {
    pub(super) core: Arc<HoneBotCore>,
    pub(super) actor: ActorIdentity,
    pub(super) session_identity: SessionIdentity,
    pub(super) session_id: String,
    pub(super) channel_target: String,
    pub(super) message_id: Option<String>,
    pub(super) restore_max_messages: Option<usize>,
    pub(super) prompt_options: PromptOptions,
    pub(super) session_metadata: Option<HashMap<String, Value>>,
    pub(super) message_metadata: MessageMetadata,
    pub(super) listeners: Vec<Arc<dyn AgentSessionListener>>,
    pub(super) recv_extra: Option<String>,
    pub(super) allow_cron: bool,
}

/// 统一串行化同一 session 的整次 run，避免多个入口同时 restore_context + run 时共享旧快照。
///
/// 使用 `Weak` 引用存储锁，当没有任何调用方持有该 session 的锁时，Map 中的条目
/// 会在下次访问时被自然替换，避免长期运行后 HashMap 无限增长。
static SESSION_RUN_LOCKS: OnceLock<
    Mutex<HashMap<String, std::sync::Weak<tokio::sync::Mutex<()>>>>,
> = OnceLock::new();

fn get_session_run_lock(session_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    let map = SESSION_RUN_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().expect("session run lock poisoned");
    guard.retain(|_, weak| weak.upgrade().is_some());
    // 尝试从已有的 Weak 引用升级；若失败（已无持有者）则创建新锁并覆盖旧条目
    if let Some(existing) = guard.get(session_id).and_then(|w| w.upgrade()) {
        return existing;
    }
    let lock = Arc::new(tokio::sync::Mutex::new(()));
    guard.insert(session_id.to_string(), Arc::downgrade(&lock));
    lock
}

impl AgentSession {
    pub(super) async fn run_runner_with_empty_success_retry(
        &self,
        runner: &dyn crate::runners::AgentRunner,
        runner_name: &str,
        session_id: &str,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let mut last_result = self
            .run_runner_with_progress_watchdog(
                runner,
                runner_name,
                session_id,
                0,
                request.clone(),
                emitter.clone(),
            )
            .await;

        for retry_idx in 0..EMPTY_SUCCESS_RETRY_LIMIT {
            // 如果运行失败，或者已经拿到了正文/工具调用，则不重试。
            // 对“支持流式但本次没有任何输出”的 runner，继续走空回复兜底逻辑。
            if should_return_runner_result(&last_result) {
                return last_result;
            }

            let attempt = retry_idx + 1;
            tracing::warn!(
                "[AgentSession] empty successful response, retrying runner={} session_id={} attempt={}/{}",
                runner_name,
                session_id,
                attempt,
                EMPTY_SUCCESS_RETRY_LIMIT
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "agent.run.retry",
                &format!("empty_success attempt={attempt}/{EMPTY_SUCCESS_RETRY_LIMIT}"),
                self.message_id.as_deref(),
                None,
            );
            self.emit(session_progress_event(
                "agent.run.retry",
                Some(format!(
                    "{runner_name} empty_success attempt={attempt}/{EMPTY_SUCCESS_RETRY_LIMIT}"
                )),
            ))
            .await;

            last_result = self
                .run_runner_with_progress_watchdog(
                    runner,
                    runner_name,
                    session_id,
                    attempt,
                    request.clone(),
                    emitter.clone(),
                )
                .await;
        }

        if last_result.response.success && last_result.response.content.trim().is_empty() {
            tracing::warn!(
                "[AgentSession] empty successful response persisted as fallback runner={} session_id={}",
                runner_name,
                session_id
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                session_id,
                "agent.run.fallback",
                "empty_success_exhausted",
                self.message_id.as_deref(),
                None,
            );
            last_result.response.content = EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string();
        }

        last_result
    }

    /// Run the underlying runner while emitting a periodic "still running" heartbeat.
    ///
    /// 背景：`runner.run(...)` 内部会一直驻留到 ACP 会话结束；一旦进入长工具链或上游静默，
    /// 外层除了 `agent.run start` 之外没有任何痕迹，直到整个 run 结束或超时才会再次落日志
    /// （参见 `docs/bugs/feishu_scheduler_run_stuck_without_cron_job_run.md`）。这里用一个
    /// `tokio::select!` ticker 在 run_fut 未完成时定期打 `agent.run.progress`，保证：
    /// - `sidecar.log` 在卡死期间仍有心跳，运维能立刻判定「执行中 vs 卡死」；
    /// - session 可见进度事件 (`session_progress_event`) 同步到 UI/下游，避免客户端以为 run 已失联。
    async fn run_runner_with_progress_watchdog(
        &self,
        runner: &dyn crate::runners::AgentRunner,
        runner_name: &str,
        session_id: &str,
        attempt: usize,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let tick = progress_watchdog_tick();
        let runner_name = runner_name.to_string();
        let session_id_s = session_id.to_string();
        let run_fut = runner.run(request, emitter);
        run_with_progress_ticks(run_fut, tick, |ticks, elapsed| {
            let runner_name = runner_name.clone();
            let session_id = session_id_s.clone();
            let elapsed_s = elapsed.as_secs();
            let detail = if attempt == 0 {
                format!("elapsed_s={elapsed_s} tick={ticks}")
            } else {
                format!("elapsed_s={elapsed_s} tick={ticks} retry_attempt={attempt}")
            };
            tracing::warn!(
                state = "agent_iterating",
                "[AgentSession] agent.run still running runner={} session_id={} {}",
                runner_name,
                session_id,
                detail
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "agent.run.progress",
                &format!("{runner_name} {detail}"),
                self.message_id.as_deref(),
                Some("agent_iterating"),
            );
            async move {
                self.emit(session_progress_event(
                    "agent.run.progress",
                    Some(format!("{runner_name} {detail}")),
                ))
                .await;
            }
        })
        .await
    }

    fn build_skill_runtime(&self) -> hone_tools::SkillRuntime {
        hone_tools::SkillRuntime::new(
            self.core.configured_system_skills_dir(),
            self.core.configured_custom_skills_dir(),
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        )
        .with_registry_path(self.core.configured_skill_registry_path())
    }

    fn restore_runtime_context(
        &self,
        session_id: &str,
        persisted_user_input: &str,
        restore_max_override: Option<usize>,
    ) -> AgentContext {
        let restore_limit = restore_max_override.or(self.restore_max_messages);
        let mut context = restore_context(
            &self.core.session_storage,
            session_id,
            restore_limit,
            Some(&self.build_skill_runtime()),
        );
        context.set_actor_identity(&self.actor);

        if let Some(last) = context.messages.last() {
            if last.role == "user" && last.content.as_deref() == Some(persisted_user_input) {
                context.messages.pop();
            }
        }

        context
    }

    fn prepare_execution_for_turn(
        &self,
        session_id: &str,
        persisted_user_input: &str,
        runtime_user_input: &str,
        options: &AgentRunOptions,
        restore_max_override: Option<usize>,
    ) -> Result<PreparedExecution, (AgentSessionErrorKind, String)> {
        let context =
            self.restore_runtime_context(session_id, persisted_user_input, restore_max_override);
        let (system_prompt, runtime_input) =
            self.resolve_prompt_input(session_id, runtime_user_input);
        ExecutionService::new(self.core.clone())
            .prepare(ExecutionRequest {
                mode: ExecutionMode::PersistentConversation,
                session_id: session_id.to_string(),
                actor: self.actor.clone(),
                channel_target: self.channel_target.clone(),
                allow_cron: self.allow_cron,
                system_prompt,
                runtime_input,
                context,
                timeout: options.timeout,
                gemini_stream: self.default_gemini_stream_options(options.timeout),
                session_metadata: self.load_session_metadata(session_id),
                model_override: options.model_override.clone(),
                runner_selection: ExecutionRunnerSelection::Configured,
                allowed_tools: None,
                max_tool_calls: None,
                prompt_audit: Some(PromptAuditMetadata {
                    session_identity: self.session_identity.clone(),
                    message_id: self.message_id.clone(),
                }),
            })
            .map_err(|err| {
                tracing::error!("[AgentSession] execution prepare failed: {}", err);
                let kind = if err.contains("sandbox") {
                    AgentSessionErrorKind::Io
                } else {
                    AgentSessionErrorKind::AgentFailed
                };
                (kind, err)
            })
    }

    pub(super) fn persist_successful_assistant_turn(
        &self,
        session_id: &str,
        response: &AgentResponse,
        context_messages: Option<&[AgentMessage]>,
    ) {
        let mut metadata = self.message_metadata.assistant.clone();
        if let Some(source) = context_messages.and_then(|messages| {
            messages
                .iter()
                .rfind(|message| message.role == "assistant")
                .and_then(|message| message.metadata.clone())
        }) {
            metadata = merge_message_metadata(metadata, source);
        }

        let Some(message) = persistable_turn_from_response(response, metadata) else {
            return;
        };

        let _ = self.core.session_storage.append_session_messages(
            session_id,
            vec![session_message_from_normalized(
                &message,
                hone_core::beijing_now_rfc3339(),
            )],
        );
    }

    async fn force_compact_for_context_overflow(&self, session_id: &str) -> Result<bool, String> {
        let outcome = SessionCompactor::new(&self.core)
            .compact_session(
                session_id,
                "context_overflow_recovery",
                true,
                Some("优先保留最近用户问题、最近结论、未完成事项，以及继续当前回答所必需的最小上下文。"),
            )
            .await
            .map_err(|err| err.to_string())?;
        Ok(outcome.compacted)
    }

    pub fn new(
        core: Arc<HoneBotCore>,
        actor: ActorIdentity,
        channel_target: impl Into<String>,
    ) -> Self {
        let session_identity = SessionIdentity::from_actor(&actor).unwrap_or_else(|_| {
            SessionIdentity::direct(&actor.channel, &actor.user_id)
                .expect("actor should always map to a direct session")
        });
        let restore_max_messages = restore_limit_before_compaction(&core.config, &session_identity);
        Self {
            core,
            actor,
            session_id: session_identity.session_id(),
            session_identity,
            channel_target: channel_target.into(),
            message_id: None,
            restore_max_messages,
            prompt_options: PromptOptions::default(),
            session_metadata: None,
            message_metadata: MessageMetadata::default(),
            listeners: Vec::new(),
            recv_extra: None,
            allow_cron: true,
        }
    }

    pub fn with_message_id(mut self, message_id: Option<String>) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn with_restore_max_messages(mut self, limit: Option<usize>) -> Self {
        self.restore_max_messages = limit;
        self
    }

    pub fn with_session_identity(mut self, session_identity: SessionIdentity) -> Self {
        self.session_id = session_identity.session_id();
        self.restore_max_messages =
            restore_limit_before_compaction(&self.core.config, &session_identity);
        self.session_identity = session_identity;
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
    }

    pub fn with_prompt_options(mut self, options: PromptOptions) -> Self {
        self.prompt_options = options;
        self
    }

    pub fn with_session_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.session_metadata = Some(metadata);
        self
    }

    pub fn with_message_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.message_metadata = metadata;
        self
    }

    pub fn with_recv_extra(mut self, extra: Option<String>) -> Self {
        self.recv_extra = extra;
        self
    }

    pub fn with_cron_allowed(mut self, allowed: bool) -> Self {
        self.allow_cron = allowed;
        self
    }

    pub fn add_listener(&mut self, listener: Arc<dyn AgentSessionListener>) {
        self.listeners.push(listener);
    }

    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }

    async fn emit(&self, event: AgentSessionEvent) {
        for listener in &self.listeners {
            listener.on_event(event.clone()).await;
        }
    }

    fn ensure_session_exists(&self) -> hone_core::HoneResult<()> {
        let session_id = self.session_id();
        if self
            .core
            .session_storage
            .load_session(&session_id)
            .ok()
            .flatten()
            .is_none()
        {
            self.core
                .session_storage
                .create_session_for_identity(&self.session_identity, Some(&self.actor))?;
        }
        Ok(())
    }

    fn update_session_metadata(&self) {
        let Some(metadata) = self.session_metadata.clone() else {
            return;
        };
        let _ = self
            .core
            .session_storage
            .update_metadata(&self.session_id, metadata);
    }

    pub(super) fn resolve_prompt_input(
        &self,
        session_id: &str,
        user_input: &str,
    ) -> (String, String) {
        let turn = PromptTurnBuilder::new(
            &self.core,
            &self.actor,
            session_id,
            self.prompt_options.clone(),
            self.allow_cron,
            self.recv_extra.as_deref(),
        )
        .resolve_prompt_input(user_input);
        (turn.system_prompt, turn.runtime_input)
    }

    fn expand_slash_skill_input(
        &self,
        session_id: &str,
        user_input: &str,
    ) -> hone_core::HoneResult<Option<SlashSkillExpansion>> {
        PromptTurnBuilder::new(
            &self.core,
            &self.actor,
            session_id,
            self.prompt_options.clone(),
            self.allow_cron,
            self.recv_extra.as_deref(),
        )
        .expand_slash_skill_input(user_input)
    }

    fn parse_compact_command(&self, user_input: &str) -> Option<CompactCommand> {
        let trimmed = user_input.trim();
        let compact = trimmed.strip_prefix("/compact")?;
        if !compact.is_empty() && !compact.starts_with(char::is_whitespace) {
            return None;
        }
        let instructions = compact.trim();
        Some(CompactCommand {
            instructions: (!instructions.is_empty()).then(|| instructions.to_string()),
        })
    }

    fn persist_invoked_skill_prompt(
        &self,
        session_id: &str,
        skill_id: &str,
        prompt: &str,
    ) -> hone_core::HoneResult<()> {
        let existing = self
            .core
            .session_storage
            .load_session(session_id)?
            .map(|session| session.metadata)
            .unwrap_or_default();
        let mut invoked = hone_memory::invoked_skills_from_metadata(&existing)
            .into_iter()
            .filter(|skill| skill.skill_name != skill_id)
            .collect::<Vec<_>>();
        invoked.push(hone_memory::InvokedSkillRecord {
            skill_name: skill_id.to_string(),
            display_name: skill_id.to_string(),
            path: format!("slash:{skill_id}"),
            prompt: prompt.to_string(),
            execution_context: "inline".to_string(),
            allowed_tools: Vec::new(),
            model: None,
            effort: None,
            agent: None,
            loaded_from: "slash".to_string(),
            updated_at: hone_core::beijing_now_rfc3339(),
        });
        let mut metadata = HashMap::new();
        metadata.insert(
            hone_memory::INVOKED_SKILLS_METADATA_KEY.to_string(),
            serde_json::to_value(invoked)
                .map_err(|err| hone_core::HoneError::Serialization(err.to_string()))?,
        );
        let _ = self
            .core
            .session_storage
            .update_metadata(session_id, metadata)?;
        Ok(())
    }

    async fn run_manual_compact(
        &self,
        session_id: String,
        raw_input: &str,
        command: CompactCommand,
    ) -> AgentSessionResult {
        self.core.log_message_received(
            &self.actor.channel,
            &self.actor.user_id,
            &self.channel_target,
            &session_id,
            raw_input,
            self.recv_extra.as_deref(),
            self.message_id.as_deref(),
        );

        self.emit(session_progress_event(
            "session.compress",
            Some("start".to_string()),
        ))
        .await;
        let started = Instant::now();
        let outcome = self
            .core
            .compact_session(&session_id, "manual", true, command.instructions.as_deref())
            .await;

        let response = match outcome {
            Ok(outcome) => {
                self.emit(session_progress_event(
                    "session.compress",
                    Some("done".to_string()),
                ))
                .await;
                let content = if outcome.compacted {
                    "Conversation compacted.".to_string()
                } else {
                    "未执行 compact：当前没有可压缩的会话内容，或压缩器暂不可用。".to_string()
                };
                AgentResponse {
                    content,
                    tool_calls_made: Vec::new(),
                    iterations: 0,
                    success: true,
                    error: None,
                }
            }
            Err(err) => {
                tracing::error!("[AgentSession] manual compact failed: {}", err);
                self.emit(session_progress_event(
                    "session.compress",
                    Some("failed".to_string()),
                ))
                .await;
                AgentResponse {
                    content: String::new(),
                    tool_calls_made: Vec::new(),
                    iterations: 0,
                    success: false,
                    error: Some(err.to_string()),
                }
            }
        };
        let elapsed_ms = started.elapsed().as_millis();

        if response.success {
            self.core.log_message_finished(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &response,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        } else {
            let err = response
                .error
                .clone()
                .unwrap_or_else(|| "manual compact failed".to_string());
            self.core.log_message_failed(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &err,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            self.emit(session_error_event(AgentSessionError {
                kind: AgentSessionErrorKind::AgentFailed,
                message: err,
            }))
            .await;
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        }

        AgentSessionResult {
            response,
            elapsed_ms,
            session_id,
        }
    }

    fn default_gemini_stream_options(&self, timeout: Option<Duration>) -> GeminiStreamOptions {
        GeminiStreamOptions {
            max_iterations: 18,
            overall_timeout: timeout.unwrap_or_else(|| self.core.config.agent.overall_timeout()),
            per_line_timeout: self.core.config.agent.step_timeout(),
        }
    }

    fn runner_emitter(&self, working_directory: String) -> Arc<dyn AgentRunnerEmitter> {
        Arc::new(SessionEventEmitter {
            listeners: self.listeners.clone(),
            channel: self.actor.channel.clone(),
            user_id: self.actor.user_id.clone(),
            session_id: self.session_id.clone(),
            message_id: self.message_id.clone(),
            working_directory,
        })
    }

    fn load_session_metadata(&self, session_id: &str) -> HashMap<String, Value> {
        self.core
            .session_storage
            .load_session(session_id)
            .ok()
            .flatten()
            .map(|session| session.metadata)
            .unwrap_or_default()
    }

    async fn fail_run(
        &self,
        session_id: String,
        kind: AgentSessionErrorKind,
        message: String,
    ) -> AgentSessionResult {
        let error = AgentSessionError {
            kind,
            message: message.clone(),
        };
        self.emit(session_error_event(error.clone())).await;
        AgentSessionResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 0,
                success: false,
                error: Some(message),
            },
            elapsed_ms: 0,
            session_id,
        }
    }

    fn reserve_conversation_quota(
        &self,
        quota_mode: AgentRunQuotaMode,
    ) -> hone_core::HoneResult<Option<ConversationQuotaReservation>> {
        if quota_mode == AgentRunQuotaMode::ScheduledTask {
            return Ok(None);
        }

        let daily_limit = self.core.config.agent.daily_conversation_limit;
        if daily_limit == 0 {
            return Ok(None);
        }

        let is_admin = self.core.is_admin_actor(&self.actor);
        match self
            .core
            .conversation_quota_storage
            .try_reserve_daily_conversation(&self.actor, daily_limit, is_admin)?
        {
            ConversationQuotaReserveResult::Reserved(reservation) => Ok(Some(reservation)),
            ConversationQuotaReserveResult::Bypassed => Ok(None),
            ConversationQuotaReserveResult::Rejected(snapshot) => {
                Err(hone_core::HoneError::Tool(format!(
                    "已达到今日对话上限（{}/{}，北京时间 {}），请明天再试",
                    snapshot.success_count + snapshot.in_flight,
                    snapshot.limit,
                    snapshot.quota_date
                )))
            }
        }
    }

    /// 一次完整的 agent run,职责按顺序:
    ///
    /// 1. 拿下 per-session 串行化锁,防止两次 run 互相读到对方半成品;
    /// 2. 保证 session 存在 + 覆写 session metadata;
    /// 3. 若用户输入是 `/compact` 走 `run_manual_compact` 直接返回;
    /// 4. 预留 daily 配额(由 `QuotaReservationGuard` 负责失败时 release);
    /// 5. 展开可能的 slash skill 并把用户消息落盘(Fast Persist);
    /// 6. 做一次自动 compact 检查(non-fatal);
    /// 7. 组装 execution 并把 runner 跑起来;
    /// 8. 若 runner 报 context overflow,强制 compact 后按更小的 restore limit 再跑一轮;
    /// 9. 成功时:commit 配额、若非流式则按 segmenter 切片发给 listener、
    ///    把 assistant turn 落盘、打 finished 日志;失败时:drop guard 让 release 生效,
    ///    按错误类型翻译 ErrorKind,再 emit Done。
    pub async fn run(&self, user_input: &str, options: AgentRunOptions) -> AgentSessionResult {
        let session_id = self.session_id();
        let _run_guard = {
            let lock = get_session_run_lock(&session_id);
            lock.lock_owned().await
        };
        if let Err(err) = self.ensure_session_exists() {
            return self
                .fail_run(
                    session_id,
                    AgentSessionErrorKind::AgentFailed,
                    err.to_string(),
                )
                .await;
        }

        self.update_session_metadata();

        if let Some(command) = self.parse_compact_command(user_input) {
            return self
                .run_manual_compact(session_id, user_input, command)
                .await;
        }

        // 配额预留；后续任何失败分支都靠 guard 在 drop 时自动把预留释放掉,
        // 不再需要每处都手写 release_daily_conversation。
        let quota_guard = match self.reserve_conversation_quota(options.quota_mode) {
            Ok(reservation) => QuotaReservationGuard::new(self.core.clone(), reservation),
            Err(err) => {
                return self
                    .fail_run(
                        session_id,
                        AgentSessionErrorKind::AgentFailed,
                        err.to_string(),
                    )
                    .await;
            }
        };

        let slash_skill = match self.expand_slash_skill_input(&session_id, user_input) {
            Ok(value) => value,
            Err(err) => {
                // quota_guard 在 return 的 drop 中自动 release
                drop(quota_guard);
                return self
                    .fail_run(
                        session_id,
                        AgentSessionErrorKind::AgentFailed,
                        err.to_string(),
                    )
                    .await;
            }
        };
        let persisted_user_input = slash_skill
            .as_ref()
            .map(|skill| skill.raw_input.as_str())
            .unwrap_or(user_input);
        let runtime_user_input = slash_skill
            .as_ref()
            .map(|skill| skill.runtime_input.as_str())
            .unwrap_or(user_input);
        let user_metadata = if let Some(skill) = &slash_skill {
            let mut extra = HashMap::new();
            extra.insert(
                hone_memory::SLASH_SKILL_METADATA_KEY.to_string(),
                Value::String(skill.skill_id.clone()),
            );
            merge_message_metadata(self.message_metadata.user.clone(), extra)
        } else {
            self.message_metadata.user.clone()
        };

        // ── Fast Persist: 立即写入用户消息 ──
        // 确保 ensureHistory 轮询时 DB 里已有此消息，避免前端因为竞态丢失消息显示
        let _ = self.core.session_storage.add_message(
            &session_id,
            "user",
            persisted_user_input,
            user_metadata,
        );
        if let Some(skill) = &slash_skill {
            let _ = self.persist_invoked_skill_prompt(
                &session_id,
                &skill.skill_id,
                &skill.invoked_prompt,
            );
        }
        self.emit(AgentSessionEvent::UserMessage {
            content: persisted_user_input.to_string(),
        })
        .await;
        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "session.persist_user",
            "done",
            self.message_id.as_deref(),
            None,
        );

        self.core.log_message_received(
            &self.actor.channel,
            &self.actor.user_id,
            &self.channel_target,
            &session_id,
            persisted_user_input,
            self.recv_extra.as_deref(),
            self.message_id.as_deref(),
        );

        self.emit(session_progress_event(
            "session.compress",
            Some("start".to_string()),
        ))
        .await;

        if let Err(err) = self.core.maybe_compress_session(&session_id).await {
            tracing::error!("[AgentSession] compress failed: {}", err);
            self.emit(session_progress_event(
                "session.compress",
                Some("failed".to_string()),
            ))
            .await;
        } else {
            self.emit(session_progress_event(
                "session.compress",
                Some("done".to_string()),
            ))
            .await;
        }

        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "agent.prepare",
            "restore_context + build_prompt + create_runner",
            self.message_id.as_deref(),
            None,
        );

        if !self.core.runner_supports_strict_actor_sandbox() {
            let message = self
                .core
                .strict_actor_sandbox_guard_message()
                .unwrap_or("当前 runner 不支持严格 actor sandbox。");
            tracing::error!("[AgentSession] strict actor sandbox guard: {}", message);
            drop(quota_guard);
            return self
                .fail_run(
                    session_id,
                    AgentSessionErrorKind::AgentFailed,
                    message.to_string(),
                )
                .await;
        }

        let mut execution = match self.prepare_execution_for_turn(
            &session_id,
            persisted_user_input,
            runtime_user_input,
            &options,
            None,
        ) {
            Ok(execution) => execution,
            Err((kind, err)) => {
                drop(quota_guard);
                return self.fail_run(session_id, kind, err).await;
            }
        };

        self.core.log_message_step(
            &self.actor.channel,
            &self.actor.user_id,
            &session_id,
            "agent.run",
            "start",
            self.message_id.as_deref(),
            None,
        );
        self.emit(session_progress_event(
            "agent.run",
            Some(execution.runner_name.to_string()),
        ))
        .await;
        let started = Instant::now();
        let mut streamed_output = false;
        let mut terminal_error_emitted = false;
        let mut context_messages: Option<Vec<AgentMessage>> = None;
        let mut response = AgentResponse {
            content: String::new(),
            tool_calls_made: Vec::new(),
            iterations: 0,
            success: false,
            error: None,
        };
        for recovery_idx in 0..=CONTEXT_OVERFLOW_RECOVERY_LIMIT {
            let runner_emitter =
                self.runner_emitter(execution.runner_request.working_directory.clone());
            let runner_result = self
                .run_runner_with_empty_success_retry(
                    execution.runner.as_ref(),
                    execution.runner_name,
                    &session_id,
                    execution.runner_request.clone(),
                    runner_emitter,
                )
                .await;
            streamed_output = runner_result.streamed_output;
            terminal_error_emitted = runner_result.terminal_error_emitted;
            if !runner_result.session_metadata_updates.is_empty() {
                let _ = self
                    .core
                    .session_storage
                    .update_metadata(&session_id, runner_result.session_metadata_updates.clone());
            }
            context_messages = runner_result.context_messages;
            response = runner_result.response;

            let should_try_recovery = !response.success
                && response
                    .error
                    .as_deref()
                    .is_some_and(is_context_overflow_error_text)
                && recovery_idx < CONTEXT_OVERFLOW_RECOVERY_LIMIT;
            if !should_try_recovery {
                break;
            }

            tracing::warn!(
                "[AgentSession] context overflow detected, compacting and retrying runner={} session_id={} attempt={}/{}",
                execution.runner_name,
                session_id,
                recovery_idx + 1,
                CONTEXT_OVERFLOW_RECOVERY_LIMIT
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "agent.run.retry",
                &format!(
                    "context_overflow attempt={}/{}",
                    recovery_idx + 1,
                    CONTEXT_OVERFLOW_RECOVERY_LIMIT
                ),
                self.message_id.as_deref(),
                None,
            );
            self.emit(session_progress_event(
                "agent.run.retry",
                Some(format!(
                    "{} context_overflow attempt={}/{}",
                    execution.runner_name,
                    recovery_idx + 1,
                    CONTEXT_OVERFLOW_RECOVERY_LIMIT
                )),
            ))
            .await;

            match self.force_compact_for_context_overflow(&session_id).await {
                Ok(compacted) => {
                    tracing::info!(
                        "[AgentSession] context overflow recovery compacted={} session_id={}",
                        compacted,
                        session_id
                    );
                }
                Err(err) => {
                    tracing::error!(
                        "[AgentSession] context overflow recovery compact failed session_id={} err={}",
                        session_id,
                        err
                    );
                    response.error = Some(CONTEXT_OVERFLOW_FALLBACK_MESSAGE.to_string());
                    break;
                }
            }

            execution = match self.prepare_execution_for_turn(
                &session_id,
                persisted_user_input,
                runtime_user_input,
                &options,
                Some(CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT),
            ) {
                Ok(execution) => execution,
                Err((_kind, err)) => {
                    tracing::error!(
                        "[AgentSession] context overflow recovery prepare failed session_id={} err={}",
                        session_id,
                        err
                    );
                    response.success = false;
                    response.error = Some(CONTEXT_OVERFLOW_FALLBACK_MESSAGE.to_string());
                    break;
                }
            };
        }

        if !response.success
            && response
                .error
                .as_deref()
                .is_some_and(is_context_overflow_error_text)
        {
            response.error = Some(CONTEXT_OVERFLOW_FALLBACK_MESSAGE.to_string());
        }
        let finalize_outcome = finalize_agent_response(
            &self.core,
            &session_id,
            &execution.runner_name,
            &mut response,
        );
        if let Some(reason) = finalize_outcome.fallback_reason {
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "agent.run.fallback",
                reason,
                self.message_id.as_deref(),
                None,
            );
        }
        let elapsed_ms = started.elapsed().as_millis();

        if response.success {
            // 成功路径：主动 commit 把预留转成当日计数,并消耗 guard 阻止
            // 后续 drop 再执行 release。
            quota_guard.commit();
            if !streamed_output {
                if let Some(segmenter) = options.segmenter.as_ref() {
                    let segments = segmenter(&response.content);
                    for seg in segments {
                        self.emit(AgentSessionEvent::Segment { text: seg }).await;
                    }
                }
            }
            self.persist_successful_assistant_turn(
                &session_id,
                &response,
                context_messages.as_deref(),
            );
            self.core.log_message_step(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                "session.persist_assistant",
                "done",
                self.message_id.as_deref(),
                None,
            );
            self.core.log_message_finished(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &response,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        } else {
            // 失败路径：显式 drop 触发 release,让配额回到预留前的状态。
            drop(quota_guard);
            let err = response
                .error
                .clone()
                .unwrap_or_else(|| "未知错误".to_string());
            let kind = if err.contains("agent_timeout") {
                AgentSessionErrorKind::AgentTimeout
            } else if err == CONTEXT_OVERFLOW_FALLBACK_MESSAGE {
                AgentSessionErrorKind::ContextWindowOverflow
            } else {
                AgentSessionErrorKind::AgentFailed
            };
            self.core.log_message_failed(
                &self.actor.channel,
                &self.actor.user_id,
                &session_id,
                &err,
                elapsed_ms,
                self.message_id.as_deref(),
            );
            if !terminal_error_emitted {
                self.emit(session_error_event(AgentSessionError {
                    kind,
                    message: err,
                }))
                .await;
            }
            self.emit(AgentSessionEvent::Done {
                response: response.clone(),
            })
            .await;
        }

        AgentSessionResult {
            response,
            elapsed_ms,
            session_id,
        }
    }
}
