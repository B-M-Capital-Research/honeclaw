use std::path::PathBuf;

#[tokio::main]
async fn main() {
    hone_core::cloud_runtime::load_dotenv_if_present();
    // 预热 build info。`current_build_info()` 是 `LazyLock`,其中的
    // `binary_sha256` 要同步读完整个二进制算 SHA-256——生产上这个二进制 278 MB,
    // 纯 CPU 就要 3.8 秒,机器繁忙时成倍放大。它此前是被 `/api/meta` 的请求路径
    // 首次触发的,于是重启后第一个 meta 请求会在 async worker 线程上同步哈希,
    // 并发请求还会全部阻塞在同一个 `Once` 上、挂住 runtime worker 线程。
    // 渠道进程早就通过 `hone_channels::bootstrap` 预热了,web 进程一直漏着。
    tokio::task::spawn_blocking(|| {
        let _ = hone_core::current_build_info();
    });
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

    let started = match hone_web_api::start_server(
        &config_path,
        data_dir.as_deref(),
        skills_dir.as_deref(),
        &deployment_mode,
    )
    .await
    {
        Ok(started) => started,
        Err(error) => {
            eprintln!("❌ hone-console-page 启动失败: {error}");
            std::process::exit(1);
        }
    };

    tracing::info!(
        "hone-console-page admin running at http://127.0.0.1:{}",
        started.admin_port
    );
    if let Some(public_port) = started.public_port {
        tracing::info!("hone-console-page public running at http://127.0.0.1:{public_port}");
    }

    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("hone-console-page shutdown");
}
