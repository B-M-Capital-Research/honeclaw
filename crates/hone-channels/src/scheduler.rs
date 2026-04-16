use std::sync::Arc;

use async_trait::async_trait;
use hone_scheduler::SchedulerEvent;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::agent_session::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionResult, GeminiStreamOptions,
};
use crate::execution::{
    ExecutionMode, ExecutionRequest, ExecutionRunnerSelection, ExecutionService,
};
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent};
use crate::runtime::{sanitize_user_visible_output, user_visible_error_message};
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
    PlainTextSuppressed,
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

    let mut candidates = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in trimmed.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start_idx) = start.take() {
                        candidates.push(&trimmed[start_idx..=idx]);
                    }
                }
            }
            _ => {}
        }
    }

    candidates
        .into_iter()
        .rev()
        .find_map(|candidate| serde_json::from_str::<HeartbeatJsonResponse>(candidate).ok())
}

fn heartbeat_internal_marker_prefix(text: &str) -> bool {
    let trimmed = text.trim_start();
    let upper = trimmed.to_ascii_uppercase();
    upper.starts_with(HEARTBEAT_INTERNAL_PREFIX)
}

fn heartbeat_internal_marker_present(text: &str) -> bool {
    let upper = text.to_ascii_uppercase();
    upper.contains(HEARTBEAT_NOOP_SENTINEL) || upper.contains(HEARTBEAT_INTERNAL_PREFIX)
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
    if trimmed == HEARTBEAT_NOOP_SENTINEL || heartbeat_internal_marker_present(trimmed) {
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
            HeartbeatOutcome::Noop,
            HeartbeatParseKind::JsonUnknownStatus,
        );
    }

    if trimmed.starts_with('{') {
        return (HeartbeatOutcome::Noop, HeartbeatParseKind::JsonMalformed);
    }

    (
        HeartbeatOutcome::Noop,
        HeartbeatParseKind::PlainTextSuppressed,
    )
}

pub struct ScheduledTaskExecution {
    pub should_deliver: bool,
    pub content: String,
    pub error: Option<String>,
    pub metadata: Value,
}

fn heartbeat_parse_error_message(parse_kind: &HeartbeatParseKind) -> Option<String> {
    match parse_kind {
        HeartbeatParseKind::JsonUnknownStatus => {
            Some("heartbeat 输出包含未知状态，任务已标记失败".to_string())
        }
        HeartbeatParseKind::JsonMalformed => {
            Some("heartbeat 输出不是合法 JSON，任务已标记失败".to_string())
        }
        _ => None,
    }
}

fn heartbeat_execution_from_content(
    content: &str,
    heartbeat_model: &str,
) -> ScheduledTaskExecution {
    let raw_preview = truncate_for_log(content.trim(), 280);
    let raw_chars = content.chars().count();
    let starts_with_json = content.trim_start().starts_with('{');
    let (outcome, parse_kind) = inspect_heartbeat_result(content);
    let metadata = json!({
        "heartbeat_model": heartbeat_model,
        "parse_kind": format!("{:?}", parse_kind),
        "raw_chars": raw_chars,
        "starts_with_json": starts_with_json,
        "raw_preview": raw_preview,
    });

    if let Some(error) = heartbeat_parse_error_message(&parse_kind) {
        return ScheduledTaskExecution {
            should_deliver: false,
            content: String::new(),
            error: Some(error),
            metadata,
        };
    }

    match outcome {
        HeartbeatOutcome::Noop => ScheduledTaskExecution {
            should_deliver: false,
            content: String::new(),
            error: None,
            metadata,
        },
        HeartbeatOutcome::Deliver(message) => {
            let sanitized_message = sanitize_scheduler_delivery_text(&message);
            if sanitized_message.trim().is_empty() {
                return ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                    metadata: json!({
                        "heartbeat_model": heartbeat_model,
                        "parse_kind": format!("{:?}", parse_kind),
                        "raw_chars": raw_chars,
                        "starts_with_json": starts_with_json,
                        "raw_preview": raw_preview,
                        "deliver_preview": truncate_for_log(message.trim(), 200),
                        "sanitized_empty": true,
                    }),
                };
            }
            let deliver_preview = truncate_for_log(message.trim(), 200);
            ScheduledTaskExecution {
                should_deliver: true,
                content: sanitized_message,
                error: None,
                metadata: json!({
                    "heartbeat_model": heartbeat_model,
                    "parse_kind": format!("{:?}", parse_kind),
                    "raw_chars": raw_chars,
                    "starts_with_json": starts_with_json,
                    "raw_preview": raw_preview,
                    "deliver_preview": deliver_preview,
                }),
            }
        }
    }
}

fn sanitize_scheduler_delivery_text(text: &str) -> String {
    let sanitized = sanitize_user_visible_output(text).content;
    let kept_lines = sanitized
        .lines()
        .filter(|line| !is_scheduler_protocol_residue(line))
        .collect::<Vec<_>>()
        .join("\n");
    kept_lines.trim().to_string()
}

fn is_scheduler_protocol_residue(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return false;
    }
    if trimmed == "{}" {
        return true;
    }

    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) else {
        return false;
    };

    let suspicious_keys = [
        "tool",
        "tool_call_id",
        "arguments",
        "parameters",
        "result",
        "name",
        "status",
    ];
    let user_visible_keys = ["message", "content", "text"];

    map.keys()
        .any(|key| suspicious_keys.contains(&key.as_str()))
        && !map
            .keys()
            .any(|key| user_visible_keys.contains(&key.as_str()))
}

pub fn build_scheduled_prompt(event: &SchedulerEvent) -> String {
    if event.heartbeat {
        return format!(
            "[心跳检测任务] 任务名称：{}。\n\
你正在执行一个每 30 分钟运行一次的后台条件检查。\n\
请使用可用工具检查用户设置的触发条件是否已经满足。\n\
\n\
规则：\n\
1. 如果条件尚未满足，优先只输出 `{{\"status\":\"noop\"}}`；为兼容旧行为，也允许只输出 `{{}}`。\n\
2. 如果条件已满足，只输出一段 JSON：`{{\"status\":\"triggered\",\"message\":\"...\"}}`。\n\
3. `message` 必须是一条可以直接发给用户的提醒消息，包含：满足的条件、关键数据、检查时间。\n\
4. 不要创建新的定时任务，也不要修改现有任务。\n\
5. 不要输出 Markdown 代码块，不要输出额外解释，不要暴露任何内部控制标记。\n\
6. 如果你不确定是否满足条件，或者输出格式不是严格 JSON，就必须返回 noop，不允许发送自由文本。\n\
\n\
以下是需要检查的用户条件：\n{}",
            event.job_name, event.task_prompt
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
                content: sanitize_scheduler_delivery_text(&response.content),
                error: None,
                metadata: Value::Null,
            }
        } else {
            let sanitized_error = Some(user_visible_error_message(response.error.as_deref()));
            ScheduledTaskExecution {
                should_deliver: true,
                content: String::new(),
                error: sanitized_error,
                metadata: Value::Null,
            }
        };
    }

    run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
    run_options.model_override = Some(core.auxiliary_model_name());
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
                    "[HeartbeatDiag] malformed heartbeat json suppressed job_id={} job={} target={} preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    raw_preview.replace('\n', "\\n"),
                );
            }
            if matches!(
                parse_kind,
                HeartbeatParseKind::JsonUnknownStatus | HeartbeatParseKind::JsonMalformed
            ) {
                tracing::warn!(
                    "[HeartbeatDiag] parse failure escalated job_id={} job={} target={} parse_kind={:?} preview=\"{}\"",
                    event.job_id,
                    event.job_name,
                    event.channel_target,
                    parse_kind,
                    raw_preview.replace('\n', "\\n"),
                );
            }
            if let HeartbeatOutcome::Deliver(message) = &outcome {
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
            }
            heartbeat_execution_from_content(&content, &heartbeat_model)
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
                metadata: json!({
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
    let bundle = build_prompt_bundle(
        &core.config,
        &core.session_storage,
        &event.actor.channel,
        &transient_session_id,
        &Default::default(),
        &prompt_options,
    );
    let timeout = run_options.timeout;
    let execution = ExecutionService::new(core.clone()).prepare(ExecutionRequest {
        mode: ExecutionMode::TransientTask,
        session_id: transient_session_id.clone(),
        actor: event.actor.clone(),
        channel_target: event.channel_target.clone(),
        allow_cron: false,
        system_prompt: bundle.system_prompt(),
        runtime_input: bundle.compose_user_input(&build_scheduled_prompt(event)),
        context: hone_core::agent::AgentContext::new(transient_session_id),
        timeout,
        gemini_stream: timeout
            .map(|duration| GeminiStreamOptions {
                overall_timeout: duration,
                per_line_timeout: core.config.agent.step_timeout(),
                ..GeminiStreamOptions::default()
            })
            .unwrap_or_default(),
        session_metadata: std::collections::HashMap::new(),
        model_override: run_options.model_override.clone(),
        runner_selection: ExecutionRunnerSelection::AuxiliaryFunctionCalling { max_iterations: 6 },
        allowed_tools: None,
        max_tool_calls: None,
        prompt_audit: None,
    })?;
    tracing::info!(
        "[HeartbeatDiag] run_start job_id={} job={} target={} runner={} model_override={} timeout_secs={}",
        event.job_id,
        event.job_name,
        event.channel_target,
        execution.runner_name,
        run_options.model_override.as_deref().unwrap_or(""),
        timeout.map(|duration| duration.as_secs()).unwrap_or(0),
    );
    let result = execution
        .runner
        .run(execution.runner_request, Arc::new(NoopEmitter))
        .await;
    if result.response.success {
        tracing::info!(
            "[HeartbeatDiag] run_finish job_id={} job={} target={} runner={} success=true content_chars={}",
            event.job_id,
            event.job_name,
            event.channel_target,
            execution.runner_name,
            result.response.content.chars().count(),
        );
        Ok(result.response.content)
    } else {
        tracing::warn!(
            "[HeartbeatDiag] run_finish job_id={} job={} target={} runner={} success=false error=\"{}\"",
            event.job_id,
            event.job_name,
            event.channel_target,
            execution.runner_name,
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
    use super::{
        HeartbeatOutcome, HeartbeatParseKind, build_scheduled_prompt,
        heartbeat_execution_from_content, inspect_heartbeat_result,
        sanitize_scheduler_delivery_text,
    };
    use hone_core::ActorIdentity;
    use hone_scheduler::SchedulerEvent;
    use serde_json::Value;

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
    fn heartbeat_plain_text_is_suppressed() {
        assert_eq!(
            inspect_heartbeat_result("闪迪股价已低于 520，当前 519.7（检查时间：09:30）"),
            (
                HeartbeatOutcome::Noop,
                HeartbeatParseKind::PlainTextSuppressed
            )
        );
    }

    #[test]
    fn heartbeat_think_wrapped_json_noop_is_suppressed() {
        let content = "<think> 当前小米股价为30.88港元，高于30港元的触发线，所以条件未满足。根据规则，我应该输出 `{\"status\":\"noop\"}` 或 `[[HEARTBEAT_NOOP]]`。 </think>\n{\"status\":\"noop\"}";
        assert_eq!(inspect_heartbeat_result(content).0, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_think_wrapped_noop_marker_is_suppressed() {
        let content = "<think>\n让我检查一下这个心跳检测任务的条件。\n\n当前北京时间：2026-04-05 08:30:00\n当前小时数：8\n当前分钟数：30\n\n用户条件：\n如果当前小时数是 0、3、6、9、12、15、18、21 其中之一\n并且当前分钟数小于 30 分钟\n当前小时数 8 不在 [0, 3, 6, 9, 12, 15, 18, 21] 这个列表中，所以条件不满足。\n\n按照规则，我应该保持静默，不输出任何内容。\n</think>\n\n[[HEARTBEAT_NOOP]]";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::SentinelNoop)
        );
    }

    #[test]
    fn heartbeat_english_think_wrapped_noop_marker_is_suppressed() {
        let content = "<think>\nLet me analyze this request carefully.\n\nThe user is asking me to check if a heartbeat condition has been met. Let me parse the condition:\nCheck if current hour (Beijing time) is one of: 0, 3, 6, 9, 12, 15, 18, 21\nAND current minute is less than 30\nCurrent time: 2026-04-05 07:30:00 (Beijing time)\nHour: 07 (7)\nMinute: 30\nIs 7 in [0, 3, 6, 9, 12, 15, 18, 21]? No.\nTherefore, the condition is NOT met.\n\n</think>\n\n[[HEARTBEAT_NOOP]]";
        assert_eq!(
            inspect_heartbeat_result(content),
            (HeartbeatOutcome::Noop, HeartbeatParseKind::SentinelNoop)
        );
    }

    #[test]
    fn heartbeat_think_wrapped_triggered_json_delivers_message_only() {
        let content = "<think> 先整理结果。最终应该输出 JSON。 </think>\n{\"status\":\"triggered\",\"message\":\"小米已跌破 30 港元，当前 29.88 港元（检查时间：22:33）\"}";
        assert_eq!(
            inspect_heartbeat_result(content),
            (
                HeartbeatOutcome::Deliver(
                    "小米已跌破 30 港元，当前 29.88 港元（检查时间：22:33）".to_string()
                ),
                HeartbeatParseKind::JsonTriggered
            )
        );
    }

    #[test]
    fn heartbeat_malformed_json_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result(r#"{"status":"noop"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn scheduler_delivery_text_strips_internal_blocks_and_tool_protocol() {
        let raw =
            "<think>先判断一下</think>\n最终答案\n\n<tool_call>{\"tool\":\"cron_job\"}</tool_call>";
        let sanitized = sanitize_scheduler_delivery_text(raw);
        assert_eq!(sanitized, "最终答案");
    }

    #[test]
    fn scheduler_delivery_text_keeps_user_visible_json_message() {
        let raw = r#"{"status":"triggered","message":"今晚 20:30 继续复盘"}"#;
        let sanitized = sanitize_scheduler_delivery_text(raw);
        assert_eq!(sanitized, raw);
    }

    #[test]
    fn heartbeat_truncated_json_prefix_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result(r#"{"status"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_single_brace_is_detected() {
        let (outcome, parse_kind) = inspect_heartbeat_result("{");
        assert_eq!(parse_kind, HeartbeatParseKind::JsonMalformed);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_unknown_json_status_is_suppressed() {
        let (outcome, parse_kind) =
            inspect_heartbeat_result(r#"{"status":"maybe","message":"foo"}"#);
        assert_eq!(parse_kind, HeartbeatParseKind::JsonUnknownStatus);
        assert_eq!(outcome, HeartbeatOutcome::Noop);
    }

    #[test]
    fn heartbeat_unknown_json_status_marks_execution_failed() {
        let execution =
            heartbeat_execution_from_content(r#"{"status":"maybe","message":"foo"}"#, "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出包含未知状态，任务已标记失败")
        );
        assert_eq!(execution.metadata["parse_kind"], "JsonUnknownStatus");
        assert_eq!(execution.metadata["heartbeat_model"], "model-x");
        assert!(
            execution.metadata["raw_preview"]
                .as_str()
                .expect("raw_preview")
                .contains("\"status\":\"maybe\"")
        );
    }

    #[test]
    fn heartbeat_malformed_json_marks_execution_failed() {
        let execution = heartbeat_execution_from_content(r#"{"status":"noop"#, "model-x");
        assert!(!execution.should_deliver);
        assert_eq!(
            execution.error.as_deref(),
            Some("heartbeat 输出不是合法 JSON，任务已标记失败")
        );
        assert_eq!(execution.metadata["parse_kind"], "JsonMalformed");
    }

    #[test]
    fn heartbeat_prompt_keeps_legacy_empty_json_example_literal() {
        let event = SchedulerEvent {
            actor: ActorIdentity::new("discord", "alice", Some("dm")).expect("actor"),
            job_id: "job-1".to_string(),
            job_name: "heartbeat".to_string(),
            task_prompt: "检查条件".to_string(),
            channel: "discord".to_string(),
            channel_scope: Some("dm".to_string()),
            channel_target: "alice".to_string(),
            delivery_key: "delivery-1".to_string(),
            push: Value::Null,
            tags: vec![],
            heartbeat: true,
        };

        let prompt = build_scheduled_prompt(&event);
        assert!(prompt.contains("也允许只输出 `{}`。"));
        assert!(!prompt.contains("[[HEARTBEAT_NOOP]]"));
    }
}
