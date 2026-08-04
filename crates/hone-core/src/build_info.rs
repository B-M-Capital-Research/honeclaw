use std::fs::File;
use std::io::Read;
use std::sync::LazyLock;

use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildSource {
    Workspace,
    DirectSourceRuntime,
    GhcrLinuxOci,
    Unknown,
}

impl BuildSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::DirectSourceRuntime => "direct_source_runtime",
            Self::GhcrLinuxOci => "ghcr_linux_oci",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    pub git_sha: Option<String>,
    pub build_timestamp: Option<String>,
    pub profile: &'static str,
    pub source: BuildSource,
    pub binary_sha256: Option<String>,
}

static CURRENT_BUILD_INFO: LazyLock<BuildInfo> = LazyLock::new(|| BuildInfo {
    git_sha: sanitized_build_value(option_env!("HONE_BUILD_GIT_SHA")),
    build_timestamp: sanitized_build_value(option_env!("HONE_BUILD_TIMESTAMP")),
    profile: if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    },
    source: normalized_build_source(option_env!("HONE_BUILD_SOURCE")),
    binary_sha256: current_binary_sha256(),
});

pub fn current_build_info() -> BuildInfo {
    CURRENT_BUILD_INFO.clone()
}

fn sanitized_build_value(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| {
            value.len() <= 128
                && value.chars().all(|ch| {
                    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.' | '+')
                })
        })
        .map(ToString::to_string)
}

fn normalized_build_source(raw: Option<&str>) -> BuildSource {
    match raw.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("workspace") => BuildSource::Workspace,
        Some("direct_source_runtime") => BuildSource::DirectSourceRuntime,
        Some("ghcr_linux_oci") => BuildSource::GhcrLinuxOci,
        Some(_) => BuildSource::Unknown,
    }
}

fn current_binary_sha256() -> Option<String> {
    let executable = std::env::current_exe().ok()?;
    let mut file = File::open(executable).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Some(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{BuildSource, normalized_build_source, sanitized_build_value};

    #[test]
    fn build_metadata_accepts_bounded_identifiers_and_rejects_paths() {
        assert_eq!(
            sanitized_build_value(Some("abc123-2026-08-02T01:02:03+08:00")),
            Some("abc123-2026-08-02T01:02:03+08:00".to_string())
        );
        assert_eq!(sanitized_build_value(Some(" /tmp/private/build ")), None);
        assert_eq!(sanitized_build_value(Some("secret value")), None);
    }

    #[test]
    fn build_source_is_a_bounded_provenance_kind_not_a_path() {
        assert_eq!(normalized_build_source(None), BuildSource::Workspace);
        assert_eq!(
            normalized_build_source(Some("direct_source_runtime")),
            BuildSource::DirectSourceRuntime
        );
        assert_eq!(
            normalized_build_source(Some("ghcr_linux_oci")),
            BuildSource::GhcrLinuxOci
        );
        assert_eq!(
            normalized_build_source(Some("/private/tmp/custom-build")),
            BuildSource::Unknown
        );
    }
}
