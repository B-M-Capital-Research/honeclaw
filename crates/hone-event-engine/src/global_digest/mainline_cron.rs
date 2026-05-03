//! 后台 cron:每天 sweep 一次所有持仓 actor,按"覆盖度"触发蒸馏。
//!
//! 触发政策(2026-04-26 用户要求):
//! - 每天 tick 一次(默认 24h interval)
//! - 对每个 portfolio actor:
//!   - 若 holdings 全部已有投资主线 → 仅当距上次蒸馏 ≥ WEEKLY_REFRESH_HOURS 才再跑
//!   - 若 holdings 有 ticker 缺主线(没画像 / 上次失败 / 新增持仓) → **立即跑**
//!     (但仍受 MIN_RETRY_INTERVAL_HOURS 节流,避免无画像时无限循环)
//!
//! 这样的好处:
//! - 新加持仓 / 新建画像后,最长 24h 就能进投资主线池
//! - 已经覆盖完整的不会被频繁打扰(每周 1 次刷新)
//! - 一直没画像的 ticker 不会每小时都重试(MIN_RETRY 节流)
//!
//! 失败哲学:cron 出错绝不能阻塞 digest scheduler。独立 task,挂掉只 warn。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use hone_core::ActorIdentity;
use hone_memory::PortfolioStorage;
use tracing::{info, warn};

use crate::global_digest::mainline_distill::{MainlineDistiller, distill_and_persist_one};
use crate::prefs::{NotificationPrefs, PrefsProvider};

/// 默认 cron tick 间隔:24 小时(每日检测一次)。
pub const DEFAULT_DISTILL_INTERVAL_HOURS: i64 = 24;

/// 覆盖完整后,触发"周更新"的最小间隔。
pub const WEEKLY_REFRESH_HOURS: i64 = 24 * 7;

/// 即使 holdings 有 missing,也要至少等这么久再重试(防止无画像 ticker 死循环)。
pub const MIN_RETRY_INTERVAL_HOURS: i64 = 6;

/// 触发原因。便于日志诊断。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerReason {
    FirstRun,
    MissingHoldings,
    WeeklyRefresh,
}

impl TriggerReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerReason::FirstRun => "first_run",
            TriggerReason::MissingHoldings => "missing_holdings",
            TriggerReason::WeeklyRefresh => "weekly_refresh",
        }
    }
}

/// 根据 prefs + holdings + now 决定是否要跑一次蒸馏。返回 None 表示跳过。
///
/// 纯函数,易测。`distill_tick` 调它做决策。
pub fn should_trigger(
    prefs: &NotificationPrefs,
    holdings: &[String],
    now: DateTime<Utc>,
) -> Option<TriggerReason> {
    let last = prefs
        .last_mainline_distilled_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let covered: HashSet<String> = prefs
        .mainline_by_ticker
        .as_ref()
        .map(|m| m.keys().map(|s| s.to_uppercase()).collect())
        .unwrap_or_default();
    let has_missing = holdings
        .iter()
        .any(|h| !covered.contains(&h.to_uppercase()));

    match last {
        None => Some(TriggerReason::FirstRun),
        Some(t) => {
            let hours_since = (now - t).num_hours();
            if has_missing && hours_since >= MIN_RETRY_INTERVAL_HOURS {
                Some(TriggerReason::MissingHoldings)
            } else if !has_missing && hours_since >= WEEKLY_REFRESH_HOURS {
                Some(TriggerReason::WeeklyRefresh)
            } else {
                None
            }
        }
    }
}

/// 单个 cron tick:扫所有 actor,按 should_trigger 决策决定是否跑。
///
/// 返回 (触发数, 跳过数) 便于日志。`_interval_hours` 参数保留是为了向后兼容
/// 旧测试签名 / 未来可能的覆写;实际触发用 [`should_trigger`] 内部硬编码的
/// `WEEKLY_REFRESH_HOURS` / `MIN_RETRY_INTERVAL_HOURS`。
pub async fn distill_tick(
    distiller: &dyn MainlineDistiller,
    prefs: &dyn PrefsProvider,
    portfolio_storage: &PortfolioStorage,
    sandbox_base: &PathBuf,
    now: DateTime<Utc>,
    _interval_hours: i64,
) -> (u32, u32) {
    let mut triggered = 0u32;
    let mut skipped = 0u32;
    for (actor, portfolio) in portfolio_storage.list_all() {
        if !actor.is_direct() {
            skipped += 1;
            continue;
        }
        let holdings: Vec<String> = portfolio
            .holdings
            .iter()
            .map(|h| h.symbol.clone())
            .collect();
        if holdings.is_empty() {
            skipped += 1;
            continue;
        }
        let p = prefs.load(&actor);
        let reason = match should_trigger(&p, &holdings, now) {
            Some(r) => r,
            None => {
                skipped += 1;
                continue;
            }
        };
        info!(
            actor = %actor_dbg(&actor),
            holdings = holdings.len(),
            reason = reason.as_str(),
            "mainline distill cron: triggering for actor"
        );
        match distill_and_persist_one(distiller, prefs, sandbox_base, &actor, &holdings).await {
            Ok(updated) => {
                triggered += 1;
                info!(
                    actor = %actor_dbg(&actor),
                    reason = reason.as_str(),
                    distilled = updated.mainline_by_ticker.as_ref().map(|m| m.len()).unwrap_or(0),
                    skipped_tickers = updated.mainline_distill_skipped.len(),
                    "mainline distill cron: actor done"
                );
            }
            Err(e) => {
                warn!(
                    actor = %actor_dbg(&actor),
                    reason = reason.as_str(),
                    "mainline distill cron failed: {e:#}"
                );
            }
        }
    }
    (triggered, skipped)
}

fn actor_dbg(a: &ActorIdentity) -> String {
    format!(
        "{}:{}:{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

/// 把 distill_tick 包装成长期运行的循环,适合 `tokio::spawn`。
///
/// 每小时 tick 一次,实际触发由 `distill_tick` 里的 staleness 判断决定。
///
/// `task_runs_dir` 不为 `None` 时每次 tick 末尾会写一行
/// `data/runtime/task_runs.YYYY-MM-DD.jsonl`(Stage 3 任务观测,跟 heartbeat 同级)。
pub async fn distill_cron_loop(
    distiller: Arc<dyn MainlineDistiller>,
    prefs: Arc<dyn PrefsProvider>,
    portfolio_storage: Arc<PortfolioStorage>,
    sandbox_base: PathBuf,
    interval_hours: i64,
    task_runs_dir: Option<std::sync::Arc<PathBuf>>,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(3600));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        ticker.tick().await;
        let now = Utc::now();
        let started_at = now;
        let (t, s) = distill_tick(
            distiller.as_ref(),
            prefs.as_ref(),
            portfolio_storage.as_ref(),
            &sandbox_base,
            now,
            interval_hours,
        )
        .await;
        if t > 0 {
            info!(
                task = "mainline_cron",
                triggered = t,
                skipped = s,
                "mainline distill cron tick complete"
            );
        }
        if let Some(dir) = task_runs_dir.as_deref() {
            // distill_tick 只返回成败计数,本身不抛 Err(失败 actor 已在内部 warn);
            // 整条 tick 总是 Ok,outcome 用 triggered 数区分:>0 → ok, =0 → skipped。
            if t > 0 {
                hone_core::task_observer::record_ok(dir, "mainline_cron", started_at, t as u64);
            } else {
                hone_core::task_observer::record_skipped(dir, "mainline_cron", started_at);
            }
        }
        let _ = s; // s 已经在 info! 用了,这里再次 silenced 让无 task_runs_dir 路径通过 warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_digest::mainline_distill::{
        MainlineDistiller, ProfileSource, actor_sandbox_dir,
    };
    use crate::prefs::{FilePrefsStorage, NotificationPrefs};
    use async_trait::async_trait;
    use hone_memory::portfolio::{Holding, Portfolio};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    struct CountingDistiller {
        calls: AtomicUsize,
    }
    #[async_trait]
    impl MainlineDistiller for CountingDistiller {
        async fn distill_mainline(&self, ticker: &str, _profile: &str) -> anyhow::Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(format!("auto: {ticker}"))
        }
        async fn distill_style(&self, _profiles: &[ProfileSource]) -> anyhow::Result<String> {
            Ok("global style".into())
        }
    }

    fn write_profile(sandbox_base: &std::path::Path, actor: &ActorIdentity, ticker: &str) {
        let dir = actor_sandbox_dir(sandbox_base, actor)
            .join("company_profiles")
            .join(ticker.to_lowercase());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("profile.md"),
            format!("ticker: {ticker}\n\n## 投资主线\nlong content"),
        )
        .unwrap();
    }

    fn make_portfolio(actor: ActorIdentity, symbols: Vec<&str>) -> Portfolio {
        Portfolio {
            actor: Some(actor.clone()),
            user_id: actor.user_id.clone(),
            holdings: symbols
                .into_iter()
                .map(|s| Holding {
                    symbol: s.to_string(),
                    asset_type: "stock".into(),
                    shares: 100.0,
                    avg_cost: 0.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: None,
                })
                .collect(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn distill_tick_skips_actor_without_portfolio() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, s) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &dir.path().join("sandboxes"),
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 0);
        assert_eq!(s, 0); // 没 actor 就没 actor
        assert_eq!(distiller.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn distill_tick_runs_when_no_prior_timestamp() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        portfolios
            .save(&actor, &make_portfolio(actor.clone(), vec!["MU", "RKLB"]))
            .unwrap();
        let sandbox_base = dir.path().join("sandboxes");
        write_profile(&sandbox_base, &actor, "MU");
        write_profile(&sandbox_base, &actor, "RKLB");

        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, _s) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &sandbox_base,
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 1, "actor 应被触发蒸馏");
        let p = prefs.load(&actor);
        assert!(p.last_mainline_distilled_at.is_some());
        assert_eq!(p.mainline_by_ticker.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn distill_tick_skips_when_full_coverage_and_recent() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        portfolios
            .save(&actor, &make_portfolio(actor.clone(), vec!["MU"]))
            .unwrap();
        let sandbox_base = dir.path().join("sandboxes");
        write_profile(&sandbox_base, &actor, "MU");

        // 全覆盖 + 5 分钟前刚蒸过 → 应跳过
        let mut p = NotificationPrefs::default();
        let mut by_ticker = std::collections::HashMap::new();
        by_ticker.insert("MU".to_string(), "existing mainline".to_string());
        p.mainline_by_ticker = Some(by_ticker);
        p.last_mainline_distilled_at =
            Some((Utc::now() - chrono::Duration::minutes(5)).to_rfc3339());
        prefs.save(&actor, &p).unwrap();

        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, _) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &sandbox_base,
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 0, "全覆盖 + 5 分钟前蒸过应跳过");
    }

    #[tokio::test]
    async fn distill_tick_triggers_when_holding_missing_thesis() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        portfolios
            .save(&actor, &make_portfolio(actor.clone(), vec!["MU", "RKLB"]))
            .unwrap();
        let sandbox_base = dir.path().join("sandboxes");
        write_profile(&sandbox_base, &actor, "MU");
        write_profile(&sandbox_base, &actor, "RKLB");

        // 只有 MU 有主线,RKLB 缺;距上次 12 小时(已过 MIN_RETRY=6h)
        let mut p = NotificationPrefs::default();
        let mut by_ticker = std::collections::HashMap::new();
        by_ticker.insert("MU".to_string(), "existing".to_string());
        p.mainline_by_ticker = Some(by_ticker);
        p.last_mainline_distilled_at =
            Some((Utc::now() - chrono::Duration::hours(12)).to_rfc3339());
        prefs.save(&actor, &p).unwrap();

        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, _) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &sandbox_base,
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 1, "RKLB 缺主线 应立即触发");
    }

    #[tokio::test]
    async fn distill_tick_skips_missing_holdings_within_min_retry() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        portfolios
            .save(&actor, &make_portfolio(actor.clone(), vec!["MU", "AAPL"]))
            .unwrap();
        let sandbox_base = dir.path().join("sandboxes");
        write_profile(&sandbox_base, &actor, "MU");
        // AAPL 故意没画像

        // 1 小时前刚试过(< MIN_RETRY=6h),即使 AAPL missing 也应跳过避免循环
        let mut p = NotificationPrefs::default();
        let mut by_ticker = std::collections::HashMap::new();
        by_ticker.insert("MU".to_string(), "x".to_string());
        p.mainline_by_ticker = Some(by_ticker);
        p.mainline_distill_skipped = vec!["AAPL".to_string()];
        p.last_mainline_distilled_at = Some((Utc::now() - chrono::Duration::hours(1)).to_rfc3339());
        prefs.save(&actor, &p).unwrap();

        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, _) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &sandbox_base,
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 0, "1 小时前刚试过应跳过(MIN_RETRY 节流)");
    }

    #[tokio::test]
    async fn distill_tick_weekly_refresh_when_full_coverage_and_old() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        portfolios
            .save(&actor, &make_portfolio(actor.clone(), vec!["MU"]))
            .unwrap();
        let sandbox_base = dir.path().join("sandboxes");
        write_profile(&sandbox_base, &actor, "MU");

        // 全覆盖 + 8 天前蒸过 → 周更新触发
        let mut p = NotificationPrefs::default();
        let mut by_ticker = std::collections::HashMap::new();
        by_ticker.insert("MU".to_string(), "existing mainline".to_string());
        p.mainline_by_ticker = Some(by_ticker);
        p.last_mainline_distilled_at = Some((Utc::now() - chrono::Duration::days(8)).to_rfc3339());
        prefs.save(&actor, &p).unwrap();

        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, _) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &sandbox_base,
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 1, "全覆盖 + 8 天前蒸过 → 周更新触发");
    }

    #[test]
    fn should_trigger_first_run_when_no_timestamp() {
        let prefs = NotificationPrefs::default();
        let holdings = vec!["MU".to_string()];
        assert_eq!(
            should_trigger(&prefs, &holdings, Utc::now()),
            Some(TriggerReason::FirstRun)
        );
    }

    #[test]
    fn should_trigger_missing_holdings_after_min_retry() {
        let mut prefs = NotificationPrefs::default();
        prefs.last_mainline_distilled_at =
            Some((Utc::now() - chrono::Duration::hours(7)).to_rfc3339());
        let holdings = vec!["MU".to_string(), "RKLB".to_string()];
        // 完全没主线 → 全部 missing
        assert_eq!(
            should_trigger(&prefs, &holdings, Utc::now()),
            Some(TriggerReason::MissingHoldings)
        );
    }

    #[test]
    fn should_trigger_none_when_full_coverage_within_weekly() {
        let mut prefs = NotificationPrefs::default();
        let mut by_ticker = std::collections::HashMap::new();
        by_ticker.insert("MU".to_string(), "x".to_string());
        prefs.mainline_by_ticker = Some(by_ticker);
        prefs.last_mainline_distilled_at =
            Some((Utc::now() - chrono::Duration::days(2)).to_rfc3339());
        let holdings = vec!["MU".to_string()];
        assert_eq!(should_trigger(&prefs, &holdings, Utc::now()), None);
    }

    #[test]
    fn should_trigger_case_insensitive_ticker_match() {
        let mut prefs = NotificationPrefs::default();
        let mut by_ticker = std::collections::HashMap::new();
        // map 里大写
        by_ticker.insert("MU".to_string(), "x".to_string());
        prefs.mainline_by_ticker = Some(by_ticker);
        prefs.last_mainline_distilled_at =
            Some((Utc::now() - chrono::Duration::hours(1)).to_rfc3339());
        // holdings 小写也算覆盖
        let holdings = vec!["mu".to_string()];
        assert_eq!(should_trigger(&prefs, &holdings, Utc::now()), None);
    }

    #[tokio::test]
    async fn distill_tick_triggers_on_invalid_timestamp() {
        let dir = tempdir().unwrap();
        let prefs = FilePrefsStorage::new(dir.path().join("prefs")).unwrap();
        let portfolios = PortfolioStorage::new(dir.path().join("portfolios"));
        let actor = ActorIdentity::new("telegram", "u1", None::<&str>).unwrap();
        portfolios
            .save(&actor, &make_portfolio(actor.clone(), vec!["MU"]))
            .unwrap();
        let sandbox_base = dir.path().join("sandboxes");
        write_profile(&sandbox_base, &actor, "MU");

        let mut p = NotificationPrefs::default();
        p.last_mainline_distilled_at = Some("not-a-date".into());
        prefs.save(&actor, &p).unwrap();

        let distiller = CountingDistiller {
            calls: AtomicUsize::new(0),
        };
        let (t, _) = distill_tick(
            &distiller,
            &prefs,
            &portfolios,
            &sandbox_base,
            Utc::now(),
            DEFAULT_DISTILL_INTERVAL_HOURS,
        )
        .await;
        assert_eq!(t, 1, "无效时间戳应按 due 处理");
        // 蒸馏后 last_distilled_at 应被覆盖成合法 RFC3339
        let reloaded = prefs.load(&actor);
        assert!(
            DateTime::parse_from_rfc3339(reloaded.last_mainline_distilled_at.as_ref().unwrap())
                .is_ok()
        );
    }
}
