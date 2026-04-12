use std::io::{self, Write};
use std::sync::Arc;

use hone_channels::HoneBotCore;
use hone_channels::agent_session::{AgentRunOptions, AgentSession};
use hone_channels::prompt::PromptOptions;

pub(crate) async fn run_chat(core: Arc<HoneBotCore>, config_path: &str) -> Result<(), String> {
    hone_core::logging::setup_logging(&core.config.logging);
    tracing::info!("Hone CLI chat started");
    core.log_startup_routing("cli", config_path);
    let actor = HoneBotCore::create_actor("cli", "cli_user", None)
        .map_err(|e| format!("cli actor 初始化失败: {e}"))?;

    println!("╭─────────────────────────────────────────╮");
    println!("│  🍯 Hone Financial — CLI                │");
    println!("│  输入消息与 AI 对话，输入 quit 退出       │");
    println!("╰─────────────────────────────────────────╯");
    println!();

    loop {
        print!("You > ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        if matches!(input, "quit" | "exit" | "q") {
            println!("👋 再见！");
            break;
        }

        let prompt_options = PromptOptions {
            is_admin: true,
            ..PromptOptions::default()
        };

        let session = AgentSession::new(core.clone(), actor.clone(), "cli")
            .with_restore_max_messages(None)
            .with_prompt_options(prompt_options);

        println!("🤔 思考中...");
        let result = session.run(input, AgentRunOptions::default()).await;
        let response = result.response;

        if response.success {
            println!("\nHone > {}", response.content);
        } else {
            let err = response.error.clone().unwrap_or_default();
            println!("\n❌ 错误: {}", err);
        }

        if !response.tool_calls_made.is_empty() {
            println!("   📌 调用了 {} 个工具", response.tool_calls_made.len());
        }
        println!();
    }

    Ok(())
}
