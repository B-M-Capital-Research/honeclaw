//! digest 批量渲染:把一组 `MarketEvent` 拼成单条面向渠道的字符串,
//! 并产出渠道无关的 `DigestPayload` 结构供 sink 选择性升级到富文本。
//!
//! 五件事:
//! - `build_digest_payload` —— 投影 + dedup,产出结构化 `DigestPayload`,无格式
//!   依赖。富文本 sink(Discord embed / Feishu card / Telegram HTML)直接吃
//!   这个 payload 自己渲染。
//! - 主入口 `render_digest` —— 内部 `build_digest_payload` 然后按 `RenderFormat`
//!   分发,拼 header 行 + `• head · title · link` 条目；Plain 保留紧凑来源,
//!   HTML/Markdown 输出 host 锚文本。
//! - `render_digest_feishu_post` —— 特殊路径,因为飞书是 struct 化 post,需要自己
//!   构造 json;
//! - `digest_event_title` —— SocialPost 截取 `payload.raw_text` 第一段非空行作为
//!   更贴近原文的标题,其它 kind 直接用 `event.title`。
//! - `dedup_for_digest` —— 按 (kind, primary_symbol, normalized payload key) 折叠
//!   同语义重复(如同一财报既出现 "in 2 days" 又出现 "on 2026-04-30"),保留首次
//!   出现顺序。

use std::collections::HashSet;

use chrono::{FixedOffset, Utc};
use hone_core::truncate_chars_append;

use crate::event::{EventKind, MarketEvent, Severity};

use super::payload::{DigestItem, DigestPayload, item_from_event};

const DIGEST_SOCIAL_TITLE_MAX_CHARS: usize = 240;

/// 构造渠道无关的 `DigestPayload`。已完成两件 sink 不该重复做的事:
/// 1. 同语义条目去重(`dedup_for_digest`),例:同一财报 T-2 / on date 两条压成一条;
/// 2. 投影成 `DigestItem`,headline 走 `digest_event_title()` 让 SocialPost 取首行。
///
/// `cap_overflow` 沿用 caller 的 `max_items_per_batch` 截断结果(footer 提示用),
/// dedup 不会再额外贡献 overflow —— 去掉的是 footer 也不该提的"同语义重复"。
pub fn build_digest_payload(
    label: impl Into<String>,
    events: &[MarketEvent],
    cap_overflow: usize,
) -> DigestPayload {
    let kept = dedup_for_digest(events);
    let max_severity = kept
        .iter()
        .map(|e| e.severity)
        .max_by_key(|s| s.rank())
        .unwrap_or(Severity::Low);
    let items: Vec<DigestItem> = kept
        .into_iter()
        .map(|e| item_from_event(e, digest_event_title(e)))
        .collect();
    DigestPayload {
        label: label.into(),
        items,
        cap_overflow,
        max_severity,
        generated_at: chrono::Utc::now(),
    }
}

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
    let payload = build_digest_payload(label.to_string(), events, cap_overflow);
    if matches!(fmt, RenderFormat::FeishuPost) {
        return render_digest_feishu_post(label, &payload, events);
    }
    render_digest_text(label, &payload, events, fmt)
}

fn render_digest_text(
    label: &str,
    payload: &DigestPayload,
    events: &[MarketEvent],
    fmt: crate::renderer::RenderFormat,
) -> String {
    use crate::renderer::RenderFormat;
    let total = payload.total();
    let raw_title = if total > 1 {
        format!("📬 {label} · {total} 条")
    } else {
        format!("📬 {label}")
    };
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
        RenderFormat::FeishuPost => unreachable!("handled in render_digest"),
    };
    let kept = dedup_for_digest(events);
    let mut out = title;
    for ev in kept {
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
    let cap_overflow = payload.cap_overflow;
    if cap_overflow > 0 {
        out.push('\n');
        out.push_str(&format!(
            "…… 另 {cap_overflow} 条因数量上限未展示,发送 /missed 查看完整清单"
        ));
    }
    out
}

fn render_digest_feishu_post(
    label: &str,
    payload: &DigestPayload,
    events: &[MarketEvent],
) -> String {
    let total = payload.total();
    let raw_title = if total > 1 {
        format!("📬 {label} · {total} 条")
    } else {
        format!("📬 {label}")
    };
    let mut content = Vec::new();
    let kept = dedup_for_digest(events);
    for ev in kept {
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
    if payload.cap_overflow > 0 {
        content.push(vec![crate::renderer::feishu_text(&format!(
            "…… 另 {} 条因数量上限未展示,发送 /missed 查看完整清单",
            payload.cap_overflow
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
    let title = if matches!(event.kind, EventKind::SocialPost) {
        if let Some(first_line) = event
            .payload
            .get("raw_text")
            .and_then(|v| v.as_str())
            .and_then(first_non_empty_line)
        {
            truncate_chars(first_line, DIGEST_SOCIAL_TITLE_MAX_CHARS)
        } else {
            event.title.clone()
        }
    } else {
        event.title.clone()
    };
    match digest_event_detail(event) {
        Some(detail) if !detail.is_empty() && !title.contains(&detail) => {
            format!("{title} · {detail}")
        }
        _ => title,
    }
}

fn digest_event_detail(event: &MarketEvent) -> Option<String> {
    match event.kind {
        EventKind::MacroEvent => {
            let summary = event.summary.trim();
            if !summary.is_empty() {
                Some(summary.to_string())
            } else {
                let label = if event.occurred_at > Utc::now() {
                    "待公布"
                } else {
                    "时间"
                };
                Some(format!(
                    "{label} {} UTC+8",
                    event
                        .occurred_at
                        .with_timezone(&FixedOffset::east_opt(8 * 3600)?)
                        .format("%m-%d %H:%M")
                ))
            }
        }
        EventKind::EarningsReleased => {
            let summary = event.summary.trim();
            (!summary.is_empty()).then(|| summary.to_string())
        }
        _ => None,
    }
}

fn first_non_empty_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    truncate_chars_append(text, max_chars.saturating_sub(1), "…")
}

/// 同语义重复折叠。保留首次出现顺序——caller(scheduler)已按 `digest_score`
/// 排序,排在前面的"代表"应当胜出。
///
/// dedup key 三段构成:
/// 1. **`(EventKind tag, primary_symbol)`** —— 同公司同类事件才有可能算重复,
///    跨 ticker 永不合并;
/// 2. **kind-specific normalized key** —— `EarningsUpcoming` 用 `payload.report_date`
///    把 "T-3"/"T-2"/"T-1"/"on date" 4 条折成 1 条;`NewsCritical`
///    取 `url` 的 `host+path` 归一化合并多源转载;其它 kind 用 `event.id`。
///
/// 设计选择:不做 LLM 标题相似度去重——成本太高。仅按"明显语义同一"的硬规则压。
pub(super) fn dedup_for_digest(events: &[MarketEvent]) -> Vec<&MarketEvent> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<&MarketEvent> = Vec::with_capacity(events.len());
    for ev in events {
        let key = dedup_key(ev);
        if seen.insert(key) {
            out.push(ev);
        }
    }
    out
}

fn dedup_key(ev: &MarketEvent) -> String {
    let kind_tag = kind_tag(&ev.kind);
    let symbol = ev
        .symbols
        .iter()
        .find(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .unwrap_or_default();
    let normalized = match &ev.kind {
        EventKind::EarningsUpcoming => ev
            .payload
            .get("report_date")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| ev.id.clone()),
        EventKind::NewsCritical => ev
            .url
            .as_deref()
            .filter(|u| !u.is_empty())
            .map(canonicalize_url)
            .unwrap_or_else(|| ev.id.clone()),
        _ => ev.id.clone(),
    };
    format!("{kind_tag}|{symbol}|{normalized}")
}

fn kind_tag(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::EarningsUpcoming => "earnings_upcoming",
        EventKind::EarningsReleased => "earnings_released",
        EventKind::EarningsCallTranscript => "earnings_transcript",
        EventKind::NewsCritical => "news",
        EventKind::PriceAlert { .. } => "price",
        EventKind::Weekly52High => "week_high",
        EventKind::Weekly52Low => "week_low",
        EventKind::Dividend => "dividend",
        EventKind::Split => "split",
        EventKind::SecFiling { .. } => "sec",
        EventKind::AnalystGrade => "grade",
        EventKind::MacroEvent => "macro",
        EventKind::SocialPost => "social",
    }
}

/// URL 归一化:取 `host` + `path`,丢弃 scheme/query/fragment。同一篇报道在
/// `?utm_source=` / `#section` 不同的链接里就能合并掉。
fn canonicalize_url(url: &str) -> String {
    let no_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let no_fragment = no_scheme.split('#').next().unwrap_or(no_scheme);
    let no_query = no_fragment.split('?').next().unwrap_or(no_fragment);
    no_query.trim_end_matches('/').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn ev(kind: EventKind, sev: Severity) -> MarketEvent {
        MarketEvent {
            id: format!(
                "id:{}:{}",
                kind_tag(&kind),
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ),
            kind,
            severity: sev,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "t".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn dedup_collapses_same_earnings_report_date() {
        let mut e1 = ev(EventKind::EarningsUpcoming, Severity::Medium);
        e1.id = "earnings:AAPL:T-2".into();
        e1.title = "AAPL earnings in 2 days (2026-04-30)".into();
        e1.payload = serde_json::json!({ "report_date": "2026-04-30" });
        let mut e2 = ev(EventKind::EarningsUpcoming, Severity::Medium);
        e2.id = "earnings:AAPL:on-date".into();
        e2.title = "AAPL earnings on 2026-04-30".into();
        e2.payload = serde_json::json!({ "report_date": "2026-04-30" });
        let events = vec![e1.clone(), e2.clone()];
        let kept = dedup_for_digest(&events);
        assert_eq!(kept.len(), 1, "同 report_date 的 earnings 应折叠");
        assert_eq!(kept[0].id, "earnings:AAPL:T-2", "应保留第一条");
    }

    #[test]
    fn dedup_keeps_distinct_earnings_for_different_tickers() {
        let mut e1 = ev(EventKind::EarningsUpcoming, Severity::Medium);
        e1.payload = serde_json::json!({ "report_date": "2026-04-30" });
        let mut e2 = ev(EventKind::EarningsUpcoming, Severity::Medium);
        e2.symbols = vec!["GOOGL".into()];
        e2.payload = serde_json::json!({ "report_date": "2026-04-30" });
        let events = vec![e1, e2];
        let kept = dedup_for_digest(&events);
        assert_eq!(kept.len(), 2, "不同 ticker 不应合并");
    }

    #[test]
    fn dedup_collapses_same_news_url_with_different_query() {
        let mut e1 = ev(EventKind::NewsCritical, Severity::High);
        e1.id = "news:1".into();
        e1.url = Some(
            "https://www.cnbc.com/2026/04/27/micron-and-sandisk.html?utm_source=twitter".into(),
        );
        let mut e2 = ev(EventKind::NewsCritical, Severity::High);
        e2.id = "news:2".into();
        e2.url = Some("https://www.cnbc.com/2026/04/27/micron-and-sandisk.html#top".into());
        let events = vec![e1, e2];
        let kept = dedup_for_digest(&events);
        assert_eq!(kept.len(), 1, "同一篇文章不同 query/fragment 应合并");
    }

    #[test]
    fn dedup_keeps_order_for_unique_events() {
        let e1 = ev(EventKind::NewsCritical, Severity::High);
        let mut e2 = ev(
            EventKind::PriceAlert {
                pct_change_bps: 600,
                window: "1d".into(),
            },
            Severity::Medium,
        );
        e2.id = "p1".into();
        let e3 = ev(EventKind::EarningsUpcoming, Severity::Medium);
        let events = vec![e1.clone(), e2.clone(), e3.clone()];
        let kept = dedup_for_digest(&events);
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].id, e1.id);
        assert_eq!(kept[1].id, e2.id);
        assert_eq!(kept[2].id, e3.id);
    }

    #[test]
    fn build_payload_picks_max_severity() {
        let mut low = ev(EventKind::SocialPost, Severity::Low);
        low.id = "s:1".into();
        let mut med = ev(EventKind::EarningsUpcoming, Severity::Medium);
        med.id = "e:1".into();
        let mut high = ev(EventKind::NewsCritical, Severity::High);
        high.id = "n:1".into();
        let events = vec![low, med, high];
        let p = build_digest_payload("test", &events, 0);
        assert_eq!(p.max_severity, Severity::High);
        assert_eq!(p.items.len(), 3);
        assert_eq!(p.cap_overflow, 0);
    }

    #[test]
    fn canonicalize_drops_scheme_query_fragment_and_lowercases() {
        assert_eq!(
            canonicalize_url("https://Example.com/Path/A?x=1#frag"),
            "example.com/path/a"
        );
        assert_eq!(canonicalize_url("http://example.com/p/"), "example.com/p");
    }
}
