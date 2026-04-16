//! Hone Discord Bot 入口
//!
//! 使用 serenity 监听 Discord 消息并调用 Hone Agent 回复。

mod attachments;
mod handlers;
mod scheduler;
mod utils;

use hone_channels::{ActorScopeResolver, MessageDeduplicator, SessionLockRegistry};
use serenity::all::{Client, GatewayIntents};
use tracing::error;

use crate::handlers::DiscordHandler;
use crate::scheduler::handle_scheduler_events;

#[tokio::main]
async fn main() {
    let runtime = hone_channels::bootstrap_channel_runtime(
        "discord",
        "Discord Bot",
        hone_core::PROCESS_LOCK_DISCORD,
        |config| config.discord.enabled,
    );
    let core = runtime.core;

    let token = core.config.discord.bot_token.trim().to_string();
    if token.is_empty() {
        eprintln!("❌ 未配置 Discord Token。请设置 config.yaml:discord.bot_token");
        std::process::exit(1);
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let handler = DiscordHandler {
        core: core.clone(),
        dedup: MessageDeduplicator::new(std::time::Duration::from_secs(120), 2048),
        session_locks: SessionLockRegistry::new(),
        scope_resolver: ActorScopeResolver::new("discord"),
        pretrigger: hone_channels::GroupPretriggerWindowRegistry::new(
            core.config.group_context.pretrigger_window_max_messages,
            std::time::Duration::from_secs(
                core.config.group_context.pretrigger_window_max_age_seconds,
            ),
        ),
    };

    let mut client = match Client::builder(token, intents).event_handler(handler).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 创建 Discord Client 失败: {e}");
            std::process::exit(1);
        }
    };

    let (scheduler, event_rx) = core.create_scheduler(vec!["discord".to_string()]);
    tokio::spawn(async move {
        scheduler.start().await;
    });

    let scheduler_http = client.http.clone();
    let scheduler_core = core.clone();
    tokio::spawn(async move {
        handle_scheduler_events(scheduler_http, scheduler_core, event_rx).await;
    });

    if let Err(e) = client.start().await {
        error!("Discord 运行失败: {}", e);
        let err_text = e.to_string().to_lowercase();
        if err_text.contains("invalid authentication") || err_text.contains("invalidauthentication")
        {
            eprintln!("[ERROR] Discord bot token 认证失败");
            eprintln!("[提示] 请检查是否重复粘贴或 token 已失效");
            eprintln!("[建议] 运行 `hone-cli configure --section channels` 重新配置");
        }
        std::process::exit(1);
    }
}
