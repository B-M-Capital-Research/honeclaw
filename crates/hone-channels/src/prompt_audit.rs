use std::fs;

use hone_core::{ActorIdentity, HoneConfig, SessionIdentity, runtime_heartbeat_dir};

#[derive(Debug, Clone)]
pub struct PromptAuditMetadata {
    pub session_identity: SessionIdentity,
    pub message_id: Option<String>,
}

pub fn write_prompt_audit(
    config: &HoneConfig,
    actor: &ActorIdentity,
    session_id: &str,
    metadata: &PromptAuditMetadata,
    system_prompt: &str,
    runtime_input: &str,
) -> Result<(), String> {
    let runtime_dir = runtime_heartbeat_dir(config);
    let audit_dir = runtime_dir
        .join("prompt-audit")
        .join(sanitize_prompt_audit_path(&actor.channel));
    fs::create_dir_all(&audit_dir)
        .map_err(|err| format!("create audit dir {} failed: {err}", audit_dir.display()))?;

    let timestamp = hone_core::beijing_now().format("%Y%m%d-%H%M%S").to_string();
    let session_slug = sanitize_prompt_audit_path(session_id);
    let prompt_path = audit_dir.join(format!("{timestamp}-{session_slug}.json"));
    let latest_path = audit_dir.join(format!("latest-{session_slug}.json"));
    let payload = serde_json::json!({
        "created_at": hone_core::beijing_now_rfc3339(),
        "channel": actor.channel,
        "actor_user_id": actor.user_id,
        "session_id": session_id,
        "session_identity": metadata.session_identity,
        "message_id": metadata.message_id,
        "system_prompt": system_prompt,
        "runtime_input": runtime_input,
    });

    let content = serde_json::to_vec_pretty(&payload)
        .map_err(|err| format!("encode prompt audit failed: {err}"))?;

    for path in [prompt_path, latest_path] {
        fs::write(&path, &content)
            .map_err(|err| format!("write prompt audit {} failed: {err}", path.display()))?;
    }

    Ok(())
}

fn sanitize_prompt_audit_path(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "session".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_prompt_audit_path;

    #[test]
    fn sanitize_prompt_audit_path_keeps_safe_characters() {
        assert_eq!(
            sanitize_prompt_audit_path("discord.group-1"),
            "discord.group-1"
        );
    }

    #[test]
    fn sanitize_prompt_audit_path_replaces_unsafe_characters() {
        assert_eq!(
            sanitize_prompt_audit_path("feishu/open id"),
            "feishu_open_id"
        );
        assert_eq!(sanitize_prompt_audit_path("   "), "session");
    }
}
