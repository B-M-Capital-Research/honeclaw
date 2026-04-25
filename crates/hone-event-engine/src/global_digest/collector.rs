//! 候选池采集 —— 从 EventStore 拉时间窗口内的 news 事件,做 source_class /
//! legal_ad / earnings_transcript / 已广播去重过滤后,交给后续 LLM Pass。
//!
//! 设计原则:
//! - 纯查询 + 内存过滤,不做副作用(不写 delivery_log,不调网络)
//! - 不挂 ticker 也保留 —— 全局 digest 的核心价值就是覆盖 portfolio 之外的"全球
//!   要闻",所以 collector 不按 symbols 过滤
//! - 跨批次去重粒度 = `event.id`(news url 派生),只看 channel='global_digest'
//!   的过往成功投递,不与 per-actor digest 共享去重(per-actor 推过的 portfolio
//!   命中事件,这边仍可作为全球候选 —— 受众不同)

use chrono::{DateTime, Utc};

use crate::event::MarketEvent;
use crate::pollers::news::{NewsSourceClass, is_earnings_call_transcript_title};
use crate::prefs::kind_tag;
use crate::store::EventStore;

/// channel 标记;`scheduler` 写 delivery_log 时也用同一字符串,collector
/// 通过它做跨批次去重。
pub const GLOBAL_DIGEST_CHANNEL: &str = "global_digest";

/// 候选池的回看上限(小时)。即使 `lookback_hours` 配得很大,采集窗口也不会超过
/// 此常量,避免冷启动一次性把几个月的历史新闻全拉出来。
pub const MAX_LOOKBACK_HOURS: u32 = 72;

#[derive(Debug, Clone)]
pub struct GlobalDigestCandidate {
    pub event: MarketEvent,
    pub source_class: NewsSourceClass,
    /// `payload.fmp.text` 的浅拷贝;Pass 1 直接读这里,避免每次都解 payload。
    pub fmp_text: String,
    /// `payload.fmp.site`;用于日志和 Pass 1 输入。
    pub site: String,
}

pub struct CandidateCollector<'a> {
    store: &'a EventStore,
}

impl<'a> CandidateCollector<'a> {
    pub fn new(store: &'a EventStore) -> Self {
        Self { store }
    }

    /// 拉 `[until - lookback_hours, until)` 窗口内的 trusted-source news 候选。
    ///
    /// `dedup_lookback_hours` 决定跨批次去重看多远 —— 一般等于 `lookback_hours`
    /// 或略大,避免边界刚被推过的事件下一批次又被纳入。
    pub fn collect(
        &self,
        until: DateTime<Utc>,
        lookback_hours: u32,
        dedup_lookback_hours: u32,
    ) -> anyhow::Result<Vec<GlobalDigestCandidate>> {
        let lookback = lookback_hours.min(MAX_LOOKBACK_HOURS) as i64;
        let since = until - chrono::Duration::hours(lookback);
        let dedup_since =
            until - chrono::Duration::hours(dedup_lookback_hours.min(MAX_LOOKBACK_HOURS) as i64);

        let raw = self
            .store
            .list_global_digest_news_candidates(since, until)?;
        let already_pushed = self
            .store
            .broadcasted_event_ids_since(GLOBAL_DIGEST_CHANNEL, dedup_since)?;

        let mut out = Vec::with_capacity(raw.len());
        for ev in raw {
            // SQL 已限定 source LIKE 'fmp.stock_news:%' 且 kind_json 含
            // 'news_critical',这里 belt-and-suspenders 再确认一次 kind 标签,
            // 防止以后加新 kind 字段误命中。
            if kind_tag(&ev.kind) != "news_critical" {
                continue;
            }
            if already_pushed.contains(&ev.id) {
                continue;
            }
            if is_earnings_call_transcript_title(&ev.title) {
                continue;
            }

            // payload 里 poller 已写入 source_class / legal_ad_template / fmp.text
            // (见 pollers::news::run_inner);这里直接读,不重算分类。
            let payload = &ev.payload;
            if payload
                .get("legal_ad_template")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }
            let source_class =
                parse_source_class(payload.get("source_class").and_then(|v| v.as_str()));
            // 全局 digest 只接 trusted —— pr_wire / opinion_blog / uncertain 都过滤掉。
            // (uncertain 走 router 现有 LLM 升级路径,与本管道职责区分)
            if source_class != NewsSourceClass::Trusted {
                continue;
            }
            let fmp_obj = payload.get("fmp");
            let fmp_text = fmp_obj
                .and_then(|f| f.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let site = fmp_obj
                .and_then(|f| f.get("site"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            out.push(GlobalDigestCandidate {
                event: ev,
                source_class,
                fmp_text,
                site,
            });
        }
        Ok(out)
    }
}

fn parse_source_class(s: Option<&str>) -> NewsSourceClass {
    match s.unwrap_or("uncertain") {
        "trusted" => NewsSourceClass::Trusted,
        "pr_wire" => NewsSourceClass::PrWire,
        "opinion_blog" => NewsSourceClass::OpinionBlog,
        _ => NewsSourceClass::Uncertain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, Severity};
    use chrono::TimeZone;
    use serde_json::json;
    use tempfile::tempdir;

    fn news_event(
        id: &str,
        title: &str,
        site: &str,
        source_class: &str,
        legal_ad: bool,
        severity: Severity,
        occurred: DateTime<Utc>,
    ) -> MarketEvent {
        news_event_with_source(
            id,
            title,
            site,
            source_class,
            legal_ad,
            severity,
            occurred,
            &format!("fmp.stock_news:{site}"),
        )
    }

    fn news_event_with_source(
        id: &str,
        title: &str,
        site: &str,
        source_class: &str,
        legal_ad: bool,
        severity: Severity,
        occurred: DateTime<Utc>,
        source: &str,
    ) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::NewsCritical,
            severity,
            symbols: vec!["AAPL".into()],
            occurred_at: occurred,
            title: title.into(),
            summary: "summary".into(),
            url: Some(format!("https://{site}/{id}")),
            source: source.to_string(),
            payload: json!({
                "source_class": source_class,
                "legal_ad_template": legal_ad,
                "earnings_call_transcript": false,
                "fmp": {
                    "site": site,
                    "text": format!("body of {id}"),
                    "title": title,
                    "url": format!("https://{site}/{id}"),
                },
            }),
        }
    }

    fn open_store() -> EventStore {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.sqlite3");
        let store = EventStore::open(&path).unwrap();
        // tempdir 析构后 path 失效,但 EventStore 持有 Connection 即可工作;
        // 测试结束 leak 是可接受的(单测进程一次性)。
        std::mem::forget(dir);
        store
    }

    #[test]
    fn collects_trusted_high_news_within_window() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n1",
                "Reuters big merger",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(2),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].event.id, "n1");
        assert_eq!(candidates[0].source_class, NewsSourceClass::Trusted);
        assert_eq!(candidates[0].site, "reuters.com");
        assert!(candidates[0].fmp_text.contains("body of n1"));
    }

    #[test]
    fn drops_pr_wire_opinion_blog_uncertain() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        for (id, site, sc) in [
            ("n_pr", "globenewswire.com", "pr_wire"),
            ("n_op", "seekingalpha.com", "opinion_blog"),
            ("n_un", "randomblog.example", "uncertain"),
        ] {
            store
                .insert_event(&news_event(
                    id,
                    "title",
                    site,
                    sc,
                    false,
                    Severity::High,
                    now - chrono::Duration::hours(1),
                ))
                .unwrap();
        }
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn drops_legal_ad_template_even_on_trusted() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_legal",
                "SHAREHOLDER ALERT class action filed",
                "reuters.com",
                "trusted",
                true, // legal_ad_template
                Severity::High,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn drops_earnings_call_transcript_titles() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_ect",
                "Apple Inc. (AAPL) Q2 2026 Earnings Call Transcript",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn drops_low_severity() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_low",
                "noise",
                "reuters.com",
                "trusted",
                false,
                Severity::Low,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn keeps_medium_severity() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_med",
                "Reuters mid-tier story",
                "reuters.com",
                "trusted",
                false,
                Severity::Medium,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].event.severity, Severity::Medium);
    }

    #[test]
    fn excludes_already_broadcast_event_ids() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_dup",
                "story",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        // 模拟上一批次已成功广播过这条
        store
            .log_delivery(
                "n_dup",
                "telegram::::1",
                GLOBAL_DIGEST_CHANNEL,
                Severity::High,
                "sent",
                None,
            )
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn does_not_dedup_against_per_actor_digest_channel() {
        // per-actor digest_item / sink 推过的事件,全局 digest 仍可纳入候选。
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_actor",
                "Reuters merger",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        store
            .log_delivery(
                "n_actor",
                "telegram::::1",
                "digest_item", // 不是 global_digest
                Severity::High,
                "sent",
                None,
            )
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert_eq!(candidates.len(), 1, "per-actor digest 不应影响全局候选池");
    }

    #[test]
    fn respects_lookback_window() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_old",
                "old",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(48),
            ))
            .unwrap();
        store
            .insert_event(&news_event(
                "n_recent",
                "recent",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(2),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].event.id, "n_recent");
    }

    #[test]
    fn returns_in_descending_occurred_order() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_first",
                "first",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(5),
            ))
            .unwrap();
        store
            .insert_event(&news_event(
                "n_latest",
                "latest",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(1),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].event.id, "n_latest");
        assert_eq!(candidates[1].event.id, "n_first");
    }

    #[test]
    fn lookback_caps_at_max_constant() {
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        // 5 天前的事件,即使 lookback_hours = 999 也不应被拉出来(MAX = 72h)。
        store
            .insert_event(&news_event(
                "n_ancient",
                "ancient",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(5 * 24),
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 9999, 9999)
            .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn collects_rss_sources_alongside_fmp() {
        // RSS 源(Bloomberg/SpaceNews/STAT)入 events 表 source = "rss:{handle}";
        // collector 必须把它们和 fmp.stock_news 一起拉出来。
        let store = open_store();
        let now = Utc.with_ymd_and_hms(2026, 4, 25, 12, 0, 0).unwrap();
        store
            .insert_event(&news_event(
                "n_fmp",
                "FMP Reuters story",
                "reuters.com",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(2),
            ))
            .unwrap();
        store
            .insert_event(&news_event_with_source(
                "n_bloomberg",
                "Hormuz Crisis Is Biggest Energy Disruption Ever",
                "bloomberg_markets",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::hours(1),
                "rss:bloomberg_markets",
            ))
            .unwrap();
        store
            .insert_event(&news_event_with_source(
                "n_spacenews",
                "SpaceX wins $57M Pentagon contract",
                "spacenews",
                "trusted",
                false,
                Severity::High,
                now - chrono::Duration::minutes(30),
                "rss:spacenews",
            ))
            .unwrap();
        let candidates = CandidateCollector::new(&store)
            .collect(now, 24, 24)
            .unwrap();
        assert_eq!(candidates.len(), 3);
        let sources: Vec<&str> = candidates.iter().map(|c| c.event.source.as_str()).collect();
        assert!(sources.contains(&"fmp.stock_news:reuters.com"));
        assert!(sources.contains(&"rss:bloomberg_markets"));
        assert!(sources.contains(&"rss:spacenews"));
    }
}
