use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use hone_channels::agent_session::{AgentSessionEvent, AgentSessionListener};

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

    pub(crate) fn push(&mut self, entry: &str) -> Option<String> {
        let normalized = entry.trim();
        if normalized.is_empty() {
            return None;
        }
        let bullet_line = format!("- {normalized}");
        if self
            .base_text
            .lines()
            .any(|line| line.trim() == bullet_line.as_str())
        {
            return None;
        }
        if self.entries.iter().any(|existing| existing == normalized) {
            return None;
        }
        self.entries.push(normalized.to_string());
        Some(self.render())
    }
}

pub(crate) struct FeishuStreamListener {
    pub(crate) buffer: Arc<RwLock<String>>,
    pub(crate) cardkit: Option<Arc<CardKitSession>>,
    pub(crate) show_reasoning: bool,
}

#[async_trait]
impl AgentSessionListener for FeishuStreamListener {
    async fn on_event(&self, event: AgentSessionEvent) {
        match event {
            AgentSessionEvent::StreamDelta { content } => {
                self.buffer.write().unwrap().push_str(&content);
                if let Some(ck) = &self.cardkit {
                    let text = self.buffer.read().unwrap().clone();
                    let processed = preprocess_markdown_for_feishu(&text, false);
                    ck.update(&processed).await;
                }
            }
            AgentSessionEvent::ToolStatus {
                status,
                tool,
                message,
                reasoning,
            } => {
                if !self.show_reasoning {
                    return;
                }
                if status == "start" {
                    if let Some(text) = reasoning.filter(|m| !m.trim().is_empty()) {
                        let snapshot = {
                            let mut buf = self.buffer.write().unwrap();
                            let mut transcript = FeishuProgressTranscript::new(&buf);
                            let Some(next) = transcript.push(&text) else {
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
                    let text = match message.filter(|m| !m.trim().is_empty()) {
                        Some(msg) => msg,
                        None => format!("调用 {} 工具完成", tool),
                    };
                    let snapshot = {
                        let mut buf = self.buffer.write().unwrap();
                        let mut transcript = FeishuProgressTranscript::new(&buf);
                        let Some(next) = transcript.push(&text) else {
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

    #[test]
    fn feishu_progress_transcript_appends_entries() {
        let mut transcript = FeishuProgressTranscript::new("<at id=\"ou_1\"></at> 正在思考中...");
        assert_eq!(
            transcript.push("正在搜索公告"),
            Some("<at id=\"ou_1\"></at> 正在思考中...\n- 正在搜索公告".to_string())
        );
        assert_eq!(
            transcript.push("正在读取财报"),
            Some("<at id=\"ou_1\"></at> 正在思考中...\n- 正在搜索公告\n- 正在读取财报".to_string())
        );
    }

    #[test]
    fn feishu_progress_transcript_skips_duplicate_entries() {
        let mut transcript = FeishuProgressTranscript::new("正在思考中...");
        assert!(transcript.push("正在搜索公告").is_some());
        assert_eq!(transcript.push("正在搜索公告"), None);
    }
}
