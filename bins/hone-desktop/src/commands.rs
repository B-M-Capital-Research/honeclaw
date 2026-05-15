use std::env;
use std::fmt;
use std::path::PathBuf;

use tauri::{AppHandle, State};

use crate::sidecar::{
    AgentSettings, AgentSettingsUpdateResult, BackendConfig, BackendStatusInfo,
    ChannelProcessCleanupResult, CliCheckResult, DesktopChannelSettings,
    DesktopChannelSettingsInput, DesktopChannelSettingsUpdateResult, DesktopState, FmpSettings,
    OpenRouterSettings, TavilySettings,
};

#[tauri::command]
pub(crate) fn get_backend_config(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendConfig, String> {
    crate::sidecar::get_backend_config_impl(app, state)
}

#[tauri::command]
pub(crate) fn set_backend_config(
    app: AppHandle,
    state: State<'_, DesktopState>,
    config: BackendConfig,
) -> Result<(), String> {
    crate::sidecar::set_backend_config_impl(app, state, config)
}

#[tauri::command]
pub(crate) async fn connect_backend(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    crate::sidecar::connect_backend_impl(app, state).await
}

#[tauri::command]
pub(crate) async fn start_bundled_backend(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    crate::sidecar::start_bundled_backend_impl(app, state).await
}

#[tauri::command]
pub(crate) async fn stop_bundled_backend(
    state: State<'_, DesktopState>,
) -> Result<BackendStatusInfo, String> {
    crate::sidecar::stop_bundled_backend_impl(state).await
}

#[tauri::command]
pub(crate) fn get_channel_settings(app: AppHandle) -> Result<DesktopChannelSettings, String> {
    crate::sidecar::get_channel_settings_impl(app)
}

#[tauri::command]
pub(crate) async fn set_channel_settings(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: DesktopChannelSettingsInput,
) -> Result<DesktopChannelSettingsUpdateResult, String> {
    crate::sidecar::set_channel_settings_impl(app, state, settings).await
}

#[tauri::command]
pub(crate) fn backend_status(app: AppHandle, state: State<'_, DesktopState>) -> BackendStatusInfo {
    crate::sidecar::backend_status_impl(app, state)
}

#[tauri::command]
pub(crate) async fn cleanup_channel_processes(
    state: State<'_, DesktopState>,
) -> Result<ChannelProcessCleanupResult, String> {
    crate::sidecar::cleanup_channel_processes_impl(state).await
}

#[tauri::command]
pub(crate) fn get_agent_settings(app: AppHandle) -> Result<AgentSettings, String> {
    crate::sidecar::get_agent_settings_impl(app)
}

#[tauri::command]
pub(crate) async fn set_agent_settings(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: AgentSettings,
) -> Result<AgentSettingsUpdateResult, String> {
    crate::sidecar::set_agent_settings_impl(app, state, settings).await
}

#[tauri::command]
pub(crate) async fn check_agent_cli(runner: String) -> Result<CliCheckResult, String> {
    crate::sidecar::check_agent_cli_impl(runner).await
}

#[tauri::command]
pub(crate) async fn test_openai_channel(
    url: String,
    model: String,
    api_key: String,
) -> Result<CliCheckResult, String> {
    crate::sidecar::test_openai_channel_impl(url, model, api_key).await
}

#[tauri::command]
pub(crate) fn get_openrouter_settings(app: AppHandle) -> Result<OpenRouterSettings, String> {
    crate::sidecar::get_openrouter_settings_impl(app)
}

#[tauri::command]
pub(crate) async fn set_openrouter_settings(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: OpenRouterSettings,
) -> Result<(), String> {
    crate::sidecar::set_openrouter_settings_impl(app, state, settings).await
}

#[tauri::command]
pub(crate) fn get_fmp_settings(app: AppHandle) -> Result<FmpSettings, String> {
    crate::sidecar::get_fmp_settings_impl(app)
}

#[tauri::command]
pub(crate) async fn set_fmp_settings(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: FmpSettings,
) -> Result<(), String> {
    crate::sidecar::set_fmp_settings_impl(app, state, settings).await
}

#[tauri::command]
pub(crate) fn get_tavily_settings(app: AppHandle) -> Result<TavilySettings, String> {
    crate::sidecar::get_tavily_settings_impl(app)
}

#[tauri::command]
pub(crate) async fn set_tavily_settings(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: TavilySettings,
) -> Result<(), String> {
    crate::sidecar::set_tavily_settings_impl(app, state, settings).await
}

pub(crate) fn run_desktop_app() {
    if desktop_smoke_server_enabled_from_env() {
        run_desktop_smoke_server();
        return;
    }

    crate::tray::setup_tray();

    let result = tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if let Err(error) = crate::sidecar::prepare_desktop_startup(app.handle().clone()) {
                crate::sidecar::record_startup_error(app.handle(), &error);
                crate::sidecar::show_startup_error_dialog(&error);
                return Err(std::io::Error::other(error).into());
            }
            crate::sidecar::bootstrap_backend_on_startup(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_backend_config,
            set_backend_config,
            connect_backend,
            start_bundled_backend,
            stop_bundled_backend,
            get_channel_settings,
            set_channel_settings,
            backend_status,
            cleanup_channel_processes,
            get_agent_settings,
            set_agent_settings,
            check_agent_cli,
            test_openai_channel,
            get_openrouter_settings,
            set_openrouter_settings,
            get_fmp_settings,
            set_fmp_settings,
            get_tavily_settings,
            set_tavily_settings
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("{}", desktop_run_error_message(error));
        std::process::exit(1);
    }
}

fn desktop_run_error_message(error: impl fmt::Display) -> String {
    format!("Hone Desktop exited with error: {error}")
}

fn desktop_smoke_server_enabled_from_env_value(value: Option<&str>) -> bool {
    matches!(
        value.map(|raw| raw.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes" | "on")
    )
}

fn desktop_smoke_server_enabled_from_env() -> bool {
    desktop_smoke_server_enabled_from_env_value(
        env::var("HONE_DESKTOP_SMOKE_SERVER").ok().as_deref(),
    )
}

fn desktop_smoke_config_path_from_env(
    config_path: Option<PathBuf>,
    user_config_path: Option<PathBuf>,
) -> PathBuf {
    config_path
        .or(user_config_path)
        .unwrap_or_else(|| PathBuf::from("config.yaml"))
}

fn desktop_smoke_data_dir_from_env(
    desktop_data_dir: Option<PathBuf>,
    data_dir: Option<PathBuf>,
) -> Option<PathBuf> {
    desktop_data_dir.or(data_dir)
}

fn run_desktop_smoke_server() {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Hone Desktop smoke server failed to create runtime: {error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = runtime.block_on(run_desktop_smoke_server_async()) {
        eprintln!("Hone Desktop smoke server failed: {error}");
        std::process::exit(1);
    }
}

async fn run_desktop_smoke_server_async() -> Result<(), String> {
    let config_path = desktop_smoke_config_path_from_env(
        env::var_os("HONE_CONFIG_PATH").map(PathBuf::from),
        env::var_os("HONE_USER_CONFIG_PATH").map(PathBuf::from),
    );
    let data_dir = desktop_smoke_data_dir_from_env(
        env::var_os("HONE_DESKTOP_DATA_DIR").map(PathBuf::from),
        env::var_os("HONE_DATA_DIR").map(PathBuf::from),
    );
    let skills_dir = env::var_os("HONE_SKILLS_DIR").map(PathBuf::from);

    unsafe {
        env::set_var("HONE_CONFIG_PATH", &config_path);
        if let Some(data_dir) = data_dir.as_deref() {
            env::set_var("HONE_DATA_DIR", data_dir);
        }
        env::set_var("HONE_DISABLE_AUTO_OPEN", "1");
    }

    let started = hone_web_api::start_server(
        &config_path.to_string_lossy(),
        data_dir.as_deref(),
        skills_dir.as_deref(),
        "local",
    )
    .await?;

    eprintln!(
        "Hone Desktop smoke server ready: admin=http://127.0.0.1:{} public={}",
        started.admin_port,
        started
            .public_port
            .map(|port| format!("http://127.0.0.1:{port}"))
            .unwrap_or_else(|| "disabled".to_string())
    );
    eprintln!("Press Ctrl-C to stop.");

    tokio::signal::ctrl_c()
        .await
        .map_err(|error| format!("failed to wait for shutdown signal: {error}"))?;

    for handle in started.task_handles {
        handle.abort();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_run_error_message_is_nonpanic_diagnostic() {
        let message = desktop_run_error_message("setup failed");

        assert_eq!(message, "Hone Desktop exited with error: setup failed");
        assert!(!message.contains("error while running hone desktop"));
    }

    #[test]
    fn desktop_smoke_server_env_parses_truthy_values() {
        for value in ["1", "true", "TRUE", "yes", "on", " on "] {
            assert!(desktop_smoke_server_enabled_from_env_value(Some(value)));
        }

        for value in [None, Some(""), Some("0"), Some("false"), Some("off")] {
            assert!(!desktop_smoke_server_enabled_from_env_value(value));
        }
    }

    #[test]
    fn desktop_smoke_config_prefers_runtime_config_then_user_config() {
        assert_eq!(
            desktop_smoke_config_path_from_env(
                Some(PathBuf::from("/tmp/effective.yaml")),
                Some(PathBuf::from("/tmp/config.yaml"))
            ),
            PathBuf::from("/tmp/effective.yaml")
        );
        assert_eq!(
            desktop_smoke_config_path_from_env(None, Some(PathBuf::from("/tmp/config.yaml"))),
            PathBuf::from("/tmp/config.yaml")
        );
        assert_eq!(
            desktop_smoke_config_path_from_env(None, None),
            PathBuf::from("config.yaml")
        );
    }

    #[test]
    fn desktop_smoke_data_dir_prefers_desktop_override() {
        assert_eq!(
            desktop_smoke_data_dir_from_env(
                Some(PathBuf::from("/tmp/desktop-data")),
                Some(PathBuf::from("/tmp/data"))
            ),
            Some(PathBuf::from("/tmp/desktop-data"))
        );
        assert_eq!(
            desktop_smoke_data_dir_from_env(None, Some(PathBuf::from("/tmp/data"))),
            Some(PathBuf::from("/tmp/data"))
        );
    }
}
