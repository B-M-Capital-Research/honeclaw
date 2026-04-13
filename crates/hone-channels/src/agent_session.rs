//! Agent session abstraction shared across channels.

use async_trait::async_trait;
use hone_core::agent::{AgentContext, AgentResponse, ToolCallMade};
use hone_core::{ActorIdentity, SessionIdentity};
use hone_memory::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, SessionStorage,
    build_tool_message_metadata, has_compact_skill_snapshot, invoked_skills_from_metadata,
    message_is_compact_boundary, message_is_slash_skill, restore_tool_message,
    select_messages_after_compact_boundary,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::HoneBotCore;
use crate::execution::{
    ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService,
};
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::prompt_audit::PromptAuditMetadata;
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult};

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
const EMPTY_SUCCESS_FALLBACK_MESSAGE: &str =
    "这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。";

fn should_return_runner_result(result: &AgentRunnerResult) -> bool {
    // 失败直接返回；成功时只要有正文或工具调用，也视为已拿到有效结果。
    //
    // 注意：`streamed_output` 仅表示 runner 具备流式能力，不代表这次真的输出过内容。
    // opencode_acp 会始终把它设为 true，因此不能再把它当成“已有输出”的依据，
    // 否则空回复成功态会被直接放过，前端就可能一直停留在“思考中”。
    !result.response.success
        || !result.response.content.trim().is_empty()
        || !result.response.tool_calls_made.is_empty()
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
    ) -> AgentRunnerResult {
        let mut last_result = runner.run(request.clone(), self.runner_emitter()).await;

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
        if !related_skills.is_empty() {
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
            prompt_options.extra_sections.push(format!(
                "【Skills relevant to your task】\n{}\n如这些技能已覆盖下一步，就直接用 skill_tool（或 MCP 下的 hone/skill_tool）；否则再调用 discover_skills（或 hone/discover_skills）。",
                listing
            ));
        }
        let bundle = build_prompt_bundle(
            &self.core.config,
            &self.core.session_storage,
            &self.actor.channel,
            session_id,
            &Default::default(),
            &prompt_options,
        );
        (
            bundle.system_prompt(),
            bundle.compose_user_input(user_input),
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
            channel: self.actor.channel.clone(),
            user_id: self.actor.user_id.clone(),
            session_id: self.session_id.clone(),
            message_id: self.message_id.clone(),
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

        let mut context = restore_context(
            &self.core.session_storage,
            &session_id,
            self.restore_max_messages,
            Some(
                &hone_tools::SkillRuntime::new(
                    self.core.configured_system_skills_dir(),
                    self.core.configured_custom_skills_dir(),
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                )
                .with_registry_path(self.core.configured_skill_registry_path()),
            ),
        );
        context.set_actor_identity(&self.actor);

        // 因为已在 fast persist 阶段把用户消息写入 DB，restore_context 会把它加载进来。
        // 但 runtime_input 也会再次传给 runner（避免重复），所以这里把末尾的用户消息弹出。
        if let Some(last) = context.messages.last() {
            if last.role == "user" && last.content.as_deref() == Some(persisted_user_input) {
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

        let (system_prompt, runtime_input) =
            self.resolve_prompt_input(&session_id, runtime_user_input);
        let execution = match ExecutionService::new(self.core.clone()).prepare(ExecutionRequest {
            mode: ExecutionMode::PersistentConversation,
            session_id: session_id.clone(),
            actor: self.actor.clone(),
            channel_target: self.channel_target.clone(),
            allow_cron: self.allow_cron,
            system_prompt,
            runtime_input,
            context,
            timeout: options.timeout,
            gemini_stream: self.default_gemini_stream_options(options.timeout),
            session_metadata: self.load_session_metadata(&session_id),
            model_override: options.model_override.clone(),
            runner_selection: ExecutionRunnerSelection::Configured,
            allowed_tools: None,
            max_tool_calls: None,
            prompt_audit: Some(PromptAuditMetadata {
                session_identity: self.session_identity.clone(),
                message_id: self.message_id.clone(),
            }),
        }) {
            Ok(execution) => execution,
            Err(err) => {
                tracing::error!("[AgentSession] execution prepare failed: {}", err);
                if let Some(reservation) = quota_reservation.as_ref() {
                    let _ = self
                        .core
                        .conversation_quota_storage
                        .release_daily_conversation(reservation);
                }
                let kind = if err.contains("sandbox") {
                    AgentSessionErrorKind::Io
                } else {
                    AgentSessionErrorKind::AgentFailed
                };
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

        let runner_result = self
            .run_runner_with_empty_success_retry(
                execution.runner.as_ref(),
                execution.runner_name,
                &session_id,
                execution.runner_request,
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
                execution.runner_name,
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
                if !should_persist_tool_result(tool_call) {
                    continue;
                }
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
                    ctx.add_user_message(&message.content);
                }
            }
            "assistant" => ctx.add_assistant_message(&message.content, None),
            "tool" => {
                if let Some((tool_call_id, tool_name, result)) =
                    restore_tool_message(&message.content, message.metadata.as_ref())
                {
                    ctx.add_tool_result(&tool_call_id, &tool_name, &result);
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
    use hone_core::config::HoneConfig;
    use hone_llm::provider::ChatResult;
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
        chat_calls: usize,
        chat_with_tools_calls: usize,
        chat_responses: std::collections::VecDeque<ChatResult>,
        responses: std::collections::VecDeque<ChatResponse>,
    }

    impl MockLlmProvider {
        fn with_chat_and_tool_responses(
            chat_responses: Vec<ChatResult>,
            responses: Vec<ChatResponse>,
        ) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    chat_responses: chat_responses.into(),
                    responses: responses.into(),
                })),
            }
        }

        fn with_chat_responses(responses: Vec<ChatResult>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    chat_responses: responses.into(),
                    responses: Default::default(),
                })),
            }
        }

        fn with_tool_responses(responses: Vec<ChatResponse>) -> Self {
            Self {
                state: Arc::new(Mutex::new(MockLlmState {
                    chat_calls: 0,
                    chat_with_tools_calls: 0,
                    chat_responses: Default::default(),
                    responses: responses.into(),
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
            state
                .chat_responses
                .pop_front()
                .ok_or_else(|| hone_core::HoneError::Llm("no more mock chat responses".to_string()))
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
        };

        assert!(!should_return_runner_result(&result));

        let mut with_content = result;
        with_content.response.content = "hello".to_string();
        assert!(should_return_runner_result(&with_content));
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
        assert_eq!(messages[0].content, "Conversation compacted");
        assert_eq!(messages[1].content, "【Compact Summary】\nsummary");
        assert_eq!(messages[2].content, "hello");
        assert_eq!(messages[3].content, "world");
        assert!(
            messages
                .iter()
                .all(|message| !message.content.contains("/compact"))
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn auto_compact_uses_low_group_threshold_and_keeps_recent_window() {
        let root = make_temp_dir("hone_channels_auto_compact_low_threshold");
        std::fs::create_dir_all(&root).expect("create root");
        let llm = MockLlmProvider::with_chat_and_tool_responses(
            vec![ChatResult {
                content: "group-summary".to_string(),
                usage: None,
            }],
            vec![ChatResponse {
                content: "after-compact".to_string(),
                tool_calls: None,
                usage: None,
            }],
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
            .map(|message| message.content.as_str())
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
