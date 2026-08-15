//! 价格档「阶梯合流」——同一波行情的多条 band 事件坍缩成一行。
//!
//! ## 背景（2026-08 SNDK/NBIS 审计）
//!
//! 盘中每跨一个 ±2% band 会产出一条独立 `price_band:*` 事件。用户勿扰时段
//! （北京 23:00–07:30）覆盖美股主交易时段，所以大涨日的整串 band 事件全部
//! `quiet_held`，在 `run_quiet_flush` 复活时逐条进入早报——2026-08-13 SNDK
//! 一条 digest 里出现 5 行「跨过 +8%/+10%/…/+16% 档」加一行收盘 +13.67%，
//! 六行说同一件事。审计显示 18 天内 141 条 band 行有 70 条是这类冗余。
//!
//! 普通 slot digest 不受此害：`DigestBuffer::price_digest_key` 已做
//! 同 symbol 同日 latest-wins 去重。**这里专门服务 quiet_flush 的复活路径**
//! （store `list_quiet_held_since` 绕开 buffer，没有任何去重）。
//!
//! ## 规则
//!
//! - 同 (symbol, 交易日, 方向) 的 band 组只留 |bps| 最大的一条，其余进 omitted；
//! - 若同 (symbol, 交易日) 还有 `price_close` 收盘事件，band 代表也并入收盘行，
//!   收盘标题追加「（盘中曾跨 +X% 档）」注记；
//! - 其他事件（含 pre/post extended、52 周高低、非价格事件）原样保留，顺序不变。

use std::collections::HashMap;

use crate::digest::curation::DigestCuration;
use crate::event::{EventKind, MarketEvent};

/// band 事件的归组 key。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LadderKey {
    symbol: String,
    trade_date: String,
    direction: String,
}

pub(crate) fn coalesce_price_alerts(events: Vec<MarketEvent>) -> DigestCuration {
    // 第一遍：找出每个阶梯组的代表（|bps| 最大）与每个 (symbol, date) 的收盘事件。
    let mut ladder_best: HashMap<LadderKey, (usize, i64)> = HashMap::new();
    let mut close_index: HashMap<(String, String), usize> = HashMap::new();
    for (idx, event) in events.iter().enumerate() {
        if let Some((key, bps)) = intraday_band_key(event) {
            let entry = ladder_best.entry(key).or_insert((idx, bps));
            if bps.abs() > entry.1.abs() {
                *entry = (idx, bps);
            }
        } else if let Some(sym_date) = close_key(event) {
            close_index.entry(sym_date).or_insert(idx);
        }
    }

    // 收盘吸收：同 (symbol, date) 有收盘事件时，band 代表也不单独成行。
    // 一个 symbol-day 可能同时有 up/down 两个方向的组（冲高回落），全部并入注记。
    let mut absorbed_by_close: HashMap<usize, Vec<i64>> = HashMap::new();
    let mut representatives: Vec<usize> = Vec::new();
    for (key, (idx, bps)) in &ladder_best {
        let sym_date = (key.symbol.clone(), key.trade_date.clone());
        if let Some(close_idx) = close_index.get(&sym_date) {
            absorbed_by_close.entry(*close_idx).or_default().push(*bps);
        } else {
            representatives.push(*idx);
        }
    }

    let representative_set: std::collections::HashSet<usize> =
        representatives.into_iter().collect();
    let mut kept = Vec::with_capacity(events.len());
    let mut omitted = Vec::new();
    for (idx, event) in events.into_iter().enumerate() {
        let is_band = intraday_band_key(&event).is_some();
        if is_band && !representative_set.contains(&idx) {
            omitted.push(event);
            continue;
        }
        if let Some(bands) = absorbed_by_close.get(&idx) {
            kept.push(annotate_close_with_bands(event, bands));
            continue;
        }
        kept.push(event);
    }
    DigestCuration { kept, omitted }
}

/// 盘中 band 事件 → (归组 key, 带方向 bps)。只认 `price_band:` id 前缀，
/// 解析失败回退到 payload/kind 字段；close/pre/post 与非价格事件返回 None。
fn intraday_band_key(event: &MarketEvent) -> Option<(LadderKey, i64)> {
    let EventKind::PriceAlert {
        pct_change_bps,
        window,
    } = &event.kind
    else {
        return None;
    };
    if window != "day" || !event.id.starts_with("price_band:") {
        return None;
    }
    // id 形如 price_band:SYM:2026-08-13:up:1600
    let parts: Vec<&str> = event.id.split(':').collect();
    let (symbol, trade_date, direction) = if parts.len() >= 5 {
        (
            parts[1].to_ascii_uppercase(),
            parts[2].to_string(),
            parts[3].to_string(),
        )
    } else {
        (
            event.symbols.first()?.to_ascii_uppercase(),
            trade_date_of(event),
            if *pct_change_bps >= 0 {
                "up".into()
            } else {
                "down".into()
            },
        )
    };
    let bps = parts
        .get(4)
        .and_then(|s| s.parse::<i64>().ok())
        .map(|b| if direction == "down" { -b } else { b })
        .unwrap_or(*pct_change_bps);
    Some((
        LadderKey {
            symbol,
            trade_date,
            direction,
        },
        bps,
    ))
}

fn close_key(event: &MarketEvent) -> Option<(String, String)> {
    let EventKind::PriceAlert { window, .. } = &event.kind else {
        return None;
    };
    if window != "close" {
        return None;
    }
    Some((
        event.symbols.first()?.to_ascii_uppercase(),
        trade_date_of(event),
    ))
}

fn trade_date_of(event: &MarketEvent) -> String {
    event
        .payload
        .get("hone_price_trade_date")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            event
                .occurred_at
                .date_naive()
                .format("%Y-%m-%d")
                .to_string()
        })
}

/// 收盘事件吸收 band 注记：`SNDK +13.67%` → `SNDK +13.67%（盘中曾跨 +16% 档）`。
fn annotate_close_with_bands(mut event: MarketEvent, bands: &[i64]) -> MarketEvent {
    let mut sorted: Vec<i64> = bands.to_vec();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.abs()));
    let notes: Vec<String> = sorted.iter().map(|bps| format_band_pct(*bps)).collect();
    event.title = format!("{}（盘中曾跨 {} 档）", event.title, notes.join("、"));
    event.payload["hone_intraday_max_band_bps"] = serde_json::json!(sorted);
    event
}

fn format_band_pct(bps: i64) -> String {
    let sign = if bps >= 0 { "+" } else { "-" };
    let abs = bps.abs();
    if abs % 100 == 0 {
        format!("{sign}{}%", abs / 100)
    } else {
        format!("{sign}{:.1}%", abs as f64 / 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Severity;
    use chrono::Utc;

    fn band(symbol: &str, date: &str, dir: &str, bps: i64) -> MarketEvent {
        MarketEvent {
            id: format!("price_band:{symbol}:{date}:{dir}:{bps}"),
            kind: EventKind::PriceAlert {
                pct_change_bps: if dir == "down" { -bps } else { bps },
                window: "day".into(),
            },
            severity: Severity::High,
            symbols: vec![symbol.into()],
            occurred_at: Utc::now(),
            title: format!(
                "{symbol} 跨过 {}{}% 档",
                if dir == "down" { "-" } else { "+" },
                bps / 100
            ),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"hone_price_trade_date": date}),
        }
    }

    fn close(symbol: &str, date: &str, pct: f64) -> MarketEvent {
        MarketEvent {
            id: format!("price_close:{symbol}:{date}"),
            kind: EventKind::PriceAlert {
                pct_change_bps: (pct * 100.0) as i64,
                window: "close".into(),
            },
            severity: Severity::Medium,
            symbols: vec![symbol.into()],
            occurred_at: Utc::now(),
            title: format!("{symbol} {pct:+.2}%"),
            summary: String::new(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"hone_price_trade_date": date}),
        }
    }

    fn news(id: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Medium,
            symbols: vec!["RKLB".into()],
            occurred_at: Utc::now(),
            title: "RKLB filed 8-K".into(),
            summary: String::new(),
            url: None,
            source: "fmp.sec".into(),
            payload: serde_json::Value::Null,
        }
    }

    /// 2026-08-13 SNDK 生产案例：5 条 band + 1 条收盘 → 只留收盘一行带注记。
    #[test]
    fn sndk_ladder_with_close_collapses_to_annotated_close() {
        let events = vec![
            band("SNDK", "2026-08-13", "up", 1600),
            band("SNDK", "2026-08-13", "up", 1400),
            band("SNDK", "2026-08-13", "up", 1200),
            band("SNDK", "2026-08-13", "up", 1000),
            band("SNDK", "2026-08-13", "up", 800),
            close("SNDK", "2026-08-13", 13.67),
        ];
        let result = coalesce_price_alerts(events);
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.omitted.len(), 5);
        let line = &result.kept[0];
        assert!(line.id.starts_with("price_close:SNDK:"));
        assert!(
            line.title.contains("（盘中曾跨 +16% 档）"),
            "title = {}",
            line.title
        );
    }

    /// 2026-08-12 NBIS 生产案例：9 条 band、无收盘 → 只留最高档。
    #[test]
    fn nbis_ladder_without_close_keeps_only_max_band() {
        let events: Vec<MarketEvent> = [1400, 1600, 1800, 2000, 2600, 2800, 3000, 3200, 3400]
            .iter()
            .map(|b| band("NBIS", "2026-08-12", "up", *b))
            .collect();
        let result = coalesce_price_alerts(events);
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].id, "price_band:NBIS:2026-08-12:up:3400");
        assert_eq!(result.omitted.len(), 8);
    }

    /// 冲高回落：同日 up + down 两组都并入收盘注记。
    #[test]
    fn whipsaw_day_annotates_both_directions() {
        let events = vec![
            band("BE", "2026-08-13", "up", 800),
            band("BE", "2026-08-13", "down", 600),
            close("BE", "2026-08-13", -2.10),
        ];
        let result = coalesce_price_alerts(events);
        assert_eq!(result.kept.len(), 1);
        let title = &result.kept[0].title;
        assert!(title.contains("+8%"), "title = {title}");
        assert!(title.contains("-6%"), "title = {title}");
    }

    /// 不同 symbol、不同日期互不影响；非价格事件原样保留且顺序不变。
    #[test]
    fn unrelated_events_pass_through_in_order() {
        let events = vec![
            news("news:1"),
            band("NBIS", "2026-08-14", "up", 800),
            close("NBIS", "2026-08-14", 8.84),
            band("CRWV", "2026-08-14", "up", 600),
            news("news:2"),
        ];
        let result = coalesce_price_alerts(events);
        let ids: Vec<&str> = result.kept.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "news:1",
                "price_close:NBIS:2026-08-14",
                "price_band:CRWV:2026-08-14:up:600",
                "news:2"
            ]
        );
        assert!(result.kept[1].title.contains("盘中曾跨 +8% 档"));
    }

    /// 🌙 凌晨曾过 recap 前缀不影响归组（按 id 归组，标题只做展示）。
    #[test]
    fn overnight_recap_prefix_does_not_break_grouping() {
        let mut stale = band("SNDK", "2026-08-13", "up", 1600);
        stale.title = format!("🌙 凌晨曾过 · {}", stale.title);
        let events = vec![stale, band("SNDK", "2026-08-13", "up", 800)];
        let result = coalesce_price_alerts(events);
        assert_eq!(result.kept.len(), 1);
        assert!(result.kept[0].title.contains("+16%"));
    }
}
