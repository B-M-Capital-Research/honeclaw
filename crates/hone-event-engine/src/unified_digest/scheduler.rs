//! `UnifiedDigestScheduler` —— 取代旧 `DigestScheduler` + `GlobalDigestScheduler`。
//!
//! 每 60s tick 一次,以 actor 为外循环、`effective_digest_slots` 为内循环;每个
//! slot 触发时:
//! 1. **per-actor 池**:`buffer.drain` + `synth countdown`(filtered against 已投递)
//! 2. **shared global pool**:同 slot 同 tick 跨 actor 复用一份
//!    `audience + collect + dedupe + pass1 + fetch_bodies + pass2_baseline`
//! 3. **per-actor pass2 personalize**(若 prefs 没把 global origin 屏蔽掉)
//! 4. **floor 分类**:High severity / earnings synth countdown / immediate_kinds 标
//!    `FloorTag`,LLM 输出 `PickCategory::MacroFloor` 也标 floor
//! 5. **合并排序**:floor prepend → 其余按 `digest_score` → topic memory + curation
//!    cap(High 与 floor 不被剔)→ `max_items_per_batch` 截断 → render + send + log
//!
//! 调用方在 `pipeline::cron_minute_tick` 里以 60s 频率调 `tick_once`。
//! `quiet_hours` 期间 actor 整体让位,`to` 时刻触发 `quiet_flush` 把 router hold
//! 的 High + buffer 累积合并一次性发出 —— 这块逻辑直接平移自旧 `DigestScheduler`。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, NaiveTime, TimeZone, Utc};
use hone_core::ActorIdentity;
use hone_memory::PortfolioStorage;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::digest::DigestBuffer;
use crate::digest::curation::{
    curate_digest_events_with_omitted_at, digest_score, suppress_recent_digest_topics_with_omitted,
};
use crate::digest::render::{build_digest_payload, render_digest};
use crate::digest::time_window::EffectiveTz;
use crate::event::MarketEvent;
use crate::fmp::FmpClient;
use crate::global_digest::audience::{AudienceBuilder, AudienceContext};
use crate::global_digest::curator::{
    Curator, PersonalizedItem, PickCategory, RankedCandidate, UserThesis,
};
use crate::global_digest::event_dedupe::{EventDeduper, PassThroughDeduper};
use crate::global_digest::fetcher::{ArticleBody, ArticleFetcher, ArticleSource};
use crate::prefs::{NotificationPrefs, PrefsProvider, QuietHours};
use crate::router::{OutboundSink, body_preview};
use crate::store::EventStore;
use crate::subscription::SharedRegistry;
use crate::unified_digest::sources::UnifiedCandidate;
use crate::unified_digest::{DigestSlot, FloorTag, GlobalNewsSource, ItemOrigin, classify_floor};

/// 与旧 `GlobalDigestScheduler` 保持一致 —— 4 并发抓全文实测最稳。
const FETCH_CONCURRENCY: usize = 4;

/// 一组共享的 Pass 1 / fetch / Pass 2 baseline 产物。同一 slot 一个 tick 内
/// **只算一次**,后续命中同 slot 的 actor 直接复用做 personalize fan-out。
#[derive(Clone)]
struct GlobalSlotCache {
    audience: AudienceContext,
    picks_with_bodies: Vec<(RankedCandidate, ArticleBody)>,
}

pub struct UnifiedDigestScheduler {
    buffer: Arc<DigestBuffer>,
    sink: Arc<dyn OutboundSink>,
    store: Arc<EventStore>,
    fmp: Arc<FmpClient>,
    portfolio_storage: Arc<PortfolioStorage>,
    prefs: Arc<dyn PrefsProvider>,
    registry: Arc<SharedRegistry>,
    curator: Option<Arc<Curator>>,
    fetcher: Arc<ArticleFetcher>,
    event_deduper: Arc<dyn EventDeduper>,
    audience_cache_dir: PathBuf,
    daily_report_dir: PathBuf,

    /// 缺省 slot:用户没设 `digest_slots`/`digest_windows` 时回退到这两个时刻。
    pre_market: String,
    post_market: String,
    /// 全局 IANA 时区的 UTC 偏移(小时),actor `prefs.timezone` 缺失时兜底。
    tz_offset_hours: i32,

    max_items_per_batch: usize,
    min_gap_minutes: u32,
    /// global pool 的回看窗口(小时),传给 `CandidateCollector::collect`。
    lookback_hours: u32,
    pass2_top_n: u32,
    final_pick_n: u32,
    fetch_full_text: bool,
    event_dedupe_enabled: bool,

    /// per-tick global 池缓存 —— key = `{date}@{slot.id}@{slot.time}`。
    /// `Mutex` 因为 tick_once 是 `&self`,且同一 tick 内多 actor 命中同 slot
    /// 时需要共享同一份 picks_with_bodies。
    global_cache: Mutex<HashMap<String, GlobalSlotCache>>,
    /// 上一次 tick 的 date_key,用于跨日清缓存。
    cache_date: Mutex<Option<String>>,
}

impl UnifiedDigestScheduler {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        buffer: Arc<DigestBuffer>,
        sink: Arc<dyn OutboundSink>,
        store: Arc<EventStore>,
        fmp: Arc<FmpClient>,
        portfolio_storage: Arc<PortfolioStorage>,
        prefs: Arc<dyn PrefsProvider>,
        registry: Arc<SharedRegistry>,
        fetcher: Arc<ArticleFetcher>,
        audience_cache_dir: impl Into<PathBuf>,
        daily_report_dir: impl Into<PathBuf>,
        pre_market: impl Into<String>,
        post_market: impl Into<String>,
    ) -> Self {
        Self {
            buffer,
            sink,
            store,
            fmp,
            portfolio_storage,
            prefs,
            registry,
            curator: None,
            fetcher,
            event_deduper: Arc::new(PassThroughDeduper),
            audience_cache_dir: audience_cache_dir.into(),
            daily_report_dir: daily_report_dir.into(),
            pre_market: pre_market.into(),
            post_market: post_market.into(),
            tz_offset_hours: 8,
            max_items_per_batch: 20,
            min_gap_minutes: 0,
            lookback_hours: 14,
            pass2_top_n: 15,
            final_pick_n: 8,
            fetch_full_text: true,
            event_dedupe_enabled: true,
            global_cache: Mutex::new(HashMap::new()),
            cache_date: Mutex::new(None),
        }
    }

    pub fn with_tz_offset_hours(mut self, offset_hours: i32) -> Self {
        self.tz_offset_hours = offset_hours;
        self
    }

    pub fn with_max_items_per_batch(mut self, n: usize) -> Self {
        self.max_items_per_batch = n;
        self
    }

    pub fn with_min_gap_minutes(mut self, minutes: u32) -> Self {
        self.min_gap_minutes = minutes;
        self
    }

    pub fn with_lookback_hours(mut self, hours: u32) -> Self {
        self.lookback_hours = hours;
        self
    }

    pub fn with_pass2_top_n(mut self, n: u32) -> Self {
        self.pass2_top_n = n;
        self
    }

    pub fn with_final_pick_n(mut self, n: u32) -> Self {
        self.final_pick_n = n;
        self
    }

    pub fn with_fetch_full_text(mut self, fetch: bool) -> Self {
        self.fetch_full_text = fetch;
        self
    }

    pub fn with_event_dedupe_enabled(mut self, enabled: bool) -> Self {
        self.event_dedupe_enabled = enabled;
        self
    }

    pub fn with_curator(mut self, curator: Arc<Curator>) -> Self {
        self.curator = Some(curator);
        self
    }

    pub fn with_event_deduper(mut self, deduper: Arc<dyn EventDeduper>) -> Self {
        self.event_deduper = deduper;
        self
    }

    pub fn tz_offset_hours(&self) -> i32 {
        self.tz_offset_hours
    }

    /// 单轮 tick:遍历所有 direct actor,按各自 `effective_digest_slots` 触发。
    /// `already_fired_today` 防止同分钟同 actor 同 slot 重复触发。
    pub async fn tick_once(
        &self,
        now: DateTime<Utc>,
        already_fired_today: &mut HashSet<String>,
    ) -> anyhow::Result<u32> {
        let mut flushed = 0u32;
        let global_today = local_date_key(now, self.tz_offset_hours);

        // 跨日清掉昨天的 slot 缓存,防止一直累积。
        {
            let mut cd = self.cache_date.lock().await;
            if cd.as_ref() != Some(&global_today) {
                self.global_cache.lock().await.clear();
                *cd = Some(global_today.clone());
            }
        }

        // ── synth 倒计时按 actor 散开(per tick 一次) ─────────────────
        let mut synth_by_actor: HashMap<ActorIdentity, Vec<MarketEvent>> = HashMap::new();
        match self.store.list_upcoming_earnings(now, 4) {
            Ok(teasers) => {
                let local_today = {
                    let offset = FixedOffset::east_opt(self.tz_offset_hours * 3600)
                        .unwrap_or(FixedOffset::east_opt(0).unwrap());
                    offset.from_utc_datetime(&now.naive_utc()).date_naive()
                };
                let synth_pool =
                    crate::pollers::earnings::synthesize_countdowns(&teasers, local_today);
                let reg = self.registry.load();
                for ev in &synth_pool {
                    for (actor, _sev) in reg.resolve(ev) {
                        if actor.is_direct() {
                            synth_by_actor.entry(actor).or_default().push(ev.clone());
                        }
                    }
                }
            }
            Err(e) => warn!("unified digest: list_upcoming_earnings failed: {e:#}"),
        }

        // ── actor 集合 = buffer 待 flush ∪ synth 命中 ∪ quiet_held ─────
        let mut actors: HashSet<ActorIdentity> =
            self.buffer.list_pending_actors().into_iter().collect();
        for a in synth_by_actor.keys() {
            actors.insert(a.clone());
        }
        // 还要把所有有 portfolio 的 direct actor 拉进来 —— 即使本 tick 没 buffer
        // 没 synth,他们仍可能命中 slot 拿到 global news 推送。
        for (actor, _) in self.portfolio_storage.list_all() {
            if actor.is_direct() {
                actors.insert(actor);
            }
        }
        let since = now - chrono::Duration::hours(12);
        match self.store.list_actors_with_quiet_held_since(since) {
            Ok(keys) => {
                for key in keys {
                    if let Some(a) = actor_from_key(&key) {
                        actors.insert(a);
                    }
                }
            }
            Err(e) => warn!("list_actors_with_quiet_held_since failed: {e:#}"),
        }

        for actor in actors {
            // 群 actor 的 buffer 直接 drain 丢弃 —— digest 是 DM-only。
            if !actor.is_direct() {
                let _ = self.buffer.drain_actor(&actor);
                continue;
            }
            let user_prefs = self.prefs.load(&actor);
            let effective_tz =
                EffectiveTz::from_actor_prefs(user_prefs.timezone.as_deref(), self.tz_offset_hours);
            let actor_key_str = actor_key(&actor);

            // ── quiet_hours 优先 ─────────────────────────────────────
            if let Some(qh) = user_prefs.quiet_hours.as_ref() {
                if effective_tz.in_quiet_window(now, &qh.from, &qh.to) {
                    continue;
                }
                if effective_tz.at_quiet_to_minute(now, &qh.to) {
                    let date = effective_tz.date_key(now);
                    let fire_key = format!("{actor_key_str}::{date}@quiet_flush@{}", qh.to);
                    if !already_fired_today.insert(fire_key) {
                        continue;
                    }
                    match self
                        .run_quiet_flush(&actor, &actor_key_str, &user_prefs, qh, now)
                        .await
                    {
                        Ok(true) => flushed += 1,
                        Ok(false) => {}
                        Err(e) => warn!(
                            actor = %actor_key_str,
                            "quiet_flush failed: {e:#}"
                        ),
                    }
                    continue;
                }
            }

            // ── 解析 actor 的 slot 列表 ───────────────────────────────
            let slots: Vec<DigestSlot> = match user_prefs.effective_digest_slots() {
                Some(v) => v,
                None => vec![
                    DigestSlot::from_legacy_window(self.pre_market.clone()),
                    DigestSlot::from_legacy_window(self.post_market.clone()),
                ],
            };
            if slots.is_empty() {
                continue; // 用户主动关 digest
            }

            // 多 slot 的 synth 取一份就好(per actor 拉过一次后 .take())
            let mut synth_for_actor = synth_by_actor.remove(&actor).unwrap_or_default();

            for slot in &slots {
                if !effective_tz.in_window(now, &slot.time) {
                    continue;
                }
                let date = effective_tz.date_key(now);
                let fire_key = format!("{actor_key_str}::{date}@slot:{}@{}", slot.id, slot.time);
                if !already_fired_today.insert(fire_key) {
                    continue;
                }

                // min-gap 跨 slot 防抖
                if self.min_gap_minutes > 0 {
                    let cutoff = now - chrono::Duration::minutes(self.min_gap_minutes as i64);
                    match self.store.last_digest_success_at(&actor_key_str) {
                        Ok(Some(last)) if last >= cutoff => {
                            info!(
                                actor = %actor_key_str,
                                slot = %slot.id,
                                "digest slot skipped by min-gap policy"
                            );
                            continue;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!(actor = %actor_key_str, "last_digest_success_at failed: {e:#}")
                        }
                    }
                }

                // ── 1) per-actor 池:buffer + synth ───────────────────
                let buffered = match self.buffer.drain_actor(&actor) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("drain_actor failed: {e:#}");
                        Vec::new()
                    }
                };
                // synth 跨 slot 去重已投递
                let mut synths_this_slot = std::mem::take(&mut synth_for_actor);
                let day_start_utc = local_day_start_utc(now, self.tz_offset_hours);
                if let Ok(seen) = self
                    .store
                    .delivered_event_ids_since(&actor_key_str, day_start_utc)
                {
                    let pre_count = synths_this_slot.len();
                    synths_this_slot.retain(|ev| !seen.contains(&ev.id));
                    if synths_this_slot.len() < pre_count {
                        info!(
                            actor = %actor_key_str,
                            slot = %slot.id,
                            dropped = pre_count - synths_this_slot.len(),
                            "synth countdown filtered (already delivered today)"
                        );
                    }
                }

                // ── 2) shared global pool(同 slot 同 tick 复用) ──────
                let cache_key = format!("{date}@{}@{}", slot.id, slot.time);
                let global_cache = self.get_or_build_global_cache(&cache_key, now, slot).await;

                // ── 3) per-actor personalize(若用户没屏蔽 global) ──
                let want_global = self.global_allowed_for(&user_prefs);
                let personalized: Vec<PersonalizedItem> = if want_global
                    && global_cache
                        .as_ref()
                        .is_some_and(|c| !c.picks_with_bodies.is_empty())
                {
                    let cache = global_cache.as_ref().unwrap();
                    let thesis = UserThesis {
                        global_style: user_prefs.investment_global_style.as_deref(),
                        theses: user_prefs.investment_theses.as_ref(),
                    };
                    let floor_macro = slot
                        .floor_macro
                        .unwrap_or(user_prefs.global_digest_floor_macro_picks);
                    match self.curator.as_ref() {
                        Some(curator) => match curator
                            .pass2_personalize(
                                cache.picks_with_bodies.clone(),
                                &cache.audience,
                                thesis,
                                floor_macro,
                                self.final_pick_n,
                            )
                            .await
                        {
                            Ok(v) => v,
                            Err(e) => {
                                warn!(
                                    actor = %actor_key_str,
                                    "pass2 personalize failed: {e:#}"
                                );
                                Vec::new()
                            }
                        },
                        None => Vec::new(),
                    }
                } else {
                    Vec::new()
                };

                // ── 4) 合并 → prefs filter → floor 分类 ───────────────
                let label = slot.label.clone().unwrap_or_else(|| {
                    default_label_for(&slot.time, &self.pre_market, &self.post_market)
                });

                let mut floor_events: Vec<MarketEvent> = Vec::new();
                let mut other_events: Vec<MarketEvent> = Vec::new();

                let push_classified =
                    |ev: MarketEvent,
                     force_floor: Option<FloorTag>,
                     floor_bin: &mut Vec<MarketEvent>,
                     other_bin: &mut Vec<MarketEvent>| {
                        let tag = force_floor.or_else(|| classify_floor(&ev, &user_prefs));
                        if tag.is_some() {
                            floor_bin.push(ev);
                        } else {
                            other_bin.push(ev);
                        }
                    };

                // Buffered + synth:走 prefs filter,再分 floor / 普通
                for ev in buffered.into_iter().chain(synths_this_slot.into_iter()) {
                    if !user_prefs.should_deliver(&ev) {
                        continue;
                    }
                    push_classified(ev, None, &mut floor_events, &mut other_events);
                }
                // Personalized 全球新闻:LLM 给的 PickCategory::MacroFloor 直接 floor
                for pi in &personalized {
                    let ev = pi.candidate.event.clone();
                    if !user_prefs.should_deliver(&ev) {
                        continue;
                    }
                    let force_floor = match pi.category {
                        PickCategory::MacroFloor => Some(FloorTag::MacroFloor),
                        _ => None,
                    };
                    push_classified(ev, force_floor, &mut floor_events, &mut other_events);
                }

                if floor_events.is_empty() && other_events.is_empty() {
                    continue;
                }

                // floor 内部按 score 降序、occurred_at 降序;不进 curation。
                floor_events.sort_by(|a, b| {
                    digest_score(b)
                        .cmp(&digest_score(a))
                        .then_with(|| b.occurred_at.cmp(&a.occurred_at))
                });
                other_events.sort_by(|a, b| {
                    digest_score(b)
                        .cmp(&digest_score(a))
                        .then_with(|| b.occurred_at.cmp(&a.occurred_at))
                });

                // topic memory + curation 仅作用于非 floor 部分。
                let mut omitted_events = Vec::new();
                let memory = suppress_recent_digest_topics_with_omitted(
                    &actor_key_str,
                    other_events,
                    &self.store,
                    now,
                );
                let mut others_kept = memory.kept;
                omitted_events.extend(memory.omitted);
                let curation = curate_digest_events_with_omitted_at(others_kept, now);
                others_kept = curation.kept;
                omitted_events.extend(curation.omitted);

                // 合并:floor 永远 prepend。
                let mut merged: Vec<MarketEvent> = floor_events;
                merged.extend(others_kept);

                if merged.is_empty() {
                    if !omitted_events.is_empty() {
                        log_omitted_digest_items(&self.store, &actor_key_str, &omitted_events);
                    }
                    continue;
                }

                let noise_omitted_count = omitted_events.len();
                let mut cap_overflow = 0usize;
                if self.max_items_per_batch > 0 && merged.len() > self.max_items_per_batch {
                    let truncated = merged.split_off(self.max_items_per_batch);
                    cap_overflow = truncated.len();
                    omitted_events.extend(truncated);
                }

                // ── 5) 渲染 + 发送 + 落审计 ───────────────────────────
                let body =
                    render_digest(&label, &merged, cap_overflow, self.sink.format_for(&actor));
                let payload = build_digest_payload(label.clone(), &merged, cap_overflow);
                let send_result = self.sink.send_digest(&actor, &payload, &body).await;

                let date_key = effective_tz.date_key(now);
                let batch_id = format!(
                    "unified-digest:{date_key}@slot:{}:{}",
                    slot.id,
                    merged.len()
                );
                let status = if send_result.is_ok() {
                    self.sink.success_status()
                } else {
                    "failed"
                };
                let _ = self.store.log_delivery(
                    &batch_id,
                    &actor_key_str,
                    "digest",
                    merged[0].severity,
                    status,
                    Some(&body),
                );
                if send_result.is_ok() {
                    for item in &merged {
                        let _ = self.store.log_delivery(
                            &item.id,
                            &actor_key_str,
                            "digest_item",
                            item.severity,
                            status,
                            None,
                        );
                    }
                    // global news 单独再落一份 `global_digest_item` 审计,沿用旧 channel。
                    for pi in &personalized {
                        if !merged.iter().any(|m| m.id == pi.candidate.event.id) {
                            continue;
                        }
                        let _ = self.store.log_delivery(
                            &pi.candidate.event.id,
                            &actor_key_str,
                            "global_digest_item",
                            pi.candidate.event.severity,
                            status,
                            None,
                        );
                    }
                    log_omitted_digest_items(&self.store, &actor_key_str, &omitted_events);
                }

                if let Err(e) = send_result {
                    warn!(
                        actor = %actor_key_str,
                        slot = %slot.id,
                        items = merged.len(),
                        body_len = body.chars().count(),
                        body_preview = %body_preview(&body),
                        "unified digest sink failed: {e:#}"
                    );
                    continue;
                }
                let item_ids: Vec<&str> = merged.iter().map(|e| e.id.as_str()).collect();
                info!(
                    actor = %actor_key_str,
                    slot = %slot.id,
                    items = merged.len(),
                    item_ids = ?item_ids,
                    cap_overflow,
                    noise_omitted = noise_omitted_count,
                    body_len = body.chars().count(),
                    body_preview = %body_preview(&body),
                    "unified digest delivered"
                );
                flushed += 1;
            }
        }
        Ok(flushed)
    }

    /// 同 slot 同 tick 第一个 actor 命中时构建一次 global pool;后续 actor 直接读缓存。
    /// 任意一步失败缓存仍写入(`picks_with_bodies` 为空),避免循环重试。
    async fn get_or_build_global_cache(
        &self,
        cache_key: &str,
        now: DateTime<Utc>,
        slot: &DigestSlot,
    ) -> Option<GlobalSlotCache> {
        if self.curator.is_none() {
            return None;
        }
        {
            let cache = self.global_cache.lock().await;
            if let Some(c) = cache.get(cache_key) {
                return Some(c.clone());
            }
        }

        // 没缓存 —— 完整跑一次 audience+collect+dedupe+pass1+fetch+pass2_baseline。
        let audience =
            AudienceBuilder::new(&self.fmp, &self.audience_cache_dir, &self.portfolio_storage)
                .build()
                .await;

        let global_source = GlobalNewsSource::new(&self.store);
        let raw = match global_source.collect(
            now,
            self.lookback_hours,
            self.lookback_hours.saturating_add(2),
        ) {
            Ok(v) => v,
            Err(e) => {
                warn!(slot = %slot.id, "global collect failed: {e:#}");
                let cache = GlobalSlotCache {
                    audience: audience.clone(),
                    picks_with_bodies: Vec::new(),
                };
                self.global_cache
                    .lock()
                    .await
                    .insert(cache_key.into(), cache.clone());
                return Some(cache);
            }
        };

        // 把 UnifiedCandidate 还原回 GlobalDigestCandidate(curator 的输入类型)。
        let raw_g: Vec<crate::global_digest::collector::GlobalDigestCandidate> = raw
            .into_iter()
            .filter_map(unified_to_global_candidate)
            .collect();
        if raw_g.is_empty() {
            self.append_audit(
                &local_date_key(now, self.tz_offset_hours),
                &format!(
                    "## {} {} — no candidates\n候选池为空,跳过本次 run。\n\n",
                    local_date_key(now, self.tz_offset_hours),
                    slot.time
                ),
            );
            let cache = GlobalSlotCache {
                audience: audience.clone(),
                picks_with_bodies: Vec::new(),
            };
            self.global_cache
                .lock()
                .await
                .insert(cache_key.into(), cache.clone());
            return Some(cache);
        }

        // event-level dedup
        let raw_count = raw_g.len();
        let (candidates, dedupe_stats, _audits) = if self.event_dedupe_enabled {
            self.event_deduper.dedupe(raw_g).await
        } else {
            (
                raw_g,
                crate::global_digest::event_dedupe::DedupeStats {
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
                "event_dedupe pass-through fallback"
            );
        }

        let curator = self.curator.as_ref().unwrap();
        let ranked = match curator
            .pass1_select(&candidates, &audience, self.pass2_top_n as usize)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                warn!(slot = %slot.id, "pass1 failed: {e:#}");
                let cache = GlobalSlotCache {
                    audience: audience.clone(),
                    picks_with_bodies: Vec::new(),
                };
                self.global_cache
                    .lock()
                    .await
                    .insert(cache_key.into(), cache.clone());
                return Some(cache);
            }
        };
        if ranked.is_empty() {
            let date = local_date_key(now, self.tz_offset_hours);
            self.append_audit(
                &date,
                &format!(
                    "## {date} {} — pass1 returned 0\n候选 {} 条,Pass 1 未选出。\n\n",
                    slot.time,
                    candidates.len()
                ),
            );
            let cache = GlobalSlotCache {
                audience: audience.clone(),
                picks_with_bodies: Vec::new(),
            };
            self.global_cache
                .lock()
                .await
                .insert(cache_key.into(), cache.clone());
            return Some(cache);
        }

        let picks_with_bodies = self.fetch_bodies(ranked).await;

        // baseline 仅用于审计落盘 —— 不影响真正下发。
        match curator
            .pass2_baseline(picks_with_bodies.clone(), &audience, self.final_pick_n)
            .await
        {
            Ok(baseline) => {
                let date = local_date_key(now, self.tz_offset_hours);
                let mut s = format!(
                    "## {date} {} — candidates={} baseline_picks={}\n",
                    slot.time,
                    candidates.len(),
                    baseline.len()
                );
                for it in &baseline {
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
                s.push('\n');
                self.append_audit(&date, &s);
            }
            Err(e) => warn!(slot = %slot.id, "pass2 baseline failed: {e:#}"),
        }

        let cache = GlobalSlotCache {
            audience,
            picks_with_bodies,
        };
        self.global_cache
            .lock()
            .await
            .insert(cache_key.into(), cache.clone());
        Some(cache)
    }

    async fn fetch_bodies(
        &self,
        ranked: Vec<RankedCandidate>,
    ) -> Vec<(RankedCandidate, ArticleBody)> {
        use futures::stream::{self, StreamExt};

        let fetcher = self.fetcher.clone();
        let fetch_full_text = self.fetch_full_text;
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

    /// quiet_flush:从旧 `DigestScheduler::run_quiet_flush` 平移,简化路径
    /// (不接 LLM,只走 buffer + held + curation)。
    async fn run_quiet_flush(
        &self,
        actor: &ActorIdentity,
        actor_key_str: &str,
        user_prefs: &NotificationPrefs,
        qh: &QuietHours,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let since = now - chrono::Duration::hours(12);
        let mut held: Vec<MarketEvent> = Vec::new();
        let mut dropped_stale = 0usize;
        match self.store.list_quiet_held_since(actor_key_str, since) {
            Ok(rows) => {
                for (event, _sent_at) in rows {
                    if event.kind.is_fresh(event.occurred_at, now) {
                        held.push(event);
                    } else {
                        let _ = self.store.log_delivery(
                            &event.id,
                            actor_key_str,
                            "sink",
                            event.severity,
                            "quiet_dropped",
                            None,
                        );
                        dropped_stale += 1;
                    }
                }
            }
            Err(e) => warn!(actor = %actor_key_str, "list_quiet_held_since failed: {e:#}"),
        }
        let buffered = match self.buffer.drain_actor(actor) {
            Ok(v) => v,
            Err(e) => {
                warn!("drain_actor failed in quiet_flush: {e:#}");
                Vec::new()
            }
        };
        let mut seen_ids: HashSet<String> = held.iter().map(|e| e.id.clone()).collect();
        let mut events = held;
        for ev in buffered {
            if seen_ids.insert(ev.id.clone()) {
                events.push(ev);
            }
        }
        if events.is_empty() && dropped_stale == 0 {
            return Ok(false);
        }
        let mut filtered: Vec<MarketEvent> = events
            .into_iter()
            .filter(|e| user_prefs.should_deliver(e))
            .collect();
        if filtered.is_empty() {
            return Ok(false);
        }
        filtered.sort_by(|a, b| {
            digest_score(b)
                .cmp(&digest_score(a))
                .then_with(|| b.occurred_at.cmp(&a.occurred_at))
        });
        let mut omitted_events = Vec::new();
        let memory =
            suppress_recent_digest_topics_with_omitted(actor_key_str, filtered, &self.store, now);
        filtered = memory.kept;
        omitted_events.extend(memory.omitted);
        let curation = curate_digest_events_with_omitted_at(filtered, now);
        filtered = curation.kept;
        omitted_events.extend(curation.omitted);
        if filtered.is_empty() {
            log_omitted_digest_items(&self.store, actor_key_str, &omitted_events);
            return Ok(false);
        }
        let mut cap_overflow = 0usize;
        if self.max_items_per_batch > 0 && filtered.len() > self.max_items_per_batch {
            let truncated = filtered.split_off(self.max_items_per_batch);
            cap_overflow = truncated.len();
            omitted_events.extend(truncated);
        }
        let label = format!("晨间静音合集 · {}", qh.to);
        let body = render_digest(&label, &filtered, cap_overflow, self.sink.format_for(actor));
        let payload = build_digest_payload(label.clone(), &filtered, cap_overflow);
        let send_result = self.sink.send_digest(actor, &payload, &body).await;

        let date = effective_tz_date_key(user_prefs, self.tz_offset_hours, now);
        let batch_id = format!("quiet-flush:{date}@{}:{}", qh.to, filtered.len());
        let status = if send_result.is_ok() {
            self.sink.success_status()
        } else {
            "failed"
        };
        let _ = self.store.log_delivery(
            &batch_id,
            actor_key_str,
            "digest",
            filtered[0].severity,
            status,
            Some(&body),
        );
        if send_result.is_ok() {
            for item in &filtered {
                let _ = self.store.log_delivery(
                    &item.id,
                    actor_key_str,
                    "digest_item",
                    item.severity,
                    status,
                    None,
                );
            }
            log_omitted_digest_items(&self.store, actor_key_str, &omitted_events);
        }
        match send_result {
            Ok(()) => {
                info!(
                    actor = %actor_key_str,
                    quiet_to = %qh.to,
                    items = filtered.len(),
                    cap_overflow,
                    dropped_stale,
                    "quiet_flush delivered"
                );
                Ok(true)
            }
            Err(e) => {
                warn!(
                    actor = %actor_key_str,
                    quiet_to = %qh.to,
                    body_len = body.chars().count(),
                    body_preview = %body_preview(&body),
                    "quiet_flush sink failed: {e:#}"
                );
                Ok(false)
            }
        }
    }

    /// 用户没有屏蔽 global news origin 时返回 true。`blocked_origins` 字段尚未
    /// 加入 prefs(commit 5),目前只看 `global_digest_enabled`(commit 5 删)。
    fn global_allowed_for(&self, prefs: &NotificationPrefs) -> bool {
        prefs.global_digest_enabled
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

// ─────────────────────────── helpers ─────────────────────────────────

fn unified_to_global_candidate(
    c: UnifiedCandidate,
) -> Option<crate::global_digest::collector::GlobalDigestCandidate> {
    if c.origin != ItemOrigin::Global {
        return None;
    }
    Some(crate::global_digest::collector::GlobalDigestCandidate {
        event: c.event,
        source_class: c.source_class?,
        fmp_text: c.fmp_text.unwrap_or_default(),
        site: c.site.unwrap_or_default(),
    })
}

fn fmp_fallback_body(url: &Option<String>, fmp_text: &str) -> ArticleBody {
    ArticleBody {
        url: url.clone().unwrap_or_default(),
        text: fmp_text.to_string(),
        source: if fmp_text.is_empty() {
            ArticleSource::Empty
        } else {
            ArticleSource::FmpFallback
        },
    }
}

fn actor_from_key(key: &str) -> Option<ActorIdentity> {
    let parts: Vec<&str> = key.splitn(3, "::").collect();
    if parts.len() != 3 {
        return None;
    }
    let channel = parts[0];
    let scope = parts[1];
    let user_id = parts[2];
    if channel.is_empty() || user_id.is_empty() {
        return None;
    }
    let scope_opt: Option<String> = if scope.is_empty() {
        None
    } else {
        Some(scope.to_string())
    };
    ActorIdentity::new(channel, user_id, scope_opt).ok()
}

fn actor_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

fn local_day_start_utc(now: DateTime<Utc>, tz_offset_hours: i32) -> DateTime<Utc> {
    let offset =
        FixedOffset::east_opt(tz_offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let midnight = local
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    offset
        .from_local_datetime(&midnight)
        .single()
        .map(|l| l.with_timezone(&Utc))
        .unwrap_or(now)
}

fn local_date_key(now: DateTime<Utc>, tz_offset_hours: i32) -> String {
    crate::digest::local_date_key(now, tz_offset_hours)
}

fn effective_tz_date_key(
    prefs: &NotificationPrefs,
    fallback_offset_hours: i32,
    now: DateTime<Utc>,
) -> String {
    EffectiveTz::from_actor_prefs(prefs.timezone.as_deref(), fallback_offset_hours).date_key(now)
}

fn default_label_for(time: &str, pre_market: &str, post_market: &str) -> String {
    if time == pre_market {
        format!("盘前摘要 · {time}")
    } else if time == post_market {
        format!("晨间摘要 · {time}")
    } else {
        format!("盘中摘要 · {time}")
    }
}

fn log_omitted_digest_items(store: &EventStore, actor_key: &str, omitted: &[MarketEvent]) {
    for item in omitted {
        let _ = store.log_delivery(
            &item.id,
            actor_key,
            "digest_item",
            item.severity,
            "omitted",
            None,
        );
    }
}
