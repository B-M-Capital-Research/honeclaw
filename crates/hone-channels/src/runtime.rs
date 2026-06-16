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
const RUNNER_USAGE_LIMIT_USER_ERROR_MESSAGE: &str =
    "当前执行额度已用尽，暂时无法继续处理。请稍后再试。";
const RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE: &str =
    "当前本机执行环境暂时不可用，请稍后再试。";
const CRON_TASK_MANAGEMENT_UNAVAILABLE_USER_MESSAGE: &str = "定时任务管理暂时不可用，请稍后再试。";

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
    value.trim_end_matches(['/', '\\'])
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
static RE_COMPACT_MARKER_LINE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)^\s*(context|conversation)\s+compacted[\s\.\u{3002}:：-]*$")
        .expect("valid regex")
});
static RE_LOCAL_MARKDOWN_LINK: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"\[(?P<label>[^\]\n]{0,240})\]\((?P<path>(?:file://)?(?:[A-Za-z]:[\\/]|/)[^)\n]+)\)"#,
    )
    .expect("valid regex")
});
static RE_FILE_URI_ABSOLUTE_PATH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?P<prefix>^|[\s\(\[\{<"'`])file://(?P<path>(?:[A-Za-z]:[\\/]|/)[^\s<>"'`]+)"#,
    )
    .expect("valid regex")
});
static RE_ABSOLUTE_PATH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?P<prefix>^|[\s\(\[\{<"'`])(?P<path>(?:[A-Za-z]:[\\/]|/)[^\s<>"'`]+)"#)
        .expect("valid regex")
});
static RE_INTERNAL_RELATIVE_PROFILE_PATH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?P<prefix>^|[\s\(\[\{<"'`])(?P<path>(?:company_profiles/[^\s<>"'`，。；、）\)\]\}]+|events/[^\s<>"'`，。；、）\)\]\}]+\.md))"#,
    )
    .expect("valid regex")
});
static RE_ENABLED_BOOLEAN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?i)`?enabled\s*=\s*(?P<value>true|false)`?"#).expect("valid regex")
});
static RE_ENABLED_BOOLEAN_COPY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?P<prefix>这\s*\d+\s*个任务目前)(?:都|全部都|都是)\s*已(?P<state>启用|停用)"#,
    )
    .expect("valid regex")
});
static RE_INTERNAL_SKILL_COPY_SENTENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?i)[^\n。！？]*(?:stock_research|deep_stock_research|image_understanding|pdf_understanding|skill|技能|工具|图片理解|图片分析|附件处理|OCR)[^\n。！？]*(?:未激活|没有激活|没激活|未成功激活|没有成功激活|没成功激活|未加载|没有加载|没加载|未成功加载|没有成功加载|没成功加载|不可读|无法读取|读取失败)[^\n。！？]*[。！？]?"#,
    )
    .expect("valid regex")
});
static RE_INTERNAL_FRAMEWORK_COPY_SENTENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"[^\n。！？]*(?:改用|改为|转而使用)[^\n。！？]*(?:技能框架|skill|tool)[^\n。！？]*[。！？]?"#)
        .expect("valid regex")
});
static RE_INTERNAL_STORAGE_COPY_SENTENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"[^\n。！？]*(?:账本文件已定位到|本地\s*data/|data/portfolio|data/cron_jobs|data/sessions\.sqlite3|sessions\.sqlite3|session_messages|session_metadata|当前沙盒|holdings\.json|空目录|本地json文件|本地 json 文件|本地json|本地 json|本地文件仍只显示|json文件仍只显示)[^\n。！？]*[。！？]?"#,
    )
    .expect("valid regex")
});
static RE_INTERNAL_USER_INFO_IDENTITY_COPY_SENTENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"[^\n。！？]*(?:当前会话\s*ID|会话\s*ID|session[_\s-]*id|open_id|chat_id|手机号等?元数据|手机号\s*metadata|飞书\s*open_id|飞书\s*chat_id)[^\n。！？]*[。！？]?"#,
    )
    .expect("valid regex")
});
static RE_INTERNAL_USER_INFO_STORAGE_ENUM_COPY_SENTENCE: LazyLock<regex::Regex> =
    LazyLock::new(|| {
        regex::Regex::new(
            r#"[^\n。！？]*(?:当前工作区|当前目录|当前本地|本地可见|存在公司画像目录)[^\n。！？]*(?:company_profiles|data/notif_prefs|data/portfolio|data/cron_jobs|data/sessions\.sqlite3|uploads|公司画像公司画像)[^\n。！？]*[。！？]?"#,
        )
        .expect("valid regex")
    });
static RE_INTERNAL_TOOLING_COPY_SENTENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"[^\n。！？]*(?:返回了?全市场列表|全市场列表而不是按标的过滤|工具过滤异常)[^\n。！？]*[。！？]?"#,
    )
    .expect("valid regex")
});
static RE_CRON_TOOL_UNAVAILABLE_COPY_SENTENCE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?:[^\n。！？]*(?:定时任务|推送任务|cron_job|scheduled_task)[^\n。！？]*(?:工具未暴露|接口未暴露|未暴露可执行|没有暴露出来|没有拿到可操作|没有写入接口|没有\s*cron_job|没有\s*scheduled_task|工具列表里没有|没有可用的[^。！？\n]*(?:入口|工具)|无法真实执行|不能真实执行|不能真实创建|不能直接完成自动创建|没有成功创建|不能确认[^。！？\n]*创建成功)[^\n。！？]*|[^\n。！？]*(?:没有\s*cron_job|没有\s*scheduled_task|工具列表里没有)[^\n。！？]*(?:cron_job|scheduled_task|定时任务|推送任务)[^\n。！？]*)[。！？]?"#,
    )
    .expect("valid regex")
});
static RE_COMPANY_PROFILE_COPY_GLITCH: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?:路径是[:：]?\s*公司画像(?:公司画像)?|本地画像[:：]?\s*公司画像|本地公司画像[:：]?\s*公司画像|画像已更新[:：]?\s*公司画像(?:公司画像)?)"#,
    )
    .expect("valid regex")
});
static RE_COMPANY_PROFILE_UPDATE_COPY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(?:我已|并)?把本轮更新补进本地画像[:：]?\s*公司画像"#)
        .expect("valid regex")
});
static RE_COMPANY_PROFILE_WRITE_COPY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"沉淀到本地公司画像[:：]?\s*公司画像"#).expect("valid regex")
});
static RE_COMPANY_PROFILE_CREATED_COPY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?:已为你建立长期画像|本轮已新增长期画像)[:：]?\s*(?:公司画像|company_profiles/[^\s<>"'`，。；、）\)\]\}]+)"#,
    )
    .expect("valid regex")
});
static RE_COMPANY_PROFILE_WRITTEN_LIST_COPY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"已写入[:：]?\s*(?:(?:[-*]\s*)?\d+\.\s*公司画像(?:\s|$|[\n\r，,；;。:：])*){1,}"#,
    )
    .expect("valid regex")
});
static RE_MARKET_DATA_FALLBACK_COPY: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r#"(?i)(?:[^。\n；]*(?:专用\s*)?data_fetch[^。\n；]*(?:未返回|不可用|校验)[^。\n；]*|[^。\n；]*未取得[^。\n；]*data_fetch[^。\n；]*返回[^。\n；]*|[^。\n]*未能取得新的\s*data_fetch\s*/\s*网页行情返回[^。\n]*|[^。\n]*data_fetch\s+quote[^。\n]*|[^。\n；]*(?:专用行情工具|可用行情接口|主行情源)[^。\n；]*(?:未返回|未取得|不可用)[^。\n；]*(?:stockanalysis|页面补充校验|公开页面|网页源|行情页|校验)[^。\n；]*)"#,
    )
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

/// 剥离 `<think>` / `<tool_code>` / `<tool_call>` 等 runner 内部 reasoning 块。
///
/// 用于 heartbeat 结构化解析、scheduler 出站净化等需要「先拿到 LLM 的公开正文再做
/// 契约判断」的链路。与 `sanitize_user_visible_output` 共用同一条规则，保证
/// 「什么算内部 reasoning」在全链路单一来源。
pub fn strip_internal_reasoning_blocks(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    strip_internal_protocol_blocks(normalized).0
}

pub fn sanitize_user_visible_output(text: &str) -> SanitizedUserVisibleOutput {
    if text.trim().is_empty() {
        return SanitizedUserVisibleOutput {
            content: String::new(),
            removed_internal: false,
            only_internal: false,
        };
    }

    let (mut sanitized, mut removed_internal) =
        strip_internal_protocol_blocks(text.replace("\r\n", "\n"));

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
        if RE_COMPACT_MARKER_LINE.is_match(trimmed) {
            removed_internal = true;
            continue;
        }
        kept_lines.push(line.to_string());
    }

    sanitized = kept_lines.join("\n");
    if let Some(stripped) = strip_internal_workflow_prelude(&sanitized) {
        removed_internal = true;
        sanitized = stripped;
    }
    let (path_sanitized, removed_paths) = redact_user_visible_local_paths(&sanitized);
    sanitized = path_sanitized;
    removed_internal |= removed_paths;
    let (copy_rewritten, removed_copy) = rewrite_user_visible_internal_copy(&sanitized);
    sanitized = copy_rewritten;
    removed_internal |= removed_copy;
    sanitized = RE_WS.replace_all(&sanitized, " ").to_string();
    sanitized = RE_NL.replace_all(&sanitized, "\n\n").to_string();
    sanitized = sanitized.trim().to_string();

    SanitizedUserVisibleOutput {
        only_internal: removed_internal && sanitized.is_empty(),
        removed_internal,
        content: sanitized,
    }
}

fn is_hone_mcp_binary_missing_error(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    normalized.contains("hone-mcp binary not found")
        || (normalized.contains("hone_mcp_bin") && normalized.contains("not found"))
}

fn strip_internal_protocol_blocks(mut value: String) -> (String, bool) {
    let mut removed_internal = false;

    let block_stripped = RE_INTERNAL_BLOCK.replace_all(&value, "\n");
    if block_stripped != value {
        removed_internal = true;
        value = block_stripped.into_owned();
    }

    let bracket_stripped = RE_BRACKET_INTERNAL_BLOCK.replace_all(&value, "");
    if bracket_stripped != value {
        removed_internal = true;
        value = bracket_stripped.into_owned();
    }

    (value, removed_internal)
}

fn redact_user_visible_local_paths(text: &str) -> (String, bool) {
    let mut removed = false;

    let markdown_stripped = RE_LOCAL_MARKDOWN_LINK.replace_all(text, |caps: &regex::Captures| {
        removed = true;
        let label = caps
            .name("label")
            .map(|m| m.as_str().trim())
            .unwrap_or_default();
        let raw_path = caps
            .name("path")
            .map(|m| m.as_str())
            .unwrap_or_default()
            .trim_start_matches("file://");
        if label.is_empty()
            || RE_ABSOLUTE_PATH.is_match(label)
            || RE_FILE_URI_ABSOLUTE_PATH.is_match(label)
        {
            mask_absolute_path(raw_path)
        } else {
            label.to_string()
        }
    });
    let mut sanitized = markdown_stripped.into_owned();

    let file_uri_stripped =
        RE_FILE_URI_ABSOLUTE_PATH.replace_all(&sanitized, |caps: &regex::Captures| {
            removed = true;
            let prefix = caps.name("prefix").map(|m| m.as_str()).unwrap_or_default();
            let raw = caps.name("path").map(|m| m.as_str()).unwrap_or_default();
            let (path, suffix) = split_trailing_path_punctuation(raw);
            format!("{prefix}{}{suffix}", mask_absolute_path(path))
        });
    sanitized = file_uri_stripped.into_owned();

    let absolute_stripped = RE_ABSOLUTE_PATH.replace_all(&sanitized, |caps: &regex::Captures| {
        removed = true;
        let prefix = caps.name("prefix").map(|m| m.as_str()).unwrap_or_default();
        let raw = caps.name("path").map(|m| m.as_str()).unwrap_or_default();
        let (path, suffix) = split_trailing_path_punctuation(raw);
        format!("{prefix}{}{suffix}", mask_absolute_path(path))
    });
    sanitized = absolute_stripped.into_owned();

    let internal_relative_stripped =
        RE_INTERNAL_RELATIVE_PROFILE_PATH.replace_all(&sanitized, |caps: &regex::Captures| {
            removed = true;
            let prefix = caps.name("prefix").map(|m| m.as_str()).unwrap_or_default();
            let display_prefix = if prefix.chars().all(char::is_whitespace) {
                ""
            } else {
                prefix
            };
            format!("{display_prefix}公司画像")
        });

    (internal_relative_stripped.into_owned(), removed)
}

fn rewrite_user_visible_internal_copy(text: &str) -> (String, bool) {
    let mut removed = false;

    let enabled_rewritten = RE_ENABLED_BOOLEAN.replace_all(text, |caps: &regex::Captures| {
        removed = true;
        match caps
            .name("value")
            .map(|value| value.as_str().to_ascii_lowercase())
            .as_deref()
        {
            Some("true") => "已启用".to_string(),
            Some("false") => "已停用".to_string(),
            _ => caps[0].to_string(),
        }
    });
    let mut rewritten = enabled_rewritten.into_owned();

    let normalized_enabled =
        RE_ENABLED_BOOLEAN_COPY.replace_all(&rewritten, "${prefix}均已${state}");
    if normalized_enabled != rewritten {
        removed = true;
        rewritten = normalized_enabled.into_owned();
    }

    let cron_tool_unavailable = RE_CRON_TOOL_UNAVAILABLE_COPY_SENTENCE
        .replace_all(&rewritten, CRON_TASK_MANAGEMENT_UNAVAILABLE_USER_MESSAGE);
    if cron_tool_unavailable != rewritten {
        removed = true;
        rewritten = cron_tool_unavailable.into_owned();
    }

    for re in [
        &RE_INTERNAL_SKILL_COPY_SENTENCE,
        &RE_INTERNAL_FRAMEWORK_COPY_SENTENCE,
        &RE_INTERNAL_STORAGE_COPY_SENTENCE,
        &RE_INTERNAL_USER_INFO_IDENTITY_COPY_SENTENCE,
        &RE_INTERNAL_USER_INFO_STORAGE_ENUM_COPY_SENTENCE,
        &RE_INTERNAL_TOOLING_COPY_SENTENCE,
    ] {
        let next = re.replace_all(&rewritten, "");
        if next != rewritten {
            removed = true;
            rewritten = next.into_owned();
        }
    }

    for (re, replacement) in [
        (&RE_COMPANY_PROFILE_CREATED_COPY, "公司画像已更新"),
        (&RE_COMPANY_PROFILE_UPDATE_COPY, "把本轮更新补进公司画像"),
        (&RE_COMPANY_PROFILE_WRITE_COPY, "沉淀到公司画像"),
        (&RE_COMPANY_PROFILE_WRITTEN_LIST_COPY, "已写入公司画像"),
        (&RE_COMPANY_PROFILE_COPY_GLITCH, "已沉淀为公司画像"),
        (
            &RE_MARKET_DATA_FALLBACK_COPY,
            "主行情源本轮未返回可用结果，已改用公开页面补充校验",
        ),
    ] {
        let next = re.replace_all(&rewritten, replacement);
        if next != rewritten {
            removed = true;
            rewritten = next.into_owned();
        }
    }

    (rewritten, removed)
}

pub fn user_visible_error_message(raw: Option<&str>) -> String {
    let Some(sanitized) = sanitized_non_empty_user_visible(raw) else {
        return GENERIC_USER_ERROR_MESSAGE.to_string();
    };

    let lowered = sanitized.to_ascii_lowercase();
    if is_hone_mcp_binary_missing_error(&sanitized) {
        return RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE.to_string();
    }
    if looks_runner_transport_disconnect_error_lowered(&lowered) {
        return RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE.to_string();
    }
    if let Some(message) = user_actionable_error_message(&sanitized, &lowered) {
        return message;
    }
    if looks_timeout_error_lowered(&lowered) {
        return TIMEOUT_USER_ERROR_MESSAGE.to_string();
    }
    if looks_sensitive_error_detail(&lowered) {
        return GENERIC_USER_ERROR_MESSAGE.to_string();
    }
    if looks_internal_error_detail(&sanitized, &lowered) {
        return GENERIC_USER_ERROR_MESSAGE.to_string();
    }

    sanitized
}

pub fn user_visible_error_message_or_none(raw: Option<&str>) -> Option<String> {
    let sanitized = sanitized_non_empty_user_visible(raw)?;
    let lowered = sanitized.to_ascii_lowercase();
    if let Some(message) = user_actionable_error_message(&sanitized, &lowered) {
        return Some(message);
    }
    if looks_internal_error_detail(&sanitized, &lowered) {
        return None;
    }
    if looks_sensitive_error_detail(&lowered) {
        return None;
    }
    if looks_timeout_error_lowered(&lowered) {
        return Some(TIMEOUT_USER_ERROR_MESSAGE.to_string());
    }
    Some(sanitized)
}

fn sanitized_non_empty_user_visible(raw: Option<&str>) -> Option<String> {
    raw.map(sanitize_user_visible_output)
        .map(|value| value.content.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn user_actionable_error_message(sanitized: &str, lowered: &str) -> Option<String> {
    quota_rejection_user_message(sanitized)
        .or_else(|| {
            looks_runner_usage_limit_error_lowered(lowered)
                .then(|| RUNNER_USAGE_LIMIT_USER_ERROR_MESSAGE.to_string())
        })
        .or_else(|| {
            looks_runner_resource_unavailable_error_lowered(lowered)
                .then(|| RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE.to_string())
        })
}

fn quota_rejection_user_message(sanitized: &str) -> Option<String> {
    let start = sanitized.find("已达到今日对话上限")?;
    let rest = sanitized[start..].trim();
    let first_line = rest.lines().next().unwrap_or(rest).trim();
    (!first_line.is_empty()).then(|| first_line.to_string())
}

pub fn is_runner_usage_limit_error(raw: &str) -> bool {
    looks_runner_usage_limit_error_lowered(&raw.to_ascii_lowercase())
}

fn looks_runner_usage_limit_error_lowered(lowered: &str) -> bool {
    (lowered.contains("codex") || lowered.contains("runner") || lowered.contains("acp"))
        && (lowered.contains("usage limit")
            || lowered.contains("usage limits")
            || lowered.contains("rate limit")
            || lowered.contains("quota exceeded")
            || lowered.contains("quota exhausted")
            || lowered.contains("insufficient quota")
            || lowered.contains("try again later"))
}

fn looks_runner_resource_unavailable_error_lowered(lowered: &str) -> bool {
    (lowered.contains("codex")
        || lowered.contains("codex-acp")
        || lowered.contains("hone-mcp")
        || lowered.contains("runner")
        || lowered.contains("acp"))
        && (lowered.contains("resource temporarily unavailable")
            || lowered.contains("os error 35")
            || lowered.contains("would block")
            || lowered.contains("failed to probe")
            || lowered.contains("version probe")
            || lowered.contains("failed to spawn")
            || lowered.contains("binary not found")
            || lowered.contains("not found near current executable"))
}

fn looks_runner_transport_disconnect_error_lowered(lowered: &str) -> bool {
    (lowered.contains("codex") || lowered.contains("runner") || lowered.contains("acp"))
        && (lowered.contains("stream disconnected before completion")
            || lowered.contains("stream closed before response")
            || lowered.contains("acp stream disconnected")
            || lowered.contains("transport disconnected"))
}

fn looks_timeout_error_lowered(lowered: &str) -> bool {
    lowered.contains("timeout") || lowered.contains("timed out")
}

fn looks_sensitive_error_detail(lowered: &str) -> bool {
    [
        "api_key=",
        "api_key:",
        "apikey=",
        "apikey:",
        "x-api-key=",
        "x-api-key:",
        "openrouter_api_key=",
        "anthropic_api_key=",
        "gemini_api_key=",
        "google_api_key=",
        "tavily_api_key=",
        "fmp_api_key=",
        "hone_cloud_api_key=",
        "client_secret=",
        "client_secret:",
        "refresh_token=",
        "refresh_token:",
        "id_token=",
        "id_token:",
        "session_token=",
        "session_token:",
        "bot_token=",
        "bot_token:",
        "authorization=",
        "authorization:",
        "bearer ",
        "basic ",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
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
        || lowered.contains("stream disconnected before completion")
        || lowered.contains("stream closed before response")
        || lowered.contains("acp stream")
}

fn strip_internal_workflow_prelude(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(paragraph_end) = trimmed.find("\n\n") {
        let first_paragraph = &trimmed[..paragraph_end];
        let rest = trimmed[paragraph_end..].trim_start();
        if !rest.is_empty() && looks_like_internal_workflow_prelude(first_paragraph) {
            return Some(rest.to_string());
        }
    }

    let first_sentence_end = trimmed.char_indices().find_map(|(idx, ch)| {
        matches!(ch, '。' | '！' | '!' | '\n').then_some(idx + ch.len_utf8())
    });
    if let Some(sentence_end) = first_sentence_end {
        let first_sentence = &trimmed[..sentence_end];
        let rest = trimmed[sentence_end..].trim_start();
        if rest.chars().count() >= 30 && looks_like_internal_workflow_prelude(first_sentence) {
            return Some(rest.to_string());
        }
    }

    None
}

fn looks_like_internal_workflow_prelude(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 320 {
        return false;
    }

    let lowered = trimmed.to_ascii_lowercase();
    if contains_any_casefolded(
        trimmed,
        &lowered,
        &[
            "todo",
            "current-plan",
            "current plan",
            "动态计划",
            "任务计划",
            "不落盘",
            "文档方面",
            "工作流",
            "检查本地是否已有相关公司画像",
            "检查本地公司画像",
            "技能未加载",
            "技能没有成功加载",
            "技能没成功加载",
            "当前运行器",
            "tool unavailable",
            "tool is unavailable",
            "skill unavailable",
            "skill is unavailable",
            "failed to load skill",
        ],
    ) {
        return true;
    }

    let starts_like_workflow = starts_with_any(
        trimmed,
        &[
            "我先",
            "我会先",
            "我接下来",
            "接下来我",
            "先",
            "先按",
            "先对齐",
            "先检查",
        ],
    );
    if !starts_like_workflow {
        return false;
    }

    if contains_any(trimmed, &["结论", "答案", "核心判断", "直接说", "简短说"]) {
        return false;
    }

    let has_workflow_verb = contains_any(
        trimmed,
        &[
            "核验",
            "检查",
            "对齐",
            "拆成",
            "整理",
            "补查",
            "调取",
            "检索",
            "看本地",
            "拉取",
            "搜索",
            "梳理",
        ],
    );
    let has_sequence = contains_any(trimmed, &["再", "然后", "最后", "之后"]);
    has_workflow_verb && has_sequence
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

/// 检测 agent 最终输出是否是过渡性计划句（而非实质答复）。
///
/// 过渡计划句通常很短（< 200 字符）且包含"我先/我再/还缺/我需要先"等执行状态描述。
/// 这类内容不应作为最终答复发送给用户。
pub(crate) fn is_transitional_planning_sentence(text: &str) -> bool {
    let char_count = text.chars().count();
    if char_count >= 200 || char_count == 0 {
        return false;
    }
    if text.contains('？') || text.contains('?') {
        return false;
    }
    let trimmed = text.trim_start();
    let starts_like_internal_planning = starts_with_any(
        trimmed,
        &[
            "我先",
            "我再",
            "我需要先",
            "我还缺",
            "我需要补",
            "先看本地",
            "先补查",
            "先调取",
            "先核验",
            "先抓取",
            "还缺一件事",
            "我还需要先",
        ],
    );
    if !starts_like_internal_planning {
        return false;
    }
    if contains_any(
        text,
        &[
            "请先确认",
            "请确认",
            "先确认",
            "请先提供",
            "请提供",
            "告诉我",
            "发我",
            "补充一下",
        ],
    ) {
        return false;
    }
    let patterns = [
        "我先",
        "我再",
        "我需要先",
        "我还缺",
        "我需要补",
        "我先调取",
        "我先补查",
        "我先看",
        "我先拿",
        "我先查",
        "先看本地",
        "先补查",
        "先调取",
        "先核验",
        "先抓取",
        "还缺一件事",
        "我还需要先",
    ];
    contains_any(text, &patterns)
}

fn contains_any(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}

fn contains_any_casefolded(text: &str, lowered: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .any(|marker| lowered.contains(marker) || text.contains(marker))
}

fn starts_with_any(text: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| text.starts_with(prefix))
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
    if let Some(pos) = search_text.rfind("---")
        && pos > 0
    {
        let mut end = pos + 3;
        let bytes = text.as_bytes();
        while end < text.len() && (bytes[end] == b'\n' || bytes[end] == b'\r' || bytes[end] == b' ')
        {
            end += 1;
        }
        return end;
    }

    // 优先级 2: 空行
    if let Some(pos) = search_text.rfind("\n\n")
        && pos > 0
    {
        return pos + 2;
    }

    // 优先级 3: 换行
    if let Some(pos) = search_text.rfind('\n')
        && pos > 0
    {
        return pos + 1;
    }

    // 优先级 4: 句末标点
    if let Some(best) = DEFAULT_STOP_CHARS
        .iter()
        .filter_map(|ch| search_text.rfind(*ch))
        .max()
        .filter(|pos| *pos > 0)
    {
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
    fn sanitize_user_visible_output_drops_acp_compact_marker_lines() {
        for variant in [
            "Context compacted",
            "context compacted.",
            "Conversation compacted",
            "  CONVERSATION COMPACTED  ",
        ] {
            let raw = format!("{}\n模型对本轮的真实回答内容。", variant);
            let sanitized = sanitize_user_visible_output(&raw);
            assert!(
                !sanitized.content.contains(variant.trim()),
                "variant {variant:?} should be removed"
            );
            assert!(
                sanitized.content.contains("模型对本轮的真实回答内容。"),
                "real reply should still remain visible for {variant:?}"
            );
            assert!(sanitized.removed_internal);
        }
    }

    #[test]
    fn sanitize_user_visible_output_strips_internal_workflow_prelude() {
        let raw = "我先把任务计划压缩成当前会话 todo，文档方面只在结论有长期变化时更新公司画像，否则说明无需更新，不落盘到 current-plan。\n\nASTS 最近下跌，核心还是发射节奏、融资预期和风险偏好三件事同时压估值。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "ASTS 最近下跌，核心还是发射节奏、融资预期和风险偏好三件事同时压估值。"
        );
    }

    #[test]
    fn sanitize_user_visible_output_strips_planning_prelude_before_answer() {
        let raw = "我先对齐今天的市场口径，再把软件被压的原因拆成四条线。\n\n核心原因是利率预期、AI capex 分流、企业预算放缓和高估值久期资产折现率上行。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "核心原因是利率预期、AI capex 分流、企业预算放缓和高估值久期资产折现率上行。"
        );
    }

    #[test]
    fn sanitize_user_visible_output_keeps_user_facing_conclusion_prelude() {
        let raw = "我先给结论：软件股被压不是单一基本面恶化，而是利率、预算和 AI 资金偏好的共同作用。\n\n后面再看个股分化。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(!sanitized.removed_internal);
        assert_eq!(sanitized.content, raw);
    }

    #[test]
    fn sanitize_user_visible_output_redacts_local_markdown_file_links() {
        let raw = "PDD 公司画像已建好：主画像 [profile.md](/Users/fengming2/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__secret/company_profiles/pdd/profile.md)，事件 [2026-05-12-init.md](file:///Users/fengming2/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__secret/company_profiles/pdd/events/2026-05-12-init.md)。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "PDD 公司画像已建好：主画像 profile.md，事件 2026-05-12-init.md。"
        );
        assert!(!sanitized.content.contains("/Users/"));
        assert!(!sanitized.content.contains("direct__secret"));
    }

    #[test]
    fn sanitize_user_visible_output_redacts_bare_absolute_paths() {
        let raw = "已写入 /Users/fengming2/Desktop/honeclaw/data/agent-sandboxes/feishu/direct__secret/company_profiles/pdd/profile.md 和 C:\\Users\\fengming\\honeclaw\\secret\\note.txt。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "已写入 <absolute-path>/profile.md 和 <absolute-path>/note.txt。"
        );
        assert!(!sanitized.content.contains("/Users/"));
        assert!(!sanitized.content.contains("C:\\Users"));
        assert!(!sanitized.content.contains("direct__secret"));
    }

    #[test]
    fn sanitize_user_visible_output_redacts_internal_relative_company_profile_paths() {
        let raw =
            "我已把 AVGO 财报前框架沉淀到 company_profiles/AVGO.md，后续财报出来可以直接对照更新。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "我已把 AVGO 财报前框架沉淀到公司画像，后续财报出来可以直接对照更新。"
        );
        assert!(!sanitized.content.contains("company_profiles/"));
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_enabled_boolean_copy() {
        let raw = "这 3 个任务目前都是 `enabled=true`。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(sanitized.content, "这 3 个任务目前均已启用。");
        assert!(!sanitized.content.contains("enabled=true"));
    }

    #[test]
    fn sanitize_user_visible_output_strips_internal_skill_and_storage_copy() {
        let raw = "Hone 的 stock_research 技能名当前没有激活，所以我改用其它技能框架。账本文件已定位到本地 data/portfolio 下，本地 json 文件仍只显示旧仓位。以下以 Hone 持仓工具为准，并补充过去 24 小时的新闻和风险。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "以下以 Hone 持仓工具为准，并补充过去 24 小时的新闻和风险。"
        );
        assert!(!sanitized.content.contains("stock_research"));
        assert!(!sanitized.content.contains("data/portfolio"));
        assert!(!sanitized.content.contains("本地 json"));
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_cron_tool_unavailable_copy() {
        for raw in [
            "本轮未暴露可执行的定时任务创建接口，因此这两个推送任务没有成功创建。任务规格如下。",
            "工具列表里没有 cron_job / scheduled_task 的 list 或 remove 接口，取消动作未完成。",
            "自动定时任务注册工具没有暴露出来，所以我不能确认任务已经正式创建成功。",
            "当前环境没有可用的定时任务写入工具，所以我不能确认“每天20:00自动推送”已经创建成功。",
        ] {
            let sanitized = sanitize_user_visible_output(raw);
            assert!(sanitized.removed_internal, "raw={raw}");
            assert!(
                sanitized
                    .content
                    .contains(CRON_TASK_MANAGEMENT_UNAVAILABLE_USER_MESSAGE),
                "content={}",
                sanitized.content
            );
            assert!(!sanitized.content.contains("工具"));
            assert!(!sanitized.content.contains("接口未暴露"));
            assert!(!sanitized.content.contains("cron_job"));
            assert!(!sanitized.content.contains("scheduled_task"));
        }
    }

    #[test]
    fn sanitize_user_visible_output_strips_cron_storage_self_inspection_copy() {
        let raw = "当前目录只看到 data/sessions.sqlite3，sessions.sqlite3 当前没有可查询的任务表，session_messages 和 session_metadata 也不能代表真实任务。请稍后再试。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(sanitized.content, "请稍后再试。");
        assert!(!sanitized.content.contains("data/sessions.sqlite3"));
        assert!(!sanitized.content.contains("sessions.sqlite3"));
        assert!(!sanitized.content.contains("session_messages"));
        assert!(!sanitized.content.contains("session_metadata"));
    }

    #[test]
    fn sanitize_user_visible_output_strips_portfolio_empty_dir_self_inspection_copy() {
        let raw = "当前沙盒 data/portfolio 是空目录，没有可读 holdings.json，所以我先沿用历史记忆里的持仓版本继续分析。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert!(sanitized.content.is_empty());
        assert!(!sanitized.content.contains("data/portfolio"));
        assert!(!sanitized.content.contains("holdings.json"));
        assert!(!sanitized.content.contains("空目录"));
    }

    #[test]
    fn sanitize_user_visible_output_strips_local_skill_file_unreadable_copy() {
        let raw = "本地技能文件路径不可读，我继续按你给的原文做通俗化改写；这轮不需要实时行情，所以不做价格结论。\n\n这段话的意思是：黄金像一辆正在下坡的车，短线还没完全刹住。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "这段话的意思是：黄金像一辆正在下坡的车，短线还没完全刹住。"
        );
        assert!(!sanitized.content.contains("本地技能"));
        assert!(!sanitized.content.contains("路径不可读"));
    }

    #[test]
    fn sanitize_user_visible_output_strips_image_skill_state_copy() {
        let raw = "我没有拿到可解析附件内容，图片理解工具也没有成功激活。请重新上传或粘贴文字。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(sanitized.content, "请重新上传或粘贴文字。");
        assert!(!sanitized.content.contains("图片理解工具"));

        let only_internal = sanitize_user_visible_output("图片分析技能也没成功加载。");
        assert!(only_internal.only_internal);
        assert!(only_internal.content.is_empty());
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_company_profile_copy_glitches() {
        let raw = "我已为腾讯控股建立长期画像，路径是：\n公司画像公司画像。画像已更新：公司画像公司画像。并把本轮更新补进本地画像：公司画像。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "我已为腾讯控股建立长期画像，已沉淀为公司画像。已沉淀为公司画像。把本轮更新补进公司画像。"
        );
        assert!(!sanitized.content.contains("公司画像公司画像"));
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_company_profile_written_numbered_list() {
        let raw = "微软画像已更新。\n已写入：\n1.公司画像\n2.公司画像";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(sanitized.content, "微软画像已更新。\n已写入公司画像");
        assert!(!sanitized.content.contains("1.公司画像"));
        assert!(!sanitized.content.contains("2.公司画像"));

        let bullet_raw = "ABSI 画像已更新。\n已写入：\n- 1.公司画像\n- 2.公司画像";
        let bullet_sanitized = sanitize_user_visible_output(bullet_raw);
        assert!(bullet_sanitized.removed_internal);
        assert_eq!(bullet_sanitized.content, "ABSI 画像已更新。\n已写入公司画像");
        assert!(!bullet_sanitized.content.contains("1.公司画像"));
        assert!(!bullet_sanitized.content.contains("2.公司画像"));
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_market_data_tool_fallback_copy() {
        let raw =
            "data_fetch 本轮未返回可用结果，已用 StockAnalysis 补充校验。以下是今晚观察池更新。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "主行情源本轮未返回可用结果，已改用公开页面补充校验。以下是今晚观察池更新。"
        );
        assert!(!sanitized.content.contains("data_fetch"));
        assert!(!sanitized.content.contains("StockAnalysis"));
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_market_data_fallback_variants() {
        for (raw, expected) in [
            (
                "data_fetch 当前未返回可用行情，已用 StockAnalysis 实时页补充校验价格与页面显示财报日期。核心股如下。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验。核心股如下。",
            ),
            (
                "本轮使用 StockAnalysis 最新可见美股价格，data_fetch 当前不可用，已用可靠网页源补充校验。核心股如下。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验。核心股如下。",
            ),
            (
                "可用行情接口未返回有效结果，已用 StockAnalysis 页面补充校验；击球区沿用本地固定区间。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验；击球区沿用本地固定区间。",
            ),
            (
                "专用 data_fetch 未返回可用结果，以下改用 StockAnalysis 校验。观察池如下。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验。观察池如下。",
            ),
            (
                "本轮未取得 data_fetch 返回，价格用 StockAnalysis 页面校验；财报日期优先沿用最近一次已校验结果。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验；财报日期优先沿用最近一次已校验结果。",
            ),
            (
                "本轮 23:00 刷新未能取得新的 data_fetch / 网页行情返回；以下沿用本会话 21:35 已校验的 StockAnalysis 最新可见美股价格。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验。",
            ),
            (
                "本轮使用 data_fetch quote 校验；当前为周六晚，对应最新可得美股价格为 2026-06-12 美股收盘附近行情。",
                "主行情源本轮未返回可用结果，已改用公开页面补充校验；当前为周六晚，对应最新可得美股价格为 2026-06-12 美股收盘附近行情。",
            ),
        ] {
            let sanitized = sanitize_user_visible_output(raw);
            assert!(sanitized.removed_internal, "raw={raw}");
            assert_eq!(sanitized.content, expected);
            assert!(!sanitized.content.contains("data_fetch"));
            assert!(!sanitized.content.contains("StockAnalysis"));
        }
    }

    #[test]
    fn sanitize_user_visible_output_rewrites_company_profile_created_copy() {
        let raw = "本轮已新增长期画像：company_profiles/DELL.md。后续可继续补充。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(sanitized.content, "公司画像已更新。后续可继续补充。");
        assert!(!sanitized.content.contains("company_profiles"));
    }

    #[test]
    fn sanitize_user_visible_output_strips_user_info_metadata_and_storage_copy() {
        let raw = "当前会话 ID：Actor_feishu__direct__ou_123。\n我能看到飞书 open_id、chat_id、手机号等元数据。\n当前工作区可见：data/sessions.sqlite3 / company_profiles 目录 / data/cron_jobs 目录 / data/portfolio 目录 / data/notif_prefs 目录 / uploads 目录。\n你当前让我长期跟踪的重点是美股 AI、半导体 和 黄金。";
        let sanitized = sanitize_user_visible_output(raw);
        assert!(sanitized.removed_internal);
        assert_eq!(
            sanitized.content,
            "你当前让我长期跟踪的重点是美股 AI、半导体 和 黄金。"
        );
        assert!(!sanitized.content.contains("会话 ID"));
        assert!(!sanitized.content.contains("open_id"));
        assert!(!sanitized.content.contains("chat_id"));
        assert!(!sanitized.content.contains("data/sessions.sqlite3"));
        assert!(!sanitized.content.contains("company_profiles"));
        assert!(!sanitized.content.contains("uploads"));
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
    fn user_visible_error_message_rewrites_missing_hone_mcp_binary_errors() {
        let err = user_visible_error_message(Some(
            "hone-mcp binary not found near current executable; tried: /tmp/hone-mcp, /tmp/hone-mcp-aarch64-apple-darwin (set HONE_MCP_BIN to override)",
        ));
        assert_eq!(err, RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE);
    }

    #[test]
    fn user_visible_error_message_preserves_wrapped_quota_rejection() {
        let err = user_visible_error_message(Some(
            "工具执行错误: 已达到今日对话上限（12/12，北京时间 2026-05-01），请明天再试",
        ));
        assert_eq!(
            err,
            "已达到今日对话上限（12/12，北京时间 2026-05-01），请明天再试"
        );
    }

    #[test]
    fn user_visible_error_message_maps_codex_usage_limit_errors() {
        let err = user_visible_error_message(Some(
            "codex acp error: You've reached your usage limit. Try again later.",
        ));
        assert_eq!(err, RUNNER_USAGE_LIMIT_USER_ERROR_MESSAGE);
        assert!(!err.contains("codex acp"));
    }

    #[test]
    fn user_visible_error_message_maps_codex_probe_resource_errors() {
        let err = user_visible_error_message(Some(
            "failed to probe codex version via `codex`: Resource temporarily unavailable (os error 35)",
        ));
        assert_eq!(err, RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE);
        assert!(!err.contains("Resource temporarily unavailable"));
        assert!(!err.contains("os error 35"));
    }

    #[test]
    fn user_visible_error_message_maps_acp_transport_disconnect_errors() {
        let err = user_visible_error_message(Some(
            "codex acp error: stream disconnected before completion",
        ));
        assert_eq!(err, RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE);
        assert!(!err.contains("stream disconnected"));
        assert!(!err.contains("codex acp"));
    }

    #[test]
    fn user_visible_error_message_maps_hone_mcp_startup_errors() {
        let err = user_visible_error_message(Some(
            "hone-mcp binary not found near current executable; tried: /private/app/hone-mcp, /private/app/hone-mcp-aarch64-apple-darwin (set HONE_MCP_BIN to override)",
        ));
        assert_eq!(err, RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE);
        assert!(!err.contains("hone-mcp binary"));
        assert!(!err.contains("/private/app"));
        assert!(!err.contains("HONE_MCP_BIN"));
    }

    #[test]
    fn user_visible_error_message_hides_sensitive_error_details() {
        let err = user_visible_error_message(Some(
            "upstream failed OPENROUTER_API_KEY=sk-secret Authorization: Basic basic-secret",
        ));
        assert_eq!(err, GENERIC_USER_ERROR_MESSAGE);
        assert!(!err.contains("sk-secret"));
        assert!(!err.contains("basic-secret"));
    }

    #[test]
    fn user_visible_error_message_or_none_suppresses_internal_acp_errors() {
        let err = user_visible_error_message_or_none(Some(
            "codex acp prompt ended before tool completion: Searching the Web",
        ));
        assert!(err.is_none());
    }

    #[test]
    fn user_visible_error_message_or_none_suppresses_acp_transport_disconnect_errors() {
        let err = user_visible_error_message_or_none(Some(
            "codex acp error: stream disconnected before completion",
        ));
        assert!(err.is_none());
    }

    #[test]
    fn user_visible_error_message_or_none_preserves_quota_rejection() {
        let err = user_visible_error_message_or_none(Some(
            "渠道错误: 已达到今日对话上限（12/12，北京时间 2026-05-01），请明天再试",
        ));
        assert_eq!(
            err.as_deref(),
            Some("已达到今日对话上限（12/12，北京时间 2026-05-01），请明天再试")
        );
    }

    #[test]
    fn user_visible_error_message_or_none_keeps_codex_usage_limit_errors() {
        let err = user_visible_error_message_or_none(Some(
            "LLM 错误: codex runner quota exceeded, please try again later",
        ));
        assert_eq!(err.as_deref(), Some(RUNNER_USAGE_LIMIT_USER_ERROR_MESSAGE));
    }

    #[test]
    fn user_visible_error_message_or_none_keeps_codex_probe_resource_errors_sanitized() {
        let err = user_visible_error_message_or_none(Some(
            "failed to probe codex version via `codex`: Resource temporarily unavailable (os error 35)",
        ));
        assert_eq!(
            err.as_deref(),
            Some(RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE)
        );
    }

    #[test]
    fn user_visible_error_message_or_none_keeps_hone_mcp_startup_errors_sanitized() {
        let err = user_visible_error_message_or_none(Some(
            "hone-mcp binary not found near current executable; tried: /private/app/hone-mcp (set HONE_MCP_BIN to override)",
        ));
        assert_eq!(
            err.as_deref(),
            Some(RUNNER_RESOURCE_UNAVAILABLE_USER_ERROR_MESSAGE)
        );
    }

    #[test]
    fn user_visible_error_message_or_none_suppresses_internal_idle_timeout() {
        let err = user_visible_error_message_or_none(Some(
            "codex acp session/prompt idle timeout (180s)",
        ));
        assert!(err.is_none());
    }

    #[test]
    fn user_visible_error_message_or_none_keeps_generic_timeout_errors() {
        let err = user_visible_error_message_or_none(Some(
            "request timed out while waiting for upstream response",
        ));
        assert_eq!(err.as_deref(), Some(TIMEOUT_USER_ERROR_MESSAGE));
    }

    #[test]
    fn user_visible_error_message_or_none_drops_sensitive_error_details() {
        let err = user_visible_error_message_or_none(Some(
            "gateway rejected x-api-key: header-secret refresh_token=json-refresh",
        ));
        assert!(err.is_none());
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
