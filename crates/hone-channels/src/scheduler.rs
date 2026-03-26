use std::sync::Arc;

use async_trait::async_trait;
use hone_core::agent::AgentContext;
use hone_memory::session::SessionPromptState;
use hone_scheduler::SchedulerEvent;

use crate::agent_session::{AgentRunOptions, AgentRunQuotaMode, AgentSessionResult, GeminiStreamOptions};
use crate::prompt::{PromptOptions, build_prompt_bundle};
use crate::runners::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest};
use crate::sandbox::ensure_actor_sandbox;
use crate::{AgentSession, HoneBotCore};

const HEARTBEAT_NOOP_SENTINEL: &str = "[[HEARTBEAT_NOOP]]";

pub struct ScheduledTaskExecution {
    pub should_deliver: bool,
    pub content: String,
    pub error: Option<String>,
}

pub fn build_scheduled_prompt(event: &SchedulerEvent) -> String {
    if event.heartbeat {
        return format!(
            "[心跳检测任务] 任务名称：{}。\n\
你正在执行一个每 30 分钟运行一次的后台条件检查。\n\
请使用可用工具检查用户设置的触发条件是否已经满足。\n\
\n\
规则：\n\
1. 如果条件尚未满足，必须只输出 `{}`，不要输出任何解释。\n\
2. 如果条件已满足，输出一条可以直接发给用户的提醒消息，包含：满足的条件、关键数据、检查时间。\n\
3. 不要创建新的定时任务，也不要修改现有任务。\n\
\n\
以下是需要检查的用户条件：\n{}",
            event.job_name,
            HEARTBEAT_NOOP_SENTINEL,
            event.task_prompt
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
            }
        } else {
            ScheduledTaskExecution {
                should_deliver: true,
                content: String::new(),
                error: response.error.or_else(|| Some("定时任务执行失败".to_string())),
            }
        };
    }

    run_options.quota_mode = AgentRunQuotaMode::ScheduledTask;
    run_options.model_override = Some(core.config.llm.openrouter.auxiliary_model().to_string());

    match run_heartbeat_task(core, event, prompt_options, run_options).await {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed == HEARTBEAT_NOOP_SENTINEL || trimmed.is_empty() {
                ScheduledTaskExecution {
                    should_deliver: false,
                    content: String::new(),
                    error: None,
                }
            } else {
                ScheduledTaskExecution {
                    should_deliver: true,
                    content,
                    error: None,
                }
            }
        }
        Err(error) => ScheduledTaskExecution {
            should_deliver: false,
            content: String::new(),
            error: Some(error),
        },
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
    let result = runner.run(request, Arc::new(NoopEmitter)).await;
    if result.response.success {
        Ok(result.response.content)
    } else {
        Err(result
            .response
            .error
            .unwrap_or_else(|| "心跳检测执行失败".to_string()))
    }
}
