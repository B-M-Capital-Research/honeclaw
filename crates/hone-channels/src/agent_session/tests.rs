//! Agent session 的回归测试。
//!
//! 这里覆盖了五大类场景:
//! - 纯 helper 断言(`should_persist_tool_result` / `persistable_turn_from_response` / …);
//! - `restore_context` 对 session 历史的还原、过滤、脱敏、metadata 保留;
//! - `AgentSession::run` 完整流程(配额、manual compact、context overflow 恢复、
//!   scheduled-task bypass);
//! - `SessionEventEmitter` 的路径脱敏;
//! - Gemini CLI runner 的 stream 解析(冒烟)。

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use hone_core::ActorIdentity;
use hone_core::SessionIdentity;
use hone_core::agent::{AgentContext, AgentMessage, AgentResponse, ToolCallMade};
use hone_core::config::HoneConfig;
use hone_llm::provider::ChatResult;
use hone_llm::{ChatResponse, LlmProvider, Message};
use hone_memory::session::{SessionRuntimeBackend, SessionStorageOptions};
use hone_memory::{
    ConversationQuotaReserveResult, SessionStorage, assistant_tool_calls_from_metadata,
    build_assistant_message_metadata, build_tool_message_metadata_parts,
    session_message_from_normalized, session_message_text,
};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crate::HoneBotCore;
use crate::response_finalizer::{
    EMPTY_SUCCESS_FALLBACK_MESSAGE, finalize_agent_response, normalize_local_image_references,
    response_leaks_system_prompt,
};
use crate::run_event::RunEvent;
use crate::runners::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    stream_gemini_prompt,
};
use crate::runtime::sanitize_user_visible_output;
use crate::sandbox::sandbox_base_dir;

use super::core::AgentSession;
use super::emitter::SessionEventEmitter;
use super::helpers::{
    CONTEXT_OVERFLOW_FALLBACK_MESSAGE, DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT,
    persistable_turn_from_response, sanitize_assistant_context_content, should_persist_tool_result,
    should_return_runner_result,
};
use super::restore::restore_context;
use super::types::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionErrorKind, AgentSessionEvent,
    AgentSessionListener, GeminiStreamOptions,
};

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
impl AgentRunner for MockEmptySuccessRunner {
    fn name(&self) -> &'static str {
        "mock_empty_success"
    }

    async fn run(
        &self,
        _request: AgentRunnerRequest,
        _emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        AgentRunnerResult {
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
    config.storage.llm_audit_db_path = root.join("llm_audit.sqlite3").to_string_lossy().to_string();
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

#[test]
fn sanitize_user_visible_output_whitespace_only_success_needs_fallback() {
    let sanitized = sanitize_user_visible_output("   ");
    assert!(sanitized.content.is_empty());
    assert!(!sanitized.only_internal);
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
    let request = AgentRunnerRequest {
        session_id: "empty-success-session".to_string(),
        actor_label: "discord:empty-success".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
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

    assert!(!result.response.success);
    assert_eq!(result.response.content, EMPTY_SUCCESS_FALLBACK_MESSAGE);
    assert_eq!(
        result.response.error.as_deref(),
        Some(EMPTY_SUCCESS_FALLBACK_MESSAGE)
    );
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
    let group = AgentSession::new(core, group_actor, "room-1").with_session_identity(group_session);
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
fn resolve_prompt_input_hides_cron_only_skills_when_cron_is_not_allowed() {
    let root = make_temp_dir("hone_channels_prompt_stage_skill_visibility");
    let system_skills = root.join("system_skills");
    let scheduled_dir = system_skills.join("scheduled_task");
    let stock_dir = system_skills.join("stock_alpha");
    std::fs::create_dir_all(&scheduled_dir).expect("create scheduled dir");
    std::fs::create_dir_all(&stock_dir).expect("create stock dir");
    std::fs::write(
        scheduled_dir.join("SKILL.md"),
        concat!(
            "---\n",
            "name: Scheduled Task\n",
            "description: cron workflow\n",
            "allowed-tools:\n",
            "  - cron_job\n",
            "---\n\n",
            "body\n"
        ),
    )
    .expect("write scheduled skill");
    std::fs::write(
        stock_dir.join("SKILL.md"),
        concat!(
            "---\n",
            "name: Stock Alpha\n",
            "description: stock workflow\n",
            "allowed-tools:\n",
            "  - data_fetch\n",
            "---\n\n",
            "body\n"
        ),
    )
    .expect("write stock skill");

    let llm = MockLlmProvider::with_tool_responses(Vec::new());
    let core = make_test_core_with_config(&root, llm, |config| {
        config.extra.insert(
            "skills_dir".to_string(),
            serde_yaml::Value::String(system_skills.to_string_lossy().to_string()),
        );
    });
    let actor = ActorIdentity::new("telegram", "alice", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "target").with_cron_allowed(false);

    let (_, runtime_input) = session.resolve_prompt_input("session-demo", "set a scheduled task");

    assert!(!runtime_input.contains("scheduled_task"));
    assert!(!runtime_input.contains("Scheduled Task"));
    assert!(!runtime_input.contains("cron workflow"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn resolve_prompt_input_places_recv_extra_before_compact_summary() {
    let root = make_temp_dir("hone_channels_prompt_recv_extra_priority");
    let storage = SessionStorage::new(root.join("sessions"));
    let actor = ActorIdentity::new("discord", "alice", Some("room-1".to_string())).expect("actor");
    let session_identity =
        SessionIdentity::group(&actor.channel, actor.channel_scope.clone().unwrap())
            .expect("group session");
    let session_id = storage
        .create_session(
            Some("session-demo"),
            Some(actor.clone()),
            Some(session_identity),
        )
        .expect("create session");
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
            "system",
            "【Compact Summary】\nsummary",
            Some(hone_memory::build_compact_summary_metadata("auto")),
        )
        .expect("add summary");

    let llm = MockLlmProvider::with_tool_responses(Vec::new());
    let core = make_test_core(&root, llm);
    let session = AgentSession::new(core, actor, "target")
        .with_session_id("session-demo")
        .with_recv_extra(Some(
            "【群聊同发言者最近往返候选】\nrecent exchange".to_string(),
        ));

    let (_, runtime_input) = session.resolve_prompt_input("session-demo", "请继续");
    let extra_pos = runtime_input
        .find("【群聊同发言者最近往返候选】")
        .expect("recv extra present");
    let summary_pos = runtime_input
        .find("【Compact Summary】")
        .expect("summary present");
    assert!(extra_pos < summary_pos);

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
fn finalize_agent_response_marks_sanitized_empty_success_as_failure() {
    let root = make_temp_dir("hone_channels_finalize_sanitized_empty");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "   ".to_string(),
        tool_calls_made: Vec::new(),
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(!response.success);
    assert_eq!(response.content, EMPTY_SUCCESS_FALLBACK_MESSAGE);
    assert_eq!(
        response.error.as_deref(),
        Some(EMPTY_SUCCESS_FALLBACK_MESSAGE)
    );
    assert_eq!(outcome.fallback_reason, Some("sanitized_empty_success"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finalize_agent_response_marks_planning_sentence_as_failure() {
    let root = make_temp_dir("hone_channels_finalize_planning_sentence");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "我先查一下你现有的定时任务。".to_string(),
        tool_calls_made: Vec::new(),
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(!response.success);
    assert_eq!(response.content, EMPTY_SUCCESS_FALLBACK_MESSAGE);
    assert_eq!(
        response.error.as_deref(),
        Some(EMPTY_SUCCESS_FALLBACK_MESSAGE)
    );
    assert_eq!(
        outcome.fallback_reason,
        Some("planning_sentence_suppressed")
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn transitional_clarification_question_is_not_treated_as_planning_sentence() {
    assert!(!crate::runtime::is_transitional_planning_sentence(
        "请先确认具体是哪只股票/资产的 ticker？确认标的后我再校验当前价格、财报、估值倍数和同业，再判断估值是否合理。"
    ));
}

#[test]
fn finalize_agent_response_keeps_user_facing_clarification_question() {
    let root = make_temp_dir("hone_channels_finalize_clarification_question");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let clarification = "请先确认具体是哪只股票/资产的 ticker？确认标的后我再校验当前价格、财报、估值倍数和同业，再判断估值是否合理。";
    let mut response = AgentResponse {
        content: clarification.to_string(),
        tool_calls_made: Vec::new(),
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert_eq!(response.content, clarification);
    assert_eq!(response.error, None);
    assert_eq!(outcome.fallback_reason, None);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn compose_invoked_skill_runtime_input_keeps_user_supplement_outside_skill_context() {
    let runtime_input = crate::turn_builder::compose_invoked_skill_runtime_input(
        "SKILL_PROMPT",
        Some("finish the task"),
    );
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

#[cfg(unix)]
#[tokio::test]
async fn normalize_local_image_references_moves_sandbox_images_into_gen_images() {
    let root = make_temp_dir("hone_channels_local_image_normalize");
    std::fs::create_dir_all(&root).expect("create root");
    let data_dir = root.join("data");
    std::fs::create_dir_all(&data_dir).expect("create data dir");

    with_temp_env_var("HONE_DATA_DIR", data_dir.as_os_str(), || async {
        let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
        let sandbox_image = sandbox_base_dir()
            .join("telegram")
            .join("chat_3a-test__probe")
            .join("artifacts")
            .join("chart.png");
        std::fs::create_dir_all(sandbox_image.parent().expect("sandbox parent"))
            .expect("create sandbox artifacts dir");
        std::fs::write(&sandbox_image, b"png-bytes").expect("write sandbox image");

        let content = format!(
            "前文<a href=\"file://{}\">查看图片</a>后文",
            sandbox_image.display()
        );
        let normalized = normalize_local_image_references(
            &core,
            "Session_telegram__group__chat_3a-test",
            &content,
        );

        assert!(!normalized.contains("<a href="));
        assert!(normalized.starts_with("前文file://"));
        assert!(normalized.ends_with("后文"));

        let copied_path = normalized
            .strip_prefix("前文file://")
            .and_then(|value| value.strip_suffix("后文"))
            .expect("normalized marker");
        assert!(copied_path.starts_with(&core.config.storage.gen_images_dir));
        assert!(std::path::Path::new(copied_path).exists());
    })
    .await;

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn normalize_local_image_references_replaces_missing_images_with_fallback_note() {
    let root = make_temp_dir("hone_channels_local_image_missing");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let missing = root.join("missing").join("chart.png");
    let content = format!("前文\nfile://{}\n后文", missing.display());

    let normalized = normalize_local_image_references(&core, "Session_telegram__missing", &content);

    assert_eq!(normalized, "前文\n（图表文件不可用，请重新生成）\n后文");

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn sanitize_assistant_context_content_redacts_local_image_markers() {
    let sanitized = sanitize_assistant_context_content(
        "前文<a href=\"file:///tmp/chart.png\">查看图片</a>后文",
    );

    assert_eq!(sanitized, "前文（上文包含图表）后文");
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
            "system",
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
    // compact_summary is skipped from message history; summary is injected via conversation_context
    assert_eq!(contents, vec!["after-compact"]);
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
            "system",
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
    // compact_summary is excluded from message history; injected via conversation_context
    assert_eq!(contents, vec!["INVOKED_SKILL_PROMPT"]);
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
            "system",
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
    // compact_summary skipped from history; skill_snapshot remains; no duplicate from metadata
    assert_eq!(contents, vec!["INVOKED_SKILL_PROMPT"]);
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
async fn run_rejects_over_daily_limit_with_user_turn_and_friendly_error() {
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
    let daily_limit = core.config.agent.daily_conversation_limit;

    for _ in 0..daily_limit {
        let reservation = match core
            .conversation_quota_storage
            .try_reserve_daily_conversation(&actor, daily_limit, false)
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
    let error = result.response.error.unwrap_or_default();
    assert!(error.contains("已达到今日对话上限"));
    assert!(
        !error.contains("工具执行错误"),
        "quota rejection should stay user-facing, got: {error}"
    );
    assert_eq!(llm.chat_with_tools_calls(), 0);
    let messages = core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .expect("messages");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content[0].text.as_deref(), Some("hello"));
    let snapshot = core
        .conversation_quota_storage
        .snapshot_for_date(&actor, &today)
        .expect("snapshot")
        .expect("row");
    assert_eq!(snapshot.success_count, daily_limit);
    assert_eq!(snapshot.in_flight, 0);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn run_zero_daily_conversation_limit_bypasses_quota() {
    let root = make_temp_dir("hone_channels_quota_unlimited");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_tool_responses(
        (0..15)
            .map(|_| ChatResponse {
                content: "ok".to_string(),
                tool_calls: None,
                usage: None,
            })
            .collect(),
    );
    let core = make_test_core_with_config(&root, llm, |config| {
        config.agent.daily_conversation_limit = 0;
    });
    let actor = ActorIdentity::new("discord", "alice", None::<String>).expect("actor");
    let session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());

    for idx in 0..15 {
        let result = session
            .run(&format!("hello-{idx}"), AgentRunOptions::default())
            .await;
        assert!(result.response.success, "{:?}", result.response.error);
    }

    let today = hone_core::beijing_now().format("%F").to_string();
    let snapshot = core
        .conversation_quota_storage
        .snapshot_for_date(&actor, &today)
        .expect("snapshot");
    assert!(snapshot.is_none());
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
    let actor = ActorIdentity::new("discord", "gina", Some("room-1".to_string())).expect("actor");
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
    let actor = ActorIdentity::new("discord", "henry", Some("room-2".to_string())).expect("actor");
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
    let daily_limit = core.config.agent.daily_conversation_limit;

    for _ in 0..daily_limit {
        let reservation = match core
            .conversation_quota_storage
            .try_reserve_daily_conversation(&actor, daily_limit, false)
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
    assert_eq!(snapshot.success_count, daily_limit);
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
        AgentSessionEvent::Run(RunEvent::Progress {
            detail: Some(detail),
            ..
        }) if detail
            == "Edit company_profiles/sandisk/profile.md and <absolute-path>/private.txt"
    ));
    assert!(matches!(
        &events[1],
        AgentSessionEvent::Run(RunEvent::ToolStatus {
            tool,
            message: Some(message),
            reasoning: Some(reasoning),
            ..
        }) if tool == "hone/skill_tool"
            && message == "Edit company_profiles/micron-technology/profile.md"
            && reasoning == "Edit data/research/notes.md and <absolute-path>/passwd"
    ));
}

#[tokio::test]
async fn session_event_emitter_suppresses_internal_tool_status_payloads() {
    let root = "/Users/fengming2/Desktop/honeclaw";
    let listener = Arc::new(RecordingListener::default());
    let emitter = SessionEventEmitter {
        listeners: vec![listener.clone()],
        channel: "web".to_string(),
        user_id: "web-user".to_string(),
        session_id: "session".to_string(),
        message_id: None,
        working_directory: root.to_string(),
    };

    emitter
        .emit(AgentRunnerEvent::ToolStatus {
            tool: format!("{root}/skills/scheduled_task"),
            status: "start".to_string(),
            message: Some(
                "【Invoked Skill Context】\nBase directory for this skill: /Users/fengming2/Desktop/honeclaw/skills/scheduled_task".to_string(),
            ),
            reasoning: Some(
                r#"{"job":{"channel_target":"web","task_prompt":"每天提醒我复盘"}} "#.trim().to_string(),
            ),
        })
        .await;

    let events = listener.events.lock().await.clone();
    assert!(matches!(
        &events[0],
        AgentSessionEvent::Run(RunEvent::ToolStatus {
            tool,
            status,
            message: None,
            reasoning: None,
        }) if tool == "skills/scheduled_task" && status == "start"
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
