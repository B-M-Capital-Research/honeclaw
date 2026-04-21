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
    format!("{}__{}__{}", sanitize(&a.channel), sanitize(scope), sanitize(&a.user_id))
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

// ── Scheduler ───────────────────────────────────────────────────────────────

/// 判断 `now` 对应的本地时间（按 `offset_hours` 解释）是否处于给定 HH:MM 的 60 秒窗口内。
pub fn in_window(now: DateTime<Utc>, hhmm: &str, offset_hours: i32) -> bool {
    let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let Ok(target) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
        return false;
    };
    let now_t = NaiveTime::from_hms_opt(local.hour(), local.minute(), 0).unwrap();
    now_t == target
}

/// 当前本地日期（粗略）—— 用于 flush key 防止同一天重复触发。
pub fn local_date_key(now: DateTime<Utc>, offset_hours: i32) -> String {
    let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    format!("{:04}-{:02}-{:02}", local.year(), local.month(), local.day())
}

pub struct DigestScheduler {
    buffer: Arc<DigestBuffer>,
    sink: Arc<dyn crate::router::OutboundSink>,
    store: Option<Arc<crate::store::EventStore>>,
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

    /// 单轮 tick：检查当前时间是否命中两个窗口，命中则 flush 所有 actor。
    /// `already_fired_today` 传入"今日已触发过的 key"集合，防止 60s 分辨率下
    /// 同一分钟被 tick 两次。
    pub async fn tick_once(
        &self,
        now: DateTime<Utc>,
        already_fired_today: &mut std::collections::HashSet<String>,
    ) -> anyhow::Result<u32> {
        let date = local_date_key(now, self.tz_offset_hours);
        let mut flushed = 0u32;
        for window in [&self.pre_market, &self.post_market] {
            if !in_window(now, window, self.tz_offset_hours) {
                continue;
            }
            let fire_key = format!("{date}@{window}");
            if !already_fired_today.insert(fire_key) {
                continue; // 同一分钟已触发
            }
            let label = if window == &self.pre_market {
                format!("盘前摘要 · {window}")
            } else {
                // post_market 默认 09:00 北京时间，定位为"晨间简报，汇总隔夜美股"。
                format!("晨间摘要 · {window}")
            };
            for actor in self.buffer.list_pending_actors() {
                // 硬规则：只向单聊推送；若历史 buffer 遗留群 actor，直接 drain 丢弃。
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
                match self.buffer.drain_actor(&actor) {
                    Ok(events) if !events.is_empty() => {
                        // flush 时按最新 prefs 再过一遍：enqueue 后用户可能已关掉推送或
                        // 调整范围——这里是最后一道拦截。
                        let mut filtered: Vec<MarketEvent> = events
                            .into_iter()
                            .filter(|e| user_prefs.should_deliver(e))
                            .collect();
                        if filtered.is_empty() {
                            tracing::info!(
                                actor = %format!(
                                    "{}::{}::{}",
                                    actor.channel,
                                    actor.channel_scope.clone().unwrap_or_default(),
                                    actor.user_id
                                ),
                                "digest skipped by user prefs"
                            );
                            continue;
                        }
                        // 批次防轰炸：按 severity 降序(High→Medium→Low)再按 occurred_at 降序排序,
                        // 超过上限的尾部丢弃并在渲染时提示。
                        filtered.sort_by(|a, b| {
                            b.severity
                                .rank()
                                .cmp(&a.severity.rank())
                                .then_with(|| b.occurred_at.cmp(&a.occurred_at))
                        });
                        let overflow = if self.max_items_per_batch > 0
                            && filtered.len() > self.max_items_per_batch
                        {
                            let dropped = filtered.len() - self.max_items_per_batch;
                            filtered.truncate(self.max_items_per_batch);
                            tracing::info!(
                                actor = %format!(
                                    "{}::{}::{}",
                                    actor.channel,
                                    actor.channel_scope.clone().unwrap_or_default(),
                                    actor.user_id
                                ),
                                dropped,
                                kept = filtered.len(),
                                "digest truncated to avoid info flooding"
                            );
                            dropped
                        } else {
                            0
                        };
                        let body = render_digest(&label, &filtered, overflow, self.sink.format());
                        let actor_key = format!(
                            "{}::{}::{}",
                            actor.channel,
                            actor.channel_scope.clone().unwrap_or_default(),
                            actor.user_id
                        );
                        let send_result = self.sink.send(&actor, &body).await;
                        if let Some(store) = &self.store {
                            // 同一 digest body 覆盖多个 event_id；用合成 id 记录"flush 批次"。
                            let batch_id =
                                format!("digest-batch:{date}@{window}:{}", filtered.len());
                            let status = if send_result.is_ok() { "sent" } else { "failed" };
                            let _ = store.log_delivery(
                                &batch_id,
                                &actor_key,
                                "digest",
                                filtered[0].severity,
                                status,
                                Some(&body),
                            );
                        }
                        if let Err(e) = send_result {
                            tracing::warn!("digest sink failed: {e:#}");
                            continue;
                        }
                        flushed += 1;
                    }
                    Ok(_) => {}
                    Err(e) => tracing::warn!("drain_actor failed: {e:#}"),
                }
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
        out.push_str(&format!("…… 另 {overflow} 条已省略（优先展示高优先级/最新）"));
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
                self.0.lock().unwrap().push((a.user_id.clone(), body.into()));
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let buf = Arc::new(DigestBuffer::new(dir.path().join("digest")).unwrap());
        let sink = Arc::new(SpySink::default());
        let prefs =
            Arc::new(FilePrefsStorage::new(dir.path().join("prefs")).unwrap());
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
                self.0.lock().unwrap().push((a.user_id.clone(), body.into()));
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
        let sched = DigestScheduler::new(buf, sink.clone(), "08:30", "17:00")
            .with_tz_offset_hours(-4);
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
        let body = render_digest(
            "盘前摘要",
            &events,
            0,
            crate::renderer::RenderFormat::Plain,
        );
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
}
