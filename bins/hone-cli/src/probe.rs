use std::sync::Arc;

use async_trait::async_trait;
use hone_channels::HoneBotCore;
use hone_channels::agent_session::{
    AgentRunOptions, AgentSession, AgentSessionEvent, AgentSessionListener,
};
use hone_channels::prompt::PromptOptions;
use hone_core::SessionIdentity;

use crate::ProbeArgs;

pub(crate) async fn run_probe(
    core: Arc<HoneBotCore>,
    config_path: &str,
    args: ProbeArgs,
) -> Result<(), String> {
    hone_core::logging::setup_logging(&core.config.logging);
    tracing::info!("Hone CLI probe started");
    core.log_startup_routing(&args.channel, config_path);
    let runner_name = core.config.agent.runner.clone();

    let actor = HoneBotCore::create_actor(&args.channel, &args.user_id, args.scope.as_deref())
        .map_err(|e| format!("probe actor 初始化失败: {e}"))?;

    let mut session = AgentSession::new(core, actor.clone(), args.channel.clone())
        .with_restore_max_messages(None)
        .with_prompt_options(PromptOptions {
            is_admin: args.admin,
            ..PromptOptions::default()
        });

    if args.group {
        let scope = args
            .scope
            .as_deref()
            .ok_or_else(|| "--group 模式下必须提供 --scope".to_string())?;
        let session_identity =
            SessionIdentity::group(&args.channel, scope).map_err(|e| e.to_string())?;
        session = session.with_session_identity(session_identity);
    }

    if args.show_events {
        session.add_listener(Arc::new(ProbeListener));
    }

    println!("config={config_path}");
    println!("runner={runner_name}");
    println!("actor={}", actor.session_id());
    println!("query={}", args.query);

    let result = session.run(&args.query, AgentRunOptions::default()).await;
    let response = result.response;

    println!("session_id={}", result.session_id);
    println!("success={}", response.success);
    println!("tool_calls={}", response.tool_calls_made.len());

    if response.success {
        println!("response:\n{}", response.content);
    } else {
        println!("error={}", response.error.unwrap_or_default());
    }

    Ok(())
}

struct ProbeListener;

#[async_trait]
impl AgentSessionListener for ProbeListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        match event {
            AgentSessionEvent::Progress { stage, detail } => {
                if let Some(detail) = detail.filter(|value| !value.trim().is_empty()) {
                    println!("[progress] {stage} :: {detail}");
                } else {
                    println!("[progress] {stage}");
                }
            }
            AgentSessionEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => {
                let mut line = format!("[tool] {tool} :: {status}");
                if let Some(message) = message.filter(|value| !value.trim().is_empty()) {
                    line.push_str(" :: ");
                    line.push_str(message.trim());
                }
                if let Some(reasoning) = reasoning.filter(|value| !value.trim().is_empty()) {
                    line.push_str(" :: ");
                    line.push_str(reasoning.trim());
                }
                println!("{line}");
            }
            AgentSessionEvent::Error { error } => {
                println!("[error] {:?} :: {}", error.kind, error.message);
            }
            AgentSessionEvent::Done { response } => {
                println!(
                    "[done] success={} tool_calls={} content_len={}",
                    response.success,
                    response.tool_calls_made.len(),
                    response.content.len()
                );
            }
            AgentSessionEvent::StreamDelta { .. }
            | AgentSessionEvent::StreamThought { .. }
            | AgentSessionEvent::UserMessage { .. }
            | AgentSessionEvent::Segment { .. } => {}
        }
    }
}
