use std::sync::Arc;

use hone_scheduler::SchedulerEvent;

use crate::agent_session::{AgentRunOptions, AgentRunQuotaMode, AgentSessionResult};
use crate::prompt::PromptOptions;
use crate::{AgentSession, HoneBotCore};

pub fn build_scheduled_prompt(event: &SchedulerEvent) -> String {
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
