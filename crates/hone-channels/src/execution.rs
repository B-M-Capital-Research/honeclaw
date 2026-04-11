use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use hone_core::ActorIdentity;
use hone_core::agent::AgentContext;
use serde_json::Value;

use crate::HoneBotCore;
use crate::agent_session::GeminiStreamOptions;
use crate::core::runtime_config_path;
use crate::prompt_audit::{PromptAuditMetadata, write_prompt_audit};
use crate::runners::{AgentRunner, AgentRunnerRequest, FunctionCallingReasoningRunner};
use crate::sandbox::ensure_actor_sandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    PersistentConversation,
    TransientTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionRunnerSelection {
    Configured,
    AuxiliaryFunctionCalling { max_iterations: u32 },
}

#[derive(Clone)]
pub struct ExecutionRequest {
    pub mode: ExecutionMode,
    pub session_id: String,
    pub actor: ActorIdentity,
    pub channel_target: String,
    pub allow_cron: bool,
    pub system_prompt: String,
    pub runtime_input: String,
    pub context: AgentContext,
    pub timeout: Option<Duration>,
    pub gemini_stream: GeminiStreamOptions,
    pub session_metadata: HashMap<String, Value>,
    pub model_override: Option<String>,
    pub runner_selection: ExecutionRunnerSelection,
    pub allowed_tools: Option<Vec<String>>,
    pub max_tool_calls: Option<u32>,
    pub prompt_audit: Option<PromptAuditMetadata>,
}

pub struct PreparedExecution {
    pub runner_name: &'static str,
    pub runner: Box<dyn AgentRunner>,
    pub runner_request: AgentRunnerRequest,
}

pub struct ExecutionService {
    core: Arc<HoneBotCore>,
}

impl ExecutionService {
    pub fn new(core: Arc<HoneBotCore>) -> Self {
        Self { core }
    }

    pub fn prepare(&self, request: ExecutionRequest) -> Result<PreparedExecution, String> {
        if let Some(metadata) = request.prompt_audit.as_ref() {
            if let Err(err) = write_prompt_audit(
                &self.core.config,
                &request.actor,
                &request.session_id,
                metadata,
                &request.system_prompt,
                &request.runtime_input,
            ) {
                tracing::warn!(
                    "[PromptAudit] failed to write audit channel={} session_id={}: {}",
                    request.actor.channel,
                    request.session_id,
                    err
                );
            }
        }

        let tool_registry = self.core.create_tool_registry(
            Some(&request.actor),
            &request.channel_target,
            request.allow_cron,
        );
        let runner: Box<dyn AgentRunner> = match request.runner_selection {
            ExecutionRunnerSelection::Configured => self.core.create_runner_with_model_override(
                &request.system_prompt,
                tool_registry,
                request.model_override.as_deref(),
            )?,
            ExecutionRunnerSelection::AuxiliaryFunctionCalling { max_iterations } => {
                let llm = self.core.auxiliary_llm.clone().ok_or_else(|| {
                    "execution prepare failed: auxiliary llm unavailable".to_string()
                })?;
                Box::new(FunctionCallingReasoningRunner::new(
                    llm,
                    Arc::new(tool_registry),
                    request.system_prompt.clone(),
                    max_iterations,
                    self.core.llm_audit.clone(),
                ))
            }
        };
        let runner_name = runner.name();
        let working_directory = ensure_actor_sandbox(&request.actor)
            .map_err(|err| format!("actor sandbox 初始化失败: {err}"))?
            .to_string_lossy()
            .to_string();
        let actor_label = match request.mode {
            ExecutionMode::PersistentConversation => request.actor.user_id.clone(),
            ExecutionMode::TransientTask => request.actor.session_id(),
        };

        Ok(PreparedExecution {
            runner_name,
            runner,
            runner_request: AgentRunnerRequest {
                session_id: request.session_id,
                actor_label,
                actor: request.actor,
                channel_target: request.channel_target,
                allow_cron: request.allow_cron,
                config_path: runtime_config_path(),
                system_prompt: request.system_prompt,
                runtime_input: request.runtime_input,
                context: request.context,
                timeout: request.timeout,
                gemini_stream: request.gemini_stream,
                session_metadata: request.session_metadata,
                working_directory,
                allowed_tools: request.allowed_tools,
                max_tool_calls: request.max_tool_calls,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService};
    use crate::HoneBotCore;
    use crate::agent_session::GeminiStreamOptions;
    use async_trait::async_trait;
    use futures::stream::{self, BoxStream};
    use hone_core::agent::AgentContext;
    use hone_core::{ActorIdentity, HoneConfig, HoneError};
    use hone_llm::provider::{ChatResponse, ChatResult};
    use hone_llm::{LlmProvider, Message};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    #[derive(Clone, Default)]
    struct MockLlmProvider;

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn chat(
            &self,
            _messages: &[Message],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResult> {
            Err(HoneError::Llm("unused chat call".to_string()))
        }

        async fn chat_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[Value],
            _model: Option<&str>,
        ) -> hone_core::HoneResult<ChatResponse> {
            Err(HoneError::Llm("unused tool chat call".to_string()))
        }

        fn chat_stream<'a>(
            &'a self,
            _messages: &'a [Message],
            _model: Option<&'a str>,
        ) -> BoxStream<'a, hone_core::HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    fn temp_root(name: &str) -> PathBuf {
        let unique = format!(
            "{}_{}_{}",
            name,
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        );
        std::env::temp_dir().join(unique)
    }

    fn make_test_core(root: &Path, runner: &str, with_auxiliary_llm: bool) -> Arc<HoneBotCore> {
        std::fs::create_dir_all(root).expect("create temp root");
        let mut config = HoneConfig::default();
        config.agent.runner = runner.to_string();
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
        if with_auxiliary_llm {
            core.auxiliary_llm = Some(Arc::new(MockLlmProvider));
        }
        Arc::new(core)
    }

    fn make_request(
        actor: ActorIdentity,
        mode: ExecutionMode,
        runner_selection: ExecutionRunnerSelection,
    ) -> ExecutionRequest {
        ExecutionRequest {
            mode,
            session_id: "session-1".to_string(),
            actor,
            channel_target: "target".to_string(),
            allow_cron: false,
            system_prompt: "system".to_string(),
            runtime_input: "runtime".to_string(),
            context: AgentContext::new("session-1".to_string()),
            timeout: None,
            gemini_stream: GeminiStreamOptions::default(),
            session_metadata: HashMap::new(),
            model_override: None,
            runner_selection,
            allowed_tools: None,
            max_tool_calls: None,
            prompt_audit: None,
        }
    }

    #[test]
    fn prepare_uses_user_id_for_persistent_actor_label() {
        let root = temp_root("execution_persistent_actor_label");
        let core = make_test_core(&root, "codex_cli", false);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let prepared = ExecutionService::new(core)
            .prepare(make_request(
                actor.clone(),
                ExecutionMode::PersistentConversation,
                ExecutionRunnerSelection::Configured,
            ))
            .expect("prepare should succeed");

        assert_eq!(prepared.runner_request.actor_label, actor.user_id);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_uses_session_id_for_transient_actor_label() {
        let root = temp_root("execution_transient_actor_label");
        let core = make_test_core(&root, "codex_cli", false);
        let actor =
            ActorIdentity::new("discord", "alice", Some("room-1".to_string())).expect("actor");
        let prepared = ExecutionService::new(core)
            .prepare(make_request(
                actor.clone(),
                ExecutionMode::TransientTask,
                ExecutionRunnerSelection::Configured,
            ))
            .expect("prepare should succeed");

        assert_eq!(prepared.runner_request.actor_label, actor.session_id());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_can_force_auxiliary_function_calling_runner() {
        let root = temp_root("execution_aux_runner");
        let core = make_test_core(&root, "codex_cli", true);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let prepared = ExecutionService::new(core)
            .prepare(make_request(
                actor,
                ExecutionMode::TransientTask,
                ExecutionRunnerSelection::AuxiliaryFunctionCalling { max_iterations: 6 },
            ))
            .expect("prepare should succeed");

        assert_eq!(prepared.runner_name, "function_calling");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_requires_auxiliary_llm_for_auxiliary_function_calling_runner() {
        let root = temp_root("execution_aux_runner_missing_llm");
        let core = make_test_core(&root, "codex_cli", false);
        let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
        let err = match ExecutionService::new(core).prepare(make_request(
            actor,
            ExecutionMode::TransientTask,
            ExecutionRunnerSelection::AuxiliaryFunctionCalling { max_iterations: 6 },
        )) {
            Ok(_) => panic!("prepare should fail without auxiliary llm"),
            Err(err) => err,
        };

        assert!(err.contains("auxiliary llm unavailable"));
        let _ = std::fs::remove_dir_all(root);
    }
}
