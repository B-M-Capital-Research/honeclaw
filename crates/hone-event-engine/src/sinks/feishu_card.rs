//! Feishu interactive card 渲染:把 `DigestPayload` 投影成飞书 v2 卡片 JSON。
//!
//! 设计选择:
//! - **header.template** 按 `payload.max_severity` 三档:High=red / Medium=yellow /
//!   Low=blue,与 Discord embed 配色保持语义一致。
//! - **每个非空 KindBucket 一组**:`div` block 标题(emoji header) + `markdown`
//!   block 内嵌 bullet 列表,链接走 `[文本](url)` 飞书原生 markdown 锚文本。
//! - **note element** 放底部"另 N 条因数量上限未展示"提示。
//! - **bucket 间 hr** 分割线,视觉清晰。
//!
//! `build_feishu_card(payload)` 是纯函数 → 返回 card JSON Value。sink 层把它
//! `to_string()` 后丢到 `content` 字段,`msg_type=interactive`。

use serde_json::{Value, json};

use crate::digest::{DigestItem, DigestPayload, group_by_kind_bucket};
use crate::event::Severity;
use crate::renderer::link_label;

pub fn build_feishu_card(payload: &DigestPayload) -> Value {
    let total = payload.total();
    let header_title = if total > 1 {
        format!("📬 {} · {} 条", payload.label, total)
    } else {
        format!("📬 {}", payload.label)
    };
    let template = severity_template(payload.max_severity);

    let mut elements: Vec<Value> = Vec::new();
    let grouped = group_by_kind_bucket(&payload.items);
    let group_count = grouped.len();
    for (idx, (bucket, items)) in grouped.into_iter().enumerate() {
        // bucket header div
        elements.push(json!({
            "tag": "div",
            "text": {
                "tag": "lark_md",
                "content": format!("**{} · {}**", bucket.header_label(), items.len()),
            }
        }));
        // bucket body markdown
        let body = render_bucket_markdown(&items);
        elements.push(json!({
            "tag": "markdown",
            "content": body,
        }));
        if idx + 1 < group_count {
            elements.push(json!({ "tag": "hr" }));
        }
    }

    if payload.cap_overflow > 0 {
        elements.push(json!({
            "tag": "note",
            "elements": [
                {
                    "tag": "plain_text",
                    "content": format!(
                        "另 {} 条因数量上限未展示,发送 /missed 查看完整清单",
                        payload.cap_overflow
                    ),
                }
            ],
        }));
    }

    if elements.is_empty() {
        // payload.items 为空时也要给个占位,防止飞书拒收空 card
        elements.push(json!({
            "tag": "div",
            "text": {
                "tag": "plain_text",
                "content": "本批无可展示内容",
            }
        }));
    }

    json!({
        "config": { "wide_screen_mode": true },
        "header": {
            "template": template,
            "title": {
                "tag": "plain_text",
                "content": header_title,
            }
        },
        "elements": elements,
    })
}

fn render_bucket_markdown(items: &[&DigestItem]) -> String {
    let mut out = String::new();
    for it in items {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("- ");
        if let Some(sym) = &it.primary_symbol {
            out.push_str(&format!("**${sym}** "));
        }
        out.push_str(it.headline.trim());
        if let Some(url) = &it.url {
            // 飞书 lark_md / markdown 都支持 [text](url) 锚
            out.push_str(&format!(" [{}]({url})", link_label(url)));
        }
    }
    out
}

fn severity_template(s: Severity) -> &'static str {
    match s {
        Severity::High => "red",
        Severity::Medium => "yellow",
        Severity::Low => "blue",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::DigestItem;
    use crate::event::EventKind;
    use chrono::Utc;

    fn item(
        kind: EventKind,
        sev: Severity,
        sym: &str,
        headline: &str,
        url: Option<&str>,
    ) -> DigestItem {
        DigestItem {
            id: format!("id:{kind:?}:{sym}:{headline}"),
            kind,
            severity: sev,
            primary_symbol: if sym.is_empty() {
                None
            } else {
                Some(sym.into())
            },
            headline: headline.into(),
            url: url.map(String::from),
            occurred_at: Utc::now(),
            origin: crate::unified_digest::ItemOrigin::Buffered,
            floor: None,
            comment: None,
            thesis_relation: None,
        }
    }

    fn payload_with(
        items: Vec<DigestItem>,
        max_sev: Severity,
        cap_overflow: usize,
    ) -> DigestPayload {
        DigestPayload {
            label: "盘前摘要 · 08:30".into(),
            items,
            cap_overflow,
            max_severity: max_sev,
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn card_template_follows_max_severity() {
        let high = payload_with(
            vec![item(
                EventKind::NewsCritical,
                Severity::High,
                "AAPL",
                "x",
                None,
            )],
            Severity::High,
            0,
        );
        assert_eq!(
            build_feishu_card(&high)["header"]["template"]
                .as_str()
                .unwrap(),
            "red"
        );
        let med = payload_with(vec![], Severity::Medium, 0);
        assert_eq!(
            build_feishu_card(&med)["header"]["template"]
                .as_str()
                .unwrap(),
            "yellow"
        );
        let low = payload_with(vec![], Severity::Low, 0);
        assert_eq!(
            build_feishu_card(&low)["header"]["template"]
                .as_str()
                .unwrap(),
            "blue"
        );
    }

    #[test]
    fn card_groups_buckets_with_hr_dividers() {
        let items = vec![
            item(EventKind::NewsCritical, Severity::High, "AAPL", "n1", None),
            item(
                EventKind::EarningsUpcoming,
                Severity::Medium,
                "GOOGL",
                "e1",
                None,
            ),
            item(EventKind::MacroEvent, Severity::Low, "", "m1", None),
        ];
        let p = payload_with(items, Severity::High, 0);
        let card = build_feishu_card(&p);
        let elements = card["elements"].as_array().unwrap();
        // 3 个 bucket × (header div + markdown) + 2 个 hr 分隔 = 8 个 element
        assert_eq!(elements.len(), 3 * 2 + 2);
        // 中间应有 hr
        assert!(elements.iter().any(|e| e["tag"].as_str() == Some("hr")));
    }

    #[test]
    fn card_includes_overflow_note() {
        let items = vec![item(
            EventKind::NewsCritical,
            Severity::High,
            "AAPL",
            "n1",
            None,
        )];
        let p = payload_with(items, Severity::High, 7);
        let card = build_feishu_card(&p);
        let elements = card["elements"].as_array().unwrap();
        let note = elements
            .iter()
            .find(|e| e["tag"].as_str() == Some("note"))
            .expect("应有 note element");
        let content = note["elements"][0]["content"].as_str().unwrap();
        assert!(content.contains("另 7 条"));
        assert!(content.contains("/missed"));
    }

    #[test]
    fn card_renders_link_source_anchor_in_markdown() {
        let items = vec![item(
            EventKind::NewsCritical,
            Severity::High,
            "MU",
            "Memory rally",
            Some("https://example.com/path"),
        )];
        let p = payload_with(items, Severity::High, 0);
        let card = build_feishu_card(&p);
        let md = card["elements"][1]["content"].as_str().unwrap();
        assert!(
            md.contains("[example.com](https://example.com/path)"),
            "md = {md}"
        );
        assert!(md.contains("**$MU**"));
    }

    #[test]
    fn card_handles_empty_payload() {
        let p = payload_with(vec![], Severity::Low, 0);
        let card = build_feishu_card(&p);
        let elements = card["elements"].as_array().unwrap();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0]["tag"].as_str().unwrap(), "div");
    }
}
