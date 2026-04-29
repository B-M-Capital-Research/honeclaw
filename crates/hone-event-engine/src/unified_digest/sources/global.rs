//! `GlobalNewsSource` —— 全球要闻候选,复用 `global_digest::collector::CandidateCollector`
//! 内核 + 把产物包成 `UnifiedCandidate { origin: Global, fmp_text, site, source_class }`。
//!
//! 这一层不做任何额外过滤,只把"已经过 trusted / legal_ad / 跨批次去重"的
//! `GlobalDigestCandidate` 平铺成 unified pipeline 的统一候选类型。
//! commit 2 暂不接 scheduler;commit 3 起由 `UnifiedDigestScheduler` 调用。

use chrono::{DateTime, Utc};

use crate::global_digest::collector::CandidateCollector;
use crate::store::EventStore;
use crate::unified_digest::ItemOrigin;
use crate::unified_digest::sources::UnifiedCandidate;

pub struct GlobalNewsSource<'a> {
    store: &'a EventStore,
}

impl<'a> GlobalNewsSource<'a> {
    pub fn new(store: &'a EventStore) -> Self {
        Self { store }
    }

    /// 在 `[until - lookback_hours, until)` 内拉 trusted news 候选;
    /// `dedup_lookback_hours` 决定跨批次去重看多远(参见
    /// `CandidateCollector::collect`)。
    pub fn collect(
        &self,
        until: DateTime<Utc>,
        lookback_hours: u32,
        dedup_lookback_hours: u32,
    ) -> anyhow::Result<Vec<UnifiedCandidate>> {
        let raw = CandidateCollector::new(self.store).collect(
            until,
            lookback_hours,
            dedup_lookback_hours,
        )?;
        Ok(raw
            .into_iter()
            .map(|c| {
                let seen_at = c.event.occurred_at;
                UnifiedCandidate {
                    event: c.event,
                    origin: ItemOrigin::Global,
                    seen_at,
                    fmp_text: Some(c.fmp_text),
                    site: Some(c.site),
                    source_class: Some(c.source_class),
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::pollers::news::NewsSourceClass;
    use chrono::TimeZone;
    use serde_json::json;
    use tempfile::tempdir;

    fn news(id: &str, sc: &str, occurred: DateTime<Utc>) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::High,
            symbols: vec!["AAPL".into()],
            occurred_at: occurred,
            title: format!("title {id}"),
            summary: String::new(),
            url: Some(format!("https://reuters.com/{id}")),
            source: "fmp.stock_news:reuters.com".into(),
            payload: json!({
                "source_class": sc,
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

    #[test]
    fn wraps_collector_output_with_global_origin_and_metadata() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news("n1", "trusted", now - chrono::Duration::hours(2)))
            .unwrap();
        let src = GlobalNewsSource::new(&store);
        let out = src.collect(now, 24, 24).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].origin, ItemOrigin::Global);
        assert_eq!(out[0].source_class, Some(NewsSourceClass::Trusted));
        assert_eq!(out[0].site.as_deref(), Some("reuters.com"));
        assert!(out[0].fmp_text.as_deref().unwrap_or("").contains("body n1"));
        // seen_at = occurred_at(global 源不取 now)
        assert_eq!(out[0].seen_at, now - chrono::Duration::hours(2));
    }

    #[test]
    fn empty_when_no_trusted_news() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news("n1", "pr_wire", now - chrono::Duration::hours(2)))
            .unwrap();
        let src = GlobalNewsSource::new(&store);
        let out = src.collect(now, 24, 24).unwrap();
        assert!(out.is_empty());
    }
}
