//! DigestBuffer — 按 actor 缓存 Medium/Low 事件，定时合并推送。
//!
//! 存储：`{buffer_dir}/{actor_key}.jsonl`，一条事件一行，append-only。
//! Flush：`drain_actor` 把文件读空并 rotate（改名加时间戳），调用方负责渲染 + 推送。
//!
//! MVP 时区处理：用固定小时偏移（见 `tz_offset_hours`），默认 Asia/Shanghai（UTC+8）。
//! 夏/冬令时按常用区域近似；接 `chrono-tz` 后替换。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Datelike, FixedOffset, NaiveTime, TimeZone, Timelike, Utc};
use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

use crate::event::MarketEvent;
use crate::router::body_preview;

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
        }
    }

    /// 设置单批最多渲染多少条事件。0 表示不截断。
    pub fn with_max_items_per_batch(mut self, n: usize) -> Self {
        self.max_items_per_batch = n;
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
                    b.severity
                        .rank()
                        .cmp(&a.severity.rank())
                        .then_with(|| b.occurred_at.cmp(&a.occurred_at))
                });
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
                            kept = filtered.len(),
                            dropped_ids = ?dropped_ids,
                            "digest truncated to avoid info flooding"
                        );
                        dropped
                    } else {
                        0
                    };
                let body = render_digest(&label, &filtered, overflow, self.sink.format());
                let send_result = self.sink.send(&actor, &body).await;
                if let Some(store) = &self.store {
                    let batch_id = format!("digest-batch:{date}@{window}:{}", filtered.len());
                    let status = if send_result.is_ok() {
                        "sent"
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
                }
                if let Err(e) = send_result {
                    tracing::warn!(
                        actor = %actor_key_str,
                        window = %window,
                        items = filtered.len(),
                        body_len = body.chars().count(),
                        body_preview = %body_preview(&body),
                        "digest sink failed: {e:#}"
                    );
                    continue;
                }
                tracing::info!(
                    actor = %actor_key_str,
                    window = %window,
                    items = filtered.len(),
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
    };
    let mut out = title;
    for ev in events {
        let head = crate::renderer::header_line_compact(ev);
        let title_inline = crate::renderer::render_inline(&ev.title, fmt);
        let head_inline = crate::renderer::render_inline(&head, fmt);
        out.push('\n');
        if head_inline.is_empty() {
            out.push_str(&format!("• {title_inline}"));
        } else {
            out.push_str(&format!("• {head_inline} · {title_inline}"));
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
