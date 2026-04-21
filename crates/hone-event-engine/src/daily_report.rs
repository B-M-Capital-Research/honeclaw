//! DailyReport — 每日本地 22:00 触发,把当日事件/推送分布**落盘成日志**。
//!
//! 注意:日报只服务于引擎运营视角(我自己看),不作为 OutboundSink 推送给用户。
//! 普通用户只关心自己持仓对应的推送,对"fmp.stock_news 今天入库了 42 条"这种
//! 指标无感。产物:
//! - `data/daily_reports/YYYY-MM-DD.md` — 人类可读的 Markdown 快照
//! - 一行 `tracing::info` 紧凑版,方便 grep 线上日志
//!
//! 设计参照 `DigestScheduler`:上层每 60s `tick_once(now, &mut fired)`,
//! 命中窗口就落盘;`fired` 防止同分钟重触发,跨日由调用方清空。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, NaiveTime, TimeZone, Utc};

use crate::digest::{in_window, local_date_key};
use crate::store::{EventStore, delivery_breakdown_per_actor, event_breakdown_by_source};

pub struct DailyReport {
    store: Arc<EventStore>,
    report_dir: PathBuf,
    tz_offset_hours: i32,
    trigger_time: String,
}

impl DailyReport {
    pub fn new(store: Arc<EventStore>, report_dir: impl Into<PathBuf>) -> Self {
        Self {
            store,
            report_dir: report_dir.into(),
            tz_offset_hours: 8,
            trigger_time: "22:00".into(),
        }
    }

    pub fn with_tz_offset_hours(mut self, offset: i32) -> Self {
        self.tz_offset_hours = offset;
        self
    }

    pub fn with_trigger_time(mut self, hhmm: impl Into<String>) -> Self {
        self.trigger_time = hhmm.into();
        self
    }

    /// 单轮 tick:命中窗口则渲染+落盘。返回是否写入(0 = 未命中,1 = 已落盘)。
    pub async fn tick_once(
        &self,
        now: DateTime<Utc>,
        already_fired_today: &mut std::collections::HashSet<String>,
    ) -> anyhow::Result<u32> {
        if !in_window(now, &self.trigger_time, self.tz_offset_hours) {
            return Ok(0);
        }
        let date = local_date_key(now, self.tz_offset_hours);
        let fire_key = format!("daily-report@{date}@{}", self.trigger_time);
        if !already_fired_today.insert(fire_key) {
            return Ok(0);
        }

        let (since, until) = local_day_bounds(now, self.tz_offset_hours);
        let events_by_source = event_breakdown_by_source(&self.store, since, until)?;
        let deliveries = delivery_breakdown_per_actor(&self.store, since, until)?;

        let body = render_body(&date, &events_by_source, &deliveries);
        write_report(&self.report_dir, &date, &body)?;

        // 紧凑日志:便于 grep 线上 "daily_report"
        let total_events: i64 = events_by_source.iter().map(|(_, n)| n).sum();
        let total_deliveries: i64 = deliveries.iter().map(|(_, _, n)| n).sum();
        tracing::info!(
            date = %date,
            events_ingested = total_events,
            deliveries = total_deliveries,
            source_count = events_by_source.len(),
            actor_count = deliveries.iter().map(|(a, _, _)| a.as_str()).collect::<std::collections::HashSet<_>>().len(),
            "daily_report written"
        );
        Ok(1)
    }
}

fn write_report(report_dir: &Path, date: &str, body: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(report_dir)?;
    let path = report_dir.join(format!("{date}.md"));
    std::fs::write(&path, body)?;
    Ok(())
}

fn local_day_bounds(now: DateTime<Utc>, offset_hours: i32) -> (DateTime<Utc>, DateTime<Utc>) {
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let midnight = local
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let local_midnight = offset
        .from_local_datetime(&midnight)
        .single()
        .unwrap_or_else(|| offset.from_utc_datetime(&midnight));
    (local_midnight.with_timezone(&Utc), now)
}

/// 渲染日报正文(Markdown)。给引擎运营看,不推送给用户。
pub fn render_body(
    date: &str,
    events_by_source: &[(String, i64)],
    deliveries: &[(String, String, i64)],
) -> String {
    let total_events: i64 = events_by_source.iter().map(|(_, n)| n).sum();
    let mut out = format!("# Hone 日报 · {date}\n\n");

    out.push_str("## 事件入库\n\n");
    if total_events == 0 {
        out.push_str("_今日 0 条事件_ —— 各 poller 均无产出,检查 FMP 密钥/网络。\n");
    } else {
        out.push_str(&format!("合计 **{total_events}** 条\n\n"));
        out.push_str("| source | count |\n|---|--:|\n");
        for (src, n) in events_by_source {
            out.push_str(&format!("| `{src}` | {n} |\n"));
        }
    }

    out.push_str("\n## 推送分布\n\n");
    if deliveries.is_empty() {
        out.push_str("_今日 0 次推送_\n");
    } else {
        // 按 actor 聚合
        use std::collections::BTreeMap;
        let mut by_actor: BTreeMap<&str, Vec<(&str, i64)>> = BTreeMap::new();
        for (actor, status, n) in deliveries {
            by_actor
                .entry(actor.as_str())
                .or_default()
                .push((status.as_str(), *n));
        }
        out.push_str("| actor | sent | queued | filtered | failed |\n|---|--:|--:|--:|--:|\n");
        for (actor, rows) in by_actor {
            let get = |s: &str| {
                rows.iter()
                    .find(|(st, _)| *st == s)
                    .map(|(_, n)| *n)
                    .unwrap_or(0)
            };
            out.push_str(&format!(
                "| `{actor}` | {} | {} | {} | {} |\n",
                get("sent"),
                get("queued"),
                get("filtered"),
                get("failed"),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_no_data_reports_zero_state() {
        let body = render_body("2026-04-21", &[], &[]);
        assert!(body.contains("# Hone 日报 · 2026-04-21"));
        assert!(body.contains("_今日 0 条事件_"));
        assert!(body.contains("_今日 0 次推送_"));
    }

    #[test]
    fn body_renders_sources_and_per_actor_delivery_tables() {
        let sources = vec![
            ("fmp.stock_news".into(), 42_i64),
            ("fmp.earning_calendar".into(), 5_i64),
        ];
        let delivs = vec![
            ("tg::::u1".into(), "sent".into(), 3_i64),
            ("tg::::u1".into(), "queued".into(), 28_i64),
            ("tg::::u2".into(), "filtered".into(), 2_i64),
        ];
        let body = render_body("2026-04-21", &sources, &delivs);
        assert!(body.contains("合计 **47** 条"));
        assert!(body.contains("| `fmp.stock_news` | 42 |"));
        assert!(body.contains("| `tg::::u1` | 3 | 28 | 0 | 0 |"));
        assert!(body.contains("| `tg::::u2` | 0 | 0 | 2 | 0 |"));
    }

    #[tokio::test]
    async fn tick_once_writes_file_inside_window_and_no_fire_outside() {
        use tempfile::tempdir;

        let tmp = tempdir().unwrap();
        let store = Arc::new(EventStore::open(tmp.path().join("events.db")).unwrap());
        let report_dir = tmp.path().join("reports");

        let report = DailyReport::new(store, &report_dir)
            .with_trigger_time("22:00")
            .with_tz_offset_hours(8);

        let mut fired = std::collections::HashSet::new();

        // 窗口外:不落盘
        let off_window = chrono::DateTime::parse_from_rfc3339("2026-04-21T00:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(report.tick_once(off_window, &mut fired).await.unwrap(), 0);
        assert!(!report_dir.join("2026-04-21.md").exists());
        assert!(fired.is_empty());

        // 22:00 UTC+8 = 14:00 UTC
        let in_window = chrono::DateTime::parse_from_rfc3339("2026-04-21T14:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let n1 = report.tick_once(in_window, &mut fired).await.unwrap();
        assert_eq!(n1, 1);
        assert!(report_dir.join("2026-04-21.md").exists());

        // 同分钟 re-tick:不重写
        let n2 = report.tick_once(in_window, &mut fired).await.unwrap();
        assert_eq!(n2, 0);
    }
}
