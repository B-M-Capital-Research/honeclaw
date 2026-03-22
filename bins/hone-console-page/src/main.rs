use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let config_path =
        std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let data_dir = std::env::var("HONE_DATA_DIR").ok().map(PathBuf::from);
    let skills_dir = std::env::var("HONE_SKILLS_DIR").ok().map(PathBuf::from);
    let deployment_mode = hone_web_api::runtime::runtime_deployment_mode();

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
