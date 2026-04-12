//! Attachment PDF helpers.

use std::path::PathBuf;

use pdf_extract::extract_text;
use tokio::task;

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

/// 全量提取 PDF 文本（无字符数上限），供长文档后续处理复用
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
