//! `acp_common` 的回归测试。覆盖:
//! - codex 字面量 compact chunk 被标记但不丢文本;
//! - opencode 用 usage 骤降 / summary boundary 识别 compact;
//! - 多字节 prefix 下跨 chunk boundary 扫描不 panic;
//! - `acp-events.log` 能把 identity grep 出来;
//! - permission request 响应走 `process_acp_payload` 的默认分支;
//! - `acp_prompt_succeeded` 语义。

use async_trait::async_trait;
use serde_json::json;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::runners::types::{AgentRunnerEmitter, AgentRunnerEvent};

use super::ingest::{acp_prompt_succeeded, handle_acp_session_update, ingest_acp_message_chunk};
use super::log::{AcpEventLogContext, acp_event_log_path, log_acp_payload};
use super::protocol::process_acp_payload;
use super::state::{AcpPermissionDecision, AcpPromptState};

struct CollectingEmitter {
    deltas: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl AgentRunnerEmitter for CollectingEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        if let AgentRunnerEvent::StreamDelta { content } = event {
            self.deltas.lock().await.push(content);
        }
    }
}

fn collecting_emitter() -> (Arc<dyn AgentRunnerEmitter>, Arc<Mutex<Vec<String>>>) {
    let deltas = Arc::new(Mutex::new(Vec::new()));
    let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(CollectingEmitter {
        deltas: deltas.clone(),
    });
    (emitter, deltas)
}

/// codex-acp 内置 compact 触发后单独发一条 `agent_message_chunk text="Context compacted\n"`。
/// 现在这类文本会继续透传给用户，但仍需在 state 上标记 compact_detected=true，
/// 供 runner 写回 metadata 触发下一轮 SP reseed。
#[tokio::test]
async fn handle_acp_session_update_marks_codex_compact_literal_chunk_without_dropping_output() {
    let mut state = AcpPromptState::default();
    let (emitter, deltas) = collecting_emitter();
    let params = json!({
        "update": {
            "sessionUpdate": "agent_message_chunk",
            "content": { "text": "Context compacted\n" }
        }
    });
    handle_acp_session_update(&params, &emitter, Some(&mut state)).await;
    assert!(state.compact_detected, "compact_detected should be set");
    assert!(
        state.full_reply == "Context compacted\n",
        "compact literal should stay in full_reply, got: {:?}",
        state.full_reply
    );
    assert!(
        deltas.lock().await.clone() == vec!["Context compacted\n".to_string()],
        "compact literal should be forwarded to sink"
    );
}

/// opencode 不推 compact 字面量，但内置 compact 后 session 体积 >50% 骤降。
/// 我们以"流内首次 usage_update.used 与上轮 peak 比下降超 50%"作为信号，
/// 同样置 compact_detected=true。
#[tokio::test]
async fn handle_acp_session_update_detects_opencode_compact_via_usage_drop() {
    // 模拟上一轮结束时 peak_used = 200_000
    let mut state = AcpPromptState {
        prev_prompt_peak_used: Some(200_000),
        ..AcpPromptState::default()
    };
    let (emitter, _) = collecting_emitter();

    // 本轮第一条 usage_update：used=10_000，远低于 prev_peak/2 (=100_000)
    let drop_event = json!({
        "update": {
            "sessionUpdate": "usage_update",
            "used": 10_000u64,
            "size": 256_000u64
        }
    });
    handle_acp_session_update(&drop_event, &emitter, Some(&mut state)).await;
    assert!(
        state.compact_detected,
        "usage drop from 200K to 10K should signal compact"
    );
    assert!(state.usage_update_seen);
    assert_eq!(state.current_prompt_peak_used, 10_000);
}

/// 平稳增长的 usage_update 不应误判为 compact（避免假阳性）。
#[tokio::test]
async fn handle_acp_session_update_no_compact_on_normal_usage_growth() {
    let mut state = AcpPromptState {
        prev_prompt_peak_used: Some(50_000),
        ..AcpPromptState::default()
    };
    let (emitter, _) = collecting_emitter();

    let event = json!({
        "update": {
            "sessionUpdate": "usage_update",
            "used": 60_000u64,
            "size": 256_000u64
        }
    });
    handle_acp_session_update(&event, &emitter, Some(&mut state)).await;
    assert!(
        !state.compact_detected,
        "monotonic growth should not trigger compact"
    );
}

/// opencode compact 后会把 markdown summary 拼到本轮真实回复后面，
/// 如 `OK\n---\n## Goal\n...`。现在不再切掉这段用户可见文本，但仍需识别
/// compact 已发生并打上 compact_detected。
#[tokio::test]
async fn handle_acp_session_update_keeps_opencode_summary_after_boundary() {
    let mut state = AcpPromptState::default();
    let (emitter, deltas) = collecting_emitter();

    let event = json!({
        "update": {
            "sessionUpdate": "agent_message_chunk",
            "content": { "text": "OK\n---\n## Goal\n- placeholder\n## Constraints\n- none" }
        }
    });
    handle_acp_session_update(&event, &emitter, Some(&mut state)).await;
    assert_eq!(
        state.full_reply, "OK\n---\n## Goal\n- placeholder\n## Constraints\n- none",
        "summary should remain in full_reply"
    );
    let captured = deltas.lock().await.clone();
    assert_eq!(
        captured,
        vec!["OK\n---\n## Goal\n- placeholder\n## Constraints\n- none".to_string()]
    );
    assert!(state.compact_detected);
}

/// opencode 实测会把 `\n---\n## Goal` 边界拆到多个 `agent_message_chunk` 里
/// （e.g. `"OK\n"`, `"---\n"`, `"## "`, `" Goal\n- placeholder"`）。
/// 单 chunk 上 regex 必漏，必须扫累积 buffer。命中后只标记 compact_detected，
/// 不再回截 full_reply。
/// 注意：本场景 compact_detected **未**预置，早于 usage_update 收到 summary 是常态，
/// 必须能直接从 chunk 流自身识别。
#[tokio::test]
async fn ingest_acp_message_chunk_detects_boundary_split_across_chunks_without_truncating() {
    let mut state = AcpPromptState::default();
    let (emitter, deltas) = collecting_emitter();

    for piece in ["OK\n", "---\n", "## ", " Goal\n- placeholder\n"] {
        ingest_acp_message_chunk(piece, &mut state, &emitter).await;
    }

    assert_eq!(
        state.full_reply, "OK\n---\n##  Goal\n- placeholder\n",
        "cross-chunk boundary should preserve the original streamed text"
    );
    assert_eq!(
        state.pending_assistant_content, "OK\n---\n##  Goal\n- placeholder\n",
        "pending_assistant_content should stay in lockstep with full_reply"
    );
    assert!(
        state.compact_detected,
        "boundary detection must mark compact_detected for SP reseed"
    );

    let captured = deltas.lock().await.clone();
    assert_eq!(
        captured,
        vec![
            "OK\n".to_string(),
            "---\n".to_string(),
            "## ".to_string(),
            " Goal\n- placeholder\n".to_string()
        ],
        "all streamed chunks should remain visible"
    );

    ingest_acp_message_chunk("## Constraints\n- none\n", &mut state, &emitter).await;
    assert_eq!(
        state.full_reply,
        "OK\n---\n##  Goal\n- placeholder\n## Constraints\n- none\n"
    );
    let captured_after = deltas.lock().await.clone();
    assert_eq!(
        captured_after.len(),
        5,
        "post-boundary chunk should still produce a new delta"
    );
}

/// 回归：scan_start 采用“按字节回看窗口”时，若 full_reply 前缀包含中文等多字节 UTF-8
/// 字符，旧实现会把切片起点落到字符中间并 panic。
#[tokio::test]
async fn ingest_acp_message_chunk_handles_multibyte_prefix_when_scanning_boundary_window() {
    let mut state = AcpPromptState::default();
    let (emitter, deltas) = collecting_emitter();

    let prefix = "我".repeat(30);
    ingest_acp_message_chunk(&prefix, &mut state, &emitter).await;
    ingest_acp_message_chunk("\n---\n## Goal\n- placeholder\n", &mut state, &emitter).await;

    assert_eq!(
        state.full_reply,
        format!("{prefix}\n---\n## Goal\n- placeholder\n")
    );
    assert_eq!(
        state.pending_assistant_content,
        format!("{prefix}\n---\n## Goal\n- placeholder\n")
    );
    assert!(state.compact_detected);
    assert_eq!(
        deltas.lock().await.clone(),
        vec![prefix, "\n---\n## Goal\n- placeholder\n".to_string()]
    );
}

#[tokio::test]
async fn acp_event_log_records_identity_for_grep() {
    let temp_root = std::env::temp_dir().join(format!(
        "hone_acp_log_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let ctx = AcpEventLogContext {
        runner_label: "codex",
        log_path: acp_event_log_path(&temp_root.to_string_lossy()),
        session_id: "session-1".to_string(),
        identity: "Actor_feishu__group-1__alice".to_string(),
        actor_channel: "feishu".to_string(),
        actor_user_id: "alice".to_string(),
        actor_channel_scope: Some("group-1".to_string()),
    };

    log_acp_payload(
        Some(&ctx),
        "recv",
        &json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": { "update": { "sessionUpdate": "usage_update", "used": 1 } }
        }),
    )
    .await;

    let content = tokio::fs::read_to_string(&ctx.log_path)
        .await
        .expect("read log");
    assert!(content.contains("\"identity\":\"Actor_feishu__group-1__alice\""));
    assert!(content.contains("\"method\":\"session/update\""));

    let _ = tokio::fs::remove_dir_all(&temp_root).await;
}

#[tokio::test]
async fn acp_permission_request_matching_expected_id_is_not_prompt_response() {
    let mut child = tokio::process::Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn cat");
    let mut stdin = child.stdin.take().expect("child stdin");
    let line = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "session/request_permission",
        "params": {
            "sessionId": "session-1",
            "options": [
                {
                    "kind": "allow_always",
                    "name": "Allow for this session",
                    "optionId": "approved-for-session"
                }
            ],
            "toolCall": {
                "title": "Approve MCP tool call"
            }
        }
    })
    .to_string();

    let result = process_acp_payload(
        "codex",
        &mut stdin,
        4,
        &line,
        None,
        None,
        None,
        None,
        None,
        AcpPermissionDecision::ApproveForSession,
        None,
    )
    .await
    .expect("process permission request");

    assert!(result.is_none());
    drop(stdin);
    let output = child.wait_with_output().await.expect("cat output");
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("\"id\":4"));
    assert!(stdout.contains("\"optionId\":\"approved-for-session\""));
}

#[test]
fn acp_prompt_success_requires_explicit_non_cancelled_stop_reason() {
    assert!(acp_prompt_succeeded(Some("end_turn")));
    assert!(acp_prompt_succeeded(Some("max_tokens")));
    assert!(!acp_prompt_succeeded(Some("cancelled")));
    assert!(!acp_prompt_succeeded(None));
}
