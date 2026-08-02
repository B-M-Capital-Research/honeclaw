use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_core::agent::{AgentContext, AgentMessage, AgentResponse};
use hone_core::config::AgentConversationStrategy;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::agent_session::GeminiStreamOptions;
pub(crate) use crate::run_event::RunEvent as AgentRunnerEvent;

/// Versioned wire profiles observed from real ACP adapters. The variants are
/// intentionally adapter-specific: sharing ACP method names does not imply
/// identical stream updates or presentation detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AcpAdapterKind {
    #[serde(rename = "codex-acp")]
    CodexAcp,
    #[serde(rename = "opencode")]
    OpenCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AcpStreamDialect {
    #[serde(rename = "codex-acp/1.1.7")]
    CodexAcp1_1_7,
    #[serde(rename = "opencode/1.18.11")]
    OpenCode1_18_11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpCompatibilityStatus {
    Validated,
    CompatibleNewer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AcpAdapterProfile {
    pub adapter: AcpAdapterKind,
    pub detected_version: String,
    pub dialect: AcpStreamDialect,
    pub compatibility: AcpCompatibilityStatus,
}

/// Adapter-specific workspace preparation that is independent from
/// conversation ownership and stream event shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSkillProjection {
    CodexWorkspace,
}

impl AcpStreamDialect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CodexAcp1_1_7 => "codex-acp/1.1.7",
            Self::OpenCode1_18_11 => "opencode/1.18.11",
        }
    }

    pub fn baseline_version(self) -> &'static str {
        match self {
            Self::CodexAcp1_1_7 => "1.1.7",
            Self::OpenCode1_18_11 => "1.18.11",
        }
    }
}

impl AcpAdapterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CodexAcp => "codex-acp",
            Self::OpenCode => "opencode",
        }
    }
}

impl AcpCompatibilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Validated => "validated",
            Self::CompatibleNewer => "compatible_newer",
        }
    }
}

impl AcpAdapterProfile {
    pub fn baseline_version(&self) -> &'static str {
        self.dialect.baseline_version()
    }
}

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
pub enum RunnerConversationInput {
    NativePersistent {
        developer_instructions: String,
        current_user_turn: String,
    },
    StructuredReplay {
        system_prompt: String,
        current_user_turn: String,
        context: AgentContext,
    },
    EphemeralCompiledPrompt {
        system_prompt: String,
        current_user_turn: String,
        context: AgentContext,
    },
}

impl RunnerConversationInput {
    pub fn prepare(
        strategy: AgentConversationStrategy,
        system_prompt: String,
        current_user_turn: String,
        context: AgentContext,
    ) -> Self {
        match strategy {
            AgentConversationStrategy::NativePersistent => Self::NativePersistent {
                developer_instructions: system_prompt,
                current_user_turn,
            },
            AgentConversationStrategy::StructuredReplay => Self::StructuredReplay {
                system_prompt,
                current_user_turn,
                context,
            },
            AgentConversationStrategy::EphemeralCompiledPrompt => Self::EphemeralCompiledPrompt {
                system_prompt,
                current_user_turn,
                context,
            },
        }
    }

    pub fn current_user_turn(&self) -> &str {
        match self {
            Self::NativePersistent {
                current_user_turn, ..
            }
            | Self::StructuredReplay {
                current_user_turn, ..
            }
            | Self::EphemeralCompiledPrompt {
                current_user_turn, ..
            } => current_user_turn,
        }
    }

    pub fn current_user_turn_mut(&mut self) -> &mut String {
        match self {
            Self::NativePersistent {
                current_user_turn, ..
            }
            | Self::StructuredReplay {
                current_user_turn, ..
            }
            | Self::EphemeralCompiledPrompt {
                current_user_turn, ..
            } => current_user_turn,
        }
    }

    pub fn native_parts(&self) -> Option<(&str, &str)> {
        match self {
            Self::NativePersistent {
                developer_instructions,
                current_user_turn,
            } => Some((developer_instructions, current_user_turn)),
            Self::StructuredReplay { .. } | Self::EphemeralCompiledPrompt { .. } => None,
        }
    }

    pub fn replay_parts(&self) -> Option<(&str, &str, &AgentContext)> {
        match self {
            Self::StructuredReplay {
                system_prompt,
                current_user_turn,
                context,
            }
            | Self::EphemeralCompiledPrompt {
                system_prompt,
                current_user_turn,
                context,
            } => Some((system_prompt, current_user_turn, context)),
            Self::NativePersistent { .. } => None,
        }
    }

    pub fn into_replay_parts(self) -> Option<(String, String, AgentContext)> {
        match self {
            Self::StructuredReplay {
                system_prompt,
                current_user_turn,
                context,
            }
            | Self::EphemeralCompiledPrompt {
                system_prompt,
                current_user_turn,
                context,
            } => Some((system_prompt, current_user_turn, context)),
            Self::NativePersistent { .. } => None,
        }
    }
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
    pub conversation: RunnerConversationInput,
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
/// **会话契约**：Runner **不应该**直接读 `SessionStorage` / `SessionMessage`。
/// 上游先选择实际 runner，再按 [`AgentConversationStrategy`] 构造一个有角色边界的
/// [`RunnerConversationInput`]。Native persistent runner 从类型上拿不到 Hone 历史；
/// replay runner 则在适配器边界消费 `AgentContext`。
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

    fn conversation_strategy(&self) -> AgentConversationStrategy {
        AgentConversationStrategy::StructuredReplay
    }

    fn acp_adapter_kind(&self) -> Option<AcpAdapterKind> {
        None
    }

    fn native_skill_projection(&self) -> Option<NativeSkillProjection> {
        None
    }
}
