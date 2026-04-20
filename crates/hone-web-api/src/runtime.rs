use std::path::PathBuf;

use hone_core::config::HoneConfig;

const DEFAULT_PORT: u16 = 8077;
const DEFAULT_PUBLIC_PORT: u16 = 8088;

fn bundled_web_dist_dir() -> Option<PathBuf> {
    std::env::var_os("HONE_INSTALL_ROOT")
        .map(PathBuf::from)
        .map(|root| root.join("share").join("honeclaw").join("web"))
}

pub fn web_dist_dir() -> PathBuf {
    std::env::var("HONE_WEB_DIST_DIR")
        .map(PathBuf::from)
        .ok()
        .or_else(bundled_web_dist_dir)
        .unwrap_or_else(|| PathBuf::from("packages/app/dist"))
}

pub fn public_web_dist_dir() -> PathBuf {
    std::env::var("HONE_PUBLIC_WEB_DIST_DIR")
        .map(PathBuf::from)
        .ok()
        .or_else(|| {
            bundled_web_dist_dir().map(|root| root.parent().unwrap_or(&root).join("web-public"))
        })
        .unwrap_or_else(|| PathBuf::from("packages/app/dist-public"))
}

pub fn runtime_config_path() -> String {
    std::env::var("HONE_CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string())
}

pub fn runtime_port() -> u16 {
    std::env::var("HONE_WEB_PORT")
        .ok()
        .or_else(|| std::env::var("WEB_TEST_PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

pub fn runtime_public_port() -> Option<u16> {
    Some(
        std::env::var("HONE_PUBLIC_WEB_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(DEFAULT_PUBLIC_PORT),
    )
}

pub fn runtime_deployment_mode() -> String {
    std::env::var("HONE_DEPLOYMENT_MODE").unwrap_or_else(|_| "local".to_string())
}

pub fn runtime_disable_auto_open() -> bool {
    std::env::var("HONE_DISABLE_AUTO_OPEN")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

pub fn apply_runtime_config_overrides(config: &mut HoneConfig) {
    let config_path = std::env::var_os("HONE_CONFIG_PATH").map(PathBuf::from);
    let skills_dir = std::env::var_os("HONE_SKILLS_DIR").map(PathBuf::from);
    config.apply_runtime_overrides(None, skills_dir.as_deref(), config_path.as_deref());
}

pub fn ensure_runtime_dirs(config: &HoneConfig) {
    config.ensure_runtime_dirs();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn web_dist_dir_prefers_explicit_env_override() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            env::set_var("HONE_WEB_DIST_DIR", "/tmp/hone-explicit-web");
            env::set_var("HONE_INSTALL_ROOT", "/tmp/hone-install-root");
        }

        assert_eq!(web_dist_dir(), PathBuf::from("/tmp/hone-explicit-web"));

        unsafe {
            env::remove_var("HONE_WEB_DIST_DIR");
            env::remove_var("HONE_INSTALL_ROOT");
        }
    }

    #[test]
    fn web_dist_dir_uses_installed_bundle_layout_before_source_fallback() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            env::remove_var("HONE_WEB_DIST_DIR");
            env::set_var("HONE_INSTALL_ROOT", "/tmp/hone-install-root");
        }

        assert_eq!(
            web_dist_dir(),
            PathBuf::from("/tmp/hone-install-root/share/honeclaw/web")
        );

        unsafe {
            env::remove_var("HONE_INSTALL_ROOT");
        }
    }

    #[test]
    fn web_dist_dir_falls_back_to_source_tree_dist() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            env::remove_var("HONE_WEB_DIST_DIR");
            env::remove_var("HONE_INSTALL_ROOT");
        }

        assert_eq!(web_dist_dir(), PathBuf::from("packages/app/dist"));
    }

    #[test]
    fn runtime_ports_fall_back_to_fixed_defaults() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            env::remove_var("HONE_WEB_PORT");
            env::remove_var("WEB_TEST_PORT");
            env::remove_var("HONE_PUBLIC_WEB_PORT");
        }

        assert_eq!(runtime_port(), 8077);
        assert_eq!(runtime_public_port(), Some(8088));
    }

    #[test]
    fn runtime_ports_honor_explicit_overrides() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            env::set_var("HONE_WEB_PORT", "19077");
            env::set_var("HONE_PUBLIC_WEB_PORT", "19088");
            env::remove_var("WEB_TEST_PORT");
        }

        assert_eq!(runtime_port(), 19077);
        assert_eq!(runtime_public_port(), Some(19088));

        unsafe {
            env::remove_var("HONE_WEB_PORT");
            env::remove_var("HONE_PUBLIC_WEB_PORT");
        }
    }
}
