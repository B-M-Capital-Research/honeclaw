use std::sync::Arc;

use hone_core::{CHANNEL_DISABLED_EXIT_CODE, HoneConfig, ProcessHeartbeat, ProcessLockGuard};
use tracing::{error, info, warn};

use crate::{HoneBotCore, load_runtime_config};

pub struct ChannelRuntimeBootstrap {
    pub core: Arc<HoneBotCore>,
    _process_lock: ProcessLockGuard,
    _heartbeat: ProcessHeartbeat,
}

pub fn bootstrap_channel_runtime<F>(
    channel: &str,
    display_name: &str,
    process_lock_name: &'static str,
    is_enabled: F,
) -> ChannelRuntimeBootstrap
where
    F: Fn(&HoneConfig) -> bool,
{
    let (config, config_path) = load_channel_config_or_exit();
    // `hone-imessage` is intentionally outside this refactor and still calls a synchronous
    // bootstrap API. Keep the one-time boundary here, isolated from every storage operation;
    // all async-capable channel binaries use `bootstrap_channel_runtime_async` below.
    let core = std::thread::Builder::new()
        .name("hone-channel-bootstrap".into())
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("channel bootstrap runtime")
                .block_on(HoneBotCore::new(config))
        })
        .expect("spawn channel bootstrap thread")
        .join()
        .expect("channel bootstrap thread panicked");
    finish_channel_runtime(
        channel,
        display_name,
        process_lock_name,
        is_enabled,
        Arc::new(core),
        &config_path,
    )
}

pub async fn bootstrap_channel_runtime_async<F>(
    channel: &str,
    display_name: &str,
    process_lock_name: &'static str,
    is_enabled: F,
) -> ChannelRuntimeBootstrap
where
    F: Fn(&HoneConfig) -> bool,
{
    let (config, config_path) = load_channel_config_or_exit();
    let core = Arc::new(HoneBotCore::new(config).await);
    finish_channel_runtime(
        channel,
        display_name,
        process_lock_name,
        is_enabled,
        core,
        &config_path,
    )
}

fn load_channel_config_or_exit() -> (HoneConfig, String) {
    match load_runtime_config() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("❌ 配置加载失败: {err}");
            std::process::exit(1);
        }
    }
}

fn finish_channel_runtime<F>(
    channel: &str,
    display_name: &str,
    process_lock_name: &'static str,
    is_enabled: F,
    core: Arc<HoneBotCore>,
    config_path: &str,
) -> ChannelRuntimeBootstrap
where
    F: Fn(&HoneConfig) -> bool,
{
    hone_core::logging::setup_logging(&core.config.logging);
    info!("🚀 Hone {display_name} 启动");
    core.log_startup_routing(channel, config_path);

    if !is_enabled(&core.config) {
        warn!("{channel}.enabled=false，{display_name} 不会启动。");
        std::process::exit(CHANNEL_DISABLED_EXIT_CODE);
    }

    let process_lock =
        match hone_core::acquire_runtime_process_lock(&core.config, process_lock_name) {
            Ok(lock) => lock,
            Err(lock_error) => {
                error!(
                    "{}",
                    hone_core::format_lock_failure_message(
                        process_lock_name,
                        &hone_core::process_lock_path(
                            &hone_core::runtime_heartbeat_dir(&core.config),
                            process_lock_name
                        ),
                        &lock_error,
                        display_name
                    )
                );
                std::process::exit(1);
            }
        };

    let heartbeat = match hone_core::spawn_process_heartbeat(&core.config, channel) {
        Ok(heartbeat) => heartbeat,
        Err(err) => {
            error!("无法启动 {display_name} heartbeat: {err}");
            std::process::exit(1);
        }
    };

    ChannelRuntimeBootstrap {
        core,
        _process_lock: process_lock,
        _heartbeat: heartbeat,
    }
}
