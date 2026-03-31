use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use flate2::read::GzDecoder;
use hone_core::ActorIdentity;
use tar::Archive;
use tokio::task;
use tracing::warn;
use zip::read::ZipArchive;

use crate::HoneBotCore;
use crate::sandbox::actor_upload_dir;

#[cfg(test)]
pub(crate) use super::vision::validate_image_shape_blocking;

const MAX_ARCHIVE_EXTRACTED_FILES: usize = 80;
const MAX_ARCHIVE_TOTAL_BYTES: u64 = 120 * 1024 * 1024;
const MAX_ARCHIVE_SINGLE_FILE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_ARCHIVE_PROMPT_FILES: usize = 25;
const MAX_ARCHIVE_PREVIEW_BYTES: u64 = 128 * 1024;
const MAX_ARCHIVE_PREVIEW_CHARS: usize = 500;
const MAX_ATTACHMENT_BYTES: u64 = 5 * 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 3 * 1024 * 1024;
const ATTACHMENT_POLICY_REJECT_PREFIX: &str = "附件未通过准入限制";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentKind {
    Image,
    Pdf,
    Spreadsheet,
    Text,
    Audio,
    Video,
    Archive,
    Other,
}

impl AttachmentKind {
    pub fn label(&self) -> &'static str {
        match self {
            AttachmentKind::Image => "图片",
            AttachmentKind::Pdf => "PDF",
            AttachmentKind::Spreadsheet => "表格",
            AttachmentKind::Text => "文本",
            AttachmentKind::Audio => "音频",
            AttachmentKind::Video => "视频",
            AttachmentKind::Archive => "压缩包",
            AttachmentKind::Other => "其他",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractedFileInfo {
    pub path: String,
    pub size: u64,
    pub kind: AttachmentKind,
    pub preview: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReceivedAttachment {
    pub filename: String,
    pub content_type: Option<String>,
    pub size: u32,
    pub url: String,
    pub kind: AttachmentKind,
    pub local_path: Option<String>,
    pub error: Option<String>,
    pub extracted_files: Vec<ExtractedFileInfo>,
    pub extraction_error: Option<String>,
    pub pdf_text_preview: Option<String>,
    pub pdf_extract_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachmentDescriptor {
    pub filename: String,
    pub content_type: Option<String>,
    pub size: u32,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct RawAttachment {
    pub filename: String,
    pub content_type: Option<String>,
    pub size: Option<u32>,
    pub url: String,
    pub local_path: Option<PathBuf>,
    pub data: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachmentIngestRequest {
    pub channel: String,
    pub actor: ActorIdentity,
    pub session_id: String,
    pub attachments: Vec<RawAttachment>,
}

#[derive(Debug, Clone)]
pub struct AttachmentPersistRequest {
    pub channel: String,
    pub user_id: String,
    pub session_id: String,
    pub attachments: Vec<ReceivedAttachment>,
}

#[async_trait]
pub trait AttachmentFetcher {
    async fn fetch(&self, descriptor: AttachmentDescriptor) -> ReceivedAttachment;
}

pub async fn fetch_and_enrich<F: AttachmentFetcher + ?Sized>(
    fetcher: &F,
    descriptor: AttachmentDescriptor,
) -> ReceivedAttachment {
    let attachment = fetcher.fetch(descriptor).await;
    if attachment.local_path.is_some() && attachment.error.is_none() {
        return enrich_attachment(attachment).await;
    }
    attachment
}

pub async fn ingest_raw_attachments(
    _core: &HoneBotCore,
    request: AttachmentIngestRequest,
) -> Vec<ReceivedAttachment> {
    let upload_dir = attachment_upload_dir(&request.actor, &request.session_id);
    if let Err(err) = tokio::fs::create_dir_all(&upload_dir).await {
        warn!(
            "[Attachments] 创建上传目录失败 {}: {}",
            upload_dir.display(),
            err
        );
    }

    let mut out = Vec::with_capacity(request.attachments.len());
    for (index, attachment) in request.attachments.into_iter().enumerate() {
        out.push(ingest_one_raw_attachment(&upload_dir, index, attachment).await);
    }
    out
}

pub fn spawn_attachment_persist_pipeline(
    core: Arc<HoneBotCore>,
    request: AttachmentPersistRequest,
) {
    tokio::spawn(async move {
        super::vector_store::persist_attachments_to_kb(core, request).await;
    });
}

async fn ingest_one_raw_attachment(
    upload_dir: &Path,
    index: usize,
    attachment: RawAttachment,
) -> ReceivedAttachment {
    let filename = if attachment.filename.trim().is_empty() {
        "attachment.bin".to_string()
    } else {
        attachment.filename.trim().to_string()
    };
    let kind = infer_attachment_kind(attachment.content_type.as_deref(), &filename);
    let mut received = ReceivedAttachment {
        filename: filename.clone(),
        content_type: attachment.content_type.clone(),
        size: attachment.size.unwrap_or(0),
        url: attachment.url.clone(),
        kind,
        local_path: None,
        error: attachment.error.clone(),
        extracted_files: vec![],
        extraction_error: None,
        pdf_text_preview: None,
        pdf_extract_error: None,
    };

    if let Some(reason) = validate_attachment_policy(kind, attachment.size) {
        received.error = Some(reason);
        return received;
    }

    if received.error.is_none() {
        match materialize_raw_attachment(upload_dir, index, &filename, attachment).await {
            Ok(path) => {
                received.local_path = Some(path.display().to_string());
            }
            Err(err) => {
                received.error = Some(err);
            }
        }
    }

    if received.local_path.is_some()
        && received.error.is_none()
        && let Some(reason) = super::vision::validate_attachment_image_shape(&received).await
    {
        if let Some(local_path) = &received.local_path {
            let _ = tokio::fs::remove_file(local_path).await;
        }
        received.local_path = None;
        received.error = Some(reason);
        return received;
    }

    if received.local_path.is_some() && received.error.is_none() {
        let extract_dir = if received.kind == AttachmentKind::Archive {
            Some(upload_dir.join(format!(
                "{}_{}_extracted",
                unique_attachment_prefix(index),
                sanitize_filename(&received.filename)
            )))
        } else {
            None
        };
        received = enrich_attachment_with_extract_dir(received, extract_dir).await;
    }

    received
}

async fn materialize_raw_attachment(
    upload_dir: &Path,
    index: usize,
    filename: &str,
    attachment: RawAttachment,
) -> Result<PathBuf, String> {
    let safe_filename = sanitize_filename(filename);
    let target = upload_dir.join(format!(
        "{}_{}_{}",
        unique_attachment_prefix(index),
        index,
        safe_filename
    ));

    if let Some(path) = attachment.local_path {
        tokio::fs::copy(&path, &target)
            .await
            .map_err(|err| format!("复制附件失败: {err}"))?;
        return Ok(target);
    }

    if let Some(data) = attachment.data {
        tokio::fs::write(&target, data)
            .await
            .map_err(|err| format!("写入附件失败: {err}"))?;
        return Ok(target);
    }

    Err("附件既无本地路径也无原始数据".to_string())
}

fn attachment_upload_dir(actor: &ActorIdentity, session_id: &str) -> PathBuf {
    actor_upload_dir(actor, session_id)
}

fn unique_attachment_prefix(index: usize) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(index as u128);
    millis.to_string()
}

impl ReceivedAttachment {
    pub fn as_prompt_line(&self, index: usize) -> String {
        let mut line = format!(
            "{index}. 文件名={} 分类={} 大小={}B 类型={} URL={}",
            self.filename,
            self.kind.label(),
            self.size,
            self.content_type.as_deref().unwrap_or("unknown"),
            self.url
        );

        if let Some(path) = &self.local_path {
            line.push_str(&format!(" 本地路径={path}"));
        }
        if let Some(err) = &self.error {
            line.push_str(&format!(" 下载状态=失败({err})"));
        } else {
            line.push_str(" 下载状态=成功");
        }
        if self.kind == AttachmentKind::Archive {
            if let Some(err) = &self.extraction_error {
                line.push_str(&format!(" 解压状态=失败({err})"));
            } else {
                line.push_str(&format!(" 解压文件数={}", self.extracted_files.len()));
            }
        } else if self.kind == AttachmentKind::Pdf {
            if let Some(err) = &self.pdf_extract_error {
                line.push_str(&format!(" PDF解析状态=失败({err})"));
            } else if self.pdf_text_preview.is_some() {
                line.push_str(" PDF解析状态=已提取文本");
            } else {
                line.push_str(" PDF解析状态=无文本");
            }
        }
        line
    }
}

fn validate_attachment_policy(kind: AttachmentKind, size: Option<u32>) -> Option<String> {
    let size = u64::from(size.unwrap_or(0));
    if size > MAX_ATTACHMENT_BYTES {
        return Some(format!(
            "{ATTACHMENT_POLICY_REJECT_PREFIX}：附件大小 {} 超过 5MB 上限",
            human_size_bytes(size)
        ));
    }

    if kind == AttachmentKind::Image && size > MAX_IMAGE_BYTES {
        return Some(format!(
            "{ATTACHMENT_POLICY_REJECT_PREFIX}：图片大小 {} 超过 3MB 上限",
            human_size_bytes(size)
        ));
    }

    None
}

pub async fn enrich_attachment(attachment: ReceivedAttachment) -> ReceivedAttachment {
    enrich_attachment_with_extract_dir(attachment, None).await
}

pub async fn enrich_attachment_with_extract_dir(
    mut attachment: ReceivedAttachment,
    extract_dir: Option<PathBuf>,
) -> ReceivedAttachment {
    let Some(local_path) = attachment.local_path.clone() else {
        return attachment;
    };
    let path = PathBuf::from(local_path);

    if attachment.kind == AttachmentKind::Archive {
        let target_dir = extract_dir.unwrap_or_else(|| {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!(
                    "{}_extracted",
                    sanitize_filename(&attachment.filename)
                ))
        });
        match extract_archive_with_limits(path.clone(), target_dir).await {
            Ok(files) => attachment.extracted_files = files,
            Err(err) => attachment.extraction_error = Some(err),
        }
    } else if attachment.kind == AttachmentKind::Pdf {
        match super::vector_store::extract_pdf_preview(path).await {
            Ok(preview) => attachment.pdf_text_preview = Some(preview),
            Err(err) => attachment.pdf_extract_error = Some(err),
        }
    }

    attachment
}

pub fn build_user_input(content: &str, attachments: &[ReceivedAttachment]) -> String {
    build_user_input_with_label(content, attachments, "用户上传了附件：")
}

pub fn build_user_input_with_label(
    content: &str,
    attachments: &[ReceivedAttachment],
    label: &str,
) -> String {
    let mut parts = Vec::new();

    if !content.trim().is_empty() {
        parts.push(content.trim().to_string());
    }

    let accepted_attachments: Vec<&ReceivedAttachment> = attachments
        .iter()
        .filter(|att| att.error.is_none())
        .collect();

    if !accepted_attachments.is_empty() {
        let mut lines = vec![label.to_string()];
        for (i, att) in accepted_attachments.iter().enumerate() {
            lines.push(att.as_prompt_line(i + 1));
        }
        parts.push(lines.join("\n"));
        if let Some(pdf_note) = build_pdf_extraction_note_from_refs(&accepted_attachments) {
            parts.push(pdf_note);
        }
        if let Some(archive_note) = build_archive_extraction_note_from_refs(&accepted_attachments) {
            parts.push(archive_note);
        }
        parts.push(build_attachment_strategy_note_from_refs(
            &accepted_attachments,
        ));
    }

    parts.join("\n\n")
}

pub fn build_attachment_ack_message(attachments: &[ReceivedAttachment]) -> String {
    let mut counts = BTreeMap::new();
    let accepted_attachments: Vec<&ReceivedAttachment> = attachments
        .iter()
        .filter(|att| att.error.is_none())
        .collect();
    let rejected_attachments: Vec<&ReceivedAttachment> = attachments
        .iter()
        .filter(|att| att.error.is_some())
        .collect();

    for att in &accepted_attachments {
        *counts.entry(att.kind.label()).or_insert(0usize) += 1;
    }
    let mut msg = if accepted_attachments.is_empty() {
        "收到附件，但都未通过准入限制。".to_string()
    } else {
        let kinds = counts
            .into_iter()
            .map(|(k, v)| format!("{k}x{v}"))
            .collect::<Vec<_>>()
            .join("、");
        format!(
            "已收到 {} 个可处理附件（{}），正在解析。",
            accepted_attachments.len(),
            kinds
        )
    };

    let archive_status: Vec<String> = accepted_attachments
        .iter()
        .filter(|a| a.kind == AttachmentKind::Archive)
        .map(|a| {
            if let Some(err) = &a.extraction_error {
                format!("{} 解压失败: {}", a.filename, truncate_chars(err, 80))
            } else {
                format!("{} 已解压 {} 个文件", a.filename, a.extracted_files.len())
            }
        })
        .collect();
    if !archive_status.is_empty() {
        msg.push_str(" 压缩包处理：");
        msg.push_str(&archive_status.join("；"));
        msg.push('。');
    }

    let pdf_status: Vec<String> = accepted_attachments
        .iter()
        .filter(|a| a.kind == AttachmentKind::Pdf)
        .map(|a| {
            if let Some(err) = &a.pdf_extract_error {
                format!("{} 解析失败: {}", a.filename, truncate_chars(err, 80))
            } else if a.pdf_text_preview.is_some() {
                format!("{} 已提取文本", a.filename)
            } else {
                format!("{} 未提取到文本", a.filename)
            }
        })
        .collect();
    if !pdf_status.is_empty() {
        msg.push_str(" PDF处理：");
        msg.push_str(&pdf_status.join("；"));
        msg.push('。');
    }

    let rejected_status: Vec<String> = rejected_attachments
        .iter()
        .map(|a| {
            let reason = a.error.as_deref().unwrap_or("未通过准入限制");
            format!("{}：{}", a.filename, truncate_chars(reason, 120))
        })
        .collect();
    if !rejected_status.is_empty() {
        msg.push_str(" 已拦截附件：");
        msg.push_str(&rejected_status.join("；"));
        msg.push('。');
    }

    msg
}

pub fn infer_attachment_kind(content_type: Option<&str>, filename: &str) -> AttachmentKind {
    let ct = content_type.unwrap_or("").to_ascii_lowercase();
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    if ct.starts_with("image/")
        || matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "heic" | "svg"
        )
    {
        return AttachmentKind::Image;
    }
    if ct == "application/pdf" || ext == "pdf" {
        return AttachmentKind::Pdf;
    }
    if ct.contains("spreadsheet")
        || ct.contains("csv")
        || matches!(ext.as_str(), "csv" | "tsv" | "xls" | "xlsx" | "numbers")
    {
        return AttachmentKind::Spreadsheet;
    }
    if ct.starts_with("text/")
        || matches!(
            ext.as_str(),
            "txt" | "md" | "json" | "yaml" | "yml" | "log" | "xml" | "html" | "htm"
        )
    {
        return AttachmentKind::Text;
    }
    if ct.starts_with("audio/")
        || matches!(ext.as_str(), "mp3" | "wav" | "m4a" | "aac" | "ogg" | "flac")
    {
        return AttachmentKind::Audio;
    }
    if ct.starts_with("video/") || matches!(ext.as_str(), "mp4" | "mov" | "mkv" | "webm" | "avi") {
        return AttachmentKind::Video;
    }
    if ct.contains("zip")
        || ct.contains("tar")
        || matches!(ext.as_str(), "zip" | "rar" | "7z" | "tar" | "gz" | "tgz")
    {
        return AttachmentKind::Archive;
    }
    AttachmentKind::Other
}

pub fn sanitize_filename(name: &str) -> String {
    let mut safe: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if safe.is_empty() {
        safe = "attachment.bin".to_string();
    }
    if safe.len() > 120 {
        safe.truncate(120);
    }
    safe
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

fn human_size_bytes(bytes: u64) -> String {
    format!("{:.1}MB", bytes as f64 / 1024f64 / 1024f64)
}

fn build_attachment_strategy_note_from_refs(attachments: &[&ReceivedAttachment]) -> String {
    let mut lines = vec![
        "【附件默认处理策略】".to_string(),
        "请根据附件类型采用默认流程，并结合用户问题给出结果：".to_string(),
    ];

    let has_image = attachments.iter().any(|a| a.kind == AttachmentKind::Image);
    let has_pdf = attachments.iter().any(|a| a.kind == AttachmentKind::Pdf);
    let has_sheet = attachments
        .iter()
        .any(|a| a.kind == AttachmentKind::Spreadsheet);
    let has_text = attachments.iter().any(|a| a.kind == AttachmentKind::Text);
    let has_audio = attachments.iter().any(|a| a.kind == AttachmentKind::Audio);
    let has_video = attachments.iter().any(|a| a.kind == AttachmentKind::Video);
    let has_archive = attachments
        .iter()
        .any(|a| a.kind == AttachmentKind::Archive);

    if has_image {
        lines.push(
            "- 图片：优先调用 skill_tool(skill_name=\"image_understanding\")，按截图/图表理解流程处理。"
                .to_string(),
        );
    }
    if has_pdf {
        lines.push(
            "- PDF：优先调用 skill_tool(skill_name=\"pdf_understanding\")；先使用“PDF提取文本”中的内容作答。若文本缺失，明确说明可能是扫描件并引导用户提供可复制文本/OCR。"
                .to_string(),
        );
    }
    if has_sheet {
        lines.push(
            "- 表格：优先调用 skill_tool(skill_name=\"portfolio_management\")，提取结构化字段（代码、数量、价格、时间），给出统计与异常值提示。"
                .to_string(),
        );
    }
    if has_text {
        lines.push("- 文本：先做摘要与要点抽取，再回答用户问题。".to_string());
    }
    if has_audio || has_video {
        lines.push(
            "- 音视频：当前无稳定转写工具时，先明确告知限制，并让用户补充关键信息或文字稿。"
                .to_string(),
        );
    }
    if has_archive {
        lines.push(
            "- 压缩包：后端已自动解压；请逐个处理“压缩包解压文件清单”里的文件，再给综合结论。"
                .to_string(),
        );
    }
    if lines.len() <= 2 {
        lines.push("- 其他文件：先解释可处理范围，再让用户说明希望如何处理。".to_string());
    }

    lines.join("\n")
}

fn build_pdf_extraction_note_from_refs(attachments: &[&ReceivedAttachment]) -> Option<String> {
    let pdfs: Vec<&ReceivedAttachment> = attachments
        .iter()
        .copied()
        .filter(|a| a.kind == AttachmentKind::Pdf)
        .collect();
    if pdfs.is_empty() {
        return None;
    }

    let mut lines = vec![
        "【PDF提取文本】".to_string(),
        "后端已尝试提取 PDF 文本，请优先基于以下内容回答：".to_string(),
    ];

    for pdf in pdfs {
        if let Some(err) = &pdf.pdf_extract_error {
            lines.push(format!(
                "- {}: 提取失败（{}）",
                pdf.filename,
                truncate_chars(err, 120)
            ));
            continue;
        }
        if let Some(preview) = &pdf.pdf_text_preview {
            lines.push(format!("- {}: 已提取文本片段", pdf.filename));
            lines.push(format!("  {}", preview.replace('\n', "\\n")));
        } else {
            lines.push(format!(
                "- {}: 未提取到可读文本（可能是扫描件）",
                pdf.filename
            ));
        }
    }

    Some(lines.join("\n"))
}

fn build_archive_extraction_note_from_refs(attachments: &[&ReceivedAttachment]) -> Option<String> {
    let archives: Vec<&ReceivedAttachment> = attachments
        .iter()
        .copied()
        .filter(|a| a.kind == AttachmentKind::Archive)
        .collect();
    if archives.is_empty() {
        return None;
    }

    let mut lines = vec![
        "【压缩包解压文件清单】".to_string(),
        "后端已自动解压，请逐个查看下列文件并分析：".to_string(),
    ];

    for archive in archives {
        if let Some(err) = &archive.extraction_error {
            lines.push(format!(
                "- {}: 解压失败（{}）",
                archive.filename,
                truncate_chars(err, 120)
            ));
            continue;
        }
        if archive.extracted_files.is_empty() {
            lines.push(format!("- {}: 未提取到有效文件", archive.filename));
            continue;
        }

        lines.push(format!(
            "- {}: 共 {} 个文件",
            archive.filename,
            archive.extracted_files.len()
        ));
        for (idx, file) in archive
            .extracted_files
            .iter()
            .take(MAX_ARCHIVE_PROMPT_FILES)
            .enumerate()
        {
            lines.push(format!(
                "  {}. {} [{} {}B]",
                idx + 1,
                file.path,
                file.kind.label(),
                file.size
            ));
            if let Some(preview) = &file.preview {
                lines.push(format!("     预览: {}", preview.replace('\n', "\\n")));
            }
        }
        if archive.extracted_files.len() > MAX_ARCHIVE_PROMPT_FILES {
            lines.push(format!(
                "  ... 其余 {} 个文件已省略",
                archive.extracted_files.len() - MAX_ARCHIVE_PROMPT_FILES
            ));
        }
    }

    Some(lines.join("\n"))
}

async fn extract_archive_with_limits(
    archive_path: PathBuf,
    extract_dir: PathBuf,
) -> Result<Vec<ExtractedFileInfo>, String> {
    task::spawn_blocking(move || extract_archive_with_limits_blocking(&archive_path, &extract_dir))
        .await
        .map_err(|e| format!("解压任务失败: {e}"))?
}

fn extract_archive_with_limits_blocking(
    archive_path: &Path,
    extract_dir: &Path,
) -> Result<Vec<ExtractedFileInfo>, String> {
    fs::create_dir_all(extract_dir).map_err(|e| format!("创建解压目录失败: {e}"))?;
    let filename_lower = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if filename_lower.ends_with(".zip") {
        extract_zip_archive(archive_path, extract_dir)
    } else if filename_lower.ends_with(".tar.gz") || filename_lower.ends_with(".tgz") {
        let file = fs::File::open(archive_path).map_err(|e| format!("打开压缩包失败: {e}"))?;
        let gz = GzDecoder::new(file);
        extract_tar_archive(gz, extract_dir)
    } else if filename_lower.ends_with(".tar") {
        let file = fs::File::open(archive_path).map_err(|e| format!("打开压缩包失败: {e}"))?;
        extract_tar_archive(file, extract_dir)
    } else {
        Err("暂不支持该压缩格式自动解压（仅支持 zip/tar/tar.gz/tgz）".to_string())
    }
}

fn extract_zip_archive(
    archive_path: &Path,
    extract_dir: &Path,
) -> Result<Vec<ExtractedFileInfo>, String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("打开 zip 失败: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("读取 zip 失败: {e}"))?;
    let mut total_bytes: u64 = 0;
    let mut files = Vec::new();

    for idx in 0..archive.len() {
        if files.len() >= MAX_ARCHIVE_EXTRACTED_FILES {
            return Err(format!(
                "压缩包文件数超过限制（>{}）",
                MAX_ARCHIVE_EXTRACTED_FILES
            ));
        }
        let mut entry = archive
            .by_index(idx)
            .map_err(|e| format!("读取 zip 条目失败: {e}"))?;
        if entry.name().ends_with('/') {
            continue;
        }
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| format!("zip 包含非法路径: {}", entry.name()))?;
        let rel = sanitize_relative_path(enclosed)?;
        let target = extract_dir.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        let mut output = fs::File::create(&target).map_err(|e| format!("创建文件失败: {e}"))?;
        let written =
            copy_reader_with_limit(&mut entry, &mut output, MAX_ARCHIVE_SINGLE_FILE_BYTES)?;
        total_bytes = total_bytes.saturating_add(written);
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(format!(
                "压缩包解压总大小超过限制（>{}MB）",
                MAX_ARCHIVE_TOTAL_BYTES / 1024 / 1024
            ));
        }
        files.push(build_extracted_file_info(&target, written));
    }

    Ok(files)
}

fn extract_tar_archive<R: Read>(
    reader: R,
    extract_dir: &Path,
) -> Result<Vec<ExtractedFileInfo>, String> {
    let mut archive = Archive::new(reader);
    let mut total_bytes: u64 = 0;
    let mut files = Vec::new();

    for entry in archive
        .entries()
        .map_err(|e| format!("读取 tar 条目失败: {e}"))?
    {
        if files.len() >= MAX_ARCHIVE_EXTRACTED_FILES {
            return Err(format!(
                "压缩包文件数超过限制（>{}）",
                MAX_ARCHIVE_EXTRACTED_FILES
            ));
        }

        let mut entry = entry.map_err(|e| format!("读取 tar 条目失败: {e}"))?;
        if entry.header().entry_type().is_dir() {
            continue;
        }

        let rel_raw = entry
            .path()
            .map_err(|e| format!("读取 tar 路径失败: {e}"))?;
        let rel = sanitize_relative_path(&rel_raw)?;
        let target = extract_dir.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }

        let mut output = fs::File::create(&target).map_err(|e| format!("创建文件失败: {e}"))?;
        let written =
            copy_reader_with_limit(&mut entry, &mut output, MAX_ARCHIVE_SINGLE_FILE_BYTES)?;
        total_bytes = total_bytes.saturating_add(written);
        if total_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err(format!(
                "压缩包解压总大小超过限制（>{}MB）",
                MAX_ARCHIVE_TOTAL_BYTES / 1024 / 1024
            ));
        }
        files.push(build_extracted_file_info(&target, written));
    }

    Ok(files)
}

fn copy_reader_with_limit<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    limit: u64,
) -> Result<u64, String> {
    let mut total = 0u64;
    let mut buf = [0u8; 8192];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("读取压缩条目失败: {e}"))?;
        if n == 0 {
            break;
        }
        total = total.saturating_add(n as u64);
        if total > limit {
            return Err(format!(
                "单文件解压后超过限制（>{}MB）",
                limit / 1024 / 1024
            ));
        }
        writer
            .write_all(&buf[..n])
            .map_err(|e| format!("写入解压文件失败: {e}"))?;
    }
    Ok(total)
}

fn sanitize_relative_path(path: &Path) -> Result<PathBuf, String> {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            _ => return Err(format!("压缩包包含非法路径: {}", path.display())),
        }
    }
    if clean.as_os_str().is_empty() {
        return Err("压缩包条目路径为空".to_string());
    }
    Ok(clean)
}

fn build_extracted_file_info(path: &Path, size: u64) -> ExtractedFileInfo {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let kind = infer_attachment_kind(None, filename);
    let preview = maybe_read_text_preview(path, kind, size);
    ExtractedFileInfo {
        path: path.display().to_string(),
        size,
        kind,
        preview,
    }
}

fn maybe_read_text_preview(path: &Path, kind: AttachmentKind, size: u64) -> Option<String> {
    if !matches!(kind, AttachmentKind::Text | AttachmentKind::Spreadsheet) {
        return None;
    }
    if size > MAX_ARCHIVE_PREVIEW_BYTES {
        return None;
    }
    let data = fs::read(path).ok()?;
    if data.contains(&0) {
        return None;
    }
    let preview = String::from_utf8_lossy(&data).trim().to_string();
    if preview.is_empty() {
        return None;
    }
    Some(truncate_chars(&preview, MAX_ARCHIVE_PREVIEW_CHARS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sanitize_filename_replaces_unsafe_chars() {
        assert_eq!(sanitize_filename("a b/c?.png"), "a_b_c_.png");
        assert_eq!(sanitize_filename(""), "attachment.bin");
    }

    #[test]
    fn infer_attachment_kind_from_content_type_and_extension() {
        assert_eq!(
            infer_attachment_kind(Some("image/png"), "x.bin"),
            AttachmentKind::Image
        );
        assert_eq!(
            infer_attachment_kind(Some("application/pdf"), "report.dat"),
            AttachmentKind::Pdf
        );
        assert_eq!(
            infer_attachment_kind(None, "positions.xlsx"),
            AttachmentKind::Spreadsheet
        );
        assert_eq!(
            infer_attachment_kind(None, "note.txt"),
            AttachmentKind::Text
        );
        assert_eq!(
            infer_attachment_kind(None, "call.mp3"),
            AttachmentKind::Audio
        );
        assert_eq!(
            infer_attachment_kind(None, "screen.mp4"),
            AttachmentKind::Video
        );
        assert_eq!(
            infer_attachment_kind(None, "pack.zip"),
            AttachmentKind::Archive
        );
        assert_eq!(
            infer_attachment_kind(None, "unknown.bin"),
            AttachmentKind::Other
        );
    }

    #[test]
    fn build_user_input_includes_attachment_notes() {
        let attachments = vec![ReceivedAttachment {
            filename: "report.pdf".to_string(),
            content_type: Some("application/pdf".to_string()),
            size: 12,
            url: "feishu://x".to_string(),
            kind: AttachmentKind::Pdf,
            local_path: Some("/tmp/report.pdf".to_string()),
            error: None,
            extracted_files: vec![],
            extraction_error: None,
            pdf_text_preview: Some("Revenue up 20% YoY.".to_string()),
            pdf_extract_error: None,
        }];
        let prompt = build_user_input("帮我看这个 PDF", &attachments);
        assert!(prompt.contains("用户上传了附件"));
        assert!(prompt.contains("PDF提取文本"));
        assert!(prompt.contains("Revenue up 20% YoY."));
    }

    #[test]
    fn build_user_input_excludes_rejected_attachments() {
        let attachments = vec![
            ReceivedAttachment {
                filename: "ok.pdf".to_string(),
                content_type: Some("application/pdf".to_string()),
                size: 12,
                url: "feishu://ok".to_string(),
                kind: AttachmentKind::Pdf,
                local_path: Some("/tmp/ok.pdf".to_string()),
                error: None,
                extracted_files: vec![],
                extraction_error: None,
                pdf_text_preview: Some("accepted".to_string()),
                pdf_extract_error: None,
            },
            ReceivedAttachment {
                filename: "bad.png".to_string(),
                content_type: Some("image/png".to_string()),
                size: 4 * 1024 * 1024,
                url: "feishu://bad".to_string(),
                kind: AttachmentKind::Image,
                local_path: None,
                error: Some("附件未通过准入限制：图片大小 4.0MB 超过 3MB 上限".to_string()),
                extracted_files: vec![],
                extraction_error: None,
                pdf_text_preview: None,
                pdf_extract_error: None,
            },
        ];

        let prompt = build_user_input("请看", &attachments);
        assert!(prompt.contains("ok.pdf"));
        assert!(!prompt.contains("bad.png"));
    }

    #[test]
    fn archive_extraction_blocks_path_escape() {
        assert!(sanitize_relative_path(Path::new("../a.txt")).is_err());
        assert!(sanitize_relative_path(Path::new("/tmp/a.txt")).is_err());
        assert!(sanitize_relative_path(Path::new("safe/a.txt")).is_ok());
    }

    #[test]
    fn zip_extraction_collects_text_preview() {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("epoch")
            .as_millis();
        let base = std::env::temp_dir().join(format!("hone-attach-test-{millis}"));
        fs::create_dir_all(&base).expect("create temp dir");
        let zip_path = base.join("sample.zip");
        let out_dir = base.join("out");

        {
            let file = fs::File::create(&zip_path).expect("create zip");
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::FileOptions::default();
            zip.start_file("nested/readme.txt", options)
                .expect("start file");
            zip.write_all(b"hello attachment world")
                .expect("write file");
            zip.finish().expect("finish zip");
        }

        let files = extract_archive_with_limits_blocking(&zip_path, &out_dir).expect("extract zip");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].kind, AttachmentKind::Text);
        assert!(
            files[0]
                .preview
                .as_deref()
                .unwrap_or_default()
                .contains("hello attachment world")
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn archive_note_contains_extracted_file_list() {
        let attachments = vec![ReceivedAttachment {
            filename: "pack.zip".to_string(),
            content_type: Some("application/zip".to_string()),
            size: 123,
            url: "https://example.com/pack.zip".to_string(),
            kind: AttachmentKind::Archive,
            local_path: Some("./data/discord_uploads/x/pack.zip".to_string()),
            error: None,
            extracted_files: vec![ExtractedFileInfo {
                path: "./data/discord_uploads/x/pack/readme.txt".to_string(),
                size: 32,
                kind: AttachmentKind::Text,
                preview: Some("hello world".to_string()),
            }],
            extraction_error: None,
            pdf_text_preview: None,
            pdf_extract_error: None,
        }];

        let refs: Vec<&ReceivedAttachment> = attachments.iter().collect();
        let note = build_archive_extraction_note_from_refs(&refs).unwrap_or_default();
        assert!(note.contains("压缩包解压文件清单"));
        assert!(note.contains("readme.txt"));
        assert!(note.contains("hello world"));
    }

    #[test]
    fn pdf_note_contains_extracted_text() {
        let attachments = vec![ReceivedAttachment {
            filename: "report.pdf".to_string(),
            content_type: Some("application/pdf".to_string()),
            size: 4096,
            url: "https://example.com/report.pdf".to_string(),
            kind: AttachmentKind::Pdf,
            local_path: Some("./data/discord_uploads/x/report.pdf".to_string()),
            error: None,
            extracted_files: Vec::new(),
            extraction_error: None,
            pdf_text_preview: Some("Revenue up 20% YoY.".to_string()),
            pdf_extract_error: None,
        }];

        let refs: Vec<&ReceivedAttachment> = attachments.iter().collect();
        let note = build_pdf_extraction_note_from_refs(&refs).unwrap_or_default();
        assert!(note.contains("PDF提取文本"));
        assert!(note.contains("report.pdf"));
        assert!(note.contains("Revenue up 20% YoY."));
    }

    #[test]
    fn attachment_policy_rejects_oversized_file() {
        let reason = validate_attachment_policy(
            AttachmentKind::Other,
            Some((MAX_ATTACHMENT_BYTES + 1) as u32),
        )
        .unwrap_or_default();
        assert!(reason.contains("超过 5MB 上限"));
    }

    #[test]
    fn attachment_policy_rejects_oversized_image() {
        let reason =
            validate_attachment_policy(AttachmentKind::Image, Some((MAX_IMAGE_BYTES + 1) as u32))
                .unwrap_or_default();
        assert!(reason.contains("超过 3MB 上限"));
    }

    #[test]
    fn attachment_policy_allows_regular_file() {
        assert!(validate_attachment_policy(AttachmentKind::Text, Some(1024)).is_none());
    }

    #[test]
    fn ack_message_summarizes_rejected_attachments() {
        let attachments = vec![ReceivedAttachment {
            filename: "bad.png".to_string(),
            content_type: Some("image/png".to_string()),
            size: 4 * 1024 * 1024,
            url: "https://example.com/bad.png".to_string(),
            kind: AttachmentKind::Image,
            local_path: None,
            error: Some("附件未通过准入限制：图片大小 4.0MB 超过 3MB 上限".to_string()),
            extracted_files: vec![],
            extraction_error: None,
            pdf_text_preview: None,
            pdf_extract_error: None,
        }];

        let msg = build_attachment_ack_message(&attachments);
        assert!(msg.contains("都未通过准入限制"));
        assert!(msg.contains("bad.png"));
    }

    #[test]
    fn image_shape_rejects_long_edge() {
        let path = write_test_image(4097, 1000, "long-edge.png");
        let reason = validate_image_shape_blocking(&path, "long-edge.png").unwrap_or_default();
        assert!(reason.contains("最长边 4097px"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn image_shape_rejects_total_pixels() {
        let path = write_test_image(4000, 4000, "too-many-pixels.png");
        let reason =
            validate_image_shape_blocking(&path, "too-many-pixels.png").unwrap_or_default();
        assert!(reason.contains("超过 1200 万像素上限"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn image_shape_rejects_aspect_ratio() {
        let path = write_test_image(4000, 900, "too-wide.png");
        let reason = validate_image_shape_blocking(&path, "too-wide.png").unwrap_or_default();
        assert!(reason.contains("图片比例异常"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn image_shape_allows_regular_dimensions() {
        let path = write_test_image(1200, 900, "regular.png");
        assert!(validate_image_shape_blocking(&path, "regular.png").is_none());
        let _ = fs::remove_file(path);
    }

    fn write_test_image(width: u32, height: u32, name: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("epoch")
            .as_millis();
        let path = std::env::temp_dir().join(format!("hone-attach-{millis}-{name}"));
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(width, height, Rgb([255, 255, 255]));
        img.save(&path).expect("save image");
        path
    }
}
