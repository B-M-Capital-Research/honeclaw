use std::path::{Path, PathBuf};

use hone_core::agent::AgentResponse;

use crate::HoneBotCore;
use crate::outbound::{ResponseContentSegment, split_response_content_segments};
use crate::runtime::{is_transitional_planning_sentence, sanitize_user_visible_output};
use crate::sandbox::sandbox_base_dir;

pub(crate) const EMPTY_SUCCESS_FALLBACK_MESSAGE: &str =
    "这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。";
const MISSING_LOCAL_IMAGE_FALLBACK_MESSAGE: &str = "（图表文件不可用，请重新生成）";

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FinalizeResponseOutcome {
    pub(crate) fallback_reason: Option<&'static str>,
}

pub(crate) fn finalize_agent_response(
    core: &HoneBotCore,
    session_id: &str,
    runner_name: &str,
    response: &mut AgentResponse,
) -> FinalizeResponseOutcome {
    let mut outcome = FinalizeResponseOutcome::default();
    if !response.success {
        return outcome;
    }

    if response_leaks_system_prompt(&response.content) {
        tracing::error!(
            "[AgentSession] blocked echoed system prompt runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned leaked system instructions".to_string());
        response.content.clear();
        return outcome;
    }

    let sanitized = sanitize_user_visible_output(&response.content);
    if sanitized.only_internal {
        tracing::error!(
            "[AgentSession] blocked internal-only assistant output runner={} session_id={}",
            runner_name,
            session_id
        );
        response.success = false;
        response.error = Some("agent returned internal-only output".to_string());
        response.content.clear();
        return outcome;
    }

    if sanitized.content.trim().is_empty() {
        tracing::warn!(
            "[AgentSession] empty visible output after sanitization runner={} session_id={} removed_internal={}",
            runner_name,
            session_id,
            sanitized.removed_internal
        );
        response.success = false;
        response.content = EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string();
        response.error = Some(EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string());
        outcome.fallback_reason = Some("sanitized_empty_success");
    } else if is_transitional_planning_sentence(sanitized.content.trim()) {
        tracing::warn!(
            "[AgentSession] transitional planning sentence detected, treating as empty runner={} session_id={} chars={}",
            runner_name,
            session_id,
            sanitized.content.trim().chars().count()
        );
        response.success = false;
        response.content = EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string();
        response.error = Some(EMPTY_SUCCESS_FALLBACK_MESSAGE.to_string());
        outcome.fallback_reason = Some("planning_sentence_suppressed");
    } else {
        response.content = sanitized.content;
    }

    response.content = normalize_local_image_references(core, session_id, &response.content);
    outcome
}

pub(crate) fn response_leaks_system_prompt(content: &str) -> bool {
    let trimmed = content.trim_start_matches(char::is_whitespace);
    trimmed.starts_with("### System Instructions ###")
}

pub(crate) fn normalize_local_image_references(
    core: &HoneBotCore,
    session_id: &str,
    content: &str,
) -> String {
    let segments = split_response_content_segments(content);
    if !segments
        .iter()
        .any(|segment| matches!(segment, ResponseContentSegment::LocalImage(_)))
    {
        return content.to_string();
    }

    let mut normalized = String::new();
    for segment in segments {
        match segment {
            ResponseContentSegment::Text(text) => normalized.push_str(&text),
            ResponseContentSegment::LocalImage(marker) => {
                if let Some(stable_path) =
                    stabilize_local_image_path(core, session_id, &marker.path)
                {
                    normalized.push_str("file://");
                    normalized.push_str(&stable_path);
                } else {
                    normalized.push_str(MISSING_LOCAL_IMAGE_FALLBACK_MESSAGE);
                }
            }
        }
    }
    normalized
}

fn stabilize_local_image_path(core: &HoneBotCore, session_id: &str, path: &str) -> Option<String> {
    let source = Path::new(path);
    if !source.is_absolute() || !source.exists() {
        return None;
    }

    let gen_images_root = PathBuf::from(&core.config.storage.gen_images_dir);
    if source.starts_with(&gen_images_root) {
        return Some(source.to_string_lossy().to_string());
    }

    let sandbox_root = sandbox_base_dir();
    if !source.starts_with(&sandbox_root) {
        return Some(source.to_string_lossy().to_string());
    }

    let target_dir = gen_images_root.join(session_id);
    if let Err(err) = std::fs::create_dir_all(&target_dir) {
        tracing::warn!(
            "[AgentSession] failed to create stable image dir session_id={} dir={} err={}",
            session_id,
            target_dir.display(),
            err
        );
        return Some(source.to_string_lossy().to_string());
    }

    let target_name = unique_stable_image_name(source);
    let target = target_dir.join(target_name);
    match std::fs::copy(source, &target) {
        Ok(_) => Some(target.to_string_lossy().to_string()),
        Err(err) => {
            tracing::warn!(
                "[AgentSession] failed to stabilize local image session_id={} source={} target={} err={}",
                session_id,
                source.display(),
                target.display(),
                err
            );
            Some(source.to_string_lossy().to_string())
        }
    }
}

fn unique_stable_image_name(source: &Path) -> String {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_filename_component)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "image".to_string());
    let ext = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "png".to_string());
    format!("{stem}-{}.{}", uuid::Uuid::new_v4().simple(), ext)
}

fn sanitize_filename_component(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
