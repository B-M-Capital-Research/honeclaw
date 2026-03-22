fn sanitize_telegram_html(text: &str) -> String {
    let normalized = normalize_markdownish_telegram_text(text);
    let normalized = normalize_telegram_line_breaks(&normalized);
    let mut out = String::with_capacity(text.len());
    let mut idx = 0usize;

    while idx < normalized.len() {
        let rest = &normalized[idx..];
        if rest.starts_with('<') {
            if let Some((tag, consumed)) = parse_allowed_telegram_tag(rest) {
                out.push_str(tag);
                idx += consumed;
                continue;
            }
            out.push_str("&lt;");
            idx += 1;
            continue;
        }

        if rest.starts_with('>') {
            out.push_str("&gt;");
            idx += 1;
            continue;
        }

        if rest.starts_with('&') {
            if let Some((entity, consumed)) = parse_html_entity(rest) {
                out.push_str(entity);
                idx += consumed;
            } else {
                out.push_str("&amp;");
                idx += 1;
            }
            continue;
        }

        let ch = rest.chars().next().unwrap_or_default();
        out.push(ch);
        idx += ch.len_utf8();
    }

    out
}

fn normalize_telegram_line_breaks(text: &str) -> String {
    text.replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
}

fn normalize_markdownish_telegram_text(text: &str) -> String {
    text.lines()
        .map(normalize_markdownish_telegram_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_markdownish_telegram_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return line.to_string();
    }

    if let Some(content) = trimmed
        .strip_prefix("### ")
        .or_else(|| trimmed.strip_prefix("## "))
        .or_else(|| trimmed.strip_prefix("# "))
    {
        return format!("<b>{}</b>", normalize_markdownish_inline(content.trim()));
    }

    if let Some(content) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
    {
        return format!("• {}", normalize_markdownish_inline(content.trim()));
    }

    if let Some(content) = trimmed.strip_prefix("> ") {
        return format!(
            "<blockquote>{}</blockquote>",
            normalize_markdownish_inline(content.trim())
        );
    }

    normalize_markdownish_inline(line)
}

fn normalize_markdownish_inline(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut idx = 0usize;

    while idx < chars.len() {
        if idx + 1 < chars.len() && chars[idx] == '*' && chars[idx + 1] == '*' {
            if let Some(end) = find_double_marker(&chars, idx + 2, '*') {
                let content: String = chars[idx + 2..end].iter().collect();
                out.push_str("<b>");
                out.push_str(&normalize_markdownish_inline(&content));
                out.push_str("</b>");
                idx = end + 2;
                continue;
            }
        }

        if idx + 1 < chars.len() && chars[idx] == '_' && chars[idx + 1] == '_' {
            if let Some(end) = find_double_marker(&chars, idx + 2, '_') {
                let content: String = chars[idx + 2..end].iter().collect();
                out.push_str("<b>");
                out.push_str(&normalize_markdownish_inline(&content));
                out.push_str("</b>");
                idx = end + 2;
                continue;
            }
        }

        if chars[idx] == '`' {
            if idx + 2 < chars.len() && chars[idx + 1] == '`' && chars[idx + 2] == '`' {
                if let Some(end) = find_triple_backtick(&chars, idx + 3) {
                    let content: String = chars[idx + 3..end].iter().collect();
                    out.push_str("<pre><code>");
                    out.push_str(&content);
                    out.push_str("</code></pre>");
                    idx = end + 3;
                    continue;
                }
            } else if let Some(end) = find_single_char(&chars, idx + 1, '`') {
                let content: String = chars[idx + 1..end].iter().collect();
                out.push_str("<code>");
                out.push_str(&content);
                out.push_str("</code>");
                idx = end + 1;
                continue;
            }
        }

        out.push(chars[idx]);
        idx += 1;
    }

    out
}

fn find_double_marker(chars: &[char], start: usize, marker: char) -> Option<usize> {
    let mut idx = start;
    while idx + 1 < chars.len() {
        if chars[idx] == marker && chars[idx + 1] == marker {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

fn find_triple_backtick(chars: &[char], start: usize) -> Option<usize> {
    let mut idx = start;
    while idx + 2 < chars.len() {
        if chars[idx] == '`' && chars[idx + 1] == '`' && chars[idx + 2] == '`' {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

fn find_single_char(chars: &[char], start: usize, target: char) -> Option<usize> {
    let mut idx = start;
    while idx < chars.len() {
        if chars[idx] == target {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

fn parse_allowed_telegram_tag(input: &str) -> Option<(&str, usize)> {
    let end = input.find('>')?;
    let raw = &input[..=end];

    const EXACT_TAGS: &[&str] = &[
        "<b>",
        "</b>",
        "<strong>",
        "</strong>",
        "<i>",
        "</i>",
        "<em>",
        "</em>",
        "<u>",
        "</u>",
        "<ins>",
        "</ins>",
        "<s>",
        "</s>",
        "<strike>",
        "</strike>",
        "<del>",
        "</del>",
        "<code>",
        "</code>",
        "<pre>",
        "</pre>",
        "<blockquote>",
        "</blockquote>",
        "<blockquote expandable>",
        "<tg-spoiler>",
        "</tg-spoiler>",
        "<span class=\"tg-spoiler\">",
        "</span>",
        "</a>",
    ];

    if EXACT_TAGS.iter().any(|tag| raw == *tag) {
        return Some((raw, raw.len()));
    }

    if raw.starts_with("<a href=\"") && raw.ends_with("\">") && !raw[9..raw.len() - 2].contains('<')
    {
        return Some((raw, raw.len()));
    }

    None
}

fn parse_html_entity(input: &str) -> Option<(&str, usize)> {
    const ENTITIES: &[&str] = &["&lt;", "&gt;", "&amp;", "&quot;", "&#39;"];
    for entity in ENTITIES {
        if input.starts_with(entity) {
            return Some((entity, entity.len()));
        }
    }
    None
}

pub(crate) fn truncate_telegram_progress(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>()
        + "..."
}

pub(crate) fn sanitize_telegram_html_public(text: &str) -> String {
    sanitize_telegram_html(text)
}

pub(crate) fn prepend_reply_prefix_placeholder(prefix: Option<&str>, text: &str) -> String {
    let Some(prefix) = prefix.map(str::trim).filter(|value| !value.is_empty()) else {
        return text.to_string();
    };

    let body = text.trim_start();
    if body.starts_with(prefix) {
        text.to_string()
    } else if body.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {body}")
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_markdownish_telegram_text, sanitize_telegram_html};

    #[test]
    fn sanitize_telegram_html_keeps_supported_tags() {
        let input = "<b>结论</b> <a href=\"tg://user?id=1\">Alice</a>";
        assert_eq!(sanitize_telegram_html(input), input);
    }

    #[test]
    fn sanitize_telegram_html_escapes_comparison_operators() {
        let input = "毛利 < 40%，增速 > 20%，利润率 & 现金流";
        assert_eq!(
            sanitize_telegram_html(input),
            "毛利 &lt; 40%，增速 &gt; 20%，利润率 &amp; 现金流"
        );
    }

    #[test]
    fn sanitize_telegram_html_escapes_unknown_tags() {
        let input = "<unknown>tag</unknown>";
        assert_eq!(
            sanitize_telegram_html(input),
            "&lt;unknown&gt;tag&lt;/unknown&gt;"
        );
    }

    #[test]
    fn normalize_markdownish_telegram_text_converts_common_markdown() {
        let input = "### 结论\n- **上涨**\n> `观察`";
        assert_eq!(
            normalize_markdownish_telegram_text(input),
            "<b>结论</b>\n• <b>上涨</b>\n<blockquote><code>观察</code></blockquote>"
        );
    }

    #[test]
    fn sanitize_telegram_html_converts_markdownish_output_to_html() {
        let input = "### 标题\n- **重点** 与 `代码`";
        assert_eq!(
            sanitize_telegram_html(input),
            "<b>标题</b>\n• <b>重点</b> 与 <code>代码</code>"
        );
    }

    #[test]
    fn sanitize_telegram_html_converts_br_tags_to_newlines() {
        let input = "第一段<br>第二段<br/>第三段<br />第四段";
        assert_eq!(
            sanitize_telegram_html(input),
            "第一段\n第二段\n第三段\n第四段"
        );
    }
}
