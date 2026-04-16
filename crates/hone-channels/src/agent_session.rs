//! Agent session abstraction shared across channels.

use async_trait::async_trait;
use hone_core::agent::{
    AgentContext, AgentMessage, AgentResponse, NormalizedConversationMessage,
    NormalizedConversationPart, ToolCallMade,
};
use hone_core::{ActorIdentity, HoneConfig, SessionIdentity};
use hone_memory::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, SessionStorage,
    assistant_tool_calls_from_metadata, build_assistant_message_metadata,
    has_compact_skill_snapshot, invoked_skills_from_metadata, message_is_compact_boundary,
    message_is_compact_summary, message_is_slash_skill, restore_tool_message,
    select_messages_after_compact_boundary, session_message_from_normalized, session_message_text,
    session_message_to_agent_messages,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::HoneBotCore;
use crate::execution::{
    ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService, PreparedExecution,
};
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::prompt_audit::PromptAuditMetadata;
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult};
use crate::runtime::{relativize_user_visible_paths, sanitize_user_visible_output};
use crate::session_compactor::SessionCompactor;

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

const DAILY_CONVERSATION_LIMIT: u32 = 12;
const EMPTY_SUCCESS_RETRY_LIMIT: usize = 2;
const CONTEXT_OVERFLOW_RECOVERY_LIMIT: usize = 1;
const DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT: usize = 20;
const EMPTY_SUCCESS_FALLBACK_MESSAGE: &str =
    "这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。";
const CONTEXT_OVERFLOW_FALLBACK_MESSAGE: &str = "当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。";

fn restore_limit_before_compaction(
    config: &HoneConfig,
    session_identity: &SessionIdentity,
) -> Option<usize> {
    if session_identity.is_group() {
        Some(
            config
                .group_context
                .recent_context_limit
                .max(config.group_context.compress_threshold_messages)
                .max(1),
        )
    } else {
        Some(DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT)
    }
}

fn should_return_runner_result(result: &AgentRunnerResult) -> bool {
    // 失败直接返回；成功时必须拿到正文，不能因为只有工具调用就把空答复当成成功。
    //
    // 注意：`streamed_output` 仅表示 runner 具备流式能力，不代表这次真的输出过内容。
    // opencode_acp 会始终把它设为 true，因此不能再把它当成“已有输出”的依据，
    // 否则空回复成功态会被直接放过，前端就可能一直停留在“思考中”。
    !result.response.success || !result.response.content.trim().is_empty()
}

fn is_context_overflow_error_text(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    normalized.contains("context window exceeds limit")
        || normalized.contains("context window overflow")
        || normalized.contains("context_window_will_overflow")
        || normalized.contains("context length exceeded")
        || normalized.contains("maximum context length")
        || normalized.contains("prompt is too long")
        || normalized.contains("too many tokens")
}

fn should_persist_tool_result(call: &ToolCallMade) -> bool {
    if matches_skill_runtime_tool_name(&call.name) {
        return false;
    }
    if call.name == "web_search" {
        if call
            .result
            .get("status")
            .and_then(|value| value.as_str())
            .is_some_and(|status| status == "unavailable")
        {
            return false;
        }
        if call.result.get("error").is_some() {
            return false;
        }
    }
    true
}

fn matches_skill_runtime_tool_name(name: &str) -> bool {
    matches!(
        name.trim(),
        "skill_tool"
            | "load_skill"
            | "discover_skills"
            | "hone/skill_tool"
            | "hone/load_skill"
            | "hone/discover_skills"
            | "Tool: hone/skill_tool"
            | "Tool: hone/load_skill"
            | "Tool: hone/discover_skills"
    )
}

#[derive(Debug, Clone)]
struct SlashSkillExpansion {
    raw_input: String,
    invoked_prompt: String,
    runtime_input: String,
    skill_id: String,
}

#[derive(Debug, Clone)]
struct CompactCommand {
    instructions: Option<String>,
}

fn merge_message_metadata(
    base: Option<HashMap<String, Value>>,
    extra: HashMap<String, Value>,
) -> Option<HashMap<String, Value>> {
    let mut merged = base.unwrap_or_default();
    for (key, value) in extra {
        merged.insert(key, value);
    }
    Some(merged)
}

fn persistable_turn_from_response(
    response: &AgentResponse,
    metadata: Option<HashMap<String, Value>>,
) -> Option<NormalizedConversationMessage> {
    let persisted_tool_calls = response
        .tool_calls_made
        .iter()
        .filter(|call| should_persist_tool_result(call))
        .map(|call| {
            serde_json::json!({
                "id": call.tool_call_id.clone().unwrap_or_default(),
                "type": "function",
                "function": {
                    "name": call.name,
                    "arguments": serde_json::to_string(&call.arguments)
                        .unwrap_or_else(|_| "null".to_string()),
                }
            })
        })
        .collect::<Vec<_>>();
    let tool_call_metadata = build_assistant_message_metadata(&persisted_tool_calls);
    let metadata = if tool_call_metadata.is_empty() {
        metadata
    } else {
        merge_message_metadata(metadata, tool_call_metadata)
    };

    let mut content = Vec::new();

    if !response.content.trim().is_empty() {
        content.push(NormalizedConversationPart {
            part_type: "final".to_string(),
            text: Some(response.content.trim().to_string()),
            id: None,
            name: None,
            args: None,
            result: None,
            metadata: None,
        });
    }

    if content.is_empty() {
        None
    } else {
        Some(NormalizedConversationMessage {
            role: "assistant".to_string(),
            content,
            status: Some("completed".to_string()),
            metadata,
        })
    }
}

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

struct SessionEventEmitter {
    listeners: Vec<Arc<dyn AgentSessionListener>>,
    channel: String,
    user_id: String,
    session_id: String,
    message_id: Option<String>,
    working_directory: String,
}

fn truncate_event_detail(detail: &str, max_chars: usize) -> String {
    if detail.chars().count() <= max_chars {
        return detail.to_string();
    }
    detail.chars().take(max_chars).collect::<String>() + "..."
}

#[async_trait]
impl AgentRunnerEmitter for SessionEventEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        let event = match event {
            AgentRunnerEvent::Progress { stage, detail } => AgentRunnerEvent::Progress {
                stage,
                detail: detail
                    .map(|value| relativize_user_visible_paths(&value, &self.working_directory)),
            },
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message: message
                    .map(|value| relativize_user_visible_paths(&value, &self.working_directory)),
                reasoning: reasoning
                    .map(|value| relativize_user_visible_paths(&value, &self.working_directory)),
            },
            AgentRunnerEvent::Error { mut error } => {
                error.message =
                    relativize_user_visible_paths(&error.message, &self.working_directory);
                AgentRunnerEvent::Error { error }
            }
            other => other,
        };

        match &event {
            AgentRunnerEvent::Progress { stage, detail } => {
                tracing::info!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_progress",
                    "[MsgFlow/{}] runner.stage={} user={} session={} detail={}",
                    self.channel,
                    stage,
                    self.user_id,
                    self.session_id,
                    detail
                        .as_deref()
                        .map(|value| truncate_event_detail(value, 280))
                        .unwrap_or_else(|| "-".to_string())
                );
            }
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => {
                tracing::info!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_tool",
                    "[MsgFlow/{}] runner.tool user={} session={} tool={} status={} message={} reasoning={}",
                    self.channel,
                    self.user_id,
                    self.session_id,
                    tool,
                    status,
                    message
                        .as_deref()
                        .map(|value| truncate_event_detail(value, 200))
                        .unwrap_or_else(|| "-".to_string()),
                    reasoning
                        .as_deref()
                        .map(|value| truncate_event_detail(value, 200))
                        .unwrap_or_else(|| "-".to_string())
                );
            }
            AgentRunnerEvent::Error { error } => {
                tracing::warn!(
                    message_id = %self.message_id.as_deref().unwrap_or("-"),
                    state = "runner_error",
                    "[MsgFlow/{}] runner.error user={} session={} kind={:?} message=\"{}\"",
                    self.channel,
                    self.user_id,
                    self.session_id,
                    error.kind,
                    truncate_event_detail(&error.message, 280)
                );
            }
            AgentRunnerEvent::StreamDelta { .. } | AgentRunnerEvent::StreamThought { .. } => {}
        }

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
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let mut last_result = runner.run(request.clone(), emitter.clone()).await;

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
            self.emit(AgentSessionEvent::Progress {
                stage: "agent.run.retry",
                detail: Some(format!(
                    "{runner_name} empty_success attempt={attempt}/{EMPTY_SUCCESS_RETRY_LIMIT}"
                )),
            })
            .await;

            last_result = runner.run(request.clone(), emitter.clone()).await;
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
    ) -> AgentContext {
        let mut context = restore_context(
            &self.core.session_storage,
            session_id,
            self.restore_max_messages,
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
    ) -> Result<PreparedExecution, (AgentSessionErrorKind, String)> {
        let context = self.restore_runtime_context(session_id, persisted_user_input);
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

    fn persist_successful_assistant_turn(
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

    fn resolve_prompt_input(&self, session_id: &str, user_input: &str) -> (String, String) {
        let mut prompt_options = self.prompt_options.clone();
        if self.allow_cron {
            prompt_options
                .extra_sections
                .push(crate::prompt::DEFAULT_CRON_TASK_POLICY.to_string());
        }
        let skill_runtime = hone_tools::SkillRuntime::new(
            self.core.configured_system_skills_dir(),
            self.core.configured_custom_skills_dir(),
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        )
        .with_registry_path(self.core.configured_skill_registry_path());
        let skill_listing = skill_runtime.build_skill_listing(4_000);
        if !skill_listing.trim().is_empty() {
            prompt_options.extra_sections.push(format!(
                "【SkillTool】\n\
                - 当用户任务明显匹配某个 skill 时，必须先调用 skill_tool，再继续回答。\n\
                - 若当前 runner 通过 MCP 暴露 namespaced 工具名，则 `skill_tool` 对应 `hone/skill_tool`，`discover_skills` 对应 `hone/discover_skills`；必须调用真实暴露出的那个工具名，不要因为带前缀就误判“工具不存在”。\n\
                - 用户可以直接输入 `/<skill-id>` 触发 user-invocable 技能；模型不要假装已经加载 skill，必须真的调用工具。\n\
                - 如果当前任务发生中途转向，或现有技能不够覆盖，再调用 discover_skills / hone/discover_skills 检索相关技能。\n\
                - turn-0 可用技能索引：\n{}",
                skill_listing
            ));
        }
        let related_skills =
            skill_runtime.search(user_input, &extract_possible_file_paths(user_input), 5);
        let bundle = build_prompt_bundle(
            &self.core.config,
            &self.core.session_storage,
            &self.actor.channel,
            session_id,
            &Default::default(),
            &prompt_options,
        );
        let runtime_user_input = if related_skills.is_empty() {
            user_input.to_string()
        } else {
            let listing = related_skills
                .into_iter()
                .map(|skill| {
                    let mut line = format!("- {}: {}", skill.id, skill.description);
                    if let Some(when_to_use) = skill
                        .when_to_use
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                    {
                        line.push_str(" - ");
                        line.push_str(when_to_use.trim());
                    }
                    line
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "【本轮相关技能提示】\n{}\n如这些技能已覆盖下一步，就直接用 skill_tool（或 MCP 下的 hone/skill_tool）；否则再调用 discover_skills（或 hone/discover_skills）。\n\n{}",
                listing, user_input
            )
        };
        (
            bundle.system_prompt(),
            bundle.compose_user_input(&runtime_user_input),
        )
    }

    fn expand_slash_skill_input(
        &self,
        session_id: &str,
        user_input: &str,
    ) -> hone_core::HoneResult<Option<SlashSkillExpansion>> {
        let trimmed = user_input.trim();
        if !trimmed.starts_with('/') {
            return Ok(None);
        }

        let runtime = hone_tools::SkillRuntime::new(
            self.core.configured_system_skills_dir(),
            self.core.configured_custom_skills_dir(),
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        )
        .with_registry_path(self.core.configured_skill_registry_path());

        if trimmed.strip_prefix("/skill").is_some() {
            let lines = trimmed.lines().collect::<Vec<_>>();
            let first_line = lines.first().copied().unwrap_or_default();
            let query = first_line.trim_start_matches("/skill").trim();
            if query.is_empty() {
                return Ok(None);
            }
            if let Some(skill) =
                runtime.resolve_skill_via_search(query, &extract_possible_file_paths(user_input))
            {
                let invoked_prompt = runtime.render_invocation_prompt(&skill, session_id, None);
                let tail = lines.iter().skip(1).copied().collect::<Vec<_>>().join("\n");
                let runtime_input =
                    compose_invoked_skill_runtime_input(&invoked_prompt, Some(tail.trim()));
                return Ok(Some(SlashSkillExpansion {
                    raw_input: user_input.to_string(),
                    invoked_prompt,
                    runtime_input,
                    skill_id: skill.id,
                }));
            }
            return Ok(None);
        }

        let command = trimmed.trim_start_matches('/');
        let mut parts = command.splitn(2, char::is_whitespace);
        let skill_id = parts.next().unwrap_or_default();
        let args = parts.next().map(str::trim);
        if let Some(skill) = runtime.resolve_user_invocable_direct(skill_id) {
            let invoked_prompt = runtime.render_invocation_prompt(&skill, session_id, args);
            return Ok(Some(SlashSkillExpansion {
                raw_input: user_input.to_string(),
                invoked_prompt: invoked_prompt.clone(),
                runtime_input: compose_invoked_skill_runtime_input(&invoked_prompt, args),
                skill_id: skill.id,
            }));
        }

        Ok(None)
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
        let mut invoked = invoked_skills_from_metadata(&existing)
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

        self.emit(AgentSessionEvent::Progress {
            stage: "session.compress",
            detail: Some("start".to_string()),
        })
        .await;
        let started = Instant::now();
        let outcome = self
            .core
            .compact_session(&session_id, "manual", true, command.instructions.as_deref())
            .await;

        let response = match outcome {
            Ok(outcome) => {
                self.emit(AgentSessionEvent::Progress {
                    stage: "session.compress",
                    detail: Some("done".to_string()),
                })
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
                self.emit(AgentSessionEvent::Progress {
                    stage: "session.compress",
                    detail: Some("failed".to_string()),
                })
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
            self.emit(AgentSessionEvent::Error {
                error: AgentSessionError {
                    kind: AgentSessionErrorKind::AgentFailed,
                    message: err,
                },
            })
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

        let slash_skill = match self.expand_slash_skill_input(&session_id, user_input) {
            Ok(value) => value,
            Err(err) => {
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

        let mut execution = match self.prepare_execution_for_turn(
            &session_id,
            persisted_user_input,
            runtime_user_input,
            &options,
        ) {
            Ok(execution) => execution,
            Err((kind, err)) => {
                if let Some(reservation) = quota_reservation.as_ref() {
                    let _ = self
                        .core
                        .conversation_quota_storage
                        .release_daily_conversation(reservation);
                }
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
        self.emit(AgentSessionEvent::Progress {
            stage: "agent.run",
            detail: Some(execution.runner_name.to_string()),
        })
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
            self.emit(AgentSessionEvent::Progress {
                stage: "agent.run.retry",
                detail: Some(format!(
                    "{} context_overflow attempt={}/{}",
                    execution.runner_name,
                    recovery_idx + 1,
                    CONTEXT_OVERFLOW_RECOVERY_LIMIT
                )),
            })
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
        if response.success && response_leaks_system_prompt(&response.content) {
            tracing::error!(
                "[AgentSession] blocked echoed system prompt runner={} session_id={}",
                execution.runner_name,
                session_id
            );
            response.success = false;
            response.error = Some("agent returned leaked system instructions".to_string());
            response.content.clear();
        }
        if response.success {
            let sanitized = sanitize_user_visible_output(&response.content);
            if sanitized.only_internal {
                tracing::error!(
                    "[AgentSession] blocked internal-only assistant output runner={} session_id={}",
                    execution.runner_name,
                    session_id
                );
                response.success = false;
                response.error = Some("agent returned internal-only output".to_string());
                response.content.clear();
            } else {
                response.content = sanitized.content;
            }
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

fn response_leaks_system_prompt(content: &str) -> bool {
    let trimmed = content.trim_start_matches(char::is_whitespace);
    trimmed.starts_with("### System Instructions ###")
}

/// Restore recent messages into AgentContext.
pub fn restore_context(
    storage: &SessionStorage,
    session_id: &str,
    max_messages: Option<usize>,
    skill_runtime: Option<&hone_tools::SkillRuntime>,
) -> AgentContext {
    let mut ctx = AgentContext::new(session_id.to_string());

    let Ok(Some(session)) = storage.load_session(session_id) else {
        return ctx;
    };

    if let Some(actor) = &session.actor {
        ctx.set_actor_identity(actor);
    }

    let messages = select_messages_after_compact_boundary(&session.messages, max_messages);
    let has_skill_snapshots = has_compact_skill_snapshot(&messages);
    if !has_skill_snapshots {
        for skill in invoked_skills_from_metadata(&session.metadata)
            .into_iter()
            .filter(|skill| !skill.prompt.trim().is_empty())
            .filter(|skill| {
                skill_runtime
                    .map(|runtime| {
                        runtime
                            .load_registered_skill(&skill.skill_name)
                            .map(|definition| definition.enabled)
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
            })
        {
            ctx.add_user_message(&skill.prompt);
        }
    }

    for message in messages {
        match message.role.as_str() {
            "user" => {
                if !message_is_slash_skill(message.metadata.as_ref()) {
                    let content = if message_is_compact_summary(message.metadata.as_ref()) {
                        sanitize_user_visible_output(&session_message_text(message)).content
                    } else {
                        session_message_text(message)
                    };
                    if !content.trim().is_empty() {
                        ctx.messages.push(AgentMessage {
                            role: "user".to_string(),
                            content: Some(content),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                            metadata: message.metadata.clone(),
                        });
                    }
                }
            }
            "assistant" | "tool" => {
                for mut restored in session_message_to_agent_messages(message) {
                    if restored.role == "assistant" {
                        let sanitized = sanitize_user_visible_output(
                            restored.content.as_deref().unwrap_or_default(),
                        );
                        let tool_calls = restored.tool_calls.clone().or_else(|| {
                            assistant_tool_calls_from_metadata(message.metadata.as_ref())
                        });
                        if sanitized.content.trim().is_empty()
                            && tool_calls.as_ref().is_none_or(|items| items.is_empty())
                        {
                            continue;
                        }
                        restored.content = Some(sanitized.content);
                        restored.tool_calls = tool_calls;
                    }
                    if restored.role == "tool"
                        && restore_tool_message(message).is_none()
                        && restored
                            .content
                            .as_deref()
                            .unwrap_or_default()
                            .trim()
                            .is_empty()
                    {
                        continue;
                    }
                    ctx.messages.push(restored);
                }
            }
            "system" => {
                if message_is_compact_boundary(message.metadata.as_ref()) {
                    continue;
                }
            }
            _ => {}
        }
    }

    ctx
}

fn extract_possible_file_paths(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .filter(|token| token.contains('/') || token.contains('\\'))
        .map(|token| token.trim_matches(|ch: char| matches!(ch, '"' | '\'' | ',' | ')' | '(')))
        .filter(|token| !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

fn compose_invoked_skill_runtime_input(
    invoked_prompt: &str,
    user_supplement: Option<&str>,
) -> String {
    if let Some(supplement) = user_supplement
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        format!("{invoked_prompt}\n\n【User Task After Invoking This Skill】\n{supplement}")
    } else {
        invoked_prompt.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runners::{AgentRunnerEvent, stream_gemini_prompt};
    use futures::stream::{self, BoxStream};
    use hone_core::ActorIdentity;
    use hone_core::agent::AgentContext;
    use hone_core::config::HoneConfig;
    use hone_llm::provider::ChatResult;
    use hone_llm::{ChatResponse, LlmProvider, Message};
    use hone_memory::session::{SessionRuntimeBackend, SessionStorageOptions};
    use hone_memory::{build_assistant_message_metadata, build_tool_message_metadata_parts};
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
    struct MockEmptySuccessRunner {
        response: AgentResponse,
    }

    #[async_trait]
    impl crate::runners::AgentRunner for MockEmptySuccessRunner {
        fn name(&self) -> &'static str {
            "mock_empty_success"
        }

        async fn run(
            &self,
            _request: crate::runners::AgentRunnerRequest,
            _emitter: Arc<dyn AgentRunnerEmitter>,
        ) -> crate::runners::AgentRunnerResult {
            crate::runners::AgentRunnerResult {
                response: self.response.clone(),
                streamed_output: true,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            }
        }
    }

    #[derive(Clone)]
    struct MockLlmProvider {
        state: Arc<Mutex<MockLlmState>>,
    }

    struct MockLlmState {
        chat_calls: usize,
        chat_with_tools_calls: usize,
        chat_responses: std::collections::VecDeque<hone_core::HoneResult<ChatResult>>,
        responses: std::collections::VecDeque<hone_core::HoneResult<ChatResponse>>,
        last_chat_messages: Option<Vec<Message>>,
    }

    impl MockLlmProvider {
        fn with_chat_and_tool_responses(
            chat_responses: Vec<hone_core::HoneResult<ChatResult>>,
            responses: Vec<hone_core::HoneResult<ChatResponse>>,
        ) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    chat_responses: chat_responses.into(),
                    responses: responses.into(),
                    last_chat_messages: None,
                })),
            }
        }

        fn with_chat_responses(responses: Vec<ChatResult>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    chat_responses: responses.into_iter().map(Ok).collect(),
                    responses: Default::default(),
                    last_chat_messages: None,
                })),
            }
        }

        fn with_tool_responses(responses: Vec<ChatResponse>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    chat_responses: Default::default(),
                    responses: responses.into_iter().map(Ok).collect(),
                    last_chat_messages: None,
                })),
            }
        }

        fn chat_calls(&self) -> usize {
            self.state.lock().expect("mock llm lock").chat_calls
        }

        fn chat_with_tools_calls(&self) -> usize {
            self.state
                .lock()
                .expect("mock llm lock")
                .chat_with_tools_calls
        }

        fn last_chat_prompt(&self) -> Option<String> {
            self.state
                .lock()
                .expect("mock llm lock")
                .last_chat_messages
                .as_ref()
                .and_then(|messages| messages.first())
                .and_then(|message| message.content.clone())
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<hone_llm::provider::ChatResult> {
            let mut state = self.state.lock().expect("mock llm lock");
            state.chat_calls += 1;
            state.last_chat_messages = Some(_messages.to_vec());
            state.chat_responses.pop_front().unwrap_or_else(|| {
                Err(hone_core::HoneError::Llm(
                    "no more mock chat responses".to_string(),
                ))
            })
        }

        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            let mut state = self.state.lock().expect("mock llm lock");
            state.chat_with_tools_calls += 1;
            state.responses.pop_front().unwrap_or_else(|| {
                Err(hone_core::HoneError::Llm(
                    "no more mock tool responses".to_string(),
                ))
            })
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
        make_test_core_with_config(root, llm, |_| {})
    }

    fn make_test_core_with_config(
        root: &std::path::Path,
        llm: MockLlmProvider,
        configure: impl FnOnce(&mut HoneConfig),
    ) -> Arc<HoneBotCore> {
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
        config.storage.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();
        configure(&mut config);

        let mut core = HoneBotCore::new(config);
        let shared_llm = Arc::new(llm);
        core.llm = Some(shared_llm.clone());
        core.auxiliary_llm = Some(shared_llm);
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
        let ctx = restore_context(&storage, "missing", Some(5), None);
        assert!(ctx.messages.is_empty());
        assert!(ctx.actor_identity().is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn should_return_runner_result_ignores_streaming_flag_when_response_is_empty() {
        let result = AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        };

        assert!(!should_return_runner_result(&result));

        let mut with_content = result;
        with_content.response.content = "hello".to_string();
        assert!(should_return_runner_result(&with_content));
    }

    #[test]
    fn should_return_runner_result_does_not_treat_tool_calls_only_as_success() {
        let result = AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: vec![ToolCallMade {
                    name: "data_fetch".to_string(),
                    arguments: serde_json::json!({"symbol": "MU"}),
                    result: serde_json::json!({"price": 101}),
                    tool_call_id: Some("call_1".to_string()),
                }],
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        };

        assert!(!should_return_runner_result(&result));
    }

    #[tokio::test]
    async fn empty_success_with_tool_calls_uses_fallback_after_retries() {
        let root = make_temp_dir("hone_channels_empty_success_tool_calls");
        std::fs::create_dir_all(&root).expect("create root");
        let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
        let actor = ActorIdentity::new("discord", "empty-success", None::<String>).expect("actor");
        let session = AgentSession::new(core, actor, "direct");
        let runner = MockEmptySuccessRunner {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: vec![ToolCallMade {
                    name: "web_search".to_string(),
                    arguments: serde_json::json!({"query": "AAOI"}),
                    result: serde_json::json!({"results": [{"title": "ok"}]}),
                    tool_call_id: Some("call_1".to_string()),
                }],
                iterations: 1,
                success: true,
                error: None,
            },
        };
        let request = crate::runners::AgentRunnerRequest {
            session_id: "empty-success-session".to_string(),
            actor_label: "discord:empty-success".to_string(),
            actor: session.actor.clone(),
            channel_target: "direct".to_string(),
            allow_cron: false,
            config_path: String::new(),
            system_prompt: "system".to_string(),
            runtime_input: "user input".to_string(),
            context: AgentContext::new("empty-success-session".to_string()),
            timeout: None,
            gemini_stream: GeminiStreamOptions::default(),
            session_metadata: HashMap::new(),
            working_directory: root.display().to_string(),
            allowed_tools: None,
            max_tool_calls: None,
        };

        let result = session
            .run_runner_with_empty_success_retry(
                &runner,
                "mock_empty_success",
                "empty-success-session",
                request,
                Arc::new(NoopEmitter),
            )
            .await;

        assert!(result.response.success);
        assert_eq!(result.response.content, EMPTY_SUCCESS_FALLBACK_MESSAGE);
        assert_eq!(result.response.tool_calls_made.len(), 1);

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

        let ctx = restore_context(&storage, &session_id, Some(4), None);
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
    fn restore_context_rehydrates_assistant_tool_calls() {
        let root = make_temp_dir("hone_channels_restore_tool_calls");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");

        storage
            .add_message(&session_id, "user", "AAOI 是什么公司", None)
            .expect("add user");
        storage
            .add_message(
                &session_id,
                "assistant",
                "我先查本地画像。",
                Some(build_assistant_message_metadata(&[serde_json::json!({
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": "local_search_files",
                        "arguments": "{\"query\":\"AAOI\"}"
                    }
                })])),
            )
            .expect("add assistant");
        storage
            .add_message(
                &session_id,
                "tool",
                "{\"matches\":[\"company_profiles/applied-optoelectronics/profile.md\"]}",
                Some(build_tool_message_metadata_parts(
                    "local_search_files",
                    Some("call_1"),
                    None,
                )),
            )
            .expect("add tool");

        let ctx = restore_context(&storage, &session_id, None, None);
        assert_eq!(ctx.messages.len(), 3);
        assert_eq!(ctx.messages[1].role, "assistant");
        let tool_calls = ctx.messages[1]
            .tool_calls
            .as_ref()
            .expect("assistant tool calls");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["id"], "call_1");
        assert_eq!(tool_calls[0]["function"]["name"], "local_search_files");
        assert_eq!(ctx.messages[2].role, "tool");
        assert_eq!(ctx.messages[2].tool_call_id.as_deref(), Some("call_1"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restore_context_preserves_message_metadata() {
        let root = make_temp_dir("hone_channels_restore_metadata");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");

        storage
            .add_message(
                &session_id,
                "assistant",
                "我先查本地画像。",
                Some(HashMap::from([
                    (
                        "assistant.tool_calls".to_string(),
                        serde_json::json!([{
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "local_search_files",
                                "arguments": "{\"query\":\"AAOI\"}"
                            }
                        }]),
                    ),
                    (
                        "codex_acp".to_string(),
                        serde_json::json!({
                            "segment_kind": "progress_note",
                            "channel_fields": {
                                "stream_kind": "agent_message_chunk"
                            }
                        }),
                    ),
                ])),
            )
            .expect("add assistant");
        storage
            .add_message(
                &session_id,
                "tool",
                "{\"matches\":[\"company_profiles/applied-optoelectronics/profile.md\"]}",
                Some(HashMap::from([
                    (
                        "tool_name".to_string(),
                        Value::String("local_search_files".to_string()),
                    ),
                    (
                        "tool_call_id".to_string(),
                        Value::String("call_1".to_string()),
                    ),
                    (
                        "codex_acp".to_string(),
                        serde_json::json!({
                            "segment_kind": "tool_result",
                            "channel_fields": {
                                "status": "completed"
                            }
                        }),
                    ),
                ])),
            )
            .expect("add tool");

        let ctx = restore_context(&storage, &session_id, None, None);
        assert_eq!(ctx.messages.len(), 2);
        assert_eq!(
            ctx.messages[0]
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("codex_acp")),
            Some(&serde_json::json!({
                "segment_kind": "progress_note",
                "channel_fields": {
                    "stream_kind": "agent_message_chunk"
                }
            }))
        );
        assert_eq!(
            ctx.messages[1]
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("codex_acp")),
            Some(&serde_json::json!({
                "segment_kind": "tool_result",
                "channel_fields": {
                    "status": "completed"
                }
            }))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn session_restore_limit_does_not_roll_before_compact_threshold() {
        let root = make_temp_dir("hone_channels_restore_limit_floor");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_tool_responses(Vec::new());
        let core = make_test_core_with_config(&root, llm, |config| {
            config.group_context.recent_context_limit = 6;
            config.group_context.compress_threshold_messages = 24;
        });

        let direct_actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let direct = AgentSession::new(core.clone(), direct_actor, "target");
        assert_eq!(
            direct.restore_max_messages,
            Some(DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT)
        );

        let group_actor =
            ActorIdentity::new("discord", "alice", Some("room-1".to_string())).expect("actor");
        let group_session =
            SessionIdentity::group(&group_actor.channel, "room-1").expect("group session");
        let group =
            AgentSession::new(core, group_actor, "room-1").with_session_identity(group_session);
        assert_eq!(group.restore_max_messages, Some(24));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_prompt_input_keeps_system_prompt_stable_when_related_skills_change() {
        let root = make_temp_dir("hone_channels_prompt_cache_stability");
        let system_skills = root.join("system_skills");
        let skill_dir = system_skills.join("alpha_skill");
        std::fs::create_dir_all(&skill_dir).expect("create skill dir");
        std::fs::write(
            skill_dir.join("SKILL.md"),
            concat!(
                "---\n",
                "name: Alpha Skill\n",
                "description: alpha analysis workflow\n",
                "when_to_use: use for alpha analysis tasks\n",
                "---\n\n",
                "body\n"
            ),
        )
        .expect("write skill");

        let llm = MockLlmProvider::with_tool_responses(Vec::new());
        let core = make_test_core_with_config(&root, llm, |config| {
            config.extra.insert(
                "skills_dir".to_string(),
                serde_yaml::Value::String(system_skills.to_string_lossy().to_string()),
            );
        });
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let session = AgentSession::new(core, actor, "target");

        let (system_with_match, runtime_with_match) =
            session.resolve_prompt_input("session-demo", "alpha skill");
        let (system_without_match, runtime_without_match) =
            session.resolve_prompt_input("session-demo", "plain greeting");

        assert_eq!(system_with_match, system_without_match);
        assert!(!system_with_match.contains("【Skills relevant to your task】"));
        assert!(runtime_with_match.contains("【本轮相关技能提示】"));
        assert!(runtime_with_match.contains("alpha_skill"));
        assert!(!runtime_without_match.contains("【本轮相关技能提示】"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn response_leaks_system_prompt_detects_prefixed_echo() {
        assert!(response_leaks_system_prompt(
            "\n### System Instructions ###\nsecret"
        ));
        assert!(!response_leaks_system_prompt("正常回复"));
    }

    #[test]
    fn compose_invoked_skill_runtime_input_keeps_user_supplement_outside_skill_context() {
        let runtime_input =
            compose_invoked_skill_runtime_input("SKILL_PROMPT", Some("finish the task"));
        assert!(runtime_input.contains("SKILL_PROMPT"));
        assert!(runtime_input.contains("【User Task After Invoking This Skill】"));
        assert!(runtime_input.contains("finish the task"));
    }

    #[test]
    fn unavailable_web_search_results_are_not_persisted() {
        let call = ToolCallMade {
            name: "web_search".to_string(),
            arguments: Value::Null,
            result: serde_json::json!({
                "status": "unavailable",
                "results": [],
            }),
            tool_call_id: None,
        };
        assert!(!should_persist_tool_result(&call));
    }

    #[test]
    fn restore_context_sanitizes_polluted_assistant_history() {
        let root = make_temp_dir("hone_channels_restore_sanitized_assistant");
        let storage = SessionStorage::new(&root);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let session_id = storage
            .create_session(
                Some("restore_sanitized"),
                Some(actor.clone()),
                Some(SessionIdentity::from_actor(&actor).expect("session identity")),
            )
            .expect("create");

        storage
            .add_message(
                &session_id,
                "assistant",
                "<think>先查一下</think>\n真正可见结论",
                None,
            )
            .expect("add assistant");
        storage
            .add_message(
                &session_id,
                "assistant",
                r#"<tool_call>{"name":"web_search","parameters":{"query":"AAPL"}}</tool_call>"#,
                None,
            )
            .expect("add polluted");

        let ctx = restore_context(&storage, &session_id, None, None);
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|message| message.content.as_deref())
            .collect();
        assert_eq!(contents, vec!["真正可见结论"]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn persistable_turn_from_response_stores_only_final_text_and_tool_call_metadata() {
        let response = AgentResponse {
            content: "最终结论：继续观察。".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "web_search".to_string(),
                arguments: serde_json::json!({"query": "AAOI latest earnings"}),
                result: serde_json::json!({"results": [{"title": "ok"}]}),
                tool_call_id: Some("call_1".to_string()),
            }],
            iterations: 2,
            success: true,
            error: None,
        };

        let message = persistable_turn_from_response(
            &response,
            Some(HashMap::from([(
                "message_id".to_string(),
                Value::String("msg-1".to_string()),
            )])),
        )
        .expect("persistable turn");

        assert_eq!(message.role, "assistant");
        assert_eq!(message.content.len(), 1);
        assert_eq!(message.content[0].part_type, "final");
        assert_eq!(
            message.content[0].text.as_deref(),
            Some("最终结论：继续观察。")
        );
        assert!(
            message
                .content
                .iter()
                .all(|part| { part.part_type != "tool_call" && part.part_type != "tool_result" })
        );

        let metadata = message.metadata.as_ref().expect("assistant metadata");
        assert_eq!(
            metadata.get("message_id").and_then(|value| value.as_str()),
            Some("msg-1")
        );
        let tool_calls = assistant_tool_calls_from_metadata(Some(metadata)).expect("tool calls");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["id"], "call_1");
        assert_eq!(tool_calls[0]["function"]["name"], "web_search");
    }

    #[test]
    fn persistable_turn_from_response_keeps_sqlite_runtime_history_on_final_text() {
        let root = make_temp_dir("hone_channels_persistable_turn_preview");
        let db_path = root.join("sessions.sqlite3");
        let storage = SessionStorage::with_options(
            root.join("sessions"),
            SessionStorageOptions {
                shadow_sqlite_db_path: Some(db_path.clone()),
                shadow_sqlite_enabled: true,
                runtime_backend: SessionRuntimeBackend::Sqlite,
            },
        );
        let actor = ActorIdentity::new("feishu", "preview-user", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");

        let response = AgentResponse {
            content: "用户可见结论".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "data_fetch".to_string(),
                arguments: serde_json::json!({"symbol": "MU"}),
                result: serde_json::json!({"price": 101}),
                tool_call_id: Some("call_preview".to_string()),
            }],
            iterations: 1,
            success: true,
            error: None,
        };
        let message = persistable_turn_from_response(&response, None).expect("persistable turn");
        storage
            .append_session_messages(
                &session_id,
                vec![session_message_from_normalized(
                    &message,
                    hone_core::beijing_now_rfc3339(),
                )],
            )
            .expect("append assistant");

        std::fs::remove_file(root.join("sessions").join(format!("{session_id}.json")))
            .expect("remove json fallback");
        let session = storage
            .load_session(&session_id)
            .expect("load session")
            .expect("session from sqlite");
        let assistant = session
            .messages
            .iter()
            .find(|message| message.role == "assistant")
            .expect("assistant message");

        assert_eq!(session_message_text(assistant), "用户可见结论");
        assert_eq!(assistant.content.len(), 1);
        assert_eq!(assistant.content[0].part_type, "final");
        assert!(
            assistant
                .content
                .iter()
                .all(|part| part.part_type != "tool_call" && part.part_type != "tool_result")
        );
        let tool_calls = assistant_tool_calls_from_metadata(assistant.metadata.as_ref())
            .expect("assistant tool call metadata");
        assert_eq!(tool_calls[0]["id"], "call_preview");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn successful_context_messages_persist_only_final_text_and_tool_metadata() {
        let root = make_temp_dir("hone_channels_context_messages_persist_sanitized");
        std::fs::create_dir_all(&root).expect("create root");
        let core = make_test_core(&root, MockLlmProvider::with_tool_responses(Vec::new()));
        let actor = ActorIdentity::new("feishu", "context-persist", None::<String>).expect("actor");
        let session = AgentSession::new(core.clone(), actor.clone(), "direct");
        core.session_storage
            .create_session_for_actor(&actor)
            .expect("create session");

        let response = AgentResponse {
            content: "最终识别结果".to_string(),
            tool_calls_made: vec![ToolCallMade {
                name: "web_search".to_string(),
                arguments: serde_json::json!({"query": "RKLB holdings screenshot"}),
                result: serde_json::json!({"results": [{"title": "ok"}]}),
                tool_call_id: Some("call_ctx_1".to_string()),
            }],
            iterations: 1,
            success: true,
            error: None,
        };
        let context_messages = vec![
            AgentMessage {
                role: "assistant".to_string(),
                content: Some("<think>先看图</think>\n处理中".to_string()),
                tool_calls: Some(vec![serde_json::json!({
                    "id": "call_ctx_1",
                    "type": "function",
                    "function": {
                        "name": "web_search",
                        "arguments": "{\"query\":\"RKLB holdings screenshot\"}"
                    }
                })]),
                tool_call_id: None,
                name: None,
                metadata: Some(HashMap::from([(
                    "runner".to_string(),
                    Value::String("opencode_acp".to_string()),
                )])),
            },
            AgentMessage {
                role: "tool".to_string(),
                content: Some(
                    "{\"session_id\":\"s1\",\"local_path\":\"/tmp/uploads/attachments.manifest.json\"}"
                        .to_string(),
                ),
                tool_calls: None,
                tool_call_id: Some("call_ctx_1".to_string()),
                name: Some("skill_tool".to_string()),
                metadata: None,
            },
        ];

        session.persist_successful_assistant_turn(
            &actor.session_id(),
            &response,
            Some(&context_messages),
        );

        let messages = core
            .session_storage
            .get_messages(&actor.session_id(), None)
            .expect("messages");
        let assistant = messages
            .iter()
            .find(|message| message.role == "assistant")
            .expect("assistant");
        assert_eq!(session_message_text(assistant), "最终识别结果");
        assert_eq!(assistant.content.len(), 1);
        assert_eq!(assistant.content[0].part_type, "final");
        assert!(
            assistant
                .content
                .iter()
                .all(|part| part.part_type != "tool_call" && part.part_type != "tool_result")
        );
        let metadata = assistant.metadata.as_ref().expect("metadata");
        assert_eq!(
            metadata.get("runner").and_then(|value| value.as_str()),
            Some("opencode_acp")
        );
        let tool_calls = assistant_tool_calls_from_metadata(Some(metadata)).expect("tool metadata");
        assert_eq!(tool_calls[0]["id"], "call_ctx_1");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn successful_web_search_results_are_persisted() {
        let call = ToolCallMade {
            name: "web_search".to_string(),
            arguments: Value::Null,
            result: serde_json::json!({
                "results": [{"title": "ok"}],
            }),
            tool_call_id: None,
        };
        assert!(should_persist_tool_result(&call));
    }

    #[test]
    fn namespaced_skill_runtime_tool_results_are_not_persisted() {
        for name in [
            "hone/skill_tool",
            "hone/load_skill",
            "hone/discover_skills",
            "Tool: hone/skill_tool",
        ] {
            let call = ToolCallMade {
                name: name.to_string(),
                arguments: Value::Null,
                result: serde_json::json!({}),
                tool_call_id: None,
            };
            assert!(!should_persist_tool_result(&call), "name={name}");
        }
    }

    #[test]
    fn restore_context_injects_invoked_skills_before_message_window() {
        let root = make_temp_dir("hone_channels_restore_invoked_skills");
        std::fs::create_dir_all(&root).expect("create root");
        let storage = hone_memory::SessionStorage::new(root.join("sessions"));
        let actor = ActorIdentity::new("discord", "bob", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");
        storage
            .add_message(&session_id, "user", "hello", None)
            .expect("add user");
        storage
            .add_message(&session_id, "assistant", "world", None)
            .expect("add assistant");
        let mut metadata = HashMap::new();
        metadata.insert(
            hone_memory::INVOKED_SKILLS_METADATA_KEY.to_string(),
            serde_json::json!([{
                "skill_name": "alpha",
                "display_name": "Alpha",
                "path": "slash:alpha",
                "prompt": "INVOKED_SKILL_PROMPT",
                "execution_context": "inline",
                "allowed_tools": [],
                "model": null,
                "effort": null,
                "agent": null,
                "loaded_from": "slash",
                "updated_at": hone_core::beijing_now_rfc3339()
            }]),
        );
        storage
            .update_metadata(&session_id, metadata)
            .expect("metadata");

        let ctx = restore_context(&storage, &session_id, Some(5), None);
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|m| m.content.as_deref())
            .collect();
        assert_eq!(contents, vec!["INVOKED_SKILL_PROMPT", "hello", "world"]);
    }

    #[test]
    fn restore_context_skips_invoked_skill_when_registry_disables_it() {
        let root = make_temp_dir("hone_channels_restore_disabled_skill");
        std::fs::create_dir_all(root.join("system/alpha")).expect("skill dir");
        std::fs::create_dir_all(root.join("custom")).expect("custom dir");
        std::fs::write(
            root.join("system/alpha/SKILL.md"),
            "---\nname: Alpha\ndescription: disabled restore\n---\n\nbody",
        )
        .expect("write skill");
        hone_tools::set_skill_enabled(
            &root.join("runtime").join("skill_registry.json"),
            "alpha",
            false,
        )
        .expect("disable alpha");

        let storage = hone_memory::SessionStorage::new(root.join("sessions"));
        let actor = ActorIdentity::new("discord", "bob", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");
        storage
            .add_message(&session_id, "assistant", "world", None)
            .expect("add assistant");
        let mut metadata = HashMap::new();
        metadata.insert(
            hone_memory::INVOKED_SKILLS_METADATA_KEY.to_string(),
            serde_json::json!([{
                "skill_name": "alpha",
                "display_name": "Alpha",
                "path": "slash:alpha",
                "prompt": "INVOKED_SKILL_PROMPT",
                "execution_context": "inline",
                "allowed_tools": [],
                "model": null,
                "effort": null,
                "agent": null,
                "loaded_from": "slash",
                "updated_at": hone_core::beijing_now_rfc3339()
            }]),
        );
        storage
            .update_metadata(&session_id, metadata)
            .expect("metadata");

        let runtime =
            hone_tools::SkillRuntime::new(root.join("system"), root.join("custom"), root.clone())
                .with_registry_path(root.join("runtime").join("skill_registry.json"));
        let ctx = restore_context(&storage, &session_id, Some(5), Some(&runtime));
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|m| m.content.as_deref())
            .collect();
        assert_eq!(contents, vec!["world"]);
    }

    #[test]
    fn restore_context_uses_only_messages_after_latest_compact_boundary() {
        let root = make_temp_dir("hone_channels_restore_after_boundary");
        std::fs::create_dir_all(&root).expect("create root");
        let storage = hone_memory::SessionStorage::new(root.join("sessions"));
        let actor = ActorIdentity::new("discord", "carol", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");
        storage
            .add_message(&session_id, "user", "before-compact", None)
            .expect("add old");
        storage
            .add_message(
                &session_id,
                "system",
                "Conversation compacted",
                Some(hone_memory::build_compact_boundary_metadata("auto", 4, 6)),
            )
            .expect("add boundary");
        storage
            .add_message(
                &session_id,
                "user",
                "【Compact Summary】\nsummary",
                Some(hone_memory::build_compact_summary_metadata("auto")),
            )
            .expect("add summary");
        storage
            .add_message(&session_id, "assistant", "after-compact", None)
            .expect("add assistant");

        let ctx = restore_context(&storage, &session_id, Some(10), None);
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|m| m.content.as_deref())
            .collect();
        assert_eq!(
            contents,
            vec!["【Compact Summary】\nsummary", "after-compact"]
        );
    }

    #[test]
    fn restore_context_keeps_invoked_skill_context_across_compact_boundary() {
        let root = make_temp_dir("hone_channels_restore_skill_after_boundary");
        std::fs::create_dir_all(&root).expect("create root");
        let storage = hone_memory::SessionStorage::new(root.join("sessions"));
        let actor = ActorIdentity::new("discord", "dana", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");

        let mut metadata = HashMap::new();
        metadata.insert(
            hone_memory::INVOKED_SKILLS_METADATA_KEY.to_string(),
            serde_json::json!([{
                "skill_name": "alpha",
                "display_name": "Alpha",
                "path": "skill:alpha",
                "prompt": "INVOKED_SKILL_PROMPT",
                "execution_context": "inline",
                "allowed_tools": [],
                "model": null,
                "effort": null,
                "agent": null,
                "loaded_from": "tool",
                "updated_at": hone_core::beijing_now_rfc3339()
            }]),
        );
        storage
            .update_metadata(&session_id, metadata)
            .expect("update metadata");
        storage
            .add_message(
                &session_id,
                "system",
                "Conversation compacted",
                Some(hone_memory::build_compact_boundary_metadata("auto", 3, 5)),
            )
            .expect("add boundary");
        storage
            .add_message(
                &session_id,
                "user",
                "【Compact Summary】\nsummary",
                Some(hone_memory::build_compact_summary_metadata("auto")),
            )
            .expect("add summary");

        let ctx = restore_context(&storage, &session_id, Some(10), None);
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|m| m.content.as_deref())
            .collect();
        assert_eq!(
            contents,
            vec!["INVOKED_SKILL_PROMPT", "【Compact Summary】\nsummary"]
        );
    }

    #[test]
    fn restore_context_avoids_duplicate_skill_prompt_when_compact_snapshot_exists() {
        let root = make_temp_dir("hone_channels_restore_skill_snapshot_dedup");
        std::fs::create_dir_all(&root).expect("create root");
        let storage = hone_memory::SessionStorage::new(root.join("sessions"));
        let actor = ActorIdentity::new("discord", "erin", None::<String>).expect("actor");
        let session_id = storage
            .create_session_for_actor(&actor)
            .expect("create session");

        let mut metadata = HashMap::new();
        metadata.insert(
            hone_memory::INVOKED_SKILLS_METADATA_KEY.to_string(),
            serde_json::json!([{
                "skill_name": "alpha",
                "display_name": "Alpha",
                "path": "skill:alpha",
                "prompt": "INVOKED_SKILL_PROMPT",
                "execution_context": "inline",
                "allowed_tools": [],
                "model": null,
                "effort": null,
                "agent": null,
                "loaded_from": "tool",
                "updated_at": hone_core::beijing_now_rfc3339()
            }]),
        );
        storage
            .update_metadata(&session_id, metadata)
            .expect("update metadata");
        storage
            .add_message(
                &session_id,
                "system",
                "Conversation compacted",
                Some(hone_memory::build_compact_boundary_metadata("auto", 3, 5)),
            )
            .expect("add boundary");
        storage
            .add_message(
                &session_id,
                "user",
                "【Compact Summary】\nsummary",
                Some(hone_memory::build_compact_summary_metadata("auto")),
            )
            .expect("add summary");
        storage
            .add_message(
                &session_id,
                "user",
                "INVOKED_SKILL_PROMPT",
                Some(hone_memory::build_compact_skill_snapshot_metadata("alpha")),
            )
            .expect("add skill snapshot");

        let ctx = restore_context(&storage, &session_id, Some(10), None);
        let contents: Vec<_> = ctx
            .messages
            .iter()
            .filter_map(|m| m.content.as_deref())
            .collect();
        assert_eq!(
            contents,
            vec!["【Compact Summary】\nsummary", "INVOKED_SKILL_PROMPT"]
        );
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
    async fn context_overflow_auto_compacts_and_retries_successfully() {
        let root = make_temp_dir("hone_channels_context_overflow_retry_success");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_chat_and_tool_responses(
            vec![Ok(ChatResult {
                content: "压缩后的摘要".to_string(),
                usage: None,
            })],
            vec![
                Err(hone_core::HoneError::Llm(
                    "LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"
                        .to_string(),
                )),
                Ok(ChatResponse {
                    content: "恢复后的正常回复".to_string(),
                    tool_calls: None,
                    usage: None,
                }),
            ],
        );
        let core = make_test_core(&root, llm.clone());
        let actor = ActorIdentity::new("discord", "overflow-ok", None::<String>).expect("actor");
        let session = AgentSession::new(core, actor, "direct");

        let result = session
            .run("请继续分析这个话题", AgentRunOptions::default())
            .await;

        assert!(result.response.success, "{:?}", result.response.error);
        assert_eq!(result.response.content, "恢复后的正常回复");
        assert_eq!(llm.chat_calls(), 1);
        assert_eq!(llm.chat_with_tools_calls(), 2);

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn context_overflow_failure_is_rewritten_to_friendly_message() {
        let root = make_temp_dir("hone_channels_context_overflow_retry_failure");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_chat_and_tool_responses(
            vec![Ok(ChatResult {
                content: "压缩后的摘要".to_string(),
                usage: None,
            })],
            vec![
                Err(hone_core::HoneError::Llm(
                    "LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"
                        .to_string(),
                )),
                Err(hone_core::HoneError::Llm(
                    "LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"
                        .to_string(),
                )),
            ],
        );
        let core = make_test_core(&root, llm.clone());
        let actor = ActorIdentity::new("discord", "overflow-fail", None::<String>).expect("actor");
        let session = AgentSession::new(core, actor, "direct");

        let result = session
            .run("请继续分析这个话题", AgentRunOptions::default())
            .await;

        assert!(!result.response.success);
        let err = result.response.error.expect("friendly error");
        assert_eq!(err, CONTEXT_OVERFLOW_FALLBACK_MESSAGE);
        assert!(!err.contains("bad_request_error"));
        assert!(!err.contains("invalid params"));
        assert_eq!(llm.chat_calls(), 1);
        assert_eq!(llm.chat_with_tools_calls(), 2);

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn manual_compact_does_not_consume_quota_or_persist_command_message() {
        let root = make_temp_dir("hone_channels_manual_compact");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_chat_responses(vec![ChatResult {
            content: "summary".to_string(),
            usage: None,
        }]);
        let core = make_test_core(&root, llm.clone());
        let actor = ActorIdentity::new("discord", "frank", None::<String>).expect("actor");
        let session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());
        core.session_storage
            .create_session_for_actor(&actor)
            .expect("create session");
        core.session_storage
            .add_message(&actor.session_id(), "user", "hello", None)
            .expect("seed user");
        core.session_storage
            .add_message(&actor.session_id(), "assistant", "world", None)
            .expect("seed assistant");

        let result = session
            .run(
                "/compact keep only the durable decisions",
                AgentRunOptions::default(),
            )
            .await;

        assert!(result.response.success, "{:?}", result.response.error);
        assert_eq!(result.response.content, "Conversation compacted.");
        assert_eq!(llm.chat_calls(), 1);

        let today = hone_core::beijing_now().format("%F").to_string();
        let snapshot = core
            .conversation_quota_storage
            .snapshot_for_date(&actor, &today)
            .expect("snapshot");
        assert!(snapshot.is_none());

        let messages = core
            .session_storage
            .get_messages(&actor.session_id(), None)
            .expect("messages");
        assert_eq!(messages.len(), 4);
        assert_eq!(
            hone_memory::session_message_text(&messages[0]),
            "Conversation compacted"
        );
        assert_eq!(
            hone_memory::session_message_text(&messages[1]),
            "【Compact Summary】\nsummary"
        );
        assert_eq!(hone_memory::session_message_text(&messages[2]), "hello");
        assert_eq!(hone_memory::session_message_text(&messages[3]), "world");
        assert!(
            messages
                .iter()
                .all(|message| !hone_memory::session_message_text(message).contains("/compact"))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn auto_compact_uses_low_group_threshold_and_keeps_recent_window() {
        let root = make_temp_dir("hone_channels_auto_compact_low_threshold");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_chat_and_tool_responses(
            vec![Ok(ChatResult {
                content: "group-summary".to_string(),
                usage: None,
            })],
            vec![Ok(ChatResponse {
                content: "after-compact".to_string(),
                tool_calls: None,
                usage: None,
            })],
        );
        let core = make_test_core_with_config(&root, llm.clone(), |config| {
            config.group_context.compress_threshold_messages = 1;
            config.group_context.compress_threshold_bytes = 1024;
            config.group_context.retain_recent_after_compress = 1;
            config.group_context.recent_context_limit = 6;
        });
        let actor =
            ActorIdentity::new("discord", "gina", Some("room-1".to_string())).expect("actor");
        let group_session =
            SessionIdentity::group(&actor.channel, actor.channel_scope.clone().unwrap())
                .expect("group session");
        let session = AgentSession::new(core.clone(), actor.clone(), "room-1")
            .with_session_identity(group_session.clone());
        core.session_storage
            .create_session_for_identity(&group_session, Some(&actor))
            .expect("create session");
        core.session_storage
            .add_message(&group_session.session_id(), "user", "old-user", None)
            .expect("seed user");
        core.session_storage
            .add_message(
                &group_session.session_id(),
                "assistant",
                "old-assistant",
                None,
            )
            .expect("seed assistant");

        let result = session.run("new-user", AgentRunOptions::default()).await;

        assert!(result.response.success, "{:?}", result.response.error);
        assert_eq!(result.response.content, "after-compact");
        assert_eq!(llm.chat_calls(), 1);
        assert_eq!(llm.chat_with_tools_calls(), 1);

        let messages = core
            .session_storage
            .get_messages(&group_session.session_id(), None)
            .expect("messages");
        let contents: Vec<_> = messages
            .iter()
            .map(hone_memory::session_message_text)
            .collect();
        assert_eq!(
            contents,
            vec![
                "Conversation compacted",
                "【Compact Summary】\ngroup-summary",
                "new-user",
                "after-compact",
            ]
        );
        assert!(hone_memory::message_is_compact_boundary(
            messages[0].metadata.as_ref()
        ));
        assert!(hone_memory::message_is_compact_summary(
            messages[1].metadata.as_ref()
        ));

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn auto_compact_summary_excludes_latest_user_turn_from_prompt() {
        let root = make_temp_dir("hone_channels_auto_compact_excludes_latest_turn");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_chat_and_tool_responses(
            vec![Ok(ChatResult {
                content: "summary".to_string(),
                usage: None,
            })],
            vec![Ok(ChatResponse {
                content: "after-compact".to_string(),
                tool_calls: None,
                usage: None,
            })],
        );
        let core = make_test_core_with_config(&root, llm.clone(), |config| {
            config.group_context.compress_threshold_messages = 1;
            config.group_context.compress_threshold_bytes = 1024;
            config.group_context.retain_recent_after_compress = 1;
            config.group_context.recent_context_limit = 6;
        });
        let actor =
            ActorIdentity::new("discord", "henry", Some("room-2".to_string())).expect("actor");
        let group_session =
            SessionIdentity::group(&actor.channel, actor.channel_scope.clone().unwrap())
                .expect("group session");
        let session = AgentSession::new(core.clone(), actor.clone(), "room-2")
            .with_session_identity(group_session.clone());
        core.session_storage
            .create_session_for_identity(&group_session, Some(&actor))
            .expect("create session");
        core.session_storage
            .add_message(&group_session.session_id(), "user", "older topic", None)
            .expect("seed older user");
        core.session_storage
            .add_message(
                &group_session.session_id(),
                "assistant",
                "older reply",
                None,
            )
            .expect("seed older assistant");

        let result = session
            .run("latest unresolved question", AgentRunOptions::default())
            .await;

        assert!(result.response.success, "{:?}", result.response.error);
        let compact_prompt = llm.last_chat_prompt().expect("compact prompt");
        assert!(compact_prompt.contains("older topic"));
        assert!(compact_prompt.contains("older reply"));
        assert!(!compact_prompt.contains("latest unresolved question"));

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

    #[derive(Default)]
    struct RecordingListener {
        events: tokio::sync::Mutex<Vec<AgentSessionEvent>>,
    }

    #[async_trait]
    impl AgentSessionListener for RecordingListener {
        async fn on_event(&self, event: AgentSessionEvent) {
            self.events.lock().await.push(event);
        }
    }

    #[tokio::test]
    async fn session_event_emitter_relativizes_user_visible_paths() {
        let root = "/tmp/hone-agent-sandboxes/telegram/direct8039067465";
        let listener = Arc::new(RecordingListener::default());
        let emitter = SessionEventEmitter {
            listeners: vec![listener.clone()],
            channel: "telegram".to_string(),
            user_id: "8039067465".to_string(),
            session_id: "session".to_string(),
            message_id: None,
            working_directory: root.to_string(),
        };

        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "tool.execute",
                detail: Some(format!(
                    "Edit {root}/company_profiles/sandisk/profile.md and /Users/bytedance/private.txt"
                )),
            })
            .await;
        emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: "hone/skill_tool".to_string(),
                status: "start".to_string(),
                message: Some(format!(
                    "Edit {root}/company_profiles/micron-technology/profile.md"
                )),
                reasoning: Some(format!(
                    "Edit {root}/data/research/notes.md and /etc/passwd"
                )),
            })
            .await;

        let events = listener.events.lock().await.clone();
        assert!(matches!(
            &events[0],
            AgentSessionEvent::Progress {
                detail: Some(detail),
                ..
            } if detail
                == "Edit company_profiles/sandisk/profile.md and <absolute-path>/private.txt"
        ));
        assert!(matches!(
            &events[1],
            AgentSessionEvent::ToolStatus {
                message: Some(message),
                reasoning: Some(reasoning),
                ..
            } if message == "Edit company_profiles/micron-technology/profile.md"
                && reasoning == "Edit data/research/notes.md and <absolute-path>/passwd"
        ));
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
