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
}

pub fn render_immediate(event: &MarketEvent, fmt: RenderFormat) -> String {
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
    };
    let title_out = render_inline(&event.title, fmt);

    let mut out = format!("{head_out}\n{title_out}");

    let summary_trim = event.summary.trim();
    if !summary_trim.is_empty() {
        out.push_str("\n\n");
        out.push_str(&render_inline(summary_trim, fmt));
    }

    if let Some(u) = event.url.as_deref().filter(|u| !u.is_empty()) {
        out.push_str("\n\n");
        out.push_str(&render_link(u, fmt));
    }
    out
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
        EventKind::NewsCritical => "🔔 关键新闻".into(),
        EventKind::PressRelease => "📣 公司公告".into(),
        EventKind::PriceAlert { .. } => "⚡ 价格异动".into(),
        EventKind::Weekly52High => "📈 52 周新高".into(),
        EventKind::Weekly52Low => "📉 52 周新低".into(),
        EventKind::VolumeSpike => "🔊 成交异动".into(),
        EventKind::Dividend => "💵 分红".into(),
        EventKind::Split => "✂️ 拆股".into(),
        EventKind::Buyback => "♻️ 回购".into(),
        EventKind::SecFiling { form } => format!("📄 SEC {form}"),
        EventKind::AnalystGrade => "⭐ 评级变动".into(),
        EventKind::MacroEvent => "🌐 宏观".into(),
        EventKind::PortfolioPreMarket => "🌅 盘前速览".into(),
        EventKind::PortfolioPostMarket => "🌇 盘后速览".into(),
    }
}

fn kind_short(kind: &EventKind) -> Option<String> {
    Some(match kind {
        EventKind::EarningsUpcoming => "[财报预告]".into(),
        EventKind::EarningsReleased => "[财报]".into(),
        EventKind::NewsCritical => "[新闻]".into(),
        EventKind::PressRelease => "[公告]".into(),
        EventKind::PriceAlert { .. } => "[价格]".into(),
        EventKind::Weekly52High => "[52W↑]".into(),
        EventKind::Weekly52Low => "[52W↓]".into(),
        EventKind::VolumeSpike => "[量能]".into(),
        EventKind::Dividend => "[分红]".into(),
        EventKind::Split => "[拆股]".into(),
        EventKind::Buyback => "[回购]".into(),
        EventKind::SecFiling { form } => format!("[{form}]"),
        EventKind::AnalystGrade => "[评级]".into(),
        EventKind::MacroEvent => "[宏观]".into(),
        _ => return None,
    })
}

// ── 渠道无关的 inline 文本渲染 ─────────────────────────────────────────

/// 按 format 转义 inline 文本（title / summary 等）。
pub fn render_inline(text: &str, fmt: RenderFormat) -> String {
    match fmt {
        RenderFormat::Plain => text.to_string(),
        RenderFormat::TelegramHtml => escape_html(text),
        RenderFormat::DiscordMarkdown => escape_md(text),
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
    }
}

fn link_label(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    without_scheme
        .split('/')
        .next()
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample(kind: EventKind) -> MarketEvent {
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
        let s = render_immediate(&sample(EventKind::EarningsReleased), RenderFormat::Plain);
        let first_line = s.lines().next().unwrap();
        assert!(
            first_line.starts_with("【要闻】 $AAPL · "),
            "got: {first_line}"
        );
        assert!(first_line.contains("财报发布"));
        assert!(!s.contains("🔴"), "不应再带 emoji 颜色球徽标");
        assert!(!s.contains("🔗"), "URL 应裸贴，不带 🔗 前缀");
        assert!(s.contains("Q2 results"));
        assert!(s.contains("EPS beat"));
        assert!(s.contains("https://x.example.com/path"));
    }

    #[test]
    fn sec_filing_includes_form_code() {
        let ev = sample(EventKind::SecFiling {
            form: "8-K".into(),
        });
        let s = render_immediate(&ev, RenderFormat::Plain);
        assert!(s.contains("SEC 8-K"));
    }

    #[test]
    fn omits_symbols_cleanly_when_absent() {
        let mut ev = sample(EventKind::MacroEvent);
        ev.symbols.clear();
        ev.url = None;
        ev.summary = String::new();
        let s = render_immediate(&ev, RenderFormat::Plain);
        let first = s.lines().next().unwrap();
        assert!(!first.contains(" · "));
        assert!(first.contains("宏观"));
        assert!(!s.contains("$"));
    }

    #[test]
    fn many_symbols_collapse_with_plus_n() {
        let mut ev = sample(EventKind::NewsCritical);
        ev.symbols = vec!["AAPL", "MSFT", "NVDA", "TSLA", "GOOG"]
            .into_iter()
            .map(String::from)
            .collect();
        let head = header_line(&ev);
        assert!(head.starts_with("$AAPL $MSFT $NVDA +2"), "got: {head}");
    }

    #[test]
    fn compact_header_for_digest_rows() {
        let ev = sample(EventKind::Split);
        let s = header_line_compact(&ev);
        assert_eq!(s, "$AAPL [拆股]");
    }

    #[test]
    fn severity_tags_are_distinct_and_low_is_unprefixed() {
        let mut ev = sample(EventKind::EarningsReleased);
        ev.severity = Severity::Medium;
        let s_med = render_immediate(&ev, RenderFormat::Plain);
        assert!(s_med.starts_with("【简讯】 "));
        ev.severity = Severity::Low;
        let s_low = render_immediate(&ev, RenderFormat::Plain);
        assert!(
            s_low.starts_with("$AAPL · "),
            "Low 不应有前缀，应直接以 cashtag 开头；got: {s_low}"
        );
    }

    #[test]
    fn telegram_html_wraps_header_and_anchor_url() {
        let s = render_immediate(&sample(EventKind::EarningsReleased), RenderFormat::TelegramHtml);
        let first = s.lines().next().unwrap();
        assert!(
            first.starts_with("<b>【要闻】 $AAPL · "),
            "头行应包在 <b>…</b>；got: {first}"
        );
        assert!(first.ends_with("</b>"));
        assert!(
            s.contains(r#"<a href="https://x.example.com/path">x.example.com</a>"#),
            "URL 应折成 host 锚文本；got: {s}"
        );
    }

    #[test]
    fn telegram_html_escapes_dangerous_chars_in_title() {
        let mut ev = sample(EventKind::NewsCritical);
        ev.title = "AT&T <div> hack".into();
        ev.url = None;
        ev.summary = String::new();
        let s = render_immediate(&ev, RenderFormat::TelegramHtml);
        assert!(s.contains("AT&amp;T &lt;div&gt; hack"));
        assert!(!s.contains("<div>"));
    }

    #[test]
    fn discord_markdown_uses_bold_and_link_syntax() {
        let s = render_immediate(
            &sample(EventKind::EarningsReleased),
            RenderFormat::DiscordMarkdown,
        );
        let first = s.lines().next().unwrap();
        assert!(
            first.starts_with("**【要闻】 $AAPL · ") && first.ends_with("**"),
            "头行应用 **…** 加粗；got: {first}"
        );
        assert!(
            s.contains("[x.example.com](https://x.example.com/path)"),
            "URL 应为 Markdown 链接语法；got: {s}"
        );
    }
}
