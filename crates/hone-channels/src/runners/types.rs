use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_core::agent::{AgentContext, AgentMessage, AgentResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::agent_session::GeminiStreamOptions;
pub(crate) use crate::run_event::RunEvent as AgentRunnerEvent;

/// Controls whether a runner may publish a narrowly bounded, irreversible
/// answer prefix while the rest of an Agent answer remains deferred.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TerminalStreamPolicy {
    /// Preserve the runner's ordinary streaming behavior.
    #[default]
    Disabled,
    /// Permit one ACKed canonical investment header beginning with
    /// `数据时间：北京时间 ...；行情口径：...`. It may be the typed service-owned
    /// Web prefix or a complete header from an eligible natural final.
    CanonicalInvestmentHeader,
}

#[async_trait]
pub trait AgentRunnerEmitter: Send + Sync {
    async fn emit(&self, event: AgentRunnerEvent);

    /// Deliver an irreversible typed answer delta and report whether the
    /// downstream transport accepted it. Ordinary emitters keep legacy
    /// behavior; the Session/Web bridge overrides this so a closed SSE receiver
    /// cannot create phantom committed bytes.
    async fn emit_committed(&self, event: AgentRunnerEvent) -> bool {
        self.emit(event).await;
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RunnerTimeouts {
    pub step: Duration,
    pub overall: Duration,
}

#[derive(Clone)]
pub struct AgentRunnerRequest {
    pub session_id: String,
    pub actor_label: String,
    pub actor: ActorIdentity,
    pub channel_target: String,
    pub allow_cron: bool,
    pub config_path: String,
    pub runtime_dir: String,
    pub system_prompt: String,
    pub runtime_input: String,
    pub context: AgentContext,
    pub timeout: Option<Duration>,
    pub gemini_stream: GeminiStreamOptions,
    pub session_metadata: HashMap<String, Value>,
    pub working_directory: String,
    pub allowed_tools: Option<Vec<String>>,
    pub max_tool_calls: Option<u32>,
    pub tool_call_limits: Option<HashMap<String, u32>>,
    /// Enables the standard same-Agent finance tool loop independently of
    /// channel-specific streaming behavior.
    pub agent_owned_finance_loop: bool,
    /// Typed, service-owned first line for a Web finance turn. Explicit ticker
    /// seeds may commit it before the first model call; otherwise the Agent may
    /// commit it only after a valid read-only DataFetch batch activates the
    /// finance protocol. This is never parsed from user text.
    pub service_owned_initial_prefix: Option<ServiceOwnedInitialPrefix>,
    pub terminal_stream_policy: TerminalStreamPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceOwnedInitialPrefix {
    pub content: String,
    pub commit_before_model: bool,
}

pub struct AgentRunnerResult {
    pub response: AgentResponse,
    pub streamed_output: bool,
    /// Exact prefix already emitted through one or more
    /// `CommittedStreamDelta` events.
    ///
    /// AgentSession uses this value to publish only the remaining suffix at
    /// the terminal boundary. `None` means no user-visible prefix was
    /// committed by this runner attempt.
    pub committed_visible_prefix: Option<String>,
    pub terminal_error_emitted: bool,
    pub session_metadata_updates: HashMap<String, Value>,
    pub context_messages: Option<Vec<AgentMessage>>,
}

/// Agent 执行器抽象。
///
/// **历史还原契约**：Runner **不应该**直接读 `SessionStorage` / `SessionMessage`。
/// 上游 `AgentSession` 会用 `restore_context` 把 session 历史构造成
/// `AgentContext`,以 `AgentRunnerRequest.context` 注入。Runner 只消费
/// `AgentContext`（或它的 `normalized_history_json()` 序列化形式）。
///
/// 这么约束的原因是让 session 持久化 schema 的任何变更都只需要改动
/// `restore_context` 一处,不需要同步到每个 runner 实现里。
#[async_trait]
pub trait AgentRunner: Send + Sync {
    fn name(&self) -> &'static str;

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult;

    /// Runner 是否自己管理对话上下文 / 历史 / 内置压缩。
    ///
    /// 返回 true 时 honeclaw 不会对其触发 SessionCompactor，也不会在每轮 prompt
    /// 里再拼接 `latest_compact_summary`，由 runner 内置的 ACP session 机制累积
    /// 与压缩。仅 ACP 系列 runner（codex_acp / opencode_acp）应当 override 为
    /// true；其它 runner 保持默认 false。
    fn manages_own_context(&self) -> bool {
        false
    }
}
