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
use hone_llm::provider::{ChatResult, FunctionCall, ToolCall};
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
use crate::investment_response_guard::{
    DeepAnalysisKind, InvestmentResponseContract, ResolvedSecurityEntity,
    build_agent_discovered_investment, prepare_verified_investment_turn,
};
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

use super::core::{AgentSession, PreparedTurnReexecutionPolicy, prepared_turn_reexecution_policy};
use super::emitter::SessionEventEmitter;
use super::helpers::{
    CONTEXT_OVERFLOW_FALLBACK_MESSAGE, DIRECT_SESSION_PRE_COMPACT_RESTORE_LIMIT,
    NON_FINANCE_BOUNDARY_REPLY, is_retryable_transient_runner_error_text,
    non_finance_boundary_reply, persistable_turn_from_response, prune_interactive_runtime_history,
    sanitize_assistant_context_content, should_persist_tool_result, should_return_runner_result,
};
use super::restore::restore_context;
use super::types::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionErrorKind, AgentSessionEvent,
    AgentSessionListener, AgentTurnOrigin, GeminiStreamOptions,
};

fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()))
}

struct NoopEmitter;

#[async_trait]
impl AgentRunnerEmitter for NoopEmitter {
    async fn emit(&self, _event: AgentRunnerEvent) {}
}

#[derive(Default)]
struct RecordingRunnerEmitter {
    events: tokio::sync::Mutex<Vec<AgentRunnerEvent>>,
}

#[async_trait]
impl AgentRunnerEmitter for RecordingRunnerEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        self.events.lock().await.push(event);
    }
}

#[derive(Clone)]
struct MockEmptySuccessRunner {
    response: AgentResponse,
}

#[derive(Clone)]
struct MockLlmRunner {
    llm: Arc<dyn LlmProvider>,
}

#[async_trait]
impl AgentRunner for MockLlmRunner {
    fn name(&self) -> &'static str {
        "mock_llm"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        _emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let messages = vec![Message {
            role: "user".to_string(),
            content: Some(request.runtime_input),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }];
        let response = match self.llm.chat_with_tools(&messages, &[], None).await {
            Ok(reply) => AgentResponse {
                content: reply.content,
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            Err(error) => AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: false,
                error: Some(error.to_string()),
            },
        };
        AgentRunnerResult {
            response,
            streamed_output: false,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        }
    }
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
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        }
    }
}

#[derive(Clone)]
struct MockSequencedRunner {
    results: Arc<Mutex<std::collections::VecDeque<AgentRunnerResult>>>,
}

struct MockStreamingRun {
    events: Vec<AgentRunnerEvent>,
    result: AgentRunnerResult,
}

#[derive(Clone)]
struct MockStreamingSequencedRunner {
    runs: Arc<Mutex<std::collections::VecDeque<MockStreamingRun>>>,
    runtime_inputs: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl AgentRunner for MockStreamingSequencedRunner {
    fn name(&self) -> &'static str {
        "mock_streaming_sequenced"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        self.runtime_inputs
            .lock()
            .expect("lock runtime inputs")
            .push(request.runtime_input);
        let run = self
            .runs
            .lock()
            .expect("lock streaming runs")
            .pop_front()
            .expect("queued streaming run");
        for event in run.events {
            emitter.emit(event).await;
        }
        run.result
    }
}

#[async_trait]
impl AgentRunner for MockSequencedRunner {
    fn name(&self) -> &'static str {
        "mock_sequenced"
    }

    async fn run(
        &self,
        _request: AgentRunnerRequest,
        _emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        self.results
            .lock()
            .expect("lock results")
            .pop_front()
            .expect("queued runner result")
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
    last_tool_messages: Option<Vec<Message>>,
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
                last_tool_messages: None,
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
                last_tool_messages: None,
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
                last_tool_messages: None,
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

    fn last_tool_transcript(&self) -> String {
        self.state
            .lock()
            .expect("mock llm lock")
            .last_tool_messages
            .as_ref()
            .into_iter()
            .flatten()
            .filter_map(|message| message.content.as_deref())
            .collect::<Vec<_>>()
            .join("\n")
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
        state.last_tool_messages = Some(_messages.to_vec());
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
    config.agent.runner = "hone_cloud".to_string();
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
    core.auxiliary_llm = Some(shared_llm.clone());
    let runner_llm: Arc<dyn LlmProvider> = shared_llm;
    core.test_runner_factory = Some(Arc::new(move || {
        Box::new(MockLlmRunner {
            llm: runner_llm.clone(),
        })
    }));
    Arc::new(core)
}

fn make_strict_tool_loop_test_core_with_config(
    root: &std::path::Path,
    llm: MockLlmProvider,
    configure: impl FnOnce(&mut HoneConfig),
) -> Arc<HoneBotCore> {
    let mut config = HoneConfig::default();
    config.agent.runner = "codex_acp".to_string();
    config.fmp.timeout = 5;
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

fn spawn_fmp_route_stub(routes: Vec<(String, Value)>) -> (String, std::thread::JoinHandle<()>) {
    use std::collections::HashSet;
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind FMP stub");
    let address = listener.local_addr().expect("FMP stub address");
    listener
        .set_nonblocking(true)
        .expect("set FMP stub nonblocking");
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(0);
    let handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build FMP stub runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener)
                .expect("create async FMP stub listener");
            ready_tx.send(()).expect("signal FMP stub ready");
            let routes = Arc::new(routes);
            let served_routes = Arc::new(Mutex::new(HashSet::new()));
            let served_count = Arc::new(AtomicUsize::new(0));
            let mut handlers = tokio::task::JoinSet::new();
            let deadline = Instant::now() + Duration::from_secs(15);

            while served_count.load(Ordering::SeqCst) < routes.len() && Instant::now() < deadline {
                let Ok(accepted) =
                    tokio::time::timeout(Duration::from_millis(100), listener.accept()).await
                else {
                    continue;
                };
                let (mut stream, _) = accepted.expect("accept FMP stub request");
                let routes = routes.clone();
                let served_routes = served_routes.clone();
                let served_count = served_count.clone();
                handlers.spawn(async move {
                    let mut request = Vec::new();
                    loop {
                        let mut chunk = [0_u8; 8192];
                        let Ok(read_result) = tokio::time::timeout(
                            Duration::from_secs(5),
                            stream.read(&mut chunk),
                        )
                        .await
                        else {
                            return;
                        };
                        match read_result.expect("read FMP stub request") {
                            0 if request.is_empty() => return,
                            0 => break,
                            bytes => {
                                request.extend_from_slice(&chunk[..bytes]);
                                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                                    break;
                                }
                            }
                        }
                    }
                    let request = String::from_utf8_lossy(&request);
                    let (route_index, value) = routes
                        .iter()
                        .enumerate()
                        .find(|(_, (needle, _))| request.contains(needle))
                        .map(|(index, (_, value))| (index, value))
                        .unwrap_or_else(|| panic!("unmatched FMP stub request: {request}"));
                    let body = value.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .await
                        .expect("write FMP stub response");
                    stream.shutdown().await.expect("shutdown FMP stub response");
                    let mut served_routes = served_routes.lock().expect("lock served routes");
                    if served_routes.insert(route_index) {
                        served_count.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }

            while let Some(handler) = handlers.join_next().await {
                handler.expect("join FMP stub handler");
            }
            let served_routes = served_routes.lock().expect("lock final served routes");
            let missing_routes = routes
                .iter()
                .enumerate()
                .filter(|(index, _)| !served_routes.contains(index))
                .map(|(_, (needle, _))| needle.as_str())
                .collect::<Vec<_>>();
            assert!(
                missing_routes.is_empty(),
                "FMP stub did not receive configured routes: {missing_routes:?}"
            );
        });
    });
    ready_rx.recv().expect("wait for FMP stub ready");
    (format!("http://{address}/api"), handle)
}

#[cfg(unix)]
fn write_mock_gemini_script(lines: &[&str]) -> (std::path::PathBuf, std::path::PathBuf) {
    write_mock_gemini_script_with_stderr(lines, "", 0)
}

#[cfg(unix)]
fn write_mock_gemini_script_with_stderr(
    lines: &[&str],
    stderr: &str,
    exit_code: i32,
) -> (std::path::PathBuf, std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;

    let root = make_temp_dir("hone_gemini_mock");
    let data_path = root.join("stream.txt");
    let stderr_path = root.join("stderr.txt");
    let content = lines.join("\n");
    std::fs::create_dir_all(&root).expect("create mock root");
    std::fs::write(&data_path, content).expect("write mock data");
    std::fs::write(&stderr_path, stderr).expect("write mock stderr");

    let script_path = root.join("gemini-mock.sh");
    let script = format!(
        "#!/bin/sh\ncat \"{}\"\ncat \"{}\" >&2\nexit {}\n",
        data_path.display(),
        stderr_path.display(),
        exit_code
    );
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
    let restored_context = restore_context(&storage, "missing", Some(5), None);
    assert!(restored_context.messages.is_empty());
    assert!(restored_context.actor_identity().is_none());
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
        committed_visible_prefix: None,
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
        committed_visible_prefix: None,
        terminal_error_emitted: false,
        session_metadata_updates: HashMap::new(),
        context_messages: None,
    };

    assert!(!should_return_runner_result(&result));
}

#[test]
fn retryable_transient_runner_error_text_matches_acp_disconnect_and_idle_timeout() {
    assert!(is_retryable_transient_runner_error_text(
        "codex acp error: stream disconnected before completion"
    ));
    assert!(is_retryable_transient_runner_error_text(
        "opencode acp session/prompt idle timeout (180s)"
    ));
    assert!(!is_retryable_transient_runner_error_text(
        "context window exceeds limit (2013)"
    ));
    assert!(!is_retryable_transient_runner_error_text(
        "request timed out while waiting for upstream response"
    ));
}

#[test]
fn non_finance_boundary_rejects_obvious_consumer_topics_without_finance_anchor() {
    assert_eq!(
        non_finance_boundary_reply("Hi hone，你了解深圳楼市吗？我现在是否适合买房？"),
        Some(NON_FINANCE_BOUNDARY_REPLY)
    );
    assert_eq!(
        non_finance_boundary_reply("AMD的电脑CPU是什么名字"),
        Some(NON_FINANCE_BOUNDARY_REPLY)
    );
}

#[test]
fn non_finance_boundary_allows_finance_framed_adjacent_topics() {
    assert_eq!(
        non_finance_boundary_reply("深圳楼市会影响哪些地产股？"),
        None
    );
    assert_eq!(
        non_finance_boundary_reply("AMD CPU业务对股价和财报有什么影响？"),
        None
    );
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
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_empty_success_retry(
            &runner,
            "mock_empty_success",
            "empty-success-session",
            request,
            Arc::new(NoopEmitter),
            PreparedTurnReexecutionPolicy::Allowed,
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

#[tokio::test]
async fn transient_runner_failure_retries_once_before_returning_success() {
    let root = make_temp_dir("hone_channels_transient_runner_retry_success");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("discord", "transient-retry", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let runner = MockSequencedRunner {
        results: Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
            AgentRunnerResult {
                response: AgentResponse {
                    content: String::new(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: false,
                    error: Some("codex acp session/prompt idle timeout (180s)".to_string()),
                },
                streamed_output: true,
                committed_visible_prefix: None,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            },
            AgentRunnerResult {
                response: AgentResponse {
                    content: "重试后成功".to_string(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: true,
                    error: None,
                },
                streamed_output: true,
                committed_visible_prefix: None,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            },
        ]))),
    };
    let request = AgentRunnerRequest {
        session_id: "transient-retry-session".to_string(),
        actor_label: "discord:transient-retry".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "user input".to_string(),
        context: AgentContext::new("transient-retry-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_empty_success_retry(
            &runner,
            "mock_sequenced",
            "transient-retry-session",
            request,
            Arc::new(NoopEmitter),
            PreparedTurnReexecutionPolicy::Allowed,
        )
        .await;

    assert!(result.response.success);
    assert_eq!(result.response.content, "重试后成功");

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn committed_terminal_prefix_makes_runner_attempt_irreversible_and_suppresses_retry() {
    let root = make_temp_dir("hone_channels_committed_terminal_prefix_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "committed-no-retry", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let committed = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 3,
                success: false,
                error: Some("codex acp stream disconnected before completion".to_string()),
            },
            streamed_output: true,
            committed_visible_prefix: Some(committed.to_string()),
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "不应在已提交可见前缀后重跑".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "committed-no-retry-session".to_string(),
        actor_label: "web:committed-no-retry".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "CRWV 和 NVDA 有什么关系".to_string(),
        context: AgentContext::new("committed-no-retry-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_empty_success_retry(
            &runner,
            "mock_sequenced",
            "committed-no-retry-session",
            request,
            Arc::new(NoopEmitter),
            PreparedTurnReexecutionPolicy::Allowed,
        )
        .await;

    assert!(!result.response.success);
    assert_eq!(result.committed_visible_prefix.as_deref(), Some(committed));
    assert_eq!(results.lock().expect("results lock").len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn explicit_persistent_operations_are_execute_once_but_research_context_is_not() {
    for input in [
        "帮我关注 NVDA",
        "把 NBIS 放进自选",
        "我买了 NBIS 100股",
        "我不持有苹果了",
        "每天9点给我看 RMBS",
        "今晚22点给我 NBIS 播报",
        "NBIS 到 100 提醒我",
        "取消所有定时任务",
        "把 NVDA 从持仓删除",
        "重启 Hone 服务",
        "please add NBIS to my watchlist",
        "remove NVDA from my portfolio",
        "add RMBS to my watchlist and analyze it",
        "把 NBIS 加入自选然后分析",
        "把 NBIS 放进持仓然后分析",
        "我刚买入 NBIS 100 股并分析",
        "卖出 NBIS 后分析",
        "加仓 NBIS 后分析",
        "减仓 NBIS 后分析",
        "put NBIS in my portfolio and analyze it",
        "buy NBIS then analyze it",
        "sell NBIS then review it",
        "reduce my NBIS holding and analyze it",
        "启动 NBIS 深度研究",
        "请对 NBIS 做深度研究",
        "deep research NBIS",
        "start deep research on NBIS",
        "please run deep research on NBIS",
    ] {
        assert_eq!(
            prepared_turn_reexecution_policy(input),
            PreparedTurnReexecutionPolicy::ExecuteOnce,
            "{input}"
        );
    }
    for input in [
        "我关注 NVDA 的原因是什么",
        "我关注 NVDA，怎么看",
        "我不持有 NVDA，怎么看",
        "如何删除定时任务",
        "现在 RMBS 怎么看",
        "analyze my watchlist",
        "how do I add a stock to a watchlist",
        "NBIS 现在适合买入吗？",
        "我是否应该卖出 NBIS？",
        "NBIS 该不该加仓？",
        "Should I buy NBIS?",
        "Is NBIS a buy?",
        "Whether to sell NBIS is my question",
        "深度研究是什么？",
        "如何做深度研究？",
        "what is deep research?",
        "how to do deep research?",
    ] {
        assert_eq!(
            prepared_turn_reexecution_policy(input),
            PreparedTurnReexecutionPolicy::Allowed,
            "{input}"
        );
    }
}

#[tokio::test]
async fn observed_persistent_tool_trace_suppresses_transient_retry() {
    let root = make_temp_dir("hone_channels_persistent_trace_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor =
        ActorIdentity::new("discord", "persistent-no-retry", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: vec![ToolCallMade {
                    name: "mcp__hone__portfolio".to_string(),
                    arguments: serde_json::json!({"action":"watch", "ticker":"NBIS"}),
                    result: serde_json::json!({
                        "status":"unknown_after_acp_failure",
                        "isError":true
                    }),
                    tool_call_id: Some("call_watch".to_string()),
                }],
                iterations: 1,
                success: false,
                error: Some("codex acp stream disconnected before completion".to_string()),
            },
            streamed_output: false,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "不应执行到这里".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "persistent-no-retry-session".to_string(),
        actor_label: "discord:persistent-no-retry".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: true,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "帮我关注 NBIS".to_string(),
        context: AgentContext::new("persistent-no-retry-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_empty_success_retry(
            &runner,
            "mock_sequenced",
            "persistent-no-retry-session",
            request,
            Arc::new(NoopEmitter),
            PreparedTurnReexecutionPolicy::Allowed,
        )
        .await;

    assert!(!result.response.success);
    assert_eq!(result.response.tool_calls_made.len(), 1);
    assert_eq!(results.lock().expect("results lock").len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn unknown_tool_trace_suppresses_transient_retry() {
    let root = make_temp_dir("hone_channels_unknown_tool_trace_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "unknown-tool-no-retry", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: vec![ToolCallMade {
                    name: "mcp__filesystem__write_file".to_string(),
                    arguments: serde_json::json!({"path":"external-state"}),
                    result: serde_json::json!({"status":"unknown_after_acp_failure"}),
                    tool_call_id: Some("unknown_write".to_string()),
                }],
                iterations: 1,
                success: false,
                error: Some("codex acp stream disconnected before completion".to_string()),
            },
            streamed_output: false,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "不应重放未知外部工具".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "unknown-tool-no-retry-session".to_string(),
        actor_label: "web:unknown-tool-no-retry".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "现在 NBIS 怎么看".to_string(),
        context: AgentContext::new("unknown-tool-no-retry-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_empty_success_retry(
            &runner,
            "mock_sequenced",
            "unknown-tool-no-retry-session",
            request,
            Arc::new(NoopEmitter),
            PreparedTurnReexecutionPolicy::Allowed,
        )
        .await;

    assert!(!result.response.success);
    assert_eq!(result.response.tool_calls_made.len(), 1);
    assert_eq!(results.lock().expect("results lock").len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn execute_once_intent_suppresses_empty_success_retry_even_without_trace() {
    let root = make_temp_dir("hone_channels_execute_once_empty_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("discord", "execute-once-empty", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "不应执行到这里".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "execute-once-empty-session".to_string(),
        actor_label: "discord:execute-once-empty".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: true,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "每天9点给我看 RMBS".to_string(),
        context: AgentContext::new("execute-once-empty-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "execute-once-empty-session",
            request,
            Arc::new(NoopEmitter),
            None,
            PreparedTurnReexecutionPolicy::ExecuteOnce,
            None,
        )
        .await;

    assert!(!result.response.success);
    assert_eq!(
        result.response.error.as_deref(),
        Some(crate::tool_trace::PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE)
    );
    assert_eq!(results.lock().expect("results lock").len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn portfolio_mutation_then_analysis_disconnect_does_not_retry_without_trace() {
    let root = make_temp_dir("hone_channels_portfolio_mutation_disconnect_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor =
        ActorIdentity::new("web", "portfolio-mutation-disconnect", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: false,
                error: Some("codex acp stream disconnected before completion".to_string()),
            },
            streamed_output: false,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "不应重复执行持仓写入".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let input = "把 NBIS 放进持仓然后分析";
    let policy = prepared_turn_reexecution_policy(input);
    assert_eq!(policy, PreparedTurnReexecutionPolicy::ExecuteOnce);
    let request = AgentRunnerRequest {
        session_id: "portfolio-mutation-disconnect-session".to_string(),
        actor_label: "web:portfolio-mutation-disconnect".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: input.to_string(),
        context: AgentContext::new("portfolio-mutation-disconnect-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "portfolio-mutation-disconnect-session",
            request,
            Arc::new(NoopEmitter),
            None,
            policy,
            None,
        )
        .await;

    assert!(!result.response.success);
    assert_eq!(
        result.response.error.as_deref(),
        Some(crate::tool_trace::PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE)
    );
    assert_eq!(results.lock().expect("results lock").len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn deep_research_start_disconnect_does_not_launch_a_second_task_without_trace() {
    let root = make_temp_dir("hone_channels_deep_research_disconnect_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor =
        ActorIdentity::new("web", "deep-research-disconnect", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: false,
                error: Some("codex acp stream disconnected before completion".to_string()),
            },
            streamed_output: false,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "不应重复启动深度研究任务".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let input = "请对 NBIS 做深度研究";
    let policy = prepared_turn_reexecution_policy(input);
    assert_eq!(policy, PreparedTurnReexecutionPolicy::ExecuteOnce);
    let request = AgentRunnerRequest {
        session_id: "deep-research-disconnect-session".to_string(),
        actor_label: "web:deep-research-disconnect".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: input.to_string(),
        context: AgentContext::new("deep-research-disconnect-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "deep-research-disconnect-session",
            request,
            Arc::new(NoopEmitter),
            None,
            policy,
            None,
        )
        .await;

    assert!(!result.response.success);
    assert!(
        result
            .response
            .error
            .as_deref()
            .is_some_and(|error| error.contains("重复启动研究任务"))
    );
    assert_eq!(results.lock().expect("results lock").len(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn post_quote_runner_failure_stays_failed_but_incomplete_success_uses_fallback() {
    let root = make_temp_dir("hone_channels_post_quote_runner_failure");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "post-quote-failure", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: String::new(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: false,
                error: Some("upstream model rejected the synthesis request".to_string()),
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: true,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "post-quote-failure-session".to_string(),
        actor_label: "web:post-quote-failure".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "现在 NBIS 怎么看".to_string(),
        context: AgentContext::new("post-quote-failure-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };
    let contract = InvestmentResponseContract {
        entities: vec![ResolvedSecurityEntity {
            mention: "nbis".into(),
            symbol: "NBIS".into(),
            name: "Nebius Group N.V.".into(),
            exchange: Some("NASDAQ".into()),
            currency: Some("USD".into()),
            asset_type: Some("stock".into()),
            profile_verified: true,
            verified_price: Some("199.51".into()),
            verified_change_percentage: Some("1.25".into()),
            quote_timestamp: None,
            quote_session: None,
            annual_financials_verified: None,
            verified_annual_financial_facts: Vec::new(),
            fund_holdings_verified: None,
            verified_fund_holding_facts: Vec::new(),
        }],
        verified_web_sources: Vec::new(),
        verified_dated_web_sources: Vec::new(),
        deep_analysis: DeepAnalysisKind::Equity,
        deep_comparison: false,
        requires_verified_price: true,
        needs_outlook_evidence: false,
        requires_recent_web_evidence: false,
        comparison: false,
        origin: AgentTurnOrigin::Interactive,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "post-quote-failure-session",
            request.clone(),
            Arc::new(NoopEmitter),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;

    assert!(!result.response.success);
    assert!(
        result.response.content.starts_with("数据时间：北京时间 "),
        "content={} calls={:?}",
        result.response.content,
        result.response.tool_calls_made
    );
    assert!(
        result
            .response
            .content
            .contains("Nebius Group N.V.（NBIS）本轮同代码现价 199.51 USD")
    );
    assert!(
        result
            .response
            .content
            .contains("upstream model rejected the synthesis request")
    );
    assert_eq!(
        result.response.error.as_deref(),
        Some(result.response.content.as_str())
    );
    assert!(!result.streamed_output);
    assert!(!result.terminal_error_emitted);
    assert!(results.lock().expect("results lock").is_empty());

    let retry_results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: "Q3 也许会上涨。".to_string(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let retry_runner = MockSequencedRunner {
        results: retry_results.clone(),
    };
    let fallback = session
        .run_runner_with_investment_contract_retry(
            &retry_runner,
            "mock_sequenced",
            "post-quote-retry-failure-session",
            request,
            Arc::new(NoopEmitter),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;
    assert!(fallback.response.success);
    assert!(fallback.response.error.is_none());
    assert!(fallback.response.content.starts_with("数据时间：北京时间 "));
    assert!(
        fallback
            .response
            .content
            .contains("Nebius Group N.V.（NBIS）本轮同代码现价 199.51 USD")
    );
    for section in 1..=9 {
        assert!(
            fallback
                .response
                .content
                .contains(&format!("## {section}."))
        );
    }
    assert!(!fallback.response.content.contains("Q3 也许会上涨"));
    assert!(!fallback.streamed_output);
    assert!(!fallback.terminal_error_emitted);
    assert!(retry_results.lock().expect("retry results lock").is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn investment_contract_uses_verified_fallback_for_incomplete_nbis_draft() {
    let root = make_temp_dir("hone_channels_investment_contract_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "investment-contract", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let incomplete_raw = "<think>内部推理不得带入修订请求</think>\nQ3 可能起飞。你成本多少？";
    let runs = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        MockStreamingRun {
            events: vec![
                AgentRunnerEvent::Progress {
                    stage: "agent.run",
                    detail: Some("first attempt".to_string()),
                },
                AgentRunnerEvent::StreamDelta {
                    content: incomplete_raw.to_string(),
                },
                AgentRunnerEvent::StreamReset,
                AgentRunnerEvent::StreamThought {
                    thought: "retry this draft".to_string(),
                },
                AgentRunnerEvent::Error {
                    error: super::types::AgentSessionError {
                        kind: AgentSessionErrorKind::AgentFailed,
                        message: "attempt-local error".to_string(),
                    },
                },
            ],
            result: AgentRunnerResult {
                response: AgentResponse {
                    content: "Q3 可能起飞。你成本多少？".to_string(),
                    tool_calls_made: Vec::new(),
                    iterations: 1,
                    success: true,
                    error: None,
                },
                streamed_output: true,
                committed_visible_prefix: None,
                terminal_error_emitted: true,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            },
        },
    ])));
    let runtime_inputs = Arc::new(Mutex::new(Vec::new()));
    let runner = MockStreamingSequencedRunner {
        runs: runs.clone(),
        runtime_inputs: runtime_inputs.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "investment-contract-session".to_string(),
        actor_label: "web:investment-contract".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "我想了解Q3的时候nbis能不能起飞".to_string(),
        context: AgentContext::new("investment-contract-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };
    let contract = InvestmentResponseContract {
        entities: vec![ResolvedSecurityEntity {
            mention: "nbis".into(),
            symbol: "NBIS".into(),
            name: "Nebius Group N.V.".into(),
            exchange: Some("NASDAQ".into()),
            currency: Some("USD".into()),
            asset_type: Some("stock".into()),
            profile_verified: true,
            verified_price: Some("199.51".into()),
            verified_change_percentage: None,
            quote_timestamp: None,
            quote_session: None,
            annual_financials_verified: None,
            verified_annual_financial_facts: Vec::new(),
            fund_holdings_verified: None,
            verified_fund_holding_facts: Vec::new(),
        }],
        verified_web_sources: Vec::new(),
        verified_dated_web_sources: Vec::new(),
        deep_analysis: DeepAnalysisKind::Equity,
        deep_comparison: false,
        requires_verified_price: true,
        needs_outlook_evidence: true,
        requires_recent_web_evidence: false,
        comparison: false,
        origin: AgentTurnOrigin::Interactive,
    };
    let downstream = Arc::new(RecordingRunnerEmitter::default());
    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_streaming_sequenced",
            "investment-contract-session",
            request,
            downstream.clone(),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;

    assert!(
        result.response.success,
        "deterministic fallback failed: {:?}",
        result.response.error
    );
    assert!(result.response.content.starts_with("数据时间：北京时间 "));
    assert!(
        result
            .response
            .content
            .contains("标的核验：Nebius Group N.V.（NBIS")
    );
    for section in 1..=9 {
        assert!(result.response.content.contains(&format!("## {section}.")));
    }
    assert!(!result.response.content.contains("Q3 可能起飞"));
    assert!(!result.response.content.contains("你成本多少"));
    assert!(!result.streamed_output);
    assert!(!result.terminal_error_emitted);
    assert!(runs.lock().expect("runs lock").is_empty());
    let runtime_inputs = runtime_inputs.lock().expect("runtime inputs lock");
    assert_eq!(runtime_inputs.len(), 1);
    assert!(!runtime_inputs[0].contains("<investment_draft>"));
    let events = downstream.events.lock().await;
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        AgentRunnerEvent::Progress {
            stage: "agent.run",
            ..
        }
    ));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn investment_fallback_fails_closed_for_unknown_tool_trace() {
    let root = make_temp_dir("hone_channels_investment_unknown_tool_no_fallback");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor =
        ActorIdentity::new("web", "investment-unknown-tool", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: "NBIS 看多。".to_string(),
                tool_calls_made: vec![ToolCallMade {
                    name: "mcp__filesystem__write_file".to_string(),
                    arguments: serde_json::json!({"path":"external-state", "content":"changed"}),
                    result: serde_json::json!({"status":"success"}),
                    tool_call_id: Some("unknown_write".to_string()),
                }],
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: Some(vec![AgentMessage {
                role: "assistant".to_string(),
                content: Some("NBIS 看多。".to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                metadata: None,
            }]),
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "investment-unknown-tool-session".to_string(),
        actor_label: "web:investment-unknown-tool".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "现在 NBIS 怎么看".to_string(),
        context: AgentContext::new("investment-unknown-tool-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };
    let contract = InvestmentResponseContract {
        entities: vec![ResolvedSecurityEntity {
            mention: "nbis".into(),
            symbol: "NBIS".into(),
            name: "Nebius Group N.V.".into(),
            exchange: Some("NASDAQ".into()),
            currency: Some("USD".into()),
            asset_type: Some("stock".into()),
            profile_verified: true,
            verified_price: Some("199.51".into()),
            verified_change_percentage: None,
            quote_timestamp: None,
            quote_session: None,
            annual_financials_verified: Some(false),
            verified_annual_financial_facts: Vec::new(),
            fund_holdings_verified: None,
            verified_fund_holding_facts: Vec::new(),
        }],
        verified_web_sources: Vec::new(),
        verified_dated_web_sources: Vec::new(),
        deep_analysis: DeepAnalysisKind::Equity,
        deep_comparison: false,
        requires_verified_price: true,
        needs_outlook_evidence: true,
        requires_recent_web_evidence: false,
        comparison: false,
        origin: AgentTurnOrigin::Interactive,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "investment-unknown-tool-session",
            request,
            Arc::new(NoopEmitter),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;

    assert!(!result.response.success);
    assert!(
        result
            .response
            .error
            .as_deref()
            .is_some_and(|error| error.contains("无法确认只读属性的工具"))
    );
    assert_eq!(result.response.tool_calls_made.len(), 1);
    assert!(result.context_messages.is_some());
    assert!(results.lock().expect("results lock").is_empty());
    let _ = std::fs::remove_dir_all(root);
}

fn repair_trace_comparison_contract() -> InvestmentResponseContract {
    let entity = |symbol: &str, name: &str, price: &str| ResolvedSecurityEntity {
        mention: symbol.to_string(),
        symbol: symbol.to_string(),
        name: name.to_string(),
        exchange: Some("NASDAQ".to_string()),
        currency: Some("USD".to_string()),
        asset_type: Some("stock".to_string()),
        profile_verified: true,
        verified_price: Some(price.to_string()),
        verified_change_percentage: None,
        quote_timestamp: None,
        quote_session: None,
        annual_financials_verified: Some(false),
        verified_annual_financial_facts: Vec::new(),
        fund_holdings_verified: None,
        verified_fund_holding_facts: Vec::new(),
    };
    InvestmentResponseContract {
        entities: vec![
            entity("CRWV", "CoreWeave, Inc.", "73.21"),
            entity("NBIS", "Nebius Group N.V.", "177.71"),
        ],
        verified_web_sources: Vec::new(),
        verified_dated_web_sources: Vec::new(),
        deep_analysis: DeepAnalysisKind::None,
        deep_comparison: true,
        requires_verified_price: true,
        needs_outlook_evidence: false,
        requires_recent_web_evidence: false,
        comparison: true,
        origin: AgentTurnOrigin::Interactive,
    }
}

fn repair_trace_request(
    session: &AgentSession,
    root: &std::path::Path,
    session_id: &str,
) -> AgentRunnerRequest {
    AgentRunnerRequest {
        session_id: session_id.to_string(),
        actor_label: format!("web:{session_id}"),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "分析下 CRWV 和 NBIS".to_string(),
        context: AgentContext::new(session_id.to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    }
}

fn repair_trace_call(name: &str, arguments: Value, id: &str) -> ToolCallMade {
    ToolCallMade {
        name: name.to_string(),
        arguments,
        result: serde_json::json!({"status":"success"}),
        tool_call_id: Some(id.to_string()),
    }
}

#[tokio::test]
async fn investment_contract_repair_keeps_initial_and_retry_tool_traces() {
    let root = make_temp_dir("hone_channels_investment_repair_trace_merge");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "repair-trace-merge", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: "只分析了 CRWV。".to_string(),
                tool_calls_made: vec![repair_trace_call(
                    "data_fetch",
                    serde_json::json!({"data_type":"search", "query":"CRWV"}),
                    "initial_search",
                )],
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
        AgentRunnerResult {
            response: AgentResponse {
                content: "比较结论：以下区分已核验事实与情景推断。\n### CRWV\nCRWV 当前价 73.21 USD；年度财务本轮未核验，估值方法采用 P/S 情景法，相关输入仍未核验。\n### NBIS\nNBIS 当前价 177.71 USD；年度财务本轮未核验，估值方法采用 P/S 情景法，相关输入仍未核验。\n风险与证伪条件：若经营口径失效则判断失效。\n动作建议与触发条件：先观察，满足触发条件后再评估。".to_string(),
                tool_calls_made: vec![repair_trace_call(
                    "web_search",
                    serde_json::json!({"query":"CRWV NBIS valuation"}),
                    "retry_search",
                )],
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let contract = repair_trace_comparison_contract();

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "repair-trace-merge-session",
            repair_trace_request(&session, &root, "repair-trace-merge-session"),
            Arc::new(NoopEmitter),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.tool_calls_made.len(), 2);
    assert_eq!(
        result
            .response
            .tool_calls_made
            .iter()
            .map(|call| call.tool_call_id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("initial_search"), Some("retry_search")]
    );
    assert!(results.lock().expect("results lock").is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn investment_contract_repair_rejects_unknown_and_persistent_retry_traces() {
    let cases = vec![
        (
            "unknown",
            repair_trace_call(
                "mcp__filesystem__write_file",
                serde_json::json!({"path":"external-state", "content":"changed"}),
                "retry_unknown_write",
            ),
            crate::tool_trace::UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE,
        ),
        (
            "persistent",
            repair_trace_call(
                "portfolio",
                serde_json::json!({"action":"add", "symbol":"NBIS"}),
                "retry_portfolio_add",
            ),
            crate::tool_trace::PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE,
        ),
    ];

    for (case_name, unsafe_retry_call, expected_message) in cases {
        let root = make_temp_dir(&format!("hone_channels_investment_repair_{case_name}"));
        std::fs::create_dir_all(&root).expect("create root");
        let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
        let actor = ActorIdentity::new("web", format!("repair-{case_name}"), None::<String>)
            .expect("actor");
        let session = AgentSession::new(core, actor, "direct");
        let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
            AgentRunnerResult {
                response: AgentResponse {
                    content: "只分析了 CRWV。".to_string(),
                    tool_calls_made: vec![repair_trace_call(
                        "data_fetch",
                        serde_json::json!({"data_type":"search", "query":"CRWV"}),
                        "initial_search",
                    )],
                    iterations: 1,
                    success: true,
                    error: None,
                },
                streamed_output: true,
                committed_visible_prefix: None,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            },
            AgentRunnerResult {
                response: AgentResponse {
                    content: "比较结论：以下区分已核验事实与情景推断。\n### CRWV\nCRWV 当前价 73.21 USD；年度财务本轮未核验，估值方法采用 P/S 情景法，相关输入仍未核验。\n### NBIS\nNBIS 当前价 177.71 USD；年度财务本轮未核验，估值方法采用 P/S 情景法，相关输入仍未核验。\n风险与证伪条件：若经营口径失效则判断失效。\n动作建议与触发条件：先观察，满足触发条件后再评估。".to_string(),
                    tool_calls_made: vec![unsafe_retry_call],
                    iterations: 1,
                    success: true,
                    error: None,
                },
                streamed_output: true,
                committed_visible_prefix: None,
                terminal_error_emitted: false,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            },
        ])));
        let runner = MockSequencedRunner {
            results: results.clone(),
        };
        let contract = repair_trace_comparison_contract();
        let session_id = format!("repair-{case_name}-session");

        let result = session
            .run_runner_with_investment_contract_retry(
                &runner,
                "mock_sequenced",
                &session_id,
                repair_trace_request(&session, &root, &session_id),
                Arc::new(NoopEmitter),
                Some(&contract),
                PreparedTurnReexecutionPolicy::Allowed,
                None,
            )
            .await;

        assert!(!result.response.success, "{case_name}");
        assert!(
            result
                .response
                .error
                .as_deref()
                .is_some_and(|error| error.contains(expected_message)),
            "{case_name}: {:?}",
            result.response.error
        );
        assert_eq!(result.response.tool_calls_made.len(), 2, "{case_name}");
        assert_eq!(
            result.response.tool_calls_made[0].tool_call_id.as_deref(),
            Some("initial_search"),
            "{case_name}"
        );
        assert!(results.lock().expect("results lock").is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}

#[tokio::test]
async fn fund_contract_discards_forbidden_financial_call_and_uses_safe_fallback() {
    let root = make_temp_dir("hone_channels_fund_forbidden_call_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "fund-contract", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let complete = "数据时间：北京时间 2026-07-16。已核验事实与情景假设分开。\n1. 结论：本轮同代码现价 30.495 美元，先观察。\n2. 基金目标、基金策略与跟踪对象：跟踪国际市场暴露是核心目标。\n3. 持仓、集中度与主要暴露：持仓与集中度按本轮数据核验。\n4. 地域、行业与货币风险：地域与汇率风险需同时管理。\n5. 流动性、基金规模与交易特征：基金规模本轮未核验；流动性与成交特征决定交易成本。\n6. 费用、跟踪误差与底层资产估值：费率与跟踪误差本轮未核验；底层估值是关键变量。\n7. Bull / Bear / Base Case：Bull 看风险偏好，Bear 看汇率，Base 看基准收益。\n8. 催化剂、风险点、证伪条件：催化是宽松，风险是波动，证伪是暴露失效。\n9. 动作建议：观察；若费率、流动性与暴露均符合条件则再评估。";
    let forbidden_call = ToolCallMade {
        name: "data_fetch".into(),
        arguments: serde_json::json!({"data_type":"financials","ticker":"INTL"}),
        result: serde_json::json!({"data":[]}),
        tool_call_id: None,
    };
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: complete.to_string(),
                tool_calls_made: vec![forbidden_call],
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: Some(vec![AgentMessage {
                role: "assistant".into(),
                content: Some("rejected fund draft".into()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                metadata: None,
            }]),
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "fund-contract-session".to_string(),
        actor_label: "web:fund-contract".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "现在 INTL 怎么看".to_string(),
        context: AgentContext::new("fund-contract-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };
    let contract = InvestmentResponseContract {
        entities: vec![ResolvedSecurityEntity {
            mention: "intl".into(),
            symbol: "INTL".into(),
            name: "Main International ETF".into(),
            exchange: Some("CBOE".into()),
            currency: Some("USD".into()),
            asset_type: Some("etf_or_fund".into()),
            profile_verified: true,
            verified_price: Some("30.495".into()),
            verified_change_percentage: None,
            quote_timestamp: None,
            quote_session: None,
            annual_financials_verified: None,
            verified_annual_financial_facts: Vec::new(),
            fund_holdings_verified: None,
            verified_fund_holding_facts: Vec::new(),
        }],
        verified_web_sources: Vec::new(),
        verified_dated_web_sources: Vec::new(),
        deep_analysis: DeepAnalysisKind::Fund,
        deep_comparison: false,
        requires_verified_price: true,
        needs_outlook_evidence: false,
        requires_recent_web_evidence: false,
        comparison: false,
        origin: AgentTurnOrigin::Interactive,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "fund-contract-session",
            request,
            Arc::new(NoopEmitter),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;

    assert!(result.response.success);
    assert!(result.response.error.is_none());
    assert!(result.response.content.starts_with("数据时间：北京时间 "));
    for section in 1..=9 {
        assert!(result.response.content.contains(&format!("## {section}.")));
    }
    assert!(result.response.content.contains("基金目标、策略与跟踪对象"));
    assert!(!result.response.content.contains("公司是什么、靠什么赚钱"));
    assert!(result.response.tool_calls_made.is_empty());
    assert!(result.context_messages.is_none());
    assert!(results.lock().expect("results lock").is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn investment_contract_sanitizes_and_server_normalizes_the_visible_text() {
    let root = make_temp_dir("hone_channels_investment_contract_visible_text");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor = ActorIdentity::new("web", "visible-contract", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let visible = "数据时间：北京时间 2026-07-16。已核验事实与情景假设分开。\nINTL 当前价 30.495 美元。\n1. 结论：本轮判断以观察为主。\n2. 基金目标、基金策略与跟踪对象：跟踪国际市场暴露是核心目标。\n3. 持仓、集中度与主要暴露：持仓与集中度按本轮数据核验。\n4. 地域、行业与货币风险：地域与汇率风险需同时管理。\n5. 流动性、基金规模与交易特征：基金规模本轮未核验；流动性与成交特征决定交易成本。\n6. 费用、跟踪误差与底层资产估值：费率与跟踪误差本轮未核验；底层估值是关键变量。\n7. Bull / Bear / Base Case：Bull 看风险偏好，Bear 看汇率，Base 看基准收益。\n8. 催化剂、风险点、证伪条件：催化是宽松，风险是波动，证伪是暴露失效。\n9. 动作建议：观察；若费率、流动性与暴露均符合条件则再评估。";
    let raw =
        format!("<think>\n1. 先规划输出\n2. 这里是内部推理，不是基金目标章节\n</think>\n{visible}");
    let results = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        AgentRunnerResult {
            response: AgentResponse {
                content: raw.clone(),
                tool_calls_made: Vec::new(),
                iterations: 1,
                success: true,
                error: None,
            },
            streamed_output: true,
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages: None,
        },
    ])));
    let runner = MockSequencedRunner {
        results: results.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "visible-contract-session".to_string(),
        actor_label: "web:visible-contract".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "现在 INTL 怎么看".to_string(),
        context: AgentContext::new("visible-contract-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };
    let contract = InvestmentResponseContract {
        entities: vec![ResolvedSecurityEntity {
            mention: "intl".into(),
            symbol: "INTL".into(),
            name: "Main International ETF".into(),
            exchange: Some("CBOE".into()),
            currency: Some("USD".into()),
            asset_type: Some("etf_or_fund".into()),
            profile_verified: true,
            verified_price: Some("30.495".into()),
            verified_change_percentage: None,
            quote_timestamp: None,
            quote_session: None,
            annual_financials_verified: None,
            verified_annual_financial_facts: Vec::new(),
            fund_holdings_verified: None,
            verified_fund_holding_facts: Vec::new(),
        }],
        verified_web_sources: Vec::new(),
        verified_dated_web_sources: Vec::new(),
        deep_analysis: DeepAnalysisKind::Fund,
        deep_comparison: false,
        requires_verified_price: true,
        needs_outlook_evidence: false,
        requires_recent_web_evidence: false,
        comparison: false,
        origin: AgentTurnOrigin::Interactive,
    };

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_sequenced",
            "visible-contract-session",
            request,
            Arc::new(NoopEmitter),
            Some(&contract),
            PreparedTurnReexecutionPolicy::Allowed,
            None,
        )
        .await;

    assert!(result.response.success);
    assert!(result.response.content.starts_with("数据时间：北京时间 "));
    assert!(
        result
            .response
            .content
            .contains("标的核验：Main International ETF（INTL")
    );
    assert!(result.response.content.contains("INTL 当前价 30.495 美元"));
    assert!(
        result
            .response
            .content
            .contains("1. 结论：本轮判断以观察为主")
    );
    assert!(result.response.content.contains("9. 动作建议：观察"));
    assert!(!result.response.content.contains("<think>"));
    assert!(
        result
            .response
            .content
            .contains("已核验事实：Main International ETF（INTL）本轮同代码现价 30.495 USD")
    );
    assert!(results.lock().expect("results lock").is_empty());
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

    let restored_context = restore_context(&storage, &session_id, Some(4), None);
    let contents: Vec<_> = restored_context
        .messages
        .iter()
        .filter_map(|m| m.content.as_deref())
        .collect();
    assert_eq!(contents, vec!["a1", "t1", "u2", "a2"]);
    assert_eq!(restored_context.messages[1].role, "tool");
    assert_eq!(
        restored_context.messages[1].name.as_deref(),
        Some("web_search")
    );
    assert_eq!(
        restored_context.messages[1].tool_call_id.as_deref(),
        Some("call_1")
    );
    assert_eq!(restored_context.actor_identity(), Some(actor));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn restore_context_rehydrates_assistant_tool_calls() {
    let root = make_temp_dir("hone_channels_restore_tool_calls");
    let storage = SessionStorage::new(&root);
    let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");
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

    let restored_context = restore_context(&storage, &session_id, None, None);
    assert_eq!(restored_context.messages.len(), 3);
    assert_eq!(restored_context.messages[1].role, "assistant");
    let tool_calls = restored_context.messages[1]
        .tool_calls
        .as_ref()
        .expect("assistant tool calls");
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0]["id"], "call_1");
    assert_eq!(tool_calls[0]["function"]["name"], "local_search_files");
    assert_eq!(restored_context.messages[2].role, "tool");
    assert_eq!(
        restored_context.messages[2].tool_call_id.as_deref(),
        Some("call_1")
    );

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

    let restored_context = restore_context(&storage, &session_id, None, None);
    assert_eq!(restored_context.messages.len(), 2);
    assert_eq!(
        restored_context.messages[0]
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
        restored_context.messages[1]
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
fn interactive_runtime_history_drops_scheduler_and_failed_turn_groups() {
    let message =
        |role: &str, content: &str, metadata: Option<HashMap<String, Value>>| AgentMessage {
            role: role.to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata,
        };
    let mut messages = vec![
        message("user", "正常历史问题", None),
        message("assistant", "正常历史回答", None),
        message(
            "user",
            "[定时任务触发] 任务名称：英伟达每日消息",
            Some(HashMap::from([(
                "source".to_string(),
                Value::String("scheduler".to_string()),
            )])),
        ),
        message(
            "assistant",
            "旧 NVDA 推送",
            Some(HashMap::from([(
                "job_id".to_string(),
                Value::String("j_nvda".to_string()),
            )])),
        ),
        message("user", "分析下 CRWV 和 NBIS", None),
        message(
            "assistant",
            "投研完整性检查失败",
            Some(HashMap::from([(
                "run_failed".to_string(),
                Value::Bool(true),
            )])),
        ),
        message("user", "crwv和英伟达什么关系，估值怎么看", None),
    ];

    let removed =
        prune_interactive_runtime_history(&mut messages, "crwv和英伟达什么关系，估值怎么看");

    assert_eq!(removed, 4);
    assert_eq!(
        messages
            .iter()
            .filter_map(|message| message.content.as_deref())
            .collect::<Vec<_>>(),
        [
            "正常历史问题",
            "正常历史回答",
            "crwv和英伟达什么关系，估值怎么看",
        ]
    );
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
    assert!(system_with_match.contains("【SkillTool】"));
    assert!(!system_with_match.contains("turn-0 可用技能索引"));
    assert!(!system_with_match.contains("alpha_skill"));
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
fn resolve_prompt_input_warns_web_cron_cannot_send_mobile_system_push() {
    let root = make_temp_dir("hone_channels_prompt_web_cron_delivery");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_tool_responses(Vec::new());
    let core = make_test_core(&root, llm);
    let actor = ActorIdentity::new("web", "web-user", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "web-user").with_cron_allowed(true);

    let (system_prompt, _) = session.resolve_prompt_input("session-demo", "3 分钟后提醒我");

    assert!(system_prompt.contains("【Web 定时任务送达边界】"));
    assert!(system_prompt.contains("只保证写入当前 Hone 会话"));
    assert!(system_prompt.contains("当前没有 Web Push / 手机系统通知能力"));
    assert!(system_prompt.contains("不要承诺会出现在手机通知中心"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn resolve_prompt_input_maps_cron_enabled_flags_to_user_language() {
    let root = make_temp_dir("hone_channels_prompt_cron_enabled_language");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_tool_responses(Vec::new());
    let core = make_test_core(&root, llm);
    let actor = ActorIdentity::new("feishu", "ou_cron", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "ou_cron").with_cron_allowed(true);

    let (system_prompt, _) = session.resolve_prompt_input("session-demo", "我有哪些定时任务");

    assert!(system_prompt.contains("【定时任务 / 心跳任务策略】"));
    assert!(system_prompt.contains("必须调用真实 `cron_job` 工具完成"));
    assert!(system_prompt.contains("不能用沙盒目录、SQLite、会话历史或文件列表自查替代"));
    assert!(system_prompt.contains("定时任务管理暂时不可用，请稍后再试"));
    assert!(system_prompt.contains("禁止向用户输出 `工具未暴露`"));
    assert!(system_prompt.contains("`sessions.sqlite3`"));
    assert!(system_prompt.contains("不要直接复述 `enabled=true`"));
    assert!(system_prompt.contains("已启用 / 已停用"));
    assert!(system_prompt.contains("豁免勿扰"));

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
fn finalize_agent_response_recovers_cron_job_confirmation_from_tool_result() {
    let root = make_temp_dir("hone_channels_finalize_cron_confirmation");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "我先处理这个监控任务，稍后给你创建结果。".to_string(),
        tool_calls_made: vec![ToolCallMade {
            name: "cron_job".to_string(),
            arguments: serde_json::json!({"action": "add"}),
            result: serde_json::json!({
                "success": true,
                "job": {
                    "id": "j_market20",
                    "name": "每日大盘监控",
                    "schedule": {
                        "hour": 20,
                        "minute": 0,
                        "repeat": "daily"
                    }
                }
            }),
            tool_call_id: None,
        }],
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert!(response.error.is_none());
    assert_eq!(
        response.content,
        "已创建定时任务：每日大盘监控（每天 20:00）。任务 ID：j_market20。"
    );
    assert!(outcome.fallback_reason.is_none());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finalize_agent_response_recovers_cron_job_list_from_tool_result() {
    let root = make_temp_dir("hone_channels_finalize_cron_list_confirmation");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "我先查一下你现有的定时任务。".to_string(),
        tool_calls_made: vec![ToolCallMade {
            name: "cron_job".to_string(),
            arguments: serde_json::json!({"action": "list"}),
            result: serde_json::json!({
                "action": "list",
                "jobs": [
                    {
                        "id": "j_open",
                        "name": "盘前简报",
                        "enabled": true,
                        "schedule": {
                            "hour": 8,
                            "minute": 30,
                            "repeat": "trading_day"
                        }
                    },
                    {
                        "id": "j_close",
                        "name": "收盘复盘",
                        "enabled": false,
                        "schedule": {
                            "hour": 20,
                            "minute": 0,
                            "repeat": "daily"
                        }
                    }
                ]
            }),
            tool_call_id: None,
        }],
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert!(response.error.is_none());
    assert_eq!(
        response.content,
        "你当前有 2 个定时任务：盘前简报（交易日 08:30），任务 ID：j_open；收盘复盘（每天 20:00，已停用），任务 ID：j_close。"
    );
    assert!(outcome.fallback_reason.is_none());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finalize_agent_response_recovers_cron_job_remove_confirmation_from_tool_result() {
    let root = make_temp_dir("hone_channels_finalize_cron_remove_confirmation");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "当前没有可用的定时任务注册入口，因此不能直接完成自动创建。".to_string(),
        tool_calls_made: vec![ToolCallMade {
            name: "cron_job".to_string(),
            arguments: serde_json::json!({"action": "remove", "job_id": "j_open"}),
            result: serde_json::json!({
                "action": "remove",
                "success": false,
                "needs_confirmation": true,
                "job": {
                    "id": "j_open",
                    "name": "盘前简报",
                    "enabled": true,
                    "schedule": {
                        "hour": 8,
                        "minute": 30,
                        "repeat": "trading_day"
                    }
                }
            }),
            tool_call_id: None,
        }],
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert!(response.error.is_none());
    assert_eq!(
        response.content,
        "删除前需要你确认：盘前简报（交易日 08:30），任务 ID：j_open。如果确认删除，请明确回复要删除这个任务。"
    );
    assert!(outcome.fallback_reason.is_none());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finalize_agent_response_recovers_cron_job_result_after_sanitization_strips_internal_copy() {
    let root = make_temp_dir("hone_channels_finalize_cron_sanitized_empty_recovery");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "自动定时任务注册工具没有暴露出来，所以我不能确认任务已经正式创建成功。"
            .to_string(),
        tool_calls_made: vec![ToolCallMade {
            name: "cron_job".to_string(),
            arguments: serde_json::json!({"action": "add"}),
            result: serde_json::json!({
                "success": true,
                "job": {
                    "id": "j_pke",
                    "name": "PKE 双周检查",
                    "schedule": {
                        "hour": 9,
                        "minute": 0,
                        "repeat": "weekly",
                        "weekday": 0
                    }
                }
            }),
            tool_call_id: None,
        }],
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert!(response.error.is_none());
    assert_eq!(
        response.content,
        "已创建定时任务：PKE 双周检查（每周一 09:00）。任务 ID：j_pke。"
    );
    assert!(outcome.fallback_reason.is_none());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finalize_agent_response_recovers_portfolio_confirmation_from_tool_result() {
    let root = make_temp_dir("hone_channels_finalize_portfolio_confirmation");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "我先把你的 RDW 持仓记录好，然后继续跟踪。".to_string(),
        tool_calls_made: vec![ToolCallMade {
            name: "portfolio".to_string(),
            arguments: serde_json::json!({
                "action": "add",
                "ticker": "rdw",
                "cost_basis": 12
            }),
            result: serde_json::json!({
                "action": "add",
                "count": 1,
                "holdings": [{
                    "ticker": "RDW",
                    "asset_type": "stock",
                    "holding_horizon": null,
                    "strategy_notes": null,
                    "promoted_from_watchlist": false
                }],
                "success": true,
                "ticker": "RDW",
                "asset_type": "stock"
            }),
            tool_call_id: None,
        }],
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert!(response.error.is_none());
    assert_eq!(
        response.content,
        "已记录持仓：RDW，成本价 12。后续跟踪会优先参考这条持仓记录。"
    );
    assert!(outcome.fallback_reason.is_none());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finalize_agent_response_recovers_portfolio_view_holding_confirmation() {
    let root = make_temp_dir("hone_channels_finalize_portfolio_view_confirmation");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let mut response = AgentResponse {
        content: "我先看一下你的 VST 持仓和计划，再给你确认。".to_string(),
        tool_calls_made: vec![ToolCallMade {
            name: "portfolio".to_string(),
            arguments: serde_json::json!({
                "action": "view",
                "ticker": "vst"
            }),
            result: serde_json::json!({
                "action": "view",
                "portfolio": {
                    "holdings": [{
                        "symbol": "VST",
                        "asset_type": "stock",
                        "shares": 215.0,
                        "avg_cost": 139.84,
                        "notes": "用户计划若价格到140.7附近减仓176股，目前仅记录为计划，未按已成交卖出处理",
                        "kind": "holding"
                    }],
                    "watchlist": [],
                    "updated_at": "2026-05-18T16:08:14Z"
                }
            }),
            tool_call_id: None,
        }],
        iterations: 1,
        success: true,
        error: None,
    };

    let outcome = finalize_agent_response(&core, "session", "mock", &mut response);

    assert!(response.success);
    assert!(response.error.is_none());
    assert_eq!(
        response.content,
        "已读取相关持仓记录：VST，215 股，成本价 139.84，备注：用户计划若价格到140.7附近减仓176股，目前仅记录为计划，未按已成交卖出处理。"
    );
    assert!(outcome.fallback_reason.is_none());

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
    assert!(response.error.is_none());
    assert!(outcome.fallback_reason.is_none());

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

    let restored_context = restore_context(&storage, &session_id, None, None);
    let contents: Vec<_> = restored_context
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

    let restored_context = restore_context(&storage, &session_id, Some(5), None);
    let contents: Vec<_> = restored_context
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
    let restored_context = restore_context(&storage, &session_id, Some(5), Some(&runtime));
    let contents: Vec<_> = restored_context
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

    let restored_context = restore_context(&storage, &session_id, Some(10), None);
    let contents: Vec<_> = restored_context
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

    let restored_context = restore_context(&storage, &session_id, Some(10), None);
    let contents: Vec<_> = restored_context
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

    let restored_context = restore_context(&storage, &session_id, Some(10), None);
    let contents: Vec<_> = restored_context
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
        reasoning_content: None,
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
        reasoning_content: None,
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

    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());
    session.add_listener(listener.clone());
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
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content[0].text.as_deref(), Some("hello"));
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content[0].text.as_deref(), Some(error.as_str()));
    let events = listener.events.lock().await.clone();
    assert!(events.iter().any(|event| {
        matches!(
            event,
            AgentSessionEvent::Done { response }
                if !response.success
                    && response
                        .error
                        .as_deref()
                        .is_some_and(|err| err.contains("已达到今日对话上限"))
        )
    }));
    assert_eq!(
        messages[1]
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("quota_rejected"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
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
async fn run_persists_failed_assistant_turn_when_strict_fallback_llm_is_missing() {
    let root = make_temp_dir("hone_channels_guard_fail_persist");
    std::fs::create_dir_all(&root).expect("create root");
    let mut config = HoneConfig::default();
    config.agent.runner = "codex_acp".to_string();
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
    let core = Arc::new(HoneBotCore::new(config));
    let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");

    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());
    session.add_listener(listener.clone());
    let result = session.run("帮我看持仓", AgentRunOptions::default()).await;

    assert!(!result.response.success);
    let messages = core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .expect("messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(
        messages[1].content[0].text.as_deref(),
        Some(
            "安全执行器不可用：普通用户不能使用具备宿主机访问能力的 CLI/ACP，且严格 function-calling LLM 未配置。"
        )
    );
    assert_eq!(
        messages[1]
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("run_failed"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    let events = listener.events.lock().await;
    assert!(!events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::Progress {
            stage: "entity_resolution.preflight" | "entity_resolution.preflight.done",
            ..
        })
    )));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn run_persists_failed_assistant_turn_for_runner_failure() {
    let root = make_temp_dir("hone_channels_runner_fail_persist");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(
        vec![],
        vec![Err(hone_core::HoneError::Llm(
            "provider transport error".to_string(),
        ))],
    );
    let core = make_test_core(&root, llm);
    let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");

    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());
    session.add_listener(listener.clone());
    let result = session
        .run("帮我看看 KLAC", AgentRunOptions::default())
        .await;

    assert!(!result.response.success);
    let messages = core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .expect("messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
    let events = listener.events.lock().await;
    assert!(!events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::Progress {
            stage: "entity_resolution.preflight",
            detail: None,
        })
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::Progress {
            stage: "agent.run",
            ..
        })
    )));
    let assistant_text = messages[1].content[0].text.as_deref().unwrap_or_default();
    assert!(!assistant_text.is_empty());
    assert!(
        !assistant_text.contains("provider transport error"),
        "persisted assistant turn should stay user-facing, got: {assistant_text}"
    );
    assert_eq!(
        messages[1]
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("run_failed"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn incomplete_named_scope_enters_main_agent_tool_loop_without_auxiliary_gate() {
    let root = make_temp_dir("hone_channels_agent_entity_discovery");
    std::fs::create_dir_all(&root).expect("create root");
    let quote_timestamp = chrono::Utc::now().timestamp() - 60;
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![
        (
            "query=NBIS".to_string(),
            serde_json::json!([{
                "symbol": "NBIS",
                "name": "Nebius Group N.V.",
                "exchangeShortName": "NASDAQ"
            }]),
        ),
        (
            "/v3/search?".to_string(),
            serde_json::json!([{
                "symbol": "NVDA",
                "name": "NVIDIA Corporation",
                "exchangeShortName": "NASDAQ"
            }]),
        ),
        (
            "/v3/quote/NBIS,NVDA".to_string(),
            serde_json::json!([
                {"symbol":"NBIS","price":177.71,"timestamp":quote_timestamp},
                {"symbol":"NVDA","price":180.25,"timestamp":quote_timestamp}
            ]),
        ),
        (
            "/v3/profile/NBIS".to_string(),
            serde_json::json!([{
                "symbol":"NBIS","companyName":"Nebius Group N.V.",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
        (
            "/v3/profile/NVDA".to_string(),
            serde_json::json!([{
                "symbol":"NVDA","companyName":"NVIDIA Corporation",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
        (
            "/v3/income-statement/NBIS".to_string(),
            serde_json::json!([{
                "symbol":"NBIS","calendarYear":"2025","period":"FY",
                "date":"2025-12-31","reportedCurrency":"USD",
                "revenue":1000000000.0,"grossProfit":300000000.0,
                "netIncome":100000000.0,"epsdiluted":1.25
            }]),
        ),
        (
            "/v3/income-statement/NVDA".to_string(),
            serde_json::json!([{
                "symbol":"NVDA","calendarYear":"2025","period":"FY",
                "date":"2025-12-31","reportedCurrency":"USD",
                "revenue":2000000000.0,"grossProfit":900000000.0,
                "netIncome":600000000.0,"epsdiluted":2.5
            }]),
        ),
    ]);
    let llm = MockLlmProvider::with_chat_and_tool_responses(
        vec![Ok(ChatResult {
            content: "not valid entity json".to_string(),
            usage: None,
        })],
        vec![
            Ok(ChatResponse {
                content: String::new(),
                reasoning_content: Some("先在主循环核验全部当前标的".to_string()),
                tool_calls: Some(vec![
                    ToolCall {
                        id: "call_nbis".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"search","query":"NBIS"}"#.to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_nvidia".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"search","query":"英伟达"}"#.to_string(),
                        },
                    },
                ]),
                usage: None,
            }),
            Ok(ChatResponse {
                content: String::new(),
                reasoning_content: Some("实体 search 已返回，继续核验同代码行情和资产类型".to_string()),
                tool_calls: Some(vec![
                    ToolCall {
                        id: "call_quote".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"quote","ticker":"NBIS,NVDA"}"#.to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_nbis_profile".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"profile","ticker":"NBIS"}"#.to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_nvda_profile".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                        },
                    },
                ]),
                usage: None,
            }),
            Ok(ChatResponse {
                content: String::new(),
                reasoning_content: Some("实体和行情已确认，按用户比较问题补齐逐标的年度财务".to_string()),
                tool_calls: Some(vec![
                    ToolCall {
                        id: "call_nbis_financials".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"financials","ticker":"NBIS"}"#.to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_nvda_financials".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"financials","ticker":"NVDA"}"#.to_string(),
                        },
                    },
                ]),
                usage: None,
            }),
            Ok(ChatResponse {
                content: "数据时间：北京时间 2026-07-18 21:05；行情口径：报价源最新可得、非逐笔\n\n比较结论：NBIS 与 NVDA 应按不同业务成熟度比较。以下区分已核验事实与情景推断。\n### NBIS\nNBIS 当前价 177.71 USD。已核验事实：年度营收与净利润字段由本轮利润表覆盖；估值方法采用 P/S 与情景法，经营兑现是假设推断。\n### NVDA\nNVDA 当前价 180.25 USD。已核验事实：年度营收与净利润字段由本轮利润表覆盖；估值方法采用 P/E 与情景法，增长持续性是假设推断。\n风险与证伪条件：若增长与现金流趋势恶化，当前判断失效。\n动作建议与触发条件：先观察，等待估值与经营数据同时满足条件。".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            }),
        ],
    );
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor = ActorIdentity::new("web", "entity-malformed", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core, actor, "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("比较 NBIS 和英伟达", AgentRunOptions::default())
        .await;

    assert!(result.response.success, "{:?}", result.response.error);
    assert!(
        result.response.content.starts_with("数据时间：北京时间 "),
        "{}; calls={:?}",
        result.response.content,
        result.response.tool_calls_made
    );
    assert_eq!(result.response.tool_calls_made.len(), 7);
    assert!(
        result
            .response
            .tool_calls_made
            .iter()
            .all(|call| call.name.contains("data_fetch"))
    );
    assert_eq!(
        llm.chat_calls(),
        0,
        "blocking auxiliary chat must be removed"
    );
    assert_eq!(llm.chat_with_tools_calls(), 4);
    let runner_prompt = llm.last_tool_transcript();
    assert!(
        runner_prompt.contains("主 Agent 工具循环"),
        "{runner_prompt}"
    );
    assert!(
        runner_prompt.contains("data_fetch(search)"),
        "{runner_prompt}"
    );
    assert!(runner_prompt.contains("NBIS"), "{runner_prompt}");
    assert!(runner_prompt.contains("英伟达"), "{runner_prompt}");
    let events = listener.events.lock().await;
    assert!(!events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::Progress {
            stage: "entity_resolution.preflight" | "entity_resolution.preflight.done",
            ..
        })
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::Progress {
            stage: "agent.run",
            ..
        })
    )));
    let visible_deltas = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_deltas.len(), 1, "{visible_deltas:?}");
    assert_eq!(visible_deltas[0], &result.response.content);
    fmp_stub.join().expect("join FMP stub");
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn agent_owned_no_coverage_clarification_is_not_replaced_and_is_emitted_once() {
    let root = make_temp_dir("hone_channels_agent_no_coverage_clarification");
    std::fs::create_dir_all(&root).expect("create root");
    let (fmp_base_url, fmp_stub) =
        spawn_fmp_route_stub(vec![("query=ZZZQ".to_string(), serde_json::json!([]))]);
    let clarification = "我已用 DataFetch 搜索 ZZZQ，但本轮权威数据源没有返回可核验的证券候选，因此现在不能确认它对应哪家公司。请补充交易所或公司全名，我再继续核验。";
    let llm = MockLlmProvider::with_tool_responses(vec![
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("先让证券数据源确认用户写的代码".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_zzzq_search".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"search","query":"ZZZQ"}"#.to_string(),
                },
            }]),
            usage: None,
        },
        ChatResponse {
            content: clarification.to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        },
    ]);
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor = ActorIdentity::new("web", "agent-no-coverage", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core, actor, "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("ZZZQ 这只到底是什么？", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, clarification);
    assert!(
        !result
            .response
            .content
            .contains("本轮主 Agent 已进入证券核验流程"),
        "the service must preserve the Agent's concrete no-coverage explanation"
    );
    assert_eq!(result.response.tool_calls_made.len(), 1);
    assert_eq!(
        result.response.tool_calls_made[0].result["data"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
    assert_eq!(llm.chat_with_tools_calls(), 2);
    let events = listener.events.lock().await;
    let visible_deltas = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_deltas, vec![clarification]);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn agent_owned_equal_candidate_clarification_is_not_replaced_and_is_emitted_once() {
    let root = make_temp_dir("hone_channels_agent_equal_candidate_clarification");
    std::fs::create_dir_all(&root).expect("create root");
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![(
        "query=Alpha".to_string(),
        serde_json::json!([
            {
                "symbol":"ALPH",
                "name":"Alpha Group International plc",
                "exchangeShortName":"LSE"
            },
            {
                "symbol":"ALPHA",
                "name":"Alpha Services and Holdings S.A.",
                "exchangeShortName":"ATH"
            }
        ]),
    )]);
    let clarification = "DataFetch 对 Alpha 返回了两个同等可行候选：ALPH（伦敦）和 ALPHA（雅典）。你指的是哪一个？确认后我再拉取对应代码的行情。";
    let llm = MockLlmProvider::with_tool_responses(vec![
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("先搜索用户给出的公司简称".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_alpha_search".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"search","query":"Alpha"}"#.to_string(),
                },
            }]),
            usage: None,
        },
        ChatResponse {
            content: clarification.to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        },
    ]);
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor = ActorIdentity::new("web", "agent-equal-candidates", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core, actor, "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("Alpha 这个标的帮我捋一捋", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, clarification);
    assert!(
        !result
            .response
            .content
            .contains("本轮主 Agent 已进入证券核验流程"),
        "the service must preserve the Agent's candidate-specific clarification"
    );
    assert_eq!(result.response.tool_calls_made.len(), 1);
    assert_eq!(
        result.response.tool_calls_made[0].result["data"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(llm.chat_with_tools_calls(), 2);
    let events = listener.events.lock().await;
    let visible_deltas = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_deltas, vec![clarification]);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn optional_agent_observation_preserves_completed_interactive_answer() {
    let root = make_temp_dir("hone_channels_optional_agent_contract_failure");
    std::fs::create_dir_all(&root).expect("create root");
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![
        (
            "query=CRWV".to_string(),
            serde_json::json!([{
                "symbol":"CRWV",
                "name":"CoreWeave, Inc.",
                "exchangeShortName":"NASDAQ"
            }]),
        ),
        (
            "/v3/quote/CRWV".to_string(),
            serde_json::json!([{
                "symbol":"CRWV",
                "price":73.21
            }]),
        ),
    ]);
    let answer = "数据时间：北京时间 2026-07-18 21:05；行情口径：本轮报价缺少报价源时间\n\nDataFetch 已确认 CRWV 对应 CoreWeave；但本轮报价缺少可用的报价源时间，所以我不把 73.21 称为实时价。估值仍可从收入增速、毛利率、资本开支、融资成本和 Forward P/S 情景入手，并把数据缺口明确列为限制。";
    let llm = MockLlmProvider::with_tool_responses(vec![
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("先核验显式 ticker".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_crwv_search_optional_contract".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            }]),
            usage: None,
        },
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("候选已确认，读取同代码报价".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "call_crwv_quote_without_timestamp".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"quote","ticker":"CRWV"}"#.to_string(),
                },
            }]),
            usage: None,
        },
        ChatResponse {
            content: answer.to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        },
    ]);
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor =
        ActorIdentity::new("web", "optional-agent-contract", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core.clone(), actor.clone(), "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("CRWV 估值怎么看", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, answer);
    assert_eq!(result.response.error, None);
    assert_eq!(result.response.tool_calls_made.len(), 2);
    assert_eq!(llm.chat_with_tools_calls(), 3);
    let messages = core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .expect("messages");
    assert_eq!(
        messages
            .iter()
            .map(|message| message.role.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant"]
    );
    assert_eq!(messages[1].content[0].text.as_deref(), Some(answer));
    assert_ne!(
        messages[1]
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("run_failed"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    let events = listener.events.lock().await;
    let visible_deltas = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_deltas, vec![answer]);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn interactive_observed_crwv_nvidia_answer_is_never_repaired_or_rewritten() {
    let root = make_temp_dir("hone_channels_crwv_nvidia_observational_contract");
    std::fs::create_dir_all(&root).expect("create root");
    let core = make_test_core(&root, MockLlmProvider::with_chat_responses(Vec::new()));
    let actor =
        ActorIdentity::new("web", "crwv-nvidia-observational", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");
    let timestamp = chrono::Utc::now().timestamp() - 60;
    let answer = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n\nCRWV 与英伟达的关系是算力云客户/供应链关系。CRWV 本轮同代码现价 73.21 USD；在情景估值里可把当前价约 73 USD 作为近似锚点。后续估值结论由 Agent 按本轮工具上下文展开。";
    let tool_call = |id: &str, arguments: Value, result: Value| ToolCallMade {
        name: "data_fetch".to_string(),
        arguments,
        result,
        tool_call_id: Some(id.to_string()),
    };
    let runs = Arc::new(Mutex::new(std::collections::VecDeque::from(vec![
        MockStreamingRun {
            events: vec![
                AgentRunnerEvent::Progress {
                    stage: "agent.run",
                    detail: Some("observed interactive run".to_string()),
                },
                AgentRunnerEvent::StreamDelta {
                    content: answer.to_string(),
                },
                AgentRunnerEvent::StreamReset,
                AgentRunnerEvent::Error {
                    error: super::types::AgentSessionError {
                        kind: AgentSessionErrorKind::AgentFailed,
                        message: "attempt-local event must stay hidden".to_string(),
                    },
                },
            ],
            result: AgentRunnerResult {
                response: AgentResponse {
                    content: answer.to_string(),
                    tool_calls_made: vec![
                        tool_call(
                            "search-crwv",
                            serde_json::json!({"data_type":"search","query":"CRWV"}),
                            serde_json::json!({"data":[{
                                "symbol":"CRWV",
                                "name":"CoreWeave, Inc.",
                                "exchangeShortName":"NASDAQ"
                            }]}),
                        ),
                        tool_call(
                            "quote-crwv",
                            serde_json::json!({"data_type":"quote","ticker":"CRWV"}),
                            serde_json::json!({"data":[{
                                "symbol":"CRWV",
                                "price":73.21,
                                "timestamp":timestamp
                            }]}),
                        ),
                        tool_call(
                            "profile-crwv",
                            serde_json::json!({"data_type":"profile","ticker":"CRWV"}),
                            serde_json::json!({"data":[{
                                "symbol":"CRWV",
                                "companyName":"CoreWeave, Inc.",
                                "exchangeShortName":"NASDAQ",
                                "currency":"USD",
                                "isEtf":false,
                                "isFund":false
                            }]}),
                        ),
                        // Reproduce the production history drift: NBIS was
                        // queried even though the current user named NVIDIA.
                        // An observational contract may diagnose this trace,
                        // but it must not repair or replace the Agent answer.
                        tool_call(
                            "quote-stale-nbis",
                            serde_json::json!({"data_type":"quote","ticker":"NBIS"}),
                            serde_json::json!({"data":[{
                                "symbol":"NBIS",
                                "price":177.71,
                                "timestamp":timestamp
                            }]}),
                        ),
                    ],
                    iterations: 4,
                    success: true,
                    error: None,
                },
                streamed_output: true,
                committed_visible_prefix: None,
                terminal_error_emitted: true,
                session_metadata_updates: HashMap::new(),
                context_messages: None,
            },
        },
    ])));
    let runtime_inputs = Arc::new(Mutex::new(Vec::new()));
    let runner = MockStreamingSequencedRunner {
        runs: runs.clone(),
        runtime_inputs: runtime_inputs.clone(),
    };
    let request = AgentRunnerRequest {
        session_id: "crwv-nvidia-observational-session".to_string(),
        actor_label: "web:crwv-nvidia-observational".to_string(),
        actor: session.actor.clone(),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: String::new(),
        runtime_dir: String::new(),
        system_prompt: "system".to_string(),
        runtime_input: "crwv和英伟达什么关系，估值怎么看".to_string(),
        context: AgentContext::new("crwv-nvidia-observational-session".to_string()),
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata: HashMap::new(),
        working_directory: root.display().to_string(),
        allowed_tools: None,
        max_tool_calls: None,
        terminal_stream_policy: Default::default(),
        tool_call_limits: None,
    };
    let downstream = Arc::new(RecordingRunnerEmitter::default());

    let result = session
        .run_runner_with_investment_contract_retry(
            &runner,
            "mock_streaming_sequenced",
            "crwv-nvidia-observational-session",
            request,
            downstream.clone(),
            None,
            PreparedTurnReexecutionPolicy::Allowed,
            Some("crwv和英伟达什么关系，估值怎么看"),
        )
        .await;

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, answer);
    assert!(result.response.error.is_none());
    assert!(!result.response.content.contains("投研完整性检查"));
    assert!(!result.response.content.contains("已停止发送"));
    assert!(!result.response.content.contains("请稍后重试"));
    assert!(result.response.content.contains("73.21 USD"));
    assert!(result.response.content.contains("约 73 USD"));
    assert!(result.response.tool_calls_made.iter().any(|call| {
        call.tool_call_id.as_deref() == Some("quote-stale-nbis")
            && call.arguments["ticker"] == "NBIS"
    }));
    assert_eq!(runtime_inputs.lock().expect("runtime inputs").len(), 1);
    assert!(runs.lock().expect("runs").is_empty());
    assert!(!result.streamed_output);
    assert!(!result.terminal_error_emitted);
    let events = downstream.events.lock().await;
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], AgentRunnerEvent::Progress { .. }));
    assert!(!events.iter().any(|event| matches!(
        event,
        AgentRunnerEvent::StreamDelta { .. }
            | AgentRunnerEvent::StreamReset
            | AgentRunnerEvent::Error { .. }
    )));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn interactive_tickers_enter_the_main_agent_loop_without_preflight_blocking() {
    let root = make_temp_dir("hone_channels_rklb_exact_entity_fast_path");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(vec![], vec![]);
    let core = make_test_core(&root, llm.clone());
    let actor = ActorIdentity::new("web", "rklb-exact", None::<String>).expect("actor");

    for input in [
        "分析下crwv和nbis的估值",
        "crwv和英伟达什么关系，估值怎么看",
        "想看看 CRWV 与 NBIS 到底谁更贵",
        "现在rklb推荐的安全区间价格是多少，暂不考虑中子",
        "现在RKLB推荐的安全区间价格是多少，暂不考虑中子发射时间，是否成功",
        "RKLB 是前面提到的 火箭实验室",
        "rklb 是前面提到的 火箭实验室",
    ] {
        let mut runtime_input = input.to_string();
        let contract = prepare_verified_investment_turn(
            &core,
            &actor,
            "direct",
            false,
            input,
            AgentTurnOrigin::Interactive,
            &mut runtime_input,
        )
        .await
        .expect("interactive entity discovery must not fail before the runner");
        assert!(
            contract.is_none(),
            "interactive entity discovery must remain inside the main agent loop: {input}"
        );
        assert!(
            runtime_input.contains("主 Agent 工具循环")
                && runtime_input.contains("第一轮工具调用")
                && runtime_input.contains("data_fetch")
                && runtime_input.contains("第一可见字符必须是“数”")
                && runtime_input.contains("数据时间：北京时间 ")
                && runtime_input.contains("；行情口径：")
                && runtime_input.contains("禁止在该行之前输出 `---`、Markdown 标题")
                && runtime_input.ends_with("不得以流程性拒答代替用户要的分析。"),
            "{input}: {runtime_input}"
        );
    }

    assert_eq!(
        llm.chat_calls(),
        0,
        "interactive entity discovery must not invoke an auxiliary model"
    );
    assert_eq!(llm.chat_with_tools_calls(), 0);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn crwv_nbis_agent_loop_batches_the_first_datafetch_and_emits_one_answer() {
    let root = make_temp_dir("hone_channels_crwv_nbis_agent_loop");
    std::fs::create_dir_all(&root).expect("create root");
    let quote_timestamp = chrono::Utc::now().timestamp() - 60;
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![
        (
            "query=CRWV".to_string(),
            serde_json::json!([
                {"symbol":"CRWV","name":"CoreWeave, Inc.","exchangeShortName":"NASDAQ"},
                {"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF","exchangeShortName":"NASDAQ","type":"etf"}
            ]),
        ),
        (
            "query=NBIS".to_string(),
            serde_json::json!([{
                "symbol":"NBIS","name":"Nebius Group N.V.","exchangeShortName":"NASDAQ"
            }]),
        ),
        (
            "/v3/quote/CRWV,NBIS".to_string(),
            serde_json::json!([
                {"symbol":"CRWV","price":73.21,"timestamp":quote_timestamp},
                {"symbol":"NBIS","price":177.71,"timestamp":quote_timestamp}
            ]),
        ),
        (
            "/v3/profile/CRWV".to_string(),
            serde_json::json!([{
                "symbol":"CRWV","companyName":"CoreWeave, Inc.",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
        (
            "/v3/profile/NBIS".to_string(),
            serde_json::json!([{
                "symbol":"NBIS","companyName":"Nebius Group N.V.",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
        (
            "/v3/income-statement/CRWV".to_string(),
            serde_json::json!([{
                "symbol":"CRWV","calendarYear":"2025","period":"FY",
                "date":"2025-12-31","reportedCurrency":"USD",
                "revenue":1000000000.0,"grossProfit":300000000.0,
                "netIncome":100000000.0,"epsdiluted":1.25
            }]),
        ),
        (
            "/v3/income-statement/NBIS".to_string(),
            serde_json::json!([{
                "symbol":"NBIS","calendarYear":"2025","period":"FY",
                "date":"2025-12-31","reportedCurrency":"USD",
                "revenue":1200000000.0,"grossProfit":360000000.0,
                "netIncome":120000000.0,"epsdiluted":1.5
            }]),
        ),
    ]);
    let llm = MockLlmProvider::with_tool_responses(vec![
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("先并行搜索两个当前 ticker".to_string()),
            tool_calls: Some(vec![
                ToolCall {
                    id: "call_crwv_search".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "call_nbis_search".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"NBIS"}"#.to_string(),
                    },
                },
            ]),
            usage: None,
        },
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("search 已确认候选，继续核验同代码行情和 profile".to_string()),
            tool_calls: Some(vec![
                ToolCall {
                    id: "call_crwv_nbis_quote".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"quote","ticker":"CRWV,NBIS"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "call_crwv_profile".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "call_nbis_profile".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"profile","ticker":"NBIS"}"#.to_string(),
                    },
                },
            ]),
            usage: None,
        },
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("按估值问题补齐两个公司的年度财务证据".to_string()),
            tool_calls: Some(vec![
                ToolCall {
                    id: "call_crwv_financials".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"financials","ticker":"CRWV"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "call_nbis_financials".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"financials","ticker":"NBIS"}"#.to_string(),
                    },
                },
            ]),
            usage: None,
        },
        ChatResponse {
            content: "数据时间：北京时间 2026-07-18 21:05；行情口径：报价源最新可得、非逐笔\n\n比较结论：CRWV 与 NBIS 的估值应结合各自增长和资本强度。以下区分已核验事实与情景推断。\n### CRWV\nCRWV 当前价 73.21 USD。已核验事实：年度营收与净利润字段由本轮利润表覆盖；估值方法采用 P/S 与情景法，订单兑现是假设推断。\n### NBIS\nNBIS 当前价 177.71 USD。已核验事实：年度营收与净利润字段由本轮利润表覆盖；估值方法采用 P/S 与情景法，算力利用率是假设推断。\n风险与证伪条件：若订单、增长或现金流明显恶化，当前判断失效。\n动作建议与触发条件：先观察，等待估值与经营数据同时满足条件。".to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        },
    ]);
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor = ActorIdentity::new("web", "crwv-nbis-agent-loop", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core, actor, "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("分析下crwv和nbis的估值", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert!(
        result.response.content.starts_with("数据时间：北京时间 "),
        "content={} calls={:?}",
        result.response.content,
        result.response.tool_calls_made
    );
    assert_eq!(result.response.tool_calls_made.len(), 7);
    let call = result
        .response
        .tool_calls_made
        .iter()
        .find(|call| call.arguments["data_type"] == "quote")
        .expect("batch quote call");
    assert!(call.name.contains("data_fetch"), "{}", call.name);
    assert_eq!(call.arguments["data_type"], "quote");
    assert_eq!(call.arguments["ticker"], "CRWV,NBIS");
    assert_eq!(call.result["data"][0]["symbol"], "CRWV");
    assert_eq!(call.result["data"][1]["symbol"], "NBIS");
    assert_eq!(call.result["data"][0]["price"], 73.21);
    assert_eq!(call.result["data"][1]["price"], 177.71);
    assert!(call.result["data"][0]["timestamp"].is_number());
    assert!(call.result["data"][1]["timestamp"].is_number());
    assert_eq!(llm.chat_calls(), 0);
    assert_eq!(llm.chat_with_tools_calls(), 4);
    let runner_prompt = llm.last_tool_transcript();
    assert!(runner_prompt.contains("CRWV"), "{runner_prompt}");
    assert!(runner_prompt.contains("NBIS"), "{runner_prompt}");

    let events = listener.events.lock().await;
    let visible_deltas = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_deltas, vec![&result.response.content]);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn omitted_explicit_seed_is_observational_and_does_not_rerun() {
    let root = make_temp_dir("hone_channels_agent_seed_observational");
    std::fs::create_dir_all(&root).expect("create root");
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![(
        "query=CRWV".to_string(),
        serde_json::json!([{
            "symbol":"CRWV","name":"CoreWeave, Inc.","exchangeShortName":"NASDAQ"
        }]),
    )]);
    let original_answer = "数据时间：北京时间 2026-07-18 21:05；行情口径：本轮尚未完成报价核验\n\n我只检查了 CRWV，尚未覆盖用户点名的 NBIS；这是本轮 Agent 的原始回答。";
    let llm = MockLlmProvider::with_tool_responses(vec![
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("首轮错误地只搜索了一个显式 ticker".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "initial_crwv_search".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "data_fetch".to_string(),
                    arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                },
            }]),
            usage: None,
        },
        ChatResponse {
            content: original_answer.to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        },
    ]);
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor =
        ActorIdentity::new("web", "agent-seed-observational", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core, actor, "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("分析下 CRWV 和 NBIS", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, original_answer);
    assert!(result.response.error.is_none());
    assert!(!result.response.content.contains("投研完整性检查"));
    assert_eq!(llm.chat_with_tools_calls(), 2);
    let transcript = llm.last_tool_transcript();
    assert!(
        !transcript.contains("主 Agent 实体发现自检"),
        "{transcript}"
    );
    assert_eq!(
        result
            .response
            .tool_calls_made
            .iter()
            .map(|call| call.tool_call_id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("initial_crwv_search")],
        "an incomplete observed trace must not authorize a second runner invocation"
    );

    let events = listener.events.lock().await;
    let visible_deltas = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_deltas, vec![&result.response.content]);
    assert!(!events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::StreamReset | RunEvent::Error { .. })
    )));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn single_agent_loop_accepts_later_exact_searches_after_empty_enriched_searches() {
    let root = make_temp_dir("hone_channels_agent_search_refinement_single_loop");
    std::fs::create_dir_all(&root).expect("create root");
    let quote_timestamp = chrono::Utc::now().timestamp() - 60;
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![
        ("query=CRWV+CoreWeave".to_string(), serde_json::json!([])),
        ("query=NBIS+Nebius".to_string(), serde_json::json!([])),
        (
            "query=CRWV".to_string(),
            serde_json::json!([
                {"symbol":"CRWV","name":"CoreWeave, Inc.","exchangeShortName":"NASDAQ"},
                {"symbol":"CWY","name":"GraniteShares YieldBOOST CRWV ETF","exchangeShortName":"NASDAQ"}
            ]),
        ),
        (
            "query=NBIS".to_string(),
            serde_json::json!([
                {"symbol":"NBIS","name":"Nebius Group N.V.","exchangeShortName":"NASDAQ"},
                {"symbol":"NBIZ","name":"T-Rex 2X Long NBIS Daily Target ETF","exchangeShortName":"CBOE"}
            ]),
        ),
        (
            "/v3/quote/CRWV,NBIS".to_string(),
            serde_json::json!([
                {"symbol":"CRWV","price":73.21,"timestamp":quote_timestamp},
                {"symbol":"NBIS","price":177.71,"timestamp":quote_timestamp}
            ]),
        ),
        (
            "/v3/profile/CRWV".to_string(),
            serde_json::json!([{
                "symbol":"CRWV","companyName":"CoreWeave, Inc.",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
        (
            "/v3/profile/NBIS".to_string(),
            serde_json::json!([{
                "symbol":"NBIS","companyName":"Nebius Group N.V.",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
    ]);
    let accepted_answer = "数据时间：北京时间 2026-07-18 21:05；行情口径：报价源最新可得、非逐笔\n\n## 估值结论\nCRWV 当前价 73.21 USD；NBIS 当前价 177.71 USD。两者应分别结合增长、毛利、资本开支和 Forward P/S 情景比较。";
    let llm = MockLlmProvider::with_tool_responses(vec![
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("先尝试带公司名的探索搜索".to_string()),
            tool_calls: Some(vec![
                ToolCall {
                    id: "refine_search_crwv_enriched".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"CRWV CoreWeave"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "refine_search_nbis_enriched".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"NBIS Nebius"}"#.to_string(),
                    },
                },
            ]),
            usage: None,
        },
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("探索结果为空，改用原始 ticker 精确搜索".to_string()),
            tool_calls: Some(vec![
                ToolCall {
                    id: "refine_search_crwv_exact".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"CRWV"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "refine_search_nbis_exact".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"NBIS"}"#.to_string(),
                    },
                },
            ]),
            usage: None,
        },
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("精确候选已返回，核验同代码行情与资产类型".to_string()),
            tool_calls: Some(vec![
                ToolCall {
                    id: "refine_quote_crwv_nbis".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"quote","ticker":"CRWV,NBIS"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "refine_profile_crwv".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"profile","ticker":"CRWV"}"#.to_string(),
                    },
                },
                ToolCall {
                    id: "refine_profile_nbis".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"profile","ticker":"NBIS"}"#.to_string(),
                    },
                },
            ]),
            usage: None,
        },
        ChatResponse {
            content: String::new(),
            reasoning_content: Some("证据已齐，进入唯一终稿阶段".to_string()),
            tool_calls: Some(vec![ToolCall {
                id: "finish_refined_crwv_nbis_research".to_string(),
                call_type: "function".to_string(),
                function: FunctionCall {
                    name: "finish_research".to_string(),
                    arguments: "{}".to_string(),
                },
            }]),
            usage: None,
        },
        ChatResponse {
            content: accepted_answer.to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        },
    ]);
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor =
        ActorIdentity::new("web", "agent-search-refinement", None::<String>).expect("actor");
    let listener = Arc::new(RecordingListener::default());
    let mut session = AgentSession::new(core.clone(), actor.clone(), "direct");
    session.add_listener(listener.clone());

    let result = session
        .run("分析下crwv和nbis的估值", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert!(result.response.content.starts_with("数据时间：北京时间 "));
    assert!(result.response.content.contains("CRWV 当前价 73.21 USD"));
    assert!(result.response.content.contains("NBIS 当前价 177.71 USD"));
    assert_eq!(result.response.content, accepted_answer);
    assert_eq!(llm.chat_with_tools_calls(), 5);
    assert_eq!(result.response.tool_calls_made.len(), 7);
    assert!(
        build_agent_discovered_investment(
            "分析下crwv和nbis的估值",
            AgentTurnOrigin::Interactive,
            &result.response.tool_calls_made,
        )
        .is_some()
    );
    let transcript = llm.last_tool_transcript();
    assert!(
        !transcript.contains("主 Agent 实体发现自检"),
        "{transcript}"
    );
    let events = listener.events.lock().await;
    let visible_chunks = events
        .iter()
        .filter_map(|event| match event {
            AgentSessionEvent::Run(RunEvent::StreamDelta { content }) => Some(content),
            AgentSessionEvent::Segment { text } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    let committed_header = format!(
        "{}\n",
        accepted_answer
            .lines()
            .next()
            .expect("canonical answer header")
    );
    assert_eq!(visible_chunks.len(), 2, "{visible_chunks:?}");
    assert_eq!(visible_chunks[0], &committed_header);
    assert_eq!(
        visible_chunks
            .iter()
            .map(|chunk| chunk.as_str())
            .collect::<String>(),
        result.response.content,
        "the early committed header plus the terminal tail must exactly equal the persisted answer"
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        AgentSessionEvent::Run(RunEvent::StreamReset | RunEvent::Error { .. })
    )));
    let messages = core
        .session_storage
        .get_messages(&actor.session_id(), None)
        .expect("persisted terminal messages");
    assert_eq!(
        messages
            .iter()
            .map(|message| message.role.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant"]
    );
    assert_eq!(
        messages[1].content[0].text.as_deref(),
        Some(result.response.content.as_str())
    );
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn scheduled_cross_market_tickers_bypass_auxiliary_entity_chat() {
    let root = make_temp_dir("hone_channels_scheduled_cross_market_entity_fast_path");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(vec![], vec![]);
    let core = make_test_core(&root, llm.clone());
    let actor =
        ActorIdentity::new("web", "scheduled-symbol-fast-path", None::<String>).expect("actor");

    for (input, origin) in [
        ("每30分钟检查 ASTS 股价", AgentTurnOrigin::Scheduled),
        ("检查 605259.SH 股价", AgentTurnOrigin::Heartbeat),
        ("检查 BRK.B 股价", AgentTurnOrigin::Scheduled),
        ("检查 0700.HK 股价", AgentTurnOrigin::Heartbeat),
    ] {
        let mut runtime_input = input.to_string();
        let error = prepare_verified_investment_turn(
            &core,
            &actor,
            "direct",
            false,
            input,
            origin,
            &mut runtime_input,
        )
        .await
        .expect_err("test config has no FMP key, so deterministic DataFetch must fail");
        assert!(error.contains("证券数据源本轮查询失败"), "{input}: {error}");
        assert!(
            !error.contains("证券实体解析暂时未能确认"),
            "{input}: {error}"
        );
    }

    assert_eq!(llm.chat_calls(), 0);
    assert_eq!(llm.chat_with_tools_calls(), 0);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn named_entity_scope_is_delegated_to_the_main_agent_instead_of_preflight_clarification() {
    let root = make_temp_dir("hone_channels_named_entity_agent_discovery");
    std::fs::create_dir_all(&root).expect("create root");
    let quote_timestamp = chrono::Utc::now().timestamp() - 60;
    let (fmp_base_url, fmp_stub) = spawn_fmp_route_stub(vec![
        (
            "/v3/search?".to_string(),
            serde_json::json!([{
                "symbol": "NVDA",
                "name": "NVIDIA Corporation",
                "exchangeShortName": "NASDAQ"
            }]),
        ),
        (
            "/v3/quote/NVDA".to_string(),
            serde_json::json!([{
                "symbol":"NVDA","price":180.25,"timestamp":quote_timestamp
            }]),
        ),
        (
            "/v3/profile/NVDA".to_string(),
            serde_json::json!([{
                "symbol":"NVDA","companyName":"NVIDIA Corporation",
                "exchangeShortName":"NASDAQ","currency":"USD",
                "isEtf":false,"isFund":false
            }]),
        ),
    ]);
    let llm = MockLlmProvider::with_chat_and_tool_responses(
        vec![Ok(ChatResult {
            content: r#"{"entities":[],"unresolved_mentions":["英伟达"]}"#.to_string(),
            usage: None,
        })],
        vec![
            Ok(ChatResponse {
                content: String::new(),
                reasoning_content: Some("先核验中文公司名".to_string()),
                tool_calls: Some(vec![ToolCall {
                    id: "call_nvidia_name".to_string(),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: "data_fetch".to_string(),
                        arguments: r#"{"data_type":"search","query":"英伟达"}"#.to_string(),
                    },
                }]),
                usage: None,
            }),
            Ok(ChatResponse {
                content: String::new(),
                reasoning_content: Some("search 已返回，继续核验 NVDA 行情和 profile".to_string()),
                tool_calls: Some(vec![
                    ToolCall {
                        id: "call_nvda_quote".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"quote","ticker":"NVDA"}"#.to_string(),
                        },
                    },
                    ToolCall {
                        id: "call_nvda_profile".to_string(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: "data_fetch".to_string(),
                            arguments: r#"{"data_type":"profile","ticker":"NVDA"}"#.to_string(),
                        },
                    },
                ]),
                usage: None,
            }),
            Ok(ChatResponse {
                content: "数据时间：北京时间 2026-07-18 21:05；行情口径：报价源最新可得、非逐笔\n\nNVDA 当前价 180.25 USD。".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            }),
        ],
    );
    let core = make_strict_tool_loop_test_core_with_config(&root, llm.clone(), |config| {
        config.fmp.api_keys = vec!["test-key".to_string()];
        config.fmp.base_url = fmp_base_url;
    });
    let actor = ActorIdentity::new("web", "entity-empty", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");

    let result = session
        .run("英伟达当前价", AgentRunOptions::default())
        .await;
    fmp_stub.join().expect("join FMP stub");

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(llm.chat_calls(), 0);
    assert_eq!(llm.chat_with_tools_calls(), 3);
    assert_eq!(result.response.tool_calls_made.len(), 3);
    assert!(
        build_agent_discovered_investment(
            "英伟达当前价",
            AgentTurnOrigin::Interactive,
            &result.response.tool_calls_made,
        )
        .is_some(),
        "direct dynamic contract build failed: {:?}",
        result.response.tool_calls_made
    );
    assert!(
        result.response.content.starts_with("数据时间：北京时间 "),
        "content={} calls={:?}",
        result.response.content,
        result.response.tool_calls_made
    );
    assert!(result.response.tool_calls_made.iter().any(|call| {
        call.arguments["data_type"] == "quote" && call.result["data"][0]["symbol"] == "NVDA"
    }));
    let runner_prompt = llm.last_tool_transcript();
    assert!(runner_prompt.contains("英伟达"), "{runner_prompt}");
    assert!(
        runner_prompt.contains("data_fetch(search)"),
        "{runner_prompt}"
    );
    assert!(
        !runner_prompt.contains("解析暂时未能确认"),
        "{runner_prompt}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn valid_empty_entity_payload_allows_an_ordinary_finance_question() {
    let root = make_temp_dir("hone_channels_ordinary_finance_empty_entity");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(
        vec![Ok(ChatResult {
            content: r#"{"entities":[],"unresolved_mentions":[]}"#.to_string(),
            usage: None,
        })],
        vec![Ok(ChatResponse {
            content: "安全边际是价格相对保守价值估计留下的缓冲。".to_string(),
            reasoning_content: None,
            tool_calls: None,
            usage: None,
        })],
    );
    let core = make_test_core(&root, llm.clone());
    let actor = ActorIdentity::new("web", "ordinary-finance", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");

    let result = session
        .run("什么是安全边际", AgentRunOptions::default())
        .await;

    assert!(result.response.success, "{:?}", result.response.error);
    assert!(result.response.content.contains("安全边际"));
    assert!(!result.response.content.contains("补充公司全名或 ticker"));
    assert_eq!(llm.chat_calls(), 0);
    assert_eq!(llm.chat_with_tools_calls(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn interactive_portfolio_wording_stays_inside_the_main_agent_tool_loop() {
    let root = make_temp_dir("hone_channels_portfolio_scope_agent_loop");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(vec![], vec![]);
    let core = make_test_core(&root, llm.clone());
    let actor = ActorIdentity::new("web", "portfolio-preflight", None::<String>).expect("actor");
    let mut runtime_input = "帮我看持仓".to_string();

    let contract = prepare_verified_investment_turn(
        &core,
        &actor,
        "direct",
        false,
        "帮我看持仓",
        AgentTurnOrigin::Interactive,
        &mut runtime_input,
    )
    .await
    .expect("interactive portfolio wording must reach the main agent");

    assert!(contract.is_none());
    assert!(
        runtime_input.contains("本轮证券实体发现：主 Agent 工具循环"),
        "{runtime_input}"
    );
    assert!(
        runtime_input.contains("完整阅读当前用户请求"),
        "{runtime_input}"
    );
    assert!(!runtime_input.contains("服务端已经执行只读 portfolio view"));
    assert_eq!(llm.chat_calls(), 0);
    assert_eq!(llm.chat_with_tools_calls(), 0);
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
                reasoning_content: None,
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
async fn run_short_circuits_obvious_non_finance_direct_query_without_llm_or_quota() {
    let root = make_temp_dir("hone_channels_domain_boundary");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_tool_responses(vec![ChatResponse {
        content: "should not be called".to_string(),
        reasoning_content: None,
        tool_calls: None,
        usage: None,
    }]);
    let core = make_test_core(&root, llm.clone());
    let actor = ActorIdentity::new("feishu", "alice", None::<String>).expect("actor");
    let session = AgentSession::new(core.clone(), actor.clone(), actor.user_id.clone());

    let result = session
        .run(
            "Hi hone，你了解深圳楼市吗？我现在是否适合买房？",
            AgentRunOptions::default(),
        )
        .await;

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, NON_FINANCE_BOUNDARY_REPLY);
    assert_eq!(llm.chat_calls(), 0);
    assert_eq!(llm.chat_with_tools_calls(), 0);

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
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(
        session_message_text(&messages[0]),
        "Hi hone，你了解深圳楼市吗？我现在是否适合买房？"
    );
    assert_eq!(
        session_message_text(&messages[1]),
        NON_FINANCE_BOUNDARY_REPLY
    );

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
                reasoning_content: None,
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
async fn context_window_exceeded_key_auto_compacts_and_retries_successfully() {
    let root = make_temp_dir("hone_channels_context_window_exceeded_key_retry_success");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(
        vec![Ok(ChatResult {
            content: "压缩后的摘要".to_string(),
            usage: None,
        })],
        vec![
            Err(hone_core::HoneError::Llm(
                "codex_error_info=context_window_exceeded".to_string(),
            )),
            Ok(ChatResponse {
                content: "压缩后恢复".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            }),
        ],
    );
    let core = make_test_core(&root, llm.clone());
    let actor = ActorIdentity::new("web", "overflow-key", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");

    let result = session
        .run("请继续分析这个话题", AgentRunOptions::default())
        .await;

    assert!(result.response.success, "{:?}", result.response.error);
    assert_eq!(result.response.content, "压缩后恢复");
    assert_eq!(llm.chat_calls(), 1);
    assert_eq!(llm.chat_with_tools_calls(), 2);

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn execute_once_context_overflow_is_not_compacted_or_retried() {
    let root = make_temp_dir("hone_channels_execute_once_context_overflow_no_retry");
    std::fs::create_dir_all(&root).expect("create root");
    let llm = MockLlmProvider::with_chat_and_tool_responses(
        vec![Ok(ChatResult {
            content: "不应压缩".to_string(),
            usage: None,
        })],
        vec![
            Err(hone_core::HoneError::Llm(
                "codex_error_info=context_window_exceeded".to_string(),
            )),
            Ok(ChatResponse {
                content: "不应重试".to_string(),
                reasoning_content: None,
                tool_calls: None,
                usage: None,
            }),
        ],
    );
    let core = make_test_core(&root, llm.clone());
    let actor = ActorIdentity::new("web", "execute-once-overflow", None::<String>).expect("actor");
    let session = AgentSession::new(core, actor, "direct");

    let result = session
        .run("取消所有定时任务", AgentRunOptions::default())
        .await;

    assert!(!result.response.success);
    assert_eq!(
        result.response.error.as_deref(),
        Some(crate::tool_trace::PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE)
    );
    assert_eq!(llm.chat_with_tools_calls(), 1);
    assert_eq!(llm.chat_calls(), 0);

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
            reasoning_content: None,
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
            reasoning_content: None,
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
        reasoning_content: None,
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

        let streamed_output = stream_gemini_prompt(
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
        assert!(streamed_output.contains("第一段"));
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
#[tokio::test]
async fn stream_gemini_prompt_bounds_exit_stderr() {
    let long_tail = "x".repeat(600);
    let stderr = format!(
        "request failed https://api.test/path?api_key=secret&token=secret2 auth=Bearer bearer-secret {long_tail}"
    );
    let (root, script_path) = write_mock_gemini_script_with_stderr(&[], &stderr, 7);
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
        assert!(matches!(err.kind, AgentSessionErrorKind::ExitFailure));
        assert!(err.message.contains("api_key=<redacted>"));
        assert!(err.message.contains("token=<redacted>"));
        assert!(err.message.contains("Bearer <redacted>"));
        assert!(!err.message.contains("secret"));
        assert!(
            err.message.chars().count() < 520,
            "stderr detail should be bounded: {}",
            err.message
        );
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
            == "Edit公司画像 and <absolute-path>/private.txt"
    ));
    assert!(matches!(
        &events[1],
        AgentSessionEvent::Run(RunEvent::ToolStatus {
            tool,
            message: Some(message),
            reasoning: Some(reasoning),
            ..
        }) if tool == "hone/skill_tool"
            && message == "Edit公司画像"
            && reasoning == "Edit data/research/notes.md and <absolute-path>/passwd"
    ));
}

#[tokio::test]
async fn session_event_emitter_suppresses_permission_progress_payloads() {
    let root = "/Users/fengming2/Desktop/honeclaw";
    let listener = Arc::new(RecordingListener::default());
    let emitter = SessionEventEmitter {
        listeners: vec![listener.clone()],
        channel: "feishu".to_string(),
        user_id: "ou_redacted".to_string(),
        session_id: "session".to_string(),
        message_id: None,
        working_directory: root.to_string(),
    };

    emitter
        .emit(AgentRunnerEvent::Progress {
            stage: "acp.permission",
            detail: Some("codex:approved-for-session:Approve MCP tool call".to_string()),
        })
        .await;

    let events = listener.events.lock().await.clone();
    assert!(matches!(
        &events[0],
        AgentSessionEvent::Run(RunEvent::Progress { detail: None, .. })
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

#[tokio::test]
async fn session_event_emitter_suppresses_internal_stream_delta_payloads() {
    let root = "/Users/fengming2/Desktop/honeclaw";
    let listener = Arc::new(RecordingListener::default());
    let emitter = SessionEventEmitter {
        listeners: vec![listener.clone()],
        channel: "feishu".to_string(),
        user_id: "ou_redacted".to_string(),
        session_id: "session".to_string(),
        message_id: None,
        working_directory: root.to_string(),
    };

    emitter
        .emit(AgentRunnerEvent::StreamDelta {
            content: "【Invoked Skill Context】\nBase directory for this skill: /Users/fengming2/Desktop/honeclaw/skills/scheduled_task\nrawOutput={\"job\":\"secret\"}".to_string(),
        })
        .await;

    assert!(listener.events.lock().await.is_empty());
}

#[tokio::test]
async fn session_event_emitter_keeps_visible_stream_delta_prefix_before_internal_payload() {
    let root = "/Users/fengming2/Desktop/honeclaw";
    let listener = Arc::new(RecordingListener::default());
    let emitter = SessionEventEmitter {
        listeners: vec![listener.clone()],
        channel: "feishu".to_string(),
        user_id: "ou_redacted".to_string(),
        session_id: "session".to_string(),
        message_id: None,
        working_directory: root.to_string(),
    };

    emitter
        .emit(AgentRunnerEvent::StreamDelta {
            content:
                "OK\n【Invoked Skill Context】\nBase directory for this skill: /Users/fengming2/Desktop/honeclaw/skills/scheduled_task"
                    .to_string(),
        })
        .await;

    let events = listener.events.lock().await.clone();
    assert!(matches!(
        &events[0],
        AgentSessionEvent::Run(RunEvent::StreamDelta { content }) if content == "OK"
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
