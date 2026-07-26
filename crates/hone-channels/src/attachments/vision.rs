//! Attachment image validation helpers.

use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::sync::OnceLock;

use image::ImageReader;
use serde::Deserialize;
#[cfg(target_os = "macos")]
use tokio::process::Command;
#[cfg(target_os = "macos")]
use tokio::sync::Mutex;
use tokio::task;

use super::ingest::{AttachmentKind, ReceivedAttachment};

const MAX_IMAGE_LONG_EDGE: u32 = 4096;
const MAX_IMAGE_TOTAL_PIXELS: u64 = 12_000_000;
const MAX_IMAGE_ASPECT_RATIO: u32 = 4;
const ATTACHMENT_POLICY_REJECT_PREFIX: &str = "附件未通过准入限制";
const MAX_IMAGE_OCR_CHARS: usize = 16_000;
#[cfg(target_os = "macos")]
const IMAGE_OCR_HELPER_REVISION: &str = "v1";
#[cfg(target_os = "macos")]
const IMAGE_OCR_SWIFT_SOURCE: &str = r#"
import AppKit
import Foundation
import Vision

struct RecognizedLine {
    let text: String
    let x: CGFloat
    let midY: CGFloat
    let height: CGFloat
}

struct Output: Codable {
    let success: Bool
    let text: String
}

struct Failure: Codable {
    let success: Bool
    let error: String
}

func fail(_ message: String, code: Int32) -> Never {
    let data = try? JSONEncoder().encode(Failure(success: false, error: message))
    if let data, let value = String(data: data, encoding: .utf8) {
        print(value)
    }
    exit(code)
}

guard CommandLine.arguments.count == 2 else {
    fail("expected one image path", code: 2)
}

let url = URL(fileURLWithPath: CommandLine.arguments[1])
guard let image = NSImage(contentsOf: url),
      let data = image.tiffRepresentation,
      let bitmap = NSBitmapImageRep(data: data),
      let cgImage = bitmap.cgImage else {
    fail("image could not be decoded", code: 3)
}

let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = true
request.recognitionLanguages = ["zh-Hans", "en-US"]
let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])

do {
    try handler.perform([request])
} catch {
    fail("Vision text recognition failed", code: 4)
}

var recognized: [RecognizedLine] = []
for observation in request.results ?? [] {
    guard let text = observation.topCandidates(1).first?.string
        .trimmingCharacters(in: .whitespacesAndNewlines),
        !text.isEmpty else {
        continue
    }
    let box = observation.boundingBox
    recognized.append(
        RecognizedLine(text: text, x: box.minX, midY: box.midY, height: box.height)
    )
}

recognized.sort {
    if abs($0.midY - $1.midY) > max($0.height, $1.height) * 0.55 {
        return $0.midY > $1.midY
    }
    return $0.x < $1.x
}

var rows: [[RecognizedLine]] = []
var rowMidpoints: [CGFloat] = []
for line in recognized {
    if let index = rowMidpoints.indices.last,
       abs(rowMidpoints[index] - line.midY) <= max(line.height * 0.65, 0.008) {
        rows[index].append(line)
        rowMidpoints[index] =
            rows[index].map(\.midY).reduce(0, +) / CGFloat(rows[index].count)
    } else {
        rows.append([line])
        rowMidpoints.append(line.midY)
    }
}

let text = rows.map { row in
    row.sorted { $0.x < $1.x }.map(\.text).joined(separator: " | ")
}.joined(separator: "\n")

let output = Output(success: true, text: text)
let encoded = try JSONEncoder().encode(output)
print(String(decoding: encoded, as: UTF8.self))
"#;

#[derive(Debug, Deserialize)]
struct ImageOcrOutput {
    success: bool,
    #[serde(default)]
    text: String,
    #[serde(default)]
    error: String,
}

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

pub(crate) async fn extract_image_text(path: &Path) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let helper = image_ocr_helper().await?;
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            Command::new(&helper).arg(path).output(),
        )
        .await
        .map_err(|_| "图片文字提取超时".to_string())?
        .map_err(|err| format!("图片文字提取启动失败: {err}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: ImageOcrOutput = serde_json::from_str(stdout.trim())
            .map_err(|err| format!("图片文字提取结果无法解析: {err}"))?;
        if !output.status.success() || !parsed.success {
            return Err(if parsed.error.trim().is_empty() {
                "图片文字提取未成功".to_string()
            } else {
                parsed.error
            });
        }
        let text = truncate_ocr_text(&parsed.text);
        if text.is_empty() {
            return Err("图片中未提取到可读文字".to_string());
        }
        return Ok(text);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err("当前主机未启用本地图片文字提取".to_string())
    }
}

fn truncate_ocr_text(text: &str) -> String {
    let normalized = text
        .replace("\r\n", "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if normalized.chars().count() <= MAX_IMAGE_OCR_CHARS {
        return normalized;
    }
    normalized
        .chars()
        .take(MAX_IMAGE_OCR_CHARS)
        .collect::<String>()
        + "\n[文字提取已截断]"
}

#[cfg(target_os = "macos")]
async fn image_ocr_helper() -> Result<PathBuf, String> {
    static HELPER: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    let helper = HELPER.get_or_init(|| Mutex::new(None));
    let mut cached = helper.lock().await;
    if let Some(path) = cached.as_ref().filter(|path| path.is_file()) {
        return Ok(path.clone());
    }

    let cache_dir = std::env::temp_dir().join(format!(
        "hone-image-ocr-{IMAGE_OCR_HELPER_REVISION}-{}",
        std::env::consts::ARCH
    ));
    tokio::fs::create_dir_all(&cache_dir)
        .await
        .map_err(|err| format!("图片文字提取缓存目录创建失败: {err}"))?;
    let source_path = cache_dir.join("main.swift");
    let helper_path = cache_dir.join("hone-image-ocr");
    if !helper_path.is_file() {
        tokio::fs::write(&source_path, IMAGE_OCR_SWIFT_SOURCE)
            .await
            .map_err(|err| format!("图片文字提取 helper 写入失败: {err}"))?;
        let temporary_helper = cache_dir.join(format!("hone-image-ocr-{}", std::process::id()));
        let compile = tokio::time::timeout(
            std::time::Duration::from_secs(45),
            Command::new("/usr/bin/swiftc")
                .arg("-O")
                .arg(&source_path)
                .arg("-o")
                .arg(&temporary_helper)
                .output(),
        )
        .await
        .map_err(|_| "图片文字提取 helper 编译超时".to_string())?
        .map_err(|err| format!("图片文字提取 helper 编译启动失败: {err}"))?;
        if !compile.status.success() {
            let detail = String::from_utf8_lossy(&compile.stderr);
            return Err(format!(
                "图片文字提取 helper 编译失败: {}",
                detail.chars().take(500).collect::<String>()
            ));
        }
        match tokio::fs::rename(&temporary_helper, &helper_path).await {
            Ok(()) => {}
            Err(err) if helper_path.is_file() => {
                let _ = tokio::fs::remove_file(&temporary_helper).await;
                tracing::debug!("image OCR helper won concurrent install: {err}");
            }
            Err(err) => {
                return Err(format!("图片文字提取 helper 安装失败: {err}"));
            }
        }
    }
    *cached = Some(helper_path.clone());
    Ok(helper_path)
}

fn human_pixels(total_pixels: u64) -> String {
    format!("{:.1}MP", total_pixels as f64 / 1_000_000f64)
}

#[cfg(test)]
mod tests {
    use super::{extract_image_text, truncate_ocr_text};

    #[test]
    fn ocr_text_normalization_removes_empty_rows_without_flattening_columns() {
        assert_eq!(
            truncate_ocr_text("  CRWV | 72.07 | 139  \r\n\r\n  NBIS | 189.78 | 170 "),
            "CRWV | 72.07 | 139\nNBIS | 189.78 | 170"
        );
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    #[ignore = "requires HONE_TEST_IMAGE_OCR_PATH and the local Apple Vision framework"]
    async fn local_apple_vision_extracts_text_from_a_real_attachment() {
        let path = std::env::var("HONE_TEST_IMAGE_OCR_PATH")
            .expect("set HONE_TEST_IMAGE_OCR_PATH to a real local image");
        let text = extract_image_text(std::path::Path::new(&path))
            .await
            .expect("extract image text");
        assert!(!text.trim().is_empty());
    }
}
