use crate::runtime::tool_display_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkRenderStyle {
    Hidden,
    MarkdownQuote,
    TelegramHtmlQuote,
    PlainText,
}

#[derive(Debug, Clone)]
pub struct ThinkStreamFormatter {
    style: ThinkRenderStyle,
    pending: String,
    block: FormatterBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatterBlock {
    Plain,
    Think,
    ToolCode,
    ToolCall,
    ToolResult,
    ToolUse,
}

impl ThinkStreamFormatter {
    pub fn new(style: ThinkRenderStyle) -> Self {
        Self {
            style,
            pending: String::new(),
            block: FormatterBlock::Plain,
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) -> String {
        const OPEN_TAG: &str = "<think>";
        const CLOSE_TAG: &str = "</think>";
        const TOOL_OPEN_TAG: &str = "<tool_code>";
        const TOOL_CLOSE_TAG: &str = "</tool_code>";
        const TOOL_CALL_OPEN_TAG: &str = "<tool_call>";
        const TOOL_CALL_CLOSE_TAG: &str = "</tool_call>";
        const TOOL_RESULT_OPEN_TAG: &str = "<tool_result>";
        const TOOL_RESULT_CLOSE_TAG: &str = "</tool_result>";
        const TOOL_USE_OPEN_TAG: &str = "<tool_use>";
        const TOOL_USE_CLOSE_TAG: &str = "</tool_use>";

        self.pending.push_str(chunk);
        let mut rendered = String::new();

        loop {
            match self.block {
                FormatterBlock::Plain => {
                    if let Some((start, next_block, tag_len)) = find_next_open_tag(
                        &self.pending,
                        &[
                            (OPEN_TAG, FormatterBlock::Think),
                            (TOOL_OPEN_TAG, FormatterBlock::ToolCode),
                            (TOOL_CALL_OPEN_TAG, FormatterBlock::ToolCall),
                            (TOOL_RESULT_OPEN_TAG, FormatterBlock::ToolResult),
                            (TOOL_USE_OPEN_TAG, FormatterBlock::ToolUse),
                        ],
                    ) {
                        rendered.push_str(&self.pending[..start]);
                        self.pending.drain(..start + tag_len);
                        self.block = next_block;
                        continue;
                    }

                    let keep = trailing_partial_prefix_len_many(
                        &self.pending,
                        &[
                            OPEN_TAG,
                            TOOL_OPEN_TAG,
                            TOOL_CALL_OPEN_TAG,
                            TOOL_RESULT_OPEN_TAG,
                            TOOL_USE_OPEN_TAG,
                        ],
                    );
                    let emit_len = self.pending.len().saturating_sub(keep);
                    if emit_len > 0 {
                        rendered.push_str(&self.pending[..emit_len]);
                        self.pending.drain(..emit_len);
                    }
                    break;
                }
                FormatterBlock::Think => {
                    let Some(end) = self.pending.find(CLOSE_TAG) else {
                        break;
                    };
                    let thought = self.pending[..end].to_string();
                    rendered.push_str(&render_think_block(&thought, self.style));
                    self.pending.drain(..end + CLOSE_TAG.len());
                    self.block = FormatterBlock::Plain;
                }
                FormatterBlock::ToolCode => {
                    let Some(end) = self.pending.find(TOOL_CLOSE_TAG) else {
                        break;
                    };
                    let tool_code = self.pending[..end].to_string();
                    rendered.push_str(&render_tool_block(&tool_code, self.style));
                    self.pending.drain(..end + TOOL_CLOSE_TAG.len());
                    self.block = FormatterBlock::Plain;
                }
                FormatterBlock::ToolCall => {
                    let Some(end) = self.pending.find(TOOL_CALL_CLOSE_TAG) else {
                        break;
                    };
                    self.pending.drain(..end + TOOL_CALL_CLOSE_TAG.len());
                    self.block = FormatterBlock::Plain;
                }
                FormatterBlock::ToolResult => {
                    let Some(end) = self.pending.find(TOOL_RESULT_CLOSE_TAG) else {
                        break;
                    };
                    self.pending.drain(..end + TOOL_RESULT_CLOSE_TAG.len());
                    self.block = FormatterBlock::Plain;
                }
                FormatterBlock::ToolUse => {
                    let Some(end) = self.pending.find(TOOL_USE_CLOSE_TAG) else {
                        break;
                    };
                    self.pending.drain(..end + TOOL_USE_CLOSE_TAG.len());
                    self.block = FormatterBlock::Plain;
                }
            }
        }

        rendered
    }

    pub fn finish(&mut self) -> String {
        if self.pending.is_empty() {
            return String::new();
        }

        match self.block {
            FormatterBlock::Think => {
                self.block = FormatterBlock::Plain;
                let thought = std::mem::take(&mut self.pending);
                render_think_block(&thought, self.style)
            }
            FormatterBlock::ToolCode => {
                self.block = FormatterBlock::Plain;
                let tool_code = std::mem::take(&mut self.pending);
                render_tool_block(&tool_code, self.style)
            }
            FormatterBlock::ToolCall | FormatterBlock::ToolResult | FormatterBlock::ToolUse => {
                self.block = FormatterBlock::Plain;
                self.pending.clear();
                String::new()
            }
            FormatterBlock::Plain => std::mem::take(&mut self.pending),
        }
    }
}

pub fn render_think_blocks(text: &str, style: ThinkRenderStyle) -> String {
    let mut formatter = ThinkStreamFormatter::new(style);
    let mut out = formatter.push_chunk(text);
    out.push_str(&formatter.finish());
    let compacted = compact_excess_blank_lines(&out);
    if style == ThinkRenderStyle::Hidden {
        compacted.trim().to_string()
    } else {
        compacted
    }
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

fn trailing_partial_prefix_len_many(text: &str, markers: &[&str]) -> usize {
    markers
        .iter()
        .map(|marker| trailing_partial_prefix_len(text, marker))
        .max()
        .unwrap_or(0)
}

fn find_next_open_tag(
    text: &str,
    tags: &[(&str, FormatterBlock)],
) -> Option<(usize, FormatterBlock, usize)> {
    tags.iter()
        .filter_map(|(tag, block)| text.find(tag).map(|idx| (idx, *block, tag.len())))
        .min_by_key(|(idx, _, _)| *idx)
}

fn render_think_block(text: &str, style: ThinkRenderStyle) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    match style {
        ThinkRenderStyle::Hidden => String::new(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolInvocation {
    name: String,
    parameters: Vec<(String, String)>,
}

fn render_tool_block(text: &str, style: ThinkRenderStyle) -> String {
    let tools = parse_tool_invocations(text);
    if tools.is_empty() {
        return String::new();
    }
    let lines = tools
        .iter()
        .map(|tool| render_tool_line(tool, style))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{lines}\n\n")
}

fn render_tool_line(tool: &ToolInvocation, style: ThinkRenderStyle) -> String {
    let label = friendly_tool_name(&tool.name);
    let params = summarize_tool_parameters(&tool.parameters);
    let body = if params.is_empty() {
        format!("调用工具：{label}")
    } else {
        format!("调用工具：{label}（{params}）")
    };
    match style {
        ThinkRenderStyle::Hidden => String::new(),
        ThinkRenderStyle::MarkdownQuote | ThinkRenderStyle::TelegramHtmlQuote => {
            format!("- {body}")
        }
        ThinkRenderStyle::PlainText => body,
    }
}

fn friendly_tool_name(name: &str) -> String {
    if name == "portfolio_view" {
        return "查询持仓".to_string();
    }
    if let Some((display_name, _)) = tool_display_map().get(name) {
        return (*display_name).to_string();
    }
    name.to_string()
}

fn summarize_tool_parameters(parameters: &[(String, String)]) -> String {
    let mut parts = parameters
        .iter()
        .take(2)
        .map(|(name, value)| format!("{name}={}", summarize_inline_value(value, 48)))
        .collect::<Vec<_>>();
    if parameters.len() > 2 {
        parts.push("...".to_string());
    }
    parts.join(", ")
}

fn summarize_inline_value(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = normalized.chars().count();
    if char_count <= max_chars {
        return normalized;
    }
    let keep = max_chars.saturating_sub(1);
    let truncated = normalized.chars().take(keep).collect::<String>();
    format!("{truncated}…")
}

fn parse_tool_invocations(text: &str) -> Vec<ToolInvocation> {
    let mut tools = Vec::new();
    let mut cursor = 0usize;

    while let Some(start_rel) = text[cursor..].find("<tool") {
        let start = cursor + start_rel;
        if text[start..].starts_with("<tool_code") {
            cursor = start + "<tool_code".len();
            continue;
        }
        let Some(open_end_rel) = text[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let open_tag = &text[start..=open_end];
        let name = extract_xml_attr(open_tag, "name").unwrap_or_else(|| "tool".to_string());

        if open_tag.trim_end().ends_with("/>") {
            tools.push(ToolInvocation {
                name,
                parameters: Vec::new(),
            });
            cursor = open_end + 1;
            continue;
        }

        let Some(close_rel) = text[open_end + 1..].find("</tool>") else {
            tools.push(ToolInvocation {
                name,
                parameters: parse_tool_parameters(&text[open_end + 1..]),
            });
            break;
        };
        let inner_end = open_end + 1 + close_rel;
        let inner = &text[open_end + 1..inner_end];
        tools.push(ToolInvocation {
            name,
            parameters: parse_tool_parameters(inner),
        });
        cursor = inner_end + "</tool>".len();
    }

    tools
}

fn parse_tool_parameters(text: &str) -> Vec<(String, String)> {
    let mut params = Vec::new();
    let mut cursor = 0usize;
    while let Some(start_rel) = text[cursor..].find("<parameter") {
        let start = cursor + start_rel;
        let Some(open_end_rel) = text[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let open_tag = &text[start..=open_end];
        let name = extract_xml_attr(open_tag, "name").unwrap_or_else(|| "arg".to_string());
        let Some(close_rel) = text[open_end + 1..].find("</parameter>") else {
            break;
        };
        let value_end = open_end + 1 + close_rel;
        let value = text[open_end + 1..value_end].trim();
        if !value.is_empty() {
            params.push((name, value.to_string()));
        }
        cursor = value_end + "</parameter>".len();
    }
    params
}

fn extract_xml_attr(tag: &str, attr: &str) -> Option<String> {
    let marker = format!(r#"{attr}=""#);
    let start = tag.find(&marker)? + marker.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
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
    fn hidden_style_strips_think_and_tool_blocks() {
        let rendered = render_think_blocks(
            "<think>foo</think>\n<tool_code><tool><parameter name=\"query\">AAPL</parameter></tool></tool_code>\nbar",
            ThinkRenderStyle::Hidden,
        );
        assert_eq!(rendered, "bar");
    }

    #[test]
    fn stream_formatter_suppresses_tool_call_blocks() {
        let mut formatter = ThinkStreamFormatter::new(ThinkRenderStyle::Hidden);
        let first = formatter.push_chunk("前文<tool_call>{\"name\":\"web_search\"}");
        let second = formatter.push_chunk("</tool_call>后文");
        let rendered = format!("{first}{second}{}", formatter.finish());
        assert_eq!(rendered, "前文后文");
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

    #[test]
    fn markdown_tool_code_becomes_single_line_entries() {
        let rendered = render_think_blocks(
            "<tool_code>\n<tool name=\"portfolio_view\">\n</tool>\n<tool name=\"web_search\">\n<parameter name=\"query\">US Iran negotiations April 13 2026 latest update Hormuz Strait ceasefire</parameter>\n</tool>\n</tool_code>",
            ThinkRenderStyle::MarkdownQuote,
        );
        assert!(rendered.contains("- 调用工具：查询持仓"));
        assert!(rendered.contains("- 调用工具：搜索信息（query=US Iran negotiations"));
        assert!(!rendered.contains("<tool"));
    }

    #[test]
    fn stream_formatter_handles_tool_code_blocks() {
        let mut formatter = ThinkStreamFormatter::new(ThinkRenderStyle::PlainText);
        assert_eq!(formatter.push_chunk("<tool_"), "");
        assert_eq!(
            formatter.push_chunk("code><tool name=\"portfolio_view\">"),
            ""
        );
        assert_eq!(
            formatter.push_chunk("</tool></tool_code>done"),
            "调用工具：查询持仓\n\ndone"
        );
        assert_eq!(formatter.finish(), "");
    }
}
