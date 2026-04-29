//! `SynthSource` —— 把 `EventStore` 里未来 4 天的 `EarningsUpcoming` teaser 合成
//! 为 T-3 / T-2 / T-1 倒计时 `MarketEvent`,按 `SubscriptionRegistry::resolve`
//! 匹配到 actor。
//!
//! 复用 `pollers::earnings::synthesize_countdowns` 的纯计算函数,逻辑与旧
//! `DigestScheduler::tick_once` 一致,只是把"per-tick 全 actor 一次算 + per-actor
//! 分发"改成"per-actor 单独取自家命中"。

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use hone_core::ActorIdentity;

use crate::pollers::earnings::synthesize_countdowns;
use crate::store::EventStore;
use crate::subscription::SharedRegistry;
use crate::unified_digest::sources::UnifiedCandidate;

pub struct SynthSource<'a> {
    store: &'a EventStore,
    registry: &'a SharedRegistry,
    /// 当 prefs.timezone 缺失时用的全局回退时区(小时整数偏移)。`SynthSource`
    /// 内部不知道 prefs,所以由调用方在 commit 3 把 `EffectiveTz` 算好的
    /// `today` 传进来更干净——本字段仅用于 `synthesize_for_today` 兜底。
    tz_offset_hours: i32,
}

impl<'a> SynthSource<'a> {
    pub fn new(store: &'a EventStore, registry: &'a SharedRegistry, tz_offset_hours: i32) -> Self {
        Self {
            store,
            registry,
            tz_offset_hours,
        }
    }

    /// 取 `actor` 命中的 T-3/T-2/T-1 倒计时事件;按 `tz_offset_hours` 解释 `now` 的"今天"。
    pub fn synthesize_for_actor(
        &self,
        actor: &ActorIdentity,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<UnifiedCandidate>> {
        let teasers = self.store.list_upcoming_earnings(now, 4)?;
        let offset = FixedOffset::east_opt(self.tz_offset_hours * 3600)
            .unwrap_or(FixedOffset::east_opt(0).unwrap());
        let local_today = offset.from_utc_datetime(&now.naive_utc()).date_naive();
        let synth_pool = synthesize_countdowns(&teasers, local_today);
        let reg = self.registry.load();
        let mut out = Vec::new();
        for ev in synth_pool {
            if reg
                .resolve(&ev)
                .into_iter()
                .any(|(a, _sev)| &a == actor && a.is_direct())
            {
                out.push(UnifiedCandidate::from_synth(ev, now));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::subscription::{PortfolioSubscription, SharedRegistry, SubscriptionRegistry};
    use crate::unified_digest::ItemOrigin;
    use tempfile::tempdir;

    fn actor() -> ActorIdentity {
        ActorIdentity::new("telegram", "u1", None::<&str>).unwrap()
    }

    fn earnings_teaser(symbol: &str, occurred: DateTime<Utc>) -> MarketEvent {
        MarketEvent {
            id: format!("teaser:{symbol}:{}", occurred.timestamp()),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![symbol.into()],
            occurred_at: occurred,
            title: format!("{symbol} earnings"),
            summary: "scheduled".into(),
            url: None,
            source: "fmp.earnings_calendar".into(),
            payload: serde_json::json!({}),
        }
    }

    fn open_store() -> EventStore {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let store = EventStore::open(&path).unwrap();
        std::mem::forget(dir);
        store
    }

    fn registry_with_holding(symbol: &str, a: &ActorIdentity) -> SharedRegistry {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            a.clone(),
            [symbol.to_string()],
        )));
        SharedRegistry::from_registry(reg)
    }

    #[test]
    fn synthesizes_t_minus_n_for_holdings_only() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        // GOOGL 财报在 4-29(T-2),AAPL 财报在 5-04(超出 T-3 窗口)
        store
            .insert_event(&earnings_teaser(
                "GOOGL",
                Utc.with_ymd_and_hms(2026, 4, 29, 20, 0, 0).unwrap(),
            ))
            .unwrap();
        store
            .insert_event(&earnings_teaser(
                "AAPL",
                Utc.with_ymd_and_hms(2026, 5, 4, 20, 0, 0).unwrap(),
            ))
            .unwrap();
        let a = actor();
        let reg = registry_with_holding("GOOGL", &a);
        let src = SynthSource::new(&store, &reg, 0); // UTC

        let out = src.synthesize_for_actor(&a, now).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].event.id.starts_with("synth:earnings:GOOGL:"));
        assert_eq!(out[0].origin, ItemOrigin::Synth);
    }

    #[test]
    fn skips_when_actor_has_no_matching_holding() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        store
            .insert_event(&earnings_teaser(
                "GOOGL",
                Utc.with_ymd_and_hms(2026, 4, 29, 20, 0, 0).unwrap(),
            ))
            .unwrap();
        let a = actor();
        // 只持有 NVDA;GOOGL 倒计时不应推 a
        let reg = registry_with_holding("NVDA", &a);
        let src = SynthSource::new(&store, &reg, 0);
        let out = src.synthesize_for_actor(&a, now).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn empty_store_returns_empty() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        let a = actor();
        let reg = registry_with_holding("GOOGL", &a);
        let src = SynthSource::new(&store, &reg, 0);
        let out = src.synthesize_for_actor(&a, now).unwrap();
        assert!(out.is_empty());
    }
}
