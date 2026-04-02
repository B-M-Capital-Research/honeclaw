//! Hone Discord Bot 入口
//!
//! 使用 serenity 监听 Discord 消息并调用 Hone Agent 回复。

mod attachments;
mod handlers;
mod scheduler;
mod utils;

use std::sync::Arc;

use hone_channels::{ActorScopeResolver, MessageDeduplicator, SessionLockRegistry};
use serenity::all::{Client, GatewayIntents};
use tracing::{error, info, warn};

use crate::handlers::DiscordHandler;
use crate::scheduler::handle_scheduler_events;

#[tokio::main]
async fn main() {
    let (config, config_path) = match hone_channels::load_runtime_config() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("❌ 配置加载失败: {e}");
            std::process::exit(1);
        }
    };
    let core = hone_channels::HoneBotCore::new(config);

    hone_core::logging::setup_logging(&core.config.logging);
    info!("🎮 Hone Discord Bot 启动");
    core.log_startup_routing("discord", &config_path);

    if !core.config.discord.enabled {
        warn!("discord.enabled=false，Discord Bot 不会启动。");
        std::process::exit(hone_core::CHANNEL_DISABLED_EXIT_CODE);
    }

    let token = core.config.discord.bot_token.trim().to_string();
    if token.is_empty() {
        eprintln!("❌ 未配置 Discord Token。请设置 config.yaml:discord.bot_token");
        std::process::exit(1);
    };

    let _process_lock = match hone_core::acquire_runtime_process_lock(
        &core.config,
        hone_core::PROCESS_LOCK_DISCORD,
    ) {
        Ok(lock) => lock,
        Err(error) => {
            error!(
                "{}",
                hone_core::format_lock_failure_message(
                    hone_core::PROCESS_LOCK_DISCORD,
                    &hone_core::process_lock_path(
                        &hone_core::runtime_heartbeat_dir(&core.config),
                        hone_core::PROCESS_LOCK_DISCORD
                    ),
                    &error,
                    "Discord"
                )
            );
            std::process::exit(1);
        }
    };

    let _heartbeat = match hone_core::spawn_process_heartbeat(&core.config, "discord") {
        Ok(heartbeat) => heartbeat,
        Err(err) => {
            error!("无法启动 Discord heartbeat: {err}");
            std::process::exit(1);
        }
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let core = Arc::new(core);
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
        std::process::exit(1);
    }
}
