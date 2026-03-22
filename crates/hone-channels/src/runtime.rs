//! 渠道运行时 — 流式处理
//!
//! 各渠道通用的流式消息处理和分段发送。

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// 默认停止字符（中英文句末标点 + 换行）
pub const DEFAULT_STOP_CHARS: &[char] = &['。', '！', '？', '\n', '.', '!', '?'];

/// 缓冲区最小长度
pub const DEFAULT_MIN_BUFFER_SIZE: usize = 100;

/// 单条消息最大长度（约手机一屏半，适合 iMessage 阅读）
pub const DEFAULT_MAX_SEGMENT_SIZE: usize = 400;

/// 流式处理结果
#[derive(Debug, Clone)]
pub struct StreamProcessResult {
    pub full_response: String,
    pub tool_calls: Vec<serde_json::Value>,
    pub tool_results: Vec<serde_json::Value>,
}

/// 发送回调类型
pub type StreamSendFn =
    Box<dyn Fn(String, String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// 工具显示名称映射
pub fn tool_display_map() -> HashMap<&'static str, (&'static str, bool)> {
    let mut map = HashMap::new();
    map.insert("load_skill", ("加载技能", false));
    map.insert("web_search", ("搜索信息", true));
    map.insert("data_fetch", ("获取数据", true));
    map.insert("portfolio", ("查询持仓", false));
    map.insert("cron_job", ("管理定时任务", false));
    map.insert("image_gen", ("生成图片", true));
    map
}

/// 获取工具状态消息
pub fn get_tool_status_message(tool_name: &str, status: &str) -> String {
    let map = tool_display_map();
    if let Some(&(display_name, should_show)) = map.get(tool_name) {
        if !should_show {
            return String::new();
        }
        match status {
            "start" => format!("正在{display_name}..."),
            "done" => format!("{display_name}完成"),
            _ => String::new(),
        }
    } else {
        String::new()
    }
}

/// 解析工具调用的 reasoning；缺失时回退到工程侧生成文案
pub fn resolve_tool_reasoning(tool_name: &str, reasoning: Option<String>) -> Option<String> {
    let cleaned = reasoning
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if cleaned.is_some() {
        return cleaned;
    }

    let map = tool_display_map();
    if let Some(&(display_name, _)) = map.get(tool_name) {
        return Some(format!("正在{display_name}..."));
    }

    Some(format!("正在调用 {tool_name}..."))
}

/// 清理消息中的特殊标记（工具调用残留、MSG 标记等）
pub fn clean_msg_markers(text: &str) -> String {
    use regex::Regex;

    let mut cleaned = text.to_string();

    // [MSG1], [MSG2] 等
    let msg_re = Regex::new(r"\[MSG\d+\]\s*").unwrap();
    cleaned = msg_re.replace_all(&cleaned, "").to_string();

    // <|...|> 标记
    let pipe_re = Regex::new(r"<\|[^|]+\|>").unwrap();
    cleaned = pipe_re.replace_all(&cleaned, "").to_string();

    // tool_name:N 标记
    let tool_re =
        Regex::new(r"\b(web_search|data_fetch|portfolio|load_skill|image_gen):\d+\s*").unwrap();
    cleaned = tool_re.replace_all(&cleaned, "").to_string();

    // JSON 工具参数
    let json_re = Regex::new(
        r#"\{[^\{\}]*"(query|action|data_type|skill_name|ticker|symbol|draft_id|approval_token|image_prompt|user_intent|image_count|regenerate_images|image_type|content|prompt)"[^\{\}]*\}"#
    ).unwrap();
    cleaned = json_re.replace_all(&cleaned, "").to_string();

    // 简单 JSON
    let simple_json_re = Regex::new(r#"\{"[^"]*":\s*"[^"]*"\}"#).unwrap();
    cleaned = simple_json_re.replace_all(&cleaned, "").to_string();

    // functions 残留
    let func_re = Regex::new(r"(functions?\.?\s*)+").unwrap();
    cleaned = func_re.replace_all(&cleaned, "").to_string();

    // tool_call/tool_result/tool_use 及其可能附带的尖括号
    let tool_keyword_re = Regex::new(
        r"</?(tool_call|tool_result|tool_use)\b[^>]*>|\b(tool_call|tool_result|tool_use)\b",
    )
    .unwrap();
    cleaned = tool_keyword_re.replace_all(&cleaned, "").to_string();

    // 多余空白（不包含换行）
    let ws_re = Regex::new(r"[ \t]+").unwrap();
    cleaned = ws_re.replace_all(&cleaned, " ").to_string();

    // 连续多个换行（可能夹杂空格）压缩为两个换行，保留段落结构
    let nl_re = Regex::new(r"\n[ \t\n]*\n").unwrap();
    cleaned = nl_re.replace_all(&cleaned, "\n\n").to_string();

    cleaned.trim().to_string()
}

/// 检测文本是否包含工具调用标记
pub fn is_tool_call_content(text: &str) -> bool {
    const MARKERS: &[&str] = &[
        "<|tool_call",
        "<|tool_calls_section",
        "tool_call_argument",
        r#"{"query""#,
        r#"{"action""#,
        r#"{"data_type""#,
        r#"{"skill_name""#,
        "_begin|>",
        "_end|>",
        "web_search:",
        "data_fetch:",
        "portfolio:",
        "load_skill:",
        "image_gen:",
        r#"{"image_type""#,
    ];
    MARKERS.iter().any(|marker| text.contains(marker))
}

/// 检测缓冲区内容是否应该跳过发送
pub fn should_skip_buffer(text: &str) -> bool {
    let cleaned = clean_msg_markers(text);
    if cleaned.len() < 10 {
        return true;
    }
    if is_tool_call_content(&cleaned) {
        return true;
    }
    // 无意义内容
    use regex::Regex;
    let patterns = [
        r"^[\s\.\,\!\?\:\;\-\_\=\+\*\/\\]+$",
        r"^(functions?\.?\s*)+$",
        r"^(tool_?call\.?\s*)+$",
        r"^[\s]*$",
    ];
    for pattern in &patterns {
        if Regex::new(pattern).unwrap().is_match(&cleaned) {
            return true;
        }
    }
    false
}

/// 在 target_pos 附近寻找自然断点
pub fn find_split_point(text: &str, target_pos: usize) -> usize {
    // Snap to the nearest valid UTF-8 char boundary (walk backward if needed)
    let mut search_end = target_pos.min(text.len());
    while search_end > 0 && !text.is_char_boundary(search_end) {
        search_end -= 1;
    }
    let search_text = &text[..search_end];

    // 优先级 1: --- 分隔线
    if let Some(pos) = search_text.rfind("---") {
        if pos > 0 {
            let mut end = pos + 3;
            let bytes = text.as_bytes();
            while end < text.len()
                && (bytes[end] == b'\n' || bytes[end] == b'\r' || bytes[end] == b' ')
            {
                end += 1;
            }
            return end;
        }
    }

    // 优先级 2: 空行
    if let Some(pos) = search_text.rfind("\n\n") {
        if pos > 0 {
            return pos + 2;
        }
    }

    // 优先级 3: 换行
    if let Some(pos) = search_text.rfind('\n') {
        if pos > 0 {
            return pos + 1;
        }
    }

    // 优先级 4: 句末标点
    let mut best = 0usize;
    for &ch in DEFAULT_STOP_CHARS {
        if let Some(pos) = search_text.rfind(ch) {
            if pos > best {
                best = pos;
            }
        }
    }
    if best > 0 {
        // Advance past the stop char (handle multi-byte)
        return best + ch_len_at(text, best);
    }

    // 兜底: 强制在 target_pos 截断
    search_end
}

/// 获取 text 在 pos 处的字符字节长度
fn ch_len_at(text: &str, pos: usize) -> usize {
    text[pos..]
        .chars()
        .next()
        .map(|c| c.len_utf8())
        .unwrap_or(1)
}

/// 将 buffer 拆分为合理大小的段（返回剩余 buffer + 所有段）
pub fn flush_buffer(mut buffer: String, max_segment_size: usize) -> (String, Vec<String>) {
    let mut segments = Vec::new();

    while buffer.len() >= max_segment_size {
        let split_pos = find_split_point(&buffer, max_segment_size);
        if split_pos == 0 {
            break;
        }

        let segment_raw = buffer[..split_pos].trim().to_string();
        buffer = buffer[split_pos..].to_string();

        let segment = clean_msg_markers(&segment_raw);
        if !segment.is_empty() && segment.len() >= 10 && !should_skip_buffer(&segment) {
            segments.push(segment);
        }
    }

    (buffer, segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_msg_markers_removes_tool_and_msg_artifacts() {
        let raw = r#"[MSG1] functions. tool_call {"query":"abc"} data_fetch:1  正常内容"#;
        let cleaned = clean_msg_markers(raw);
        assert_eq!(cleaned, "正常内容");
    }

    #[test]
    fn clean_msg_markers_preserves_newlines() {
        let raw = "第一段。\n\n第二段开始。\n- 列表项\n  - 子项\n";
        let cleaned = clean_msg_markers(raw);
        assert!(cleaned.contains("\n\n"));
        assert!(cleaned.contains("\n- 列表项\n"));
    }

    #[test]
    fn should_skip_buffer_for_tool_call_content() {
        assert!(should_skip_buffer(r#"{"query":"AAPL"}"#));
        assert!(should_skip_buffer("tool_call tool_call"));
        assert!(!should_skip_buffer("这是用户可读的正常回复内容。"));
    }

    #[test]
    fn find_split_point_prefers_paragraph_boundary() {
        let text = "第一段。\n\n第二段开始，内容很多很多很多。";
        let pos = find_split_point(text, 20);
        assert_eq!(&text[..pos], "第一段。\n\n");
    }

    #[test]
    fn flush_buffer_splits_and_keeps_meaningful_segments() {
        let input = "第一段结尾。\n\n第二段内容较长，需要被拆分。".to_string();
        let (remain, segments) = flush_buffer(input, 18);
        assert!(segments.iter().any(|s| s.contains("第一段结尾")));
        assert!(remain.len() < 18);
    }
}
