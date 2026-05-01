//! EventStore — SQLite 持久化与去重，附 JSONL 镜像 + append-only 推送审计。
//!
//! 表结构：
//! - `events (id PK, kind, severity, symbols_json, occurred_at_ts, title, summary,
//!            url, source, payload_json, created_at_ts)`
//! - `engine_meta (key PK, value)` — 存 `baseline_at_ts` 等单例标量
//! - `delivery_log (rowid AUTOINCREMENT, event_id, actor, channel, severity,
//!                  sent_at_ts, status, body)` — **append-only** 推送审计
//!
//! 幂等语义：`insert_event` 使用 `INSERT OR IGNORE`；同 id 只落一次。
//! baseline：首次打开 DB 时写入 `baseline_at_ts = now`，之后读取；低于 baseline
//! 的事件由调用方根据语义决定是否入库/推送（store 层不拦截）。
//!
//! JSONL 镜像：`with_jsonl_path(...)` 可选，`insert_event` 新写入时同步 append
//! 一行完整事件 JSON；用于 SQLite 损坏时的人肉回放。
//!
//! 清理：`purge_events_older_than(days)` 按 `created_at_ts` 删除旧 events；
//! delivery_log 单独按 `sent_at_ts` 清。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use crate::event::MarketEvent;

pub struct EventStore {
    conn: Mutex<Connection>,
    jsonl_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct DeliveryLogFilter {
    pub since_ts: Option<i64>,
    pub until_ts: Option<i64>,
    pub actor: Option<String>,
    pub actor_channel: Option<String>,
    pub actor_user_id: Option<String>,
    pub event_id: Option<String>,
    pub status: Option<String>,
    pub delivery_channel: Option<String>,
    pub top_level_only: bool,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub struct DeliveryLogRecord {
    pub id: i64,
    pub event_id: String,
    pub actor: String,
    pub channel: String,
    pub severity: String,
    pub sent_at_ts: i64,
    pub status: String,
    pub body: Option<String>,
    pub event_title: Option<String>,
    pub event_summary: Option<String>,
    pub event_kind: Option<String>,
    pub event_source: Option<String>,
    pub event_url: Option<String>,
    pub event_symbols: Vec<String>,
}

impl EventStore {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id              TEXT PRIMARY KEY,
                kind_json       TEXT NOT NULL,
                severity        TEXT NOT NULL,
                symbols_json    TEXT NOT NULL,
                occurred_at_ts  INTEGER NOT NULL,
                title           TEXT NOT NULL,
                summary         TEXT NOT NULL,
                url             TEXT,
                source          TEXT NOT NULL,
                payload_json    TEXT NOT NULL,
                created_at_ts   INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_events_occurred_at
                ON events(occurred_at_ts);
            CREATE INDEX IF NOT EXISTS idx_events_source
                ON events(source);

            CREATE TABLE IF NOT EXISTS engine_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS delivery_log (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id   TEXT NOT NULL,
                actor      TEXT NOT NULL,
                channel    TEXT NOT NULL,
                severity   TEXT NOT NULL,
                sent_at_ts INTEGER NOT NULL,
                status     TEXT NOT NULL,
                body       TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_delivery_event_actor
                ON delivery_log(event_id, actor, sent_at_ts);
            CREATE INDEX IF NOT EXISTS idx_delivery_sent_at
                ON delivery_log(sent_at_ts);
            "#,
        )?;
        let store = Self {
            conn: Mutex::new(conn),
            jsonl_path: None,
        };
        store.ensure_baseline(Utc::now())?;
        Ok(store)
    }

    /// 开启 JSONL 镜像：每次新事件入库后，把完整事件 JSON 追加一行到
    /// 指定文件；用作 SQLite 故障时的人肉兜底。
    pub fn with_jsonl_path(mut self, path: impl Into<PathBuf>) -> Self {
        let p = path.into();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        self.jsonl_path = Some(p);
        self
    }

    fn ensure_baseline(&self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO engine_meta(key, value) VALUES ('baseline_at_ts', ?1)",
            params![now.timestamp()],
        )?;
        Ok(())
    }

    pub fn baseline_at(&self) -> anyhow::Result<DateTime<Utc>> {
        let conn = self.conn.lock().unwrap();
        let ts: Option<i64> = conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM engine_meta WHERE key='baseline_at_ts'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let ts = ts.ok_or_else(|| anyhow::anyhow!("baseline 未初始化"))?;
        Utc.timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("baseline 时间戳无效: {ts}"))
    }

    /// 插入一条事件。若 `id` 已存在，返回 `Ok(false)`；首次写入返回 `Ok(true)`。
    /// 首次写入成功 + 启用了 JSONL 镜像时，同步 append 一行事件 JSON；写失败只
    /// 记 warn，不影响 SQLite 事务结果。
    pub fn insert_event(&self, ev: &MarketEvent) -> anyhow::Result<bool> {
        let affected = {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                r#"
                INSERT OR IGNORE INTO events (
                    id, kind_json, severity, symbols_json, occurred_at_ts,
                    title, summary, url, source, payload_json, created_at_ts
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    ev.id,
                    serde_json::to_string(&ev.kind)?,
                    severity_tag(&ev.severity),
                    serde_json::to_string(&ev.symbols)?,
                    ev.occurred_at.timestamp(),
                    ev.title,
                    ev.summary,
                    ev.url,
                    ev.source,
                    serde_json::to_string(&ev.payload)?,
                    Utc::now().timestamp(),
                ],
            )?
        };
        let is_new = affected > 0;
        if is_new {
            if let Err(e) = self.append_jsonl_mirror(ev) {
                tracing::warn!("events jsonl mirror append failed: {e:#}");
            }
        }
        Ok(is_new)
    }

    fn append_jsonl_mirror(&self, ev: &MarketEvent) -> anyhow::Result<()> {
        let Some(path) = self.jsonl_path.as_ref() else {
            return Ok(());
        };
        use std::io::Write;
        let line = serde_json::to_string(ev)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    /// 按 `created_at_ts` 删除早于 `cutoff_days` 天的 events，返回删除行数。
    /// delivery_log 单独按 `sent_at_ts` 清，`purge_delivery_log_older_than`。
    pub fn purge_events_older_than(&self, cutoff_days: i64) -> anyhow::Result<usize> {
        let cutoff = Utc::now().timestamp() - cutoff_days * 86_400;
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM events WHERE created_at_ts < ?1",
            params![cutoff],
        )?;
        Ok(n)
    }

    pub fn purge_delivery_log_older_than(&self, cutoff_days: i64) -> anyhow::Result<usize> {
        let cutoff = Utc::now().timestamp() - cutoff_days * 86_400;
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM delivery_log WHERE sent_at_ts < ?1",
            params![cutoff],
        )?;
        Ok(n)
    }

    pub fn count_events(&self) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        Ok(n)
    }

    /// 列出 `[start, end]` 窗口内、`symbol` 命中的事件的 kind tag (snake_case
    /// 字符串,如 `"price_alert"` / `"earnings_released"` / `"sec_filing"`)。
    ///
    /// 用途:
    /// - 新闻多信号合流:`[news_ts - 12h, news_ts + 1h]` 查硬信号
    /// - 财报窗口升级:`[news_ts - 1d, news_ts + 2d]` 查 earnings_upcoming /
    ///   earnings_released (含未来财报日)
    ///
    /// 注意:`occurred_at` 是**事件真实发生时刻**,不是入库时刻——所以
    /// `earnings_upcoming` 在财报日当天 00:00,查询窗口必须向未来延伸才能命中。
    pub fn symbol_signal_kinds_in_window(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT kind_json FROM events
            WHERE occurred_at_ts >= ?1 AND occurred_at_ts <= ?2
              AND symbols_json LIKE ?3
            "#,
        )?;
        let rows = stmt.query_map(params![start.timestamp(), end.timestamp(), needle], |row| {
            row.get::<_, String>(0)
        })?;
        let mut out: Vec<String> = Vec::new();
        for r in rows {
            let json = r?;
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(t) = v.get("type").and_then(|v| v.as_str()) {
                    out.push(t.to_string());
                }
            }
        }
        Ok(out)
    }

    /// 历史兼容:旧的 "since 12h" 语义 shim,内部委派给窗口查询。
    pub fn today_signal_kinds(
        &self,
        symbol: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        self.symbol_signal_kinds_in_window(symbol, since, Utc::now())
    }

    /// 列出未来 `within_days` 天内的 `EarningsUpcoming` teaser 事件。
    ///
    /// 用于 `UnifiedDigestScheduler` 在每个 slot 触发时把"今天应该提醒 T-3/T-2/T-1"
    /// 的财报现算出来(见 `pollers::earnings::synthesize_countdowns`),这样
    /// 即使 poller 的 cron tick 漂移也不会让倒计时 off-by-one。
    pub fn list_upcoming_earnings(
        &self,
        now: DateTime<Utc>,
        within_days: i64,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let start = now.timestamp();
        let end = (now + chrono::Duration::days(within_days)).timestamp();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, kind_json, severity, symbols_json, occurred_at_ts,
                   title, summary, url, source, payload_json
            FROM events
            WHERE occurred_at_ts >= ?1 AND occurred_at_ts <= ?2
              AND kind_json LIKE '%"earnings_upcoming"%'
            "#,
        )?;
        let rows = stmt.query_map(params![start, end], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, kind_json, sev, syms_json, ts, title, summary, url, source, payload_json) = r?;
            let Ok(kind) = serde_json::from_str(&kind_json) else {
                continue;
            };
            let severity = match sev.as_str() {
                "high" => crate::event::Severity::High,
                "medium" => crate::event::Severity::Medium,
                _ => crate::event::Severity::Low,
            };
            let symbols: Vec<String> = serde_json::from_str(&syms_json).unwrap_or_default();
            let payload: serde_json::Value =
                serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);
            let Some(occurred_at) = DateTime::<Utc>::from_timestamp(ts, 0) else {
                continue;
            };
            out.push(MarketEvent {
                id,
                kind,
                severity,
                symbols,
                occurred_at,
                title,
                summary,
                url,
                source,
                payload,
            });
        }
        Ok(out)
    }

    /// 该 actor 在 `[since, now]` 窗口内通过 sink 成功送达的 High 事件数。
    /// 用于 Router 执行 `high_severity_daily_cap` 硬上限:超了自动降级到 digest,
    /// 避免同一天被同一股票的 8-K / 财报 / 价格异动轮番轰炸。
    pub fn count_high_sent_since(&self, actor: &str, since: DateTime<Utc>) -> anyhow::Result<i64> {
        self.count_high_sent_since_for_category(actor, since, "all")
    }

    /// 该 actor 在 `[since, now]` 窗口内某一事件类别通过 sink 成功送达的 High 数。
    /// `category="all"` 维持旧语义；其它类别用于把 price/news/filing/earnings/macro
    /// 的 high cap 分桶，避免互相挤占。
    pub fn count_high_sent_since_for_category(
        &self,
        actor: &str,
        since: DateTime<Utc>,
        category: &str,
    ) -> anyhow::Result<i64> {
        if category == "all" {
            return self.count_high_sent_since_all(actor, since);
        }
        let Some(tags) = category_kind_tags(category) else {
            return self.count_high_sent_since_all(actor, since);
        };
        let predicates = vec!["e.kind_json LIKE ?"; tags.len()].join(" OR ");
        let sql = format!(
            r#"
            SELECT COUNT(*) FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?
              AND d.severity = 'high'
              AND d.status = 'sent'
              AND d.channel = 'sink'
              AND d.sent_at_ts >= ?
              AND ({predicates})
            "#
        );
        let mut values = Vec::with_capacity(2 + tags.len());
        values.push(SqlValue::Text(actor.to_string()));
        values.push(SqlValue::Integer(since.timestamp()));
        for tag in tags {
            values.push(SqlValue::Text(format!("%\"{tag}\"%")));
        }
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row(&sql, params_from_iter(values), |row| row.get(0))?;
        Ok(n)
    }

    fn count_high_sent_since_all(&self, actor: &str, since: DateTime<Utc>) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row(
            r#"
            SELECT COUNT(*) FROM delivery_log
            WHERE actor = ?1
              AND severity = 'high'
              AND status = 'sent'
              AND channel = 'sink'
              AND sent_at_ts >= ?2
            "#,
            params![actor, since.timestamp()],
            |row| row.get(0),
        )?;
        Ok(n)
    }

    /// 该 actor 针对 `symbol` 最近一次 High 成功送达 sink 的时刻。
    /// 用于 Router 对同一 ticker 的短时冷却:防止 5 分钟内价格异动 + 新闻 + 盈利三连推。
    /// 返回 None 表示该 symbol 在 delivery_log 里从未命中 High+sent+sink。
    pub fn last_high_sink_send_for_symbol(
        &self,
        actor: &str,
        symbol: &str,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        self.last_high_sink_send_for_symbol_category(actor, symbol, "all", None)
    }

    /// 该 actor 针对 symbol + category 最近一次 High 成功送达 sink 的时刻。
    /// `firm` 仅当 category 命中 kind 列表时附加 `payload_json.gradingCompany` 过滤,
    /// 用于把 AnalystGrade 的冷却 key 拆到 (symbol, firm) 粒度,这样同 ticker 不同
    /// 投行同分钟到达不会互相冷却。其他 category 一律传 `None`。
    pub fn last_high_sink_send_for_symbol_category(
        &self,
        actor: &str,
        symbol: &str,
        category: &str,
        firm: Option<&str>,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        if category == "all" {
            return self.last_high_sink_send_for_symbol_all(actor, symbol);
        }
        let Some(tags) = category_kind_tags(category) else {
            return self.last_high_sink_send_for_symbol_all(actor, symbol);
        };
        let predicates = vec!["e.kind_json LIKE ?"; tags.len()].join(" OR ");
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let firm_clause = if firm.is_some() {
            "AND json_extract(e.payload_json, '$.gradingCompany') = ?"
        } else {
            ""
        };
        let sql = format!(
            r#"
            SELECT MAX(d.sent_at_ts) FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?
              AND d.severity = 'high'
              AND d.status = 'sent'
              AND d.channel = 'sink'
              AND e.symbols_json LIKE ?
              AND ({predicates})
              {firm_clause}
            "#
        );
        let mut values = Vec::with_capacity(2 + tags.len() + firm.is_some() as usize);
        values.push(SqlValue::Text(actor.to_string()));
        values.push(SqlValue::Text(needle));
        for tag in tags {
            values.push(SqlValue::Text(format!("%\"{tag}\"%")));
        }
        if let Some(f) = firm {
            values.push(SqlValue::Text(f.to_string()));
        }
        let conn = self.conn.lock().unwrap();
        let row: Option<i64> = conn.query_row(&sql, params_from_iter(values), |row| {
            row.get::<_, Option<i64>>(0)
        })?;
        Ok(row.and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0)))
    }

    fn last_high_sink_send_for_symbol_all(
        &self,
        actor: &str,
        symbol: &str,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let conn = self.conn.lock().unwrap();
        let row: Option<i64> = conn.query_row(
            r#"
            SELECT MAX(d.sent_at_ts) FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?1
              AND d.severity = 'high'
              AND d.status = 'sent'
              AND d.channel = 'sink'
              AND e.symbols_json LIKE ?2
            "#,
            params![actor, needle],
            |row| row.get::<_, Option<i64>>(0),
        )?;
        Ok(row.and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0)))
    }

    /// 返回 `since` 之后 actor 在 (symbol, direction) 上**已被 sink 推过的最大
    /// band bps**(从 `price_band:SYM:DATE:up:BPS` 的 id 末段解析)。供 dispatch
    /// 的「monotone 新高 + N」单一推送规则用 —— 新档 pct 必须比该值高出
    /// `price_band_min_advance_pct` 才允许直推,否则降级 digest。
    pub fn last_price_band_max_bps_for_symbol_direction(
        &self,
        actor: &str,
        symbol: &str,
        direction: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Option<i64>> {
        let Some(pattern) = price_band_id_pattern(symbol, direction) else {
            return Ok(None);
        };
        let needle = format!("%\"{}\"%", symbol.to_uppercase());
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?1
              AND d.severity = 'high'
              AND d.status = 'sent'
              AND d.channel = 'sink'
              AND d.sent_at_ts >= ?2
              AND e.symbols_json LIKE ?3
              AND e.id LIKE ?4
            "#,
        )?;
        let rows = stmt.query_map(params![actor, since.timestamp(), needle, pattern], |row| {
            row.get::<_, String>(0)
        })?;
        let mut max_bps: Option<i64> = None;
        for r in rows {
            let id = r?;
            if let Some(bps) = parse_bps_from_band_id(&id) {
                max_bps = Some(max_bps.map_or(bps, |m| m.max(bps)));
            }
        }
        Ok(max_bps)
    }

    pub fn last_digest_success_at(&self, actor: &str) -> anyhow::Result<Option<DateTime<Utc>>> {
        let conn = self.conn.lock().unwrap();
        let row: Option<i64> = conn.query_row(
            r#"
            SELECT MAX(sent_at_ts) FROM delivery_log
            WHERE actor = ?1
              AND channel = 'digest'
              AND status IN ('sent', 'dryrun')
            "#,
            params![actor],
            |row| row.get::<_, Option<i64>>(0),
        )?;
        Ok(row.and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0)))
    }

    /// 列出 `since` 之后某 actor 在 digest 流程里**被吞掉**的事件 + 各自的吞掉
    /// 原因(curation 噪音过滤 / 单批数量上限 / 同 ticker 冷却 / 用户 prefs 过滤
    /// 等)。供 `/missed` 斜杠命令查询。
    ///
    /// status 取值含义:
    /// - `omitted` —— 被 curation 砍(per-symbol/source/domain cap、jaccard 同主题、
    ///   `should_omit_from_digest` 把 opinion_blog 等噪音砍了)
    /// - `capped` / `price_capped` —— `max_items_per_batch` 单批数量上限截断
    /// - `cooled_down` / `price_cooled_down` —— 同 ticker / 同 symbol 冷却命中
    /// - `filtered` —— 用户 prefs(`should_deliver`)主动过滤
    pub fn list_missed_digest_items_since(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<(MarketEvent, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.kind_json, e.severity, e.symbols_json, e.occurred_at_ts,
                   e.title, e.summary, e.url, e.source, e.payload_json, d.status
            FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?1
              AND d.channel IN ('digest_item', 'prefs')
              AND d.status NOT IN ('sent', 'dryrun', 'queued')
              AND d.sent_at_ts >= ?2
            ORDER BY d.sent_at_ts DESC
            "#,
        )?;
        let rows = stmt.query_map(params![actor, since.timestamp()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (
                id,
                kind_json,
                sev,
                syms_json,
                ts,
                title,
                summary,
                url,
                source,
                payload_json,
                status,
            ) = r?;
            let Ok(kind) = serde_json::from_str(&kind_json) else {
                continue;
            };
            let severity = match sev.as_str() {
                "high" => crate::event::Severity::High,
                "medium" => crate::event::Severity::Medium,
                _ => crate::event::Severity::Low,
            };
            let symbols: Vec<String> = serde_json::from_str(&syms_json).unwrap_or_default();
            let payload: serde_json::Value =
                serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);
            let Some(occurred_at) = DateTime::<Utc>::from_timestamp(ts, 0) else {
                continue;
            };
            out.push((
                MarketEvent {
                    id,
                    kind,
                    severity,
                    symbols,
                    occurred_at,
                    title,
                    summary,
                    url,
                    source,
                    payload,
                },
                status,
            ));
        }
        Ok(out)
    }

    /// 列出 `since` 之后某 actor 已经成功推送过的 event_id 集合。
    /// 与 `list_recent_digest_item_events` 的区别:这里**不 JOIN events 表**,
    /// 因此能覆盖"只在 delivery_log 留痕"的合成事件(例如
    /// `digest.synth.earnings_countdown`,scheduler 自己造的事件不会写
    /// `events` 表)。专供 synth 跨 flush 去重用。
    pub fn delivered_event_ids_since(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<std::collections::HashSet<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT DISTINCT event_id FROM delivery_log
            WHERE actor = ?1
              AND status IN ('sent', 'dryrun')
              AND sent_at_ts >= ?2
            "#,
        )?;
        let rows = stmt.query_map(params![actor, since.timestamp()], |row| {
            row.get::<_, String>(0)
        })?;
        let mut out = std::collections::HashSet::new();
        for r in rows.flatten() {
            out.insert(r);
        }
        Ok(out)
    }

    /// 列出在 `since` 之后有 `quiet_held` 行的 distinct actor key。供 UnifiedDigestScheduler
    /// 在 quiet.to 分钟把这些 actor 也加入 tick 迭代集合 —— 否则只 buffer 为空、
    /// 仅靠 router hold 的 actor 永远等不到 quiet_flush。
    pub fn list_actors_with_quiet_held_since(
        &self,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT DISTINCT actor FROM delivery_log
            WHERE channel = 'sink'
              AND status = 'quiet_held'
              AND sent_at_ts >= ?1
            "#,
        )?;
        let rows = stmt.query_map(params![since.timestamp()], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 列出某 actor 在 `since` 之后被 router 因 quiet_hours hold 住的事件。
    /// 用于 `quiet_flush` 在 `quiet_hours.to` 时刻把这批事件按保鲜期筛选后合并发送。
    /// 返回 `(MarketEvent, sent_at_ts)`，`sent_at_ts` 是当初被 hold 时刻（用于排序）。
    /// LEFT JOIN 风格：events 表里查不到的 hold 行（synth/已被清理）直接跳过。
    pub fn list_quiet_held_since(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<(MarketEvent, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.kind_json, e.severity, e.symbols_json, e.occurred_at_ts,
                   e.title, e.summary, e.url, e.source, e.payload_json, d.sent_at_ts
            FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?1
              AND d.channel = 'sink'
              AND d.status = 'quiet_held'
              AND d.sent_at_ts >= ?2
            ORDER BY d.sent_at_ts ASC
            "#,
        )?;
        let rows = stmt.query_map(params![actor, since.timestamp()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (
                id,
                kind_json,
                sev,
                syms_json,
                ts,
                title,
                summary,
                url,
                source,
                payload_json,
                sent_at,
            ) = r?;
            let Ok(kind) = serde_json::from_str(&kind_json) else {
                continue;
            };
            let severity = match sev.as_str() {
                "high" => crate::event::Severity::High,
                "medium" => crate::event::Severity::Medium,
                _ => crate::event::Severity::Low,
            };
            let symbols: Vec<String> = serde_json::from_str(&syms_json).unwrap_or_default();
            let payload: serde_json::Value =
                serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);
            let Some(occurred_at) = DateTime::<Utc>::from_timestamp(ts, 0) else {
                continue;
            };
            out.push((
                MarketEvent {
                    id,
                    kind,
                    severity,
                    symbols,
                    occurred_at,
                    title,
                    summary,
                    url,
                    source,
                    payload,
                },
                sent_at,
            ));
        }
        Ok(out)
    }

    pub fn list_recent_digest_item_events(
        &self,
        actor: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT e.id, e.kind_json, e.severity, e.symbols_json, e.occurred_at_ts,
                   e.title, e.summary, e.url, e.source, e.payload_json
            FROM delivery_log d
            JOIN events e ON d.event_id = e.id
            WHERE d.actor = ?1
              AND d.channel = 'digest_item'
              AND d.status IN ('sent', 'dryrun')
              AND d.sent_at_ts >= ?2
            "#,
        )?;
        let rows = stmt.query_map(params![actor, since.timestamp()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, kind_json, sev, syms_json, ts, title, summary, url, source, payload_json) = r?;
            let Ok(kind) = serde_json::from_str(&kind_json) else {
                continue;
            };
            let severity = match sev.as_str() {
                "high" => crate::event::Severity::High,
                "medium" => crate::event::Severity::Medium,
                _ => crate::event::Severity::Low,
            };
            let symbols: Vec<String> = serde_json::from_str(&syms_json).unwrap_or_default();
            let payload: serde_json::Value =
                serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);
            let Some(occurred_at) = DateTime::<Utc>::from_timestamp(ts, 0) else {
                continue;
            };
            out.push(MarketEvent {
                id,
                kind,
                severity,
                symbols,
                occurred_at,
                title,
                summary,
                url,
                source,
                payload,
            });
        }
        Ok(out)
    }

    /// 按 `occurred_at_ts` 拉一段窗口内的 News + Macro 事件,供 global_digest
    /// collector 二次过滤(source_class / legal_ad / 已广播)。**不**做 source class
    /// 解析——那是 collector 的职责;这里只做 SQL 层能高效完成的过滤
    /// (kind / severity / 时间窗口 / source 前缀)。
    ///
    /// **severity 门槛非对称**(2026-04-27 POC 复盘后调整):
    /// - RSS 源(Bloomberg/SpaceNews/STAT 等):无脑 High,severity 不再二次过滤
    /// - FMP `trusted` 域(reuters/wsj/cnbc/marketwatch 等):允许 Low 进入候选池
    ///   —— `pollers::news::classify_severity` 只在命中 distress/M&A 关键词时才升 High,
    ///   导致 GOOGL 财报预告、Tokyo Electron 半导体上下游等 thesis 硬料被砍。
    ///   POC 实测 24h 多出 19 条 trusted-Low,其中 ~25% 是 thesis 相关硬料,
    ///   工作日扩量 ~80-180 条仍在 Pass1 prompt 容量内。
    /// - FMP 非 trusted 域(opinion_blog / pr_wire / uncertain):仍按 high/medium 严格门槛,
    ///   防止 seekingalpha listicle、律所 PR 灌进来。
    pub fn list_global_digest_news_candidates(
        &self,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MarketEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, kind_json, severity, symbols_json, occurred_at_ts,
                   title, summary, url, source, payload_json
            FROM events
            WHERE occurred_at_ts >= ?1
              AND occurred_at_ts < ?2
              AND kind_json LIKE '%news_critical%'
              AND (
                    source LIKE 'rss:%'
                 OR (source LIKE 'fmp.stock_news:%' AND severity IN ('high', 'medium'))
                 OR (source LIKE 'fmp.stock_news:%'
                     AND json_extract(payload_json, '$.source_class') = 'trusted')
              )
            ORDER BY occurred_at_ts DESC
            "#,
        )?;
        let rows = stmt.query_map(params![since.timestamp(), until.timestamp()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, kind_json, sev, syms_json, ts, title, summary, url, source, payload_json) = r?;
            let Ok(kind) = serde_json::from_str(&kind_json) else {
                continue;
            };
            let severity = match sev.as_str() {
                "high" => crate::event::Severity::High,
                "medium" => crate::event::Severity::Medium,
                _ => crate::event::Severity::Low,
            };
            let symbols: Vec<String> = serde_json::from_str(&syms_json).unwrap_or_default();
            let payload: serde_json::Value =
                serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);
            let Some(occurred_at) = DateTime::<Utc>::from_timestamp(ts, 0) else {
                continue;
            };
            out.push(MarketEvent {
                id,
                kind,
                severity,
                symbols,
                occurred_at,
                title,
                summary,
                url,
                source,
                payload,
            });
        }
        Ok(out)
    }

    /// 列出某 channel 在 `since` 之后所有 actor 的成功投递 event_id 集合。
    /// 用于 global_digest 跨批次去重——一旦某条新闻被某次 broadcast 推过,后续
    /// 批次不再纳入候选池。
    pub fn broadcasted_event_ids_since(
        &self,
        channel: &str,
        since: DateTime<Utc>,
    ) -> anyhow::Result<std::collections::HashSet<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT DISTINCT event_id FROM delivery_log
            WHERE channel = ?1
              AND status IN ('sent', 'dryrun')
              AND sent_at_ts >= ?2
            "#,
        )?;
        let rows = stmt.query_map(params![channel, since.timestamp()], |row| {
            row.get::<_, String>(0)
        })?;
        Ok(rows.flatten().collect())
    }

    /// Append-only 追加一条推送审计。同一 (event, actor) 可以多行，表达
    /// queued → sent / failed 等状态迁移。`body` 是实际下发给 sink 的正文（含
    /// LLM 润色后的结果），用于回放对账；digest 入队阶段传 `None`（flush
    /// 时再写入渲染后的 digest 正文）。
    pub fn log_delivery(
        &self,
        event_id: &str,
        actor: &str,
        channel: &str,
        severity: crate::event::Severity,
        status: &str,
        body: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO delivery_log
              (event_id, actor, channel, severity, sent_at_ts, status, body)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                event_id,
                actor,
                channel,
                severity_tag(&severity),
                Utc::now().timestamp(),
                status,
                body,
            ],
        )?;
        Ok(())
    }

    pub fn list_recent_delivery_logs(
        &self,
        filter: &DeliveryLogFilter,
    ) -> anyhow::Result<Vec<DeliveryLogRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            r#"
            SELECT
                d.id, d.event_id, d.actor, d.channel, d.severity,
                d.sent_at_ts, d.status, d.body,
                e.title, e.summary, e.kind_json, e.source, e.url, e.symbols_json
            FROM delivery_log d
            LEFT JOIN events e ON e.id = d.event_id
            WHERE 1=1
            "#,
        );
        let mut values: Vec<SqlValue> = Vec::new();

        if let Some(ts) = filter.since_ts {
            sql.push_str(" AND d.sent_at_ts >= ?");
            values.push(SqlValue::Integer(ts));
        }
        if let Some(ts) = filter.until_ts {
            sql.push_str(" AND d.sent_at_ts <= ?");
            values.push(SqlValue::Integer(ts));
        }
        if let Some(actor) = filter.actor.as_deref().filter(|v| !v.is_empty()) {
            sql.push_str(" AND d.actor = ?");
            values.push(SqlValue::Text(actor.to_string()));
        } else {
            if let Some(channel) = filter.actor_channel.as_deref().filter(|v| !v.is_empty()) {
                sql.push_str(" AND d.actor LIKE ?");
                values.push(SqlValue::Text(format!("{channel}::%")));
            }
            if let Some(user_id) = filter.actor_user_id.as_deref().filter(|v| !v.is_empty()) {
                sql.push_str(" AND d.actor LIKE ?");
                values.push(SqlValue::Text(format!("%::{user_id}")));
            }
        }
        if let Some(event_id) = filter.event_id.as_deref().filter(|v| !v.is_empty()) {
            sql.push_str(" AND d.event_id = ?");
            values.push(SqlValue::Text(event_id.to_string()));
        }
        if let Some(status) = filter.status.as_deref().filter(|v| !v.is_empty()) {
            sql.push_str(" AND d.status = ?");
            values.push(SqlValue::Text(status.to_string()));
        }
        if let Some(channel) = filter.delivery_channel.as_deref().filter(|v| !v.is_empty()) {
            sql.push_str(" AND d.channel = ?");
            values.push(SqlValue::Text(channel.to_string()));
        }
        if filter.top_level_only {
            sql.push_str(" AND d.channel NOT IN ('router', 'digest_item', 'global_digest_item')");
        }

        sql.push_str(" ORDER BY d.sent_at_ts DESC, d.id DESC LIMIT ?");
        values.push(SqlValue::Integer(filter.limit.max(1) as i64));

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(values), |row| {
            let kind_json: Option<String> = row.get(10)?;
            let symbols_json: Option<String> = row.get(13)?;
            let event_kind = kind_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                .and_then(|value| {
                    value
                        .get("type")
                        .and_then(|type_value| type_value.as_str())
                        .map(str::to_string)
                });
            let event_symbols = symbols_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
                .unwrap_or_default();
            Ok(DeliveryLogRecord {
                id: row.get(0)?,
                event_id: row.get(1)?,
                actor: row.get(2)?,
                channel: row.get(3)?,
                severity: row.get(4)?,
                sent_at_ts: row.get(5)?,
                status: row.get(6)?,
                body: row.get(7)?,
                event_title: row.get(8)?,
                event_summary: row.get(9)?,
                event_kind,
                event_source: row.get(11)?,
                event_url: row.get(12)?,
                event_symbols,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(anyhow::Error::from)
    }
}

/// 按 `source` 分组的事件入库数——用于 daily report 展示"各 poller 产出多少"。
/// 返回排序过的 `(source, count)` 列表,count 降序。
pub fn event_breakdown_by_source(
    store: &EventStore,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> anyhow::Result<Vec<(String, i64)>> {
    let conn = store.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT source, COUNT(*) FROM events
        WHERE created_at_ts >= ?1 AND created_at_ts < ?2
        GROUP BY source ORDER BY 2 DESC
        "#,
    )?;
    let rows = stmt.query_map(params![since.timestamp(), until.timestamp()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(anyhow::Error::from)
}

/// 按 `actor` + `status` 分组的推送统计——用于 daily report 按用户切片。
/// status 值通常是 `sent` / `queued` / `failed` / `filtered` 等。
pub fn delivery_breakdown_per_actor(
    store: &EventStore,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> anyhow::Result<Vec<(String, String, i64)>> {
    let conn = store.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        r#"
        SELECT actor, status, COUNT(*) FROM delivery_log
        WHERE sent_at_ts >= ?1 AND sent_at_ts < ?2
        GROUP BY actor, status ORDER BY actor, status
        "#,
    )?;
    let rows = stmt.query_map(params![since.timestamp(), until.timestamp()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(anyhow::Error::from)
}

fn severity_tag(s: &crate::event::Severity) -> &'static str {
    match s {
        crate::event::Severity::Low => "low",
        crate::event::Severity::Medium => "medium",
        crate::event::Severity::High => "high",
    }
}

fn category_kind_tags(category: &str) -> Option<&'static [&'static str]> {
    match category {
        "price" => Some(&["price_alert", "weekly52_high", "weekly52_low"]),
        "news" => Some(&["news_critical", "social_post"]),
        "filing" => Some(&["sec_filing"]),
        "earnings" => Some(&[
            "earnings_upcoming",
            "earnings_released",
            "earnings_call_transcript",
        ]),
        "macro" => Some(&["macro_event"]),
        "corp_action" => Some(&["dividend", "split"]),
        "analyst" => Some(&["analyst_grade"]),
        _ => None,
    }
}

fn parse_bps_from_band_id(id: &str) -> Option<i64> {
    if !id.starts_with("price_band:") {
        return None;
    }
    id.rsplit(':').next().and_then(|s| s.parse::<i64>().ok())
}

fn price_band_id_pattern(symbol: &str, direction: &str) -> Option<String> {
    let direction = match direction {
        "up" | "down" => direction,
        _ => return None,
    };
    Some(format!(
        "price_band:{}:%:{}:%",
        symbol.to_uppercase(),
        direction
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use tempfile::tempdir;

    fn sample_event(id: &str) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "Apple earnings".into(),
            summary: String::new(),
            url: None,
            source: "fmp.earning_calendar".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn insert_is_idempotent_per_id() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let ev = sample_event("earnings:AAPL:2026-04-30");
        assert!(store.insert_event(&ev).unwrap()); // 首次
        assert!(!store.insert_event(&ev).unwrap()); // 重复
        assert_eq!(store.count_events().unwrap(), 1);
    }

    #[test]
    fn distinct_ids_are_all_stored() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        assert!(store.insert_event(&sample_event("a")).unwrap());
        assert!(store.insert_event(&sample_event("b")).unwrap());
        assert!(store.insert_event(&sample_event("c")).unwrap());
        assert_eq!(store.count_events().unwrap(), 3);
    }

    #[test]
    fn baseline_is_set_on_first_open_and_preserved() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("events.db");
        let baseline_a = {
            let store = EventStore::open(&path).unwrap();
            store.baseline_at().unwrap()
        };
        // 重新打开不应重写 baseline
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let store = EventStore::open(&path).unwrap();
        let baseline_b = store.baseline_at().unwrap();
        assert_eq!(baseline_a, baseline_b);
    }

    #[test]
    fn delivery_log_is_append_only_across_retries() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        store
            .log_delivery(
                "ev1",
                "imessage:u1",
                "imessage",
                Severity::High,
                "failed",
                Some("body v1"),
            )
            .unwrap();
        // 同一 (event, actor) 二次写入应保留两行，而非覆盖
        store
            .log_delivery(
                "ev1",
                "imessage:u1",
                "imessage",
                Severity::High,
                "sent",
                Some("body v2"),
            )
            .unwrap();
        let conn = store.conn.lock().unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM delivery_log WHERE event_id='ev1' AND actor='imessage:u1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2, "delivery_log 应 append-only 保留每次尝试");
        let last_status: String = conn
            .query_row(
                "SELECT status FROM delivery_log WHERE event_id='ev1' ORDER BY sent_at_ts DESC, id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(last_status, "sent");
    }

    #[test]
    fn list_recent_delivery_logs_keeps_operator_level_rows() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        store
            .log_delivery(
                "ev-no-actor",
                "event_engine::::no_actor",
                "router",
                Severity::Low,
                "no_actor",
                None,
            )
            .unwrap();
        store
            .log_delivery(
                "ev-item",
                "discord::::u1",
                "digest_item",
                Severity::Medium,
                "omitted",
                None,
            )
            .unwrap();
        store
            .log_delivery(
                "ev-sink",
                "discord::::u1",
                "sink",
                Severity::High,
                "sent",
                Some("body"),
            )
            .unwrap();

        let rows = store
            .list_recent_delivery_logs(&DeliveryLogFilter {
                top_level_only: true,
                limit: 20,
                ..DeliveryLogFilter::default()
            })
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_id, "ev-sink");
        assert_eq!(rows[0].channel, "sink");
    }

    #[test]
    fn list_recent_delivery_logs_exposes_event_kind_type() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let mut event = sample_event("ev-kind");
        event.kind = EventKind::SecFiling {
            form: "8-K".to_string(),
        };
        store.insert_event(&event).unwrap();
        store
            .log_delivery(
                "ev-kind",
                "discord::::u1",
                "sink",
                Severity::High,
                "sent",
                Some("body"),
            )
            .unwrap();

        let rows = store
            .list_recent_delivery_logs(&DeliveryLogFilter {
                actor: Some("discord::::u1".to_string()),
                top_level_only: true,
                limit: 20,
                ..DeliveryLogFilter::default()
            })
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_kind.as_deref(), Some("sec_filing"));
    }

    #[test]
    fn jsonl_mirror_appends_once_per_new_event() {
        let dir = tempdir().unwrap();
        let mirror = dir.path().join("events.jsonl");
        let store = EventStore::open(dir.path().join("events.db"))
            .unwrap()
            .with_jsonl_path(&mirror);
        let ev = sample_event("e-jsonl");
        assert!(store.insert_event(&ev).unwrap());
        // 重复入库走 IGNORE，不再 append 镜像
        assert!(!store.insert_event(&ev).unwrap());
        let lines = std::fs::read_to_string(&mirror).unwrap();
        assert_eq!(lines.lines().count(), 1);
        assert!(lines.contains("e-jsonl"));
    }

    #[test]
    fn count_high_sent_since_only_counts_high_sink_sent() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let actor = "tg::::u1";
        // 真正算数的:高优 + sink + sent —— 4 条
        for i in 0..4 {
            store
                .log_delivery(
                    &format!("e{i}"),
                    actor,
                    "sink",
                    Severity::High,
                    "sent",
                    None,
                )
                .unwrap();
        }
        // 不算数的对照组
        store
            .log_delivery("e-medium", actor, "sink", Severity::Medium, "sent", None)
            .unwrap();
        store
            .log_delivery("e-failed", actor, "sink", Severity::High, "failed", None)
            .unwrap();
        store
            .log_delivery("e-digest", actor, "digest", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery(
                "e-filtered",
                actor,
                "prefs",
                Severity::High,
                "filtered",
                None,
            )
            .unwrap();
        store
            .log_delivery("e-other", "tg::::u2", "sink", Severity::High, "sent", None)
            .unwrap();

        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(store.count_high_sent_since(actor, since).unwrap(), 4);

        // 未来时间点:当然 0
        let future = Utc::now() + chrono::Duration::days(1);
        assert_eq!(store.count_high_sent_since(actor, future).unwrap(), 0);
    }

    #[test]
    fn high_counts_are_bucketed_by_event_category() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let actor = "tg::::u1";
        let mut price = sample_event("price-aapl");
        price.kind = EventKind::PriceAlert {
            pct_change_bps: 700,
            window: "day".into(),
        };
        let mut filing = sample_event("sec-aapl");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        store.insert_event(&price).unwrap();
        store.insert_event(&filing).unwrap();
        store
            .log_delivery(&price.id, actor, "sink", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery(&filing.id, actor, "sink", Severity::High, "sent", None)
            .unwrap();

        let since = Utc::now() - chrono::Duration::minutes(1);
        assert_eq!(
            store
                .count_high_sent_since_for_category(actor, since, "price")
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .count_high_sent_since_for_category(actor, since, "filing")
                .unwrap(),
            1
        );
        assert_eq!(store.count_high_sent_since(actor, since).unwrap(), 2);
    }

    #[test]
    fn last_high_sink_send_for_symbol_matches_case_insensitive_and_ignores_other_rows() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let actor = "tg::::u1";

        // 给 AAPL 和 NVDA 分别入库一条事件
        let mut aapl = sample_event("ev-aapl");
        aapl.symbols = vec!["AAPL".into()];
        let mut nvda = sample_event("ev-nvda");
        nvda.symbols = vec!["NVDA".into()];
        store.insert_event(&aapl).unwrap();
        store.insert_event(&nvda).unwrap();

        // 初始状态:无记录
        assert!(
            store
                .last_high_sink_send_for_symbol(actor, "AAPL")
                .unwrap()
                .is_none()
        );

        // High + sink + sent AAPL —— 应命中
        store
            .log_delivery("ev-aapl", actor, "sink", Severity::High, "sent", None)
            .unwrap();
        // Medium 不算,failed 不算,digest 渠道不算
        let mut medium_ev = sample_event("ev-medium");
        medium_ev.symbols = vec!["AAPL".into()];
        store.insert_event(&medium_ev).unwrap();
        store
            .log_delivery("ev-medium", actor, "sink", Severity::Medium, "sent", None)
            .unwrap();
        let mut failed_ev = sample_event("ev-failed");
        failed_ev.symbols = vec!["AAPL".into()];
        store.insert_event(&failed_ev).unwrap();
        store
            .log_delivery("ev-failed", actor, "sink", Severity::High, "failed", None)
            .unwrap();
        // 另一个 actor 的 sent 不算
        store
            .log_delivery("ev-aapl", "tg::::u2", "sink", Severity::High, "sent", None)
            .unwrap();
        // NVDA 的不应串到 AAPL
        store
            .log_delivery("ev-nvda", actor, "sink", Severity::High, "sent", None)
            .unwrap();

        let t_aapl = store.last_high_sink_send_for_symbol(actor, "aapl").unwrap();
        assert!(t_aapl.is_some(), "AAPL(小写查询)应命中");
        // 不存在的 symbol
        assert!(
            store
                .last_high_sink_send_for_symbol(actor, "TSLA")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn last_high_sink_send_with_firm_filter_distinguishes_grading_company() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let actor = "tg::::u1";

        let mk = |id: &str, firm: &str| MarketEvent {
            id: id.into(),
            kind: EventKind::AnalystGrade,
            severity: Severity::High,
            symbols: vec!["SNDK".into()],
            occurred_at: Utc::now(),
            title: "grade".into(),
            summary: String::new(),
            url: None,
            source: "fmp.grade".into(),
            payload: serde_json::json!({"gradingCompany": firm}),
        };
        let goldman = mk("g1", "Goldman Sachs");
        let raymond = mk("r1", "Raymond James");
        store.insert_event(&goldman).unwrap();
        store.insert_event(&raymond).unwrap();
        store
            .log_delivery("g1", actor, "sink", Severity::High, "sent", None)
            .unwrap();

        // 不带 firm 过滤 → 命中 Goldman 的 sent
        assert!(
            store
                .last_high_sink_send_for_symbol_category(actor, "SNDK", "analyst", None)
                .unwrap()
                .is_some()
        );
        // 带 firm = Goldman → 命中
        assert!(
            store
                .last_high_sink_send_for_symbol_category(
                    actor,
                    "SNDK",
                    "analyst",
                    Some("Goldman Sachs"),
                )
                .unwrap()
                .is_some()
        );
        // 带 firm = Raymond James → 没记录,应返回 None
        assert!(
            store
                .last_high_sink_send_for_symbol_category(
                    actor,
                    "SNDK",
                    "analyst",
                    Some("Raymond James"),
                )
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn event_breakdown_counts_by_source() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let mut a = sample_event("a");
        a.source = "fmp.stock_news".into();
        let mut b = sample_event("b");
        b.source = "fmp.stock_news".into();
        let mut c = sample_event("c");
        c.source = "fmp.earning_calendar".into();
        store.insert_event(&a).unwrap();
        store.insert_event(&b).unwrap();
        store.insert_event(&c).unwrap();
        let since = Utc::now() - chrono::Duration::minutes(1);
        let until = Utc::now() + chrono::Duration::minutes(1);
        let breakdown = event_breakdown_by_source(&store, since, until).unwrap();
        // news=2 排在 earnings=1 前面
        assert_eq!(breakdown[0], ("fmp.stock_news".into(), 2));
        assert_eq!(breakdown[1], ("fmp.earning_calendar".into(), 1));
    }

    #[test]
    fn delivery_breakdown_groups_per_actor_and_status() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        store
            .log_delivery("e1", "u1", "tg", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery("e2", "u1", "tg", Severity::Medium, "queued", None)
            .unwrap();
        store
            .log_delivery("e3", "u1", "tg", Severity::High, "sent", None)
            .unwrap();
        store
            .log_delivery("e4", "u2", "tg", Severity::High, "failed", None)
            .unwrap();
        let since = Utc::now() - chrono::Duration::minutes(1);
        let until = Utc::now() + chrono::Duration::minutes(1);
        let breakdown = delivery_breakdown_per_actor(&store, since, until).unwrap();
        assert!(breakdown.contains(&("u1".into(), "sent".into(), 2)));
        assert!(breakdown.contains(&("u1".into(), "queued".into(), 1)));
        assert!(breakdown.contains(&("u2".into(), "failed".into(), 1)));
    }

    #[test]
    fn today_signal_kinds_returns_same_day_symbol_hits() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();

        // 今日 AAPL 价格异动
        let mut price = sample_event("price:AAPL:today");
        price.kind = EventKind::PriceAlert {
            pct_change_bps: 650,
            window: "day".into(),
        };
        price.occurred_at = Utc::now();
        store.insert_event(&price).unwrap();

        // 今日 AAPL 8-K
        let mut filing = sample_event("sec:AAPL:today");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        filing.occurred_at = Utc::now();
        store.insert_event(&filing).unwrap();

        // 其他 ticker（不应命中）
        let mut other = sample_event("price:NVDA:today");
        other.kind = EventKind::PriceAlert {
            pct_change_bps: 300,
            window: "day".into(),
        };
        other.symbols = vec!["NVDA".into()];
        other.occurred_at = Utc::now();
        store.insert_event(&other).unwrap();

        // 昨日 AAPL（不应命中）
        let mut stale = sample_event("earnings:AAPL:yesterday");
        stale.kind = EventKind::EarningsReleased;
        stale.occurred_at = Utc::now() - chrono::Duration::days(2);
        store.insert_event(&stale).unwrap();

        let since = Utc::now() - chrono::Duration::hours(12);
        let mut tags = store.today_signal_kinds("AAPL", since).unwrap();
        tags.sort();
        assert_eq!(tags, vec!["price_alert", "sec_filing"]);
    }

    #[test]
    fn list_upcoming_earnings_returns_in_window_only() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();

        // 未来 5 天后的 AAPL earnings —— 应命中(within_days=14)
        let mut future = sample_event("earnings:AAPL:2026-04-26");
        future.kind = EventKind::EarningsUpcoming;
        future.symbols = vec!["AAPL".into()];
        future.occurred_at = Utc::now() + chrono::Duration::days(5);
        store.insert_event(&future).unwrap();

        // 未来 30 天后的 NVDA —— 超出 14 天窗口,应不命中
        let mut far_future = sample_event("earnings:NVDA:2026-05-21");
        far_future.kind = EventKind::EarningsUpcoming;
        far_future.symbols = vec!["NVDA".into()];
        far_future.occurred_at = Utc::now() + chrono::Duration::days(30);
        store.insert_event(&far_future).unwrap();

        // 昨天的 TSLA earnings —— 过去,不命中
        let mut past = sample_event("earnings:TSLA:2026-04-20");
        past.kind = EventKind::EarningsUpcoming;
        past.symbols = vec!["TSLA".into()];
        past.occurred_at = Utc::now() - chrono::Duration::days(1);
        store.insert_event(&past).unwrap();

        // 未来 2 天的 AAPL 8-K —— 不是 earnings_upcoming,不命中
        let mut filing = sample_event("sec:AAPL:future");
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        filing.symbols = vec!["AAPL".into()];
        filing.occurred_at = Utc::now() + chrono::Duration::days(2);
        store.insert_event(&filing).unwrap();

        let upcoming = store.list_upcoming_earnings(Utc::now(), 14).unwrap();
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].id, "earnings:AAPL:2026-04-26");
        assert!(matches!(upcoming[0].kind, EventKind::EarningsUpcoming));
    }

    #[test]
    fn purge_events_removes_older_rows() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        assert!(store.insert_event(&sample_event("old")).unwrap());
        // 人工把这条改到 40 天前
        {
            let conn = store.conn.lock().unwrap();
            let cutoff = Utc::now().timestamp() - 40 * 86_400;
            conn.execute(
                "UPDATE events SET created_at_ts = ?1 WHERE id = 'old'",
                params![cutoff],
            )
            .unwrap();
        }
        assert!(store.insert_event(&sample_event("new")).unwrap());
        let removed = store.purge_events_older_than(30).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.count_events().unwrap(), 1);
    }

    /// `delivered_event_ids_since` 是 digest synth 跨 flush 去重的底座 ——
    /// 必须只收 status=sent/dryrun、必须按 actor 隔离、必须**不需要 events 表
    /// 行**(synth 事件不会写 events 表,只在 delivery_log 留痕)。
    #[test]
    fn delivered_event_ids_since_filters_by_actor_status_and_time() {
        let dir = tempdir().unwrap();
        let store = EventStore::open(dir.path().join("events.db")).unwrap();
        let actor = "tg::::u1";
        let other = "tg::::u2";
        let earlier = chrono::Utc::now() - chrono::Duration::hours(1);

        // synth 事件本身不写 events 表,直接 log_delivery 也应能查出来
        store
            .log_delivery(
                "synth:earnings:GOOGL:2026-04-29:countdown:2026-04-26",
                actor,
                "digest_item",
                Severity::Medium,
                "sent",
                None,
            )
            .unwrap();
        // queued 不算已投递
        store
            .log_delivery(
                "synth:earnings:BE:2026-04-28:countdown:2026-04-26",
                actor,
                "digest_item",
                Severity::Medium,
                "queued",
                None,
            )
            .unwrap();
        // 其他 actor 不应混入本 actor 的结果
        store
            .log_delivery(
                "ev-other",
                other,
                "digest_item",
                Severity::Medium,
                "sent",
                None,
            )
            .unwrap();

        let ids = store.delivered_event_ids_since(actor, earlier).unwrap();
        assert!(ids.contains("synth:earnings:GOOGL:2026-04-29:countdown:2026-04-26"));
        assert!(!ids.contains("synth:earnings:BE:2026-04-28:countdown:2026-04-26"));
        assert!(!ids.contains("ev-other"));
    }
}
