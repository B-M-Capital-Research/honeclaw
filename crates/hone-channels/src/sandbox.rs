use hone_core::actor::ActorIdentity;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const HONE_AGENT_SANDBOX_DIR_ENV: &str = "HONE_AGENT_SANDBOX_DIR";

pub fn sandbox_base_dir() -> PathBuf {
    let fallback = safe_sandbox_base_dir();
    std::env::var(HONE_AGENT_SANDBOX_DIR_ENV)
        .map(PathBuf::from)
        .ok()
        .filter(|path| !is_inside_current_repo(path))
        .unwrap_or(fallback)
}

pub(crate) fn actor_sandbox_root(actor: &ActorIdentity) -> PathBuf {
    sandbox_base_dir()
        .join(actor.channel_fs_component())
        .join(actor.scoped_user_fs_key())
}

pub(crate) fn ensure_actor_sandbox(actor: &ActorIdentity) -> io::Result<PathBuf> {
    let root = actor_sandbox_root(actor);
    fs::create_dir_all(root.join("uploads"))?;
    fs::create_dir_all(root.join("runtime"))?;
    // Pre-create company_profiles so tools return an empty listing rather than
    // "directory not found", which causes repeated search retries in the model.
    fs::create_dir_all(root.join("company_profiles"))?;
    remove_sensitive_legacy_files(&root)?;
    Ok(root)
}

pub(crate) fn actor_upload_dir(actor: &ActorIdentity, session_id: &str) -> PathBuf {
    actor_sandbox_root(actor).join("uploads").join(session_id)
}

pub fn channel_download_dir(channel: &str) -> PathBuf {
    sandbox_base_dir()
        .join("downloads")
        .join(sanitize_component(channel))
}

#[cfg(test)]
pub(crate) fn sandbox_env_test_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

#[cfg(test)]
fn path_within_root(root: &std::path::Path, candidate: &std::path::Path) -> bool {
    match (fs::canonicalize(root), fs::canonicalize(candidate)) {
        (Ok(root), Ok(candidate)) => candidate.starts_with(root),
        _ => false,
    }
}

fn sanitize_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        if byte.is_ascii_alphanumeric() || *byte == b'-' {
            out.push(char::from(*byte));
        } else {
            out.push('_');
            out.push_str(&format!("{byte:02x}"));
        }
    }
    out
}

fn safe_sandbox_base_dir() -> PathBuf {
    std::env::temp_dir().join("hone-agent-sandboxes")
}

fn is_inside_current_repo(path: &Path) -> bool {
    let Ok(current_dir) = std::env::current_dir() else {
        return false;
    };
    let Some(repo_root) = nearest_git_root(&current_dir) else {
        return false;
    };
    let candidate = path
        .canonicalize()
        .unwrap_or_else(|_| absolutize_lossy(path));
    candidate.starts_with(repo_root)
}

fn nearest_git_root(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        if ancestor.join(".git").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn absolutize_lossy(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn remove_sensitive_legacy_files(root: &Path) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let file_type = entry.file_type()?;
        let is_portfolio_json = file_type.is_file()
            && name.starts_with("portfolio_")
            && path.extension().and_then(|value| value.to_str()) == Some("json");
        let is_portfolio_dir =
            file_type.is_dir() && matches!(name.as_ref(), "portfolio" | "portfolios");

        if is_portfolio_json {
            fs::remove_file(path)?;
        } else if is_portfolio_dir {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        actor_sandbox_root, channel_download_dir, path_within_root, safe_sandbox_base_dir,
    };
    use hone_core::actor::ActorIdentity;
    use std::path::Path;

    #[test]
    fn actor_root_uses_channel_then_scoped_identity() {
        let actor = ActorIdentity::new("discord", "4836", Some("direct")).expect("actor");
        let root = actor_sandbox_root(&actor);
        let rendered = root.to_string_lossy();
        assert!(rendered.contains("discord/direct__4836"));
    }

    #[test]
    fn path_within_root_rejects_siblings() {
        let root = Path::new("/tmp");
        let sibling = Path::new("/var/tmp");
        assert!(!path_within_root(root, sibling));
    }

    #[test]
    fn channel_download_dir_honors_env_override() {
        let _guard = super::sandbox_env_test_lock().lock().expect("env lock");
        let temp = std::env::temp_dir().join("hone_sandbox_test_root");
        unsafe {
            std::env::set_var("HONE_AGENT_SANDBOX_DIR", &temp);
        }
        let dir = channel_download_dir("telegram");
        assert_eq!(dir, temp.join("downloads").join("telegram"));
        unsafe {
            std::env::remove_var("HONE_AGENT_SANDBOX_DIR");
        }
    }

    #[test]
    fn sandbox_base_dir_falls_back_to_temp_not_data_dir() {
        let _guard = super::sandbox_env_test_lock().lock().expect("env lock");
        let temp = std::env::temp_dir().join("hone_sandbox_data_root");
        unsafe {
            std::env::remove_var("HONE_AGENT_SANDBOX_DIR");
            std::env::set_var("HONE_DATA_DIR", &temp);
        }
        assert_eq!(super::sandbox_base_dir(), safe_sandbox_base_dir());
        unsafe {
            std::env::remove_var("HONE_DATA_DIR");
        }
    }

    #[test]
    fn ensure_actor_sandbox_removes_legacy_portfolio_files() {
        let _guard = super::sandbox_env_test_lock().lock().expect("env lock");
        let temp = std::env::temp_dir().join(format!(
            "hone_sandbox_sensitive_cleanup_{}_{}",
            std::process::id(),
            hone_core::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        unsafe {
            std::env::set_var("HONE_AGENT_SANDBOX_DIR", &temp);
        }
        let actor = ActorIdentity::new("web", "alice", None::<String>).expect("actor");
        let root = super::actor_sandbox_root(&actor);
        std::fs::create_dir_all(root.join("portfolio")).expect("portfolio dir");
        std::fs::write(root.join("portfolio_web__direct__bob.json"), "{}").expect("portfolio json");
        std::fs::write(root.join("notes.md"), "keep").expect("notes");

        let root = super::ensure_actor_sandbox(&actor).expect("ensure");
        assert!(!root.join("portfolio").exists());
        assert!(!root.join("portfolio_web__direct__bob.json").exists());
        assert!(root.join("notes.md").exists());

        let _ = std::fs::remove_dir_all(&temp);
        unsafe {
            std::env::remove_var("HONE_AGENT_SANDBOX_DIR");
        }
    }
}
