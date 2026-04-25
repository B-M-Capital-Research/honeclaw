//! 全局 digest 渲染 —— 把 Pass 2 personalize 的 PersonalizedItem 列表渲染成
//! 发到 sink 的 body。
//!
//! 设计:
//! - 每条带 category emoji 标识(🎯 印证 / ⚠️ 反证 / 🌍 宏观),让用户秒识"为什么我看到这条"
//! - 支持 TelegramHtml(<b><a>) / Plain(纯文本) / DiscordMarkdown(**[ ]())/ FeishuPost(降级 plain)
//! - 长度截断到 ~3800 chars,避免 Telegram 单条 4096 上限被超
//! - 空列表 → 占位文本 + 简短解释,不静默

use crate::global_digest::curator::{PersonalizedItem, PickCategory, ThesisRelation};
use crate::renderer::{RenderFormat, render_inline};

/// Telegram 单条消息上限 4096,留 ~300 给标题/截断说明。
const MAX_BODY_CHARS: usize = 3800;

pub fn render_global_digest(items: &[PersonalizedItem], date: &str, fmt: RenderFormat) -> String {
    if items.is_empty() {
        return render_empty(date, fmt);
    }

    let header = format!("今日全球要闻 · {} 条 · {date}", items.len());
    let header_out = match fmt {
        RenderFormat::TelegramHtml => format!("<b>{}</b>", render_inline(&header, fmt)),
        RenderFormat::DiscordMarkdown => format!("**{}**", render_inline(&header, fmt)),
        _ => header,
    };

    let mut body = String::new();
    body.push_str(&header_out);
    body.push_str("\n\n");

    let mut omitted = 0usize;
    for item in items {
        let block = render_item(item, fmt);
        if body.chars().count() + block.chars().count() > MAX_BODY_CHARS {
            omitted += 1;
            continue;
        }
        body.push_str(&block);
        body.push_str("\n\n");
    }
    if omitted > 0 {
        body.push_str(&format!("…(另 {omitted} 条因长度上限省略)\n"));
    }
    body.trim_end().to_string()
}

fn render_empty(date: &str, fmt: RenderFormat) -> String {
    let head = format!("今日全球要闻 · {date}");
    let body = "今日候选池中没有触达持仓 / 大盘 / 行业级硬料的事件。\n\
                若 thesis 配置较严格,可适当放宽。";
    match fmt {
        RenderFormat::TelegramHtml => format!("<b>{}</b>\n\n{body}", render_inline(&head, fmt)),
        RenderFormat::DiscordMarkdown => format!("**{}**\n\n{body}", render_inline(&head, fmt)),
        _ => format!("{head}\n\n{body}"),
    }
}

fn render_item(item: &PersonalizedItem, fmt: RenderFormat) -> String {
    let label = label_for(item.category, item.thesis_relation);
    let title_inline = render_inline(&item.candidate.event.title, fmt);
    let title_line = match fmt {
        RenderFormat::TelegramHtml => format!("{label} <b>{title_inline}</b>"),
        RenderFormat::DiscordMarkdown => format!("{label} **{title_inline}**"),
        _ => format!("{label} {title_inline}"),
    };

    let comment = render_inline(&item.comment, fmt);
    let symbols = if item.candidate.event.symbols.is_empty() {
        String::new()
    } else {
        format!(" · ${}", item.candidate.event.symbols.join(" $"))
    };
    let source = render_inline(&item.candidate.event.source, fmt);
    let url_line = item
        .candidate
        .event
        .url
        .as_deref()
        .map(|u| match fmt {
            RenderFormat::TelegramHtml => {
                format!("<a href=\"{}\">{}</a>", escape_attr_html(u), short_host(u))
            }
            RenderFormat::DiscordMarkdown => format!("[{}]({u})", short_host(u)),
            _ => u.to_string(),
        })
        .unwrap_or_default();

    let mut out = format!("{title_line}\n   {comment}\n   {source}{symbols}");
    if !url_line.is_empty() {
        out.push_str("\n   ");
        out.push_str(&url_line);
    }
    out
}

fn label_for(cat: PickCategory, rel: ThesisRelation) -> &'static str {
    match (cat, rel) {
        (PickCategory::ThesisAligned, ThesisRelation::Aligned) => "🎯 [印证]",
        (PickCategory::ThesisCounter, _) | (_, ThesisRelation::Counter) => "⚠️ [反证]",
        (PickCategory::MacroFloor, _) => "🌍 [宏观]",
        (PickCategory::ThesisAligned, _) => "📰 [要闻]",
    }
}

fn escape_attr_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn short_host(url: &str) -> String {
    let s = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    s.split('/').next().unwrap_or(s).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::global_digest::collector::GlobalDigestCandidate;
    use crate::global_digest::curator::{PersonalizedItem, PickCategory, ThesisRelation};
    use crate::global_digest::fetcher::{ArticleBody, ArticleSource};
    use crate::pollers::news::NewsSourceClass;
    use chrono::Utc;

    fn item(
        title: &str,
        comment: &str,
        category: PickCategory,
        rel: ThesisRelation,
        rank: u32,
        symbols: Vec<&str>,
    ) -> PersonalizedItem {
        PersonalizedItem {
            candidate: GlobalDigestCandidate {
                event: MarketEvent {
                    id: title.into(),
                    kind: EventKind::NewsCritical,
                    severity: Severity::High,
                    symbols: symbols.into_iter().map(String::from).collect(),
                    occurred_at: Utc::now(),
                    title: title.into(),
                    summary: "".into(),
                    url: Some(format!("https://example.com/{}", title.replace(' ', "-"))),
                    source: "fmp.stock_news:reuters.com".into(),
                    payload: serde_json::json!({}),
                },
                source_class: NewsSourceClass::Trusted,
                fmp_text: "".into(),
                site: "reuters.com".into(),
            },
            article: ArticleBody {
                url: "https://example.com/x".into(),
                text: "body".into(),
                source: ArticleSource::Fetched,
            },
            rank,
            comment: comment.into(),
            category,
            thesis_relation: rel,
        }
    }

    #[test]
    fn renders_empty_with_explanation() {
        let body = render_global_digest(&[], "2026-04-26", RenderFormat::Plain);
        assert!(body.contains("今日全球要闻"));
        assert!(body.contains("没有触达"));
    }

    #[test]
    fn renders_three_categories_with_distinct_labels() {
        let items = vec![
            item(
                "GOOGL Anthropic",
                "印证 Gemini 飞轮",
                PickCategory::ThesisAligned,
                ThesisRelation::Aligned,
                1,
                vec!["GOOGL"],
            ),
            item(
                "Intel turnaround",
                "对 AMD 不构成实质反证",
                PickCategory::ThesisCounter,
                ThesisRelation::Counter,
                2,
                vec!["INTC", "AMD"],
            ),
            item(
                "Macron Hormuz",
                "波及电力叙事",
                PickCategory::MacroFloor,
                ThesisRelation::NotApplicable,
                3,
                vec![],
            ),
        ];
        let body = render_global_digest(&items, "2026-04-26", RenderFormat::Plain);
        assert!(body.contains("🎯 [印证]"));
        assert!(body.contains("⚠️ [反证]"));
        assert!(body.contains("🌍 [宏观]"));
        assert!(body.contains("$GOOGL"));
        assert!(body.contains("$INTC $AMD"));
        // 标题里没 ticker 时不渲染 $ 前缀
        assert!(!body.contains("$Macron"));
    }

    #[test]
    fn telegram_html_escapes_title_and_uses_anchor() {
        let items = vec![item(
            "Apple's iPhone & iPad <update>",
            "回购增加",
            PickCategory::ThesisAligned,
            ThesisRelation::Aligned,
            1,
            vec!["AAPL"],
        )];
        let body = render_global_digest(&items, "2026-04-26", RenderFormat::TelegramHtml);
        assert!(body.contains("<b>"));
        // & < > 必须被转义
        assert!(body.contains("&amp;"));
        assert!(body.contains("&lt;update&gt;"));
        // URL 是 <a href>,且 attr 内的 & < 也应转义
        assert!(body.contains("<a href=\""));
        assert!(body.contains("&amp;-iPad-&lt;update&gt;"));
    }

    #[test]
    fn truncates_when_exceeds_max_chars() {
        // 制造大量长 comment
        let long = "X".repeat(800);
        let items: Vec<_> = (0..10)
            .map(|i| {
                item(
                    &format!("Title {i}"),
                    &long,
                    PickCategory::ThesisAligned,
                    ThesisRelation::Aligned,
                    i as u32 + 1,
                    vec!["AAPL"],
                )
            })
            .collect();
        let body = render_global_digest(&items, "2026-04-26", RenderFormat::Plain);
        assert!(body.contains("因长度上限省略"));
        assert!(body.chars().count() <= MAX_BODY_CHARS + 200);
    }

    #[test]
    fn label_falls_back_when_category_and_relation_dont_match() {
        // ThesisAligned + Neutral → 没有"印证" 关键词,应用通用 [要闻] label
        let items = vec![item(
            "T",
            "c",
            PickCategory::ThesisAligned,
            ThesisRelation::Neutral,
            1,
            vec![],
        )];
        let body = render_global_digest(&items, "2026-04-26", RenderFormat::Plain);
        assert!(body.contains("📰 [要闻]"));
    }

    #[test]
    fn short_host_strips_protocol_and_path() {
        assert_eq!(
            short_host("https://www.reuters.com/foo/bar"),
            "www.reuters.com"
        );
        assert_eq!(short_host("http://example.com"), "example.com");
        assert_eq!(short_host("no-protocol"), "no-protocol");
    }
}
