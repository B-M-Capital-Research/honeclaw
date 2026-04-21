use hone_core::actor::ActorIdentity;
use std::fs;
use std::io;
use std::path::PathBuf;

const HONE_AGENT_SANDBOX_DIR_ENV: &str = "HONE_AGENT_SANDBOX_DIR";
const HONE_DATA_DIR_ENV: &str = "HONE_DATA_DIR";

pub fn sandbox_base_dir() -> PathBuf {
    std::env::var(HONE_AGENT_SANDBOX_DIR_ENV)
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var(HONE_DATA_DIR_ENV)
                .map(PathBuf::from)
                .map(|path| path.join("agent-sandboxes"))
        })
        .unwrap_or_else(|_| std::env::temp_dir().join("hone-agent-sandboxes"))
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

#[cfg(test)]
mod tests {
    use super::{actor_sandbox_root, channel_download_dir, path_within_root};
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
    fn sandbox_base_dir_prefers_hone_data_dir_before_temp() {
        let temp = std::env::temp_dir().join("hone_sandbox_data_root");
        unsafe {
            std::env::remove_var("HONE_AGENT_SANDBOX_DIR");
            std::env::set_var("HONE_DATA_DIR", &temp);
        }
        assert_eq!(super::sandbox_base_dir(), temp.join("agent-sandboxes"));
        unsafe {
            std::env::remove_var("HONE_DATA_DIR");
        }
    }
}
