use hone_channels::runtime::{DEFAULT_MAX_SEGMENT_SIZE, flush_buffer};
use serde_json::{Value, json};

use super::types::RenderedMessage;

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
    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn convert_table_to_feishu(lines: &[&str]) -> String {
    if lines.len() < 2 {
        return lines.join("\n");
    }
    let headers = parse_table_row(lines[0]);
    let data_lines = if lines.len() > 2 {
        &lines[2..]
    } else {
        &lines[0..0]
    };

    let columns: Vec<Value> = headers
        .iter()
        .enumerate()
        .map(|(idx, title)| json!({"title": title, "dataIndex": format!("col{}", idx)}))
        .collect();

    let data: Vec<Value> = data_lines
        .iter()
        .map(|row_line| {
            let cells = parse_table_row(row_line);
            let mut obj = serde_json::Map::new();
            for (idx, cell) in cells.iter().enumerate() {
                if idx < headers.len() {
                    obj.insert(format!("col{}", idx), Value::String(cell.clone()));
                }
            }
            Value::Object(obj)
        })
        .collect();

    let columns_json = serde_json::to_string(&columns).unwrap_or_default();
    let data_json = serde_json::to_string(&data).unwrap_or_default();
    format!(
        "<table columns={{{}}} data={{{}}}/>",
        columns_json, data_json
    )
}

pub(crate) fn preprocess_markdown_for_feishu(text: &str, convert_tables: bool) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut output = String::with_capacity(text.len() + 64);
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(heading_text) = extract_deep_heading(line) {
            output.push_str("**");
            output.push_str(heading_text.trim());
            output.push_str("**");
            output.push('\n');
            i += 1;
            continue;
        }

        if convert_tables
            && is_table_header_line(line)
            && i + 1 < lines.len()
            && is_table_separator_line(lines[i + 1])
        {
            let mut table_lines = vec![line];
            let mut j = i + 1;
            while j < lines.len() && lines[j].contains('|') {
                table_lines.push(lines[j]);
                j += 1;
            }
            if table_lines.len() >= 2 {
                output.push_str(&convert_table_to_feishu(&table_lines));
                output.push('\n');
            } else {
                output.push_str(line);
                output.push('\n');
            }
            i = j;
            continue;
        }

        output.push_str(line);
        if i + 1 < lines.len() || text.ends_with('\n') {
            output.push('\n');
        }
        i += 1;
    }

    if !text.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }
    output
}

pub(crate) fn split_into_segments(text: &str, max_segment_size: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![];
    }

    let target_size = max_segment_size.clamp(100, 3500);
    let mut segments = Vec::new();
    let mut buf = text.to_string();

    loop {
        let (remaining, flushed) = flush_buffer(buf, target_size.min(DEFAULT_MAX_SEGMENT_SIZE * 9));
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

pub(crate) fn render_outbound_messages(
    markdown: &str,
    max_message_length: usize,
) -> Vec<RenderedMessage> {
    let segments = split_into_segments(markdown, max_message_length);

    segments
        .into_iter()
        .map(|segment| {
            let processed = preprocess_markdown_for_feishu(&segment, true);
            let content = json!({
                "schema": "2.0",
                "config": {"wide_screen_mode": true},
                "body": {
                    "elements": [
                        {
                            "tag": "markdown",
                            "content": processed,
                            "text_size": "heading"
                        }
                    ]
                }
            })
            .to_string();
            RenderedMessage {
                msg_type: "interactive",
                content,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_h3_heading_becomes_bold() {
        let input = "### 三级标题\n正文";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(output.contains("**三级标题**"), "h3 应转为加粗: {output}");
        assert!(output.contains("正文"));
        assert!(!output.contains("###"), "不应保留 ### 语法");
    }

    #[test]
    fn preprocess_h4_heading_becomes_bold() {
        let input = "#### 四级标题";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(output.contains("**四级标题**"));
        assert!(!output.contains("####"));
    }

    #[test]
    fn preprocess_h1_h2_remain_unchanged() {
        let input = "# 一级\n## 二级";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(output.contains("# 一级"));
        assert!(output.contains("## 二级"));
    }

    #[test]
    fn preprocess_table_converted_to_feishu_format() {
        let input = "| 名称 | 数量 |\n|------|------|\n| 苹果 | 10 |\n| 香蕉 | 5 |";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(output.contains("<table"), "表格应转为飞书格式: {output}");
        assert!(output.contains("columns="), "应包含 columns 属性");
        assert!(output.contains("data="), "应包含 data 属性");
        assert!(output.contains("名称"), "应保留表头文字");
        assert!(output.contains("苹果"), "应保留数据");
    }

    #[test]
    fn preprocess_table_skipped_when_convert_tables_false() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(!output.contains("<table"), "流式模式下不应转换表格");
        assert!(output.contains("|"));
    }

    #[test]
    fn preprocess_no_trailing_newline_added_if_absent() {
        let input = "hello";
        let output = preprocess_markdown_for_feishu(input, false);
        assert_eq!(output, "hello");
    }

    #[test]
    fn is_table_separator_line_basic() {
        assert!(is_table_separator_line("|---|---|"));
        assert!(is_table_separator_line("| :--- | ---: |"));
        assert!(!is_table_separator_line("| 普通行 | 数据 |"));
        assert!(!is_table_separator_line("正文"));
    }

    #[test]
    fn convert_table_to_feishu_minimal() {
        let lines = ["| 股票 | 涨幅 |", "|------|------|", "| AAPL | 1.5% |"];
        let result = convert_table_to_feishu(&lines);
        assert!(
            result.starts_with("<table"),
            "结果应以 <table 开头: {result}"
        );
        assert!(result.contains("\"title\":\"股票\""));
        assert!(result.contains("\"title\":\"涨幅\""));
        assert!(result.contains("\"col0\":\"AAPL\""));
        assert!(result.contains("\"col1\":\"1.5%\""));
    }
}
