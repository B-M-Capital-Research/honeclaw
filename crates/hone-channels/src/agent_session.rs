//! Agent session abstraction shared across channels.

use async_trait::async_trait;
use hone_core::agent::{AgentContext, AgentResponse};
use hone_core::{ActorIdentity, SessionIdentity, runtime_heartbeat_dir};
use hone_memory::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, SessionStorage,
    build_tool_message_metadata, restore_tool_message, select_context_messages,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::HoneBotCore;
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult};
use crate::sandbox::ensure_actor_sandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionErrorKind {
    SpawnFailed,
    StdoutUnavailable,
    TimeoutOverall,
    TimeoutPerLine,
    GeminiError,
    ContextWindowOverflow,
    Io,
    ExitFailure,
    AgentTimeout,
    AgentFailed,
}

#[derive(Debug, Clone)]
pub struct AgentSessionError {
    pub kind: AgentSessionErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum AgentSessionEvent {
    Progress {
        stage: &'static str,
        detail: Option<String>,
    },
    UserMessage {
        content: String,
    },
    StreamDelta {
        content: String,
    },
    StreamThought {
        thought: String,
    },
    ToolStatus {
        tool: String,
        status: String,
        message: Option<String>,
        reasoning: Option<String>,
    },
    Segment {
        text: String,
    },
    Done {
        response: AgentResponse,
    },
    Error {
        error: AgentSessionError,
    },
}

#[async_trait]
pub trait AgentSessionListener: Send + Sync {
    async fn on_event(&self, event: AgentSessionEvent);
}

#[derive(Debug, Clone, Default)]
pub struct MessageMetadata {
    pub user: Option<HashMap<String, Value>>,
    pub assistant: Option<HashMap<String, Value>>,
}

#[derive(Clone)]
pub struct AgentRunOptions {
    pub timeout: Option<Duration>,
    pub segmenter: Option<Arc<dyn Fn(&str) -> Vec<String> + Send + Sync>>,
    pub quota_mode: AgentRunQuotaMode,
    pub model_override: Option<String>,
}

impl Default for AgentRunOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            segmenter: None,
            quota_mode: AgentRunQuotaMode::UserConversation,
            model_override: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentRunQuotaMode {
    #[default]
    UserConversation,
    ScheduledTask,
}

#[derive(Clone)]
pub struct GeminiStreamOptions {
    pub max_iterations: u32,
    pub overall_timeout: Duration,
    pub per_line_timeout: Duration,
}

impl Default for GeminiStreamOptions {
    fn default() -> Self {
        Self {
            max_iterations: 18,
            overall_timeout: Duration::from_secs(600),
            per_line_timeout: Duration::from_secs(180),
        }
    }
}

const DAILY_CONVERSATION_LIMIT: u32 = 20;
const EMPTY_SUCCESS_RETRY_LIMIT: usize = 2;
const EMPTY_SUCCESS_FALLBACK_MESSAGE: &str =
    "这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。";

pub struct AgentSessionResult {
    pub response: AgentResponse,
    pub elapsed_ms: u128,
    pub session_id: String,
}

pub struct AgentSession {
    core: Arc<HoneBotCore>,
    actor: ActorIdentity,
    session_identity: SessionIdentity,
    session_id: String,
    channel_target: String,
    message_id: Option<String>,
    restore_max_messages: Option<usize>,
    prompt_options: PromptOptions,
    session_metadata: Option<HashMap<String, Value>>,
    message_metadata: MessageMetadata,
    listeners: Vec<Arc<dyn AgentSessionListener>>,
    recv_extra: Option<String>,
    allow_cron: bool,
}

/// 统一串行化同一 session 的整次 run，避免多个入口同时 restore_context + run 时共享旧快照。
static SESSION_RUN_LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> =
    OnceLock::new();

fn get_session_run_lock(session_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    let map = SESSION_RUN_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().expect("session run lock poisoned");
    if let Some(lock) = guard.get(session_id) {
        lock.clone()
    } else {
        let lock = Arc::new(tokio::sync::Mutex::new(()));
        guard.insert(session_id.to_string(), lock.clone());
        lock
    }
}

struct SessionEventEmitter {
    listeners: Vec<Arc<dyn AgentSessionListener>>,
}

#[async_trait]
impl AgentRunnerEmitter for SessionEventEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        let mapped = match event {
            AgentRunnerEvent::Progress { stage, detail } => {
                AgentSessionEvent::Progress { stage, detail }
            }
            AgentRunnerEvent::StreamDelta { content } => AgentSessionEvent::StreamDelta { content },
            AgentRunnerEvent::StreamThought { thought } => {
                AgentSessionEvent::StreamThought { thought }
            }
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => AgentSessionEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            },
            AgentRunnerEvent::Error { error } => AgentSessionEvent::Error { error },
        };

        for listener in &self.listeners {
            listener.on_event(mapped.clone()).await;
        }
    }
}

impl AgentSession {
    async fn run_runner_with_empty_success_retry(
        &self,
        runner: &dyn crate::runners::AgentRunner,
        runner_name: &str,
        session_id: &str,
        request: AgentRunnerRequest,
    ) -> AgentRunnerResult {
        let mut last_result = runner.run(request.clone(), self.runner_emitter()).await;

        for retry_idx in 0..EMPTY_SUCCESS_RETRY_LIMIT {
            // 如果运行失败，或者内容不为空，或者已经产生了流式输出/工具调用，则不重试。
            // 理由：agent 指令可能只包含工具调用而没有最终回复文本；
            // 且如果已经有了流式输出，重试会导致前端看到重复的消息（Triple Message Bug）。
            if !last_result.response.success
                || !last_result.response.content.trim().is_empty()
                || !last_result.response.tool_calls_made.is_empty()
                || last_result.streamed_output
            {
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
            self.emit(AgentSessionEvent::Progress {
                stage: "agent.run.retry",
                detail: Some(format!(
                    "{runner_name} empty_success attempt={attempt}/{EMPTY_SUCCESS_RETRY_LIMIT}"
                )),
            })
            .await;

            last_result = runner.run(request.clone(), self.runner_emitter()).await;
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

    pub fn new(
        core: Arc<HoneBotCore>,
        actor: ActorIdentity,
        channel_target: impl Into<String>,
    ) -> Self {
        let session_identity = SessionIdentity::from_actor(&actor).unwrap_or_else(|_| {
            SessionIdentity::direct(&actor.channel, &actor.user_id)
                .expect("actor should always map to a direct session")
        });
        let restore_max_messages = if session_identity.is_group() {
            Some(core.config.group_context.recent_context_limit.max(1))
        } else {
            Some(12)
        };
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
        self.restore_max_messages = if session_identity.is_group() {
            Some(self.core.config.group_context.recent_context_limit.max(1))
        } else {
            self.restore_max_messages
        };
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

    fn resolve_prompt_input(&self, session_id: &str, user_input: &str) -> (String, String) {
        let mut prompt_options = self.prompt_options.clone();
        if self.allow_cron {
            prompt_options.extra_sections.push(
                crate::prompt::DEFAULT_CRON_TASK_POLICY.to_string(),
            );
        }
        let prompt_state = self
            .core
            .session_storage
            .ensure_prompt_state(session_id)
            .ok()
            .flatten()
            .unwrap_or_default();
        let bundle = build_prompt_bundle(
            &self.core.config,
            &self.core.session_storage,
            &self.actor.channel,
            session_id,
            &prompt_state,
            &prompt_options,
        );
        (
            bundle.system_prompt(),
            bundle.compose_user_input(user_input),
        )
    }

    fn audit_effective_prompt(&self, session_id: &str, system_prompt: &str, runtime_input: &str) {
        let runtime_dir = runtime_heartbeat_dir(&self.core.config);
        let audit_dir = runtime_dir
            .join("prompt-audit")
            .join(sanitize_prompt_audit_path(&self.actor.channel));
        if let Err(err) = fs::create_dir_all(&audit_dir) {
            tracing::warn!(
                "[PromptAudit] failed to create audit dir for channel={} session_id={}: {}",
                self.actor.channel,
                session_id,
                err
            );
            return;
        }

        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let session_slug = sanitize_prompt_audit_path(session_id);
        let prompt_path = audit_dir.join(format!("{timestamp}-{session_slug}.json"));
        let latest_path = audit_dir.join(format!("latest-{session_slug}.json"));
        let payload = serde_json::json!({
            "created_at": chrono::Local::now().to_rfc3339(),
            "channel": self.actor.channel,
            "actor_user_id": self.actor.user_id,
            "session_id": session_id,
            "session_identity": self.session_identity,
            "message_id": self.message_id,
            "system_prompt": system_prompt,
            "runtime_input": runtime_input,
        });

        let content = match serde_json::to_vec_pretty(&payload) {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!(
                    "[PromptAudit] failed to encode payload for channel={} session_id={}: {}",
                    self.actor.channel,
                    session_id,
                    err
                );
                return;
            }
        };

        for path in [prompt_path, latest_path] {
            if let Err(err) = fs::write(&path, &content) {
                tracing::warn!("[PromptAudit] failed to write {}: {}", path.display(), err);
            }
        }
    }

    fn default_gemini_stream_options(&self, timeout: Option<Duration>) -> GeminiStreamOptions {
        let timeout_secs = timeout
            .unwrap_or_else(|| Duration::from_secs(self.core.config.llm.openrouter.timeout))
            .as_secs()
            .clamp(180, 300);
        GeminiStreamOptions {
            max_iterations: 18,
            overall_timeout: Duration::from_secs(timeout_secs),
            per_line_timeout: Duration::from_secs(90),
        }
    }

    fn runner_emitter(&self) -> Arc<dyn AgentRunnerEmitter> {
        Arc::new(SessionEventEmitter {
            listeners: self.listeners.clone(),
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
        self.emit(AgentSessionEvent::Error {
            error: error.clone(),
        })
        .await;
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

        let is_admin = self.core.is_admin_actor(&self.actor);
        match self
            .core
            .conversation_quota_storage
            .try_reserve_daily_conversation(&self.actor, DAILY_CONVERSATION_LIMIT, is_admin)?
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

    pub async fn run(&self, user_input: &str, options: AgentRunOptions) -> AgentSessionResult {
        let session_id = self.session_id();
        let _run_guard = {
            let lock = get_session_run_lock(&session_id);
            lock.lock_owned().await
        };
        let quota_reservation = match self.reserve_conversation_quota(options.quota_mode) {
            Ok(reservation) => reservation,
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

        if let Err(err) = self.ensure_session_exists() {
            if let Some(reservation) = quota_reservation.as_ref() {
                let _ = self
                    .core
                    .conversation_quota_storage
                    .release_daily_conversation(reservation);
            }
            return self
                .fail_run(
                    session_id,
                    AgentSessionErrorKind::AgentFailed,
                    err.to_string(),
                )
                .await;
        }

        self.update_session_metadata();

        // ── Fast Persist: 立即写入用户消息 ──
        // 确保 ensureHistory 轮询时 DB 里已有此消息，避免前端因为竞态丢失消息显示
        let _ = self.core.session_storage.add_message(
            &session_id,
            "user",
            user_input,
            self.message_metadata.user.clone(),
        );
        self.emit(AgentSessionEvent::UserMessage {
            content: user_input.to_string(),
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
            user_input,
            self.recv_extra.as_deref(),
            self.message_id.as_deref(),
        );

        self.emit(AgentSessionEvent::Progress {
            stage: "session.compress",
            detail: Some("start".to_string()),
        })
        .await;

        if let Err(err) = self.core.maybe_compress_session(&session_id).await {
            tracing::error!("[AgentSession] compress failed: {}", err);
            self.emit(AgentSessionEvent::Progress {
                stage: "session.compress",
                detail: Some("failed".to_string()),
            })
            .await;
        } else {
            self.emit(AgentSessionEvent::Progress {
                stage: "session.compress",
                detail: Some("done".to_string()),
            })
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

        let mut context = restore_context(
            &self.core.session_storage,
            &session_id,
            self.restore_max_messages,
        );
        context.set_actor_identity(&self.actor);

        // 因为已在 fast persist 阶段把用户消息写入 DB，restore_context 会把它加载进来。
        // 但 runtime_input 也会再次传给 runner（避免重复），所以这里把末尾的用户消息弹出。
        if let Some(last) = context.messages.last() {
            if last.role == "user" && last.content.as_deref() == Some(user_input) {
                context.messages.pop();
            }
        }

        if !self.core.runner_supports_strict_actor_sandbox() {
            let message = self
                .core
                .strict_actor_sandbox_guard_message()
                .unwrap_or("当前 runner 不支持严格 actor sandbox。");
            tracing::error!("[AgentSession] strict actor sandbox guard: {}", message);
            if let Some(reservation) = quota_reservation.as_ref() {
                let _ = self
                    .core
                    .conversation_quota_storage
                    .release_daily_conversation(reservation);
            }
            return self
                .fail_run(
                    session_id,
                    AgentSessionErrorKind::AgentFailed,
                    message.to_string(),
                )
                .await;
        }

        let (system_prompt, runtime_input) = self.resolve_prompt_input(&session_id, user_input);
        self.audit_effective_prompt(&session_id, &system_prompt, &runtime_input);

        let tool_registry = self.core.create_tool_registry(
            Some(&self.actor),
            &self.channel_target,
            self.allow_cron,
        );
        let runner = match self.core.create_runner_with_model_override(
            &system_prompt,
            tool_registry,
            options.model_override.as_deref(),
        ) {
            Ok(r) => r,
            Err(err) => {
                tracing::error!("[AgentSession] create_runner 失败: {}", err);
                if let Some(reservation) = quota_reservation.as_ref() {
                    let _ = self
                        .core
                        .conversation_quota_storage
                        .release_daily_conversation(reservation);
                }
                return self
                    .fail_run(session_id, AgentSessionErrorKind::AgentFailed, err)
                    .await;
            }
        };
        let runner_name = runner.name();
        let working_directory = match ensure_actor_sandbox(&self.actor) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(err) => {
                tracing::error!(
                    "[AgentSession] actor sandbox 初始化失败 actor={} err={}",
                    self.actor.session_id(),
                    err
                );
                if let Some(reservation) = quota_reservation.as_ref() {
                    let _ = self
                        .core
                        .conversation_quota_storage
                        .release_daily_conversation(reservation);
                }
                return self
                    .fail_run(
                        session_id,
                        AgentSessionErrorKind::Io,
                        format!("actor sandbox 初始化失败: {err}"),
                    )
                    .await;
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
        self.emit(AgentSessionEvent::Progress {
            stage: "agent.run",
            detail: Some(runner_name.to_string()),
        })
        .await;
        let started = Instant::now();

        let runner_request = AgentRunnerRequest {
            session_id: session_id.clone(),
            actor_label: self.actor.user_id.clone(),
            actor: self.actor.clone(),
            channel_target: self.channel_target.clone(),
            allow_cron: self.allow_cron,
            config_path: crate::core::runtime_config_path(),
            system_prompt,
            runtime_input,
            context,
            timeout: options.timeout,
            gemini_stream: self.default_gemini_stream_options(options.timeout),
            session_metadata: self.load_session_metadata(&session_id),
            working_directory,
        };

        let runner_result = self
            .run_runner_with_empty_success_retry(
                runner.as_ref(),
                runner_name,
                &session_id,
                runner_request,
            )
            .await;
        let streamed_output = runner_result.streamed_output;
        let terminal_error_emitted = runner_result.terminal_error_emitted;
        if !runner_result.session_metadata_updates.is_empty() {
            let _ = self
                .core
                .session_storage
                .update_metadata(&session_id, runner_result.session_metadata_updates.clone());
        }
        let mut response = runner_result.response;
        if response.success && response_leaks_system_prompt(&response.content) {
            tracing::error!(
                "[AgentSession] blocked echoed system prompt runner={} session_id={}",
                runner_name,
                session_id
            );
            response.success = false;
            response.error = Some("agent returned leaked system instructions".to_string());
            response.content.clear();
        }
        let elapsed_ms = started.elapsed().as_millis();

        if response.success {
            if let Some(reservation) = quota_reservation.as_ref() {
                let _ = self
                    .core
                    .conversation_quota_storage
                    .commit_daily_conversation(reservation);
            }
            if !streamed_output {
                if let Some(segmenter) = options.segmenter.as_ref() {
                    let segments = segmenter(&response.content);
                    for seg in segments {
                        self.emit(AgentSessionEvent::Segment { text: seg }).await;
                    }
                }
            }
            for tool_call in &response.tool_calls_made {
                let result_str =
                    serde_json::to_string(&tool_call.result).unwrap_or_else(|_| "{}".to_string());
                let _ = self.core.session_storage.add_message(
                    &session_id,
                    "tool",
                    &result_str,
                    Some(build_tool_message_metadata(tool_call)),
                );
            }
            let _ = self.core.session_storage.add_message(
                &session_id,
                "assistant",
                &response.content,
                self.message_metadata.assistant.clone(),
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
            if let Some(reservation) = quota_reservation.as_ref() {
                let _ = self
                    .core
                    .conversation_quota_storage
                    .release_daily_conversation(reservation);
            }
            let err = response
                .error
                .clone()
                .unwrap_or_else(|| "未知错误".to_string());
            let kind = if err.contains("agent_timeout") {
                AgentSessionErrorKind::AgentTimeout
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
                self.emit(AgentSessionEvent::Error {
                    error: AgentSessionError { kind, message: err },
                })
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

fn sanitize_prompt_audit_path(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

fn response_leaks_system_prompt(content: &str) -> bool {
    let trimmed = content.trim_start_matches(char::is_whitespace);
    trimmed.starts_with("### System Instructions ###")
}

/// Restore recent messages into AgentContext.
pub fn restore_context(
    storage: &SessionStorage,
    session_id: &str,
    max_messages: Option<usize>,
) -> AgentContext {
    let mut ctx = AgentContext::new(session_id.to_string());

    let Ok(Some(session)) = storage.load_session(session_id) else {
        return ctx;
    };

    if let Some(actor) = &session.actor {
        ctx.set_actor_identity(actor);
    }

    let messages = select_context_messages(&session.messages, max_messages);

    for message in messages {
        match message.role.as_str() {
            "user" => ctx.add_user_message(&message.content),
            "assistant" => ctx.add_assistant_message(&message.content, None),
            "tool" => {
                if let Some((tool_call_id, tool_name, result)) =
                    restore_tool_message(&message.content, message.metadata.as_ref())
                {
                    ctx.add_tool_result(&tool_call_id, &tool_name, &result);
                }
            }
            _ => {}
        }
    }

    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runners::{AgentRunnerEvent, stream_gemini_prompt};
    use futures::stream::{self, BoxStream};
    use hone_core::ActorIdentity;
    use hone_core::config::HoneConfig;
    use hone_llm::{ChatResponse, LlmProvider, Message};
    use serde_json::Value;
    use std::env;
    use std::sync::{Arc, Mutex, OnceLock};

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()))
    }

    struct NoopEmitter;

    #[async_trait]
    impl AgentRunnerEmitter for NoopEmitter {
        async fn emit(&self, _event: AgentRunnerEvent) {}
    }

    #[derive(Clone)]
    struct MockLlmProvider {
        state: Arc<Mutex<MockLlmState>>,
    }

    struct MockLlmState {
        chat_with_tools_calls: usize,
        responses: std::collections::VecDeque<ChatResponse>,
    }

    impl MockLlmProvider {
        fn with_tool_responses(responses: Vec<ChatResponse>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_with_tools_calls: 0,
                    responses: responses.into(),
                })),
            }
        }

        fn chat_with_tools_calls(&self) -> usize {
            self.state
                .lock()
                .expect("mock llm lock")
                .chat_with_tools_calls
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            Err(hone_core::HoneError::Llm(
                "chat should not be called in these tests".to_string(),
            ))
        }

        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            let mut state = self.state.lock().expect("mock llm lock");
            state.chat_with_tools_calls += 1;
            state
                .responses
                .pop_front()
                .ok_or_else(|| hone_core::HoneError::Llm("no more mock tool responses".to_string()))
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    fn make_test_core(root: &std::path::Path, llm: MockLlmProvider) -> Arc<HoneBotCore> {
        let mut config = HoneConfig::default();
        config.agent.runner = "function_calling".to_string();
        config.agent.max_iterations = 3;
        config.storage.sessions_dir = root.join("sessions").to_string_lossy().to_string();
        config.storage.conversation_quota_dir = root
            .join("conversation_quota")
            .to_string_lossy()
            .to_string();
        config.storage.llm_audit_enabled = false;
        config.storage.llm_audit_db_path =
            root.join("llm_audit.sqlite3").to_string_lossy().to_string();
        config.storage.portfolio_dir = root.join("portfolio").to_string_lossy().to_string();
        config.storage.cron_jobs_dir = root.join("cron_jobs").to_string_lossy().to_string();
        config.storage.reports_dir = root.join("reports").to_string_lossy().to_string();
        config.storage.x_drafts_dir = root.join("x_drafts").to_string_lossy().to_string();
        config.storage.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();
        config.storage.kb_dir = root.join("kb").to_string_lossy().to_string();

        let mut core = HoneBotCore::new(config);
        core.llm = Some(Arc::new(llm));
        Arc::new(core)
    }

    #[cfg(unix)]
    fn write_mock_gemini_script(lines: &[&str]) -> (std::path::PathBuf, std::path::PathBuf) {
        use std::os::unix::fs::PermissionsExt;

        let root = make_temp_dir("hone_gemini_mock");
        let data_path = root.join("stream.txt");
        let content = lines.join("\n");
        std::fs::create_dir_all(&root).expect("create mock root");
        std::fs::write(&data_path, content).expect("write mock data");

        let script_path = root.join("gemini-mock.sh");
        let script = format!("#!/bin/sh\ncat \"{}\"\n", data_path.display());
        std::fs::write(&script_path, script).expect("write mock script");
        let mut perms = std::fs::metadata(&script_path)
            .expect("stat mock script")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("chmod mock script");

        (root, script_path)
    }

    #[test]
    fn restore_context_missing_session_returns_empty() {
        let root = make_temp_dir("hone_channels_restore_missing");
        let storage = SessionStorage::new(&root);
        let ctx = restore_context(&storage, "missing", Some(5));
        assert!(ctx.messages.is_empty());
        assert!(ctx.actor_identity().is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restore_context_filters_and_limits_messages() {
        let root = make_temp_dir("hone_channels_restore_filter");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let session_id = storage
            .create_session(
                Some("restore_test"),
                Some(actor.clone()),
                Some(SessionIdentity::from_actor(&actor).expect("session identity")),
            )
            .expect("create");

        storage
            .add_message(&session_id, "user", "u1", None)
            .expect("add u1");
        storage
            .add_message(&session_id, "assistant", "a1", None)
            .expect("add a1");
        storage
            .add_message(
                &session_id,
                "tool",
                "t1",
                Some(HashMap::from([
                    (
                        "tool_name".to_string(),
                        Value::String("web_search".to_string()),
                    ),
                    (
                        "tool_call_id".to_string(),
                        Value::String("call_1".to_string()),
                    ),
                ])),
            )
            .expect("add t1");
        storage
            .add_message(&session_id, "user", "u2", None)
            .expect("add u2");
        storage
            .add_message(&session_id, "assistant", "a2", None)
            .expect("add a2");

        let ctx = restore_context(&storage, &session_id, Some(4));
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|m| m.content.as_deref())
            .collect();
        assert_eq!(contents, vec!["a1", "t1", "u2", "a2"]);
        assert_eq!(ctx.messages[1].role, "tool");
        assert_eq!(ctx.messages[1].name.as_deref(), Some("web_search"));
        assert_eq!(ctx.messages[1].tool_call_id.as_deref(), Some("call_1"));
        assert_eq!(ctx.actor_identity(), Some(actor));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn response_leaks_system_prompt_detects_prefixed_echo() {
        assert!(response_leaks_system_prompt(
            "\n### System Instructions ###\nsecret"
        ));
        assert!(!response_leaks_system_prompt("正常回复"));
    }

    #[tokio::test]
    async fn run_success_commits_daily_conversation_quota() {
        let root = make_temp_dir("hone_channels_quota_success");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_tool_responses(vec![ChatResponse {
            content: "ok".to_string(),
            tool_calls: None,
            usage: None,
        }]);
        let core = make_test_core(&root, llm);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());

        let result = session.run("hello", AgentRunOptions::default()).await;
        assert!(result.response.success, "{:?}", result.response.error);

        let today = hone_core::beijing_now().format("%F").to_string();
        let snapshot = core
            .conversation_quota_storage
            .snapshot_for_date(&actor, &today)
            .expect("snapshot")
            .expect("row");
        assert_eq!(snapshot.success_count, 1);
        assert_eq!(snapshot.in_flight, 0);

        let messages = core
            .session_storage
            .get_messages(&actor.session_id(), None)
            .expect("messages");
        assert_eq!(messages.len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn run_rejects_over_daily_limit_without_persisting_user_message() {
        let root = make_temp_dir("hone_channels_quota_reject");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_tool_responses(vec![ChatResponse {
            content: "unused".to_string(),
            tool_calls: None,
            usage: None,
        }]);
        let core = make_test_core(&root, llm.clone());
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let today = hone_core::beijing_now().format("%F").to_string();

        for _ in 0..DAILY_CONVERSATION_LIMIT {
            let reservation = match core
                .conversation_quota_storage
                .try_reserve_daily_conversation(&actor, DAILY_CONVERSATION_LIMIT, false)
                .expect("reserve")
            {
                ConversationQuotaReserveResult::Reserved(reservation) => reservation,
                other => panic!("unexpected reserve result: {other:?}"),
            };
            core.conversation_quota_storage
                .commit_daily_conversation(&reservation)
                .expect("commit");
        }

        let session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());
        let result = session.run("hello", AgentRunOptions::default()).await;

        assert!(!result.response.success);
        assert!(
            result
                .response
                .error
                .unwrap_or_default()
                .contains("已达到今日对话上限")
        );
        assert_eq!(llm.chat_with_tools_calls(), 0);
        assert!(
            core.session_storage
                .get_messages(&actor.session_id(), None)
                .expect("messages")
                .is_empty()
        );
        let snapshot = core
            .conversation_quota_storage
            .snapshot_for_date(&actor, &today)
            .expect("snapshot")
            .expect("row");
        assert_eq!(snapshot.success_count, DAILY_CONVERSATION_LIMIT);
        assert_eq!(snapshot.in_flight, 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn scheduled_task_mode_skips_daily_quota() {
        let root = make_temp_dir("hone_channels_quota_scheduled");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_tool_responses(vec![ChatResponse {
            content: "scheduled ok".to_string(),
            tool_calls: None,
            usage: None,
        }]);
        let core = make_test_core(&root, llm);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let today = hone_core::beijing_now().format("%F").to_string();

        for _ in 0..DAILY_CONVERSATION_LIMIT {
            let reservation = match core
                .conversation_quota_storage
                .try_reserve_daily_conversation(&actor, DAILY_CONVERSATION_LIMIT, false)
                .expect("reserve")
            {
                ConversationQuotaReserveResult::Reserved(reservation) => reservation,
                other => panic!("unexpected reserve result: {other:?}"),
            };
            core.conversation_quota_storage
                .commit_daily_conversation(&reservation)
                .expect("commit");
        }

        let session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());
        let result = session
            .run(
                "run scheduled task",
                AgentRunOptions {
                    quota_mode: AgentRunQuotaMode::ScheduledTask,
                    ..AgentRunOptions::default()
                },
            )
            .await;

        assert!(result.response.success, "{:?}", result.response.error);
        let snapshot = core
            .conversation_quota_storage
            .snapshot_for_date(&actor, &today)
            .expect("snapshot")
            .expect("row");
        assert_eq!(snapshot.success_count, DAILY_CONVERSATION_LIMIT);
        assert_eq!(snapshot.in_flight, 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stream_gemini_prompt_collects_content() {
        let (root, script_path) = write_mock_gemini_script(&[
            r#"{"type":"content","value":"第一段。\n\n第二段开始。"}"#,
            r#"{"type":"thought","value":"thinking..."}"#,
            r#"{"type":"finished","value":{}}"#,
        ]);
        with_temp_env_var("HONE_GEMINI_BIN", script_path.as_os_str(), || async {
            let mut full = String::new();
            let mut raw_lines = 0u32;
            let options = GeminiStreamOptions {
                max_iterations: 1,
                overall_timeout: Duration::from_secs(3),
                per_line_timeout: Duration::from_secs(3),
            };

            let buf = stream_gemini_prompt(
                "hi",
                "tester",
                &root.to_string_lossy(),
                1,
                &options,
                &mut full,
                &mut raw_lines,
                Arc::new(NoopEmitter),
            )
            .await
            .expect("stream ok");
            assert!(buf.contains("第一段"));
            assert!(full.contains("第一段"));
            assert!(full.contains("\n\n第二段开始。"));
        })
        .await;
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stream_gemini_prompt_handles_error_event() {
        let (root, script_path) = write_mock_gemini_script(&[
            r#"{"type":"error","value":"boom"}"#,
            r#"{"type":"finished","value":{}}"#,
        ]);
        with_temp_env_var("HONE_GEMINI_BIN", script_path.as_os_str(), || async {
            let mut full = String::new();
            let mut raw_lines = 0u32;
            let options = GeminiStreamOptions {
                max_iterations: 1,
                overall_timeout: Duration::from_secs(3),
                per_line_timeout: Duration::from_secs(3),
            };

            let err = stream_gemini_prompt(
                "hi",
                "tester",
                &root.to_string_lossy(),
                1,
                &options,
                &mut full,
                &mut raw_lines,
                Arc::new(NoopEmitter),
            )
            .await
            .expect_err("should fail");
            assert!(matches!(err.kind, AgentSessionErrorKind::GeminiError));
        })
        .await;
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stream_gemini_prompt_handles_context_overflow() {
        let (root, script_path) = write_mock_gemini_script(&[
            r#"{"type":"context_window_will_overflow","value":{"estimatedRequestTokenCount":123,"remainingTokenCount":4}}"#,
            r#"{"type":"finished","value":{}}"#,
        ]);
        with_temp_env_var("HONE_GEMINI_BIN", script_path.as_os_str(), || async {
            let mut full = String::new();
            let mut raw_lines = 0u32;
            let options = GeminiStreamOptions {
                max_iterations: 1,
                overall_timeout: Duration::from_secs(3),
                per_line_timeout: Duration::from_secs(3),
            };

            let err = stream_gemini_prompt(
                "hi",
                "tester",
                &root.to_string_lossy(),
                1,
                &options,
                &mut full,
                &mut raw_lines,
                Arc::new(NoopEmitter),
            )
            .await
            .expect_err("should fail");
            assert!(matches!(
                err.kind,
                AgentSessionErrorKind::ContextWindowOverflow
            ));
        })
        .await;
        let _ = std::fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    async fn with_temp_env_var<F, Fut>(key: &str, value: &std::ffi::OsStr, f: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let _guard = env_lock().lock().await;
        unsafe {
            let old = env::var_os(key);
            env::set_var(key, value);
            f().await;
            if let Some(prev) = old {
                env::set_var(key, prev);
            } else {
                env::remove_var(key);
            }
        }
    }

    #[cfg(unix)]
    fn env_lock() -> &'static tokio::sync::Mutex<()> {
        static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }
}
