use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::agent_session::{
    AgentRunOptions, AgentSession, AgentSessionEvent, AgentSessionListener, AgentSessionResult,
};
use crate::runtime::{flush_buffer, user_visible_error_message};

#[async_trait]
pub trait OutboundAdapter: Clone + Send + Sync + 'static {
    type Placeholder: Clone + Send + Sync + 'static;

    async fn send_placeholder(&self, text: &str) -> Option<Self::Placeholder>;

    async fn update_progress(&self, placeholder: Option<&Self::Placeholder>, text: &str);

    async fn send_response(&self, placeholder: Option<&Self::Placeholder>, text: &str) -> usize;

    async fn send_error(&self, placeholder: Option<&Self::Placeholder>, text: &str);

    fn show_reasoning(&self) -> bool {
        true
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
        if matches!(event, AgentSessionEvent::StreamDelta { .. }) {
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

    fn push(&mut self, entry: &str) -> Option<String> {
        let normalized = entry.trim();
        if normalized.is_empty() {
            return None;
        }
        if self.entries.iter().any(|existing| existing == normalized) {
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
        let AgentSessionEvent::ToolStatus {
            status, reasoning, ..
        } = event
        else {
            return;
        };
        if !self.adapter.show_reasoning() {
            return;
        }
        if status != "start" {
            return;
        }
        let Some(text) = reasoning.filter(|value| !value.trim().is_empty()) else {
            return;
        };
        let Some(content) = self.progress.lock().await.push(&text) else {
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
        ProgressTranscript, scan_html_tags, scan_markdown_fences, split_html_segments,
        split_markdown_segments,
    };

    #[test]
    fn progress_transcript_appends_entries_to_placeholder() {
        let mut transcript = ProgressTranscript::new("@alice 正在思考中...");
        assert_eq!(
            transcript.push("正在搜索公告"),
            Some("@alice 正在思考中...\n- 正在搜索公告".to_string())
        );
        assert_eq!(
            transcript.push("正在读取财报"),
            Some("@alice 正在思考中...\n- 正在搜索公告\n- 正在读取财报".to_string())
        );
    }

    #[test]
    fn progress_transcript_skips_duplicate_entries() {
        let mut transcript = ProgressTranscript::new("正在思考中...");
        assert!(transcript.push("正在搜索公告").is_some());
        assert_eq!(transcript.push("正在搜索公告"), None);
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
}
