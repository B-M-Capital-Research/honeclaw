use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use clap::{Args, Subcommand};
use reqwest::StatusCode;
use tokio::process::Command;

use crate::common::{load_cli_config, resolve_runtime_paths};
use crate::start;

const DEFAULT_PUBLIC_WEB_PORT: u16 = 8088;

#[derive(Subcommand, Debug)]
pub(crate) enum WebCommands {
    /// 构建并启动管理后台 Web UI。
    AdminUi(WebUiArgs),
    /// 构建并启动公开用户端 Web UI。
    UserUi(WebUiArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct WebUiArgs {
    /// 源码 checkout 根目录；默认从当前目录向上查找，或读取 HONE_SOURCE_ROOT。
    #[arg(long, value_name = "DIR")]
    pub(crate) source_root: Option<PathBuf>,
    /// 使用 Vite dev server，不先构建静态产物。
    #[arg(long)]
    pub(crate) dev: bool,
    /// 使用已有静态产物启动 Vite preview，跳过本次 build。
    #[arg(long)]
    pub(crate) no_build: bool,
    /// 前端服务端口；管理端默认 3000，用户端默认 3001。
    #[arg(long)]
    pub(crate) port: Option<u16>,
    /// 前端开发代理要连接的后端地址；默认读取本地 runtime web port。
    #[arg(long)]
    pub(crate) backend_url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WebSurface {
    Admin,
    User,
}

impl WebSurface {
    fn cli_name(self) -> &'static str {
        match self {
            Self::Admin => "admin-ui",
            Self::User => "user-ui",
        }
    }

    fn app_surface(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "public",
        }
    }

    fn default_frontend_port(self) -> u16 {
        match self {
            Self::Admin => 3000,
            Self::User => 3001,
        }
    }

    fn static_out_dir(self) -> &'static str {
        match self {
            Self::Admin => "dist",
            Self::User => "dist-public",
        }
    }

    fn bundled_ready_url(self, admin_port: u16, public_port: u16) -> String {
        match self {
            Self::Admin => format!("http://127.0.0.1:{admin_port}/api/meta"),
            Self::User => format!("http://127.0.0.1:{public_port}/"),
        }
    }

    fn bundled_display_url(self, admin_port: u16, public_port: u16) -> String {
        match self {
            Self::Admin => format!("http://127.0.0.1:{admin_port}"),
            Self::User => format!("http://127.0.0.1:{public_port}"),
        }
    }
}

pub(crate) async fn run_web_command(
    explicit_config: Option<&Path>,
    command: WebCommands,
) -> Result<(), String> {
    match command {
        WebCommands::AdminUi(args) => run_web_ui(explicit_config, WebSurface::Admin, args).await,
        WebCommands::UserUi(args) => run_web_ui(explicit_config, WebSurface::User, args).await,
    }
}

async fn run_web_ui(
    explicit_config: Option<&Path>,
    surface: WebSurface,
    args: WebUiArgs,
) -> Result<(), String> {
    let source_root = frontend_source_root(&args)?;
    if let Some(source_root) = source_root {
        run_source_frontend(explicit_config, surface, args, &source_root).await
    } else {
        run_bundled_frontend(explicit_config, surface, args).await
    }
}

fn frontend_source_root(args: &WebUiArgs) -> Result<Option<PathBuf>, String> {
    let explicit = args.source_root.as_deref();
    let source_root = start::source_root_from_env_or_cwd(explicit);
    match source_root {
        Some(root) if has_frontend_source(&root) => Ok(Some(root)),
        Some(root) if explicit.is_some() => Err(format!(
            "{} 不是可用的 Hone 源码 checkout；需要包含 package.json 和 packages/app/package.json",
            root.display()
        )),
        Some(_) if args.dev || args.no_build => Err(
            "找不到可用的 Hone 前端源码；`--dev` / `--no-build` 需要在源码 checkout 中运行，或传入 --source-root"
                .to_string(),
        ),
        Some(_) | None if explicit.is_some() => Err(
            "找不到可用的 Hone 前端源码；请检查 --source-root 是否指向仓库根目录".to_string(),
        ),
        Some(_) | None if args.dev || args.no_build => Err(
            "找不到可用的 Hone 前端源码；`--dev` / `--no-build` 需要在源码 checkout 中运行，或传入 --source-root"
                .to_string(),
        ),
        Some(_) | None => Ok(None),
    }
}

fn has_frontend_source(root: &Path) -> bool {
    root.join("package.json").is_file() && root.join("packages/app/package.json").is_file()
}

async fn run_source_frontend(
    explicit_config: Option<&Path>,
    surface: WebSurface,
    args: WebUiArgs,
    source_root: &Path,
) -> Result<(), String> {
    let paths = resolve_runtime_paths(explicit_config, false).map_err(|e| e.to_string())?;
    let port = args.port.unwrap_or_else(|| surface.default_frontend_port());
    let backend_url = args
        .backend_url
        .unwrap_or_else(|| format!("http://127.0.0.1:{}", paths.web_port));
    let app_dir = source_root.join("packages/app");
    let envs = frontend_envs(surface, port, &backend_url);

    if args.dev {
        println!(
            "[INFO] starting {} Vite dev server at http://127.0.0.1:{port}",
            surface.cli_name()
        );
        println!("[INFO] backend proxy: {backend_url}");
        let command_args = vec!["run".to_string(), "dev".to_string()];
        return run_bun_command(&app_dir, &command_args, &envs, "Vite dev server").await;
    }

    if !args.no_build {
        println!(
            "[INFO] building {} web assets in {}",
            surface.cli_name(),
            app_dir.display()
        );
        let command_args = vec!["run".to_string(), "build".to_string()];
        run_bun_command(&app_dir, &command_args, &envs, "Web build").await?;
    }

    println!(
        "[INFO] starting {} preview at http://127.0.0.1:{port}",
        surface.cli_name()
    );
    println!("[INFO] backend proxy: {backend_url}");
    let command_args = vec![
        "run".to_string(),
        "preview".to_string(),
        "--".to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        port.to_string(),
        "--outDir".to_string(),
        surface.static_out_dir().to_string(),
    ];
    run_bun_command(&app_dir, &command_args, &envs, "Vite preview").await
}

fn frontend_envs(surface: WebSurface, port: u16, backend_url: &str) -> Vec<(String, String)> {
    vec![
        (
            "HONE_APP_SURFACE".to_string(),
            surface.app_surface().to_string(),
        ),
        (
            "HONE_APP_OUT_DIR".to_string(),
            surface.static_out_dir().to_string(),
        ),
        ("HONE_APP_PORT".to_string(), port.to_string()),
        ("HONE_WEB_BACKEND_URL".to_string(), backend_url.to_string()),
    ]
}

async fn run_bun_command(
    current_dir: &Path,
    args: &[String],
    envs: &[(String, String)],
    label: &str,
) -> Result<(), String> {
    let mut command = Command::new("bun");
    command
        .args(args)
        .current_dir(current_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    for (key, value) in envs {
        command.env(key, value);
    }
    let status = command
        .status()
        .await
        .map_err(|e| format!("运行 bun 失败：{e}"))?;
    if !status.success() {
        return Err(format!("{label} 失败（status={status}）"));
    }
    Ok(())
}

async fn run_bundled_frontend(
    explicit_config: Option<&Path>,
    surface: WebSurface,
    args: WebUiArgs,
) -> Result<(), String> {
    if args.dev || args.no_build || args.source_root.is_some() {
        return Err(
            "安装包模式不支持 --dev / --no-build / --source-root；需要源码模式时请在仓库根目录运行"
                .to_string(),
        );
    }
    if args.port.is_some() || args.backend_url.is_some() {
        return Err(
            "安装包模式不支持 --port / --backend-url；请通过 HONE_WEB_PORT / HONE_PUBLIC_WEB_PORT 覆盖运行时端口，或在源码模式使用这些参数"
                .to_string(),
        );
    }

    let (_, paths) = load_cli_config(explicit_config, true).map_err(|e| e.to_string())?;
    let admin_port = paths.web_port;
    let public_port = public_web_port_from_env();
    let ready_url = surface.bundled_ready_url(admin_port, public_port);
    let display_url = surface.bundled_display_url(admin_port, public_port);

    if http_ready_once(&ready_url).await? {
        println!(
            "[INFO] {} already available at {display_url}",
            surface.cli_name()
        );
        return Ok(());
    }

    let lock_names = [hone_core::PROCESS_LOCK_CONSOLE_PAGE];
    let mut on_warn = |message: &str| println!("[WARN] {message}");
    if let Err(error) = hone_core::preflight_process_locks_with_cleanup(
        &paths.runtime_dir,
        &lock_names,
        &mut on_warn,
    ) {
        return Err(hone_core::format_lock_failure_message(
            &error.process,
            &error.path,
            &error.source,
            "Hone CLI",
        ));
    }

    println!(
        "[INFO] starting bundled web server for {} using {}",
        surface.cli_name(),
        paths.effective_config_path.to_string_lossy()
    );
    let mut child = start::spawn_binary("hone-console-page", &paths, &[], None).await?;
    start::ensure_child_alive("hone-console-page", &mut child).await?;
    start::wait_for_http_ready(&ready_url).await?;
    println!("[INFO] {} ready at {display_url}", surface.cli_name());
    println!("[INFO] press Ctrl-C to stop.");

    let mut ctrl_c = Box::pin(tokio::signal::ctrl_c());
    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                println!();
                println!("[INFO] shutdown requested");
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Ok(());
            }
            _ = tokio::time::sleep(Duration::from_millis(300)) => {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        return Err(format!("hone-console-page exited unexpectedly: {status}"));
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(format!("检查 hone-console-page 进程状态失败：{error}"));
                    }
                }
            }
        }
    }
}

fn public_web_port_from_env() -> u16 {
    std::env::var("HONE_PUBLIC_WEB_PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PUBLIC_WEB_PORT)
}

async fn http_ready_once(url: &str) -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    match client.get(url).send().await {
        Ok(response) => Ok(response.status() == StatusCode::OK),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_source_root(prefix: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("packages/app")).unwrap();
        std::fs::write(root.join("package.json"), "{}").unwrap();
        std::fs::write(root.join("packages/app/package.json"), "{}").unwrap();
        root
    }

    #[test]
    fn frontend_envs_pin_surface_and_out_dir() {
        let admin = frontend_envs(WebSurface::Admin, 3000, "http://127.0.0.1:8077");
        assert!(
            admin
                .iter()
                .any(|(key, value)| key == "HONE_APP_SURFACE" && value == "admin")
        );
        assert!(
            admin
                .iter()
                .any(|(key, value)| key == "HONE_APP_OUT_DIR" && value == "dist")
        );

        let user = frontend_envs(WebSurface::User, 3001, "http://127.0.0.1:8088");
        assert!(
            user.iter()
                .any(|(key, value)| key == "HONE_APP_SURFACE" && value == "public")
        );
        assert!(
            user.iter()
                .any(|(key, value)| key == "HONE_APP_OUT_DIR" && value == "dist-public")
        );
    }

    #[test]
    fn explicit_source_root_requires_frontend_package_files() {
        let root = std::env::temp_dir().join(format!(
            "hone_missing_frontend_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let args = WebUiArgs {
            source_root: Some(root.clone()),
            ..WebUiArgs::default()
        };

        let error = frontend_source_root(&args).expect_err("invalid source root should fail");
        assert!(error.contains("package.json"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn explicit_source_root_detects_frontend_source() {
        let root = temp_source_root("hone_frontend_source");
        let args = WebUiArgs {
            source_root: Some(root.clone()),
            ..WebUiArgs::default()
        };

        let resolved = frontend_source_root(&args).expect("source root should resolve");
        assert_eq!(resolved, Some(root.clone()));

        let _ = std::fs::remove_dir_all(root);
    }
}
