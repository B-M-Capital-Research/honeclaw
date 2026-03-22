//! Attachment KB / PDF helpers.

use std::path::PathBuf;
use std::sync::Arc;

use hone_memory::KbSaveRequest;
use pdf_extract::extract_text;
use tokio::task;
use tracing::warn;

use super::ingest::{AttachmentKind, AttachmentPersistRequest};
use crate::run_kb_analysis;

pub(crate) async fn persist_attachments_to_kb(
    core: Arc<crate::HoneBotCore>,
    request: AttachmentPersistRequest,
) {
    let AttachmentPersistRequest {
        channel,
        user_id,
        session_id,
        attachments,
    } = request;

    for attachment in attachments {
        if attachment.local_path.is_none() || attachment.error.is_some() {
            continue;
        }

        let source_path = PathBuf::from(attachment.local_path.as_deref().unwrap_or(""));
        let (parsed_text, parse_error) = if attachment.kind == AttachmentKind::Pdf {
            match extract_full_pdf_text(&source_path).await {
                Ok(text) => (Some(text), None),
                Err(err) => (None, Some(err)),
            }
        } else {
            (None, None)
        };

        let save_request = KbSaveRequest {
            filename: attachment.filename.clone(),
            kind: format!("{:?}", attachment.kind),
            size: attachment.size,
            content_type: attachment.content_type.clone(),
            channel: channel.clone(),
            user_id: user_id.clone(),
            session_id: session_id.clone(),
            source_path,
            parsed_text: parsed_text.clone(),
            parse_error,
        };

        match core.kb_storage.save_attachment(save_request).await {
            Ok(entry) => {
                if let Some(text) = parsed_text {
                    let stock_table = core.stock_table.clone();
                    let core_for_analysis = core.clone();
                    tokio::spawn(async move {
                        let analyzed =
                            run_kb_analysis(&core_for_analysis, &entry, &text, &stock_table).await;
                        if analyzed
                            && let Err(err) =
                                core_for_analysis.kb_storage.mark_analyzed(&entry.id).await
                        {
                            warn!("[Attachments/KB] mark_analyzed 失败: {err}");
                        }
                    });
                }
            }
            Err(err) => {
                warn!(
                    "[Attachments/KB] 保存附件失败: channel={} user={} session={} file={} err={}",
                    channel, user_id, session_id, attachment.filename, err
                );
            }
        }
    }
}

pub(crate) async fn extract_pdf_preview(pdf_path: PathBuf) -> Result<String, String> {
    task::spawn_blocking(move || {
        let raw = extract_text(&pdf_path).map_err(|e| format!("pdf_extract 解析失败: {}", e))?;
        let normalized = raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if normalized.trim().is_empty() {
            return Err("未提取到可读文本（可能是扫描件图片 PDF）".to_string());
        }
        Ok(truncate_chars(&normalized, MAX_PDF_PREVIEW_CHARS))
    })
    .await
    .map_err(|e| format!("PDF 提取任务失败: {e}"))?
}

/// 全量提取 PDF 文本（无字符数上限），专用于知识库存储
pub async fn extract_full_pdf_text(pdf_path: &std::path::Path) -> Result<String, String> {
    let path = pdf_path.to_path_buf();
    task::spawn_blocking(move || {
        let raw = extract_text(&path).map_err(|e| format!("pdf_extract 解析失败: {}", e))?;
        let normalized = raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if normalized.trim().is_empty() {
            return Err("未提取到可读文本（可能是扫描件图片 PDF）".to_string());
        }
        Ok(normalized)
    })
    .await
    .map_err(|e| format!("PDF 全量提取任务失败: {e}"))?
}

const MAX_PDF_PREVIEW_CHARS: usize = 4000;

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}
