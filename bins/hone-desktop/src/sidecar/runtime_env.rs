use super::*;
use hone_core::config::runtime_overlay_path;

pub(super) fn normalize_base_url(raw: &str) -> String {
    raw.trim().trim_end_matches('/').to_string()
}

fn timestamp_string() -> String {
    let tz = chrono::FixedOffset::east_opt(8 * 3600).expect("valid tz");
    chrono::Utc::now()
        .with_timezone(&tz)
        .format("%Y-%m-%d %H:%M:%S%.3f")
        .to_string()
}

pub(super) fn append_log(path: &PathBuf, level: &str, message: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "[{}] {:<5} {}", timestamp_string(), level, message);
    }
}

pub(super) fn diagnostic_paths(app: &AppHandle) -> Result<DiagnosticPaths, String> {
    let config_dir = desktop_config_dir(app)?;
    let data_dir = if let Ok(override_dir) = std::env::var("HONE_DESKTOP_DATA_DIR") {
        PathBuf::from(override_dir)
    } else {
        app.path().app_data_dir().map_err(|e| e.to_string())?
    };
    let logs_dir = data_dir.join("runtime").join("logs");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;

    Ok(DiagnosticPaths {
        config_dir: config_dir.to_string_lossy().to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        logs_dir: logs_dir.to_string_lossy().to_string(),
        desktop_log: logs_dir.join("desktop.log").to_string_lossy().to_string(),
        sidecar_log: logs_dir.join("sidecar.log").to_string_lossy().to_string(),
    })
}

pub(super) fn log_desktop(app: &AppHandle, level: &str, message: impl AsRef<str>) {
    if let Ok(paths) = diagnostic_paths(app) {
        append_log(&PathBuf::from(paths.desktop_log), level, message.as_ref());
    }
}

pub(super) fn desktop_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(override_dir) = std::env::var("HONE_DESKTOP_CONFIG_DIR") {
        let path = PathBuf::from(override_dir);
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        return Ok(path);
    }

    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub(super) fn config_store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = desktop_config_dir(app)?;
    Ok(dir.join("backend.json"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn resource_or_repo_path(app: &AppHandle, resource: &str) -> PathBuf {
    app.path()
        .resource_dir()
        .ok()
        .map(|dir| dir.join(resource))
        .filter(|path| path.exists())
        .unwrap_or_else(|| repo_root().join(resource))
}

fn current_target_triple() -> Option<String> {
    let arch = match env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        "x86" => "i686",
        other => other,
    };
    let os = match env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        _ => return None,
    };
    Some(format!("{arch}-{os}"))
}

fn bundled_binary_candidate_names(binary: &str) -> Vec<String> {
    let mut names = Vec::new();
    let base = if cfg!(windows) {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    };
    names.push(base);

    if let Some(triple) = current_target_triple() {
        let suffixed = if cfg!(windows) {
            format!("{binary}-{triple}.exe")
        } else {
            format!("{binary}-{triple}")
        };
        names.push(suffixed);
    }

    names
}

fn bundled_binary_search_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            dirs.push(parent.to_path_buf());
        }
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        dirs.push(resource_dir.clone());
        dirs.push(resource_dir.join("binaries"));
    }

    dirs
}

fn resolve_bundled_binary(app: &AppHandle, binary: &str) -> Option<PathBuf> {
    for dir in bundled_binary_search_dirs(app) {
        for name in bundled_binary_candidate_names(binary) {
            let candidate = dir.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn set_process_env(key: &str, value: impl AsRef<std::ffi::OsStr>) {
    unsafe {
        env::set_var(key, value);
    }
}

fn prepend_path_entry(dir: &Path) {
    let mut entries = vec![dir.to_path_buf()];
    if let Some(existing) = env::var_os("PATH") {
        entries.extend(env::split_paths(&existing));
    }
    if let Ok(joined) = env::join_paths(entries) {
        set_process_env("PATH", joined);
    }
}

fn should_import_shell_env_key(key: &str) -> bool {
    matches!(
        key,
        "PATH"
            | "HOME"
            | "USER"
            | "LOGNAME"
            | "SHELL"
            | "LANG"
            | "TMPDIR"
            | "TERM"
            | "COLORTERM"
            | "SSH_AUTH_SOCK"
            | "SSL_CERT_FILE"
            | "SSL_CERT_DIR"
            | "HTTP_PROXY"
            | "HTTPS_PROXY"
            | "ALL_PROXY"
            | "NO_PROXY"
    ) || key.starts_with("LC_")
        || key.starts_with("HOMEBREW_")
        || key.starts_with("BUN_")
        || key.starts_with("CARGO_")
        || key.starts_with("RUSTUP_")
        || key.starts_with("OPENAI_")
        || key.starts_with("OPENROUTER_")
        || key.starts_with("GEMINI_")
        || key.starts_with("ANTHROPIC_")
        || key.starts_with("NVM_")
        || key.starts_with("VOLTA_")
        || key.starts_with("ASDF_")
}

fn hydrate_login_shell_env(app: &AppHandle) {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let output = match StdCommand::new(&shell).args(["-lc", "env -0"]).output() {
        Ok(output) => output,
        Err(error) => {
            log_desktop(
                app,
                "WARN",
                format!("failed to hydrate login shell env via {shell}: {error}"),
            );
            return;
        }
    };

    if !output.status.success() {
        log_desktop(
            app,
            "WARN",
            format!(
                "login shell env command exited with status {}",
                output.status
            ),
        );
        return;
    }

    for pair in output.stdout.split(|byte| *byte == 0) {
        if pair.is_empty() {
            continue;
        }
        let Ok(rendered) = std::str::from_utf8(pair) else {
            continue;
        };
        let Some((key, value)) = rendered.split_once('=') else {
            continue;
        };
        if should_import_shell_env_key(key) {
            set_process_env(key, value);
        }
    }
}

pub(super) fn configure_desktop_runtime_env(app: &AppHandle, runtime: &RuntimePaths) {
    hydrate_login_shell_env(app);

    set_process_env("HONE_CONFIG_PATH", &runtime.config_path);
    set_process_env("HONE_DATA_DIR", &runtime.data_dir);
    set_process_env("HONE_RUNTIME_DIR", &runtime.runtime_dir);
    set_process_env("HONE_SKILLS_DIR", &runtime.skills_dir);
    set_process_env(
        "HONE_AGENT_SANDBOX_DIR",
        runtime.data_dir.join("agent-sandboxes"),
    );

    if let Some(mcp_path) = resolve_bundled_binary(app, "hone-mcp") {
        set_process_env("HONE_MCP_BIN", &mcp_path);
    }
    if let Some(opencode_path) = resolve_bundled_binary(app, "opencode") {
        if let Some(parent) = opencode_path.parent() {
            prepend_path_entry(parent);
        }
        set_process_env("HONE_BUNDLED_OPENCODE_BIN", &opencode_path);
    }
}

pub(super) fn ensure_runtime_paths(app: &AppHandle) -> Result<RuntimePaths, String> {
    let config_dir = desktop_config_dir(app)?;

    let data_dir = if let Ok(override_dir) = std::env::var("HONE_DESKTOP_DATA_DIR") {
        PathBuf::from(override_dir)
    } else if cfg!(debug_assertions) {
        let dev_data = repo_root().join("data");
        if dev_data.is_dir() {
            dev_data
        } else {
            app.path().app_data_dir().map_err(|e| e.to_string())?
        }
    } else {
        app.path().app_data_dir().map_err(|e| e.to_string())?
    };
    let runtime_dir = data_dir.join("runtime");
    let logs_dir = runtime_dir.join("logs");
    let locks_dir = runtime_dir.join("locks");
    let sandbox_dir = data_dir.join("agent-sandboxes");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&runtime_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&locks_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&sandbox_dir).map_err(|e| e.to_string())?;

    let config_path = runtime_dir.join("config_runtime.yaml");
    if !config_path.exists() {
        let seed = {
            let root_config = resource_or_repo_path(app, "config.yaml");
            if root_config.exists() {
                root_config
            } else {
                resource_or_repo_path(app, "config.example.yaml")
            }
        };
        fs::copy(&seed, &config_path)
            .map_err(|e| format!("无法初始化 config_runtime.yaml（来源: {seed:?}）: {e}"))?;
        let overlay_path = runtime_overlay_path(&config_path);
        if overlay_path.exists() {
            let _ = fs::remove_file(&overlay_path);
        }
    }

    let soul_dest = runtime_dir.join("soul.md");
    if !soul_dest.exists() {
        let soul_src = resource_or_repo_path(app, "soul.md");
        if soul_src.exists() {
            fs::copy(&soul_src, &soul_dest)
                .map_err(|e| format!("无法复制 soul.md 到 runtime 目录: {e}"))?;
        }
    }

    let skills_dir = resource_or_repo_path(app, "skills");
    Ok(RuntimePaths {
        config_path,
        data_dir,
        runtime_dir,
        skills_dir,
    })
}
