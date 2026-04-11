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
    let (config, config_path) = match load_runtime_config() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("❌ 配置加载失败: {err}");
            std::process::exit(1);
        }
    };
    let core = Arc::new(HoneBotCore::new(config));

    hone_core::logging::setup_logging(&core.config.logging);
    info!("🚀 Hone {display_name} 启动");
    core.log_startup_routing(channel, &config_path);

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
