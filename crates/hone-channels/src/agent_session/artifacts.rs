use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use hone_core::agent::AgentResponse;

const MAX_GENERATED_ATTACHMENTS: usize = 6;
const MAX_SCAN_DEPTH: usize = 4;
const RECENT_FILE_GRACE: Duration = Duration::from_secs(2);

pub(super) fn attach_web_generated_files(
    response: &mut AgentResponse,
    working_directory: &str,
    run_started_at: SystemTime,
    oss: Option<&OssPromotion<'_>>,
) -> usize {
    if !response.success || response.content.contains("[附件: ") {
        return 0;
    }

    let files = collect_recent_mentioned_files(
        &response.content,
        Path::new(working_directory),
        run_started_at,
    );
    if files.is_empty() {
        return 0;
    }

    for path in &files {
        // 云端模式下沙箱盘是临时的：把生成物先传到对象存储，附件里记 oss:// 引用，
        // 否则容器重启或多实例部署后下载链接就失效了。
        let reference = oss
            .and_then(|promotion| promotion.promote(path))
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        response
            .content
            .push_str(&format!("\n[附件: {reference}]"));
    }
    files.len()
}

/// 把沙箱里的生成物上传到对象存储，返回 `oss://` 引用。上传失败时返回 `None`，
/// 调用方回退到本地路径 —— 宁可给一个当前实例能打开的链接，也不要丢附件。
pub(super) struct OssPromotion<'a> {
    pub store: &'a hone_core::cloud_runtime::OssObjectStore,
    pub actor: &'a hone_core::ActorIdentity,
    pub session_id: &'a str,
}

impl OssPromotion<'_> {
    fn promote(&self, path: &Path) -> Option<String> {
        let bytes = fs::read(path).ok()?;
        let name = path.file_name()?.to_string_lossy().to_string();
        let key = self.store.actor_upload_key(self.actor, self.session_id, &name);
        let content_type = content_type_for(path);
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(self.store.put_object(&key, bytes, content_type))
            })
        } else {
            tokio::runtime::Runtime::new()
                .ok()?
                .block_on(self.store.put_object(&key, bytes, content_type))
        };
        match result {
            Ok(()) => Some(self.store.object_uri(&key)),
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    "failed to upload generated attachment to OSS: {error}"
                );
                None
            }
        }
    }
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => "text/csv",
        "json" => "application/json",
        "md" | "txt" => "text/plain; charset=utf-8",
        "pdf" => "application/pdf",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

fn collect_recent_mentioned_files(
    content: &str,
    working_directory: &Path,
    run_started_at: SystemTime,
) -> Vec<PathBuf> {
    if content.trim().is_empty() || !working_directory.is_dir() {
        return Vec::new();
    }

    let cutoff = run_started_at
        .checked_sub(RECENT_FILE_GRACE)
        .unwrap_or(run_started_at);
    let mut found = Vec::new();
    collect_recent_mentioned_files_inner(content, working_directory, cutoff, 0, &mut found);
    found
}

fn collect_recent_mentioned_files_inner(
    content: &str,
    dir: &Path,
    cutoff: SystemTime,
    depth: usize,
    found: &mut Vec<PathBuf>,
) {
    if depth > MAX_SCAN_DEPTH || found.len() >= MAX_GENERATED_ATTACHMENTS {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        if found.len() >= MAX_GENERATED_ATTACHMENTS {
            return;
        }
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            collect_recent_mentioned_files_inner(content, &path, cutoff, depth + 1, found);
            continue;
        }
        if !metadata.is_file() || !is_downloadable_artifact_path(&path) {
            continue;
        }
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if modified < cutoff {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if content.contains(name) {
            found.push(path);
        }
    }
}

fn is_downloadable_artifact_path(path: &Path) -> bool {
    let Some(ext) = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
    else {
        return false;
    };
    matches!(
        ext.as_str(),
        "csv"
            | "doc"
            | "docx"
            | "gif"
            | "jpeg"
            | "jpg"
            | "json"
            | "md"
            | "pdf"
            | "png"
            | "ppt"
            | "pptx"
            | "txt"
            | "webp"
            | "xls"
            | "xlsx"
            | "zip"
    )
}

#[cfg(test)]
mod tests {
    use super::attach_web_generated_files;
    use hone_core::agent::AgentResponse;
    use std::time::{Duration, SystemTime};

    fn successful_response(content: &str) -> AgentResponse {
        AgentResponse {
            content: content.to_string(),
            tool_calls_made: Vec::new(),
            iterations: 1,
            success: true,
            error: None,
        }
    }

    #[test]
    fn web_generated_file_is_attached_when_final_only_mentions_filename() {
        let root = tempfile::tempdir().expect("temp dir");
        let run_started_at = SystemTime::now();
        let file_path = root.path().join("A股三年投资策略表.xlsx");
        std::fs::write(&file_path, b"xlsx").expect("write generated file");
        let mut response = successful_response("已整理成 Excel：A股三年投资策略表.xlsx");

        let attached = attach_web_generated_files(
            &mut response,
            root.path().to_string_lossy().as_ref(),
            run_started_at,
            None,
        );

        assert_eq!(attached, 1);
        assert!(response.content.contains("[附件: "));
        assert!(response.content.contains("A股三年投资策略表.xlsx]"));
    }

    #[test]
    fn stale_sandbox_files_are_not_attached_to_new_turns() {
        let root = tempfile::tempdir().expect("temp dir");
        let file_path = root.path().join("old.csv");
        std::fs::write(&file_path, b"old").expect("write stale file");
        let run_started_at = SystemTime::now() + Duration::from_secs(10);
        let mut response = successful_response("文件 old.csv 已生成");

        let attached = attach_web_generated_files(
            &mut response,
            root.path().to_string_lossy().as_ref(),
            run_started_at,
            None,
        );

        assert_eq!(attached, 0);
        assert!(!response.content.contains("[附件: "));
    }
}
