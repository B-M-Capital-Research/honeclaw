use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::logging::LogBuffer;
use crate::public_auth::PublicAuthLimiter;

/// 调度器 → 浏览器的主动推送事件
#[derive(Debug, Clone, Serialize)]
pub struct PushEvent {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub event: String,
    pub data: Value,
}

/// iMessage Bot → Web Console 事件推送请求体
#[derive(Debug, Deserialize)]
pub struct IMessageEventRequest {
    pub channel: String,
    pub user_id: String,
    pub channel_scope: Option<String>,
    pub event_type: String,
    pub data: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeHeartbeatRequest {
    pub channel: String,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeHeartbeatProcess {
    pub channel: String,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct HeartbeatRegistry {
    entries: Mutex<HashMap<String, HashMap<u32, RuntimeHeartbeatProcess>>>,
}

impl HeartbeatRegistry {
    pub fn record(&self, request: RuntimeHeartbeatRequest) {
        let mut entries = self.entries.lock().unwrap();
        let channel_entries = entries.entry(request.channel.clone()).or_default();
        channel_entries.insert(
            request.pid,
            RuntimeHeartbeatProcess {
                channel: request.channel,
                pid: request.pid,
                started_at: request.started_at,
                updated_at: request.updated_at,
            },
        );
    }

    pub fn channel_processes(&self, channel: &str) -> Vec<RuntimeHeartbeatProcess> {
        let mut entries = self.entries.lock().unwrap();
        prune_registry(&mut entries);
        entries
            .get(channel)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default()
    }
}

fn prune_registry(entries: &mut HashMap<String, HashMap<u32, RuntimeHeartbeatProcess>>) {
    let cutoff = Utc::now() - chrono::Duration::hours(24);
    entries.retain(|_, processes| {
        processes.retain(|_, process| process.updated_at >= cutoff);
        !processes.is_empty()
    });
}

/// 服务端权威的聊天任务状态。
///
/// 这个状态只描述当前进程里真实存活的 runner；quota 的 `in_flight` 是计费预留，
/// 不能用于判断任务是否仍在运行。进程重启后 registry 为空，客户端据此把遗留的
/// user-only turn 标记为中断，而不是继续显示一个永不结束的“思考中”。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ActiveChatRun {
    pub run_id: String,
    pub started_at_ms: i64,
    pub updated_at_ms: i64,
    pub phase: String,
    pub status_text: String,
    /// The stages this run has already passed through, newest last.
    ///
    /// Only the latest status used to survive here, so a refresh mid-run
    /// replaced a visible trail — searched, read quotes, read statements —
    /// with whichever single line happened to be current. The pre-turn pass
    /// alone issues around twenty provider calls, so that trail is most of
    /// what tells the user the wait is doing something.
    pub steps: Vec<String>,
}

/// Matches the client's own cap and de-duplication, so a trail recovered after
/// a refresh reads exactly like the one that was on screen before it.
const ACTIVE_RUN_MAX_STEPS: usize = 8;

fn append_active_run_step(steps: &mut Vec<String>, status_text: &str) {
    let normalized = status_text.trim();
    if normalized.is_empty() || steps.last().is_some_and(|last| last == normalized) {
        return;
    }
    steps.push(normalized.to_string());
    if steps.len() > ACTIVE_RUN_MAX_STEPS {
        let overflow = steps.len() - ACTIVE_RUN_MAX_STEPS;
        steps.drain(..overflow);
    }
}

#[derive(Default)]
pub struct ActiveChatRunRegistry {
    entries: Mutex<HashMap<String, ActiveChatRun>>,
}

impl ActiveChatRunRegistry {
    pub fn try_begin(
        self: &Arc<Self>,
        session_id: String,
    ) -> Result<ActiveChatRunGuard, ActiveChatRun> {
        let now_ms = Utc::now().timestamp_millis();
        let mut entries = self.entries.lock().unwrap();
        if let Some(active) = entries.get(&session_id) {
            return Err(active.clone());
        }
        let run = ActiveChatRun {
            run_id: Uuid::new_v4().to_string(),
            started_at_ms: now_ms,
            updated_at_ms: now_ms,
            phase: "thinking".to_string(),
            status_text: "正在准备并核验所需信息".to_string(),
            steps: vec!["正在准备并核验所需信息".to_string()],
        };
        entries.insert(session_id.clone(), run.clone());
        Ok(ActiveChatRunGuard {
            handle: ActiveChatRunHandle {
                registry: self.clone(),
                session_id,
                run_id: run.run_id.clone(),
            },
        })
    }

    pub fn get(&self, session_id: &str) -> Option<ActiveChatRun> {
        self.entries.lock().unwrap().get(session_id).cloned()
    }

    pub fn count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    fn update(
        &self,
        session_id: &str,
        run_id: &str,
        phase: &str,
        status_text: &str,
    ) -> Option<ActiveChatRun> {
        let mut entries = self.entries.lock().unwrap();
        let run = entries.get_mut(session_id)?;
        if run.run_id != run_id {
            return None;
        }
        run.phase = phase.to_string();
        run.status_text = status_text.to_string();
        append_active_run_step(&mut run.steps, status_text);
        run.updated_at_ms = Utc::now().timestamp_millis();
        Some(run.clone())
    }

    fn heartbeat(
        &self,
        session_id: &str,
        run_id: &str,
        phase: &str,
        status_text: &str,
        min_silence: Duration,
    ) -> Option<ActiveChatRun> {
        let now_ms = Utc::now().timestamp_millis();
        let min_silence_ms = i64::try_from(min_silence.as_millis()).unwrap_or(i64::MAX);
        let mut entries = self.entries.lock().unwrap();
        let run = entries.get_mut(session_id)?;
        if run.run_id != run_id || now_ms.saturating_sub(run.updated_at_ms) < min_silence_ms {
            return None;
        }
        // A heartbeat proves liveness; it must not rewind a more specific
        // entity/quote/model stage back to a generic "still running" label.
        // The supplied values are fallbacks for legacy/empty snapshots only.
        if run.phase.trim().is_empty() {
            run.phase = phase.to_string();
        }
        if run.status_text.trim().is_empty() {
            run.status_text = status_text.to_string();
            append_active_run_step(&mut run.steps, status_text);
        }
        // A heartbeat proves liveness, not progress, so it never appends a
        // step of its own — the trail would fill with "still working".
        run.updated_at_ms = now_ms;
        Some(run.clone())
    }

    fn finish(&self, session_id: &str, run_id: &str) {
        let mut entries = self.entries.lock().unwrap();
        if entries
            .get(session_id)
            .is_some_and(|run| run.run_id == run_id)
        {
            entries.remove(session_id);
        }
    }
}

#[derive(Clone)]
pub struct ActiveChatRunHandle {
    registry: Arc<ActiveChatRunRegistry>,
    session_id: String,
    run_id: String,
}

impl ActiveChatRunHandle {
    pub fn update(&self, phase: &str, status_text: &str) -> Option<ActiveChatRun> {
        self.registry
            .update(&self.session_id, &self.run_id, phase, status_text)
    }

    /// Emit a liveness update only after the run has been otherwise silent.
    /// Returning the updated snapshot lets SSE consumers keep the original
    /// server start time across refreshes.
    pub fn heartbeat(
        &self,
        phase: &str,
        status_text: &str,
        min_silence: Duration,
    ) -> Option<ActiveChatRun> {
        self.registry.heartbeat(
            &self.session_id,
            &self.run_id,
            phase,
            status_text,
            min_silence,
        )
    }

    /// Remove the run before publishing its terminal SSE frame. The guard may
    /// still be alive for a few instructions, but refresh recovery must not
    /// append a fresh thinking card after the persisted final answer.
    pub fn finish(&self) {
        self.registry.finish(&self.session_id, &self.run_id);
    }
}

pub struct ActiveChatRunGuard {
    handle: ActiveChatRunHandle,
}

impl ActiveChatRunGuard {
    pub fn run(&self) -> Option<ActiveChatRun> {
        self.handle.registry.get(&self.handle.session_id)
    }

    pub fn handle(&self) -> ActiveChatRunHandle {
        self.handle.clone()
    }
}

impl Drop for ActiveChatRunGuard {
    fn drop(&mut self) {
        self.handle
            .registry
            .finish(&self.handle.session_id, &self.handle.run_id);
    }
}

pub struct AppState {
    pub core: Arc<hone_channels::HoneBotCore>,
    pub web_auth: Arc<hone_memory::WebAuthStorage>,
    pub billing: Arc<hone_memory::BillingStorage>,
    pub email_verification_sender: Arc<dyn crate::email_verification::EmailVerificationSender>,
    pub public_auth_limiter: PublicAuthLimiter,
    pub push_tx: broadcast::Sender<PushEvent>,
    pub http_client: reqwest::Client,
    pub log_buffer: LogBuffer,
    pub deployment_mode: String,
    pub auth: AuthState,
    pub heartbeat_registry: HeartbeatRegistry,
    pub active_chat_runs: Arc<ActiveChatRunRegistry>,
}

pub struct AuthState {
    pub bearer_token: Option<String>,
    pub sse_tickets: Mutex<HashMap<String, Instant>>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::{ACTIVE_RUN_MAX_STEPS, ActiveChatRunRegistry};

    #[test]
    fn active_chat_run_keeps_stable_start_and_is_removed_by_guard() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-1".to_string())
            .expect("first run should start");
        let initial = guard.run().expect("active run");

        let updated = guard
            .handle()
            .update("running", "正在核验数据")
            .expect("updated active run");
        assert_eq!(updated.run_id, initial.run_id);
        assert_eq!(updated.started_at_ms, initial.started_at_ms);
        assert_eq!(updated.phase, "running");
        assert_eq!(updated.status_text, "正在核验数据");
        assert!(registry.try_begin("session-1".to_string()).is_err());

        drop(guard);
        assert!(registry.get("session-1").is_none());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn active_chat_run_can_finish_before_its_guard_drops() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-1".to_string())
            .expect("first run should start");
        let handle = guard.handle();
        let stage = handle
            .update("running", "正在核验数据")
            .expect("stage should update the live run");

        let heartbeat = handle
            .heartbeat("running", "仍在处理中", std::time::Duration::ZERO)
            .expect("heartbeat should update the live run");
        assert_eq!(heartbeat.started_at_ms, stage.started_at_ms);
        assert_eq!(heartbeat.phase, "running");
        assert_eq!(heartbeat.status_text, "正在核验数据");
        assert!(heartbeat.updated_at_ms >= stage.updated_at_ms);

        handle.finish();
        assert_eq!(registry.count(), 0);
        assert!(guard.run().is_none());

        // If another request starts in the small interval before the old task
        // unwinds, the old guard's run-id fence must not remove the new run.
        let replacement = registry
            .try_begin("session-1".to_string())
            .expect("replacement run should start after terminal state");
        let replacement_id = replacement.run().unwrap().run_id;
        drop(guard);
        assert_eq!(registry.get("session-1").unwrap().run_id, replacement_id);
        drop(replacement);
        assert_eq!(registry.count(), 0);
    }

    /// The pre-turn pass alone issues around twenty provider calls before the
    /// first token. Only the newest status used to survive a refresh, so the
    /// visible trail — searched, read quotes, read statements — collapsed to
    /// one line and the run read as if it had forgotten what it was doing.
    #[test]
    fn a_run_keeps_the_stages_it_already_passed_through() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-steps".to_string())
            .expect("first run");
        let handle = guard.handle();

        handle.update("running", "正在核验 NBIS 的证券身份");
        handle.update("running", "正在读取 NBIS 的行情、季度财报与估值口径");
        // A repeated status is the same stage, not a new one.
        handle.update("running", "正在读取 NBIS 的行情、季度财报与估值口径");
        handle.update("running", "正在输出最终回答");

        let run = registry.get("session-steps").expect("active run");
        assert_eq!(
            run.steps,
            [
                "正在准备并核验所需信息",
                "正在核验 NBIS 的证券身份",
                "正在读取 NBIS 的行情、季度财报与估值口径",
                "正在输出最终回答",
            ]
        );
        assert_eq!(run.status_text, "正在输出最终回答");
    }

    #[test]
    fn a_long_run_keeps_only_the_most_recent_stages() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-cap".to_string())
            .expect("first run");
        let handle = guard.handle();
        for index in 0..12 {
            handle.update("running", &format!("阶段 {index}"));
        }

        let run = registry.get("session-cap").expect("active run");
        // Bounded so a long run cannot grow this in-memory entry without limit.
        assert_eq!(run.steps.len(), ACTIVE_RUN_MAX_STEPS);
        assert_eq!(run.steps.last().unwrap(), "阶段 11");
        assert!(
            !run.steps
                .iter()
                .any(|step| step == "正在准备并核验所需信息")
        );
    }

    /// A heartbeat proves the run is alive, not that it moved on.
    #[test]
    fn heartbeats_do_not_pad_the_trail() {
        let registry = Arc::new(ActiveChatRunRegistry::default());
        let guard = registry
            .try_begin("session-heartbeat".to_string())
            .expect("first run");
        let handle = guard.handle();
        handle.update("running", "正在读取行情数据");
        let before = registry.get("session-heartbeat").expect("run").steps;

        for _ in 0..5 {
            handle.heartbeat("running", "仍在处理中", Duration::ZERO);
        }

        let after = registry.get("session-heartbeat").expect("run").steps;
        assert_eq!(before, after);
        assert_eq!(after.last().unwrap(), "正在读取行情数据");
    }
}
