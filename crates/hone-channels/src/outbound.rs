use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::agent_session::{
    AgentRunOptions, AgentSession, AgentSessionEvent, AgentSessionListener, AgentSessionResult,
};
use crate::runtime::flush_buffer;

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
        let err = response
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        adapter
            .send_error(
                placeholder.as_ref(),
                &format!("抱歉，处理失败：{}", truncate_chars(&err, 300)),
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

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect::<String>() + "..."
}

#[cfg(test)]
mod tests {
    use super::ProgressTranscript;

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
}
