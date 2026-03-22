//! Hone CLI — 交互式终端
//!
//! 提供简单的文本 REPL 用于调试和本地测试。

use std::io::{self, Write};
use std::sync::Arc;

use hone_channels::agent_session::{AgentRunOptions, AgentSession};
use hone_channels::prompt::PromptOptions;

#[tokio::main]
async fn main() {
    // 从配置文件创建 HoneBotCore
    let core = match hone_channels::HoneBotCore::from_config_file("config.yaml") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 配置加载失败: {e}");
            eprintln!("   请确保 config.yaml 存在且格式正确");
            std::process::exit(1);
        }
    };

    let core = Arc::new(core);

    hone_core::logging::setup_logging(&core.config.logging);
    tracing::info!("Hone CLI 启动成功 ✅");
    core.log_startup_routing("cli", "config.yaml");
    let actor = hone_channels::HoneBotCore::create_actor("cli", "cli_user", None)
        .expect("cli actor should be valid");

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
        if input == "quit" || input == "exit" || input == "q" {
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
}
