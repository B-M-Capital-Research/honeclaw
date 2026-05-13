use hone_core::config::GeminiAcpConfig;

use super::acp_common::CliVersion;

const MIN_GEMINI_ACP_VERSION: CliVersion = CliVersion {
    major: 0,
    minor: 30,
    patch: 0,
};

pub(crate) fn validate_gemini_version(version: CliVersion) -> Result<(), String> {
    if version < MIN_GEMINI_ACP_VERSION {
        return Err(format!(
            "gemini_acp requires gemini >= {MIN_GEMINI_ACP_VERSION}; found {version}. Update with `npm install -g @google/gemini-cli@latest`."
        ));
    }
    Ok(())
}

pub(crate) fn gemini_acp_effective_args(config: &GeminiAcpConfig) -> Vec<String> {
    let mut args = Vec::new();
    let mut iter = config.args.iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--yolo" | "-y" | "--sandbox" | "-s" => continue,
            "--approval-mode" | "--policy" | "--include-directories" => {
                let _ = iter.next();
                continue;
            }
            _ => args.push(arg.clone()),
        }
    }

    if !args
        .iter()
        .any(|arg| arg == "--acp" || arg == "--experimental-acp")
    {
        args.push("--acp".to_string());
    }
    args.push("--approval-mode".to_string());
    args.push("plan".to_string());
    args
}
