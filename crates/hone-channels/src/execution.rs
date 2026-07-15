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
use crate::runners::{AgentRunner, AgentRunnerRequest};
use crate::sandbox::ensure_actor_sandbox;

fn absolute_runtime_path(path: &str) -> String {
    let candidate = std::path::PathBuf::from(path);
    if candidate.is_absolute() {
        return candidate.to_string_lossy().to_string();
    }
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(candidate)
        .to_string_lossy()
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionMode {
    PersistentConversation,
    TransientTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExecutionRunnerSelection {
    Configured,
}

#[derive(Clone)]
pub(crate) struct ExecutionRequest {
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
    pub tool_call_limits: Option<HashMap<String, u32>>,
    pub prompt_audit: Option<PromptAuditMetadata>,
}

pub(crate) struct PreparedExecution {
    pub runner_name: &'static str,
    pub runner: Box<dyn AgentRunner>,
    pub runner_request: AgentRunnerRequest,
}

pub(crate) struct ExecutionService {
    core: Arc<HoneBotCore>,
}

impl ExecutionService {
    pub(crate) fn new(core: Arc<HoneBotCore>) -> Self {
        Self { core }
    }

    pub(crate) fn prepare(
        &self,
        mut request: ExecutionRequest,
    ) -> Result<PreparedExecution, String> {
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
        let use_strict_fallback = matches!(
            request.runner_selection,
            ExecutionRunnerSelection::Configured
        ) && !self.core.is_admin_actor(&request.actor)
            && self.core.configured_runner_requires_trusted_host_access();
        let runner: Box<dyn AgentRunner> = match request.runner_selection {
            ExecutionRunnerSelection::Configured if use_strict_fallback => {
                tracing::warn!(
                    channel = %request.actor.channel,
                    user_id = %request.actor.user_id,
                    configured_runner = %self.core.config.agent.runner,
                    "untrusted actor routed to strict function-calling runner"
                );
                self.core
                    .create_strict_actor_runner(&request.system_prompt, tool_registry)?
            }
            ExecutionRunnerSelection::Configured => self.core.create_runner_with_model_override(
                &request.system_prompt,
                tool_registry,
                request.model_override.as_deref(),
            )?,
        };
        if use_strict_fallback {
            let removed = sanitize_function_calling_context(&mut request.context);
            if removed > 0 {
                tracing::info!(
                    channel = %request.actor.channel,
                    user_id = %request.actor.user_id,
                    removed_messages = removed,
                    "removed incompatible historical tool protocol before strict fallback"
                );
            }
        }
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
                config_path: absolute_runtime_path(&runtime_config_path()),
                runtime_dir: absolute_runtime_path(
                    &self.core.configured_runtime_dir().to_string_lossy(),
                ),
                system_prompt: request.system_prompt,
                runtime_input: request.runtime_input,
                context: request.context,
                timeout: request.timeout,
                gemini_stream: request.gemini_stream,
                session_metadata: request.session_metadata,
                working_directory,
                allowed_tools: request.allowed_tools,
                max_tool_calls: request.max_tool_calls,
                tool_call_limits: request.tool_call_limits,
            },
        })
    }
}

fn sanitize_function_calling_context(context: &mut AgentContext) -> usize {
    let original_len = context.messages.len();
    let mut sanitized = Vec::with_capacity(original_len);
    let mut index = 0usize;

    while index < context.messages.len() {
        let mut message = context.messages[index].clone();
        if message.role == "tool" {
            index += 1;
            continue;
        }

        let Some(tool_calls) = message
            .tool_calls
            .as_ref()
            .filter(|tool_calls| !tool_calls.is_empty())
        else {
            sanitized.push(message);
            index += 1;
            continue;
        };

        let expected_ids = tool_calls
            .iter()
            .filter_map(|call| call.get("id").and_then(Value::as_str))
            .map(str::to_string)
            .collect::<std::collections::HashSet<_>>();
        let mut following_tools = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();
        let mut cursor = index + 1;
        let mut valid = expected_ids.len() == tool_calls.len();

        while cursor < context.messages.len() && context.messages[cursor].role == "tool" {
            let tool_message = context.messages[cursor].clone();
            let Some(tool_call_id) = tool_message
                .tool_call_id
                .as_deref()
                .filter(|id| expected_ids.contains(*id))
            else {
                valid = false;
                cursor += 1;
                continue;
            };
            if !seen_ids.insert(tool_call_id.to_string()) {
                valid = false;
            }
            following_tools.push(tool_message);
            cursor += 1;
        }

        valid &= seen_ids == expected_ids;
        if valid {
            sanitized.push(message);
            sanitized.extend(following_tools);
        } else {
            message.tool_calls = None;
            if message
                .content
                .as_deref()
                .is_some_and(|content| !content.trim().is_empty())
            {
                sanitized.push(message);
            }
        }
        index = cursor;
    }

    let removed = original_len.saturating_sub(sanitized.len());
    context.messages = sanitized;
    removed
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

    fn make_test_core(root: &Path, runner: &str) -> Arc<HoneBotCore> {
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
        config.storage.gen_images_dir = root.join("gen_images").to_string_lossy().to_string();

        Arc::new(HoneBotCore::new(config))
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
            tool_call_limits: None,
            prompt_audit: None,
        }
    }

    #[test]
    fn prepare_uses_user_id_for_persistent_actor_label() {
        let root = temp_root("execution_persistent_actor_label");
        let core = make_test_core(&root, "codex_cli");
        let actor = ActorIdentity::new("cli", "alice", None::<String>).expect("actor");
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
        let core = make_test_core(&root, "codex_cli");
        let actor = ActorIdentity::new("cli", "alice", Some("room-1".to_string())).expect("actor");
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
    fn prepare_absolutizes_relative_runtime_paths() {
        let root = temp_root("execution_absolute_runtime_config");
        let core = make_test_core(&root, "codex_cli");
        let actor = ActorIdentity::new("cli", "alice", None::<String>).expect("actor");
        let previous = std::env::var_os("HONE_CONFIG_PATH");
        unsafe {
            std::env::set_var("HONE_CONFIG_PATH", "config.yaml");
        }

        let prepared = ExecutionService::new(core)
            .prepare(make_request(
                actor,
                ExecutionMode::PersistentConversation,
                ExecutionRunnerSelection::Configured,
            ))
            .expect("prepare should succeed");

        assert!(std::path::Path::new(&prepared.runner_request.config_path).is_absolute());
        assert!(std::path::Path::new(&prepared.runner_request.runtime_dir).is_absolute());

        match previous {
            Some(value) => unsafe { std::env::set_var("HONE_CONFIG_PATH", value) },
            None => unsafe { std::env::remove_var("HONE_CONFIG_PATH") },
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_fails_closed_when_strict_fallback_llm_is_missing() {
        let root = temp_root("execution_non_admin_native_runner");
        let core = make_test_core(&root, "codex_acp");
        let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");

        let err = match ExecutionService::new(core).prepare(make_request(
            actor,
            ExecutionMode::PersistentConversation,
            ExecutionRunnerSelection::Configured,
        )) {
            Ok(_) => panic!("non-admin native runner must fail closed"),
            Err(err) => err,
        };

        assert!(err.contains("普通用户不能使用"));
        assert!(err.contains("严格 function-calling LLM 未配置"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_routes_non_admin_native_runner_to_strict_function_calling() {
        let root = temp_root("execution_non_admin_strict_fallback");
        let core = make_test_core(&root, "codex_acp");
        let mut core = match Arc::try_unwrap(core) {
            Ok(core) => core,
            Err(_) => panic!("test core should have a unique owner"),
        };
        core.llm = Some(Arc::new(MockLlmProvider));
        let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");

        let prepared = ExecutionService::new(Arc::new(core))
            .prepare(make_request(
                actor,
                ExecutionMode::PersistentConversation,
                ExecutionRunnerSelection::Configured,
            ))
            .expect("non-admin should use strict fallback");

        assert_eq!(prepared.runner_name, "function_calling");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_ignores_repo_internal_sandbox_override() {
        let _guard = crate::sandbox::sandbox_env_test_lock()
            .lock()
            .expect("env lock");
        let root = temp_root("execution_repo_internal_sandbox_override");
        let repo_internal =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/agent-sandboxes-test");
        unsafe {
            std::env::set_var("HONE_AGENT_SANDBOX_DIR", &repo_internal);
        }
        let core = make_test_core(&root, "hone_cloud");
        let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");
        let prepared = ExecutionService::new(core)
            .prepare(make_request(
                actor,
                ExecutionMode::PersistentConversation,
                ExecutionRunnerSelection::Configured,
            ))
            .expect("prepare should succeed");

        assert!(
            prepared
                .runner_request
                .working_directory
                .contains("hone-agent-sandboxes")
        );
        assert!(
            !prepared
                .runner_request
                .working_directory
                .contains("/data/agent-sandboxes-test")
        );

        unsafe {
            std::env::remove_var("HONE_AGENT_SANDBOX_DIR");
        }
        let _ = std::fs::remove_dir_all(root);
    }
}
