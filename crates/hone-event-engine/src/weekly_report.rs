//! WeeklyReport — 每周六本地 10:00 给每个单聊 actor 推一份周度复盘
//! (2026-08,推送体检 item 7)。
//!
//! 与 [`crate::daily_report::DailyReport`](运营日志,不推用户)不同,周报是
//! **用户视角**的:美股周五收盘后、北京周六上午,把这一周的持仓盈亏归因、
//! 评级净修正、推送质量与下周财报日历合成一条消息。
//!
//! 组成(数据不可得的 section 自动省略,绝不编数字):
//! 1. 持仓周度盈亏 + 归因 —— FMP 日线(周五收盘 vs 上周五收盘)× registry
//!    持仓快照;
//! 2. 评级净修正 —— store 近 7 日 analyst_grade 按标的聚合(复用 item 3 的
//!    共识计数,汇总文只计入其 counts);
//! 3. 推送质量 —— delivery_log 状态分布 + 汇总文拦截数 + band 阶梯合流数,
//!    即离线回放 `replay_push_quality_audit` 的同口径线上版;
//! 4. 定时任务健康 —— `task_runs.jsonl` 里 `cron.*` 的近 7 日成败率;
//! 5. 下周财报 —— store 里未来 7 天的 earnings_upcoming。
//!
//! tick contract 与 DailyReport 相同:上层每 60s `tick_once(now, &mut fired)`。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, Utc, Weekday};

use crate::digest::{in_window_timezone, local_date_key_timezone};
use crate::fmp::FmpClient;
use crate::pollers::analyst_grade::consensus_counts_from_payloads;
use crate::router::OutboundSink;
use crate::store::EventStore;
use crate::subscription::{PositionSnapshot, SharedRegistry};

pub struct WeeklyReport {
    store: Arc<EventStore>,
    registry: Arc<SharedRegistry>,
    sink: Arc<dyn OutboundSink>,
    client: Option<FmpClient>,
    report_dir: PathBuf,
    /// `task_runs.jsonl` 所在目录(即 `data/runtime/`)。为 `None` 时省略
    /// "定时任务健康"section。
    task_runs_dir: Option<PathBuf>,
    timezone: hone_core::RuntimeTimezone,
    trigger_weekday: Weekday,
    trigger_time: String,
}

impl WeeklyReport {
    pub fn new(
        store: Arc<EventStore>,
        registry: Arc<SharedRegistry>,
        sink: Arc<dyn OutboundSink>,
        report_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            store,
            registry,
            sink,
            client: None,
            report_dir: report_dir.into(),
            task_runs_dir: None,
            timezone: hone_core::runtime_timezone(),
            trigger_weekday: Weekday::Sat,
            trigger_time: "10:00".into(),
        }
    }

    pub fn with_fmp_client(mut self, client: FmpClient) -> Self {
        self.client = Some(client);
        self
    }

    pub fn with_task_runs_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.task_runs_dir = Some(dir.into());
        self
    }

    #[cfg(test)]
    pub fn with_tz_offset_hours(mut self, offset: i32) -> Self {
        self.timezone = hone_core::RuntimeTimezone::fixed_offset_seconds(offset * 3600);
        self
    }

    pub fn with_runtime_timezone(mut self, timezone: hone_core::RuntimeTimezone) -> Self {
        self.timezone = timezone;
        self
    }

    /// 单轮 tick:本地周 `trigger_weekday` 的 `trigger_time` 命中即发。
    /// 返回成功 sink 送达的 actor 数。
    pub async fn tick_once(
        &self,
        now: DateTime<Utc>,
        already_fired: &mut std::collections::HashSet<String>,
    ) -> anyhow::Result<u32> {
        let local_weekday = self.timezone.at_utc(now).weekday();
        if local_weekday != self.trigger_weekday
            || !in_window_timezone(now, &self.trigger_time, &self.timezone)
        {
            return Ok(0);
        }
        let date = local_date_key_timezone(now, &self.timezone);
        if !already_fired.insert(format!("weekly-report@{date}")) {
            return Ok(0);
        }

        let registry = self.registry.load();
        let mut sent = 0u32;
        let mut file_sections: Vec<String> = Vec::new();
        for actor in registry.actors() {
            let positions = registry.positions_for(&actor).cloned().unwrap_or_default();
            if positions.is_empty() {
                continue;
            }
            let week_rows = self.fetch_week_rows(&positions).await;
            let body = self.render_for_actor(now, &positions, &week_rows);
            if body.trim().is_empty() {
                continue;
            }
            let message = format!("📆 周度复盘 · {date}\n\n{body}");
            match self.sink.send(&actor, &message).await {
                Ok(()) => sent += 1,
                Err(error) => {
                    tracing::warn!(user = %actor.user_id, "weekly report send failed: {error:#}")
                }
            }
            file_sections.push(format!("## {}:{}\n\n{body}", actor.channel, actor.user_id));
        }

        if !file_sections.is_empty() {
            let _ = write_report(&self.report_dir, &date, &file_sections.join("\n\n"));
        }
        tracing::info!(date = %date, sent, "weekly report fanout");
        Ok(sent)
    }

    /// 每持仓标的拉近 10 个交易日收盘(FMP 不可用 → 空 map,P&L 段省略)。
    async fn fetch_week_rows(
        &self,
        positions: &HashMap<String, PositionSnapshot>,
    ) -> HashMap<String, Vec<f64>> {
        let Some(client) = &self.client else {
            return HashMap::new();
        };
        let mut out = HashMap::new();
        let mut symbols: Vec<&String> = positions.keys().collect();
        symbols.sort();
        for symbol in symbols {
            let path = format!("/v3/historical-price-full/{symbol}?timeseries=10&serietype=line");
            match client.get_json(&path).await {
                Ok(raw) => {
                    // FMP 返回日期降序:index 0 = 最近一个交易日(周六跑 = 周五收盘)。
                    let closes: Vec<f64> = raw
                        .get("historical")
                        .and_then(|v| v.as_array())
                        .map(|rows| {
                            rows.iter()
                                .filter_map(|r| r.get("close").and_then(|c| c.as_f64()))
                                .collect()
                        })
                        .unwrap_or_default();
                    if closes.len() >= 6 {
                        out.insert(symbol.clone(), closes);
                    }
                }
                Err(error) => {
                    tracing::warn!(%symbol, "weekly report 日线拉取失败: {error:#}")
                }
            }
        }
        out
    }

    fn render_for_actor(
        &self,
        now: DateTime<Utc>,
        positions: &HashMap<String, PositionSnapshot>,
        week_rows: &HashMap<String, Vec<f64>>,
    ) -> String {
        let mut sections: Vec<String> = Vec::new();
        if let Some(pnl) = render_week_pnl(positions, week_rows) {
            sections.push(pnl);
        }
        if let Some(revisions) = self.render_analyst_revisions(now, positions) {
            sections.push(revisions);
        }
        if let Some(quality) = self.render_push_quality(now) {
            sections.push(quality);
        }
        if let Some(cron_health) = self.render_cron_health(now) {
            sections.push(cron_health);
        }
        if let Some(earnings) = self.render_upcoming_earnings(now, positions) {
            sections.push(earnings);
        }
        sections.join("\n\n")
    }

    /// 近 7 日各持仓标的的评级净修正(全零标的省略)。
    fn render_analyst_revisions(
        &self,
        now: DateTime<Utc>,
        positions: &HashMap<String, PositionSnapshot>,
    ) -> Option<String> {
        let since = now - Duration::days(7);
        let mut lines: Vec<String> = Vec::new();
        let mut symbols: Vec<&String> = positions.keys().collect();
        symbols.sort();
        for symbol in symbols {
            let payloads = self
                .store
                .list_analyst_grade_payloads_in_window(symbol, since, now)
                .unwrap_or_default();
            let counts = consensus_counts_from_payloads(&payloads);
            if counts.total() == 0 {
                continue;
            }
            let mut parts = Vec::new();
            if counts.down > 0 {
                parts.push(format!("{} 下调", counts.down));
            }
            if counts.up > 0 {
                parts.push(format!("{} 上调", counts.up));
            }
            if counts.init > 0 {
                parts.push(format!("{} 首评", counts.init));
            }
            if counts.reiter > 0 {
                parts.push(format!("{} 重申", counts.reiter));
            }
            lines.push(format!("· {symbol}:{}", parts.join(" / ")));
        }
        if lines.is_empty() {
            return None;
        }
        Some(format!("🗓 本周评级动向\n{}", lines.join("\n")))
    }

    /// 近 7 日推送质量:delivery 状态分布 + 两道防线的拦截量
    /// (`replay_push_quality_audit` 的同口径线上版)。
    fn render_push_quality(&self, now: DateTime<Utc>) -> Option<String> {
        let since = now - Duration::days(7);
        let deliveries =
            crate::store::delivery_breakdown_per_actor(&self.store, since, now).ok()?;
        let mut by_status: HashMap<String, i64> = HashMap::new();
        for (_, status, count) in &deliveries {
            *by_status.entry(status.clone()).or_default() += count;
        }
        let roundups_intercepted = self
            .store
            .count_event_ids_in_window("grade_roundup:", since, now)
            .unwrap_or(0);
        let band_events = self
            .store
            .count_event_ids_in_window("price_band:", since, now)
            .unwrap_or(0);
        let sent = by_status.get("sent").copied().unwrap_or(0);
        let queued = by_status.get("queued").copied().unwrap_or(0);
        let quiet_held = by_status.get("quiet_held").copied().unwrap_or(0);
        if sent + queued + quiet_held == 0 && roundups_intercepted == 0 && band_events == 0 {
            return None;
        }
        let mut line = format!("📮 本周推送:即时 {sent} · 进摘要 {queued}");
        if quiet_held > 0 {
            line.push_str(&format!(" · 勿扰暂留 {quiet_held}"));
        }
        if roundups_intercepted > 0 {
            line.push_str(&format!("\n🛡 汇总文拦截 {roundups_intercepted} 篇"));
        }
        if band_events > 0 {
            line.push_str(&format!(" · 盘中跨档事件 {band_events} 条(阶梯已合流)"));
        }
        Some(line)
    }

    /// 近 7 日定时任务健康:成败率按 `cron.{channel}.{kind}` 汇总。
    ///
    /// 数据来自 `task_runs.jsonl`——这正是让用户 cron 写这份账的目的:
    /// 2026-08-15 翻生产库才发现客户定时任务失败率长期 30%–50%、持续两周无人
    /// 发现,因为它此前完全不在任何主动出现在人眼前的视图里。
    fn render_cron_health(&self, now: DateTime<Utc>) -> Option<String> {
        let dir = self.task_runs_dir.as_deref()?;
        let since = now - Duration::days(7);
        let runs = hone_core::task_observer::read_recent_task_runs(dir, 7, 20_000);
        let mut ok = 0_u32;
        let mut skipped = 0_u32;
        let mut failed = 0_u32;
        let mut last_error: Option<String> = None;
        for run in runs
            .iter()
            .filter(|run| run.task.starts_with("cron.") && run.started_at >= since)
        {
            match run.outcome {
                hone_core::TaskOutcome::Ok => ok += 1,
                hone_core::TaskOutcome::Skipped => skipped += 1,
                hone_core::TaskOutcome::Failed => {
                    failed += 1;
                    if last_error.is_none() {
                        last_error = run.error.clone();
                    }
                }
            }
        }
        // 数据不可得就整段省略,绝不编数字。
        let attempted = ok + failed;
        if attempted == 0 && skipped == 0 {
            return None;
        }
        let mut line = format!("⏰ 本周定时任务:成功 {ok} · 失败 {failed}");
        if skipped > 0 {
            line.push_str(&format!(" · 无触发 {skipped}"));
        }
        if attempted > 0 {
            let rate = (failed as f64) * 100.0 / (attempted as f64);
            line.push_str(&format!("（失败率 {rate:.0}%）"));
        }
        if let Some(error) = last_error {
            let brief: String = error.chars().take(60).collect();
            line.push_str(&format!("\n   最近失败:{brief}"));
        }
        Some(line)
    }

    /// 未来 7 天持仓标的的财报日历。
    fn render_upcoming_earnings(
        &self,
        now: DateTime<Utc>,
        positions: &HashMap<String, PositionSnapshot>,
    ) -> Option<String> {
        let upcoming = self.store.list_upcoming_earnings(now, 7).ok()?;
        let mut lines: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for event in upcoming {
            let Some(symbol) = event.symbols.first() else {
                continue;
            };
            let symbol = symbol.to_ascii_uppercase();
            if !positions.contains_key(&symbol) || !seen.insert(symbol.clone()) {
                continue;
            }
            let days = (event.occurred_at.date_naive() - now.date_naive())
                .num_days()
                .max(0);
            lines.push(format!(
                "· {symbol}:{}（{days} 天后）",
                event.occurred_at.date_naive()
            ));
        }
        if lines.is_empty() {
            return None;
        }
        lines.sort();
        Some(format!("📅 下周财报\n{}", lines.join("\n")))
    }
}

/// 周度盈亏 + 归因。`week_rows` 为日期降序收盘价(≥6 个),[0] 是本周五、
/// [5] 是上周五。缺数据的标的跳过;全缺 → None。
fn render_week_pnl(
    positions: &HashMap<String, PositionSnapshot>,
    week_rows: &HashMap<String, Vec<f64>>,
) -> Option<String> {
    let mut rows: Vec<(String, f64, f64)> = Vec::new(); // (symbol, week_pct, week_usd)
    for (symbol, position) in positions {
        let Some(closes) = week_rows.get(symbol) else {
            continue;
        };
        let (now_close, prev_close) = (closes[0], closes[5]);
        if prev_close <= 0.0 {
            continue;
        }
        let week_pct = (now_close - prev_close) / prev_close * 100.0;
        let week_usd = position.shares * (now_close - prev_close);
        rows.push((symbol.clone(), week_pct, week_usd));
    }
    if rows.is_empty() {
        return None;
    }
    // 归因排序:美元影响绝对值大的在前
    rows.sort_by(|a, b| {
        b.2.abs()
            .partial_cmp(&a.2.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let total_usd: f64 = rows.iter().map(|(_, _, usd)| usd).sum();
    let total_sign = if total_usd >= 0.0 { "+" } else { "-" };
    let mut lines = vec![format!(
        "💼 本周持仓盈亏:{total_sign}${:.0}",
        total_usd.abs()
    )];
    for (symbol, pct, usd) in &rows {
        let sign = if *usd >= 0.0 { "+" } else { "-" };
        lines.push(format!("· {symbol}:{pct:+.1}%（{sign}${:.0}）", usd.abs()));
    }
    Some(lines.join("\n"))
}

fn write_report(dir: &Path, date: &str, body: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{date}.md"));
    std::fs::write(&path, format!("# 周度复盘 {date}\n\n{body}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(shares: f64) -> PositionSnapshot {
        PositionSnapshot {
            shares,
            avg_cost: 100.0,
            weight_pct: None,
        }
    }

    /// 周度盈亏:SNDK 13 股 380→400 (+$260,+5.3%),MU 19 股 961→900
    /// (−$1159,−6.3%);MU 美元影响大 → 归因排前;合计 −$899。
    #[test]
    fn week_pnl_attribution_sorts_by_dollar_impact() {
        let mut positions = HashMap::new();
        positions.insert("SNDK".to_string(), position(13.0));
        positions.insert("MU".to_string(), position(19.0));
        let mut rows = HashMap::new();
        rows.insert(
            "SNDK".to_string(),
            vec![400.0, 398.0, 395.0, 390.0, 385.0, 380.0],
        );
        rows.insert(
            "MU".to_string(),
            vec![900.0, 910.0, 930.0, 940.0, 950.0, 961.0],
        );
        let text = render_week_pnl(&positions, &rows).unwrap();
        assert!(text.contains("💼 本周持仓盈亏:-$899"), "{text}");
        let mu_pos = text.find("MU").unwrap();
        let sndk_pos = text.find("SNDK").unwrap();
        assert!(mu_pos < sndk_pos, "MU 美元影响更大应排前: {text}");
        assert!(text.contains("MU:-6.3%（-$1159）"), "{text}");
        assert!(text.contains("SNDK:+5.3%（+$260）"), "{text}");
    }

    #[test]
    fn week_pnl_skips_symbols_without_data_and_none_when_all_missing() {
        let mut positions = HashMap::new();
        positions.insert("SNDK".to_string(), position(13.0));
        assert!(render_week_pnl(&positions, &HashMap::new()).is_none());
    }

    /// 定时任务健康:从 `task_runs.jsonl` 聚合 cron.* 的成败,只统计 7 日窗口内。
    /// 没接 task_runs_dir 或窗口内无记录时整段省略,绝不编数字。
    #[test]
    fn cron_health_summarizes_task_runs_and_omits_when_unavailable() {
        use crate::router::LogSink;
        use crate::subscription::SubscriptionRegistry;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let make = |runs_dir: Option<PathBuf>| {
            let mut report = WeeklyReport::new(
                store.clone(),
                Arc::new(SharedRegistry::from_registry(SubscriptionRegistry::new())),
                Arc::new(LogSink),
                dir.path().join("weekly"),
            );
            if let Some(runs_dir) = runs_dir {
                report = report.with_task_runs_dir(runs_dir);
            }
            report
        };

        let now = Utc::now();
        // 没配 task_runs_dir → 省略
        assert!(make(None).render_cron_health(now).is_none());

        // 配了但目录为空 → 仍省略
        let runs_dir = dir.path().join("runtime");
        std::fs::create_dir_all(&runs_dir).unwrap();
        assert!(
            make(Some(runs_dir.clone()))
                .render_cron_health(now)
                .is_none()
        );

        let started = now - Duration::hours(2);
        for _ in 0..3 {
            hone_core::task_observer::record_ok(&runs_dir, "cron.web.heartbeat", started, 1);
        }
        hone_core::task_observer::record_failed(
            &runs_dir,
            "cron.web.heartbeat",
            started,
            "heartbeat 输出不是结构化 JSON，任务已标记失败",
        );
        hone_core::task_observer::record_skipped(&runs_dir, "cron.feishu.scheduled", started);
        // 非 cron 任务不得混入
        hone_core::task_observer::record_failed(&runs_dir, "poller.fmp.news", started, "boom");

        let text = make(Some(runs_dir)).render_cron_health(now).unwrap();
        assert!(text.contains("成功 3"), "{text}");
        assert!(text.contains("失败 1"), "{text}");
        assert!(text.contains("无触发 1"), "{text}");
        assert!(text.contains("失败率 25%"), "{text}");
        assert!(text.contains("结构化 JSON"), "{text}");
        assert!(!text.contains("boom"), "非 cron 任务不应计入: {text}");
    }

    /// 触发窗口:仅本地周六 trigger_time 命中,且同日只发一次。
    #[tokio::test]
    async fn fires_only_on_saturday_window_once() {
        use crate::router::LogSink;
        use crate::subscription::SubscriptionRegistry;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(EventStore::open(dir.path().join("e.db")).unwrap());
        let report = WeeklyReport::new(
            store,
            Arc::new(SharedRegistry::from_registry(SubscriptionRegistry::new())),
            Arc::new(LogSink),
            dir.path().join("weekly"),
        )
        .with_tz_offset_hours(8);

        let mut fired = std::collections::HashSet::new();
        // 2026-08-15 是周六;10:00 北京 = 02:00 UTC
        let saturday = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 8, 15, 2, 0, 30).unwrap();
        let friday = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 8, 14, 2, 0, 30).unwrap();
        // 周五不触发
        assert_eq!(report.tick_once(friday, &mut fired).await.unwrap(), 0);
        assert!(fired.is_empty());
        // 周六触发(无持仓 actor → sent=0,但 fire key 已占用)
        assert_eq!(report.tick_once(saturday, &mut fired).await.unwrap(), 0);
        assert_eq!(fired.len(), 1);
        // 同窗口重入不再触发
        assert_eq!(report.tick_once(saturday, &mut fired).await.unwrap(), 0);
        assert_eq!(fired.len(), 1);
    }
}
