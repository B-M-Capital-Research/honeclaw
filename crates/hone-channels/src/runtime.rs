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
const GENERIC_USER_ERROR_MESSAGE: &str = "抱歉，这次处理失败了。请稍后再试。";
const TIMEOUT_USER_ERROR_MESSAGE: &str = "抱歉，处理超时了。请稍后再试。";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedUserVisibleOutput {
    pub content: String,
    pub removed_internal: bool,
    pub only_internal: bool,
}

/// 工具显示名称映射
pub fn tool_display_map() -> HashMap<&'static str, (&'static str, bool)> {
    let mut map = HashMap::new();
    map.insert("skill_tool", ("执行技能", false));
    map.insert("discover_skills", ("检索技能", false));
    map.insert("load_skill", ("兼容加载技能", false));
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
        .map(sanitize_user_visible_output)
        .map(|value| value.content.trim().to_string())
        .filter(|value| !value.is_empty());
    if cleaned.is_some() {
        return cleaned;
    }

    let map = tool_display_map();
    if let Some(&(display_name, _)) = map.get(tool_name) {
        return Some(format!("正在{display_name}..."));
    }

    Some(format!("正在调用 {tool_name}..."))
}

/// 将用户可见进度中的 sandbox 绝对路径改写为相对路径；sandbox 外绝对路径做占位隐藏。
pub fn relativize_user_visible_paths(text: &str, sandbox_root: &str) -> String {
    let normalized_root = trim_trailing_path_separators(sandbox_root);
    if text.trim().is_empty() || normalized_root.is_empty() {
        return text.to_string();
    }

    RE_ABSOLUTE_PATH
        .replace_all(text, |caps: &regex::Captures| {
            let prefix = caps.name("prefix").map(|m| m.as_str()).unwrap_or_default();
            let raw = caps.name("path").map(|m| m.as_str()).unwrap_or_default();
            let (path, suffix) = split_trailing_path_punctuation(raw);
            if let Some(relative) = relativize_path_within_root(path, normalized_root) {
                return format!("{prefix}{relative}{suffix}");
            }
            format!("{prefix}{}{suffix}", mask_absolute_path(path))
        })
        .to_string()
}

fn trim_trailing_path_separators(value: &str) -> &str {
    value.trim_end_matches(|ch| ch == '/' || ch == '\\')
}

fn split_trailing_path_punctuation(raw: &str) -> (&str, &str) {
    let mut end = raw.len();
    loop {
        let slice = &raw[..end];
        if slice.ends_with("...") {
            end -= 3;
            continue;
        }
        let Some(ch) = slice.chars().next_back() else {
            break;
        };
        if matches!(ch, ',' | ';' | ')' | ']' | '}' | '>' | '"' | '\'' | '`') {
            end -= ch.len_utf8();
            continue;
        }
        if ch == ':' {
            end -= ch.len_utf8();
            continue;
        }
        break;
    }
    (&raw[..end], &raw[end..])
}

fn relativize_path_within_root<'a>(path: &'a str, sandbox_root: &str) -> Option<&'a str> {
    if path == sandbox_root {
        return Some(".");
    }
    let rest = path.strip_prefix(sandbox_root)?;
    rest.strip_prefix('/').or_else(|| rest.strip_prefix('\\'))
}

fn mask_absolute_path(path: &str) -> String {
    let trimmed = trim_trailing_path_separators(path);
    let basename = trimmed
        .rsplit(['/', '\\'])
        .find(|segment| !segment.is_empty())
        .unwrap_or_default();
    if basename.is_empty() {
        "<absolute-path>".to_string()
    } else {
        format!("<absolute-path>/{basename}")
    }
}

// ── 静态正则（编译一次，避免热路径重复编译）─────────────────────────────────
use std::sync::LazyLock;

static RE_MSG: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\[MSG\d+\]\s*").expect("valid regex"));
static RE_PIPE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"<\|[^|]+\|>").expect("valid regex"));
static RE_TOOL_TAG: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"\b(web_search|data_fetch|portfolio|load_skill|skill_tool|discover_skills|image_gen):\d+\s*",
    )
    .expect("valid regex")
});
static RE_JSON_TOOL: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"\{[^\{\}]*"(query|action|data_type|skill_name|ticker|symbol|draft_id|approval_token|image_prompt|user_intent|image_count|regenerate_images|image_type|content|prompt)"[^\{\}]*\}"#,
    )
    .expect("valid regex")
});
static RE_SIMPLE_JSON: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"\{"[^"]*":\s*"[^"]*"\}"#).expect("valid regex"));
static RE_FUNC: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(functions?\.?\s*)+").expect("valid regex"));
static RE_TOOL_KEYWORD: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"</?(tool_call|tool_result|tool_use)\b[^>]*>|\b(tool_call|tool_result|tool_use)\b",
    )
    .expect("valid regex")
});
static RE_WS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[ \t]+").expect("valid regex"));
static RE_NL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\n[ \t\n]*\n").expect("valid regex"));
static RE_INTERNAL_BLOCK: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?is)<think\b[^>]*>.*?</think>|<tool_code\b[^>]*>.*?</tool_code>|</?(tool_call|tool_result|tool_use)\b[^>]*>",
    )
    .expect("valid regex")
});
static RE_BRACKET_INTERNAL_BLOCK: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?is)\[(?:/)?TOOL_(?:CALL|RESULT|USE)[^\]]*\]").expect("valid regex")
});
static RE_INTERNAL_PROTOCOL_LINE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?ix)
        ^
        (
            <(?:tool_call|tool_result|tool_use|parameter)\b
            |
            </(?:tool_call|tool_result|tool_use|parameter)>
            |
            \[(?:/)?TOOL_(?:CALL|RESULT|USE)[^\]]*\]
            |
            \{[^{}]*(?:"name"\s*:\s*"[^"]+"|"parameters"\s*:|"queryType"\s*:|"maxResults"\s*:)[^{}]*\}
        )
        "#,
    )
    .expect("valid regex")
});
static RE_ABSOLUTE_PATH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?P<prefix>^|[\s\(\[\{<"'`])(?P<path>(?:[A-Za-z]:[\\/]|/)[^\s<>"'`]+)"#)
        .expect("valid regex")
});

// ── skip-buffer 检测正则 ──────────────────────────────────────────────────────
static RE_ONLY_PUNCT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[\s\.\,\!\?\:\;\-\_\=\+\*\/\\]+$").expect("valid regex"));
static RE_ONLY_FUNC: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^(functions?\.?\s*)+$").expect("valid regex"));
static RE_ONLY_TOOL_CALL: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^(tool_?call\.?\s*)+$").expect("valid regex"));

/// 清理消息中的特殊标记（工具调用残留、MSG 标记等）
pub fn clean_msg_markers(text: &str) -> String {
    let mut cleaned = text.to_string();

    // [MSG1], [MSG2] 等
    cleaned = RE_MSG.replace_all(&cleaned, "").to_string();
    // <|...|> 标记
    cleaned = RE_PIPE.replace_all(&cleaned, "").to_string();
    // tool_name:N 标记
    cleaned = RE_TOOL_TAG.replace_all(&cleaned, "").to_string();
    // JSON 工具参数
    cleaned = RE_JSON_TOOL.replace_all(&cleaned, "").to_string();
    // 简单 JSON
    cleaned = RE_SIMPLE_JSON.replace_all(&cleaned, "").to_string();
    // functions 残留
    cleaned = RE_FUNC.replace_all(&cleaned, "").to_string();
    // tool_call/tool_result/tool_use 及其可能附带的尖括号
    cleaned = RE_TOOL_KEYWORD.replace_all(&cleaned, "").to_string();
    // 多余空白（不包含换行）
    cleaned = RE_WS.replace_all(&cleaned, " ").to_string();
    // 连续多个换行（可能夹杂空格）压缩为两个换行，保留段落结构
    cleaned = RE_NL.replace_all(&cleaned, "\n\n").to_string();

    cleaned.trim().to_string()
}

pub fn sanitize_user_visible_output(text: &str) -> SanitizedUserVisibleOutput {
    if text.trim().is_empty() {
        return SanitizedUserVisibleOutput {
            content: String::new(),
            removed_internal: false,
            only_internal: false,
        };
    }

    let mut removed_internal = false;
    let mut sanitized = text.replace("\r\n", "\n");

    let block_stripped = RE_INTERNAL_BLOCK.replace_all(&sanitized, "\n");
    if block_stripped != sanitized {
        removed_internal = true;
        sanitized = block_stripped.into_owned();
    }

    let bracket_stripped = RE_BRACKET_INTERNAL_BLOCK.replace_all(&sanitized, "");
    if bracket_stripped != sanitized {
        removed_internal = true;
        sanitized = bracket_stripped.into_owned();
    }

    let mut kept_lines = Vec::new();
    for line in sanitized.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            kept_lines.push(String::new());
            continue;
        }
        if RE_INTERNAL_PROTOCOL_LINE.is_match(trimmed) || is_tool_call_content(trimmed) {
            removed_internal = true;
            continue;
        }
        kept_lines.push(line.to_string());
    }

    sanitized = kept_lines.join("\n");
    sanitized = RE_WS.replace_all(&sanitized, " ").to_string();
    sanitized = RE_NL.replace_all(&sanitized, "\n\n").to_string();
    sanitized = sanitized.trim().to_string();

    SanitizedUserVisibleOutput {
        only_internal: removed_internal && sanitized.is_empty(),
        removed_internal,
        content: sanitized,
    }
}

pub fn user_visible_error_message(raw: Option<&str>) -> String {
    let sanitized = raw
        .map(sanitize_user_visible_output)
        .map(|value| value.content.trim().to_string())
        .filter(|value| !value.is_empty());

    let Some(sanitized) = sanitized else {
        return GENERIC_USER_ERROR_MESSAGE.to_string();
    };

    let lowered = sanitized.to_ascii_lowercase();
    if lowered.contains("timeout") || lowered.contains("timed out") {
        return TIMEOUT_USER_ERROR_MESSAGE.to_string();
    }

    if looks_internal_error_detail(&sanitized, &lowered) {
        return GENERIC_USER_ERROR_MESSAGE.to_string();
    }

    sanitized
}

fn looks_internal_error_detail(sanitized: &str, lowered: &str) -> bool {
    sanitized.contains("LLM 错误")
        || sanitized.contains("HTTP 错误")
        || sanitized.contains("渠道错误")
        || sanitized.contains("工具执行错误")
        || sanitized.contains("序列化错误")
        || sanitized.contains("IO 错误")
        || lowered.contains("max_iterations_exceeded")
        || lowered.contains("bad_request_error")
        || lowered.contains("invalid params")
        || lowered.contains("tool_call_id")
        || lowered.contains("tool call result")
        || lowered.contains("function arguments")
        || lowered.contains("provider")
        || lowered.contains("session/prompt")
        || lowered.contains("codex acp")
        || lowered.contains("stream closed before response")
        || lowered.contains("acp stream")
}

/// 检测文本是否包含工具调用标记
pub fn is_tool_call_content(text: &str) -> bool {
    const MARKERS: &[&str] = &[
        "<think",
        "</think>",
        "<tool_call",
        "</tool_call>",
        "<tool_result",
        "</tool_result>",
        "<tool_use",
        "</tool_use>",
        "<parameter",
        "</parameter>",
        "[TOOL_CALL]",
        "[/TOOL_CALL]",
        "[TOOL_RESULT]",
        "[/TOOL_RESULT]",
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
        "skill_tool:",
        "discover_skills:",
        "image_gen:",
        r#"{"image_type""#,
    ];
    MARKERS.iter().any(|marker| text.contains(marker))
}

pub(crate) fn is_context_overflow_error(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    normalized.contains("context window exceeds limit")
        || normalized.contains("context window overflow")
        || normalized.contains("context_window_will_overflow")
        || normalized.contains("context length exceeded")
        || normalized.contains("maximum context length")
        || normalized.contains("prompt is too long")
        || normalized.contains("too many tokens")
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
    // 无意义内容（使用静态正则，避免重复编译）
    if RE_ONLY_PUNCT.is_match(&cleaned)
        || RE_ONLY_FUNC.is_match(&cleaned)
        || RE_ONLY_TOOL_CALL.is_match(&cleaned)
        || cleaned.trim().is_empty()
    {
        return true;
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
    fn sanitize_user_visible_output_strips_internal_blocks_and_keeps_answer() {
        let raw = "<think>\n先查一下。\n</think>\n最终结论：公司今日上涨主要因为财报超预期。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "最终结论：公司今日上涨主要因为财报超预期。"
        );
        assert!(!sanitized.only_internal);
    }

    #[test]
    fn sanitize_user_visible_output_drops_raw_tool_protocol_only_payload() {
        let raw = r#"<tool_call>{"name":"web_search","parameters":{"query":"Tempus AI stock surge today"}}</tool_call>"#;
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert!(sanitized.only_internal);
        assert!(sanitized.content.is_empty());
    }

    #[test]
    fn user_visible_error_message_rewrites_provider_protocol_errors() {
        let err = user_visible_error_message(Some(
            "LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013), tool_call_id: call_123",
        ));
        assert_eq!(err, GENERIC_USER_ERROR_MESSAGE);
        assert!(!err.contains("bad_request_error"));
        assert!(!err.contains("tool_call_id"));
    }

    #[test]
    fn user_visible_error_message_maps_timeout_errors() {
        let err =
            user_visible_error_message(Some("opencode acp session/prompt idle timeout (180s)"));
        assert_eq!(err, TIMEOUT_USER_ERROR_MESSAGE);
    }

    #[test]
    fn user_visible_error_message_preserves_already_friendly_text() {
        let err = user_visible_error_message(Some(
            "当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。",
        ));
        assert!(err.contains("当前会话上下文过长"));
        assert!(!err.contains("bad_request_error"));
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

    #[test]
    fn relativize_user_visible_paths_strips_sandbox_prefix() {
        let root = "/tmp/hone-agent-sandboxes/telegram/direct8039067465";
        let text =
            format!("Edit {root}/company_profiles/sandisk/profile.md, {root}/data/foo/bar.txt");
        let sanitized = relativize_user_visible_paths(&text, root);
        assert_eq!(
            sanitized,
            "Edit company_profiles/sandisk/profile.md, data/foo/bar.txt"
        );
    }

    #[test]
    fn relativize_user_visible_paths_masks_outside_sandbox_paths() {
        let text = "Edit /Users/bytedance/secret/profile.md and C:\\Users\\foo\\private\\note.txt";
        let sanitized = relativize_user_visible_paths(text, "/tmp/hone-agent-sandboxes/demo");
        assert_eq!(
            sanitized,
            "Edit <absolute-path>/profile.md and <absolute-path>/note.txt"
        );
    }

    #[test]
    fn relativize_user_visible_paths_keeps_relative_paths() {
        let text = "Run rtk sed -n '1,260p' company_profiles/sandisk/profile.md";
        let sanitized =
            relativize_user_visible_paths(text, "/tmp/hone-agent-sandboxes/telegram/demo");
        assert_eq!(sanitized, text);
    }
}
