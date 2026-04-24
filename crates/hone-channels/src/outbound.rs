use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::agent_session::{
    AgentRunOptions, AgentSession, AgentSessionEvent, AgentSessionListener, AgentSessionResult,
};
use crate::run_event::RunEvent;
use crate::runtime::{flush_buffer, tool_display_map, user_visible_error_message};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasoningVisibility {
    Hidden,
    Full,
    Compact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalImageMarker {
    pub uri: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResponseContentSegment {
    Text(String),
    LocalImage(LocalImageMarker),
}

pub const LOCAL_IMAGE_CONTEXT_PLACEHOLDER: &str = "（上文包含图表）";

pub fn split_response_content_segments(text: &str) -> Vec<ResponseContentSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;

    while let Some((start, end, marker)) = find_next_local_image_reference(text, cursor) {
        if start > cursor {
            segments.push(ResponseContentSegment::Text(
                text[cursor..start].to_string(),
            ));
        }
        segments.push(ResponseContentSegment::LocalImage(marker));
        cursor = end;
    }

    if cursor < text.len() {
        segments.push(ResponseContentSegment::Text(text[cursor..].to_string()));
    }

    if segments.is_empty() && !text.is_empty() {
        segments.push(ResponseContentSegment::Text(text.to_string()));
    }

    segments
}

pub fn collect_local_image_markers(text: &str) -> Vec<LocalImageMarker> {
    split_response_content_segments(text)
        .into_iter()
        .filter_map(|segment| match segment {
            ResponseContentSegment::LocalImage(marker) => Some(marker),
            ResponseContentSegment::Text(_) => None,
        })
        .collect()
}

pub fn replace_local_image_markers(text: &str, placeholder: &str) -> String {
    let segments = split_response_content_segments(text);
    if !segments
        .iter()
        .any(|segment| matches!(segment, ResponseContentSegment::LocalImage(_)))
    {
        return text.to_string();
    }

    let mut replaced = String::new();
    for segment in segments {
        match segment {
            ResponseContentSegment::Text(value) => replaced.push_str(&value),
            ResponseContentSegment::LocalImage(_) => replaced.push_str(placeholder),
        }
    }
    replaced
}

fn find_next_local_image_reference(
    text: &str,
    mut cursor: usize,
) -> Option<(usize, usize, LocalImageMarker)> {
    while cursor < text.len() {
        let relative_start = text[cursor..].find("file:///")?;
        let uri_start = cursor + relative_start;

        if let Some(found) = html_anchor_local_image_at(text, uri_start)
            .or_else(|| markdown_link_local_image_at(text, uri_start))
            .or_else(|| bare_local_image_at(text, uri_start))
        {
            return Some(found);
        }

        cursor = uri_start + "file:///".len();
    }

    None
}

fn bare_local_image_at(text: &str, start: usize) -> Option<(usize, usize, LocalImageMarker)> {
    let (end, marker) = local_image_marker_at(text, start)?;
    Some((start, end, marker))
}

fn html_anchor_local_image_at(
    text: &str,
    uri_start: usize,
) -> Option<(usize, usize, LocalImageMarker)> {
    let open_start = text[..uri_start].rfind("<a ")?;
    let open_end = open_start + text[open_start..].find('>')? + 1;
    if uri_start >= open_end {
        return None;
    }

    let open_tag = &text[open_start..open_end];
    let (href_marker, quote) = if open_tag.contains("href=\"file:///") {
        ("href=\"", '"')
    } else if open_tag.contains("href='file:///") {
        ("href='", '\'')
    } else {
        return None;
    };

    let href_value_start = open_start + open_tag.find(href_marker)? + href_marker.len();
    if href_value_start != uri_start {
        return None;
    }

    let href_value_end = href_value_start + text[href_value_start..].find(quote)?;
    let uri = &text[href_value_start..href_value_end];
    let path = parse_local_image_uri(uri)?;
    let end = text[open_end..]
        .find("</a>")
        .map(|relative| open_end + relative + "</a>".len())
        .unwrap_or(open_end);

    Some((
        open_start,
        end,
        LocalImageMarker {
            uri: uri.to_string(),
            path,
        },
    ))
}

fn markdown_link_local_image_at(
    text: &str,
    uri_start: usize,
) -> Option<(usize, usize, LocalImageMarker)> {
    let open_paren = uri_start.checked_sub(1)?;
    if text.as_bytes().get(open_paren).copied() != Some(b'(') {
        return None;
    }

    let label_end = open_paren.checked_sub(1)?;
    if text.as_bytes().get(label_end).copied() != Some(b']') {
        return None;
    }

    let open_bracket = text[..label_end].rfind('[')?;
    let start = if open_bracket > 0 && text.as_bytes().get(open_bracket - 1).copied() == Some(b'!')
    {
        open_bracket - 1
    } else {
        open_bracket
    };

    let close_paren = uri_start + text[uri_start..].find(')')?;
    let uri = &text[uri_start..close_paren];
    let path = parse_local_image_uri(uri)?;

    Some((
        start,
        close_paren + 1,
        LocalImageMarker {
            uri: uri.to_string(),
            path,
        },
    ))
}

#[async_trait]
pub trait OutboundAdapter: Clone + Send + Sync + 'static {
    type Placeholder: Clone + Send + Sync + 'static;

    async fn send_placeholder(&self, text: &str) -> Option<Self::Placeholder>;

    async fn update_progress(&self, placeholder: Option<&Self::Placeholder>, text: &str);

    async fn send_response(&self, placeholder: Option<&Self::Placeholder>, text: &str) -> usize;

    async fn send_error(&self, placeholder: Option<&Self::Placeholder>, text: &str);

    fn reasoning_visibility(&self) -> ReasoningVisibility {
        ReasoningVisibility::Full
    }
}

pub struct OutboundRunSummary {
    pub result: AgentSessionResult,
    pub placeholder_sent: bool,
    pub sent_segments: usize,
}

#[derive(Clone, Default)]
pub struct StreamActivityProbe {
    saw_stream_delta: Arc<AtomicBool>,
}

impl StreamActivityProbe {
    pub fn saw_stream_delta(&self) -> bool {
        self.saw_stream_delta.load(Ordering::Relaxed)
    }
}

struct StreamActivityListener {
    probe: StreamActivityProbe,
}

#[async_trait]
impl AgentSessionListener for StreamActivityListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        if matches!(event, AgentSessionEvent::Run(RunEvent::StreamDelta { .. })) {
            self.probe.saw_stream_delta.store(true, Ordering::Relaxed);
        }
    }
}

struct OutboundReasoningListener<A: OutboundAdapter> {
    adapter: A,
    placeholder: Arc<Mutex<Option<A::Placeholder>>>,
    progress: Arc<Mutex<ProgressTranscript>>,
}

#[derive(Clone)]
struct ProgressTranscript {
    base_text: String,
    entries: Vec<String>,
}

impl ProgressTranscript {
    fn new(base_text: &str) -> Self {
        Self {
            base_text: base_text.trim().to_string(),
            entries: Vec::new(),
        }
    }

    fn push(&mut self, entry: &str, dedupe: bool) -> Option<String> {
        let normalized = entry.trim();
        if normalized.is_empty() {
            return None;
        }
        if dedupe && self.entries.iter().any(|existing| existing == normalized) {
            return None;
        }
        self.entries.push(normalized.to_string());
        Some(self.render())
    }

    fn render(&self) -> String {
        let mut lines = Vec::new();
        if !self.base_text.is_empty() {
            lines.push(self.base_text.clone());
        }
        lines.extend(self.entries.iter().map(|entry| format!("- {entry}")));
        lines.join("\n")
    }
}

#[async_trait]
impl<A: OutboundAdapter> AgentSessionListener for OutboundReasoningListener<A> {
    async fn on_event(&self, event: AgentSessionEvent) {
        let AgentSessionEvent::Run(RunEvent::ToolStatus {
            tool,
            status,
            reasoning,
            ..
        }) = event
        else {
            return;
        };
        if status != "start" {
            return;
        }
        let visibility = self.adapter.reasoning_visibility();
        let text = match visibility {
            ReasoningVisibility::Hidden => None,
            ReasoningVisibility::Full => reasoning.filter(|value| !value.trim().is_empty()),
            ReasoningVisibility::Compact => Some(render_compact_tool_status_start(
                &tool,
                reasoning.as_deref(),
            )),
        };
        let Some(text) = text else {
            return;
        };
        let dedupe = !matches!(visibility, ReasoningVisibility::Compact);
        let Some(content) = self.progress.lock().await.push(&text, dedupe) else {
            return;
        };
        let placeholder = self.placeholder.lock().await.clone();
        self.adapter
            .update_progress(placeholder.as_ref(), &content)
            .await;
    }
}

pub async fn run_session_with_outbound<A: OutboundAdapter>(
    session: &mut AgentSession,
    adapter: A,
    input: &str,
    placeholder_text: &str,
    run_options: AgentRunOptions,
) -> OutboundRunSummary {
    let placeholder = adapter.send_placeholder(placeholder_text).await;
    let placeholder_sent = placeholder.is_some();
    let placeholder_ref = Arc::new(Mutex::new(placeholder));
    let progress_ref = Arc::new(Mutex::new(ProgressTranscript::new(placeholder_text)));
    session.add_listener(Arc::new(OutboundReasoningListener {
        adapter: adapter.clone(),
        placeholder: placeholder_ref.clone(),
        progress: progress_ref,
    }));

    let result = session.run(input, run_options).await;
    let response = &result.response;
    let placeholder = placeholder_ref.lock().await.clone();

    let sent_segments = if response.success {
        let content = if response.content.trim().is_empty() {
            "收到。".to_string()
        } else {
            response.content.trim().to_string()
        };
        adapter.send_response(placeholder.as_ref(), &content).await
    } else {
        adapter
            .send_error(
                placeholder.as_ref(),
                &user_visible_error_message(response.error.as_deref()),
            )
            .await;
        0
    };

    OutboundRunSummary {
        result,
        placeholder_sent,
        sent_segments,
    }
}

pub fn render_compact_tool_status_start(tool: &str, reasoning: Option<&str>) -> String {
    format!("正在{}...", compact_tool_subject(tool, reasoning))
}

pub fn render_compact_tool_status_done(tool: &str, reasoning: Option<&str>) -> String {
    format!("{}完成", compact_tool_subject(tool, reasoning))
}

fn compact_tool_subject(tool: &str, reasoning: Option<&str>) -> String {
    compact_tool_subject_candidate(tool)
        .or_else(|| reasoning.and_then(compact_tool_subject_candidate_from_reasoning))
        .unwrap_or_else(|| "调用工具".to_string())
}

fn compact_tool_subject_candidate_from_reasoning(reasoning: &str) -> Option<String> {
    let trimmed = reasoning.trim();
    let candidate = trimmed
        .strip_prefix("正在执行：")
        .or_else(|| trimmed.strip_prefix("执行完成："))
        .unwrap_or(trimmed)
        .split('；')
        .next()
        .unwrap_or(trimmed)
        .trim();
    compact_tool_subject_candidate(candidate)
}

fn compact_tool_subject_candidate(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let base = trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .trim_matches(|ch: char| matches!(ch, ':' | ';' | ','));

    if let Some(label) = compact_tool_subject_from_name(base) {
        return Some(label.to_string());
    }

    if looks_like_command(trimmed) {
        return Some("执行命令".to_string());
    }

    if is_generic_tool_name(base) {
        return None;
    }

    if is_safe_tool_identifier(base) {
        return Some(format!("调用工具 {base}"));
    }

    None
}

fn compact_tool_subject_from_name(name: &str) -> Option<&'static str> {
    match name {
        "deep_research" => Some("执行深度研究"),
        "local_search_files" | "local_find_files" => Some("查找本地文件"),
        "local_read_text_file" | "read_file" | "view" => Some("读取本地文件"),
        "local_write_text_file" | "write_file" | "replace_file" | "edit_file" => {
            Some("修改本地文件")
        }
        "local_list_directory" => Some("浏览本地目录"),
        "shell" | "execute" | "command" => Some("执行命令"),
        _ => tool_display_map()
            .get(name)
            .map(|(display_name, _)| *display_name),
    }
}

fn is_generic_tool_name(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "" | "tool" | "tool_call" | "toolcall"
    )
}

fn looks_like_command(text: &str) -> bool {
    let first = text.split_whitespace().next().unwrap_or_default();
    matches!(
        first,
        "rtk"
            | "bash"
            | "sh"
            | "zsh"
            | "/bin/bash"
            | "/bin/sh"
            | "/bin/zsh"
            | "python"
            | "python3"
            | "cargo"
            | "git"
            | "ls"
            | "cat"
            | "sed"
            | "rg"
            | "find"
            | "cp"
            | "mv"
            | "rm"
            | "bun"
            | "node"
    ) || first.contains('/')
        || first.contains('\\')
}

fn is_safe_tool_identifier(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 32
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

pub fn attach_stream_activity_probe(session: &mut AgentSession) -> StreamActivityProbe {
    let probe = StreamActivityProbe::default();
    session.add_listener(Arc::new(StreamActivityListener {
        probe: probe.clone(),
    }));
    probe
}

pub fn split_segments(text: &str, max_segment_size: usize, hard_max: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![];
    }

    let target_size = max_segment_size.clamp(100, hard_max.max(100));
    let mut segments = Vec::new();
    let mut buf = text.to_string();

    loop {
        let (remaining, flushed) = flush_buffer(buf, target_size);
        segments.extend(flushed);
        buf = remaining;
        if buf.len() < target_size {
            break;
        }
    }

    let tail = buf.trim().to_string();
    if !tail.is_empty() {
        segments.push(tail);
    }

    if segments.is_empty() {
        segments.push(text.trim().to_string());
    }

    segments
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HtmlOpenTag {
    name: String,
    opening_raw: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MarkdownFence {
    marker: char,
    marker_len: usize,
    opening_line: String,
}

impl MarkdownFence {
    fn closing_line(&self) -> String {
        std::iter::repeat_n(self.marker, self.marker_len).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HtmlTagKind {
    Open,
    Close,
    SelfClosing,
}

pub fn split_html_segments(text: &str, max_segment_size: usize, hard_max: usize) -> Vec<String> {
    rebalance_html_segments(split_segments(text, max_segment_size, hard_max))
}

pub fn split_markdown_segments(
    text: &str,
    max_segment_size: usize,
    hard_max: usize,
) -> Vec<String> {
    rebalance_markdown_segments(split_segments(text, max_segment_size, hard_max))
}

/// 平台差异化的消息分段适配器。
///
/// 目的：把「每个渠道各自的硬上限常量」从散落调用点集中到一个类型上，避免
/// 同一个常量（比如 Discord 的 1900 字符）在多个文件里各自复制。
///
/// 使用方式：每个 bin 只定义一个 zero-size struct 并 `impl PlatformMessageSplitter`，
/// 然后调用 `.split_markdown(...)` / `.split_html(...)` 即可，内部自动带上
/// 平台硬上限参数。`max_segment_size` 仍由调用方按当前消息渲染策略传入。
///
/// 例子：
/// ```ignore
/// pub(crate) struct DiscordSplitter;
/// impl hone_channels::outbound::PlatformMessageSplitter for DiscordSplitter {
///     fn hard_max_chars(&self) -> usize { 1900 }
/// }
/// let segments = DiscordSplitter.split_markdown(text, max_len);
/// ```
pub trait PlatformMessageSplitter {
    /// 单条消息的硬上限（比如 Discord 原生 2000，减去一点 buffer 即 1900）。
    fn hard_max_chars(&self) -> usize;

    /// 按 markdown 语义分段：调用方给 soft 上限,硬上限由实现提供。
    fn split_markdown(&self, text: &str, max_segment_size: usize) -> Vec<String> {
        split_markdown_segments(text, max_segment_size, self.hard_max_chars())
    }

    /// 按 HTML 语义分段：调用方给 soft 上限,硬上限由实现提供。
    /// 默认按 markdown 处理的 bin 可以不 override。
    fn split_html(&self, text: &str, max_segment_size: usize) -> Vec<String> {
        split_html_segments(text, max_segment_size, self.hard_max_chars())
    }
}

fn local_image_marker_at(text: &str, start: usize) -> Option<(usize, LocalImageMarker)> {
    let mut raw_end = start;
    for (offset, ch) in text[start..].char_indices() {
        if ch.is_whitespace() || ch == '<' {
            break;
        }
        raw_end = start + offset + ch.len_utf8();
    }
    if raw_end <= start {
        return None;
    }

    let mut uri_end = raw_end;
    while uri_end > start {
        let candidate = &text[start..uri_end];
        if let Some(path) = parse_local_image_uri(candidate) {
            return Some((
                uri_end,
                LocalImageMarker {
                    uri: candidate.to_string(),
                    path,
                },
            ));
        }

        let last = candidate.chars().last()?;
        if !matches!(last, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}') {
            break;
        }
        uri_end -= last.len_utf8();
    }

    None
}

fn parse_local_image_uri(candidate: &str) -> Option<String> {
    let path = candidate.strip_prefix("file://")?;
    if !path.starts_with('/') {
        return None;
    }

    let extension = path.rsplit_once('.')?.1.to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
    ) {
        return None;
    }

    Some(path.to_string())
}

fn rebalance_html_segments(raw_segments: Vec<String>) -> Vec<String> {
    let mut stack = Vec::<HtmlOpenTag>::new();
    let mut segments = Vec::new();

    for raw in raw_segments {
        let prefix = reopen_html_tags(&stack);
        let mut next_stack = stack.clone();
        scan_html_tags(&raw, &mut next_stack);

        let mut segment = String::new();
        segment.push_str(&prefix);
        segment.push_str(&raw);
        segment.push_str(&close_html_tags(&next_stack));
        segments.push(segment);
        stack = next_stack;
    }

    segments
}

fn rebalance_markdown_segments(raw_segments: Vec<String>) -> Vec<String> {
    let mut open_fence: Option<MarkdownFence> = None;
    let mut segments = Vec::new();

    for raw in raw_segments {
        let prefix = open_fence
            .as_ref()
            .map(|fence| format!("{}\n", fence.opening_line))
            .unwrap_or_default();

        let mut next_fence = open_fence.clone();
        scan_markdown_fences(&raw, &mut next_fence);

        let mut segment = String::new();
        segment.push_str(&prefix);
        segment.push_str(&raw);
        if let Some(fence) = &next_fence {
            if !segment.ends_with('\n') {
                segment.push('\n');
            }
            segment.push_str(&fence.closing_line());
        }
        segments.push(segment);
        open_fence = next_fence;
    }

    segments
}

fn reopen_html_tags(stack: &[HtmlOpenTag]) -> String {
    stack
        .iter()
        .map(|tag| tag.opening_raw.as_str())
        .collect::<String>()
}

fn close_html_tags(stack: &[HtmlOpenTag]) -> String {
    stack
        .iter()
        .rev()
        .map(|tag| format!("</{}>", tag.name))
        .collect::<String>()
}

fn scan_html_tags(segment: &str, stack: &mut Vec<HtmlOpenTag>) {
    let mut cursor = 0usize;
    while cursor < segment.len() {
        let remainder = &segment[cursor..];
        if let Some((len, kind, name, raw)) = parse_html_tag(remainder) {
            match kind {
                HtmlTagKind::Open => stack.push(HtmlOpenTag {
                    name,
                    opening_raw: raw,
                }),
                HtmlTagKind::Close => {
                    if let Some(pos) = stack.iter().rposition(|tag| tag.name == name) {
                        stack.truncate(pos);
                    }
                }
                HtmlTagKind::SelfClosing => {}
            }
            cursor += len;
            continue;
        }

        if let Some(len) = parse_html_entity_len(remainder) {
            cursor += len;
            continue;
        }

        let char_len = remainder
            .chars()
            .next()
            .map(|ch| ch.len_utf8())
            .unwrap_or(1);
        cursor += char_len;
    }
}

fn parse_html_tag(input: &str) -> Option<(usize, HtmlTagKind, String, String)> {
    if !input.starts_with('<') {
        return None;
    }
    let end = input.find('>')?;
    let raw = &input[..=end];
    let inner = raw[1..raw.len() - 1].trim();
    if inner.is_empty() || inner.starts_with('!') || inner.starts_with('?') {
        return None;
    }

    if let Some(rest) = inner.strip_prefix('/') {
        let name = parse_html_tag_name(rest)?;
        return Some((raw.len(), HtmlTagKind::Close, name, raw.to_string()));
    }

    let name = parse_html_tag_name(inner)?;
    let kind = if inner.ends_with('/') {
        HtmlTagKind::SelfClosing
    } else {
        HtmlTagKind::Open
    };
    Some((raw.len(), kind, name, raw.to_string()))
}

fn parse_html_tag_name(input: &str) -> Option<String> {
    let mut name = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            name.push(ch.to_ascii_lowercase());
        } else {
            break;
        }
    }
    if name.is_empty() { None } else { Some(name) }
}

fn parse_html_entity_len(input: &str) -> Option<usize> {
    const ENTITIES: &[&str] = &["&lt;", "&gt;", "&amp;", "&quot;", "&#39;"];
    ENTITIES
        .iter()
        .find_map(|entity| input.starts_with(entity).then_some(entity.len()))
}

fn scan_markdown_fences(segment: &str, open_fence: &mut Option<MarkdownFence>) {
    for line in segment.split_inclusive('\n') {
        let line_no_newline = line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(fence) = parse_markdown_fence(line_no_newline) {
            match open_fence {
                Some(current)
                    if current.marker == fence.marker
                        && fence.marker_len >= current.marker_len
                        && line_no_newline
                            .trim_start()
                            .trim_start_matches(fence.marker)
                            .trim()
                            .is_empty() =>
                {
                    *open_fence = None;
                }
                None => {
                    *open_fence = Some(fence);
                }
                _ => {}
            }
        }
    }
}

fn parse_markdown_fence(line: &str) -> Option<MarkdownFence> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }

    let marker_len = trimmed.chars().take_while(|ch| *ch == marker).count();
    if marker_len < 3 {
        return None;
    }

    Some(MarkdownFence {
        marker,
        marker_len,
        opening_line: trimmed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        LOCAL_IMAGE_CONTEXT_PLACEHOLDER, ProgressTranscript, ResponseContentSegment,
        collect_local_image_markers, render_compact_tool_status_done,
        render_compact_tool_status_start, replace_local_image_markers, scan_html_tags,
        scan_markdown_fences, split_html_segments, split_markdown_segments,
        split_response_content_segments,
    };
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct LocalImageMarkerFixture {
        name: String,
        input: String,
        part_types: Vec<String>,
        uris: Vec<String>,
        paths: Vec<String>,
    }

    #[test]
    fn progress_transcript_appends_entries_to_placeholder() {
        let mut transcript = ProgressTranscript::new("@alice 正在思考中...");
        assert_eq!(
            transcript.push("正在搜索公告", true),
            Some("@alice 正在思考中...\n- 正在搜索公告".to_string())
        );
        assert_eq!(
            transcript.push("正在读取财报", true),
            Some("@alice 正在思考中...\n- 正在搜索公告\n- 正在读取财报".to_string())
        );
    }

    #[test]
    fn progress_transcript_skips_duplicate_entries() {
        let mut transcript = ProgressTranscript::new("正在思考中...");
        assert!(transcript.push("正在搜索公告", true).is_some());
        assert_eq!(transcript.push("正在搜索公告", true), None);
    }

    #[test]
    fn progress_transcript_compact_mode_keeps_repeated_entries() {
        let mut transcript = ProgressTranscript::new("正在思考中...");
        assert!(transcript.push("正在搜索信息...", false).is_some());
        assert_eq!(
            transcript.push("正在搜索信息...", false),
            Some("正在思考中...\n- 正在搜索信息...\n- 正在搜索信息...".to_string())
        );
    }

    #[test]
    fn compact_tool_status_hides_query_details() {
        assert_eq!(
            render_compact_tool_status_start(
                "web_search query=\"Tempus AI stock surge today\"",
                None
            ),
            "正在搜索信息..."
        );
        assert_eq!(
            render_compact_tool_status_done(
                "web_search query=\"Tempus AI stock surge today\"",
                None
            ),
            "搜索信息完成"
        );
    }

    #[test]
    fn compact_tool_status_hides_command_and_paths() {
        assert_eq!(
            render_compact_tool_status_start("rtk sed -n '1,20p' /tmp/foo/bar.txt", None),
            "正在执行命令..."
        );
        assert_eq!(
            render_compact_tool_status_done("/bin/bash -lc rtk rg company_profiles /tmp/foo", None),
            "执行命令完成"
        );
    }

    #[test]
    fn compact_tool_status_uses_reasoning_when_tool_name_is_generic() {
        assert_eq!(
            render_compact_tool_status_start(
                "Tool",
                Some("正在执行：web_search query=\"Tempus AI stock surge today\"")
            ),
            "正在搜索信息..."
        );
        assert_eq!(
            render_compact_tool_status_done(
                "tool",
                Some("正在执行：rtk sed -n '1,20p' /tmp/foo/bar.txt")
            ),
            "执行命令完成"
        );
    }

    #[test]
    fn split_html_segments_rebalances_open_tags_across_segments() {
        let text = "<b>结论</b>\n<pre>第一行内容比较长，用来逼近分段阈值。\n第二行内容比较长，用来逼近分段阈值。\n第三行内容比较长，用来逼近分段阈值。\n第四行内容比较长，用来逼近分段阈值。</pre>\n尾部总结";
        let segments = split_html_segments(text, 24, 24);

        assert!(segments.len() > 1);
        assert!(segments.iter().any(|segment| segment.contains("</pre>")));

        for segment in &segments {
            let mut stack = Vec::new();
            scan_html_tags(segment, &mut stack);
            assert!(
                stack.is_empty(),
                "segment html tags should be balanced: {segment}"
            );
        }
    }

    #[test]
    fn split_markdown_segments_rebalances_code_fences_across_segments() {
        let text = "```rust\nfn main() {\n    println!(\"hello from a long code block segment one\");\n    println!(\"hello from a long code block segment two\");\n    println!(\"hello from a long code block segment three\");\n}\n```\n\n后续总结";
        let segments = split_markdown_segments(text, 32, 32);

        assert!(segments.len() > 1);
        assert!(segments[0].ends_with("\n```"));
        assert!(
            segments
                .iter()
                .skip(1)
                .any(|segment| segment.starts_with("```rust\n"))
        );

        for segment in &segments {
            let mut open_fence = None;
            scan_markdown_fences(segment, &mut open_fence);
            assert!(
                open_fence.is_none(),
                "segment markdown fences should be balanced: {segment}"
            );
        }
    }

    #[test]
    fn response_content_segments_preserve_text_image_text_order() {
        let text = "先看趋势：\nfile:///tmp/chart.png\n再看风险。";
        let segments = split_response_content_segments(text);

        assert_eq!(segments.len(), 3);
        assert!(matches!(
            &segments[0],
            ResponseContentSegment::Text(value) if value == "先看趋势：\n"
        ));
        assert!(matches!(
            &segments[1],
            ResponseContentSegment::LocalImage(marker)
                if marker.uri == "file:///tmp/chart.png" && marker.path == "/tmp/chart.png"
        ));
        assert!(matches!(
            &segments[2],
            ResponseContentSegment::Text(value) if value == "\n再看风险。"
        ));
    }

    #[test]
    fn collect_local_image_markers_ignores_trailing_punctuation() {
        let text = "图一 file:///tmp/chart-one.png, 图二(file:///tmp/chart-two.jpeg).";
        let markers = collect_local_image_markers(text);

        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].uri, "file:///tmp/chart-one.png");
        assert_eq!(markers[1].uri, "file:///tmp/chart-two.jpeg");
    }

    #[test]
    fn local_image_marker_contract_matches_shared_fixture() {
        let fixtures: Vec<LocalImageMarkerFixture> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/local_image_markers.json"
        ))
        .expect("local image marker fixture");

        for fixture in fixtures {
            let segments = split_response_content_segments(&fixture.input);
            let part_types = segments
                .iter()
                .map(|segment| match segment {
                    ResponseContentSegment::Text(_) => "text",
                    ResponseContentSegment::LocalImage(_) => "image",
                })
                .collect::<Vec<_>>();
            assert_eq!(part_types, fixture.part_types, "{}", fixture.name);

            let markers = collect_local_image_markers(&fixture.input);
            assert_eq!(
                markers
                    .iter()
                    .map(|marker| marker.uri.as_str())
                    .collect::<Vec<_>>(),
                fixture.uris,
                "{}",
                fixture.name
            );
            assert_eq!(
                markers
                    .iter()
                    .map(|marker| marker.path.as_str())
                    .collect::<Vec<_>>(),
                fixture.paths,
                "{}",
                fixture.name
            );
        }
    }

    #[test]
    fn response_content_segments_extract_html_anchor_local_images() {
        let text = "前文<a href=\"file:///tmp/chart.png\">file:///tmp/chart.png</a>后文";
        let segments = split_response_content_segments(text);

        assert_eq!(segments.len(), 3);
        assert!(matches!(
            &segments[0],
            ResponseContentSegment::Text(value) if value == "前文"
        ));
        assert!(matches!(
            &segments[1],
            ResponseContentSegment::LocalImage(marker)
                if marker.uri == "file:///tmp/chart.png" && marker.path == "/tmp/chart.png"
        ));
        assert!(matches!(
            &segments[2],
            ResponseContentSegment::Text(value) if value == "后文"
        ));
    }

    #[test]
    fn response_content_segments_extract_markdown_local_images() {
        let text = "前文[图表](file:///tmp/chart.png)后文";
        let segments = split_response_content_segments(text);

        assert_eq!(segments.len(), 3);
        assert!(matches!(
            &segments[0],
            ResponseContentSegment::Text(value) if value == "前文"
        ));
        assert!(matches!(
            &segments[1],
            ResponseContentSegment::LocalImage(marker)
                if marker.uri == "file:///tmp/chart.png" && marker.path == "/tmp/chart.png"
        ));
        assert!(matches!(
            &segments[2],
            ResponseContentSegment::Text(value) if value == "后文"
        ));
    }

    #[test]
    fn response_content_segments_extract_bare_local_images_before_html_tags() {
        let text = "前文file:///tmp/chart.png<br>后文";
        let segments = split_response_content_segments(text);

        assert_eq!(segments.len(), 3);
        assert!(matches!(
            &segments[0],
            ResponseContentSegment::Text(value) if value == "前文"
        ));
        assert!(matches!(
            &segments[1],
            ResponseContentSegment::LocalImage(marker)
                if marker.uri == "file:///tmp/chart.png" && marker.path == "/tmp/chart.png"
        ));
        assert!(matches!(
            &segments[2],
            ResponseContentSegment::Text(value) if value == "<br>后文"
        ));
    }

    #[test]
    fn response_content_segments_extract_real_telegram_retry_sample() {
        let text = "@chetzhang file:///Users/bytedance/Codes/honeclaw/data/gen_images/Session_telegram__group__chat_3a-1002012381143/rklb_three_case_valuation_retry.png-a825b14f.png<br>当前价位 82.93 美元已经高于 base case 的 74.4 美元，但距离最激进 bull case 的 118.2 美元仍有一段距离。";
        let segments = split_response_content_segments(text);

        assert_eq!(segments.len(), 3);
        assert!(matches!(
            &segments[0],
            ResponseContentSegment::Text(value) if value == "@chetzhang "
        ));
        assert!(matches!(
            &segments[1],
            ResponseContentSegment::LocalImage(marker)
                if marker.uri == "file:///Users/bytedance/Codes/honeclaw/data/gen_images/Session_telegram__group__chat_3a-1002012381143/rklb_three_case_valuation_retry.png-a825b14f.png"
                    && marker.path == "/Users/bytedance/Codes/honeclaw/data/gen_images/Session_telegram__group__chat_3a-1002012381143/rklb_three_case_valuation_retry.png-a825b14f.png"
        ));
        assert!(matches!(
            &segments[2],
            ResponseContentSegment::Text(value)
                if value == "<br>当前价位 82.93 美元已经高于 base case 的 74.4 美元，但距离最激进 bull case 的 118.2 美元仍有一段距离。"
        ));
    }

    #[test]
    fn replace_local_image_markers_preserves_surrounding_text() {
        let text = "前文<a href=\"file:///tmp/chart.png\">查看图片</a>后文";

        let replaced = replace_local_image_markers(text, LOCAL_IMAGE_CONTEXT_PLACEHOLDER);

        assert_eq!(replaced, "前文（上文包含图表）后文");
    }
}
