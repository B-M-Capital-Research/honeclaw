//! ACP adapter 版本选择与有界运行时证据。
//!
//! 方言选择只信任当前连接的 `initialize.agentInfo.version`；CLI 版本解析
//! 仅供 companion 工具下限检查。未知或不兼容的 adapter 版本会在创建会话
//! 和发送 prompt 前失败，不会退回静态 runner 标签。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::runners::types::{
    AcpAdapterKind, AcpAdapterProfile, AcpCompatibilityStatus, AcpStreamDialect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct CliVersion {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
}

pub(crate) const CODEX_ACP_BASELINE_VERSION: CliVersion = CliVersion {
    major: 1,
    minor: 1,
    patch: 7,
};

pub(crate) const OPENCODE_BASELINE_VERSION: CliVersion = CliVersion {
    major: 1,
    minor: 18,
    patch: 11,
};

const CODEX_ACP_INITIALIZE_AGENT_NAME: &str = "@agentclientprotocol/codex-acp";
const OPENCODE_INITIALIZE_AGENT_NAME: &str = "opencode";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AcpRuntimeProfileRecord {
    pub(crate) runner: String,
    pub(crate) adapter: AcpAdapterKind,
    pub(crate) detected_version: String,
    pub(crate) baseline_version: String,
    pub(crate) dialect: AcpStreamDialect,
    pub(crate) compatibility: AcpCompatibilityStatus,
    pub(crate) companion_versions: BTreeMap<String, String>,
    pub(crate) detected_at: String,
    pub(crate) build_git_sha: Option<String>,
}

impl std::fmt::Display for CliVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub(crate) fn parse_cli_version(raw: &str) -> Option<CliVersion> {
    raw.split(|ch: char| !ch.is_ascii_digit() && ch != '.')
        .find_map(|segment| {
            let mut parts = segment.split('.');
            let major = parts.next()?.parse().ok()?;
            let minor = parts.next()?.parse().ok()?;
            let patch = parts.next()?.parse().ok()?;
            Some(CliVersion {
                major,
                minor,
                patch,
            })
        })
}

pub(crate) fn select_acp_adapter_profile(
    adapter: AcpAdapterKind,
    detected: CliVersion,
) -> Result<AcpAdapterProfile, String> {
    let (baseline, dialect) = match adapter {
        AcpAdapterKind::CodexAcp => (CODEX_ACP_BASELINE_VERSION, AcpStreamDialect::CodexAcp1_1_7),
        AcpAdapterKind::OpenCode => (OPENCODE_BASELINE_VERSION, AcpStreamDialect::OpenCode1_18_11),
    };
    if detected < baseline {
        return Err(format!(
            "{} requires version >= {baseline}; found {detected}",
            adapter.as_str()
        ));
    }
    if detected.major != baseline.major {
        return Err(format!(
            "{} version {detected} has unsupported major {}; latest validated dialect is {baseline}",
            adapter.as_str(),
            detected.major
        ));
    }
    Ok(AcpAdapterProfile {
        adapter,
        detected_version: detected.to_string(),
        dialect,
        compatibility: if detected == baseline {
            AcpCompatibilityStatus::Validated
        } else {
            AcpCompatibilityStatus::CompatibleNewer
        },
    })
}

pub(crate) fn select_acp_adapter_profile_from_initialize(
    adapter: AcpAdapterKind,
    initialize_result: &Value,
) -> Result<(CliVersion, AcpAdapterProfile), String> {
    let adapter_name = initialize_result
        .get("agentInfo")
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    let expected_agent_name = match adapter {
        AcpAdapterKind::CodexAcp => CODEX_ACP_INITIALIZE_AGENT_NAME,
        AcpAdapterKind::OpenCode => OPENCODE_INITIALIZE_AGENT_NAME,
    };
    if adapter_name != expected_agent_name {
        return Err(format!(
            "{} initialize returned a missing or unexpected agentInfo.name",
            adapter.as_str()
        ));
    }
    let version_text = initialize_result
        .get("agentInfo")
        .and_then(|value| value.get("version"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let version = parse_cli_version(version_text).ok_or_else(|| {
        format!(
            "{} initialize returned a missing or unparseable agentInfo.version",
            adapter.as_str()
        )
    })?;
    let profile = select_acp_adapter_profile(adapter, version)?;
    Ok((version, profile))
}

pub(crate) async fn persist_acp_runtime_profile(
    runtime_dir: &str,
    runner: &str,
    profile: &AcpAdapterProfile,
    companion_versions: BTreeMap<String, String>,
) -> std::io::Result<PathBuf> {
    let profile_dir = Path::new(runtime_dir).join("acp-profiles");
    tokio::fs::create_dir_all(&profile_dir).await?;
    let target = profile_dir.join(format!("{}.json", profile.adapter.as_str()));
    let temporary = profile_dir.join(format!(
        ".{}.{}.{}.tmp",
        profile.adapter.as_str(),
        std::process::id(),
        Uuid::new_v4()
    ));
    let record = AcpRuntimeProfileRecord {
        runner: runner.to_string(),
        adapter: profile.adapter,
        detected_version: profile.detected_version.clone(),
        baseline_version: profile.baseline_version().to_string(),
        dialect: profile.dialect,
        compatibility: profile.compatibility,
        companion_versions,
        detected_at: Utc::now().to_rfc3339(),
        build_git_sha: hone_core::current_build_info().git_sha,
    };
    let payload = serde_json::to_vec_pretty(&record).map_err(std::io::Error::other)?;
    tokio::fs::write(&temporary, payload).await?;
    if let Err(error) = tokio::fs::rename(&temporary, &target).await {
        if target.exists() {
            tokio::fs::remove_file(&target).await?;
            tokio::fs::rename(&temporary, &target).await?;
        } else {
            return Err(error);
        }
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_exact_and_newer_adapter_profiles_without_claiming_same_validation() {
        let exact =
            select_acp_adapter_profile(AcpAdapterKind::CodexAcp, CODEX_ACP_BASELINE_VERSION)
                .expect("exact profile");
        assert_eq!(exact.dialect, AcpStreamDialect::CodexAcp1_1_7);
        assert_eq!(exact.compatibility, AcpCompatibilityStatus::Validated);

        let newer = select_acp_adapter_profile(
            AcpAdapterKind::CodexAcp,
            CliVersion {
                major: 1,
                minor: 2,
                patch: 0,
            },
        )
        .expect("newer profile");
        assert_eq!(newer.dialect, AcpStreamDialect::CodexAcp1_1_7);
        assert_eq!(newer.compatibility, AcpCompatibilityStatus::CompatibleNewer);
        assert_eq!(newer.detected_version, "1.2.0");
        assert_eq!(newer.baseline_version(), "1.1.7");
    }

    #[test]
    fn rejects_adapter_versions_below_their_own_baseline() {
        let codex = select_acp_adapter_profile(
            AcpAdapterKind::CodexAcp,
            CliVersion {
                major: 1,
                minor: 1,
                patch: 6,
            },
        );
        assert!(codex.expect_err("old codex adapter").contains("1.1.7"));

        let opencode = select_acp_adapter_profile(
            AcpAdapterKind::OpenCode,
            CliVersion {
                major: 1,
                minor: 18,
                patch: 10,
            },
        );
        assert!(opencode.expect_err("old opencode").contains("1.18.11"));

        let unknown_major = select_acp_adapter_profile(
            AcpAdapterKind::OpenCode,
            CliVersion {
                major: 2,
                minor: 0,
                patch: 0,
            },
        );
        assert!(
            unknown_major
                .expect_err("unknown major must fail closed")
                .contains("unsupported major")
        );
    }

    #[test]
    fn initialize_result_is_the_authoritative_version_boundary() {
        let exact = serde_json::json!({
            "protocolVersion": 1,
            "agentInfo": {
                "name": "@agentclientprotocol/codex-acp",
                "version": "1.1.7"
            }
        });
        let (version, profile) =
            select_acp_adapter_profile_from_initialize(AcpAdapterKind::CodexAcp, &exact)
                .expect("versioned initialize profile");
        assert_eq!(version, CODEX_ACP_BASELINE_VERSION);
        assert_eq!(profile.compatibility, AcpCompatibilityStatus::Validated);

        let missing = serde_json::json!({
            "protocolVersion": 1,
            "agentInfo": {"name": "opencode"}
        });
        let error = select_acp_adapter_profile_from_initialize(AcpAdapterKind::OpenCode, &missing)
            .expect_err("missing version must fail closed");
        assert!(error.contains("opencode"));
        assert!(error.contains("agentInfo.version"));

        let wrong_identity = serde_json::json!({
            "protocolVersion": 1,
            "agentInfo": {"name": "opencode", "version": "1.1.7"}
        });
        let error =
            select_acp_adapter_profile_from_initialize(AcpAdapterKind::CodexAcp, &wrong_identity)
                .expect_err("a different adapter identity must fail closed");
        assert!(error.contains("codex-acp"));
        assert!(error.contains("agentInfo.name"));

        let unobserved_alias = serde_json::json!({
            "protocolVersion": 1,
            "agentInfo": {"name": "codex-acp", "version": "1.1.7"}
        });
        let error =
            select_acp_adapter_profile_from_initialize(AcpAdapterKind::CodexAcp, &unobserved_alias)
                .expect_err("an unobserved shorthand must not widen the identity contract");
        assert!(error.contains("agentInfo.name"));

        let oversized_version = serde_json::json!({
            "protocolVersion": 1,
            "agentInfo": {
                "name": "@agentclientprotocol/codex-acp",
                "version": "not-a-version-with-private-or-unbounded-diagnostic-content"
            }
        });
        let error = select_acp_adapter_profile_from_initialize(
            AcpAdapterKind::CodexAcp,
            &oversized_version,
        )
        .expect_err("invalid version must fail without echoing external content");
        assert!(!error.contains("private-or-unbounded"));
    }

    #[tokio::test]
    async fn persisted_profile_contains_only_bounded_runtime_provenance() {
        let temporary = tempfile::tempdir().expect("temp runtime");
        let profile =
            select_acp_adapter_profile(AcpAdapterKind::OpenCode, OPENCODE_BASELINE_VERSION)
                .expect("OpenCode profile");
        let path = persist_acp_runtime_profile(
            &temporary.path().display().to_string(),
            "opencode_acp",
            &profile,
            BTreeMap::new(),
        )
        .await
        .expect("persist profile");
        let payload = std::fs::read_to_string(path).expect("read profile");
        assert!(payload.contains("\"detected_version\": \"1.18.11\""));
        assert!(payload.contains("\"compatibility\": \"validated\""));
        assert!(!payload.contains(temporary.path().to_string_lossy().as_ref()));
        assert!(!payload.contains("prompt"));
        assert!(!payload.contains("api_key"));
    }
}
