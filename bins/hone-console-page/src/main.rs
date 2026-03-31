use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let config_path =
        std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let data_dir = std::env::var("HONE_DATA_DIR").ok().map(PathBuf::from);
    let skills_dir = std::env::var("HONE_SKILLS_DIR").ok().map(PathBuf::from);
    let deployment_mode = hone_web_api::runtime::runtime_deployment_mode();
    let runtime_dir = if let Some(data_dir) = data_dir.as_ref() {
        data_dir.join("runtime")
    } else {
        match hone_core::HoneConfig::from_file(&config_path) {
            Ok(config) => hone_core::runtime_heartbeat_dir(&config),
            Err(error) => {
                eprintln!("❌ hone-console-page 启动失败: 配置加载失败: {error}");
                std::process::exit(1);
            }
        }
    };
    let _process_lock =
        match hone_core::acquire_process_lock(&runtime_dir, hone_core::PROCESS_LOCK_CONSOLE_PAGE) {
            Ok(lock) => lock,
            Err(error) => {
                eprintln!(
                    "❌ hone-console-page 启动失败: {}",
                    hone_core::format_lock_failure_message(
                        hone_core::PROCESS_LOCK_CONSOLE_PAGE,
                        &hone_core::process_lock_path(
                            &runtime_dir,
                            hone_core::PROCESS_LOCK_CONSOLE_PAGE
                        ),
                        &error,
                        "Hone"
                    )
                );
                std::process::exit(1);
            }
        };

    let (_, port) = match hone_web_api::start_server(
        &config_path,
        data_dir.as_deref(),
        skills_dir.as_deref(),
        &deployment_mode,
    )
    .await
    {
        Ok(pair) => pair,
        Err(error) => {
            eprintln!("❌ hone-console-page 启动失败: {error}");
            std::process::exit(1);
        }
    };

    tracing::info!("hone-console-page running at http://127.0.0.1:{port}");

    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("hone-console-page shutdown");
}
