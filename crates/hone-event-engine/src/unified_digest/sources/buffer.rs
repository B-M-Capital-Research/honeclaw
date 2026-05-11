//! `BufferSource` —— 从 per-actor `DigestBuffer` 抽事件成 `UnifiedCandidate`。
//!
//! 本层复用 `DigestBuffer::drain_actor`,只负责把 `MarketEvent` 包成
//! `UnifiedCandidate { origin: Buffered, .. }`,供 `UnifiedDigestScheduler` 调用。

use chrono::Utc;
use hone_core::ActorIdentity;

use crate::digest::DigestBuffer;
use crate::unified_digest::sources::UnifiedCandidate;

pub struct BufferSource<'a> {
    buffer: &'a DigestBuffer,
}

impl<'a> BufferSource<'a> {
    pub fn new(buffer: &'a DigestBuffer) -> Self {
        Self { buffer }
    }

    /// 把 `actor` 的 buffer 文件原子改名 + 读出,逐条包成 `UnifiedCandidate`。
    /// 失败行已在 `DigestBuffer::drain_actor` 内 `tracing::warn!`,这里不再附加。
    ///
    /// `seen_at` 取调用时的 `Utc::now()` —— buffer 里没有 enqueued_at 字段对外
    /// 暴露,且 floor 排序仅看 `event.severity` / `event.kind`,seen_at 只参与
    /// 同优先级条目的 tie-break,近似为 drain 时刻足矣。
    pub fn drain(&self, actor: &ActorIdentity) -> anyhow::Result<Vec<UnifiedCandidate>> {
        let now = Utc::now();
        let events = self.buffer.drain_actor(actor)?;
        Ok(events
            .into_iter()
            .map(|e| UnifiedCandidate::from_buffered(e, now))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use crate::unified_digest::ItemOrigin;
    use chrono::TimeZone;
    use tempfile::tempdir;

    fn actor() -> ActorIdentity {
        ActorIdentity::new("telegram", "u1", None::<&str>).unwrap()
    }

    fn news(id: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap(),
            title: format!("title {id}"),
            summary: String::new(),
            url: None,
            source: "fmp.stock_news:reuters.com".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn drain_returns_empty_when_no_buffer_file() {
        let temp_dir = tempdir().unwrap();
        let digest_buffer = DigestBuffer::new(temp_dir.path()).unwrap();
        let buffer_source = BufferSource::new(&digest_buffer);
        let candidates = buffer_source.drain(&actor()).unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn drain_wraps_each_event_with_buffered_origin() {
        let temp_dir = tempdir().unwrap();
        let digest_buffer = DigestBuffer::new(temp_dir.path()).unwrap();
        let test_actor = actor();
        digest_buffer.enqueue(&test_actor, &news("e1")).unwrap();
        digest_buffer.enqueue(&test_actor, &news("e2")).unwrap();
        let buffer_source = BufferSource::new(&digest_buffer);
        let candidates = buffer_source.drain(&test_actor).unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|c| c.origin == ItemOrigin::Buffered));
        assert!(
            candidates
                .iter()
                .all(|c| c.fmp_text.is_none() && c.site.is_none())
        );
        let ids: Vec<_> = candidates.iter().map(|c| c.event.id.as_str()).collect();
        assert!(ids.contains(&"e1") && ids.contains(&"e2"));
    }

    #[test]
    fn drain_consumes_buffer_and_second_drain_is_empty() {
        let temp_dir = tempdir().unwrap();
        let digest_buffer = DigestBuffer::new(temp_dir.path()).unwrap();
        let test_actor = actor();
        digest_buffer.enqueue(&test_actor, &news("e1")).unwrap();
        let buffer_source = BufferSource::new(&digest_buffer);
        assert_eq!(buffer_source.drain(&test_actor).unwrap().len(), 1);
        // drain 是破坏性的:第二次应空。
        assert!(buffer_source.drain(&test_actor).unwrap().is_empty());
    }

    #[test]
    fn price_alert_latest_dedup_carries_through() {
        // 同 symbol 同日两条 PriceAlert,buffer 只留最新一条;BufferSource 不增加额外去重。
        let temp_dir = tempdir().unwrap();
        let digest_buffer = DigestBuffer::new(temp_dir.path()).unwrap();
        let test_actor = actor();
        let mut stale_price_alert = news("p1");
        stale_price_alert.kind = EventKind::PriceAlert {
            pct_change_bps: 500,
            window: "1d".into(),
        };
        let mut latest_price_alert = news("p2");
        latest_price_alert.kind = EventKind::PriceAlert {
            pct_change_bps: 800,
            window: "close".into(),
        };
        digest_buffer
            .enqueue(&test_actor, &stale_price_alert)
            .unwrap();
        digest_buffer
            .enqueue(&test_actor, &latest_price_alert)
            .unwrap();
        let buffer_source = BufferSource::new(&digest_buffer);
        let candidates = buffer_source.drain(&test_actor).unwrap();
        assert_eq!(candidates.len(), 1, "buffer 应保留最后一条 PriceAlert");
        assert_eq!(candidates[0].event.id, "p2");
    }
}
