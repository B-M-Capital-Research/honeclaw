//! Renderer — 把 MarketEvent 渲染成人可读消息。
//!
//! 排版原则（面向 Telegram / 飞书 / iMessage / Discord 的跨渠道基线）：
//! 1. 头一行：`{【要闻】|【简讯】} {$TICKER…} · {类别}`，Low 不带严重度前缀
//! 2. 标题整行单独成段
//! 3. summary 可空，有就独立一段
//! 4. URL 独立一段；HTML/Markdown 模式下折成可点击锚文本（显示 host）
//! 5. symbol 列表 ≤3 只取前 3，多出部分显示 "+N"
//!
//! 渠道格式差异通过 `RenderFormat` 体现——`Plain` 保留纯文本基线，
//! `TelegramHtml` 用 `<b>…</b>` 与 `<a href>`，`DiscordMarkdown` 用 `**…**` 与 `[text](url)`。

use std::borrow::Cow;

use crate::event::{EventKind, MarketEvent, Severity};

/// 渠道消息格式。Sink 通过 `OutboundSink::format()` 声明自己期望哪种。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderFormat {
    /// 纯文本，适用于 iMessage / 飞书基础文本 / 测试预览。
    #[default]
    Plain,
    /// Telegram `parse_mode=HTML`。
    TelegramHtml,
    /// Discord 消息 Markdown。
    DiscordMarkdown,
    /// Feishu rich text `post` message content JSON.
    FeishuPost,
}

pub fn render_immediate(event: &MarketEvent, fmt: RenderFormat) -> String {
    render_immediate_with_mainline(event, fmt, None)
}

/// 渲染 actor 级即时消息。只有财报质量事件会消费 `mainline`;其他 kind 保持
/// 与 `render_immediate` 完全相同。这里不让模型替用户修改主线，只把已确认的
/// actor 主线与通用财报事实放在同一张快报里，供用户继续核验。
pub fn render_immediate_with_mainline(
    event: &MarketEvent,
    fmt: RenderFormat,
    mainline: Option<&str>,
) -> String {
    if matches!(fmt, RenderFormat::FeishuPost) {
        return render_immediate_feishu_post(event, mainline);
    }

    let tag = severity_tag(event.severity);
    let head = header_line(event);
    let head_plain = if tag.is_empty() {
        head
    } else {
        format!("{tag} {head}")
    };

    let head_out = match fmt {
        RenderFormat::Plain => head_plain.clone(),
        RenderFormat::TelegramHtml => format!("<b>{}</b>", escape_html(&head_plain)),
        RenderFormat::DiscordMarkdown => format!("**{}**", escape_md(&head_plain)),
        RenderFormat::FeishuPost => unreachable!("handled above"),
    };
    let title_out = render_inline(&event.title, fmt);

    let mut out = format!("{head_out}\n{title_out}");

    let body = effective_body_with_mainline(event, mainline);
    let body_trim = body.trim();
    if !body_trim.is_empty() {
        out.push_str("\n\n");
        out.push_str(&render_inline(body_trim, fmt));
    }

    if let Some(u) = event.user_visible_url() {
        out.push_str("\n\n");
        out.push_str(&render_link(u, fmt));
    }
    out
}

/// 选事件正文渲染所用的字符串。
///
/// 默认走 `event.summary`(poller 写的简短描述,通常是一行字段)。但当事件是
/// `EventKind::SecFiling` 且 `payload.llm_summary` 非空时,优先用 LLM 生成的
/// ~200 字业务摘要 —— filing 的 `summary` 字段只是 filing date,信息量近零;
/// 有 LLM 摘要时,filing date 已经在 `occurred_at_ts` 里持久化,不需要再渲染
/// 进 body。
///
/// 失败 / enrichment 关闭 / payload 字段不存在 → fallback 到 `event.summary`。
pub fn effective_body(event: &MarketEvent) -> Cow<'_, str> {
    if matches!(event.kind, EventKind::SecFiling { .. })
        && let Some(summary) = event.normalized_llm_summary()
    {
        return summary;
    }
    Cow::Borrowed(&event.summary)
}

fn effective_body_with_mainline<'a>(
    event: &'a MarketEvent,
    mainline: Option<&str>,
) -> Cow<'a, str> {
    let body = effective_body(event);
    if !matches!(event.kind, EventKind::EarningsReleased)
        || event
            .payload
            .get("earnings_quality_review_applied")
            .and_then(|value| value.as_bool())
            != Some(true)
    {
        return body;
    }

    let conclusion = event
        .payload
        .pointer("/earnings_quality_review/conclusion")
        .and_then(|value| value.as_str())
        .map(earnings_conclusion_label)
        .unwrap_or("待判断");
    let profile_context = mainline
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            format!(
                "你的长期主线（仅用于本次对照，不会自动改写）：{}\n主线初判：财报综合信号为{conclusion}；是否正式强化或削弱主线，仍需按关键因子确认。",
                truncate_chars(value, 240)
            )
        })
        .unwrap_or_else(|| {
            "个性化状态：尚未建立这家公司的用户主线；当前仅为通用事实卡，不能冒充个性化判断。"
                .to_string()
        });

    Cow::Owned(if body.trim().is_empty() {
        profile_context
    } else {
        format!("{}\n\n{profile_context}", body.trim())
    })
}

fn earnings_conclusion_label(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "positive" => "正面",
        "mixed_positive" => "混合偏正",
        "neutral" => "中性",
        "mixed_negative" => "混合偏负",
        "negative" => "负面",
        _ => "待判断",
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let head = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

fn render_immediate_feishu_post(event: &MarketEvent, mainline: Option<&str>) -> String {
    let tag = severity_tag(event.severity);
    let head = header_line(event);
    let head_plain = if tag.is_empty() {
        head
    } else {
        format!("{tag} {head}")
    };

    let mut content = Vec::new();
    let mut title_row = vec![feishu_text(&event.title)];
    if let Some(url) = event.user_visible_url() {
        title_row.push(feishu_text(" · "));
        title_row.push(feishu_link_icon(url));
    }
    content.push(title_row);

    let body = effective_body_with_mainline(event, mainline);
    let body_trim = body.trim();
    if !body_trim.is_empty() {
        content.push(vec![feishu_text(body_trim)]);
    }

    serde_json::json!({
        "zh_cn": {
            "title": head_plain,
            "content": content,
        }
    })
    .to_string()
}

/// High → "【要闻】"、Medium → "【简讯】"、Low → ""（无前缀）。
pub fn severity_tag(s: Severity) -> &'static str {
    match s {
        Severity::High => "【要闻】",
        Severity::Medium => "【简讯】",
        Severity::Low => "",
    }
}

/// 头部行：有 symbol 时 `$AAPL · 📊 财报发布`；无 symbol 时只留类别。
pub fn header_line(event: &MarketEvent) -> String {
    let label = kind_label(&event.kind);
    match compact_symbols(&event.symbols) {
        Some(sym) => format!("{sym} · {label}"),
        None => label,
    }
}

/// 摘要条目里用的紧凑头：`$AAPL [财报]`；无 symbol 时只给标签。
pub fn header_line_compact(event: &MarketEvent) -> String {
    let label = kind_short(&event.kind);
    match (compact_symbols(&event.symbols), label) {
        (Some(sym), Some(lab)) => format!("{sym} {lab}"),
        (Some(sym), None) => sym,
        (None, Some(lab)) => lab,
        (None, None) => String::new(),
    }
}

fn compact_symbols(symbols: &[String]) -> Option<String> {
    let clean: Vec<&str> = symbols
        .iter()
        .map(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .collect();
    if clean.is_empty() {
        return None;
    }
    let head: Vec<String> = clean.iter().take(3).map(|s| format!("${s}")).collect();
    let extra = clean.len().saturating_sub(3);
    Some(if extra > 0 {
        format!("{} +{extra}", head.join(" "))
    } else {
        head.join(" ")
    })
}

fn kind_label(kind: &EventKind) -> String {
    match kind {
        EventKind::EarningsUpcoming => "📅 财报预告".into(),
        EventKind::EarningsReleased => "📊 财报发布".into(),
        EventKind::EarningsCallTranscript => "📝 财报纪要".into(),
        EventKind::NewsCritical => "🔔 关键新闻".into(),
        EventKind::PriceAlert { window, .. } => match window.as_str() {
            "close" => "🔔 收盘".into(),
            "pre" => "☀️ 盘前".into(),
            "post" => "🌃 盘后".into(),
            _ => "⚡ 价格异动".into(),
        },
        EventKind::Weekly52High => "📈 52 周新高".into(),
        EventKind::Weekly52Low => "📉 52 周新低".into(),
        EventKind::Dividend => "💵 分红".into(),
        EventKind::Split => "✂️ 拆股".into(),
        EventKind::SecFiling { form } => format!("📄 SEC {form}"),
        EventKind::AnalystGrade => "⭐ 评级变动".into(),
        EventKind::MacroEvent => "🌐 宏观".into(),
        EventKind::SocialPost => "🗣 社交".into(),
    }
}

fn kind_short(kind: &EventKind) -> Option<String> {
    Some(match kind {
        EventKind::EarningsUpcoming => "[财报预告]".into(),
        EventKind::EarningsReleased => "[财报]".into(),
        EventKind::EarningsCallTranscript => "[财报纪要]".into(),
        EventKind::NewsCritical => "[新闻]".into(),
        EventKind::PriceAlert { window, .. } => match window.as_str() {
            "close" => "[收盘]".into(),
            "pre" => "[盘前]".into(),
            "post" => "[盘后]".into(),
            _ => "[价格]".into(),
        },
        EventKind::Weekly52High => "[52W↑]".into(),
        EventKind::Weekly52Low => "[52W↓]".into(),
        EventKind::Dividend => "[分红]".into(),
        EventKind::Split => "[拆股]".into(),
        EventKind::SecFiling { form } => format!("[{form}]"),
        EventKind::AnalystGrade => "[评级]".into(),
        EventKind::MacroEvent => "[宏观]".into(),
        EventKind::SocialPost => "[社交]".into(),
    })
}

// ── 渠道无关的 inline 文本渲染 ─────────────────────────────────────────

/// 按 format 转义 inline 文本（title / summary 等）。
pub fn render_inline(text: &str, fmt: RenderFormat) -> String {
    match fmt {
        RenderFormat::Plain => text.to_string(),
        RenderFormat::TelegramHtml => escape_html(text),
        RenderFormat::DiscordMarkdown => escape_md(text),
        RenderFormat::FeishuPost => text.to_string(),
    }
}

/// 按 format 渲染一个 URL——HTML/Markdown 折叠成显示 host 的锚文本，Plain 裸贴。
pub fn render_link(url: &str, fmt: RenderFormat) -> String {
    match fmt {
        RenderFormat::Plain => url.to_string(),
        RenderFormat::TelegramHtml => format!(
            "<a href=\"{}\">{}</a>",
            escape_html_attr(url),
            escape_html(&link_label(url)),
        ),
        RenderFormat::DiscordMarkdown => {
            format!("[{}]({})", escape_md(&link_label(url)), url)
        }
        RenderFormat::FeishuPost => format!("🔗 {}", link_label(url)),
    }
}

pub fn render_link_icon(url: &str, fmt: RenderFormat) -> String {
    match fmt {
        RenderFormat::Plain => format!("🔗 {}", link_label(url)),
        RenderFormat::TelegramHtml => {
            format!(
                "<a href=\"{}\">{}</a>",
                escape_html_attr(url),
                escape_html(&link_label(url)),
            )
        }
        RenderFormat::DiscordMarkdown => format!("[{}]({url})", escape_md(&link_label(url))),
        RenderFormat::FeishuPost => "🔗".into(),
    }
}

pub(crate) fn link_label(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    without_scheme
        .split('/')
        .next()
        .map(|host| host.strip_prefix("www.").unwrap_or(host))
        .filter(|s| !s.is_empty())
        .unwrap_or(url)
        .to_string()
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_html_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_md(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | '*' | '_' | '~' | '`' | '|' | '>' | '[' | ']' | '(' | ')' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

pub(crate) fn feishu_text(text: &str) -> serde_json::Value {
    serde_json::json!({
        "tag": "text",
        "text": text,
    })
}

pub(crate) fn feishu_link_icon(url: &str) -> serde_json::Value {
    serde_json::json!({
        "tag": "a",
        "text": "🔗",
        "href": url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn event_with_kind(kind: EventKind) -> MarketEvent {
        MarketEvent {
            id: "x".into(),
            kind,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "Q2 results".into(),
            summary: "EPS beat".into(),
            url: Some("https://x.example.com/path".into()),
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn plain_high_starts_with_text_severity_tag() {
        let rendered = render_immediate(
            &event_with_kind(EventKind::EarningsReleased),
            RenderFormat::Plain,
        );
        let first_line = rendered.lines().next().unwrap();
        assert!(
            first_line.starts_with("【要闻】 $AAPL · "),
            "got: {first_line}"
        );
        assert!(first_line.contains("财报发布"));
        assert!(!rendered.contains("🔴"), "不应再带 emoji 颜色球徽标");
        assert!(!rendered.contains("🔗"), "URL 应裸贴，不带 🔗 前缀");
        assert!(rendered.contains("Q2 results"));
        assert!(rendered.contains("EPS beat"));
        assert!(rendered.contains("https://x.example.com/path"));
    }

    #[test]
    fn sec_filing_includes_form_code() {
        let event = event_with_kind(EventKind::SecFiling { form: "8-K".into() });
        let rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(rendered.contains("SEC 8-K"));
    }

    #[test]
    fn sec_filing_prefers_llm_summary_over_event_summary() {
        let mut event = event_with_kind(EventKind::SecFiling {
            form: "10-Q".into(),
        });
        event.summary = "2026-04-20".into();
        event.payload = serde_json::json!({
            "llm_summary": "这份 filing 最值得 长期主线投资者关注的是 GE Vernova 的 backlog 同比增加 25%。"
        });
        let rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(
            rendered.contains("这份 filing 最值得"),
            "expected LLM summary in body, got:\n{rendered}"
        );
        assert!(
            !rendered.contains("2026-04-20"),
            "expected filing date to be replaced by LLM summary, got:\n{rendered}"
        );
    }

    #[test]
    fn sec_filing_unwraps_json_summary_for_every_immediate_channel() {
        let mut event = event_with_kind(EventKind::SecFiling { form: "8-K".into() });
        event.summary = "2026-07-30".into();
        event.payload = serde_json::json!({
            "llm_summary": "{\"summary\":\"这份 filing 最值得关注的是新增订单。\"}"
        });

        for format in [
            RenderFormat::Plain,
            RenderFormat::TelegramHtml,
            RenderFormat::DiscordMarkdown,
            RenderFormat::FeishuPost,
        ] {
            let rendered = render_immediate(&event, format);
            assert!(
                rendered.contains("这份 filing 最值得关注的是新增订单。"),
                "{format:?} should render the unwrapped summary; got:\n{rendered}"
            );
            assert!(
                !rendered.contains("\\\"summary\\\"") && !rendered.contains("\"summary\""),
                "{format:?} should not expose the JSON envelope; got:\n{rendered}"
            );
        }
    }

    #[test]
    fn sec_filing_falls_back_to_summary_when_no_llm_summary() {
        let mut event = event_with_kind(EventKind::SecFiling {
            form: "10-Q".into(),
        });
        event.summary = "2026-04-20".into();
        event.payload = serde_json::Value::Null;
        let rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(
            rendered.contains("2026-04-20"),
            "expected fallback to filing date, got:\n{rendered}"
        );
    }

    #[test]
    fn sec_filing_falls_back_when_llm_summary_blank() {
        let mut event = event_with_kind(EventKind::SecFiling {
            form: "10-Q".into(),
        });
        event.summary = "2026-04-20".into();
        event.payload = serde_json::json!({"llm_summary": "   "});
        let rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(
            rendered.contains("2026-04-20"),
            "blank llm_summary should fallback to summary; got:\n{rendered}"
        );
    }

    #[test]
    fn non_secfiling_ignores_llm_summary_payload() {
        // 防御回归:effective_body 只在 SecFiling 上看 payload.llm_summary,
        // 其他 kind 即使 payload 里有 llm_summary 也应保持原 summary 行为。
        let mut event = event_with_kind(EventKind::EarningsReleased);
        event.summary = "EPS beat".into();
        event.payload = serde_json::json!({"llm_summary": "should not show up"});
        let rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(rendered.contains("EPS beat"));
        assert!(!rendered.contains("should not show up"));
    }

    #[test]
    fn reviewed_earnings_renders_actor_mainline_without_claiming_it_was_rewritten() {
        let mut event = event_with_kind(EventKind::EarningsReleased);
        event.title = "营收与毛利率显著改善".into();
        event.summary = "结论：数据中心驱动增长\n关键证据：收入增长79%；现金流转正\n反向项：消费端环比下降\n尚未确认：量价贡献\n后续核验：电话会核验订单能见度".into();
        event.payload = serde_json::json!({
            "earnings_quality_review_applied": true,
            "earnings_quality_review": {"conclusion": "mixed_positive"}
        });

        for format in [
            RenderFormat::Plain,
            RenderFormat::TelegramHtml,
            RenderFormat::DiscordMarkdown,
            RenderFormat::FeishuPost,
        ] {
            let rendered = render_immediate_with_mainline(
                &event,
                format,
                Some("AI 数据层扩容；重点核验企业级 SSD、客户采用和供给纪律。"),
            );
            assert!(
                rendered.contains("关键证据：收入增长79%"),
                "{format:?}: {rendered}"
            );
            assert!(
                rendered.contains("反向项：消费端环比下降"),
                "{format:?}: {rendered}"
            );
            assert!(
                rendered.contains("尚未确认：量价贡献"),
                "{format:?}: {rendered}"
            );
            assert!(
                rendered.contains("后续核验：电话会核验订单能见度"),
                "{format:?}: {rendered}"
            );
            assert!(rendered.contains("你的长期主线"), "{format:?}: {rendered}");
            assert!(rendered.contains("AI 数据层扩容"), "{format:?}: {rendered}");
            assert!(
                rendered.contains("财报综合信号为混合偏正"),
                "{format:?}: {rendered}"
            );
            assert!(rendered.contains("不会自动改写"), "{format:?}: {rendered}");
        }
    }

    #[test]
    fn reviewed_earnings_without_mainline_declares_generic_fallback() {
        let mut event = event_with_kind(EventKind::EarningsReleased);
        event.payload = serde_json::json!({
            "earnings_quality_review_applied": true,
            "earnings_quality_review": {"conclusion": "positive"}
        });
        let rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(rendered.contains("尚未建立这家公司的用户主线"));
        assert!(rendered.contains("不能冒充个性化判断"));
    }

    #[test]
    fn omits_symbols_cleanly_when_absent() {
        let mut event = event_with_kind(EventKind::MacroEvent);
        event.symbols.clear();
        event.url = None;
        event.summary = String::new();
        let rendered = render_immediate(&event, RenderFormat::Plain);
        let first = rendered.lines().next().unwrap();
        assert!(!first.contains(" · "));
        assert!(first.contains("宏观"));
        assert!(!rendered.contains("$"));
    }

    #[test]
    fn many_symbols_collapse_with_plus_n() {
        let mut event = event_with_kind(EventKind::NewsCritical);
        event.symbols = vec!["AAPL", "MSFT", "NVDA", "TSLA", "GOOG"]
            .into_iter()
            .map(String::from)
            .collect();
        let head = header_line(&event);
        assert!(head.starts_with("$AAPL $MSFT $NVDA +2"), "got: {head}");
    }

    #[test]
    fn compact_header_for_digest_rows() {
        let event = event_with_kind(EventKind::Split);
        let rendered = header_line_compact(&event);
        assert_eq!(rendered, "$AAPL [拆股]");
    }

    #[test]
    fn severity_tags_are_distinct_and_low_is_unprefixed() {
        let mut event = event_with_kind(EventKind::EarningsReleased);
        event.severity = Severity::Medium;
        let medium_rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(medium_rendered.starts_with("【简讯】 "));
        event.severity = Severity::Low;
        let low_rendered = render_immediate(&event, RenderFormat::Plain);
        assert!(
            low_rendered.starts_with("$AAPL · "),
            "Low 不应有前缀，应直接以 cashtag 开头；got: {low_rendered}"
        );
    }

    #[test]
    fn telegram_html_wraps_header_and_anchor_url() {
        let rendered = render_immediate(
            &event_with_kind(EventKind::EarningsReleased),
            RenderFormat::TelegramHtml,
        );
        let first = rendered.lines().next().unwrap();
        assert!(
            first.starts_with("<b>【要闻】 $AAPL · "),
            "头行应包在 <b>…</b>；got: {first}"
        );
        assert!(first.ends_with("</b>"));
        assert!(
            rendered.contains(r#"<a href="https://x.example.com/path">x.example.com</a>"#),
            "URL 应折成 host 锚文本；got: {rendered}"
        );
    }

    #[test]
    fn telegram_html_escapes_dangerous_chars_in_title() {
        let mut event = event_with_kind(EventKind::NewsCritical);
        event.title = "AT&T <div> hack".into();
        event.url = None;
        event.summary = String::new();
        let rendered = render_immediate(&event, RenderFormat::TelegramHtml);
        assert!(rendered.contains("AT&amp;T &lt;div&gt; hack"));
        assert!(!rendered.contains("<div>"));
    }

    #[test]
    fn discord_markdown_uses_bold_and_link_syntax() {
        let rendered = render_immediate(
            &event_with_kind(EventKind::EarningsReleased),
            RenderFormat::DiscordMarkdown,
        );
        let first = rendered.lines().next().unwrap();
        assert!(
            first.starts_with("**【要闻】 $AAPL · ") && first.ends_with("**"),
            "头行应用 **…** 加粗；got: {first}"
        );
        assert!(
            rendered.contains("[x.example.com](https://x.example.com/path)"),
            "URL 应为 Markdown 链接语法；got: {rendered}"
        );
    }

    #[test]
    fn feishu_post_renders_link_icon_element() {
        let rendered = render_immediate(
            &event_with_kind(EventKind::EarningsReleased),
            RenderFormat::FeishuPost,
        );
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(
            parsed
                .pointer("/zh_cn/title")
                .and_then(|v| v.as_str())
                .unwrap(),
            "【要闻】 $AAPL · 📊 财报发布"
        );
        assert_eq!(
            parsed
                .pointer("/zh_cn/content/0/2")
                .and_then(|v| v.get("tag"))
                .and_then(|v| v.as_str()),
            Some("a")
        );
        assert_eq!(
            parsed
                .pointer("/zh_cn/content/0/2")
                .and_then(|v| v.get("text"))
                .and_then(|v| v.as_str()),
            Some("🔗")
        );
    }

    #[test]
    fn immediate_render_omits_unstable_thefly_ajax_url() {
        let mut event = event_with_kind(EventKind::AnalystGrade);
        event.url = Some("https://thefly.com/ajax/news_get.php?id=4357265".into());

        let plain = render_immediate(&event, RenderFormat::Plain);
        assert!(!plain.contains("news_get.php"), "got:\n{plain}");

        event.url = Some("https://news.example.com/path".into());
        let plain = render_immediate(&event, RenderFormat::Plain);
        assert!(plain.contains("https://news.example.com/path"));
    }
}
