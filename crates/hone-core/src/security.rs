use std::fs;
use std::io;
use std::path::Path;

pub fn harden_private_dir(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;
    set_mode(path, 0o700)
}

pub fn harden_private_file(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(());
    }
    set_mode(path, 0o600)
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::{harden_private_dir, harden_private_file};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn private_paths_are_owner_only() {
        let root = std::env::temp_dir().join(format!(
            "hone_private_permissions_{}_{}",
            std::process::id(),
            crate::beijing_now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        ));
        std::fs::create_dir_all(&root).expect("root");
        let file = root.join("config.yaml");
        std::fs::write(&file, "secret").expect("file");

        harden_private_dir(&root).expect("harden dir");
        harden_private_file(&file).expect("harden file");

        assert_eq!(
            std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
