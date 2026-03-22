use tauri::{AppHandle, State};

use crate::sidecar::{
    AgentSettings, BackendConfig, BackendStatusInfo, CliCheckResult, DesktopChannelSettings,
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
pub(crate) fn get_agent_settings(app: AppHandle) -> Result<AgentSettings, String> {
    crate::sidecar::get_agent_settings_impl(app)
}

#[tauri::command]
pub(crate) async fn set_agent_settings(
    app: AppHandle,
    state: State<'_, DesktopState>,
    settings: AgentSettings,
) -> Result<(), String> {
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
    crate::tray::setup_tray();

    tauri::Builder::default()
        .manage(DesktopState::default())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_backend_config,
            set_backend_config,
            connect_backend,
            start_bundled_backend,
            stop_bundled_backend,
            get_channel_settings,
            set_channel_settings,
            backend_status,
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
        .run(tauri::generate_context!())
        .expect("error while running hone desktop");
}
