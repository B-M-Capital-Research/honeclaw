//! Attachment PDF helpers.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use hone_core::truncate_chars_append;
use pdf_extract::extract_text;
use tokio::task;

pub(crate) async fn extract_pdf_preview(pdf_path: PathBuf) -> Result<String, String> {
    task::spawn_blocking(move || {
        let raw = extract_pdf_text_safely(|| {
            extract_text(&pdf_path).map_err(|e| format!("pdf_extract 解析失败: {}", e))
        })?;
        let normalized = normalize_pdf_text(&raw)?;
        Ok(truncate_chars_append(
            &normalized,
            MAX_PDF_PREVIEW_CHARS,
            "...",
        ))
    })
    .await
    .map_err(|e| format!("PDF 提取任务失败: {e}"))?
}

/// 全量提取 PDF 文本（无字符数上限），供长文档后续处理复用
pub async fn extract_full_pdf_text(pdf_path: &std::path::Path) -> Result<String, String> {
    let path = pdf_path.to_path_buf();
    task::spawn_blocking(move || {
        let raw = extract_pdf_text_safely(|| {
            extract_text(&path).map_err(|e| format!("pdf_extract 解析失败: {}", e))
        })?;
        normalize_pdf_text(&raw)
    })
    .await
    .map_err(|e| format!("PDF 全量提取任务失败: {e}"))?
}

fn normalize_pdf_text(raw: &str) -> Result<String, String> {
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
}

fn extract_pdf_text_safely(
    extract: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    match catch_unwind(AssertUnwindSafe(extract)) {
        Ok(result) => result.map_err(|err| sanitize_pdf_extract_error(&err)),
        Err(_) => Err(PDF_TEXT_EXTRACT_PANIC_ERROR.to_string()),
    }
}

pub(crate) fn sanitize_pdf_extract_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("panicked")
        || lower.contains("index out of bounds")
        || lower.contains("adobe-cmap-parser")
        || lower.contains("/users/")
        || lower.contains("/src/")
    {
        return PDF_TEXT_EXTRACT_PANIC_ERROR.to_string();
    }

    truncate_chars_append(error.trim(), MAX_PDF_ERROR_CHARS, "...")
}

const PDF_TEXT_EXTRACT_PANIC_ERROR: &str =
    "pdf_text_extract_failed: PDF 文本解析器内部错误，已跳过文本预览";
const MAX_PDF_ERROR_CHARS: usize = 160;
const MAX_PDF_PREVIEW_CHARS: usize = 4000;

#[cfg(test)]
mod tests {
    use super::{
        PDF_TEXT_EXTRACT_PANIC_ERROR, extract_pdf_text_safely, sanitize_pdf_extract_error,
    };
    use std::panic;

    #[test]
    fn pdf_extract_panic_is_caught_and_sanitized() {
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = extract_pdf_text_safely(|| {
            panic!(
                "index out of bounds: the len is 11414 but the index is 11414 at /Users/example/.cargo/adobe-cmap-parser/src/lib.rs"
            )
        });
        panic::set_hook(previous_hook);

        assert_eq!(
            result.unwrap_err(),
            "pdf_text_extract_failed: PDF 文本解析器内部错误，已跳过文本预览"
        );
    }

    #[test]
    fn pdf_extract_error_sanitizer_strips_internal_panic_details() {
        let sanitized = sanitize_pdf_extract_error(
            "PDF 提取任务失败: task 42 panicked with message \"index out of bounds\" at /Users/example/.cargo/adobe-cmap-parser-0.4.1/src/lib.rs:195:41",
        );

        assert_eq!(sanitized, PDF_TEXT_EXTRACT_PANIC_ERROR);
        assert!(!sanitized.contains("/Users/"));
        assert!(!sanitized.contains("adobe-cmap-parser"));
        assert!(!sanitized.contains("index out of bounds"));
    }
}
