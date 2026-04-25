//! 单 tick window-convergence 升级的可观测统计。
//!
//! 由 `process_events` 在每批 poller 入口 reset、tick 末尾 snapshot 一份用于
//! 汇总日志(`upgraded / skipped_per_tick_cap / skipped_per_symbol_cap` +
//! 触发因子 / top symbol)。本身只是数据,没有副作用。

use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NewsUpgradeTickStats {
    pub upgraded: u32,
    pub skipped_per_tick_cap: u32,
    pub skipped_per_symbol_cap: u32,
    pub trigger_counts: HashMap<String, u32>,
    pub symbol_counts: HashMap<String, u32>,
}

impl NewsUpgradeTickStats {
    pub fn has_activity(&self) -> bool {
        self.upgraded > 0 || self.skipped_per_tick_cap > 0 || self.skipped_per_symbol_cap > 0
    }

    pub fn top_symbols(&self, limit: usize) -> Vec<(String, u32)> {
        let mut items: Vec<_> = self
            .symbol_counts
            .iter()
            .map(|(sym, count)| (sym.clone(), *count))
            .collect();
        items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        items.truncate(limit);
        items
    }
}
