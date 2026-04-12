use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hone_channels::HoneBotCore;
use hone_core::config::{
    canonical_config_candidate, effective_config_path, ensure_canonical_config,
    generate_effective_config, legacy_runtime_config_path, legacy_runtime_warning,
};
use hone_core::{HoneConfig, HoneResult};

#[derive(Debug, Clone)]
pub(crate) struct ResolvedRuntimePaths {
    pub(crate) canonical_config_path: PathBuf,
    pub(crate) effective_config_path: PathBuf,
    pub(crate) legacy_runtime_config_path: PathBuf,
    pub(crate) data_dir: PathBuf,
    pub(crate) runtime_dir: PathBuf,
    pub(crate) skills_dir: PathBuf,
    pub(crate) root_dir: PathBuf,
    pub(crate) web_port: u16,
    pub(crate) legacy_warning: Option<String>,
}

fn absolute_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn configured_root_dir(canonical_config_path: &Path) -> PathBuf {
    if let Some(home) = env::var_os("HONE_HOME") {
        return absolute_path(PathBuf::from(home));
    }
    if let Some(path) = env::var_os("HONE_USER_CONFIG_PATH") {
        let path = absolute_path(PathBuf::from(path));
        if let Some(parent) = path.parent() {
            return parent.to_path_buf();
        }
    }
    canonical_config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn resolve_canonical_config_path(explicit_config: Option<&Path>) -> PathBuf {
    absolute_path(
        explicit_config
            .map(Path::to_path_buf)
            .or_else(|| env::var_os("HONE_USER_CONFIG_PATH").map(PathBuf::from))
            .or_else(|| {
                env::var_os("HONE_HOME").map(|home| PathBuf::from(home).join("config.yaml"))
            })
            .unwrap_or_else(canonical_config_candidate),
    )
}

fn canonical_seed_source(root_dir: &Path, canonical_config_path: &Path) -> Option<PathBuf> {
    if canonical_config_path.exists() {
        return None;
    }
    let example = root_dir.join("config.example.yaml");
    if example.exists() {
        return Some(example);
    }
    let canonical = root_dir.join("config.yaml");
    if canonical.exists() && canonical != canonical_config_path {
        return Some(canonical);
    }
    None
}

pub(crate) fn resolve_runtime_paths(
    explicit_config: Option<&Path>,
    _for_write: bool,
) -> HoneResult<ResolvedRuntimePaths> {
    let canonical_config_path = resolve_canonical_config_path(explicit_config);
    let root_dir = configured_root_dir(&canonical_config_path);
    let data_dir = env::var_os("HONE_DATA_DIR")
        .map(PathBuf::from)
        .map(absolute_path)
        .unwrap_or_else(|| root_dir.join("data"));
    let runtime_dir = data_dir.join("runtime");
    let effective_config_path = effective_config_path(&runtime_dir);
    let legacy_runtime_config_path = legacy_runtime_config_path(&runtime_dir);
    let skills_dir = env::var_os("HONE_SKILLS_DIR")
        .map(PathBuf::from)
        .map(absolute_path)
        .unwrap_or_else(|| root_dir.join("skills"));
    let web_port = env::var("HONE_WEB_PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(8077);
    let legacy_warning =
        legacy_runtime_warning(&canonical_config_path, &legacy_runtime_config_path);

    Ok(ResolvedRuntimePaths {
        canonical_config_path,
        effective_config_path,
        legacy_runtime_config_path,
        data_dir,
        runtime_dir,
        skills_dir,
        root_dir,
        web_port,
        legacy_warning,
    })
}

pub(crate) fn load_cli_config(
    explicit_config: Option<&Path>,
    for_write: bool,
) -> HoneResult<(HoneConfig, ResolvedRuntimePaths)> {
    let paths = resolve_runtime_paths(explicit_config, for_write)?;
    let seed_source = if explicit_config.is_none() && for_write {
        canonical_seed_source(&paths.root_dir, &paths.canonical_config_path)
    } else {
        None
    };
    ensure_canonical_config(
        &paths.canonical_config_path,
        &paths.legacy_runtime_config_path,
        seed_source.as_deref(),
    )?;
    if for_write {
        let _ =
            generate_effective_config(&paths.canonical_config_path, &paths.effective_config_path)?;
    }
    let mut config = HoneConfig::from_file(&paths.canonical_config_path)?;
    config.apply_runtime_overrides(
        Some(paths.data_dir.as_path()),
        Some(paths.skills_dir.as_path()),
        Some(paths.canonical_config_path.as_path()),
    );
    config.ensure_runtime_dirs();
    Ok((config, paths))
}

pub(crate) fn load_cli_core(
    explicit_config: Option<&Path>,
) -> HoneResult<(Arc<HoneBotCore>, ResolvedRuntimePaths)> {
    let (config, paths) = load_cli_config(explicit_config, false)?;
    Ok((Arc::new(HoneBotCore::new(config)), paths))
}
