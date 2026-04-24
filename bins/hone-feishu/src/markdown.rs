use hone_channels::outbound::PlatformMessageSplitter;
use serde_json::{Value, json};

use super::types::RenderedMessage;

/// 飞书富文本卡片单段硬上限（内部经验值,低于平台 5K/字符限制留出 buffer）。
pub(crate) const FEISHU_HARD_MAX_CHARS: usize = 3500;

/// Feishu 分段适配器。
pub(crate) struct FeishuSplitter;

impl PlatformMessageSplitter for FeishuSplitter {
    fn hard_max_chars(&self) -> usize {
        FEISHU_HARD_MAX_CHARS
    }
}

fn render_feishu_table(columns: &[Value], data: &[Value]) -> String {
    let columns_json = serde_json::to_string(columns).unwrap_or_default();
    let data_json = serde_json::to_string(data).unwrap_or_default();
    format!(
        "<table columns={{{}}} data={{{}}}/>",
        columns_json, data_json
    )
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

    render_feishu_table(&columns, &data)
}

fn escape_feishu_table_fragment(fragment: &str) -> String {
    let escaped = fragment
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\r', "")
        .replace('\n', "\\n");
    format!("[表格片段已降级为文本] {escaped}")
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
        for (rel, ch) in tag[index..].char_indices() {
            let abs = index + rel;
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
                        close_index = None;
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        close_index = Some(abs);
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

fn canonicalize_table_columns(value: &Value) -> Option<Vec<Value>> {
    let columns = value.as_array()?;
    columns
        .iter()
        .map(|column| {
            let object = column.as_object()?;
            let title = object.get("title")?.as_str()?;
            let data_index = object.get("dataIndex")?.as_str()?;
            if title.trim().is_empty() || data_index.trim().is_empty() {
                return None;
            }
            Some(json!({
                "title": title,
                "dataIndex": data_index,
            }))
        })
        .collect()
}

fn canonicalize_table_data(value: &Value, columns: &[Value]) -> Option<Vec<Value>> {
    let rows = value.as_array()?;
    let keys: Vec<&str> = columns
        .iter()
        .map(|column| column.get("dataIndex")?.as_str())
        .collect::<Option<_>>()?;

    rows.iter()
        .map(|row| {
            let object = row.as_object()?;
            let mut normalized = serde_json::Map::new();
            for key in &keys {
                if let Some(cell) = object.get(*key) {
                    normalized.insert(
                        (*key).to_string(),
                        Value::String(json_cell_to_string(cell)?),
                    );
                }
            }
            Some(Value::Object(normalized))
        })
        .collect()
}

fn normalize_raw_feishu_table_tag(tag: &str) -> Option<String> {
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
    let columns = canonicalize_table_columns(&columns_value)?;
    let data = canonicalize_table_data(&data_value, &columns)?;
    Some(render_feishu_table(&columns, &data))
}

fn broken_table_fragment_end(text: &str, start: usize) -> usize {
    let mut candidates = vec![text.len()];
    if let Some(rel) = text[start..].find("\n\n") {
        candidates.push(start + rel);
    }
    if let Some(rel) = text[start..].find("\r\n\r\n") {
        candidates.push(start + rel);
    }
    candidates.into_iter().min().unwrap_or(text.len())
}

fn sanitize_raw_feishu_tables(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(rel) = text[cursor..].find("<table") {
        let start = cursor + rel;
        output.push_str(&text[cursor..start]);
        let remaining = &text[start..];

        if let Some(end_rel) = remaining.find("/>") {
            let end = start + end_rel + 2;
            let fragment = &text[start..end];
            if let Some(normalized) = normalize_raw_feishu_table_tag(fragment) {
                output.push_str(&normalized);
            } else {
                output.push_str(&escape_feishu_table_fragment(fragment));
            }
            cursor = end;
            continue;
        }

        let end = broken_table_fragment_end(text, start);
        let fragment = &text[start..end];
        output.push_str(&escape_feishu_table_fragment(fragment));
        cursor = end;
    }

    output.push_str(&text[cursor..]);
    output
}

pub(crate) fn preprocess_markdown_for_feishu(text: &str, convert_tables: bool) -> String {
    let sanitized = sanitize_raw_feishu_tables(text);
    let lines: Vec<&str> = sanitized.lines().collect();
    let mut output = String::with_capacity(sanitized.len() + 64);
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
        if i + 1 < lines.len() || sanitized.ends_with('\n') {
            output.push('\n');
        }
        i += 1;
    }

    if !sanitized.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }
    output
}

pub(crate) fn split_into_segments(text: &str, max_segment_size: usize) -> Vec<String> {
    // clamp 把调用方可能传入的 max_segment_size 收敛到安全区间,
    // 避免过小造成碎片过多或过大直接超过平台限制。
    FeishuSplitter.split_markdown(text, max_message_length_bound(max_segment_size))
}

fn max_message_length_bound(max_segment_size: usize) -> usize {
    max_segment_size.clamp(100, FEISHU_HARD_MAX_CHARS)
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

    #[test]
    fn preprocess_valid_raw_feishu_table_normalized() {
        let input = "<table columns={[{\"title\":\"股票\",\"dataIndex\":\"ticker\"}]} data={[{\"ticker\":\"AVGO\"}]}/>";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(
            output.starts_with("<table columns={["),
            "合法 raw table 应保留: {output}"
        );
        assert!(output.contains("\"title\":\"股票\""));
        assert!(output.contains("\"dataIndex\":\"ticker\""));
        assert!(output.contains("\"ticker\":\"AVGO\""));
    }

    #[test]
    fn preprocess_invalid_raw_feishu_table_is_downgraded() {
        let input = "<tablecolumns={[{\"datalndex\":\"col0\",\"title\":\"股票\"}]}\ndata={[{\"col0\":\"AVGO\"}]}";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(
            !output.contains("<table"),
            "损坏的 raw table 不应保留: {output}"
        );
        assert!(output.contains("[表格片段已降级为文本]"));
        assert!(output.contains("&lt;tablecolumns="));
    }

    #[test]
    fn preprocess_invalid_raw_table_is_sanitized_in_stream_mode() {
        let input = "开始\n<table columns={[{\"title\":\"股票\",\"datalndex\":\"col0\"}]}/>\n结束";
        let output = preprocess_markdown_for_feishu(input, false);
        assert!(
            !output.contains("<table"),
            "stream 模式也不应泄漏 raw table: {output}"
        );
        assert!(output.contains("开始"));
        assert!(output.contains("结束"));
    }

    #[test]
    fn preprocess_mixed_markdown_and_raw_tables_keep_order() {
        let input = "### 标题\n说明段落\n<table columns={[{\"title\":\"股票\",\"dataIndex\":\"col0\"}]} data={[{\"col0\":\"MSFT\"}]}/>\n| 名称 | 数量 |\n|---|---|\n| 苹果 | 10 |";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(output.starts_with("**标题**\n说明段落\n<table"));
        assert!(output.contains("\"col0\":\"MSFT\""));
        assert!(output.contains("\"title\":\"名称\""));
        assert!(output.contains("\"col0\":\"苹果\""));
    }

    #[test]
    fn preprocess_user_reported_broken_table_has_no_live_tag() {
        let input = "<tablecolumns={[{\"datalndex\":\"colo\",\"title\":\"股票\",{\"datalndex\":\"col1\",\"title\":\"名称\"},{\"datalndex\":\"col2\",\"title\":\"现价\"}(\"datalndex\":\"col3\"\"title\":\"距击球区上限\"}，{\"datalndex\":\"col4\",\"title\":\"距击球区下限\"},\n{\"datalndex\":\"col5\",\"title\":\"PE\"},\n{\"datalndex\":\"col6\",\"title\":\"MA50\"},\n\"datalndex\":\"col7\",\"title\":\"MA200\"}]}\ndata={[{\"col0\":\"AVGO\",\"col1\":\"博\n通\"\"co|2\":\"$369.25\"\"col3\":\"-26.5%\"\"col4\":\"+34.1%\"\"col5\":\"73x\"\"col6\":\"$329.15\"\"col7:\n\"$288.22\"},{\"col0\":\"MSFT\"\"col1\":\"微\n软\",\"col2\":\"$358.42\",\"col3\":\"-8.9%\",\"col4\":\"+7.5%\",\"col5\":\"22x\"\"col6\":\"$381.04\"\"col7\":\"$406.25\"),\"col0\":\"GO0GL\",\"col1\":\"谷\n歌\",\"col2\":\"$295.37\",\"col3\":\"-21.3%\",\"col4\":\"+17.1%\"\"col5\":\"27x\",\"col6\":\"$309.81\"\"col7\":\"$286.07\"},(\"col0\":\"META\"\"col1\":\"Meta\",\"col2\":\"$612.40\",\"col3\":\"-30.6%\",\"col4\":\"+64.2%\",\"col5\":\"26x\",\"col6\":\"$615.93\",\"col7\":\"$609.55\"},(\"col0\":\"AAPL\n\"col1\":\"苹\n果\",\"col2\":\"$252.78\",\"col3\":\"-24.1%\",\"col4\":\"+30.8%\"\"col5\":\"32x\",\"col6\":\"$255.32\"\"col7\":\"$226.84\"},{\"col0\":\"AMZN\"\"col1\":\"亚马逊\",\"col2\":\"$232.17\",\"col3\":\"-21.4%\",\"col4\":\"+48.3%\"\"col5\":\"31x\"\"col6\":\"$214.87\"\"col7\":\"$198.45\"},(\"col0\":\"NVDA\",\"col1\":\"英伟《达\",\"co|2\":\"$195.63\",\"col3\":\"-16.5%\",\"col4+30.4%\",\"col5\":\"40x\",\"col6\":\"$177.23\",\"col7\":";
        let output = preprocess_markdown_for_feishu(input, true);
        assert!(
            !output.contains("<table"),
            "用户样例不应留下 live table: {output}"
        );
        assert!(output.contains("[表格片段已降级为文本]"));
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
        for message in messages {
            assert!(
                !message.content.contains("<table"),
                "拆分后的消息不应泄漏 live table: {}",
                message.content
            );
        }
    }
}
