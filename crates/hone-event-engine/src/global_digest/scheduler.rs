//! 全局 digest scheduler —— minute-tick + 多 schedule + 每用户独立 personalize fan-out。
//!
//! 流程(POC e2e 验证):
//! 1. minute-tick:每 60s 检查 `config.schedules` 里的"HH:MM"是否命中本地分钟
//!    且今天该 slot 未触发
//! 2. 命中后跑一次 run:
//!    - audience.build() → 持仓概览(FMP profile + portfolio.notes)
//!    - collector.collect() → 候选池(FMP + RSS,去重已广播)
//!    - 候选为空 → 写 daily_report 备注 + return
//!    - curator.pass1_select() → top_n RankedCandidate
//!    - 并发 fetch full body(rate limit + UA fallback)
//!    - curator.pass2_baseline() → 写 daily_report 审计
//!    - 对每个 direct + global_digest_enabled=true 的 actor:
//!      - 读 prefs (thesis + floor)
//!      - curator.pass2_personalize() → PersonalizedItem
//!      - render → sink.send → log_delivery
//! 3. fired_today.insert("{date}@{schedule}")
//!
//! 失败容错:任何一步抛 Err,scheduler 不退出 —— 写 warn,跳过本次 run,
//! fired_today 仍标记(避免一分钟内疯狂重试)。下一个 schedule 重新尝试。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use hone_core::ActorIdentity;
use hone_core::config::event_engine::{GlobalDigestConfig, tz_offset_hours};
use hone_memory::PortfolioStorage;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::digest::time_window::{in_window, local_date_key};
use crate::fmp::FmpClient;
use crate::global_digest::audience::AudienceBuilder;
use crate::global_digest::collector::{CandidateCollector, GLOBAL_DIGEST_CHANNEL};
use crate::global_digest::curator::{BaselineCuratedItem, Curator, PersonalizedItem, UserThesis};
use crate::global_digest::event_dedupe::{
    ClusterAudit, DedupeStats, EventDeduper, PassThroughDeduper,
};
use crate::global_digest::fetcher::{ArticleFetcher, ArticleSource};
use crate::global_digest::renderer::render_global_digest;
use crate::prefs::PrefsProvider;
use crate::router::OutboundSink;
use crate::store::EventStore;

/// 并发抓全文的上限 —— 太多并发被多个 host 一起 rate-limit;太少耗时。
/// POC 实测 sequential 30s,4 并发可压到 ~10s 且不被封。
const FETCH_CONCURRENCY: usize = 4;

pub struct GlobalDigestScheduler {
    config: GlobalDigestConfig,
    store: Arc<EventStore>,
    fmp: Arc<FmpClient>,
    portfolio_storage: Arc<PortfolioStorage>,
    prefs: Arc<dyn PrefsProvider>,
    sink: Arc<dyn OutboundSink>,
    curator: Arc<Curator>,
    fetcher: Arc<ArticleFetcher>,
    event_deduper: Arc<dyn EventDeduper>,
    audience_cache_dir: PathBuf,
    daily_report_dir: PathBuf,
    fired_today: Mutex<HashSet<String>>, // {date}@{HH:MM}
    tz_offset: i32,
}

impl GlobalDigestScheduler {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: GlobalDigestConfig,
        store: Arc<EventStore>,
        fmp: Arc<FmpClient>,
        portfolio_storage: Arc<PortfolioStorage>,
        prefs: Arc<dyn PrefsProvider>,
        sink: Arc<dyn OutboundSink>,
        curator: Arc<Curator>,
        fetcher: Arc<ArticleFetcher>,
        audience_cache_dir: impl Into<PathBuf>,
        daily_report_dir: impl Into<PathBuf>,
    ) -> Self {
        let tz_offset = tz_offset_hours(&config.timezone);
        Self {
            config,
            store,
            fmp,
            portfolio_storage,
            prefs,
            sink,
            curator,
            fetcher,
            event_deduper: Arc::new(PassThroughDeduper),
            audience_cache_dir: audience_cache_dir.into(),
            daily_report_dir: daily_report_dir.into(),
            fired_today: Mutex::new(HashSet::new()),
            tz_offset,
        }
    }

    /// 注入事件级去重器。`config.event_dedupe_enabled=false` 时即使注入也走 pass-through。
    pub fn with_event_deduper(mut self, deduper: Arc<dyn EventDeduper>) -> Self {
        self.event_deduper = deduper;
        self
    }

    /// 单 tick:检查所有 schedule,命中且未触发 → 跑一次 run。返回触发的 schedule 数。
    pub async fn tick(&self, now: DateTime<Utc>) -> u32 {
        if !self.config.enabled || self.config.schedules.is_empty() {
            return 0;
        }
        let date = local_date_key(now, self.tz_offset);
        let mut fired = 0u32;
        for schedule in &self.config.schedules {
            if !in_window(now, schedule, self.tz_offset) {
                continue;
            }
            let key = format!("{date}@{schedule}");
            {
                let mut set = self.fired_today.lock().await;
                if !set.insert(key.clone()) {
                    continue;
                }
                // 维护时清理跨日:fired_today 只保留当天 key
                set.retain(|k| k.starts_with(&date));
            }
            info!(schedule = %schedule, date = %date, "global digest run starting");
            if let Err(e) = self.run_once(now, schedule).await {
                warn!(schedule = %schedule, "global digest run failed: {e:#}");
            }
            fired += 1;
        }
        fired
    }

    async fn run_once(&self, now: DateTime<Utc>, schedule: &str) -> anyhow::Result<()> {
        // 1. audience
        let audience =
            AudienceBuilder::new(&self.fmp, &self.audience_cache_dir, &self.portfolio_storage)
                .build()
                .await;

        // 2. collect
        let collector = CandidateCollector::new(&self.store);
        let raw_candidates = collector.collect(
            now,
            self.config.lookback_hours,
            self.config.lookback_hours.saturating_add(2),
        )?;
        if raw_candidates.is_empty() {
            self.write_audit_no_candidates(now, schedule);
            return Ok(());
        }

        // 2.5 event-level dedup(grok 二阶段聚类,失败降级透传)
        let raw_count = raw_candidates.len();
        let (candidates, dedupe_stats, dedupe_audits) = if self.config.event_dedupe_enabled {
            self.event_deduper.dedupe(raw_candidates).await
        } else {
            (
                raw_candidates,
                DedupeStats {
                    input: raw_count,
                    clusters: raw_count,
                    multi_clusters: 0,
                    silent_drops_recovered: 0,
                    fell_back_to_pass_through: false,
                },
                Vec::new(),
            )
        };
        if dedupe_stats.fell_back_to_pass_through {
            warn!(
                input = dedupe_stats.input,
                "event_dedupe fell back to pass-through (LLM call or parse failed)"
            );
        }
        info!(
            raw = dedupe_stats.input,
            clusters = dedupe_stats.clusters,
            multi = dedupe_stats.multi_clusters,
            recovered = dedupe_stats.silent_drops_recovered,
            "event_dedupe done"
        );

        // 3. Pass 1
        let ranked = self
            .curator
            .pass1_select(&candidates, &audience, self.config.pass2_top_n as usize)
            .await?;
        if ranked.is_empty() {
            self.write_audit_no_pass1_picks(now, schedule, candidates.len());
            return Ok(());
        }

        // 4. fetch bodies
        let picks_with_bodies = self.fetch_bodies(ranked).await;

        // 5. Pass 2 baseline (审计)
        let baseline = match self
            .curator
            .pass2_baseline(
                picks_with_bodies.clone(),
                &audience,
                self.config.final_pick_n,
            )
            .await
        {
            Ok(v) => v,
            Err(e) => {
                warn!("pass2 baseline failed: {e:#}");
                Vec::new()
            }
        };
        self.write_audit(
            now,
            schedule,
            candidates.len(),
            &baseline,
            &dedupe_stats,
            &dedupe_audits,
        );

        // 6. fan-out personalize
        let direct_actors = self.collect_direct_actors();
        for actor in direct_actors {
            let prefs = self.prefs.load(&actor);
            if !prefs.global_digest_enabled {
                continue;
            }
            let thesis = UserThesis {
                global_style: prefs.investment_global_style.as_deref(),
                theses: prefs.investment_theses.as_ref(),
            };
            let personalized = match self
                .curator
                .pass2_personalize(
                    picks_with_bodies.clone(),
                    &audience,
                    thesis,
                    prefs.global_digest_floor_macro_picks,
                    self.config.final_pick_n,
                )
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    warn!(actor = %actor_key(&actor), "pass2 personalize failed: {e:#}");
                    continue;
                }
            };
            let date = local_date_key(now, self.tz_offset);
            let body = render_global_digest(&personalized, &date, self.sink.format_for(&actor));
            let send_result = self.sink.send(&actor, &body).await;
            self.log_send(&actor, &personalized, &body, send_result);
        }

        Ok(())
    }

    async fn fetch_bodies(
        &self,
        ranked: Vec<crate::global_digest::curator::RankedCandidate>,
    ) -> Vec<(
        crate::global_digest::curator::RankedCandidate,
        crate::global_digest::ArticleBody,
    )> {
        use futures::stream::{self, StreamExt};

        let fetcher = self.fetcher.clone();
        let fetch_full_text = self.config.fetch_full_text;
        stream::iter(ranked)
            .map(|rc| {
                let fetcher = fetcher.clone();
                let fmp_text: String = rc.candidate.fmp_text.clone();
                let url_opt: Option<String> = rc.candidate.event.url.clone();
                async move {
                    let body = if fetch_full_text {
                        if let Some(url_str) = url_opt {
                            fetcher.fetch(&url_str, &fmp_text).await
                        } else {
                            fmp_fallback_body(&rc.candidate.event.url, &fmp_text)
                        }
                    } else {
                        fmp_fallback_body(&rc.candidate.event.url, &fmp_text)
                    };
                    (rc, body)
                }
            })
            .buffer_unordered(FETCH_CONCURRENCY)
            .collect::<Vec<_>>()
            .await
    }

    fn collect_direct_actors(&self) -> Vec<ActorIdentity> {
        self.portfolio_storage
            .list_all()
            .into_iter()
            .filter_map(|(actor, _)| if actor.is_direct() { Some(actor) } else { None })
            .collect()
    }

    fn log_send(
        &self,
        actor: &ActorIdentity,
        items: &[PersonalizedItem],
        body: &str,
        result: anyhow::Result<()>,
    ) {
        let actor_str = actor_key(actor);
        let success = result.is_ok();
        let status = if success {
            self.sink.success_status()
        } else {
            "failed"
        };
        let batch_id = format!(
            "global-digest-batch:{}:{}",
            chrono::Utc::now().timestamp(),
            items.len()
        );
        let severity = items
            .first()
            .map(|i| i.candidate.event.severity)
            .unwrap_or(crate::event::Severity::Medium);
        if let Err(e) = self.store.log_delivery(
            &batch_id,
            &actor_str,
            GLOBAL_DIGEST_CHANNEL,
            severity,
            status,
            Some(body),
        ) {
            warn!("global_digest log_delivery (batch) failed: {e}");
        }
        if success {
            for item in items {
                if let Err(e) = self.store.log_delivery(
                    &item.candidate.event.id,
                    &actor_str,
                    "global_digest_item",
                    item.candidate.event.severity,
                    status,
                    None,
                ) {
                    warn!("global_digest log_delivery (item) failed: {e}");
                }
            }
            info!(actor = %actor_str, items = items.len(), "global digest sent");
        } else if let Err(e) = result {
            warn!(actor = %actor_str, "global digest send failed: {e:#}");
        }
    }

    fn write_audit_no_candidates(&self, now: DateTime<Utc>, schedule: &str) {
        let date = local_date_key(now, self.tz_offset);
        let line = format!("## {date} {schedule} — no candidates\n候选池为空,跳过本次 run。\n\n");
        self.append_audit(&date, &line);
    }

    fn write_audit_no_pass1_picks(&self, now: DateTime<Utc>, schedule: &str, n_cands: usize) {
        let date = local_date_key(now, self.tz_offset);
        let line = format!(
            "## {date} {schedule} — pass1 returned 0\n候选 {n_cands} 条,Pass 1 未选出任何条目。\n\n"
        );
        self.append_audit(&date, &line);
    }

    fn write_audit(
        &self,
        now: DateTime<Utc>,
        schedule: &str,
        n_cands: usize,
        baseline: &[BaselineCuratedItem],
        dedupe_stats: &DedupeStats,
        dedupe_audits: &[ClusterAudit],
    ) {
        let date = local_date_key(now, self.tz_offset);
        let dedupe_note = if dedupe_stats.fell_back_to_pass_through {
            " dedupe=pass-through(LLM 失败)".to_string()
        } else if dedupe_stats.input == dedupe_stats.clusters {
            String::new()
        } else {
            format!(
                " dedupe={}→{}({} 簇合并, {} 救回)",
                dedupe_stats.input,
                dedupe_stats.clusters,
                dedupe_stats.multi_clusters,
                dedupe_stats.silent_drops_recovered
            )
        };
        let mut s = format!(
            "## {date} {schedule} — candidates={n_cands} baseline_picks={}{dedupe_note}\n",
            baseline.len()
        );
        for it in baseline {
            s.push_str(&format!(
                "  #{} [{}] {} — {}\n",
                it.rank,
                it.candidate.event.source,
                it.candidate
                    .event
                    .title
                    .chars()
                    .take(80)
                    .collect::<String>(),
                it.comment.chars().take(120).collect::<String>(),
            ));
        }
        // 把 dedup 多簇 audit 写在末尾,便于诊断
        let multi_audits: Vec<&ClusterAudit> = dedupe_audits
            .iter()
            .filter(|a| !a.merged_event_ids.is_empty())
            .collect();
        if !multi_audits.is_empty() {
            s.push_str("  dedupe-clusters:\n");
            for a in multi_audits {
                s.push_str(&format!(
                    "    {} kept={} merged={}\n",
                    a.id,
                    a.kept_event_id,
                    a.merged_event_ids.join(",")
                ));
            }
        }
        s.push('\n');
        self.append_audit(&date, &s);
    }

    fn append_audit(&self, date: &str, body: &str) {
        if std::fs::create_dir_all(&self.daily_report_dir).is_err() {
            return;
        }
        let path = self
            .daily_report_dir
            .join(format!("{date}-global-digest.md"));
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = f.write_all(body.as_bytes());
        }
    }
}

fn fmp_fallback_body(url: &Option<String>, fmp_text: &str) -> crate::global_digest::ArticleBody {
    crate::global_digest::ArticleBody {
        url: url.clone().unwrap_or_default(),
        text: fmp_text.to_string(),
        source: if fmp_text.is_empty() {
            ArticleSource::Empty
        } else {
            ArticleSource::FmpFallback
        },
    }
}

fn actor_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::RenderFormat;
    use async_trait::async_trait;
    use chrono::TimeZone;
    use futures::stream::{self, BoxStream};
    use hone_core::config::FmpConfig;
    use hone_core::{HoneError, HoneResult};
    use hone_llm::{ChatResponse, LlmProvider, Message, provider::ChatResult};
    use std::sync::Mutex as StdMutex;
    use tempfile::tempdir;

    struct MockSink {
        sent: StdMutex<Vec<(String, String)>>, // (actor_key, body)
    }
    #[async_trait]
    impl OutboundSink for MockSink {
        async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
            self.sent
                .lock()
                .unwrap()
                .push((actor_key(actor), body.into()));
            Ok(())
        }
        fn success_status(&self) -> &'static str {
            "sent"
        }
        fn format(&self) -> RenderFormat {
            RenderFormat::Plain
        }
    }

    struct StubProvider {
        pass1_response: String,
        pass2_response: String,
        calls: StdMutex<Vec<String>>, // 记录 model 名字以分辨 pass1 / pass2
    }
    #[async_trait]
    impl LlmProvider for StubProvider {
        async fn chat(&self, _m: &[Message], model: Option<&str>) -> HoneResult<ChatResult> {
            let model = model.unwrap_or("").to_string();
            self.calls.lock().unwrap().push(model.clone());
            let content = if model.contains("pass1") || model.contains("nova-lite") {
                self.pass1_response.clone()
            } else {
                self.pass2_response.clone()
            };
            Ok(ChatResult {
                content,
                usage: None,
            })
        }
        async fn chat_with_tools(
            &self,
            _: &[Message],
            _: &[serde_json::Value],
            _: Option<&str>,
        ) -> HoneResult<ChatResponse> {
            Err(HoneError::Llm("nu".into()))
        }
        fn chat_stream<'a>(
            &'a self,
            _: &'a [Message],
            _: Option<&'a str>,
        ) -> BoxStream<'a, HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    fn make_scheduler(
        config: GlobalDigestConfig,
        store: Arc<EventStore>,
        portfolio_storage: Arc<PortfolioStorage>,
        prefs: Arc<dyn PrefsProvider>,
        sink: Arc<dyn OutboundSink>,
        provider: Arc<dyn LlmProvider>,
        tmpdir: &std::path::Path,
    ) -> GlobalDigestScheduler {
        let fmp = Arc::new(FmpClient::from_config(&FmpConfig::default()));
        let curator = Arc::new(Curator::new(provider, "pass1-stub-nova-lite", "pass2-stub"));
        let fetcher = Arc::new(ArticleFetcher::new());
        GlobalDigestScheduler::new(
            config,
            store,
            fmp,
            portfolio_storage,
            prefs,
            sink,
            curator,
            fetcher,
            tmpdir.join("audience_cache"),
            tmpdir.join("daily_reports"),
        )
    }

    #[tokio::test]
    async fn tick_disabled_does_nothing() {
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let storage = Arc::new(PortfolioStorage::new(dir.path().join("p")));
        let prefs: Arc<dyn PrefsProvider> = Arc::new(crate::prefs::AllowAllPrefs);
        let sink: Arc<dyn OutboundSink> = Arc::new(MockSink {
            sent: StdMutex::new(vec![]),
        });
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            pass1_response: "{}".into(),
            pass2_response: "{}".into(),
            calls: StdMutex::new(vec![]),
        });
        let mut config = GlobalDigestConfig::default();
        config.enabled = false; // off
        config.schedules = vec!["09:00".into()];
        let s = make_scheduler(config, store, storage, prefs, sink, provider, dir.path());
        let now = Utc.with_ymd_and_hms(2026, 4, 26, 1, 0, 0).unwrap(); // 09:00 CST
        assert_eq!(s.tick(now).await, 0);
    }

    #[tokio::test]
    async fn tick_outside_window_does_nothing() {
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let storage = Arc::new(PortfolioStorage::new(dir.path().join("p")));
        let prefs: Arc<dyn PrefsProvider> = Arc::new(crate::prefs::AllowAllPrefs);
        let sink: Arc<dyn OutboundSink> = Arc::new(MockSink {
            sent: StdMutex::new(vec![]),
        });
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            pass1_response: "{}".into(),
            pass2_response: "{}".into(),
            calls: StdMutex::new(vec![]),
        });
        let mut config = GlobalDigestConfig::default();
        config.enabled = true;
        config.schedules = vec!["09:00".into()];
        let s = make_scheduler(config, store, storage, prefs, sink, provider, dir.path());
        let now = Utc.with_ymd_and_hms(2026, 4, 26, 5, 0, 0).unwrap(); // 13:00 CST,非 09:00
        assert_eq!(s.tick(now).await, 0);
    }

    #[tokio::test]
    async fn tick_at_window_with_no_candidates_writes_audit() {
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let storage = Arc::new(PortfolioStorage::new(dir.path().join("p")));
        let prefs: Arc<dyn PrefsProvider> = Arc::new(crate::prefs::AllowAllPrefs);
        let sink: Arc<dyn OutboundSink> = Arc::new(MockSink {
            sent: StdMutex::new(vec![]),
        });
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            pass1_response: "{}".into(),
            pass2_response: "{}".into(),
            calls: StdMutex::new(vec![]),
        });
        let mut config = GlobalDigestConfig::default();
        config.enabled = true;
        config.schedules = vec!["09:00".into()];
        let s = make_scheduler(
            config,
            store,
            storage,
            prefs,
            sink.clone(),
            provider,
            dir.path(),
        );
        let now = Utc.with_ymd_and_hms(2026, 4, 26, 1, 0, 0).unwrap(); // 09:00 CST
        assert_eq!(s.tick(now).await, 1);
        // 同分钟第二次 tick 不重复触发
        assert_eq!(s.tick(now).await, 0);
        // 写了 audit (no candidates)
        let audit_path = dir
            .path()
            .join("daily_reports")
            .join("2026-04-26-global-digest.md");
        let content = std::fs::read_to_string(&audit_path).unwrap();
        assert!(content.contains("no candidates"));
    }

    #[tokio::test]
    async fn tick_at_window_with_candidates_runs_full_pipeline() {
        use crate::event::{EventKind, MarketEvent, Severity};
        use serde_json::json;

        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let storage = Arc::new(PortfolioStorage::new(dir.path().join("p")));

        // 注册一个 direct actor + portfolio,让 fan-out 至少有 1 个目标
        let actor = ActorIdentity::new("telegram", "user1", None::<&str>).unwrap();
        let portfolio = hone_memory::portfolio::Portfolio {
            actor: Some(actor.clone()),
            user_id: "user1".into(),
            holdings: vec![hone_memory::portfolio::Holding {
                symbol: "AAPL".into(),
                asset_type: "stock".into(),
                shares: 10.0,
                avg_cost: 150.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                tracking_only: None,
            }],
            updated_at: Utc::now().to_rfc3339(),
        };
        storage.save(&actor, &portfolio).unwrap();

        // 插一个候选事件
        let now = Utc.with_ymd_and_hms(2026, 4, 26, 1, 0, 0).unwrap(); // 09:00 CST
        store
            .insert_event(&MarketEvent {
                id: "news:abc".into(),
                kind: EventKind::NewsCritical,
                severity: Severity::High,
                symbols: vec![],
                occurred_at: now - chrono::Duration::hours(2),
                title: "Big news".into(),
                summary: "summary".into(),
                url: Some("https://example.com/abc".into()),
                source: "fmp.stock_news:reuters.com".into(),
                payload: json!({
                    "source_class": "trusted",
                    "legal_ad_template": false,
                    "fmp": {"site": "reuters.com", "text": "body", "url": "https://example.com/abc"}
                }),
            })
            .unwrap();

        let prefs: Arc<dyn PrefsProvider> = Arc::new(crate::prefs::AllowAllPrefs);
        let sink_inner = Arc::new(MockSink {
            sent: StdMutex::new(vec![]),
        });
        let sink: Arc<dyn OutboundSink> = sink_inner.clone();

        let pass1 =
            r#"{"items":[{"idx":0,"score":5,"cluster":"x","takeaway":"hot story"}]}"#.to_string();
        let pass2 =
            r#"{"picks":[{"idx":0,"rank":1,"title":"Big news","url":"https://example.com/abc","comment":"important","category":"thesis_aligned","thesis_relation":"中立"}]}"#.to_string();
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            pass1_response: pass1,
            pass2_response: pass2,
            calls: StdMutex::new(vec![]),
        });

        let mut config = GlobalDigestConfig::default();
        config.enabled = true;
        config.schedules = vec!["09:00".into()];
        config.fetch_full_text = false; // 不走网络

        let s = make_scheduler(config, store, storage, prefs, sink, provider, dir.path());
        assert_eq!(s.tick(now).await, 1);
        // sink 收到 1 条
        let sent = sink_inner.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1);
        let (actor_str, body) = &sent[0];
        assert!(actor_str.contains("user1"));
        assert!(body.contains("Big news"));
        assert!(body.contains("important"));
        // audit 落了 baseline
        let audit_path = dir
            .path()
            .join("daily_reports")
            .join("2026-04-26-global-digest.md");
        let audit = std::fs::read_to_string(&audit_path).unwrap();
        assert!(audit.contains("baseline_picks"));
        assert!(audit.contains("Big news"));
    }

    #[tokio::test]
    async fn next_day_same_window_can_fire_again() {
        let dir = tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let storage = Arc::new(PortfolioStorage::new(dir.path().join("p")));
        let prefs: Arc<dyn PrefsProvider> = Arc::new(crate::prefs::AllowAllPrefs);
        let sink: Arc<dyn OutboundSink> = Arc::new(MockSink {
            sent: StdMutex::new(vec![]),
        });
        let provider: Arc<dyn LlmProvider> = Arc::new(StubProvider {
            pass1_response: "{}".into(),
            pass2_response: "{}".into(),
            calls: StdMutex::new(vec![]),
        });
        let mut config = GlobalDigestConfig::default();
        config.enabled = true;
        config.schedules = vec!["09:00".into()];
        let s = make_scheduler(config, store, storage, prefs, sink, provider, dir.path());

        let day1 = Utc.with_ymd_and_hms(2026, 4, 26, 1, 0, 0).unwrap();
        assert_eq!(s.tick(day1).await, 1);
        assert_eq!(s.tick(day1).await, 0);
        let day2 = Utc.with_ymd_and_hms(2026, 4, 27, 1, 0, 0).unwrap();
        // 第二天同时间应该能再次触发
        assert_eq!(s.tick(day2).await, 1);
    }
}
