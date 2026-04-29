//! Discord embed 渲染:把 `DigestPayload` 投影成 Discord webhook/bot 接受的
//! `embeds` JSON 数组,外加消息级 flags(SUPPRESS_EMBEDS=4) 抑制自动 URL unfurl。
//!
//! 设计选择:
//! - **单条 message + 单个 embed**:digest 语义上就是"一次推送",不拆分。
//! - **embed.title** 用 `📬 {label} · N 条`,与现有 plain/Telegram 字符串首行一致。
//! - **embed.color** 按 payload.max_severity:High=0xE74C3C(红) / Medium=0xF1C40F
//!   (黄) / Low=0x5865F2(Blurple),让用户扫一眼就知道这批有没有要紧条目。
//! - **embed.fields**:每个非空 KindBucket 一个 inline=false field,name 是 emoji
//!   header + bucket 内 dedup 后的条目数,value 是逐条 markdown 文本。
//! - **链接处理**:每条尾部追加 ` [→](url)`(短锚文本),`flags=4` 抑制消息文字
//!   里 URL 的 unfurl 大卡片;embed 内手填 link 不受影响。
//! - **长度边界**:Discord 单 field value ≤1024,embed 总和 ≤6000——greedy 装箱,
//!   超出在 field 末尾追加 `…还有 N 条`,全局溢出落到 footer。
//!
//! 纯函数 `build_discord_embed_message(payload, now)`,与 reqwest IO 解耦,可单测。

use serde_json::{Value, json};

use crate::digest::{DigestItem, DigestPayload, KindBucket, group_by_kind_bucket};
use crate::event::Severity;

/// Discord 单 field value 字符上限。
const FIELD_VALUE_MAX: usize = 1024;
/// Discord 单 embed 字符总和上限(title+description+fields+footer)。
const EMBED_TOTAL_MAX: usize = 6000;
/// SUPPRESS_EMBEDS bit:抑制消息中 URL 的自动 unfurl 卡片,不影响 embeds 数组。
const FLAG_SUPPRESS_EMBEDS: u64 = 4;

/// 构造完整的 Discord message body —— `{ flags, embeds: [...] }`,直接 POST。
pub fn build_discord_embed_message(
    payload: &DigestPayload,
    now: chrono::DateTime<chrono::Utc>,
) -> Value {
    let embed = build_discord_embed(payload, now);
    json!({
        "flags": FLAG_SUPPRESS_EMBEDS,
        "embeds": [embed],
    })
}

fn build_discord_embed(payload: &DigestPayload, now: chrono::DateTime<chrono::Utc>) -> Value {
    let total = payload.total();
    let title = if total > 1 {
        format!("📬 {} · {} 条", payload.label, total)
    } else {
        format!("📬 {}", payload.label)
    };
    let description = if payload.cap_overflow > 0 {
        format!(
            "共 {} 条事件 · 另 {} 条因数量上限未展示,/missed 查看完整清单",
            total, payload.cap_overflow
        )
    } else if total > 1 {
        format!("共 {total} 条事件")
    } else {
        String::new()
    };
    let color = severity_color(payload.max_severity);

    let mut budget = EMBED_TOTAL_MAX
        .saturating_sub(title.chars().count())
        .saturating_sub(description.chars().count());
    let mut fields: Vec<Value> = Vec::new();

    let grouped = group_by_kind_bucket(&payload.items);
    for (bucket, items) in grouped {
        let (field_name, field_value, consumed) = build_field(bucket, &items, budget);
        if consumed == 0 {
            // 一条都塞不下了 —— 折叠剩余 bucket 为单行,break 出循环
            let remaining: usize = items.len();
            if remaining > 0 {
                let collapsed_name = "📂 其它".to_string();
                let collapsed_value = format!("…还有 {remaining} 条");
                let cost = collapsed_name.chars().count() + collapsed_value.chars().count();
                if budget >= cost {
                    fields.push(json!({
                        "name": collapsed_name,
                        "value": collapsed_value,
                        "inline": false,
                    }));
                }
            }
            break;
        }
        budget = budget.saturating_sub(consumed);
        fields.push(json!({
            "name": field_name,
            "value": field_value,
            "inline": false,
        }));
    }

    let mut embed = json!({
        "title": title,
        "color": color,
        "timestamp": now.to_rfc3339(),
        "footer": { "text": "honeclaw" },
        "fields": fields,
    });
    if !description.is_empty() {
        embed["description"] = json!(description);
    }
    embed
}

/// 为单个 bucket 构造一个 field。返回 `(name, value, total_chars_consumed)`。
/// 当 budget 不足以放下 field name 时返回 `consumed=0`,caller break。
fn build_field(
    bucket: KindBucket,
    items: &[&DigestItem],
    budget: usize,
) -> (String, String, usize) {
    let name = format!("{} · {}", bucket.header_label(), items.len());
    let name_cost = name.chars().count();
    if budget < name_cost + 16 {
        // 16 = 至少能放下一行 "…还有 N 条" 的余量,否则整组放弃
        return (String::new(), String::new(), 0);
    }
    let mut remaining_budget = budget.saturating_sub(name_cost);
    let mut value = String::new();
    let mut emitted = 0usize;
    for it in items {
        let line = render_item_line(it);
        let line_chars = line.chars().count() + 1; // \n
        if value.chars().count() + line_chars > FIELD_VALUE_MAX {
            break;
        }
        if line_chars > remaining_budget {
            break;
        }
        if !value.is_empty() {
            value.push('\n');
        }
        value.push_str(&line);
        remaining_budget = remaining_budget.saturating_sub(line_chars);
        emitted += 1;
    }
    let leftover = items.len().saturating_sub(emitted);
    if leftover > 0 {
        let tail = format!("\n…还有 {leftover} 条");
        let tail_chars = tail.chars().count();
        if value.chars().count() + tail_chars <= FIELD_VALUE_MAX && tail_chars <= remaining_budget {
            value.push_str(&tail);
        }
    }
    let consumed = name_cost + value.chars().count();
    (name, value, consumed)
}

/// 单条 item 渲染:`• **$AAPL** {headline} [→](url)`。
/// `headline` 不做 markdown 转义——Discord embed 里 markdown 控制字符当字面量也
/// 不会泄露成格式,且转义会让财报标题里的 `*` 之类变难看。链接锚文本只有 `→`,
/// 避免长 URL 在 embed 内霸屏。
fn render_item_line(it: &DigestItem) -> String {
    let mut out = String::from("• ");
    if let Some(sym) = &it.primary_symbol {
        out.push_str(&format!("**${sym}** "));
    }
    out.push_str(it.headline.trim());
    if let Some(url) = &it.url {
        out.push_str(&format!(" [→]({url})"));
    }
    out
}

fn severity_color(s: Severity) -> u32 {
    match s {
        Severity::High => 0xE7_4C_3C,   // 红
        Severity::Medium => 0xF1_C4_0F, // 黄
        Severity::Low => 0x58_65_F2,    // Blurple
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
    fn embed_carries_suppress_embeds_flag() {
        let p = payload_with(
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
        let msg = build_discord_embed_message(&p, Utc::now());
        assert_eq!(msg["flags"].as_u64().unwrap(), 4);
    }

    #[test]
    fn embed_color_follows_max_severity() {
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
        let high_msg = build_discord_embed_message(&high, Utc::now());
        assert_eq!(
            high_msg["embeds"][0]["color"].as_u64().unwrap() as u32,
            0xE7_4C_3C
        );
        let med = payload_with(vec![], Severity::Medium, 0);
        let med_msg = build_discord_embed_message(&med, Utc::now());
        assert_eq!(
            med_msg["embeds"][0]["color"].as_u64().unwrap() as u32,
            0xF1_C4_0F
        );
        let low = payload_with(vec![], Severity::Low, 0);
        let low_msg = build_discord_embed_message(&low, Utc::now());
        assert_eq!(
            low_msg["embeds"][0]["color"].as_u64().unwrap() as u32,
            0x58_65_F2
        );
    }

    #[test]
    fn embed_groups_items_into_buckets() {
        let items = vec![
            item(EventKind::NewsCritical, Severity::High, "AAPL", "n1", None),
            item(
                EventKind::EarningsUpcoming,
                Severity::Medium,
                "GOOGL",
                "e1",
                None,
            ),
            item(
                EventKind::PriceAlert {
                    pct_change_bps: 100,
                    window: "1d".into(),
                },
                Severity::Low,
                "NUAI",
                "p1",
                None,
            ),
            item(EventKind::MacroEvent, Severity::Low, "", "m1", None),
        ];
        let p = payload_with(items, Severity::High, 0);
        let msg = build_discord_embed_message(&p, Utc::now());
        let fields = msg["embeds"][0]["fields"].as_array().unwrap();
        // 4 个 bucket: Price, Earnings, NewsFiling, Macro
        assert_eq!(fields.len(), 4);
        // 第一个 field 应是价格异动(KindBucket 排序最前)
        assert!(fields[0]["name"].as_str().unwrap().contains("价格异动"));
        assert!(fields[0]["value"].as_str().unwrap().contains("$NUAI"));
        // 财报组里有 GOOGL
        assert!(fields[1]["name"].as_str().unwrap().contains("财报"));
        assert!(fields[1]["value"].as_str().unwrap().contains("$GOOGL"));
    }

    #[test]
    fn embed_includes_overflow_in_description() {
        let items = vec![item(
            EventKind::NewsCritical,
            Severity::High,
            "AAPL",
            "n1",
            None,
        )];
        let p = payload_with(items, Severity::High, 5);
        let msg = build_discord_embed_message(&p, Utc::now());
        let desc = msg["embeds"][0]["description"].as_str().unwrap();
        assert!(desc.contains("另 5 条因数量上限未展示"));
        assert!(desc.contains("/missed"));
    }

    #[test]
    fn embed_renders_link_arrow_anchor() {
        let items = vec![item(
            EventKind::NewsCritical,
            Severity::High,
            "MU",
            "Memory rally continues",
            Some("https://www.cnbc.com/2026/04/27/memory.html"),
        )];
        let p = payload_with(items, Severity::High, 0);
        let msg = build_discord_embed_message(&p, Utc::now());
        let value = msg["embeds"][0]["fields"][0]["value"].as_str().unwrap();
        assert!(
            value.contains("[→](https://www.cnbc.com/2026/04/27/memory.html)"),
            "应使用箭头锚文本,value = {value}"
        );
    }

    #[test]
    fn embed_truncates_field_value_under_1024() {
        // 30 条长 headline,确保单 bucket 撑爆 1024
        let items: Vec<DigestItem> = (0..30)
            .map(|i| {
                item(
                    EventKind::NewsCritical,
                    Severity::High,
                    "AAPL",
                    &format!(
                        "very long headline number {i:03} with extra padding text padding padding"
                    ),
                    None,
                )
            })
            .collect();
        let p = payload_with(items, Severity::High, 0);
        let msg = build_discord_embed_message(&p, Utc::now());
        let value = msg["embeds"][0]["fields"][0]["value"].as_str().unwrap();
        assert!(
            value.chars().count() <= FIELD_VALUE_MAX,
            "value 超长:{}",
            value.chars().count()
        );
        assert!(value.contains("…还有"), "应有溢出尾行,value = {value}");
    }

    #[test]
    fn embed_total_under_6000_for_huge_payload() {
        let items: Vec<DigestItem> = (0..100)
            .map(|i| {
                item(
                    EventKind::NewsCritical,
                    Severity::High,
                    "AAPL",
                    &format!("padding padding padding very long headline {i:03} more padding"),
                    Some("https://example.com/very/long/path/that/takes/space"),
                )
            })
            .collect();
        let p = payload_with(items, Severity::High, 0);
        let msg = build_discord_embed_message(&p, Utc::now());
        let total: usize = serde_json::to_string(&msg["embeds"][0])
            .unwrap()
            .chars()
            .count();
        // JSON 编码后字符数包含 key/quote 等噪音,实际 embed 内部文字应 <6000
        let title = msg["embeds"][0]["title"].as_str().unwrap_or("");
        let desc = msg["embeds"][0]["description"].as_str().unwrap_or("");
        let fields_chars: usize = msg["embeds"][0]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                f["name"].as_str().unwrap_or("").chars().count()
                    + f["value"].as_str().unwrap_or("").chars().count()
            })
            .sum();
        let visible = title.chars().count() + desc.chars().count() + fields_chars;
        assert!(visible <= EMBED_TOTAL_MAX, "可见字符 {visible} 超 6000");
        let _ = total;
    }
}
