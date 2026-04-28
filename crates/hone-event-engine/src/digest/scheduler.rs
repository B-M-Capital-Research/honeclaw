//! `DigestScheduler` —— 每 60s tick 检查本地时间,命中 pre/post-market 窗口就把
//! actor 的 buffer 原子 drain、过 prefs、过 curation、过 topic memory,然后推送。
//!
//! 三层过滤:
//! 1. **per-actor prefs** (`user_prefs.should_deliver`):用户临时关了某 kind / 窗口;
//! 2. **recent topic memory** (`suppress_recent_digest_topics_with_omitted`):
//!    过去 24h digest 已经出过同主题的 Low/Medium,不再重复塞入;
//! 3. **curation caps** (`curate_digest_events_with_omitted_at`):per-symbol /
//!    per-source / per-domain / topic-jaccard 级联 cap。
//!
//! 通过后按 `digest_score` 降序截到 `max_items_per_batch` 条,渲染后 send,
//! 全量落 delivery_log(成功的落 sent+digest_item,omitted 的落 digest_item:omitted,
//! 整批 digest-batch:… 一条)。

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use hone_core::ActorIdentity;

use crate::event::MarketEvent;
use crate::router::body_preview;

use super::buffer::DigestBuffer;
use super::curation::{
    curate_digest_events_with_omitted_at, digest_score, suppress_recent_digest_topics_with_omitted,
};
use super::render::render_digest;
use super::time_window::EffectiveTz;

pub struct DigestScheduler {
    buffer: Arc<DigestBuffer>,
    sink: Arc<dyn crate::router::OutboundSink>,
    store: Option<Arc<crate::store::EventStore>>,
    /// 可选订阅注册中心:有则在每次 flush 时把 T-3/T-2/T-1 的 earnings 倒计时
    /// 现算并注入到匹配 actor 的 digest payload 里(见 `synthesize_countdowns`)。
    /// 这样即使 poller cron 漂移也不会让倒计时 off-by-one。
    registry: Option<Arc<crate::subscription::SharedRegistry>>,
    prefs: Arc<dyn crate::prefs::PrefsProvider>,
    pre_market: String,
    post_market: String,
    /// pre/post-market 时刻解释所用的 UTC 偏移（小时）。
    tz_offset_hours: i32,
    /// 单批最多渲染多少条事件,超出截断并在 footer 提示。0 = 不限制。
    max_items_per_batch: usize,
    min_gap_minutes: u32,
}

impl DigestScheduler {
    /// 默认按 Asia/Shanghai（UTC+8）解释 pre/post-market 时间。
    pub fn new(
        buffer: Arc<DigestBuffer>,
        sink: Arc<dyn crate::router::OutboundSink>,
        pre_market: impl Into<String>,
        post_market: impl Into<String>,
    ) -> Self {
        Self {
            buffer,
            sink,
            store: None,
            registry: None,
            prefs: Arc::new(crate::prefs::AllowAllPrefs),
            pre_market: pre_market.into(),
            post_market: post_market.into(),
            tz_offset_hours: 8,
            max_items_per_batch: 20,
            min_gap_minutes: 0,
        }
    }

    /// 设置单批最多渲染多少条事件。0 表示不截断。
    pub fn with_max_items_per_batch(mut self, n: usize) -> Self {
        self.max_items_per_batch = n;
        self
    }

    pub fn with_min_gap_minutes(mut self, minutes: u32) -> Self {
        self.min_gap_minutes = minutes;
        self
    }

    /// 可选注入 store：flush 成功/失败时把渲染后的 digest body 写 delivery_log。
    pub fn with_store(mut self, store: Arc<crate::store::EventStore>) -> Self {
        self.store = Some(store);
        self
    }

    /// 注入订阅注册中心,开启 flush 时现算 earnings 倒计时。只有同时注入 store
    /// 才会生效(synth 需要从 store 查 teaser)。
    pub fn with_registry(mut self, registry: Arc<crate::subscription::SharedRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// 注入用户偏好源。flush 时按 actor 重读一次，运行时改立即生效。
    pub fn with_prefs(mut self, prefs: Arc<dyn crate::prefs::PrefsProvider>) -> Self {
        self.prefs = prefs;
        self
    }

    pub fn with_tz_offset_hours(mut self, offset_hours: i32) -> Self {
        self.tz_offset_hours = offset_hours;
        self
    }

    pub fn tz_offset_hours(&self) -> i32 {
        self.tz_offset_hours
    }

    /// 单轮 tick：以 actor 为外循环,各自按 `prefs.timezone` 与 `prefs.digest_windows`
    /// (缺省回到全局 pre/post-market)判断是否命中本分钟。`already_fired_today` 的
    /// key 是 `{actor}::{date}@{window}`,保证不同 actor 的相同窗口互不去重,
    /// 同 actor 的同窗口同分钟不重复。
    pub async fn tick_once(
        &self,
        now: DateTime<Utc>,
        already_fired_today: &mut std::collections::HashSet<String>,
    ) -> anyhow::Result<u32> {
        let mut flushed = 0u32;

        // ── Read-time earnings 倒计时合成 ─────────────────────────────
        // 与窗口/actor 都无关,每个 tick 算一次然后按 actor 分发。注入 store +
        // registry 才启用,缺一回到纯 buffer flush。
        let mut synth_by_actor: HashMap<ActorIdentity, Vec<MarketEvent>> = HashMap::new();
        if let (Some(store), Some(registry)) = (&self.store, &self.registry) {
            match store.list_upcoming_earnings(now, 4) {
                Ok(teasers) => {
                    let local_today = {
                        let offset = FixedOffset::east_opt(self.tz_offset_hours * 3600)
                            .unwrap_or(FixedOffset::east_opt(0).unwrap());
                        offset.from_utc_datetime(&now.naive_utc()).date_naive()
                    };
                    let synth_pool =
                        crate::pollers::earnings::synthesize_countdowns(&teasers, local_today);
                    let reg = registry.load();
                    for ev in &synth_pool {
                        for (actor, _sev) in reg.resolve(ev) {
                            if !actor.is_direct() {
                                continue;
                            }
                            synth_by_actor.entry(actor).or_default().push(ev.clone());
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "digest: list_upcoming_earnings failed, skip synth overlay: {e:#}"
                    );
                }
            }
        }

        // 合并 actor 集合:buffer 待 flush ∪ synth 命中 ∪ 有 quiet_held 行的 actor。
        // 后者必要:router 在 quiet 期间 hold 的 High 事件**只**写 delivery_log,
        // 不入 buffer;若不在这里把这些 actor 拉进来,quiet_flush 永远不会被触发。
        let mut actors: std::collections::HashSet<ActorIdentity> =
            self.buffer.list_pending_actors().into_iter().collect();
        for a in synth_by_actor.keys() {
            actors.insert(a.clone());
        }
        if let Some(store) = &self.store {
            let since = now - chrono::Duration::hours(12);
            match store.list_actors_with_quiet_held_since(since) {
                Ok(keys) => {
                    for key in keys {
                        if let Some(a) = actor_from_key(&key) {
                            actors.insert(a);
                        }
                    }
                }
                Err(e) => tracing::warn!("list_actors_with_quiet_held_since failed: {e:#}"),
            }
        }

        for actor in actors {
            // 硬规则:只向单聊推送;群 actor 的 buffer 直接 drain 丢弃。
            if !actor.is_direct() {
                let _ = self.buffer.drain_actor(&actor);
                tracing::info!(
                    channel = %actor.channel,
                    scope = ?actor.channel_scope,
                    "digest drained & dropped for group actor (push is DM-only)"
                );
                continue;
            }
            let user_prefs = self.prefs.load(&actor);
            let effective_tz =
                EffectiveTz::from_actor_prefs(user_prefs.timezone.as_deref(), self.tz_offset_hours);
            // 没设 digest_windows → 用全局 pre/post;设了空数组 → 用户主动关 digest。
            let actor_windows: Vec<String> = match user_prefs.digest_windows.as_deref() {
                Some(v) => v.to_vec(),
                None => vec![self.pre_market.clone(), self.post_market.clone()],
            };
            let actor_key_str = format!(
                "{}::{}::{}",
                actor.channel,
                actor.channel_scope.clone().unwrap_or_default(),
                actor.user_id
            );
            // quiet_hours：用户设了勿扰区间时,本 actor 的 digest 触发完全让位给
            // quiet_flush —— 区间内一律 continue 让 buffer 自然累积;命中 to 分钟
            // 时调 run_quiet_flush,把 router 在区间内 hold 的 High + buffer 里
            // 累积的 Medium/Low + (本来在 to 时刻 fire 的) digest_windows 的料
            // 全部合并成一条早间合集。
            if let Some(qh) = user_prefs.quiet_hours.as_ref() {
                if effective_tz.in_quiet_window(now, &qh.from, &qh.to) {
                    tracing::trace!(
                        actor = %actor_key_str,
                        quiet_from = %qh.from,
                        quiet_to = %qh.to,
                        "actor inside quiet_hours, skip all digest fires"
                    );
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
                        Err(e) => tracing::warn!(
                            actor = %actor_key_str,
                            "quiet_flush failed: {e:#}"
                        ),
                    }
                    continue;
                }
            }
            if actor_windows.is_empty() {
                continue;
            }
            for window in &actor_windows {
                if !effective_tz.in_window(now, window) {
                    continue;
                }
                let date = effective_tz.date_key(now);
                let fire_key = format!("{actor_key_str}::{date}@{window}");
                if !already_fired_today.insert(fire_key) {
                    continue; // 同一分钟同 actor 已触发(理论上不会)
                }
                if self.min_gap_minutes > 0 {
                    if let Some(store) = &self.store {
                        let cutoff = now - chrono::Duration::minutes(self.min_gap_minutes as i64);
                        match store.last_digest_success_at(&actor_key_str) {
                            Ok(Some(last)) if last >= cutoff => {
                                tracing::info!(
                                    actor = %actor_key_str,
                                    window = %window,
                                    last_digest_at = %last,
                                    min_gap_minutes = self.min_gap_minutes,
                                    "digest window skipped by min-gap policy"
                                );
                                continue;
                            }
                            Ok(_) => {}
                            Err(e) => tracing::warn!(
                                actor = %actor_key_str,
                                "last_digest_success_at failed: {e:#}"
                            ),
                        }
                    }
                }
                let label = if window == &self.pre_market {
                    format!("盘前摘要 · {window}")
                } else if window == &self.post_market {
                    format!("晨间摘要 · {window}")
                } else {
                    format!("盘中摘要 · {window}")
                };

                let buffered = match self.buffer.drain_actor(&actor) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!("drain_actor failed: {e:#}");
                        Vec::new()
                    }
                };
                // synth 事件每 tick 重算同 id(`synth:earnings:GOOGL:2026-04-29:countdown:2026-04-26`),
                // 所以 08:30 推过的 GOOGL 倒计时,17:00 不应再推一遍。查 delivery_log
                // 拿同 actor 同日已成功投递的 event_id 集合做过滤。
                let mut synths = synth_by_actor.remove(&actor).unwrap_or_default();
                if let Some(store) = &self.store {
                    let day_start_utc = local_day_start_utc(now, self.tz_offset_hours);
                    if let Ok(seen) = store.delivered_event_ids_since(&actor_key_str, day_start_utc)
                    {
                        let pre_count = synths.len();
                        synths.retain(|ev| !seen.contains(&ev.id));
                        if synths.len() < pre_count {
                            tracing::info!(
                                actor = %actor_key_str,
                                window = %window,
                                dropped = pre_count - synths.len(),
                                "synth countdown filtered (already delivered today)"
                            );
                        }
                    }
                }
                let mut events = buffered;
                events.extend(synths);
                if events.is_empty() {
                    continue;
                }
                // flush 时再过一遍 prefs:enqueue 后用户可能已关推送或缩小范围。
                let mut filtered: Vec<MarketEvent> = events
                    .into_iter()
                    .filter(|e| user_prefs.should_deliver(e))
                    .collect();
                if filtered.is_empty() {
                    tracing::info!(
                        actor = %actor_key_str,
                        "digest skipped by user prefs"
                    );
                    continue;
                }
                filtered.sort_by(|a, b| {
                    digest_score(b)
                        .cmp(&digest_score(a))
                        .then_with(|| b.occurred_at.cmp(&a.occurred_at))
                });
                let mut omitted_events = Vec::new();
                if let Some(store) = &self.store {
                    let memory = suppress_recent_digest_topics_with_omitted(
                        &actor_key_str,
                        filtered,
                        store,
                        now,
                    );
                    filtered = memory.kept;
                    omitted_events.extend(memory.omitted);
                }
                let curation = curate_digest_events_with_omitted_at(filtered, now);
                filtered = curation.kept;
                omitted_events.extend(curation.omitted);
                if filtered.is_empty() {
                    if let Some(store) = &self.store {
                        log_omitted_digest_items(store, &actor_key_str, &omitted_events);
                    }
                    tracing::info!(
                        actor = %actor_key_str,
                        window = %window,
                        omitted = omitted_events.len(),
                        "digest skipped after curation/topic memory"
                    );
                    continue;
                }
                // 区分两类「未展示」:
                // - **curation/topic-memory 噪音**(opinion_blog 重复、PR-wire、同 ticker
                //   第 5 条新闻 …):用户**完全不需要看见**,footer 不再提及。它们仍写
                //   delivery_log,通过 `/missed` 可以查到。
                // - **`max_items_per_batch` 单批数量上限截断**:这些是**真有内容**只是
                //   挤不进当批,footer 才提示"另 N 条因数量上限未展示,/missed 查看"。
                let noise_omitted_count = omitted_events.len();
                let mut cap_overflow = 0usize;
                if self.max_items_per_batch > 0 && filtered.len() > self.max_items_per_batch {
                    let truncated = filtered.split_off(self.max_items_per_batch);
                    let dropped_ids: Vec<String> = truncated.iter().map(|e| e.id.clone()).collect();
                    cap_overflow = dropped_ids.len();
                    omitted_events.extend(truncated);
                    tracing::info!(
                        actor = %actor_key_str,
                        cap_overflow,
                        noise_omitted = noise_omitted_count,
                        kept = filtered.len(),
                        dropped_ids = ?dropped_ids,
                        "digest truncated to avoid info flooding"
                    );
                }
                let body = render_digest(
                    &label,
                    &filtered,
                    cap_overflow,
                    self.sink.format_for(&actor),
                );
                let send_result = self.sink.send(&actor, &body).await;
                if let Some(store) = &self.store {
                    let batch_id = format!("digest-batch:{date}@{window}:{}", filtered.len());
                    let status = if send_result.is_ok() {
                        self.sink.success_status()
                    } else {
                        "failed"
                    };
                    let _ = store.log_delivery(
                        &batch_id,
                        &actor_key_str,
                        "digest",
                        filtered[0].severity,
                        status,
                        Some(&body),
                    );
                    if send_result.is_ok() {
                        for item in &filtered {
                            let _ = store.log_delivery(
                                &item.id,
                                &actor_key_str,
                                "digest_item",
                                item.severity,
                                status,
                                None,
                            );
                        }
                        log_omitted_digest_items(store, &actor_key_str, &omitted_events);
                    }
                }
                if let Err(e) = send_result {
                    let item_ids: Vec<&str> = filtered.iter().map(|e| e.id.as_str()).collect();
                    tracing::warn!(
                        actor = %actor_key_str,
                        window = %window,
                        items = filtered.len(),
                        item_ids = ?item_ids,
                        body_len = body.chars().count(),
                        body_preview = %body_preview(&body),
                        "digest sink failed: {e:#}"
                    );
                    continue;
                }
                let item_ids: Vec<&str> = filtered.iter().map(|e| e.id.as_str()).collect();
                tracing::info!(
                    actor = %actor_key_str,
                    window = %window,
                    items = filtered.len(),
                    item_ids = ?item_ids,
                    cap_overflow,
                    noise_omitted = noise_omitted_count,
                    body_len = body.chars().count(),
                    body_preview = %body_preview(&body),
                    "digest delivered"
                );
                flushed += 1;
            }
        }
        Ok(flushed)
    }

    /// 在 `quiet_hours.to` 时刻触发的早间合集。把 router 期间 hold 的事件 + buffer
    /// 里累积的 Medium/Low 合并发一条。过保鲜期的 hold 事件直接 drop 并写
    /// `delivery_log status='quiet_dropped'` 审计。
    ///
    /// 返回 `true` = 实际发出了一条；`false` = 候选为空 / 全部噪音被过滤。
    async fn run_quiet_flush(
        &self,
        actor: &ActorIdentity,
        actor_key_str: &str,
        user_prefs: &crate::prefs::NotificationPrefs,
        qh: &crate::prefs::QuietHours,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        // 1. 拉 router 在 quiet 期间 hold 住的事件。since=now-12h 覆盖任意 quiet 跨度。
        let since = now - chrono::Duration::hours(12);
        let mut held: Vec<MarketEvent> = Vec::new();
        let mut dropped_stale = 0usize;
        if let Some(store) = &self.store {
            match store.list_quiet_held_since(actor_key_str, since) {
                Ok(rows) => {
                    for (event, _sent_at) in rows {
                        if event.kind.is_fresh(event.occurred_at, now) {
                            held.push(event);
                        } else {
                            // 过保鲜期 → 写审计,不进合集
                            let _ = store.log_delivery(
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
                Err(e) => tracing::warn!(
                    actor = %actor_key_str,
                    "list_quiet_held_since failed: {e:#}"
                ),
            }
        }
        // 2. drain buffer 里 quiet 期间累积的 Medium/Low(router 早就 enqueue 进去了)
        let buffered = match self.buffer.drain_actor(actor) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("drain_actor failed in quiet_flush: {e:#}");
                Vec::new()
            }
        };
        // 3. 按 event_id 去重合并(同一事件被 hold 又入 buffer 极少见,但保险起见)
        let mut seen_ids: std::collections::HashSet<String> =
            held.iter().map(|e| e.id.clone()).collect();
        let mut events = held;
        for ev in buffered {
            if seen_ids.insert(ev.id.clone()) {
                events.push(ev);
            }
        }
        if events.is_empty() && dropped_stale == 0 {
            tracing::info!(
                actor = %actor_key_str,
                quiet_to = %qh.to,
                "quiet_flush: nothing held or buffered, skip"
            );
            return Ok(false);
        }
        // 4. 复用现有过滤管线:prefs.should_deliver → topic memory → curation → cap
        let mut filtered: Vec<MarketEvent> = events
            .into_iter()
            .filter(|e| user_prefs.should_deliver(e))
            .collect();
        if filtered.is_empty() {
            tracing::info!(
                actor = %actor_key_str,
                quiet_to = %qh.to,
                dropped_stale,
                "quiet_flush: all candidates filtered by prefs"
            );
            return Ok(false);
        }
        filtered.sort_by(|a, b| {
            digest_score(b)
                .cmp(&digest_score(a))
                .then_with(|| b.occurred_at.cmp(&a.occurred_at))
        });
        let mut omitted_events = Vec::new();
        if let Some(store) = &self.store {
            let memory =
                suppress_recent_digest_topics_with_omitted(actor_key_str, filtered, store, now);
            filtered = memory.kept;
            omitted_events.extend(memory.omitted);
        }
        let curation = curate_digest_events_with_omitted_at(filtered, now);
        filtered = curation.kept;
        omitted_events.extend(curation.omitted);
        if filtered.is_empty() {
            if let Some(store) = &self.store {
                log_omitted_digest_items(store, actor_key_str, &omitted_events);
            }
            tracing::info!(
                actor = %actor_key_str,
                quiet_to = %qh.to,
                omitted = omitted_events.len(),
                dropped_stale,
                "quiet_flush: skipped after curation/topic memory"
            );
            return Ok(false);
        }
        let noise_omitted_count = omitted_events.len();
        let mut cap_overflow = 0usize;
        if self.max_items_per_batch > 0 && filtered.len() > self.max_items_per_batch {
            let truncated = filtered.split_off(self.max_items_per_batch);
            cap_overflow = truncated.len();
            omitted_events.extend(truncated);
        }
        let label = format!("晨间静音合集 · {}", qh.to);
        let body = render_digest(&label, &filtered, cap_overflow, self.sink.format_for(actor));
        let send_result = self.sink.send(actor, &body).await;
        if let Some(store) = &self.store {
            let date = effective_tz_date_key(user_prefs, self.tz_offset_hours, now);
            let batch_id = format!("quiet-flush:{date}@{}:{}", qh.to, filtered.len());
            let status = if send_result.is_ok() {
                self.sink.success_status()
            } else {
                "failed"
            };
            let _ = store.log_delivery(
                &batch_id,
                actor_key_str,
                "digest",
                filtered[0].severity,
                status,
                Some(&body),
            );
            if send_result.is_ok() {
                for item in &filtered {
                    let _ = store.log_delivery(
                        &item.id,
                        actor_key_str,
                        "digest_item",
                        item.severity,
                        status,
                        None,
                    );
                }
                log_omitted_digest_items(store, actor_key_str, &omitted_events);
            }
        }
        match send_result {
            Ok(()) => {
                tracing::info!(
                    actor = %actor_key_str,
                    quiet_to = %qh.to,
                    items = filtered.len(),
                    cap_overflow,
                    noise_omitted = noise_omitted_count,
                    dropped_stale,
                    body_len = body.chars().count(),
                    body_preview = %body_preview(&body),
                    "quiet_flush delivered"
                );
                Ok(true)
            }
            Err(e) => {
                tracing::warn!(
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
}

fn effective_tz_date_key(
    prefs: &crate::prefs::NotificationPrefs,
    fallback_offset_hours: i32,
    now: DateTime<Utc>,
) -> String {
    EffectiveTz::from_actor_prefs(prefs.timezone.as_deref(), fallback_offset_hours).date_key(now)
}

/// `delivery_log.actor` 列存的是 `channel::scope::user_id` 的字符串(`actor_key`
/// 函数生成),这里反向解析回 `ActorIdentity`。scope 为空 → direct session。
/// 解析失败返回 None,调用方 skip 即可。
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

/// 当前 UTC 时刻按给定 `tz_offset_hours` 解释成本地日的 00:00,然后还原成
/// UTC。给 synth 跨 flush 去重的 `delivered_event_ids_since(since=...)` 用。
fn local_day_start_utc(
    now: chrono::DateTime<chrono::Utc>,
    tz_offset_hours: i32,
) -> chrono::DateTime<chrono::Utc> {
    use chrono::{NaiveTime, TimeZone};
    let offset =
        FixedOffset::east_opt(tz_offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let midnight = local
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    offset
        .from_local_datetime(&midnight)
        .single()
        .map(|l| l.with_timezone(&chrono::Utc))
        .unwrap_or(now)
}

fn log_omitted_digest_items(
    store: &crate::store::EventStore,
    actor_key: &str,
    omitted: &[MarketEvent],
) {
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
