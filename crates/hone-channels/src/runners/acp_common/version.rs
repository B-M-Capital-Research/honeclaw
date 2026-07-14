//! 从 `<cli> --version` 的文本里解析出三段语义化版本号。
//!
//! 启用中的 ACP runner 各有 CLI 版本下限(例:codex-acp 要 ≥1.1.2)。
//! 版本解析失败时提前给人类可读的错误,而不是让 JSON-RPC 再挂。

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CliVersion {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
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
