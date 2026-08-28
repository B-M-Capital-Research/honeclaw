use std::io::{self, Read, Write};
use std::sync::Arc;

use hone_channels::HoneBotCore;
use hone_channels::agent_session::{AgentRunOptions, AgentSession};
use hone_channels::prompt::PromptOptions;

use crate::ChatArgs;

pub(crate) async fn run_chat(
    core: Arc<HoneBotCore>,
    config_path: &str,
    args: ChatArgs,
) -> Result<(), String> {
    hone_core::logging::setup_logging(&core.config.logging);
    tracing::info!("Hone CLI chat started");
    core.log_startup_routing("cli", config_path);
    let actor_id = args.actor_id.as_deref().unwrap_or("cli_user");
    let actor = HoneBotCore::create_actor("cli", actor_id, None)
        .map_err(|e| format!("cli actor 初始化失败：{e}"))?;

    if args.once {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("读取单次问题失败：{error}"))?;
        let input = input.trim();
        if input.is_empty() {
            return Err("单次问题不能为空".to_string());
        }
        let prompt_options = PromptOptions {
            is_admin: true,
            ..PromptOptions::default()
        };
        let session = AgentSession::new(core, actor, "cli")
            .with_restore_max_messages(None)
            .with_prompt_options(prompt_options);
        let response = session
            .run(input, AgentRunOptions::default())
            .await
            .response;
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "success": response.success,
                    "content": response.content,
                    "error": response.error,
                    "tool_calls_made": response.tool_calls_made.len(),
                })
            );
        } else if response.success {
            println!("{}", response.content);
        } else {
            println!("{}", response.error.clone().unwrap_or_default());
        }
        return if response.success {
            Ok(())
        } else {
            Err("单次回答失败".to_string())
        };
    }

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

        println!("🤔 思考中…");
        let result = session.run(input, AgentRunOptions::default()).await;
        let response = result.response;

        if response.success {
            println!("\nHone > {}", response.content);
        } else {
            let err = response.error.clone().unwrap_or_default();
            println!("\n❌ 错误：{}", err);
        }

        if !response.tool_calls_made.is_empty() {
            println!("   📌 调用了 {} 个工具", response.tool_calls_made.len());
        }
        println!();
    }

    Ok(())
}
