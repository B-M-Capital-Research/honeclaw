//! `BufferSource` —— 从 per-actor `DigestBuffer` 抽事件成 `UnifiedCandidate`。
//!
//! commit 2 仅是 thin wrapper:`drain_actor` 仍由旧 `DigestBuffer` 实现,本层
//! 只负责把 `MarketEvent` 包成 `UnifiedCandidate { origin: Buffered, .. }`。
//! commit 3 起才由 `UnifiedDigestScheduler` 调用。

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
    /// 同优先级条目的 tie-break(commit 3 引入),近似为 drain 时刻足矣。
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
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let src = BufferSource::new(&buf);
        let out = src.drain(&actor()).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn drain_wraps_each_event_with_buffered_origin() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor();
        buf.enqueue(&a, &news("e1")).unwrap();
        buf.enqueue(&a, &news("e2")).unwrap();
        let src = BufferSource::new(&buf);
        let out = src.drain(&a).unwrap();
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|c| c.origin == ItemOrigin::Buffered));
        assert!(out.iter().all(|c| c.fmp_text.is_none() && c.site.is_none()));
        let ids: Vec<_> = out.iter().map(|c| c.event.id.as_str()).collect();
        assert!(ids.contains(&"e1") && ids.contains(&"e2"));
    }

    #[test]
    fn drain_consumes_buffer_and_second_drain_is_empty() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor();
        buf.enqueue(&a, &news("e1")).unwrap();
        let src = BufferSource::new(&buf);
        assert_eq!(src.drain(&a).unwrap().len(), 1);
        // drain 是破坏性的:第二次应空。
        assert!(src.drain(&a).unwrap().is_empty());
    }

    #[test]
    fn price_alert_latest_dedup_carries_through() {
        // 同 symbol 同日两条 PriceAlert,buffer 只留最新一条;BufferSource 不增加额外去重。
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor();
        let mut e1 = news("p1");
        e1.kind = EventKind::PriceAlert {
            pct_change_bps: 500,
            window: "1d".into(),
        };
        let mut e2 = news("p2");
        e2.kind = EventKind::PriceAlert {
            pct_change_bps: 800,
            window: "close".into(),
        };
        buf.enqueue(&a, &e1).unwrap();
        buf.enqueue(&a, &e2).unwrap();
        let src = BufferSource::new(&buf);
        let out = src.drain(&a).unwrap();
        assert_eq!(out.len(), 1, "buffer 应保留最后一条 PriceAlert");
        assert_eq!(out[0].event.id, "p2");
    }
}
