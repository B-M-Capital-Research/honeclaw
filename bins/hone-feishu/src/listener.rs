use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use hone_channels::agent_session::{AgentSessionEvent, AgentSessionListener};
use hone_channels::outbound::{
    ReasoningVisibility, render_compact_tool_status_done, render_compact_tool_status_start,
};
use hone_channels::think::{ThinkStreamFormatter, append_compacted};

use super::card::CardKitSession;
use super::markdown::preprocess_markdown_for_feishu;

#[derive(Clone)]
pub(crate) struct FeishuProgressTranscript {
    base_text: String,
    entries: Vec<String>,
}

impl FeishuProgressTranscript {
    pub(crate) fn new(base_text: &str) -> Self {
        Self {
            base_text: base_text.trim().to_string(),
            entries: Vec::new(),
        }
    }

    fn render(&self) -> String {
        let mut lines = Vec::new();
        if !self.base_text.is_empty() {
            lines.push(self.base_text.clone());
        }
        lines.extend(self.entries.iter().map(|entry| format!("- {entry}")));
        lines.join("\n")
    }

    pub(crate) fn push(&mut self, entry: &str, dedupe: bool) -> Option<String> {
        let normalized = entry.trim();
        if normalized.is_empty() {
            return None;
        }
        let bullet_line = format!("- {normalized}");
        if dedupe
            && self
                .base_text
                .lines()
                .any(|line| line.trim() == bullet_line.as_str())
        {
            return None;
        }
        if dedupe && self.entries.iter().any(|existing| existing == normalized) {
            return None;
        }
        self.entries.push(normalized.to_string());
        Some(self.render())
    }
}

pub(crate) struct FeishuStreamListener {
    pub(crate) buffer: Arc<RwLock<String>>,
    pub(crate) cardkit: Option<Arc<CardKitSession>>,
    pub(crate) reasoning_visibility: ReasoningVisibility,
    pub(crate) think_formatter: Arc<RwLock<ThinkStreamFormatter>>,
}

#[async_trait]
impl AgentSessionListener for FeishuStreamListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        match event {
            AgentSessionEvent::StreamDelta { content } => {
                let rendered = {
                    let mut formatter = self.think_formatter.write().unwrap();
                    formatter.push_chunk(&content)
                };
                if !rendered.is_empty() {
                    append_compacted(&mut self.buffer.write().unwrap(), &rendered);
                }
                if let Some(ck) = &self.cardkit {
                    let text = self.buffer.read().unwrap().clone();
                    let processed = preprocess_markdown_for_feishu(&text, false);
                    ck.update(&processed).await;
                }
            }
            AgentSessionEvent::Done { .. } => {
                let trailing = {
                    let mut formatter = self.think_formatter.write().unwrap();
                    formatter.finish()
                };
                if !trailing.is_empty() {
                    append_compacted(&mut self.buffer.write().unwrap(), &trailing);
                    if let Some(ck) = &self.cardkit {
                        let text = self.buffer.read().unwrap().clone();
                        let processed = preprocess_markdown_for_feishu(&text, false);
                        ck.update(&processed).await;
                    }
                }
            }
            AgentSessionEvent::ToolStatus {
                status,
                tool,
                message,
                reasoning,
            } => {
                if matches!(self.reasoning_visibility, ReasoningVisibility::Hidden) {
                    return;
                }
                if status == "start" {
                    let text = match self.reasoning_visibility {
                        ReasoningVisibility::Hidden => None,
                        ReasoningVisibility::Full => reasoning
                            .as_deref()
                            .filter(|m| !m.trim().is_empty())
                            .map(str::to_string),
                        ReasoningVisibility::Compact => Some(render_compact_tool_status_start(
                            &tool,
                            reasoning.as_deref(),
                        )),
                    };
                    if let Some(text) = text {
                        let dedupe =
                            !matches!(self.reasoning_visibility, ReasoningVisibility::Compact);
                        let snapshot = {
                            let mut buf = self.buffer.write().unwrap();
                            let mut transcript = FeishuProgressTranscript::new(&buf);
                            let Some(next) = transcript.push(&text, dedupe) else {
                                return;
                            };
                            *buf = next.clone();
                            next
                        };
                        if let Some(ck) = &self.cardkit {
                            let processed = preprocess_markdown_for_feishu(&snapshot, false);
                            ck.force_update(&processed).await;
                        }
                    }
                }
                if status == "done" {
                    let text = match self.reasoning_visibility {
                        ReasoningVisibility::Hidden => return,
                        ReasoningVisibility::Compact => {
                            render_compact_tool_status_done(&tool, reasoning.as_deref())
                        }
                        ReasoningVisibility::Full => match message.filter(|m| !m.trim().is_empty())
                        {
                            Some(msg) => msg,
                            None => format!("调用 {} 工具完成", tool),
                        },
                    };
                    let dedupe = !matches!(self.reasoning_visibility, ReasoningVisibility::Compact);
                    let snapshot = {
                        let mut buf = self.buffer.write().unwrap();
                        let mut transcript = FeishuProgressTranscript::new(&buf);
                        let Some(next) = transcript.push(&text, dedupe) else {
                            return;
                        };
                        *buf = next.clone();
                        next
                    };
                    if let Some(ck) = &self.cardkit {
                        let processed = preprocess_markdown_for_feishu(&snapshot, false);
                        ck.force_update(&processed).await;
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FeishuProgressTranscript;
    use hone_channels::think::{ThinkRenderStyle, render_think_blocks};

    #[test]
    fn feishu_progress_transcript_appends_entries() {
        let mut transcript = FeishuProgressTranscript::new("<at id=\"ou_1\"></at> 正在思考中...");
        assert_eq!(
            transcript.push("正在搜索公告", true),
            Some("<at id=\"ou_1\"></at> 正在思考中...\n- 正在搜索公告".to_string())
        );
        assert_eq!(
            transcript.push("正在读取财报", true),
            Some("<at id=\"ou_1\"></at> 正在思考中...\n- 正在搜索公告\n- 正在读取财报".to_string())
        );
    }

    #[test]
    fn feishu_progress_transcript_skips_duplicate_entries() {
        let mut transcript = FeishuProgressTranscript::new("正在思考中...");
        assert!(transcript.push("正在搜索公告", true).is_some());
        assert_eq!(transcript.push("正在搜索公告", true), None);
    }

    #[test]
    fn compact_mode_keeps_repeated_entries() {
        let mut transcript = FeishuProgressTranscript::new("正在思考中...");
        assert!(transcript.push("正在搜索信息...", false).is_some());
        assert_eq!(
            transcript.push("正在搜索信息...", false),
            Some("正在思考中...\n- 正在搜索信息...\n- 正在搜索信息...".to_string())
        );
    }

    #[test]
    fn feishu_think_blocks_render_as_markdown_quotes() {
        let rendered =
            render_think_blocks("<think>foo</think>\nbar", ThinkRenderStyle::MarkdownQuote);
        assert!(rendered.contains("> foo"));
        assert!(rendered.ends_with("bar"));
    }

    #[test]
    fn hidden_style_does_not_expose_think_text() {
        let rendered = render_think_blocks("<think>foo</think>\nbar", ThinkRenderStyle::Hidden);
        assert_eq!(rendered, "bar");
    }
}
