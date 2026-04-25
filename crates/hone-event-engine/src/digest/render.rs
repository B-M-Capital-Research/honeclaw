//! digest 批量渲染:把一组 `MarketEvent` 拼成单条面向渠道的字符串。
//!
//! 三件事:
//! - 主入口 `render_digest` —— 按 `RenderFormat` 分发(Plain/Telegram/Discord/
//!   FeishuPost),拼 header 行 + `• head · title · 🔗` 条目;
//! - `render_digest_feishu_post` —— 特殊路径,因为飞书是 struct 化 post,需要自己
//!   构造 json;
//! - `digest_event_title` —— SocialPost 截取 `payload.raw_text` 第一段非空行作为
//!   更贴近原文的标题,其它 kind 直接用 `event.title`。

use hone_core::truncate_chars_append;

use crate::event::{EventKind, MarketEvent};

const DIGEST_SOCIAL_TITLE_MAX_CHARS: usize = 240;

/// 渲染 digest 摘要。`label` 由调用方控制(比如 "盘前摘要 · 08:30"),
/// 本函数只负责拼标题头 + 条目行。
///
/// `cap_overflow` 是**单批数量上限截断的条数**。和 curation/topic-memory 砍掉的
/// 那种"明确噪音(opinion_blog 重复 / pr_wire / 同 ticker 第 5 条解读)"不一样:
/// curation 噪音对用户没价值,不需要在 footer 提及。被 `max_items_per_batch`
/// 截断的事件**有内容、只是挤不进当批**,才在 footer 写"另 N 条因数量上限未展示,
/// /missed 查看完整清单"——告诉用户去哪儿看,而不是制造焦虑。
///
/// 格式示例(Plain):
/// ```text
/// 📬 盘前摘要 · 08:30 · 3 条
/// • $NVDA [拆股] · NVDA 宣布 1-for-10 拆股,生效日 2026-05-20
/// • [宏观] · [US] CPI MoM (Mar) · est 0.3 · prev 0.2
/// ```
/// 单条时省略 "· N 条"。`fmt` 控制标题是否加粗、条目文字是否转义。
pub fn render_digest(
    label: &str,
    events: &[MarketEvent],
    cap_overflow: usize,
    fmt: crate::renderer::RenderFormat,
) -> String {
    use crate::renderer::RenderFormat;
    let total = events.len() + cap_overflow;
    let raw_title = if total > 1 {
        format!("📬 {label} · {total} 条")
    } else {
        format!("📬 {label}")
    };
    if matches!(fmt, RenderFormat::FeishuPost) {
        return render_digest_feishu_post(&raw_title, events, cap_overflow);
    }
    let title = match fmt {
        RenderFormat::Plain => raw_title,
        RenderFormat::TelegramHtml => format!(
            "<b>{}</b>",
            crate::renderer::render_inline(&raw_title, RenderFormat::TelegramHtml)
        ),
        RenderFormat::DiscordMarkdown => format!(
            "**{}**",
            crate::renderer::render_inline(&raw_title, RenderFormat::DiscordMarkdown)
        ),
        RenderFormat::FeishuPost => unreachable!("handled above"),
    };
    let mut out = title;
    for ev in events {
        let head = crate::renderer::header_line_compact(ev);
        let display_title = digest_event_title(ev);
        let title_inline = crate::renderer::render_inline(&display_title, fmt);
        let head_inline = crate::renderer::render_inline(&head, fmt);
        let link_inline = ev
            .url
            .as_deref()
            .filter(|u| !u.is_empty())
            .map(|u| crate::renderer::render_link_icon(u, fmt));
        out.push('\n');
        if head_inline.is_empty() {
            out.push_str(&format!("• {title_inline}"));
        } else {
            out.push_str(&format!("• {head_inline} · {title_inline}"));
        }
        if let Some(link_inline) = link_inline {
            out.push_str(" · ");
            out.push_str(&link_inline);
        }
    }
    if cap_overflow > 0 {
        out.push('\n');
        out.push_str(&format!(
            "…… 另 {cap_overflow} 条因数量上限未展示,发送 /missed 查看完整清单"
        ));
    }
    out
}

fn render_digest_feishu_post(
    raw_title: &str,
    events: &[MarketEvent],
    cap_overflow: usize,
) -> String {
    let mut content = Vec::new();
    for ev in events {
        let head = crate::renderer::header_line_compact(ev);
        let display_title = digest_event_title(ev);
        let mut row = Vec::new();
        row.push(crate::renderer::feishu_text("• "));
        if !head.is_empty() {
            row.push(crate::renderer::feishu_text(&head));
            row.push(crate::renderer::feishu_text(" · "));
        }
        row.push(crate::renderer::feishu_text(&display_title));
        if let Some(url) = ev.url.as_deref().filter(|u| !u.is_empty()) {
            row.push(crate::renderer::feishu_text(" · "));
            row.push(crate::renderer::feishu_link_icon(url));
        }
        content.push(row);
    }
    if cap_overflow > 0 {
        content.push(vec![crate::renderer::feishu_text(&format!(
            "…… 另 {cap_overflow} 条因数量上限未展示,发送 /missed 查看完整清单"
        ))]);
    }
    serde_json::json!({
        "zh_cn": {
            "title": raw_title,
            "content": content,
        }
    })
    .to_string()
}

pub(super) fn digest_event_title(event: &MarketEvent) -> String {
    if matches!(event.kind, EventKind::SocialPost) {
        if let Some(first_line) = event
            .payload
            .get("raw_text")
            .and_then(|v| v.as_str())
            .and_then(first_non_empty_line)
        {
            return truncate_chars(first_line, DIGEST_SOCIAL_TITLE_MAX_CHARS);
        }
    }
    event.title.clone()
}

fn first_non_empty_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    truncate_chars_append(text, max_chars.saturating_sub(1), "…")
}
