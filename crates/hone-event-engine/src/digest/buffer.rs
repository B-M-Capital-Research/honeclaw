//! `DigestBuffer` —— 按 actor 缓存 Medium/Low 事件的 append-only 文件槽位。
//!
//! 存储布局:`{buffer_dir}/{actor_slug}.jsonl`,一条事件一行;`drain_actor`
//! 把文件原子改名成 `.flushed-{ts}` 再读,避免 reader/writer race。
//!
//! Price alert 单独走 `enqueue_latest`:同一天同 symbol 同 window 只保留最新
//! 那条,不会在 digest 里出现同一标的多条价格提醒。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

use crate::event::{EventKind, MarketEvent};

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
        if let Some(key) = price_digest_key(event) {
            return self.enqueue_latest(actor, &line, &key);
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.file_for(actor))?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    fn enqueue_latest(&self, actor: &ActorIdentity, line: &str, key: &str) -> anyhow::Result<()> {
        let path = self.file_for(actor);
        let mut kept = Vec::new();
        if path.exists() {
            let f = std::fs::File::open(&path)?;
            for line in BufReader::new(f).lines().map_while(Result::ok) {
                if line.trim().is_empty() {
                    continue;
                }
                let should_replace = serde_json::from_str::<BufferRecord>(&line)
                    .ok()
                    .and_then(|rec| price_digest_key(&rec.event))
                    .as_deref()
                    == Some(key);
                if !should_replace {
                    kept.push(line);
                }
            }
        }
        kept.push(line.to_string());

        let tmp = path.with_extension(format!(
            "jsonl.tmp-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&tmp)?;
            for item in kept {
                writeln!(f, "{item}")?;
            }
        }
        std::fs::rename(tmp, path)?;
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

/// 同 symbol 同日的所有 PriceAlert(无论 window 是 day band 还是 close)共用
/// 一个 key,buffer 只留最后一条。原来 key 带 `window` 时,band(intraday)+
/// close(end-of-day)会各占一条,digest 里就重复出现 `AMD 跨过 +12% 档` 和
/// `AMD +13.91%` 两条几乎一样的价格行。"最新写入胜出"对用户够用——盘后
/// close 总是最后到,代表当天的总结性涨跌幅。
fn price_digest_key(event: &MarketEvent) -> Option<String> {
    let EventKind::PriceAlert { .. } = &event.kind else {
        return None;
    };
    let symbol = event.symbols.first()?.to_uppercase();
    let date = event
        .payload
        .get("hone_price_trade_date")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            event
                .occurred_at
                .date_naive()
                .format("%Y-%m-%d")
                .to_string()
        });
    Some(format!("{symbol}:{date}"))
}
