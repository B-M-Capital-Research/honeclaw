use std::sync::Arc;

use async_trait::async_trait;
use hone_core::agent::AgentContext;
use hone_memory::session::SessionPromptState;
use hone_scheduler::SchedulerEvent;
use serde::Deserialize;
use serde_json::Value;

use crate::agent_session::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionResult, GeminiStreamOptions,
};
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest};
use crate::sandbox::ensure_actor_sandbox;
use crate::{AgentSession, HoneBotCore};

const HEARTBEAT_NOOP_SENTINEL: &str = "[[HEARTBEAT_NOOP]]";
const HEARTBEAT_INTERNAL_PREFIX: &str = "[[HEART";

#[derive(Debug, PartialEq, Eq)]
enum HeartbeatOutcome {
    Noop,
    Deliver(String),
}

#[derive(Debug, PartialEq, Eq)]
enum HeartbeatParseKind {
    Empty,
    SentinelNoop,
    InternalMarker,
    JsonNoop,
    JsonTriggered,
    JsonUnknownStatus,
    JsonMalformed,
    PlainText,
}

#[derive(Debug, Deserialize)]
struct HeartbeatJsonResponse {
    status: Option<String>,
    message: Option<String>,
}

fn parse_heartbeat_json_payload(content: &str) -> Option<HeartbeatJsonResponse> {
    let trimmed = content.trim();
    if let Ok(parsed) = serde_json::from_str::<HeartbeatJsonResponse>(trimmed) {
        return Some(parsed);
    }

    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str::<HeartbeatJsonResponse>(&trimmed[start..=end]).ok()
}

fn heartbeat_internal_marker_prefix(text: &str) -> bool {
    let trimmed = text.trim_start();
    let upper = trimmed.to_ascii_uppercase();
    upper.starts_with(HEARTBEAT_INTERNAL_PREFIX)
}

fn truncate_for_log(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

fn inspect_heartbeat_result(content: &str) -> (HeartbeatOutcome, HeartbeatParseKind) {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::Empty);
    }
    if trimmed == HEARTBEAT_NOOP_SENTINEL {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::SentinelNoop);
    }
    if heartbeat_internal_marker_prefix(trimmed) {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::InternalMarker);
    }

    if let Some(parsed) = parse_heartbeat_json_payload(trimmed) {
        let status = parsed.status.unwrap_or_default();
        if status.eq_ignore_ascii_case("noop") {
            return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonNoop);
        }
        if status.eq_ignore_ascii_case("triggered") {
            let message = parsed.message.unwrap_or_default().trim().to_string();
            if message.is_empty() || heartbeat_internal_marker_prefix(&message) {
                return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonTriggered);
            }
            return (
                HeartbeatOutcome::Deliver(message),
                HeartbeatParseKind::JsonTriggered,
            );
        }
        return (
            HeartbeatOutcome::Deliver(content.to_string()),
            HeartbeatParseKind::JsonUnknownStatus,
        );
    }

    if trimmed.contains('{') && trimmed.contains("\"status\"") {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonMalformed);
    }

    (
        HeartbeatOutcome::Deliver(content.to_string()),
        HeartbeatParseKind::PlainText,
    )
}

pub struct ScheduledTaskExecution {
    pub should_deliver: bool,
    pub content: String,
    pub error: Option<String>,
    pub metadata: Value,
}

pub fn build_scheduled_prompt(event: &SchedulerEvent) -> String {
    if event.heartbeat {
        return format!(
            "[心跳检测任务] 任务名称：{}。\n\
你正在执行一个每 30 分钟运行一次的后台条件检查。\n\
请使用可用工具检查用户设置的触发条件是否已经满足。\n\
\n\
规则：\n\
1. 如果条件尚未满足，优先只输出 `{{\"status\":\"noop\"}}`；为兼容旧行为，也允许只输出 `{}`。\n\
2. 如果条件已满足，只输出一段 JSON：`{{\"status\":\"triggered\",\"message\":\"...\"}}`。\n\
3. `message` 必须是一条可以直接发给用户的提醒消息，包含：满足的条件、关键数据、检查时间。\n\
4. 不要创建新的定时任务，也不要修改现有任务。\n\
5. 不要输出 Markdown 代码块，不要输出额外解释，不要暴露任何内部控制标记。\n\
\n\
以下是需要检查的用户条件：\n{}",
            event.job_name, HEARTBEAT_NOOP_SENTINEL, event.task_prompt
        );
    }
    let trigger_note = format!(
        "[定时任务触发] 任务名称：{}。请执行以下指令：",
        event.job_name
    );
    format!("{}\n\n{}", trigger_note, event.task_prompt)
}

pub async fn run_scheduled_task(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    mut run_options: AgentRunOptions,
) -> AgentSessionResult {
    let full_prompt = build_scheduled_prompt(event);
    run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
    let session = AgentSession::new(core, event.actor.clone(), event.channel_target.clone())
        .with_prompt_options(prompt_options);
    session.run(&full_prompt, run_options).await
}

pub async fn execute_scheduler_event(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    mut run_options: AgentRunOptions,
) -> ScheduledTaskExecution {
    if !event.heartbeat {
        let result = run_scheduled_task(core, event, prompt_options, run_options).await;
        let response = result.response;
        return if response.success {
            ScheduledTaskExecution {
                should_deliver: true,
                content: response.content,
                error: None,
                metadata: Value::Null,
            }
        } else {
            ScheduledTaskExecution {
                should_deliver: true,
                content: String::new(),
                error: response
                    .error
                    .or_else(|| Some("定时任务执行失败".to_string())),
                metadata: Value::Null,
            }
        };
    }

    run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
    run_options.model_override = Some(core.config.llm.openrouter.auxiliary_model().to_string());
    let heartbeat_model = run_options.model_override.clone().unwrap_or_default();

    match run_heartbeat_task(core, event, prompt_options, run_options).await {
        Ok(content) => {
            let raw_preview = truncate_for_log(content.trim(), 280);
            let raw_chars = content.chars().count();
            let starts_with_json = content.trim_start().starts_with('{');
            let (outcome, parse_kind) = inspect_heartbeat_result(&content);
            tracing::info!(
                "[HeartbeatDiag] job_id={} job={} target={} model={} raw_chars={} starts_with_json={} parse_kind={:?} raw_preview=\"{}\"",
                event.job_id,
                event.job_name,
                event.channel_target,
                heartbeat_model,
                raw_chars,
                starts_with_json,
                parse_kind,
                raw_preview.replace('\n', "\\n"),
            );
            if parse_kind == HeartbeatParseKind::JsonMalformed {
                tracing::warn!(
                    "[HeartbeatDiag] malformed heartbeat json delivered job_id={} job={} target={} preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    raw_preview.replace('\n', "\\n"),
                );
            }
            match outcome {
                HeartbeatOutcome::Noop => ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                    metadata: serde_json::json!({
                        "heartbeat_model": heartbeat_model,
                        "parse_kind": format!("{:?}", parse_kind),
                        "raw_chars": raw_chars,
                        "starts_with_json": starts_with_json,
                    }),
                },
                HeartbeatOutcome::Deliver(message) => {
                    let deliver_preview = truncate_for_log(message.trim(), 200);
                    tracing::info!(
                        "[HeartbeatDiag] deliver job_id={} job={} target={} parse_kind={:?} deliver_chars={} deliver_preview=\"{}\"",
                        event.job_id,
                        event.job_name,
                        event.channel_target,
                        parse_kind,
                        message.chars().count(),
                        deliver_preview.replace('\n', "\\n"),
                    );
                    ScheduledTaskExecution {
                        should_deliver: true,
                        content: message,
                        error: None,
                        metadata: serde_json::json!({
                            "heartbeat_model": heartbeat_model,
                            "parse_kind": format!("{:?}", parse_kind),
                            "raw_chars": raw_chars,
                            "starts_with_json": starts_with_json,
                            "deliver_preview": deliver_preview,
                        }),
                    }
                }
            }
        }
        Err(error) => {
            tracing::warn!(
                "[HeartbeatDiag] runner_error job_id={} job={} target={} model={} error=\"{}\"",
                event.job_id,
                event.job_name,
                event.channel_target,
                heartbeat_model,
                truncate_for_log(&error, 280).replace('\n', "\\n"),
            );
            ScheduledTaskExecution {
                should_deliver: false,
                content: String::new(),
                error: Some(error),
                metadata: serde_json::json!({
                    "heartbeat_model": heartbeat_model,
                }),
            }
        }
    }
}

struct NoopEmitter;

#[async_trait]
impl AgentRunnerEmitter for NoopEmitter {
    async fn emit(&self, _event: AgentRunnerEvent) {}
}

async fn run_heartbeat_task(
    core: Arc<HoneBotCore>,
    event: &SchedulerEvent,
    prompt_options: PromptOptions,
    run_options: AgentRunOptions,
) -> Result<String, String> {
    let transient_session_id = format!("heartbeat_probe::{}", event.job_id);
    let prompt_state = SessionPromptState::default();
    let bundle = build_prompt_bundle(
        &core.config,
        &core.session_storage,
        &event.actor.channel,
        &transient_session_id,
        &prompt_state,
        &prompt_options,
    );
    let system_prompt = bundle.system_prompt();
    let runtime_input = bundle.compose_user_input(&build_scheduled_prompt(event));
    let tool_registry = core.create_tool_registry(Some(&event.actor), &event.channel_target, false);
    let runner = core
        .create_runner_with_model_override(
            &system_prompt,
            tool_registry,
            run_options.model_override.as_deref(),
        )
        .map_err(|err| format!("heartbeat task create_runner failed: {err}"))?;
    let runner_name = runner.name();

    let working_directory = ensure_actor_sandbox(&event.actor)
        .map_err(|err| format!("heartbeat task sandbox init failed: {err}"))?
        .to_string_lossy()
        .to_string();
    let timeout = run_options.timeout;
    let gemini_stream = timeout
        .map(|duration| GeminiStreamOptions {
            overall_timeout: duration,
            per_line_timeout: std::time::Duration::from_secs(90),
            ..GeminiStreamOptions::default()
        })
        .unwrap_or_default();
    let request = AgentRunnerRequest {
        session_id: transient_session_id.clone(),
        actor_label: event.actor.session_id(),
        actor: event.actor.clone(),
        channel_target: event.channel_target.clone(),
        allow_cron: false,
        config_path: crate::core::runtime_config_path(),
        system_prompt,
        runtime_input,
        context: AgentContext::new(transient_session_id),
        timeout,
        gemini_stream,
        session_metadata: std::collections::HashMap::new(),
        working_directory,
    };
    tracing::info!(
        "[HeartbeatDiag] run_start job_id={} job={} target={} runner={} model_override={} timeout_secs={}",
        event.job_id,
        event.job_name,
        event.channel_target,
        runner_name,
        run_options.model_override.as_deref().unwrap_or(""),
        timeout.map(|duration| duration.as_secs()).unwrap_or(0),
    );
    let result = runner.run(request, Arc::new(NoopEmitter)).await;
    if result.response.success {
        tracing::info!(
            "[HeartbeatDiag] run_finish job_id={} job={} target={} runner={} success=true content_chars={}",
            event.job_id,
            event.job_name,
            event.channel_target,
            runner_name,
            result.response.content.chars().count(),
        );
        Ok(result.response.content)
    } else {
        tracing::warn!(
            "[HeartbeatDiag] run_finish job_id={} job={} target={} runner={} success=false error=\"{}\"",
            event.job_id,
            event.job_name,
            event.channel_target,
            runner_name,
            truncate_for_log(
                result
                    .response
                    .error
                    .as_deref()
                    .unwrap_or("心跳检测执行失败"),
                280
            )
            .replace('\n', "\\n"),
        );
        Err(result
            .response
            .error
            .unwrap_or_else(|| "心跳检测执行失败".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{HeartbeatOutcome, HeartbeatParseKind, inspect_heartbeat_result};

    #[test]
    fn heartbeat_exact_noop_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("[[HEARTBEAT_NOOP]]").0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_partial_internal_marker_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("[[HEART").0,
            HeartbeatOutcome::Noop
        );
        assert_eq!(
            inspect_heartbeat_result("  [[HEARTBEAT").0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_json_noop_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result(r#"{"status":"noop"}"#).0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_json_triggered_delivers_message_only() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"{"status":"triggered","message":"闪迪股价已低于 520，当前 519.7（检查时间：09:30）"}"#
            )
            .0,
            HeartbeatOutcome::Deliver(
                "闪迪股价已低于 520，当前 519.7（检查时间：09:30）".to_string()
            )
        );
    }

    #[test]
    fn heartbeat_prefixed_json_triggered_delivers_message_only() {
        assert_eq!(
            inspect_heartbeat_result(
                r#"当前时间：09:00:58，小时数为9，分钟数0 < 30，条件满足。正在查询原油价格...
{"status":"triggered","message":"【原油价格播报 - 09:00】"}"#
            )
            .0,
            HeartbeatOutcome::Deliver("【原油价格播报 - 09:00】".to_string())
        );
    }

    #[test]
    fn heartbeat_prefixed_json_noop_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("先检查一下...\n{\"status\":\"noop\"}").0,
            HeartbeatOutcome::Noop
        );
    }

    #[test]
    fn heartbeat_plain_text_alert_still_delivers() {
        assert_eq!(
            inspect_heartbeat_result("闪迪股价已低于 520，当前 519.7（检查时间：09:30）").0,
            HeartbeatOutcome::Deliver(
                "闪迪股价已低于 520，当前 519.7（检查时间：09:30）".to_string()
            )
        );
    }

    #[test]
    fn heartbeat_malformed_json_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result(r#"{"status":"noop"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }
}
