//! NewsCritical 升级判定:把硬信号合流(price/earnings/sec/analyst)和 LLM
//! 仲裁两条独立的「Low → Medium」升级路径放在一处。
//!
//! 两条路径都是**纯升级**(只 clone + 改 severity),从不会降低,从不写共享
//! 状态(只读 store + 改 tick 计数器)。把它们拎出来后,`dispatch.rs` 只剩下
//! 「按事件分发」的骨架,不再跟新闻分类细节缠在一起。

use crate::event::{EventKind, MarketEvent, Severity};
use crate::news_classifier::Importance;
use crate::prefs::NotificationPrefs;

use super::config::NotificationRouter;

/// 同日命中后可以把 Low 新闻升到 Medium 的硬信号 kind tag 集合。
/// 语义：ticker 当天已出现过这些"事实性"事件时,同 ticker 的低优先级新闻
/// 很可能是相关解读,升到 Medium 让它进 digest 而不是沉底。
const NEWS_CONVERGENCE_HARD_SIGNALS: &[&str] = &[
    "price_alert",
    "earnings_released",
    "earnings_upcoming",
    "sec_filing",
    "analyst_grade",
];

impl NotificationRouter {
    /// 新闻多信号合流 + 财报窗口升级:当事件为 `NewsCritical + Low`,且同一 ticker
    /// 在 `[news_ts - 1d, news_ts + 2d]` 窗口内出现过硬信号
    /// (price_alert / earnings_released / earnings_upcoming / sec_filing /
    /// analyst_grade),把 severity 升到 Medium。
    ///
    /// 窗口既覆盖"前 24h 内已发生"的硬信号(#10 多信号合流),也覆盖"未来 48h 内"
    /// 的 earnings_upcoming(#13 财报窗口——因为 earnings_upcoming 的 occurred_at
    /// 是财报当日 00:00,T-1/T 新闻必须向未来扩窗才能命中)。
    ///
    /// 升级是幂等 clone,原事件不被修改。
    pub(super) fn maybe_upgrade_news(&self, event: &MarketEvent) -> MarketEvent {
        if !matches!(event.kind, EventKind::NewsCritical) || event.severity != Severity::Low {
            return event.clone();
        }
        if news_source_class_is_low_signal(event) {
            return event.clone();
        }
        let mut trigger_tag: Option<String> = None;
        for sym in &event.symbols {
            let recent_start = event.occurred_at - chrono::Duration::hours(6);
            let recent_end = event.occurred_at + chrono::Duration::hours(1);
            match self
                .store
                .symbol_signal_kinds_in_window(sym, recent_start, recent_end)
            {
                Ok(tags) => {
                    if let Some(hit) = tags
                        .iter()
                        .find(|t| NEWS_CONVERGENCE_HARD_SIGNALS.contains(&t.as_str()))
                        .filter(|t| hard_signal_correlates(event, t))
                    {
                        trigger_tag = Some(hit.clone());
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("symbol_signal_kinds_in_window failed for {sym}: {e:#}");
                }
            }
            let earnings_start = event.occurred_at - chrono::Duration::hours(12);
            let earnings_end = event.occurred_at + chrono::Duration::days(2);
            match self
                .store
                .symbol_signal_kinds_in_window(sym, earnings_start, earnings_end)
            {
                Ok(tags) => {
                    if tags.iter().any(|t| t == "earnings_upcoming")
                        && hard_signal_correlates(event, "earnings_upcoming")
                    {
                        trigger_tag = Some("earnings_upcoming".to_string());
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("symbol_signal_kinds_in_window failed for {sym}: {e:#}");
                }
            }
        }
        let Some(tag) = trigger_tag else {
            return event.clone();
        };
        if self.news_upgrade_per_tick_cap > 0 {
            if let Ok(n) = self.news_upgrade_total_counter.lock() {
                if *n >= self.news_upgrade_per_tick_cap {
                    if let Ok(mut stats) = self.news_upgrade_tick_stats.lock() {
                        stats.skipped_per_tick_cap += 1;
                    }
                    tracing::info!(
                        event_id = %event.id,
                        symbols = ?event.symbols,
                        cap = self.news_upgrade_per_tick_cap,
                        "news upgrade skipped (per-tick cap reached)"
                    );
                    return event.clone();
                }
            }
        }
        // per-symbol per-tick 升级上限:命中后维持 Low,不污染 digest 顶端。
        // 取 event.symbols 中已经升过最多次的那个 symbol 的计数代表本事件;
        // 若任一相关 symbol 都已超过 cap,则跳过升级,但所有相关 symbol 都不再
        // 计数(因为本事件未升级,不应推高计数)。
        if self.news_upgrade_per_symbol_per_tick_cap > 0 {
            if let Ok(map) = self.news_upgrade_counter.lock() {
                let already_capped = event.symbols.iter().any(|sym| {
                    map.get(sym)
                        .copied()
                        .map(|n| n >= self.news_upgrade_per_symbol_per_tick_cap)
                        .unwrap_or(false)
                });
                if already_capped {
                    if let Ok(mut stats) = self.news_upgrade_tick_stats.lock() {
                        stats.skipped_per_symbol_cap += 1;
                    }
                    tracing::info!(
                        event_id = %event.id,
                        symbols = ?event.symbols,
                        cap = self.news_upgrade_per_symbol_per_tick_cap,
                        "news upgrade skipped (per-symbol per-tick cap reached)"
                    );
                    return event.clone();
                }
            }
        }
        // 升级落地:对所有相关 symbol +1。即使某个 symbol 之前 0 次,
        // 这次升级也算它的一次"相关升级"。
        if let Ok(mut map) = self.news_upgrade_counter.lock() {
            for sym in &event.symbols {
                *map.entry(sym.clone()).or_insert(0) += 1;
            }
        }
        if let Ok(mut n) = self.news_upgrade_total_counter.lock() {
            *n += 1;
        }
        if let Ok(mut stats) = self.news_upgrade_tick_stats.lock() {
            stats.upgraded += 1;
            *stats.trigger_counts.entry(tag.clone()).or_insert(0) += 1;
            for sym in &event.symbols {
                *stats.symbol_counts.entry(sym.clone()).or_insert(0) += 1;
            }
        }
        let mut upgraded = event.clone();
        upgraded.severity = Severity::Medium;
        tracing::info!(
            event_id = %event.id,
            symbols = ?event.symbols,
            trigger = %tag,
            "news severity upgraded Low→Medium (window convergence)"
        );
        upgraded
    }

    /// 检查该事件是否是"不确定来源 Low NewsCritical",需要 LLM 仲裁器
    /// 介入决定是否升级。返回 `Some(upgraded_event)` 表示 LLM 判 important,
    /// router 应使用升级后的 severity=Medium。返回 `None` 表示无需升级
    /// (源/类型/分类器/LLM 输出 均不满足)。
    pub(super) async fn maybe_llm_upgrade_for_actor(
        &self,
        event: &MarketEvent,
        prefs: &NotificationPrefs,
    ) -> Option<MarketEvent> {
        // 仅对 NewsCritical / SocialPost 的 Low + uncertain 源走 LLM 路径;其它类型直接跳过。
        // SocialPost 由 Telegram / Truth Social 等社交 poller 产出,payload.source_class
        // 一律写 "uncertain",所以每条帖子都经 LLM 仲裁判是否升 Medium。
        if !matches!(event.kind, EventKind::NewsCritical | EventKind::SocialPost)
            || event.severity != Severity::Low
        {
            return None;
        }
        let source_class = event
            .payload
            .get("source_class")
            .and_then(|v| v.as_str())
            .unwrap_or("uncertain");
        if source_class != "uncertain" {
            return None;
        }
        // 律所模板已被 poller 强制 Low,LLM 也不应再"复活"它。
        let is_legal_ad = event
            .payload
            .get("legal_ad_template")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_legal_ad {
            return None;
        }
        let classifier = self.news_classifier.as_ref()?;
        let prompt = prefs
            .news_importance_prompt
            .as_deref()
            .unwrap_or(&self.default_importance_prompt);
        match classifier.classify(event, prompt).await {
            Some(Importance::Important) => {
                let mut upgraded = event.clone();
                upgraded.severity = Severity::Medium;
                tracing::info!(
                    event_id = %event.id,
                    "uncertain-source news upgraded Low→Medium by LLM classifier"
                );
                Some(upgraded)
            }
            _ => None,
        }
    }
}

pub(super) fn news_source_class_is_low_signal(event: &MarketEvent) -> bool {
    matches!(
        event.payload.get("source_class").and_then(|v| v.as_str()),
        Some("opinion_blog" | "pr_wire")
    )
}

fn hard_signal_correlates(event: &MarketEvent, tag: &str) -> bool {
    let text = format!("{} {}", event.title, event.summary).to_ascii_lowercase();
    let any = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));
    match tag {
        "price_alert" => any(&[
            "price", "stock", "share", "shares", "surge", "jump", "rally", "fall", "drop", "slump",
            "plunge",
        ]),
        "earnings_released" | "earnings_upcoming" | "earnings_call_transcript" => any(&[
            "earnings",
            "results",
            "revenue",
            "profit",
            "eps",
            "guidance",
            "quarter",
            "transcript",
        ]),
        "sec_filing" => any(&["sec", "filing", "8-k", "10-k", "10-q", "investigation"]),
        "analyst_grade" => any(&["analyst", "upgrade", "downgrade", "price target", "rating"]),
        _ => true,
    }
}
