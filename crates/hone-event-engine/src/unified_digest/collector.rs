//! `UnifiedCollector` —— 把三个 source 编排成两层产物:
//!
//! - **per-actor pool**:`BufferSource.drain(actor)` ∪ `SynthSource.synthesize_for_actor(actor)`,
//!   每个 actor 独立产出,合并到一个 `Vec<UnifiedCandidate>`。
//! - **shared pool**:`GlobalNewsSource.collect(until, lookback, dedup_lookback)`,
//!   一次 slot 只取一次,所有 actor 共享,后续在 scheduler 里再做 per-actor `prefs` 过滤。
//!
//! commit 2 没有 scheduler 接入,本类型只提供组合 API + 测试覆盖三 source 的合并。
//! Synth / Global 两个 source 都是可选的——`SynthSource` 需要 store + registry 才有意义,
//! `GlobalNewsSource` 在 `prefs.blocked_origins` 或 dryrun 关闭全球新闻时不创建。

use chrono::{DateTime, Utc};
use hone_core::ActorIdentity;

use crate::digest::DigestBuffer;
use crate::store::EventStore;
use crate::subscription::SharedRegistry;
use crate::unified_digest::sources::{
    BufferSource, GlobalNewsSource, SynthSource, UnifiedCandidate,
};

pub struct UnifiedCollector<'a> {
    pub buffer: BufferSource<'a>,
    pub synth: Option<SynthSource<'a>>,
    pub global: Option<GlobalNewsSource<'a>>,
}

impl<'a> UnifiedCollector<'a> {
    /// 仅 buffer source —— 旧 `DigestScheduler` 在 store/registry 都没注入时的退化路径。
    pub fn buffer_only(buffer: &'a DigestBuffer) -> Self {
        Self {
            buffer: BufferSource::new(buffer),
            synth: None,
            global: None,
        }
    }

    /// 完整三源 —— commit 3 起 `UnifiedDigestScheduler` 走这条。
    pub fn new(
        buffer: &'a DigestBuffer,
        store: &'a EventStore,
        registry: &'a SharedRegistry,
        tz_offset_hours: i32,
    ) -> Self {
        Self {
            buffer: BufferSource::new(buffer),
            synth: Some(SynthSource::new(store, registry, tz_offset_hours)),
            global: Some(GlobalNewsSource::new(store)),
        }
    }

    /// 关掉 global source —— 用户在 prefs 里 block 全球 origin、或 dryrun 不想跑 LLM 时用。
    pub fn without_global(mut self) -> Self {
        self.global = None;
        self
    }

    /// per-actor 池:buffer drain + synth 倒计时。两路任一失败仍返回另一路成果,
    /// 失败原因落 `tracing::warn!`(`BufferSource` 内部已 warn,`SynthSource` 在此处 warn)。
    pub fn collect_per_actor(
        &self,
        actor: &ActorIdentity,
        now: DateTime<Utc>,
    ) -> Vec<UnifiedCandidate> {
        let mut out = match self.buffer.drain(actor) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(actor = ?actor, "buffer drain failed: {e:#}");
                Vec::new()
            }
        };
        if let Some(synth) = &self.synth {
            match synth.synthesize_for_actor(actor, now) {
                Ok(mut v) => out.append(&mut v),
                Err(e) => tracing::warn!(actor = ?actor, "synth failed: {e:#}"),
            }
        }
        out
    }

    /// shared global pool;`global` 未配置时返回空。
    pub fn collect_global(
        &self,
        until: DateTime<Utc>,
        lookback_hours: u32,
        dedup_lookback_hours: u32,
    ) -> Vec<UnifiedCandidate> {
        let Some(g) = &self.global else {
            return Vec::new();
        };
        match g.collect(until, lookback_hours, dedup_lookback_hours) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("global news collect failed: {e:#}");
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::DigestBuffer;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::subscription::{PortfolioSubscription, SharedRegistry, SubscriptionRegistry};
    use crate::unified_digest::ItemOrigin;
    use chrono::TimeZone;
    use serde_json::json;
    use tempfile::tempdir;

    fn actor() -> ActorIdentity {
        ActorIdentity::new("telegram", "u1", None::<&str>).unwrap()
    }

    fn buffered_event(id: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc.with_ymd_and_hms(2026, 4, 27, 12, 0, 0).unwrap(),
            title: format!("buf {id}"),
            summary: String::new(),
            url: None,
            source: "fmp.stock_news:reuters.com".into(),
            payload: json!({}),
        }
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
            payload: json!({}),
        }
    }

    fn global_news(id: &str, occurred: DateTime<Utc>) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::High,
            symbols: vec!["MSFT".into()],
            occurred_at: occurred,
            title: format!("global {id}"),
            summary: String::new(),
            url: Some(format!("https://reuters.com/{id}")),
            source: "fmp.stock_news:reuters.com".into(),
            payload: json!({
                "source_class": "trusted",
                "legal_ad_template": false,
                "fmp": { "site": "reuters.com", "text": format!("body {id}") },
            }),
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
    fn buffer_only_skips_synth_and_global() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor();
        buf.enqueue(&a, &buffered_event("e1")).unwrap();
        let collector = UnifiedCollector::buffer_only(&buf);
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        let per_actor = collector.collect_per_actor(&a, now);
        assert_eq!(per_actor.len(), 1);
        assert_eq!(per_actor[0].origin, ItemOrigin::Buffered);
        let global = collector.collect_global(now, 24, 24);
        assert!(global.is_empty(), "buffer_only 路径不应触达 global source");
    }

    #[test]
    fn full_collector_merges_buffer_and_synth_per_actor() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let store = open_store();
        let a = actor();
        buf.enqueue(&a, &buffered_event("e1")).unwrap();
        // GOOGL T-2 倒计时(now=4-27,event=4-29)
        store
            .insert_event(&earnings_teaser(
                "GOOGL",
                Utc.with_ymd_and_hms(2026, 4, 29, 20, 0, 0).unwrap(),
            ))
            .unwrap();
        let reg = registry_with_holding("GOOGL", &a);
        let collector = UnifiedCollector::new(&buf, &store, &reg, 0);
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        let per_actor = collector.collect_per_actor(&a, now);

        assert_eq!(per_actor.len(), 2);
        let origins: Vec<ItemOrigin> = per_actor.iter().map(|c| c.origin).collect();
        assert!(origins.contains(&ItemOrigin::Buffered));
        assert!(origins.contains(&ItemOrigin::Synth));
    }

    #[test]
    fn full_collector_pulls_global_news_independently() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let store = open_store();
        let a = actor();
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        store
            .insert_event(&global_news("g1", now - chrono::Duration::hours(2)))
            .unwrap();
        let reg = registry_with_holding("AAPL", &a);
        let collector = UnifiedCollector::new(&buf, &store, &reg, 0);

        let global = collector.collect_global(now, 24, 24);
        assert_eq!(global.len(), 1);
        assert_eq!(global[0].origin, ItemOrigin::Global);
        assert!(
            global[0]
                .fmp_text
                .as_deref()
                .unwrap_or("")
                .contains("body g1")
        );

        // per-actor 不应混入 global news
        let per_actor = collector.collect_per_actor(&a, now);
        assert!(
            per_actor.iter().all(|c| c.origin != ItemOrigin::Global),
            "global news 必须只走 collect_global,per-actor 池里不能出现 Global origin"
        );
    }

    #[test]
    fn without_global_returns_empty_global_pool() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let store = open_store();
        let a = actor();
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        store
            .insert_event(&global_news("g1", now - chrono::Duration::hours(2)))
            .unwrap();
        let reg = registry_with_holding("AAPL", &a);
        let collector = UnifiedCollector::new(&buf, &store, &reg, 0).without_global();

        let global = collector.collect_global(now, 24, 24);
        assert!(global.is_empty());
    }

    #[test]
    fn price_alert_latest_dedup_preserved_in_per_actor_pool() {
        // buffer 自带"同 symbol 同日 PriceAlert 只留最新"的去重,
        // UnifiedCollector 不应抹掉这条语义。
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor();
        let mut e1 = buffered_event("p1");
        e1.kind = EventKind::PriceAlert {
            pct_change_bps: 500,
            window: "1d".into(),
        };
        let mut e2 = buffered_event("p2");
        e2.kind = EventKind::PriceAlert {
            pct_change_bps: 800,
            window: "close".into(),
        };
        buf.enqueue(&a, &e1).unwrap();
        buf.enqueue(&a, &e2).unwrap();
        let collector = UnifiedCollector::buffer_only(&buf);
        let now = Utc.with_ymd_and_hms(2026, 4, 27, 13, 0, 0).unwrap();
        let per_actor = collector.collect_per_actor(&a, now);
        assert_eq!(per_actor.len(), 1);
        assert_eq!(per_actor[0].event.id, "p2");
    }
}
