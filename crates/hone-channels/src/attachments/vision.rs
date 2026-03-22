//! Attachment image validation helpers.

use std::path::{Path, PathBuf};

use image::ImageReader;
use tokio::task;

use super::ingest::{AttachmentKind, ReceivedAttachment};

const MAX_IMAGE_LONG_EDGE: u32 = 4096;
const MAX_IMAGE_TOTAL_PIXELS: u64 = 12_000_000;
const MAX_IMAGE_ASPECT_RATIO: u32 = 4;
const ATTACHMENT_POLICY_REJECT_PREFIX: &str = "附件未通过准入限制";

pub(crate) async fn validate_attachment_image_shape(
    attachment: &ReceivedAttachment,
) -> Option<String> {
    if attachment.kind != AttachmentKind::Image {
        return None;
    }
    let local_path = attachment.local_path.clone()?;
    let path = PathBuf::from(local_path);
    let filename = attachment.filename.clone();

    task::spawn_blocking(move || validate_image_shape_blocking(&path, &filename))
        .await
        .ok()
        .flatten()
}

pub(crate) fn validate_image_shape_blocking(path: &Path, filename: &str) -> Option<String> {
    let reader = ImageReader::open(path).ok()?;
    let reader = reader.with_guessed_format().ok()?;
    let (width, height) = reader.into_dimensions().ok()?;

    if width == 0 || height == 0 {
        return Some(format!(
            "{ATTACHMENT_POLICY_REJECT_PREFIX}：图片 {filename} 尺寸无效"
        ));
    }

    let long_edge = width.max(height);
    if long_edge > MAX_IMAGE_LONG_EDGE {
        return Some(format!(
            "{ATTACHMENT_POLICY_REJECT_PREFIX}：图片分辨率过大（最长边 {long_edge}px，超过 4096px 上限）"
        ));
    }

    let total_pixels = u64::from(width) * u64::from(height);
    if total_pixels > MAX_IMAGE_TOTAL_PIXELS {
        return Some(format!(
            "{ATTACHMENT_POLICY_REJECT_PREFIX}：图片总像素过大（{}，超过 1200 万像素上限）",
            human_pixels(total_pixels)
        ));
    }

    let wide = width.max(height);
    let narrow = width.min(height);
    if u64::from(wide) > u64::from(narrow) * u64::from(MAX_IMAGE_ASPECT_RATIO) {
        return Some(format!(
            "{ATTACHMENT_POLICY_REJECT_PREFIX}：图片比例异常（{width}x{height}，超出 1:4 到 4:1 范围）"
        ));
    }

    None
}

fn human_pixels(total_pixels: u64) -> String {
    format!("{:.1}MP", total_pixels as f64 / 1_000_000f64)
}
