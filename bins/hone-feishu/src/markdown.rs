use hone_channels::outbound::PlatformMessageSplitter;
use serde_json::{Value, json};

use super::types::RenderedMessage;

/// 飞书富文本卡片单段硬上限（内部经验值，低于平台限制留出 buffer）。
pub(crate) const FEISHU_HARD_MAX_CHARS: usize = 3500;

const FEISHU_MAX_NATIVE_TABLES_PER_CARD: usize = 5;
const FEISHU_MAX_NATIVE_TABLE_COLUMNS: usize = 50;
const INVALID_TABLE_FALLBACK: &str = "表格内容解析失败，请稍后重试。";

/// Feishu 分段适配器。
pub(crate) struct FeishuSplitter;

impl PlatformMessageSplitter for FeishuSplitter {
    fn hard_max_chars(&self) -> usize {
        FEISHU_HARD_MAX_CHARS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableSource {
    Markdown,
    RawComponent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TableModel {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ContentBlock {
    Markdown(String),
    Table {
        model: TableModel,
        source: TableSource,
    },
}

fn extract_deep_heading(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("###") {
        return None;
    }
    let rest = trimmed.trim_start_matches('#');
    let num_hashes = trimmed.len() - rest.len();
    if num_hashes >= 3 && rest.starts_with(' ') {
        Some(&rest[1..])
    } else {
        None
    }
}

fn is_table_header_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
        && trimmed.contains('-')
}

fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in inner.chars() {
        if escaped {
            if ch != '|' && ch != '\\' {
                current.push('\\');
            }
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '|' => {
                cells.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if escaped {
        current.push('\\');
    }
    cells.push(current.trim().to_string());
    cells
}

fn parse_markdown_table(lines: &[&str]) -> Option<TableModel> {
    if lines.len() < 2 {
        return None;
    }
    let headers = parse_table_row(lines[0]);
    if headers.is_empty() {
        return None;
    }
    let rows = lines
        .iter()
        .skip(2)
        .map(|line| {
            let mut cells = parse_table_row(line);
            cells.resize(headers.len(), String::new());
            cells.truncate(headers.len());
            cells
        })
        .collect();
    Some(TableModel { headers, rows })
}

fn markdown_table_cell(cell: &str) -> String {
    cell.replace('\r', "")
        .replace('\n', " ")
        .replace('|', "\\|")
}

fn render_markdown_table(model: &TableModel) -> String {
    let mut lines = Vec::with_capacity(model.rows.len() + 2);
    lines.push(format!(
        "| {} |",
        model
            .headers
            .iter()
            .map(|cell| markdown_table_cell(cell))
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    lines.push(format!(
        "|{}|",
        std::iter::repeat_n("---", model.headers.len())
            .collect::<Vec<_>>()
            .join("|")
    ));
    for row in &model.rows {
        lines.push(format!(
            "| {} |",
            row.iter()
                .map(|cell| markdown_table_cell(cell))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    lines.join("\n")
}

fn fallback_cell(cell: &str) -> String {
    cell.replace(['\r', '\n'], " ").trim().to_string()
}

fn render_table_fallback(model: &TableModel) -> String {
    let headers = model
        .headers
        .iter()
        .map(|header| fallback_cell(header))
        .collect::<Vec<_>>();
    let mut lines = vec![format!("**{}**", headers.join(" · "))];
    if model.rows.is_empty() {
        lines.push("- 暂无数据".to_string());
        return lines.join("\n");
    }

    for row in &model.rows {
        let fields = headers
            .iter()
            .zip(row.iter())
            .map(|(header, value)| format!("**{header}**：{}", fallback_cell(value)))
            .collect::<Vec<_>>();
        lines.push(format!("- {}", fields.join("；")));
    }
    lines.join("\n")
}

fn find_attribute_json(tag: &str, attr: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    for (attr_pos, _) in tag.match_indices(attr) {
        if attr_pos > 0 {
            let prev = bytes[attr_pos - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                continue;
            }
        }

        let mut index = attr_pos + attr.len();
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index).copied() != Some(b'=') {
            continue;
        }
        index += 1;

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index).copied() != Some(b'{') {
            continue;
        }

        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;
        let mut close_index = None;
        for (relative, ch) in tag[index..].char_indices() {
            let absolute = index + relative;
            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                match ch {
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        close_index = Some(absolute);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(close_index) = close_index {
            return Some(tag[index + 1..close_index].to_string());
        }
    }
    None
}

fn json_cell_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Null => Some(String::new()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).ok(),
    }
}

fn parse_raw_table_component(tag: &str) -> Option<TableModel> {
    if !tag.starts_with("<table") {
        return None;
    }
    let mut suffix = tag["<table".len()..].chars();
    if !matches!(suffix.next(), Some(ch) if ch.is_whitespace()) {
        return None;
    }

    let columns_json = find_attribute_json(tag, "columns")?;
    let data_json = find_attribute_json(tag, "data")?;
    let columns_value: Value = serde_json::from_str(&columns_json).ok()?;
    let data_value: Value = serde_json::from_str(&data_json).ok()?;

    let column_defs = columns_value
        .as_array()?
        .iter()
        .map(|column| {
            let object = column.as_object()?;
            let title = object.get("title")?.as_str()?.trim();
            let data_index = object.get("dataIndex")?.as_str()?.trim();
            if title.is_empty() || data_index.is_empty() {
                return None;
            }
            Some((title.to_string(), data_index.to_string()))
        })
        .collect::<Option<Vec<_>>>()?;
    if column_defs.is_empty() {
        return None;
    }

    let rows = data_value
        .as_array()?
        .iter()
        .map(|row| {
            let object = row.as_object()?;
            column_defs
                .iter()
                .map(|(_, key)| match object.get(key) {
                    Some(value) => json_cell_to_string(value),
                    None => Some(String::new()),
                })
                .collect::<Option<Vec<_>>>()
        })
        .collect::<Option<Vec<_>>>()?;

    Some(TableModel {
        headers: column_defs.into_iter().map(|(title, _)| title).collect(),
        rows,
    })
}

fn broken_table_fragment_end(text: &str, start: usize) -> usize {
    let mut candidates = vec![text.len()];
    if let Some(relative) = text[start..].find("\n\n") {
        candidates.push(start + relative);
    }
    if let Some(relative) = text[start..].find("\r\n\r\n") {
        candidates.push(start + relative);
    }
    candidates.into_iter().min().unwrap_or(text.len())
}

fn raw_table_fragment_end(remaining: &str) -> Option<usize> {
    let self_closing = remaining.find("/>").map(|index| index + 2);
    let paired = remaining
        .find("</table>")
        .map(|index| index + "</table>".len());
    match (self_closing, paired) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(end), None) | (None, Some(end)) => Some(end),
        (None, None) => None,
    }
}

fn push_markdown_block(blocks: &mut Vec<ContentBlock>, text: impl Into<String>) {
    let text = text.into();
    if text.trim().is_empty() {
        return;
    }
    if let Some(ContentBlock::Markdown(existing)) = blocks.last_mut() {
        if !existing.ends_with('\n') && !text.starts_with('\n') {
            existing.push('\n');
        }
        existing.push_str(&text);
    } else {
        blocks.push(ContentBlock::Markdown(text));
    }
}

fn parse_markdown_blocks(text: &str, blocks: &mut Vec<ContentBlock>) {
    if text.is_empty() {
        return;
    }
    let lines = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r'))
        .collect::<Vec<_>>();
    let mut text_lines = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        if is_table_header_line(lines[index])
            && index + 1 < lines.len()
            && is_table_separator_line(lines[index + 1])
        {
            if !text_lines.is_empty() {
                push_markdown_block(blocks, text_lines.join("\n"));
                text_lines.clear();
            }
            let mut end = index + 2;
            while end < lines.len() && is_table_header_line(lines[end]) {
                end += 1;
            }
            if let Some(model) = parse_markdown_table(&lines[index..end]) {
                blocks.push(ContentBlock::Table {
                    model,
                    source: TableSource::Markdown,
                });
            } else {
                push_markdown_block(blocks, lines[index..end].join("\n"));
            }
            index = end;
            continue;
        }

        text_lines.push(lines[index]);
        index += 1;
    }

    if !text_lines.is_empty() {
        push_markdown_block(blocks, text_lines.join("\n"));
    }
}

fn parse_content_blocks(text: &str) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut cursor = 0usize;

    while let Some(relative) = text[cursor..].find("<table") {
        let start = cursor + relative;
        parse_markdown_blocks(&text[cursor..start], &mut blocks);
        let remaining = &text[start..];

        if let Some(end_relative) = raw_table_fragment_end(remaining) {
            let end = start + end_relative;
            let fragment = &text[start..end];
            if let Some(model) = parse_raw_table_component(fragment) {
                blocks.push(ContentBlock::Table {
                    model,
                    source: TableSource::RawComponent,
                });
            } else {
                push_markdown_block(&mut blocks, INVALID_TABLE_FALLBACK);
            }
            cursor = end;
            continue;
        }

        let end = broken_table_fragment_end(text, start);
        push_markdown_block(&mut blocks, INVALID_TABLE_FALLBACK);
        cursor = end;
    }

    parse_markdown_blocks(&text[cursor..], &mut blocks);
    blocks
}

fn preprocess_text_markdown(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = String::with_capacity(text.len() + 32);

    for (index, line) in lines.iter().enumerate() {
        if let Some(heading_text) = extract_deep_heading(line) {
            output.push_str("**");
            output.push_str(heading_text.trim());
            output.push_str("**");
        } else {
            output.push_str(line);
        }
        if index + 1 < lines.len() || text.ends_with('\n') {
            output.push('\n');
        }
    }

    if !text.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }
    output
}

fn append_rendered_block(output: &mut String, rendered: &str) {
    if rendered.trim().is_empty() {
        return;
    }
    if !output.is_empty() && !output.ends_with('\n') && !rendered.starts_with('\n') {
        output.push('\n');
    }
    output.push_str(rendered);
}

/// 只返回飞书 Markdown 元素可安全承载的文本。
///
/// `convert_tables=true` 用于 CardKit 等只能更新单个 Markdown 元素的旧路径，此时表格
/// 降级成可读列表；完整卡片发送应使用 `render_feishu_card_content` 生成原生表格元素。
pub(crate) fn preprocess_markdown_for_feishu(text: &str, convert_tables: bool) -> String {
    let mut output = String::with_capacity(text.len() + 64);
    for block in parse_content_blocks(text) {
        match block {
            ContentBlock::Markdown(markdown) => {
                append_rendered_block(&mut output, &preprocess_text_markdown(&markdown));
            }
            ContentBlock::Table { model, source } => {
                let rendered = if !convert_tables && source == TableSource::Markdown {
                    render_markdown_table(&model)
                } else {
                    render_table_fallback(&model)
                };
                append_rendered_block(&mut output, &rendered);
            }
        }
    }
    output
}

fn native_table_element(model: &TableModel) -> Option<Value> {
    if model.headers.is_empty() || model.headers.len() > FEISHU_MAX_NATIVE_TABLE_COLUMNS {
        return None;
    }

    let columns = model
        .headers
        .iter()
        .enumerate()
        .map(|(index, title)| {
            json!({
                "name": format!("col{index}"),
                "display_name": title,
                "data_type": "text",
                "width": "auto",
                "vertical_align": "top",
                "horizontal_align": "left"
            })
        })
        .collect::<Vec<_>>();
    let rows = model
        .rows
        .iter()
        .map(|row| {
            let mut object = serde_json::Map::new();
            for index in 0..model.headers.len() {
                object.insert(
                    format!("col{index}"),
                    Value::String(row.get(index).cloned().unwrap_or_default()),
                );
            }
            Value::Object(object)
        })
        .collect::<Vec<_>>();

    Some(json!({
        "tag": "table",
        "page_size": model.rows.len().clamp(1, 10),
        "row_height": "auto",
        "row_max_height": "160px",
        "freeze_first_column": model.headers.len() > 3,
        "header_style": {
            "text_align": "left",
            "text_size": "normal",
            "background_style": "grey",
            "text_color": "default",
            "bold": true,
            "lines": 2
        },
        "columns": columns,
        "rows": rows
    }))
}

fn append_markdown_buffer(buffer: &mut String, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    if !buffer.is_empty() && !buffer.ends_with('\n') && !text.starts_with('\n') {
        buffer.push('\n');
    }
    buffer.push_str(text);
}

fn flush_markdown_element(elements: &mut Vec<Value>, buffer: &mut String) {
    if buffer.trim().is_empty() {
        buffer.clear();
        return;
    }
    let content = preprocess_text_markdown(buffer);
    buffer.clear();
    if content.trim().is_empty() {
        return;
    }
    elements.push(json!({
        "tag": "markdown",
        "content": content,
        "text_size": "heading"
    }));
}

fn render_card_elements(blocks: &[ContentBlock]) -> Vec<Value> {
    let mut elements = Vec::new();
    let mut markdown_buffer = String::new();
    let mut native_table_count = 0usize;

    for block in blocks {
        match block {
            ContentBlock::Markdown(markdown) => {
                append_markdown_buffer(&mut markdown_buffer, markdown);
            }
            ContentBlock::Table { model, .. } => {
                if native_table_count < FEISHU_MAX_NATIVE_TABLES_PER_CARD
                    && let Some(table) = native_table_element(model)
                {
                    flush_markdown_element(&mut elements, &mut markdown_buffer);
                    elements.push(table);
                    native_table_count += 1;
                } else {
                    append_markdown_buffer(&mut markdown_buffer, &render_table_fallback(model));
                }
            }
        }
    }
    flush_markdown_element(&mut elements, &mut markdown_buffer);

    if elements.is_empty() {
        elements.push(json!({
            "tag": "markdown",
            "content": "收到。",
            "text_size": "heading"
        }));
    }
    elements
}

fn render_card_content_from_blocks(blocks: &[ContentBlock]) -> String {
    json!({
        "schema": "2.0",
        "config": {"wide_screen_mode": true},
        "body": {
            "elements": render_card_elements(blocks)
        }
    })
    .to_string()
}

pub(crate) fn render_feishu_card_content(markdown: &str) -> String {
    render_card_content_from_blocks(&parse_content_blocks(markdown))
}

pub(crate) fn split_into_segments(text: &str, max_segment_size: usize) -> Vec<String> {
    FeishuSplitter.split_markdown(text, max_message_length_bound(max_segment_size))
}

fn max_message_length_bound(max_segment_size: usize) -> usize {
    max_segment_size.clamp(100, FEISHU_HARD_MAX_CHARS)
}

fn estimated_block_chars(block: &ContentBlock) -> usize {
    match block {
        ContentBlock::Markdown(markdown) => markdown.chars().count(),
        ContentBlock::Table { model, .. } => native_table_element(model)
            .map(|table| table.to_string().chars().count())
            .unwrap_or_else(|| render_table_fallback(model).chars().count()),
    }
}

fn split_markdown_blocks_for_delivery(
    blocks: Vec<ContentBlock>,
    max_message_length: usize,
) -> Vec<ContentBlock> {
    let mut expanded = Vec::new();
    for block in blocks {
        match block {
            ContentBlock::Markdown(markdown) => {
                expanded.extend(
                    split_into_segments(&markdown, max_message_length)
                        .into_iter()
                        .filter(|segment| !segment.trim().is_empty())
                        .map(ContentBlock::Markdown),
                );
            }
            table => expanded.push(table),
        }
    }
    expanded
}

fn group_blocks_for_cards(
    blocks: Vec<ContentBlock>,
    max_message_length: usize,
) -> Vec<Vec<ContentBlock>> {
    let target = max_message_length_bound(max_message_length);
    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut current_chars = 0usize;
    let mut current_tables = 0usize;

    for block in split_markdown_blocks_for_delivery(blocks, target) {
        let block_chars = estimated_block_chars(&block);
        let is_table = matches!(block, ContentBlock::Table { .. });
        let exceeds_table_limit = is_table && current_tables >= FEISHU_MAX_NATIVE_TABLES_PER_CARD;
        let exceeds_target =
            !current.is_empty() && current_chars.saturating_add(block_chars) > target;
        if exceeds_table_limit || exceeds_target {
            groups.push(std::mem::take(&mut current));
            current_chars = 0;
            current_tables = 0;
        }

        current_chars = current_chars.saturating_add(block_chars);
        if is_table {
            current_tables += 1;
        }
        current.push(block);
    }

    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

pub(crate) fn render_outbound_messages(
    markdown: &str,
    max_message_length: usize,
) -> Vec<RenderedMessage> {
    group_blocks_for_cards(parse_content_blocks(markdown), max_message_length)
        .into_iter()
        .map(|blocks| RenderedMessage {
            msg_type: "interactive",
            content: render_card_content_from_blocks(&blocks),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_card(content: &str) -> Value {
        serde_json::from_str(content).expect("valid card json")
    }

    fn card_elements(card: &Value) -> &[Value] {
        card["body"]["elements"].as_array().expect("card elements")
    }

    #[test]
    fn preprocess_h3_heading_becomes_bold() {
        let output = preprocess_markdown_for_feishu("### 三级标题\n正文", false);
        assert!(output.contains("**三级标题**"), "h3 应转为加粗: {output}");
        assert!(output.contains("正文"));
        assert!(!output.contains("###"), "不应保留 ### 语法");
    }

    #[test]
    fn preprocess_h4_heading_becomes_bold() {
        let output = preprocess_markdown_for_feishu("#### 四级标题", false);
        assert!(output.contains("**四级标题**"));
        assert!(!output.contains("####"));
    }

    #[test]
    fn preprocess_h1_h2_remain_unchanged() {
        let output = preprocess_markdown_for_feishu("# 一级\n## 二级", false);
        assert!(output.contains("# 一级"));
        assert!(output.contains("## 二级"));
    }

    #[test]
    fn markdown_only_preprocessor_uses_readable_table_fallback() {
        let input = "| 名称 | 数量 |\n|------|------|\n| 苹果 | 10 |\n| 香蕉 | 5 |";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(!output.contains("<table"), "不得生成 raw tag: {output}");
        assert!(output.contains("**名称 · 数量**"));
        assert!(output.contains("**名称**：苹果"));
        assert!(output.contains("**数量**：10"));
    }

    #[test]
    fn stream_preprocessor_keeps_standard_markdown_table() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(!output.contains("<table"));
        assert_eq!(output, input);
    }

    #[test]
    fn preprocess_no_trailing_newline_added_if_absent() {
        assert_eq!(preprocess_markdown_for_feishu("hello", false), "hello");
    }

    #[test]
    fn is_table_separator_line_basic() {
        assert!(is_table_separator_line("|---|---|"));
        assert!(is_table_separator_line("| :--- | ---: |"));
        assert!(!is_table_separator_line("| 普通行 | 数据 |"));
        assert!(!is_table_separator_line("正文"));
    }

    #[test]
    fn markdown_table_becomes_native_card_element() {
        let input = "| 名称 | 数量 |\n|------|------|\n| 苹果 | 10 |\n| 香蕉 | 5 |";
        let content = render_feishu_card_content(input);
        let card = parse_card(&content);
        let elements = card_elements(&card);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0]["tag"], "table");
        assert_eq!(elements[0]["columns"][0]["name"], "col0");
        assert_eq!(elements[0]["columns"][0]["display_name"], "名称");
        assert_eq!(elements[0]["columns"][0]["data_type"], "text");
        assert_eq!(elements[0]["rows"][0]["col0"], "苹果");
        assert_eq!(elements[0]["rows"][0]["col1"], "10");
        assert!(!content.contains("<table"));
        assert!(!content.contains("dataIndex"));
    }

    #[test]
    fn escaped_pipe_stays_in_native_table_cell() {
        let content = render_feishu_card_content("| 逻辑 |\n|---|\n| A \\| B |");
        let card = parse_card(&content);
        assert_eq!(card_elements(&card)[0]["rows"][0]["col0"], "A | B");
    }

    #[test]
    fn valid_raw_table_is_migrated_to_native_element() {
        let input = "<table columns={[{\"title\":\"股票\",\"dataIndex\":\"ticker\"}]} data={[{\"ticker\":\"AVGO\"}]}/>";
        let content = render_feishu_card_content(input);
        let card = parse_card(&content);
        let table = &card_elements(&card)[0];
        assert_eq!(table["tag"], "table");
        assert_eq!(table["columns"][0]["display_name"], "股票");
        assert_eq!(table["rows"][0]["col0"], "AVGO");
        assert!(!content.contains("<table"));
        assert!(!content.contains("dataIndex"));
    }

    #[test]
    fn valid_raw_table_never_survives_markdown_only_path() {
        let input = "<table columns={[{\"title\":\"股票\",\"dataIndex\":\"ticker\"}]} data={[{\"ticker\":\"AVGO\"}]}/>";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(!output.contains("<table"));
        assert!(!output.contains("dataIndex"));
        assert!(output.contains("**股票**"));
        assert!(output.contains("AVGO"));
    }

    #[test]
    fn invalid_raw_table_is_replaced_without_source_code() {
        let input = "<tablecolumns={[{\"datalndex\":\"col0\",\"title\":\"股票\"}]}\ndata={[{\"col0\":\"AVGO\"}]}";
        let output = preprocess_markdown_for_feishu(input, true);
        assert_eq!(output, INVALID_TABLE_FALLBACK);
        assert!(!output.contains("<table"));
        assert!(!output.contains("columns="));
        assert!(!output.contains("datalndex"));
    }

    #[test]
    fn invalid_raw_table_is_sanitized_in_stream_mode() {
        let input = "开始\n<table columns={[{\"title\":\"股票\",\"datalndex\":\"col0\"}]}/>\n结束";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(!output.contains("<table"));
        assert!(output.contains("开始"));
        assert!(output.contains(INVALID_TABLE_FALLBACK));
        assert!(output.contains("结束"));
    }

    #[test]
    fn mixed_markdown_and_tables_keep_native_element_order() {
        let input = "### 标题\n说明段落\n<table columns={[{\"title\":\"股票\",\"dataIndex\":\"col0\"}]} data={[{\"col0\":\"MSFT\"}]}/>\n| 名称 | 数量 |\n|---|---|\n| 苹果 | 10 |";
        let card = parse_card(&render_feishu_card_content(input));
        let elements = card_elements(&card);
        assert_eq!(
            elements
                .iter()
                .map(|element| element["tag"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["markdown", "table", "table"]
        );
        assert!(
            elements[0]["content"]
                .as_str()
                .unwrap()
                .contains("**标题**")
        );
        assert_eq!(elements[1]["rows"][0]["col0"], "MSFT");
        assert_eq!(elements[2]["rows"][0]["col0"], "苹果");
    }

    #[test]
    fn user_reported_sections_render_as_two_native_tables() {
        let input = "四、机构评级与目标价\n| 标的 | 机构 | 评级 | 目标价 | 逻辑 |\n|---|---|---|---|---|\n| BE | 未核验 | — | — | — |\n| MU | BofA | Buy | $1,550 | AI memory 需求 |\n\n五、本周关键日历\n| 日期 | 标的 | 事件 |\n|---|---|---|\n| 7/28 盘后 | BE | 财报 |";
        let messages = render_outbound_messages(input, FEISHU_HARD_MAX_CHARS);
        assert_eq!(messages.len(), 1);
        let card = parse_card(&messages[0].content);
        let elements = card_elements(&card);
        assert_eq!(
            elements
                .iter()
                .map(|element| element["tag"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["markdown", "table", "markdown", "table"]
        );
        assert_eq!(elements[1]["rows"][1]["col3"], "$1,550");
        assert_eq!(elements[3]["rows"][0]["col2"], "财报");
        assert!(!messages[0].content.contains("<table columns="));
        assert!(!messages[0].content.contains("dataIndex"));
    }

    #[test]
    fn more_than_five_tables_split_across_cards() {
        let input = (0..6)
            .map(|index| format!("| 编号 |\n|---|\n| {index} |"))
            .collect::<Vec<_>>()
            .join("\n\n");
        let messages = render_outbound_messages(&input, FEISHU_HARD_MAX_CHARS);
        assert!(messages.len() >= 2);
        let mut table_count = 0usize;
        for message in messages {
            let card = parse_card(&message.content);
            let per_card = card_elements(&card)
                .iter()
                .filter(|element| element["tag"] == "table")
                .count();
            assert!(per_card <= FEISHU_MAX_NATIVE_TABLES_PER_CARD);
            table_count += per_card;
        }
        assert_eq!(table_count, 6);
    }

    #[test]
    fn render_outbound_messages_do_not_leak_split_invalid_tables() {
        let prefix = "前言段落\n".repeat(60);
        let input = format!(
            "{prefix}{}",
            "<table columns={[{\"title\":\"股票\",\"datalndex\":\"col0\"}]} data={[{\"col0\":\"AVGO\"}]}\n结尾"
        );
        let messages = render_outbound_messages(&input, 120);
        assert!(messages.len() > 1, "应拆成多段");
        let rendered = messages
            .iter()
            .map(|message| message.content.as_str())
            .collect::<String>();
        assert!(!rendered.contains("<table"));
        assert!(!rendered.contains("datalndex"));
        assert!(!rendered.contains("\"AVGO\""));
        assert!(rendered.contains(INVALID_TABLE_FALLBACK));
    }
}
