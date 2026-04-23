//! DigestBuffer — 按 actor 缓存 Medium/Low 事件，定时合并推送。
//!
//! 存储：`{buffer_dir}/{actor_key}.jsonl`，一条事件一行，append-only。
//! Flush：`drain_actor` 把文件读空并 rotate（改名加时间戳），调用方负责渲染 + 推送。
//!
//! MVP 时区处理：用固定小时偏移（见 `tz_offset_hours`），默认 Asia/Shanghai（UTC+8）。
//! 夏/冬令时按常用区域近似；接 `chrono-tz` 后替换。

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Datelike, FixedOffset, NaiveTime, TimeZone, Timelike, Utc};
use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

use crate::event::{EventKind, MarketEvent, Severity};
use crate::router::body_preview;

const DIGEST_SOCIAL_TITLE_MAX_CHARS: usize = 240;
const DIGEST_MAX_SOCIAL_ITEMS: usize = 3;
const DIGEST_MAX_ITEMS_PER_SYMBOL: usize = 4;
const DIGEST_MAX_ITEMS_PER_SOURCE: usize = 3;
const DIGEST_MAX_ITEMS_PER_DOMAIN: usize = 2;

pub struct DigestBuffer {
    dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BufferRecord {
    actor: ActorIdentity,
    event: MarketEvent,
    enqueued_at: DateTime<Utc>,
}

impl DigestBuffer {
    pub fn new(dir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn file_for(&self, actor: &ActorIdentity) -> PathBuf {
        self.dir.join(format!("{}.jsonl", actor_slug(actor)))
    }

    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    pub fn enqueue(&self, actor: &ActorIdentity, event: &MarketEvent) -> anyhow::Result<()> {
        let rec = BufferRecord {
            actor: actor.clone(),
            event: event.clone(),
            enqueued_at: Utc::now(),
        };
        let line = serde_json::to_string(&rec)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.file_for(actor))?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    /// 原子地把 actor 的 buffer 文件改名为 `*.flushed-{ts}`，再读出事件返回。
    /// 读失败的行忽略（保留已改名文件以便人工排查）。
    pub fn drain_actor(&self, actor: &ActorIdentity) -> anyhow::Result<Vec<MarketEvent>> {
        let path = self.file_for(actor);
        if !path.exists() {
            return Ok(vec![]);
        }
        let ts = Utc::now().timestamp();
        let rotated = path.with_extension(format!("flushed-{ts}"));
        std::fs::rename(&path, &rotated)?;
        let flushed_at = DateTime::<Utc>::from_timestamp(ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| ts.to_string());
        tracing::info!(
            actor = %actor_key(actor),
            path = %path.display(),
            rotated = %rotated.display(),
            flushed_at = %flushed_at,
            "digest buffer rotated"
        );

        let f = std::fs::File::open(&rotated)?;
        let mut out = Vec::new();
        for line in BufReader::new(f).lines().map_while(Result::ok) {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<BufferRecord>(&line) {
                Ok(rec) => out.push(rec.event),
                Err(e) => tracing::warn!("digest buffer parse skip: {e}"),
            }
        }
        Ok(out)
    }

    /// 列出 buffer 目录下所有有待 flush 的 actor。
    pub fn list_pending_actors(&self) -> Vec<ActorIdentity> {
        let mut actors = HashMap::new();
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return vec![];
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            if let Ok(f) = std::fs::File::open(&path) {
                if let Some(Ok(first)) = BufReader::new(f).lines().next() {
                    if let Ok(rec) = serde_json::from_str::<BufferRecord>(&first) {
                        actors.insert(actor_slug(&rec.actor), rec.actor);
                    }
                }
            }
        }
        actors.into_values().collect()
    }
}

fn actor_slug(a: &ActorIdentity) -> String {
    // 简单 slug：channel_scope_user；用 `__` 分隔避免与系统路径冲突。
    let scope = a.channel_scope.as_deref().unwrap_or("direct");
    format!(
        "{}__{}__{}",
        sanitize(&a.channel),
        sanitize(scope),
        sanitize(&a.user_id)
    )
}

fn actor_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// ── Scheduler ───────────────────────────────────────────────────────────────

/// 判断 `now` 对应的本地时间（按 `offset_hours` 解释）是否处于给定 HH:MM 的 60 秒窗口内。
pub fn in_window(now: DateTime<Utc>, hhmm: &str, offset_hours: i32) -> bool {
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let Ok(target) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
        return false;
    };
    let now_t = NaiveTime::from_hms_opt(local.hour(), local.minute(), 0).unwrap();
    now_t == target
}

/// 把 `HH:MM` 形式的时间点向前偏移 `offset_mins` 分钟,用于 cron-align pollers
/// 计算"比 flush 窗口早 N 分钟去拉数据"的 target。
/// 非法输入按原样返回。跨日回绕取模处理(例如 "00:10" - 30min → "23:40")。
pub fn shift_hhmm_earlier(hhmm: &str, offset_mins: u32) -> String {
    let Ok(t) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
        return hhmm.into();
    };
    let total = t.hour() as i64 * 60 + t.minute() as i64;
    let shifted = (total - offset_mins as i64).rem_euclid(24 * 60);
    format!("{:02}:{:02}", shifted / 60, shifted % 60)
}

/// 当前本地日期（粗略）—— 用于 flush key 防止同一天重复触发。
pub fn local_date_key(now: DateTime<Utc>, offset_hours: i32) -> String {
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    format!(
        "{:04}-{:02}-{:02}",
        local.year(),
        local.month(),
        local.day()
    )
}

/// 调度器内部用的"有效时区"——优先 IANA 名称(尊重 DST/历史偏移),否则回到全局
/// FixedOffset。这层抽象让 actor 的 prefs.timezone 与全局 `digest.timezone` 共用同
/// 一套窗口/日期判断函数,不必双份实现。
#[derive(Debug, Clone)]
enum EffectiveTz {
    Iana(chrono_tz::Tz),
    Fixed(FixedOffset),
}

impl EffectiveTz {
    fn from_actor_prefs(prefs_tz: Option<&str>, fallback_offset_hours: i32) -> Self {
        if let Some(name) = prefs_tz {
            if let Ok(tz) = name.parse::<chrono_tz::Tz>() {
                return EffectiveTz::Iana(tz);
            }
            tracing::warn!(
                "actor prefs.timezone {name:?} 解析失败,回到全局 fallback_offset_hours={fallback_offset_hours}"
            );
        }
        let offset = FixedOffset::east_opt(fallback_offset_hours * 3600)
            .unwrap_or(FixedOffset::east_opt(0).unwrap());
        EffectiveTz::Fixed(offset)
    }

    fn local_hm(&self, now: DateTime<Utc>) -> (u32, u32) {
        match self {
            EffectiveTz::Iana(tz) => {
                let local = tz.from_utc_datetime(&now.naive_utc());
                (local.hour(), local.minute())
            }
            EffectiveTz::Fixed(off) => {
                let local = off.from_utc_datetime(&now.naive_utc());
                (local.hour(), local.minute())
            }
        }
    }

    fn date_key(&self, now: DateTime<Utc>) -> String {
        let (y, m, d) = match self {
            EffectiveTz::Iana(tz) => {
                let local = tz.from_utc_datetime(&now.naive_utc());
                (local.year(), local.month(), local.day())
            }
            EffectiveTz::Fixed(off) => {
                let local = off.from_utc_datetime(&now.naive_utc());
                (local.year(), local.month(), local.day())
            }
        };
        format!("{y:04}-{m:02}-{d:02}")
    }

    fn in_window(&self, now: DateTime<Utc>, hhmm: &str) -> bool {
        let Ok(target) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
            return false;
        };
        let (h, m) = self.local_hm(now);
        h == target.hour() && m == target.minute()
    }
}

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

        // 合并 actor 集合:buffer 待 flush ∪ synth 命中。
        let mut actors: std::collections::HashSet<ActorIdentity> =
            self.buffer.list_pending_actors().into_iter().collect();
        for a in synth_by_actor.keys() {
            actors.insert(a.clone());
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
            if actor_windows.is_empty() {
                continue;
            }
            let actor_key_str = format!(
                "{}::{}::{}",
                actor.channel,
                actor.channel_scope.clone().unwrap_or_default(),
                actor.user_id
            );
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
                let synths = synth_by_actor.remove(&actor).unwrap_or_default();
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
                let mut overflow = 0usize;
                if let Some(store) = &self.store {
                    let before_memory = filtered.len();
                    filtered = suppress_recent_digest_topics(&actor_key_str, filtered, store, now);
                    overflow += before_memory.saturating_sub(filtered.len());
                }
                let before_curation = filtered.len();
                filtered = curate_digest_events(filtered);
                overflow += before_curation.saturating_sub(filtered.len());
                if filtered.is_empty() {
                    tracing::info!(
                        actor = %actor_key_str,
                        window = %window,
                        overflow,
                        "digest skipped after curation/topic memory"
                    );
                    continue;
                }
                let overflow =
                    if self.max_items_per_batch > 0 && filtered.len() > self.max_items_per_batch {
                        let dropped_ids: Vec<String> = filtered[self.max_items_per_batch..]
                            .iter()
                            .map(|e| e.id.clone())
                            .collect();
                        let dropped = dropped_ids.len();
                        filtered.truncate(self.max_items_per_batch);
                        tracing::info!(
                            actor = %actor_key_str,
                            dropped,
                            curated = overflow,
                            kept = filtered.len(),
                            dropped_ids = ?dropped_ids,
                            "digest truncated to avoid info flooding"
                        );
                        overflow += dropped;
                        overflow
                    } else {
                        overflow
                    };
                let body = render_digest(&label, &filtered, overflow, self.sink.format_for(&actor));
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
                    overflow,
                    body_len = body.chars().count(),
                    body_preview = %body_preview(&body),
                    "digest delivered"
                );
                flushed += 1;
            }
        }
        Ok(flushed)
    }
}

fn curate_digest_events(events: Vec<MarketEvent>) -> Vec<MarketEvent> {
    let mut out = Vec::with_capacity(events.len());
    let mut social_count = 0usize;
    let mut by_symbol: HashMap<String, usize> = HashMap::new();
    let mut by_source: HashMap<String, usize> = HashMap::new();
    let mut by_domain: HashMap<String, usize> = HashMap::new();
    let mut title_keys: HashSet<String> = HashSet::new();
    let mut topic_tokens: Vec<(String, HashSet<String>)> = Vec::new();

    for event in events {
        let is_high = event.severity.rank() >= crate::event::Severity::High.rank();
        if !is_high {
            if matches!(event.kind, EventKind::SocialPost) {
                if social_count >= DIGEST_MAX_SOCIAL_ITEMS {
                    continue;
                }
            }
            if let Some(symbol) = primary_symbol_key(&event) {
                if by_symbol.get(&symbol).copied().unwrap_or(0) >= DIGEST_MAX_ITEMS_PER_SYMBOL {
                    continue;
                }
            }
            if !event.source.is_empty()
                && by_source.get(&event.source).copied().unwrap_or(0) >= DIGEST_MAX_ITEMS_PER_SOURCE
            {
                continue;
            }
            if let Some(domain) = event_domain_key(&event) {
                if by_domain.get(&domain).copied().unwrap_or(0) >= DIGEST_MAX_ITEMS_PER_DOMAIN {
                    continue;
                }
            }
            if let Some(title_key) = digest_title_dedupe_key(&event) {
                if !title_keys.insert(title_key) {
                    continue;
                }
            }
            if let Some((topic_key, tokens)) = digest_topic_tokens(&event) {
                if topic_tokens
                    .iter()
                    .any(|(key, seen)| key == &topic_key && token_jaccard(seen, &tokens) >= 0.55)
                {
                    continue;
                }
                topic_tokens.push((topic_key, tokens));
            }
        }

        if matches!(event.kind, EventKind::SocialPost) {
            social_count += 1;
        }
        if let Some(symbol) = primary_symbol_key(&event) {
            *by_symbol.entry(symbol).or_default() += 1;
        }
        if !event.source.is_empty() {
            *by_source.entry(event.source.clone()).or_default() += 1;
        }
        if let Some(domain) = event_domain_key(&event) {
            *by_domain.entry(domain).or_default() += 1;
        }
        out.push(event);
    }

    out
}

fn suppress_recent_digest_topics(
    actor_key: &str,
    events: Vec<MarketEvent>,
    store: &crate::store::EventStore,
    now: DateTime<Utc>,
) -> Vec<MarketEvent> {
    let since = now - chrono::Duration::hours(24);
    let Ok(recent) = store.list_recent_digest_item_events(actor_key, since) else {
        return events;
    };
    let recent_topics: Vec<(String, HashSet<String>)> =
        recent.iter().filter_map(digest_topic_tokens).collect();
    if recent_topics.is_empty() {
        return events;
    }

    events
        .into_iter()
        .filter(|event| {
            if event.severity == Severity::High {
                return true;
            }
            let Some((topic_key, tokens)) = digest_topic_tokens(event) else {
                return true;
            };
            let duplicate = recent_topics
                .iter()
                .any(|(key, seen)| key == &topic_key && token_jaccard(seen, &tokens) >= 0.55);
            if duplicate {
                tracing::info!(
                    actor = %actor_key,
                    event_id = %event.id,
                    topic = %topic_key,
                    "digest topic suppressed by recent memory"
                );
            }
            !duplicate
        })
        .collect()
}

fn digest_score(event: &MarketEvent) -> i32 {
    let mut score = match event.severity {
        Severity::High => 300,
        Severity::Medium => 200,
        Severity::Low => 100,
    };
    score += match event.kind {
        EventKind::EarningsReleased | EventKind::SecFiling { .. } => 50,
        EventKind::EarningsCallTranscript => 15,
        EventKind::PriceAlert { ref window, .. } if window != "close" => 35,
        EventKind::Dividend | EventKind::Split | EventKind::Buyback => 30,
        EventKind::MacroEvent => 20,
        EventKind::NewsCritical => 10,
        EventKind::SocialPost => -35,
        _ => 0,
    };
    if matches!(
        event.payload.get("source_class").and_then(|v| v.as_str()),
        Some("trusted")
    ) {
        score += 20;
    }
    if matches!(
        event.payload.get("source_class").and_then(|v| v.as_str()),
        Some("pr_wire" | "opinion_blog")
    ) {
        score -= 35;
    }
    if is_low_quality_social_source(event) {
        score -= 30;
    }
    if matches!(event.kind, EventKind::EarningsUpcoming) {
        let days_until = (event.occurred_at.date_naive() - Utc::now().date_naive()).num_days();
        if days_until > 7 {
            score -= 40;
        } else if days_until <= 3 {
            score += 25;
        }
    }
    score
}

fn primary_symbol_key(event: &MarketEvent) -> Option<String> {
    event
        .symbols
        .iter()
        .find(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_ascii_uppercase())
}

fn event_domain_key(event: &MarketEvent) -> Option<String> {
    if let Some(url) = event.url.as_deref() {
        let without_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        let host = without_scheme.split('/').next().unwrap_or_default();
        let host = host.strip_prefix("www.").unwrap_or(host).trim();
        if !host.is_empty() {
            return Some(host.to_ascii_lowercase());
        }
    }
    event
        .source
        .split_once(':')
        .map(|(_, domain)| domain.trim().to_ascii_lowercase())
        .filter(|domain| !domain.is_empty())
}

fn digest_title_dedupe_key(event: &MarketEvent) -> Option<String> {
    if !matches!(
        event.kind,
        EventKind::NewsCritical | EventKind::PressRelease | EventKind::SocialPost
    ) {
        return None;
    }
    let title = digest_event_title(event);
    let normalized: Vec<String> = title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            (token.len() > 2).then_some(token)
        })
        .take(10)
        .collect();
    if normalized.is_empty() {
        return None;
    }
    let symbol = primary_symbol_key(event).unwrap_or_else(|| "-".into());
    Some(format!("{symbol}:{}", normalized.join(" ")))
}

fn digest_topic_tokens(event: &MarketEvent) -> Option<(String, HashSet<String>)> {
    if !matches!(
        event.kind,
        EventKind::NewsCritical | EventKind::PressRelease | EventKind::SocialPost
    ) {
        return None;
    }
    let tokens: HashSet<String> = digest_event_title(event)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            if token.len() <= 2 || DIGEST_STOPWORDS.contains(&token.as_str()) {
                None
            } else {
                Some(token)
            }
        })
        .collect();
    if tokens.len() < 3 {
        return None;
    }
    let symbol = primary_symbol_key(event).unwrap_or_else(|| "-".into());
    Some((format!("{symbol}:{}", kind_topic_tag(&event.kind)), tokens))
}

fn token_jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn kind_topic_tag(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::SocialPost => "social",
        EventKind::PressRelease => "press",
        _ => "news",
    }
}

fn is_low_quality_social_source(event: &MarketEvent) -> bool {
    let source = event.source.to_ascii_lowercase();
    matches!(event.kind, EventKind::SocialPost)
        && (source.contains("watcherguru") || source.contains("truth_social"))
}

const DIGEST_STOPWORDS: &[&str] = &[
    "the",
    "and",
    "for",
    "with",
    "from",
    "that",
    "this",
    "after",
    "before",
    "into",
    "over",
    "under",
    "says",
    "said",
    "stock",
    "stocks",
    "shares",
    "share",
    "inc",
    "corp",
    "ltd",
    "company",
    "announces",
    "announced",
    "update",
    "market",
];

/// 渲染 digest 摘要。`label` 由调用方控制（比如 "盘前摘要 · 08:30"），
/// 本函数只负责拼标题头 + 条目行。`overflow` > 0 时在 footer 提示"另 N 条已省略"。
///
/// 格式示例（Plain）：
/// ```text
/// 📬 盘前摘要 · 08:30 · 3 条
/// • $NVDA [拆股] · NVDA 宣布 1-for-10 拆股，生效日 2026-05-20
/// • [宏观] · [US] CPI MoM (Mar) · est 0.3 · prev 0.2
/// ```
/// 单条时省略 "· N 条"。`fmt` 控制标题是否加粗、条目文字是否转义。
pub fn render_digest(
    label: &str,
    events: &[MarketEvent],
    overflow: usize,
    fmt: crate::renderer::RenderFormat,
) -> String {
    use crate::renderer::RenderFormat;
    let total = events.len() + overflow;
    let raw_title = if total > 1 {
        format!("📬 {label} · {total} 条")
    } else {
        format!("📬 {label}")
    };
    if matches!(fmt, RenderFormat::FeishuPost) {
        return render_digest_feishu_post(&raw_title, events, overflow);
    }
    let title = match fmt {
        RenderFormat::Plain => raw_title,
        RenderFormat::TelegramHtml => format!(
            "<b>{}</b>",
            crate::renderer::render_inline(&raw_title, RenderFormat::TelegramHtml)
        ),
        RenderFormat::DiscordMarkdown => format!(
            "**{}**",
            crate::renderer::render_inline(&raw_title, RenderFormat::DiscordMarkdown)
        ),
        RenderFormat::FeishuPost => unreachable!("handled above"),
    };
    let mut out = title;
    for ev in events {
        let head = crate::renderer::header_line_compact(ev);
        let display_title = digest_event_title(ev);
        let title_inline = crate::renderer::render_inline(&display_title, fmt);
        let head_inline = crate::renderer::render_inline(&head, fmt);
        let link_inline = ev
            .url
            .as_deref()
            .filter(|u| !u.is_empty())
            .map(|u| crate::renderer::render_link_icon(u, fmt));
        out.push('\n');
        if head_inline.is_empty() {
            out.push_str(&format!("• {title_inline}"));
        } else {
            out.push_str(&format!("• {head_inline} · {title_inline}"));
        }
        if let Some(link_inline) = link_inline {
            out.push_str(" · ");
            out.push_str(&link_inline);
        }
    }
    if overflow > 0 {
        out.push('\n');
        out.push_str(&format!(
            "…… 另 {overflow} 条已省略（优先展示高优先级/最新）"
        ));
    }
    out
}

fn render_digest_feishu_post(raw_title: &str, events: &[MarketEvent], overflow: usize) -> String {
    let mut content = Vec::new();
    for ev in events {
        let head = crate::renderer::header_line_compact(ev);
        let display_title = digest_event_title(ev);
        let mut row = Vec::new();
        row.push(crate::renderer::feishu_text("• "));
        if !head.is_empty() {
            row.push(crate::renderer::feishu_text(&head));
            row.push(crate::renderer::feishu_text(" · "));
        }
        row.push(crate::renderer::feishu_text(&display_title));
        if let Some(url) = ev.url.as_deref().filter(|u| !u.is_empty()) {
            row.push(crate::renderer::feishu_text(" · "));
            row.push(crate::renderer::feishu_link_icon(url));
        }
        content.push(row);
    }
    if overflow > 0 {
        content.push(vec![crate::renderer::feishu_text(&format!(
            "…… 另 {overflow} 条已省略（优先展示高优先级/最新）"
        ))]);
    }
    serde_json::json!({
        "zh_cn": {
            "title": raw_title,
            "content": content,
        }
    })
    .to_string()
}

fn digest_event_title(event: &MarketEvent) -> String {
    if matches!(event.kind, EventKind::SocialPost) {
        if let Some(first_line) = event
            .payload
            .get("raw_text")
            .and_then(|v| v.as_str())
            .and_then(first_non_empty_line)
        {
            return truncate_chars(first_line, DIGEST_SOCIAL_TITLE_MAX_CHARS);
        }
    }
    event.title.clone()
}

fn first_non_empty_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, Severity};
    use chrono::{TimeZone, Utc};
    use tempfile::tempdir;

    fn actor(user: &str) -> ActorIdentity {
        ActorIdentity::new("imessage", user, None::<&str>).unwrap()
    }

    fn ev(id: &str, sym: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![sym.into()],
            occurred_at: Utc::now(),
            title: format!("{sym} earnings"),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn enqueue_then_drain_returns_events_in_order() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor("u1");
        buf.enqueue(&a, &ev("1", "AAPL")).unwrap();
        buf.enqueue(&a, &ev("2", "MSFT")).unwrap();
        let drained = buf.drain_actor(&a).unwrap();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].id, "1");
        assert_eq!(drained[1].id, "2");
    }

    #[test]
    fn drain_leaves_no_unflushed_file() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor("u1");
        buf.enqueue(&a, &ev("1", "AAPL")).unwrap();
        let _ = buf.drain_actor(&a).unwrap();
        // 再次 drain 得到空
        assert!(buf.drain_actor(&a).unwrap().is_empty());
    }

    #[test]
    fn list_pending_actors_dedups() {
        let dir = tempdir().unwrap();
        let buf = DigestBuffer::new(dir.path()).unwrap();
        let a = actor("u1");
        let b = actor("u2");
        buf.enqueue(&a, &ev("1", "AAPL")).unwrap();
        buf.enqueue(&a, &ev("2", "MSFT")).unwrap();
        buf.enqueue(&b, &ev("3", "TSLA")).unwrap();
        let pending = buf.list_pending_actors();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn in_window_matches_local_time_exactly() {
        // 2026-04-21 12:30 UTC == 08:30 ET (UTC-4)
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
        assert!(in_window(now, "08:30", -4));
        // 一分钟偏差不算命中
        let now_off = Utc.with_ymd_and_hms(2026, 4, 21, 12, 31, 0).unwrap();
        assert!(!in_window(now_off, "08:30", -4));
        // UTC+8（北京）下 2026-04-21 00:30 UTC == 08:30 上海
        let now_sh = Utc.with_ymd_and_hms(2026, 4, 21, 0, 30, 0).unwrap();
        assert!(in_window(now_sh, "08:30", 8));
    }

    #[tokio::test]
    async fn scheduler_respects_disabled_prefs_at_flush_time() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
        use crate::router::OutboundSink;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<(String, String)>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0
                    .lock()
                    .unwrap()
                    .push((a.user_id.clone(), body.into()));
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let sink = Arc::new(SpySink::default());
        let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
        buf.enqueue(&actor("u1"), &ev("1", "AAPL")).unwrap();
        // u1 在 enqueue 之后把推送关了
        prefs
            .save(
                &actor("u1"),
                &NotificationPrefs {
                    enabled: false,
                    ..Default::default()
                },
            )
            .unwrap();

        let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
            .with_tz_offset_hours(-4)
            .with_prefs(prefs);
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
        let mut fired = HashSet::new();
        let n = sched.tick_once(now, &mut fired).await.unwrap();
        assert_eq!(n, 0, "prefs.enabled=false 下不应推送 digest");
        assert!(sink.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn scheduler_flushes_buffer_and_avoids_duplicate_fire() {
        use crate::router::OutboundSink;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<(String, String)>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0
                    .lock()
                    .unwrap()
                    .push((a.user_id.clone(), body.into()));
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path()).unwrap());
        let sink = Arc::new(SpySink::default());
        buf.enqueue(&actor("u1"), &ev("1", "AAPL")).unwrap();
        buf.enqueue(&actor("u1"), &ev("2", "MSFT")).unwrap();
        buf.enqueue(&actor("u2"), &ev("3", "TSLA")).unwrap();

        // 显式按 ET (-4) 解释窗口，复用原有 UTC 12:30 == 08:30 ET 的测试向量。
        let sched =
            DigestScheduler::new(buf, sink.clone(), "08:30", "17:00").with_tz_offset_hours(-4);
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
        let mut fired = HashSet::new();
        let n = sched.tick_once(now, &mut fired).await.unwrap();
        assert_eq!(n, 2, "应为两个 actor 各 flush 一次");
        // 同一分钟再 tick 不应重复
        let n2 = sched.tick_once(now, &mut fired).await.unwrap();
        assert_eq!(n2, 0);

        let calls = sink.0.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().any(|(_, b)| b.contains("AAPL")));
    }

    #[test]
    fn render_digest_appends_overflow_footer_when_truncated() {
        let events: Vec<MarketEvent> = (0..3).map(|i| ev(&format!("e{i}"), "AAPL")).collect();
        let body = render_digest(
            "盘前摘要 · 08:30",
            &events,
            7,
            crate::renderer::RenderFormat::Plain,
        );
        // 标题里的总数应为 events + overflow = 10 条
        assert!(body.contains("· 10 条"), "title 应显示总量,body = {body}");
        assert!(
            body.contains("另 7 条已省略"),
            "应附加 overflow footer,body = {body}"
        );
    }

    #[test]
    fn render_digest_omits_footer_when_no_overflow() {
        let events: Vec<MarketEvent> = (0..2).map(|i| ev(&format!("e{i}"), "AAPL")).collect();
        let body = render_digest("盘前摘要", &events, 0, crate::renderer::RenderFormat::Plain);
        assert!(!body.contains("已省略"), "无 overflow 时不应出现省略提示");
    }

    #[test]
    fn render_digest_recovers_social_title_from_raw_text() {
        let full = "JUST IN: Polymarket to launch 24/7 perpetual futures trading for crypto, equities, commodities, and FX markets next quarter.";
        let mut event = ev("social-1", "");
        event.kind = EventKind::SocialPost;
        event.title =
            "JUST IN: Polymarket to launch 24/7 perpetual futures trading for crypto, equiti…"
                .into();
        event.payload = serde_json::json!({ "raw_text": full });

        let body = render_digest(
            "盘前摘要 · 19:00",
            &[event],
            0,
            crate::renderer::RenderFormat::Plain,
        );

        assert!(body.contains(full), "body = {body}");
        assert!(!body.contains("equiti…"), "body = {body}");
    }

    #[test]
    fn render_digest_adds_compact_source_link_for_plain() {
        let mut event = ev("news-1", "AAPL");
        event.title = "Apple supplier update".into();
        event.url = Some("https://news.example.com/path/to/story".into());

        let body = render_digest(
            "盘前摘要 · 19:00",
            &[event],
            0,
            crate::renderer::RenderFormat::Plain,
        );

        assert!(body.contains("🔗 news.example.com"), "body = {body}");
        assert!(
            !body.contains("https://news.example.com/path/to/story"),
            "plain digest should not expand long source URLs: {body}"
        );
    }

    #[test]
    fn render_digest_adds_icon_link_for_telegram_and_discord() {
        let mut event = ev("news-1", "AAPL");
        event.url = Some("https://news.example.com/path/to/story".into());

        let telegram = render_digest(
            "盘前摘要 · 19:00",
            &[event.clone()],
            0,
            crate::renderer::RenderFormat::TelegramHtml,
        );
        assert!(
            telegram.contains(r#"<a href="https://news.example.com/path/to/story">🔗</a>"#),
            "telegram = {telegram}"
        );

        let discord = render_digest(
            "盘前摘要 · 19:00",
            &[event],
            0,
            crate::renderer::RenderFormat::DiscordMarkdown,
        );
        assert!(
            discord.contains("[🔗](https://news.example.com/path/to/story)"),
            "discord = {discord}"
        );
    }

    #[test]
    fn render_digest_feishu_post_uses_link_icon_element() {
        let mut event = ev("news-1", "AAPL");
        event.url = Some("https://news.example.com/path/to/story".into());

        let body = render_digest(
            "盘前摘要 · 19:00",
            &[event],
            0,
            crate::renderer::RenderFormat::FeishuPost,
        );
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            parsed
                .pointer("/zh_cn/content/0/5")
                .and_then(|v| v.get("tag"))
                .and_then(|v| v.as_str()),
            Some("a")
        );
        assert_eq!(
            parsed
                .pointer("/zh_cn/content/0/5")
                .and_then(|v| v.get("text"))
                .and_then(|v| v.as_str()),
            Some("🔗")
        );
        assert_eq!(
            parsed
                .pointer("/zh_cn/content/0/5")
                .and_then(|v| v.get("href"))
                .and_then(|v| v.as_str()),
            Some("https://news.example.com/path/to/story")
        );
    }

    #[test]
    fn curation_caps_social_and_source_noise() {
        let mut events = Vec::new();
        for (i, topic) in ["bitcoin", "ethereum", "fed", "oil", "tesla", "spacex"]
            .iter()
            .enumerate()
        {
            let mut event = ev(&format!("social-{i}"), "");
            event.kind = EventKind::SocialPost;
            event.severity = Severity::Low;
            event.source = "telegram.watcherguru".into();
            event.title = format!("JUST IN: {topic} market update");
            event.payload = serde_json::json!({ "raw_text": event.title });
            events.push(event);
        }

        let curated = curate_digest_events(events);
        assert_eq!(curated.len(), DIGEST_MAX_SOCIAL_ITEMS);
        assert!(
            curated
                .iter()
                .all(|e| matches!(e.kind, EventKind::SocialPost))
        );
    }

    #[test]
    fn curation_dedupes_repeated_news_titles() {
        let mut first = ev("news-1", "GEV");
        first.kind = EventKind::NewsCritical;
        first.severity = Severity::Medium;
        first.source = "fmp.stock_news:site-a.example".into();
        first.title = "GE Vernova stock soars as data center demand lifts outlook".into();
        first.url = Some("https://site-a.example/story".into());

        let mut duplicate = first.clone();
        duplicate.id = "news-2".into();
        duplicate.source = "fmp.stock_news:site-b.example".into();
        duplicate.url = Some("https://site-b.example/story".into());

        let mut distinct = first.clone();
        distinct.id = "news-3".into();
        distinct.title = "GE Vernova raises annual revenue forecast".into();
        distinct.url = Some("https://site-c.example/story".into());

        let curated = curate_digest_events(vec![first, duplicate, distinct]);
        let ids: Vec<&str> = curated.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["news-1", "news-3"]);
    }

    #[test]
    fn curation_dedupes_similar_same_symbol_news_titles() {
        let mut first = ev("news-1", "AMD");
        first.kind = EventKind::NewsCritical;
        first.severity = Severity::Medium;
        first.title = "AMD shares rally after data center demand lifts outlook".into();
        first.source = "fmp.stock_news:site-a.example".into();

        let mut similar = first.clone();
        similar.id = "news-2".into();
        similar.title = "AMD stock jumps as data center demand boosts outlook".into();
        similar.source = "fmp.stock_news:site-b.example".into();

        let curated = curate_digest_events(vec![first, similar]);
        assert_eq!(curated.len(), 1, "同 symbol 同主题相似标题应折叠");
    }

    #[test]
    fn digest_score_prefers_trusted_portfolio_signal_over_social_noise() {
        let mut social = ev("social-1", "");
        social.kind = EventKind::SocialPost;
        social.severity = Severity::Medium;
        social.source = "telegram.watcherguru".into();
        social.title = "JUST IN: generic crypto headline".into();

        let mut filing = ev("sec-1", "AAPL");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        filing.severity = Severity::Medium;
        filing.source = "sec.gov".into();
        filing.title = "AAPL files 8-K".into();

        assert!(digest_score(&filing) > digest_score(&social));
    }

    #[test]
    fn curation_keeps_high_items_even_when_caps_are_hit() {
        let mut events = Vec::new();
        for i in 0..DIGEST_MAX_ITEMS_PER_SYMBOL {
            let mut event = ev(&format!("aapl-low-{i}"), "AAPL");
            event.severity = Severity::Low;
            event.source = format!("source-{i}");
            events.push(event);
        }
        let mut high = ev("aapl-high", "AAPL");
        high.severity = Severity::High;
        high.title = "AAPL critical filing".into();
        high.source = "source-high".into();
        events.push(high);

        let curated = curate_digest_events(events);
        assert!(
            curated.iter().any(|e| e.id == "aapl-high"),
            "high severity digest item must not be dropped by curation caps"
        );
    }

    #[tokio::test]
    async fn scheduler_caps_batch_and_prioritizes_high_severity() {
        use crate::event::Severity;
        use crate::router::OutboundSink;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<String>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0.lock().unwrap().push(body.into());
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let sink = Arc::new(SpySink::default());
        let a = actor("u1");
        // 5 条 Low + 1 条 Medium;cap=3 应保留 1 Medium + 2 Low(排序后)
        for i in 0..5 {
            let mut e = ev(&format!("low-{i}"), "AAPL");
            e.severity = Severity::Low;
            e.title = format!("LOW-{i}");
            buf.enqueue(&a, &e).unwrap();
        }
        let mut mev = ev("mid-1", "AAPL");
        mev.severity = Severity::Medium;
        mev.title = "MID-KEEP".into();
        buf.enqueue(&a, &mev).unwrap();

        let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
            .with_tz_offset_hours(-4)
            .with_max_items_per_batch(3);
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
        let mut fired = HashSet::new();
        let n = sched.tick_once(now, &mut fired).await.unwrap();
        assert_eq!(n, 1);

        let calls = sink.0.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let body = &calls[0];
        // 总条数应为 6 (3 kept + 3 overflow)
        assert!(body.contains("· 6 条"), "body = {body}");
        // Medium 优先保留
        assert!(body.contains("MID-KEEP"), "Medium 应被保留,body = {body}");
        // 溢出提示
        assert!(body.contains("另 3 条已省略"), "body = {body}");
    }

    #[tokio::test]
    async fn scheduler_min_gap_skips_close_digest_windows_without_draining() {
        use crate::router::OutboundSink;
        use crate::store::EventStore;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<String>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0.lock().unwrap().push(body.into());
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let store = Arc::new(EventStore::open(dir.path().join("events.db")).unwrap());
        let sink = Arc::new(SpySink::default());
        let a = actor("u1");
        buf.enqueue(&a, &ev("first", "AAPL")).unwrap();

        let sched = DigestScheduler::new(buf.clone(), sink.clone(), "08:30", "12:00")
            .with_tz_offset_hours(8)
            .with_store(store)
            .with_min_gap_minutes(240);
        let mut fired = HashSet::new();
        let morning = Utc.with_ymd_and_hms(2026, 4, 21, 0, 30, 0).unwrap();
        assert_eq!(sched.tick_once(morning, &mut fired).await.unwrap(), 1);

        buf.enqueue(&a, &ev("second", "MSFT")).unwrap();
        let noon = Utc.with_ymd_and_hms(2026, 4, 21, 4, 0, 0).unwrap();
        assert_eq!(sched.tick_once(noon, &mut fired).await.unwrap(), 0);
        assert_eq!(
            buf.drain_actor(&a).unwrap().len(),
            1,
            "min-gap skip 不应 drain buffer"
        );
    }

    #[tokio::test]
    async fn scheduler_suppresses_recently_delivered_similar_topic() {
        use crate::router::OutboundSink;
        use crate::store::EventStore;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<String>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0.lock().unwrap().push(body.into());
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let store = Arc::new(EventStore::open(dir.path().join("events.db")).unwrap());
        let sink = Arc::new(SpySink::default());
        let a = actor("u1");
        let mut first = ev("news-amd-1", "AMD");
        first.kind = EventKind::NewsCritical;
        first.title = "AMD shares rally after data center demand lifts outlook".into();
        first.source = "fmp.stock_news:site-a.example".into();
        let mut second = first.clone();
        second.id = "news-amd-2".into();
        second.title = "AMD stock jumps as data center demand boosts outlook".into();
        second.source = "fmp.stock_news:site-b.example".into();

        store.insert_event(&first).unwrap();
        buf.enqueue(&a, &first).unwrap();
        let sched = DigestScheduler::new(buf.clone(), sink.clone(), "08:30", "09:00")
            .with_tz_offset_hours(8)
            .with_store(store.clone());
        let mut fired = HashSet::new();
        let morning = Utc.with_ymd_and_hms(2026, 4, 21, 0, 30, 0).unwrap();
        assert_eq!(sched.tick_once(morning, &mut fired).await.unwrap(), 1);

        store.insert_event(&second).unwrap();
        buf.enqueue(&a, &second).unwrap();
        let later = Utc.with_ymd_and_hms(2026, 4, 21, 1, 0, 0).unwrap();
        assert_eq!(sched.tick_once(later, &mut fired).await.unwrap(), 0);
        assert_eq!(
            sink.0.lock().unwrap().len(),
            1,
            "相似主题 24h 内不应再次形成摘要"
        );
    }

    #[tokio::test]
    async fn per_actor_windows_and_timezones_fire_independently() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
        use crate::router::OutboundSink;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<(String, String)>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0
                    .lock()
                    .unwrap()
                    .push((a.user_id.clone(), body.into()));
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let sink = Arc::new(SpySink::default());
        let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());

        let sh = actor("sh");
        let ny = actor("ny");
        // 给两人各 enqueue 一条 Medium
        buf.enqueue(&sh, &ev("e-sh", "AAPL")).unwrap();
        buf.enqueue(&ny, &ev("e-ny", "MSFT")).unwrap();

        // sh: 上海时区,只在本地 19:00 推一次。
        prefs
            .save(
                &sh,
                &NotificationPrefs {
                    timezone: Some("Asia/Shanghai".into()),
                    digest_windows: Some(vec!["19:00".into()]),
                    ..Default::default()
                },
            )
            .unwrap();
        // ny: 纽约时区,只在本地 07:00 推一次。
        prefs
            .save(
                &ny,
                &NotificationPrefs {
                    timezone: Some("America/New_York".into()),
                    digest_windows: Some(vec!["07:00".into()]),
                    ..Default::default()
                },
            )
            .unwrap();

        // 全局兜底窗口设个不会命中的 ("00:00") + 偏移设 0 即 UTC,确保命中由 prefs 决定。
        let sched = DigestScheduler::new(buf.clone(), sink.clone(), "00:00", "00:00")
            .with_tz_offset_hours(0)
            .with_prefs(prefs.clone());

        // T1: 2026-04-21 11:00 UTC == 19:00 上海 (CST=UTC+8) == 07:00 纽约 (EDT=UTC-4 in April)
        // 两个 actor 同时命中各自窗口。
        let now1 = Utc.with_ymd_and_hms(2026, 4, 21, 11, 0, 0).unwrap();
        let mut fired = HashSet::new();
        let n1 = sched.tick_once(now1, &mut fired).await.unwrap();
        assert_eq!(n1, 2, "两个 actor 各自命中本地窗口,应都 flush");

        let calls = sink.0.lock().unwrap();
        let users: Vec<&str> = calls.iter().map(|(u, _)| u.as_str()).collect();
        assert!(users.contains(&"sh"));
        assert!(users.contains(&"ny"));
        drop(calls);

        // 同一分钟再 tick 不重复
        let n_again = sched.tick_once(now1, &mut fired).await.unwrap();
        assert_eq!(n_again, 0);

        // T2: 同一天 ny actor 又来一条事件,sh 已经过 19:00 但还没到次日。
        // 23:00 UTC == 07:00 (next day) 上海 / 19:00 纽约 — 两边都不命中。
        buf.enqueue(&ny, &ev("e-ny-2", "GOOG")).unwrap();
        let now2 = Utc.with_ymd_and_hms(2026, 4, 21, 23, 0, 0).unwrap();
        let n2 = sched.tick_once(now2, &mut fired).await.unwrap();
        assert_eq!(n2, 0, "23:00 UTC 两个本地窗口都不命中");
    }

    #[tokio::test]
    async fn per_actor_empty_windows_disables_digest_entirely() {
        use crate::prefs::{FilePrefsStorage, NotificationPrefs, PrefsProvider};
        use crate::router::OutboundSink;
        use async_trait::async_trait;
        use std::collections::HashSet;
        use std::sync::Mutex;

        #[derive(Default)]
        struct SpySink(Mutex<Vec<String>>);
        #[async_trait]
        impl OutboundSink for SpySink {
            async fn send(&self, _a: &ActorIdentity, body: &str) -> anyhow::Result<()> {
                self.0.lock().unwrap().push(body.into());
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let sink = Arc::new(SpySink::default());
        let prefs = Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());

        let a = actor("quiet");
        buf.enqueue(&a, &ev("e1", "AAPL")).unwrap();
        prefs
            .save(
                &a,
                &NotificationPrefs {
                    digest_windows: Some(vec![]), // 显式关 digest
                    ..Default::default()
                },
            )
            .unwrap();

        // 把全局窗口设成 08:30,UTC 偏移 -4 → UTC 12:30 命中。但该 actor 应被 prefs 关闭。
        let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
            .with_tz_offset_hours(-4)
            .with_prefs(prefs);
        let now = Utc.with_ymd_and_hms(2026, 4, 21, 12, 30, 0).unwrap();
        let mut fired = HashSet::new();
        let n = sched.tick_once(now, &mut fired).await.unwrap();
        assert_eq!(n, 0, "digest_windows=Some(vec![]) 应彻底关 digest");
        assert!(sink.0.lock().unwrap().is_empty());
    }
}
