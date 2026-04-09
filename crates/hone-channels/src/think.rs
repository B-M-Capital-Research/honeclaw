#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkRenderStyle {
    MarkdownQuote,
    TelegramHtmlQuote,
    PlainText,
}

#[derive(Debug, Clone)]
pub struct ThinkStreamFormatter {
    style: ThinkRenderStyle,
    pending: String,
    inside_think: bool,
}

impl ThinkStreamFormatter {
    pub fn new(style: ThinkRenderStyle) -> Self {
        Self {
            style,
            pending: String::new(),
            inside_think: false,
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) -> String {
        const OPEN_TAG: &str = "<think>";
        const CLOSE_TAG: &str = "</think>";

        self.pending.push_str(chunk);
        let mut rendered = String::new();

        loop {
            if self.inside_think {
                let Some(end) = self.pending.find(CLOSE_TAG) else {
                    break;
                };
                let thought = self.pending[..end].to_string();
                rendered.push_str(&render_think_block(&thought, self.style));
                self.pending.drain(..end + CLOSE_TAG.len());
                self.inside_think = false;
                continue;
            }

            if let Some(start) = self.pending.find(OPEN_TAG) {
                rendered.push_str(&self.pending[..start]);
                self.pending.drain(..start + OPEN_TAG.len());
                self.inside_think = true;
                continue;
            }

            let keep = trailing_partial_prefix_len(&self.pending, OPEN_TAG);
            let emit_len = self.pending.len().saturating_sub(keep);
            if emit_len > 0 {
                rendered.push_str(&self.pending[..emit_len]);
                self.pending.drain(..emit_len);
            }
            break;
        }

        rendered
    }

    pub fn finish(&mut self) -> String {
        if self.pending.is_empty() {
            return String::new();
        }

        if self.inside_think {
            self.inside_think = false;
            let thought = std::mem::take(&mut self.pending);
            return render_think_block(&thought, self.style);
        }

        std::mem::take(&mut self.pending)
    }
}

pub fn render_think_blocks(text: &str, style: ThinkRenderStyle) -> String {
    let mut formatter = ThinkStreamFormatter::new(style);
    let mut out = formatter.push_chunk(text);
    out.push_str(&formatter.finish());
    compact_excess_blank_lines(&out)
}

pub fn append_compacted(buffer: &mut String, addition: &str) {
    if addition.is_empty() {
        return;
    }

    buffer.push_str(addition);
    let compacted = compact_excess_blank_lines(buffer);
    buffer.clear();
    buffer.push_str(&compacted);
}

fn compact_excess_blank_lines(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut consecutive_newlines = 0usize;

    for ch in text.chars() {
        if ch == '\r' {
            continue;
        }

        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                output.push(ch);
            }
            continue;
        }

        consecutive_newlines = 0;
        output.push(ch);
    }

    output
}

fn trailing_partial_prefix_len(text: &str, marker: &str) -> usize {
    let max_len = text.len().min(marker.len().saturating_sub(1));
    for len in (1..=max_len).rev() {
        if text.ends_with(&marker[..len]) {
            return len;
        }
    }
    0
}

fn render_think_block(text: &str, style: ThinkRenderStyle) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    match style {
        ThinkRenderStyle::MarkdownQuote => render_markdown_quote(trimmed),
        ThinkRenderStyle::TelegramHtmlQuote => render_telegram_quote(trimmed),
        ThinkRenderStyle::PlainText => render_plain_text(trimmed),
    }
}

fn render_markdown_quote(text: &str) -> String {
    let quoted = text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                ">".to_string()
            } else {
                format!("> {}", line.trim_end())
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{quoted}\n\n")
}

fn render_telegram_quote(text: &str) -> String {
    let open_tag = if text.lines().count() > 1 || text.chars().count() > 120 {
        "<blockquote expandable>"
    } else {
        "<blockquote>"
    };
    format!("{open_tag}{text}</blockquote>\n\n")
}

fn render_plain_text(text: &str) -> String {
    format!("思考：\n{text}\n\n")
}

#[cfg(test)]
mod tests {
    use super::{ThinkRenderStyle, ThinkStreamFormatter, append_compacted, render_think_blocks};

    #[test]
    fn markdown_think_becomes_quote_block() {
        let rendered = render_think_blocks(
            "<think>a\nb</think>\nhello",
            ThinkRenderStyle::MarkdownQuote,
        );
        assert!(rendered.contains("> a"));
        assert!(rendered.contains("> b"));
        assert!(rendered.ends_with("hello"));
        assert!(!rendered.contains("<think>"));
    }

    #[test]
    fn telegram_think_becomes_html_blockquote() {
        let rendered = render_think_blocks(
            "<think>hello</think>\nworld",
            ThinkRenderStyle::TelegramHtmlQuote,
        );
        assert!(rendered.contains("<blockquote>hello</blockquote>"));
        assert!(rendered.ends_with("world"));
    }

    #[test]
    fn stream_formatter_handles_split_open_tag() {
        let mut formatter = ThinkStreamFormatter::new(ThinkRenderStyle::PlainText);
        assert_eq!(formatter.push_chunk("<thi"), "");
        assert_eq!(formatter.push_chunk("nk>alpha"), "");
        assert_eq!(
            formatter.push_chunk("</think>beta"),
            "思考：\nalpha\n\nbeta"
        );
        assert_eq!(formatter.finish(), "");
    }

    #[test]
    fn rendered_output_compacts_large_blank_runs() {
        let rendered = render_think_blocks(
            "<think>hello</think>\n\n\nhi",
            ThinkRenderStyle::MarkdownQuote,
        );
        assert!(!rendered.contains("\n\n\n"));
    }

    #[test]
    fn append_compacted_collapses_boundary_blank_lines() {
        let mut buffer = String::from("> hello\n\n");
        append_compacted(&mut buffer, "\n\nhi");
        assert_eq!(buffer, "> hello\n\nhi");
    }
}
