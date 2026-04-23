//! 订阅层 — 事件到 actor 的映射。
//!
//! MVP 提供两类订阅：
//! - `PortfolioSubscription`：一个 actor 一个实例，命中条件 =
//!   `event.symbols` 与该 actor 的 holdings 有交集。
//! - `GlobalSubscription`：覆盖所有传入 actor，用于宏观事件等全员播报场景。
//!
//! 未来的 `NaturalLanguageSubscription` 只需实现 `Subscription` trait 即可挂入
//! 注册中心，事件引擎无须修改。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use hone_core::ActorIdentity;
use hone_memory::PortfolioStorage;

use crate::event::{MarketEvent, Severity};

/// 订阅 trait。引擎把事件依次交给每个 sub；sub 自行判断命中与受众。
pub trait Subscription: Send + Sync {
    fn id(&self) -> &str;
    fn matches(&self, event: &MarketEvent) -> bool;
    fn actors(&self) -> Vec<ActorIdentity>;
    /// 可选：按事件上下文升降级 severity；默认不改。
    fn severity_override(&self, _event: &MarketEvent) -> Option<Severity> {
        None
    }
    /// 返回该订阅关心的 symbol 列表（用于 price watch pool 聚合）。
    /// 默认空 —— GlobalSubscription / 宏观订阅无 symbol 概念。
    fn watch_symbols(&self) -> Vec<String> {
        Vec::new()
    }
}

/// 基于持仓的订阅（一个 actor 一个实例）。
pub struct PortfolioSubscription {
    id: String,
    actor: ActorIdentity,
    symbols: HashSet<String>,
}

impl PortfolioSubscription {
    pub fn new(actor: ActorIdentity, symbols: impl IntoIterator<Item = String>) -> Self {
        let id = format!(
            "portfolio:{}:{}:{}",
            actor.channel,
            actor.channel_scope.clone().unwrap_or_default(),
            actor.user_id,
        );
        let symbols = symbols
            .into_iter()
            .map(|s| s.to_ascii_uppercase())
            .filter(|s| !s.is_empty())
            .collect();
        Self { id, actor, symbols }
    }

    pub fn symbols(&self) -> &HashSet<String> {
        &self.symbols
    }
}

impl Subscription for PortfolioSubscription {
    fn id(&self) -> &str {
        &self.id
    }

    fn matches(&self, event: &MarketEvent) -> bool {
        event
            .symbols
            .iter()
            .any(|s| self.symbols.contains(&s.to_ascii_uppercase()))
    }

    fn actors(&self) -> Vec<ActorIdentity> {
        vec![self.actor.clone()]
    }

    fn watch_symbols(&self) -> Vec<String> {
        self.symbols.iter().cloned().collect()
    }
}

/// 对所有注册 actor 统一分发的订阅（用于宏观事件）。
pub struct GlobalSubscription {
    id: String,
    actors: Vec<ActorIdentity>,
    only_kinds: Option<HashSet<String>>,
}

impl GlobalSubscription {
    pub fn new(id: impl Into<String>, actors: Vec<ActorIdentity>) -> Self {
        Self {
            id: id.into(),
            actors,
            only_kinds: None,
        }
    }

    /// 限定只对某些 kind 命中（kind 序列化 tag，例如 `"macro_event"`）。
    pub fn with_kinds(mut self, kinds: impl IntoIterator<Item = String>) -> Self {
        self.only_kinds = Some(kinds.into_iter().collect());
        self
    }
}

impl Subscription for GlobalSubscription {
    fn id(&self) -> &str {
        &self.id
    }

    fn matches(&self, event: &MarketEvent) -> bool {
        let Some(allow) = &self.only_kinds else {
            return true;
        };
        let tag = serde_json::to_value(&event.kind)
            .ok()
            .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(str::to_string))
            .unwrap_or_default();
        allow.contains(&tag)
    }

    fn actors(&self) -> Vec<ActorIdentity> {
        self.actors.clone()
    }
}

/// 订阅注册中心。
pub struct SubscriptionRegistry {
    subs: Vec<Box<dyn Subscription>>,
}

impl SubscriptionRegistry {
    pub fn new() -> Self {
        Self { subs: Vec::new() }
    }

    pub fn register(&mut self, sub: Box<dyn Subscription>) {
        self.subs.push(sub);
    }

    pub fn len(&self) -> usize {
        self.subs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subs.is_empty()
    }

    /// 所有命中订阅的 actor（去重,仅保留单聊）。
    /// 用于 DailyReport 这类"给每个注册 actor 都推一条"的广播场景。
    pub fn actors(&self) -> Vec<ActorIdentity> {
        use std::collections::HashMap;
        let mut dedup: HashMap<String, ActorIdentity> = HashMap::new();
        for sub in &self.subs {
            for a in sub.actors() {
                if !a.is_direct() {
                    continue;
                }
                dedup.insert(actor_storage_key(&a), a);
            }
        }
        let mut out: Vec<ActorIdentity> = dedup.into_values().collect();
        out.sort_by(|a, b| actor_storage_key(a).cmp(&actor_storage_key(b)));
        out
    }

    /// 聚合所有订阅关心的 symbol —— 用于 PricePoller 的 watch pool。
    pub fn watch_pool(&self) -> Vec<String> {
        let mut set: HashSet<String> = HashSet::new();
        for sub in &self.subs {
            for s in sub.watch_symbols() {
                set.insert(s.to_ascii_uppercase());
            }
        }
        let mut out: Vec<String> = set.into_iter().collect();
        out.sort();
        out
    }

    /// 返回所有命中该事件的 (actor, effective_severity) 对。
    /// 同一 actor 可能被多个 sub 命中 —— 去重后取最高 severity。
    ///
    /// **硬规则**：群聊（`channel_scope` 非空）一律不推送，直接过滤掉。
    /// 主动推送只走单聊，这是产品决策——group scope 一律在此拦截。
    pub fn resolve(&self, event: &MarketEvent) -> Vec<(ActorIdentity, Severity)> {
        use std::collections::HashMap;
        let mut best: HashMap<String, (ActorIdentity, Severity)> = HashMap::new();
        for sub in &self.subs {
            if !sub.matches(event) {
                continue;
            }
            let sev = sub.severity_override(event).unwrap_or(event.severity);
            for actor in sub.actors() {
                if !actor.is_direct() {
                    continue; // 群聊：硬跳过
                }
                let key = actor_storage_key(&actor);
                best.entry(key)
                    .and_modify(|(_, cur)| {
                        if severity_rank(sev) > severity_rank(*cur) {
                            *cur = sev;
                        }
                    })
                    .or_insert((actor, sev));
            }
        }
        best.into_values().collect()
    }
}

impl Default for SubscriptionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn severity_rank(s: Severity) -> u8 {
    match s {
        Severity::Low => 0,
        Severity::Medium => 1,
        Severity::High => 2,
    }
}

pub(crate) fn actor_storage_key(a: &ActorIdentity) -> String {
    format!(
        "{}::{}::{}",
        a.channel,
        a.channel_scope.clone().unwrap_or_default(),
        a.user_id
    )
}

/// 线程安全的 registry 容器——支持运行时热刷新。
///
/// 读路径：`load()` 拿 `Arc<SubscriptionRegistry>` 快照，读锁极短、只克隆 Arc，
/// 之后 resolve/watch_pool 都走快照，不阻塞刷新。
///
/// 写路径：`refresh()` 从磁盘重读 `portfolio_dir` 下所有 actor 持仓，
/// 构造新 Registry 后原子替换。调用方可以定时触发（后台任务），
/// 也可以在 portfolio 保存路径上显式触发。
pub struct SharedRegistry {
    inner: RwLock<Arc<SubscriptionRegistry>>,
    portfolio_dir: Option<PathBuf>,
}

impl SharedRegistry {
    /// 从 portfolio 目录构建：初次读盘填充 registry，并记住目录以便后续 refresh。
    pub fn from_portfolio_dir(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let storage = PortfolioStorage::new(&dir);
        let reg = registry_from_portfolios(&storage);
        Self {
            inner: RwLock::new(Arc::new(reg)),
            portfolio_dir: Some(dir),
        }
    }

    /// 基于已有 registry 构造（测试用 / 外部手工装配订阅）；refresh 将不可用。
    pub fn from_registry(reg: SubscriptionRegistry) -> Self {
        Self {
            inner: RwLock::new(Arc::new(reg)),
            portfolio_dir: None,
        }
    }

    /// 拿当前注册表快照。读锁极短——只是克隆一个 Arc。
    pub fn load(&self) -> Arc<SubscriptionRegistry> {
        self.inner
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|p| p.into_inner().clone())
    }

    /// 从磁盘重建 registry 并原子替换。返回新 registry 的订阅数。
    /// 若未绑定 portfolio 目录（测试构造），返回 `None`。
    pub fn refresh(&self) -> Option<usize> {
        let dir = self.portfolio_dir.as_ref()?;
        let storage = PortfolioStorage::new(dir);
        let new_reg = registry_from_portfolios(&storage);
        let n = new_reg.len();
        match self.inner.write() {
            Ok(mut g) => *g = Arc::new(new_reg),
            Err(p) => *p.into_inner() = Arc::new(new_reg),
        }
        Some(n)
    }

    pub fn portfolio_dir(&self) -> Option<&Path> {
        self.portfolio_dir.as_deref()
    }
}

/// 扫描 PortfolioStorage 下所有 actor，构建一份初始 Registry。
///
/// 组成：
/// - 每个有持仓的 direct actor → `PortfolioSubscription`（按 ticker 命中）
/// - 所有 direct actor 汇总后 → 一个 `GlobalSubscription`(kinds=[`social_post`,
///   `macro_event`])。社交事件默认进 digest/LLM 仲裁；宏观事件经 router 的
///   due-window 保护后，远期日历只进摘要，临近 high 才即时播报。
pub fn registry_from_portfolios(storage: &PortfolioStorage) -> SubscriptionRegistry {
    let mut reg = SubscriptionRegistry::new();
    let mut direct_actors: Vec<ActorIdentity> = Vec::new();
    for (actor, portfolio) in storage.list_all() {
        // 硬规则：群聊持仓不订阅——主动推送只走单聊。
        // 这里跳过可以避免群聊 holdings 污染 watch pool，省下不必要的 FMP 拉取。
        if !actor.is_direct() {
            tracing::debug!(
                channel = %actor.channel,
                scope = ?actor.channel_scope,
                "skip group portfolio for push subscription"
            );
            continue;
        }
        let symbols: Vec<String> = portfolio
            .holdings
            .iter()
            .map(|h| h.symbol.clone())
            .collect();
        // 即便 holdings 为空,也把 actor 纳入 social 全员订阅——社交源帖子无 ticker,
        // 否则会被 registry.resolve 漏过、连 LLM 仲裁都走不到。
        if !symbols.is_empty() {
            reg.register(Box::new(PortfolioSubscription::new(actor.clone(), symbols)));
        }
        direct_actors.push(actor);
    }
    if !direct_actors.is_empty() {
        reg.register(Box::new(
            GlobalSubscription::new("social_global", direct_actors)
                .with_kinds(["social_post".to_string(), "macro_event".to_string()]),
        ));
    }
    reg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, MarketEvent, Severity};
    use chrono::Utc;

    fn actor(channel: &str, user: &str) -> ActorIdentity {
        ActorIdentity::new(channel, user, None::<&str>).unwrap()
    }

    fn ev(id: &str, sym: &str, sev: Severity, kind: EventKind) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind,
            severity: sev,
            symbols: vec![sym.into()],
            occurred_at: Utc::now(),
            title: "t".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn portfolio_sub_matches_case_insensitive() {
        let sub = PortfolioSubscription::new(actor("imessage", "u1"), vec!["aapl".into()]);
        assert!(sub.matches(&ev(
            "e1",
            "AAPL",
            Severity::Medium,
            EventKind::EarningsUpcoming
        )));
        assert!(!sub.matches(&ev(
            "e2",
            "TSLA",
            Severity::Medium,
            EventKind::EarningsUpcoming
        )));
    }

    #[test]
    fn registry_dedups_actor_and_keeps_max_severity() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("imessage", "u1"),
            vec!["AAPL".into()],
        )));
        // 第二个订阅也命中同一 actor，但 severity_override 抬升到 High
        struct Upgrader(ActorIdentity);
        impl Subscription for Upgrader {
            fn id(&self) -> &str {
                "upg"
            }
            fn matches(&self, _e: &MarketEvent) -> bool {
                true
            }
            fn actors(&self) -> Vec<ActorIdentity> {
                vec![self.0.clone()]
            }
            fn severity_override(&self, _e: &MarketEvent) -> Option<Severity> {
                Some(Severity::High)
            }
        }
        reg.register(Box::new(Upgrader(actor("imessage", "u1"))));

        let hits = reg.resolve(&ev(
            "e1",
            "AAPL",
            Severity::Medium,
            EventKind::EarningsUpcoming,
        ));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].1, Severity::High);
    }

    #[test]
    fn global_sub_kind_filter() {
        let sub = GlobalSubscription::new("g1", vec![actor("imessage", "u1")])
            .with_kinds(["macro_event".to_string()]);
        assert!(sub.matches(&ev("e", "", Severity::Low, EventKind::MacroEvent)));
        assert!(!sub.matches(&ev("e", "AAPL", Severity::Low, EventKind::EarningsUpcoming)));
    }

    #[test]
    fn registry_from_portfolios_broadcasts_macro_to_direct_actors() {
        use hone_memory::PortfolioStorage;
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path());
        let a = actor("telegram", "u_macro");
        storage.upsert_watch(&a, "AAPL", "stock").unwrap();

        let reg = registry_from_portfolios(&storage);
        let mut macro_ev = ev("macro", "", Severity::High, EventKind::MacroEvent);
        macro_ev.symbols.clear();
        let hits = reg.resolve(&macro_ev);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0.user_id, "u_macro");
    }

    #[test]
    fn resolve_skips_group_actors() {
        // 哪怕 Subscription 命中，群聊 actor 都应该被硬过滤。
        let mut reg = SubscriptionRegistry::new();
        let group_actor = actor("feishu", "u1");
        let group_actor =
            ActorIdentity::new(group_actor.channel, group_actor.user_id, Some("chat:42")).unwrap();
        reg.register(Box::new(PortfolioSubscription::new(
            group_actor.clone(),
            vec!["AAPL".into()],
        )));
        // 一个单聊 actor 同 symbol
        reg.register(Box::new(PortfolioSubscription::new(
            actor("feishu", "u2"),
            vec!["AAPL".into()],
        )));
        let hits = reg.resolve(&ev(
            "e1",
            "AAPL",
            Severity::High,
            EventKind::EarningsReleased,
        ));
        assert_eq!(hits.len(), 1);
        assert!(hits[0].0.is_direct(), "群 actor 不应出现在推送目标里");
        assert_eq!(hits[0].0.user_id, "u2");
    }

    #[test]
    fn registry_from_portfolios_skips_group_portfolios() {
        use hone_memory::PortfolioStorage;
        use hone_memory::portfolio::{Holding, Portfolio};
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path());

        // 单聊 actor → 应注册
        let dm = actor("feishu", "u1");
        let p_dm = Portfolio {
            actor: Some(dm.clone()),
            user_id: "u1".into(),
            holdings: vec![Holding {
                symbol: "AAPL".into(),
                asset_type: "stock".into(),
                shares: 1.0,
                avg_cost: 100.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                tracking_only: None,
            }],
            updated_at: "2026-04-21".into(),
        };
        storage.save(&dm, &p_dm).unwrap();

        // 群 actor → 应跳过
        let group = ActorIdentity::new("feishu", "u2", Some("chat:42")).unwrap();
        let p_group = Portfolio {
            actor: Some(group.clone()),
            user_id: "u2".into(),
            holdings: vec![Holding {
                symbol: "NVDA".into(),
                asset_type: "stock".into(),
                shares: 1.0,
                avg_cost: 100.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                tracking_only: None,
            }],
            updated_at: "2026-04-21".into(),
        };
        storage.save(&group, &p_group).unwrap();

        let reg = registry_from_portfolios(&storage);
        // 1 direct actor portfolio sub + 1 社交/宏观全员 sub(direct actor 集合)
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.watch_pool(), vec!["AAPL"], "NVDA 来自群持仓应被跳过");
    }

    #[test]
    fn resolve_empty_when_no_subs_match() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("imessage", "u1"),
            vec!["AAPL".into()],
        )));
        let hits = reg.resolve(&ev(
            "e",
            "NVDA",
            Severity::Medium,
            EventKind::EarningsUpcoming,
        ));
        assert!(hits.is_empty());
    }

    #[test]
    fn watch_pool_aggregates_portfolio_symbols_and_dedups() {
        let mut reg = SubscriptionRegistry::new();
        reg.register(Box::new(PortfolioSubscription::new(
            actor("imessage", "u1"),
            vec!["AAPL".into(), "msft".into()],
        )));
        reg.register(Box::new(PortfolioSubscription::new(
            actor("imessage", "u2"),
            vec!["MSFT".into(), "NVDA".into()],
        )));
        // GlobalSubscription 不贡献 watch pool
        reg.register(Box::new(
            GlobalSubscription::new("g1", vec![actor("imessage", "u3")])
                .with_kinds(["macro_event".to_string()]),
        ));
        let pool = reg.watch_pool();
        assert_eq!(pool, vec!["AAPL", "MSFT", "NVDA"]);
    }

    #[test]
    fn shared_registry_refresh_picks_up_new_portfolio() {
        use hone_memory::PortfolioStorage;
        use hone_memory::portfolio::{Holding, Portfolio};
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path());

        // 初始：无持仓
        let shared = SharedRegistry::from_portfolio_dir(dir.path());
        assert!(shared.load().watch_pool().is_empty());
        assert_eq!(shared.load().len(), 0);

        // 用户上线并写入持仓
        let u1 = actor("telegram", "u1");
        let portfolio = Portfolio {
            actor: Some(u1.clone()),
            user_id: "u1".into(),
            holdings: vec![Holding {
                symbol: "AAPL".into(),
                asset_type: "stock".into(),
                shares: 10.0,
                avg_cost: 100.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                tracking_only: None,
            }],
            updated_at: "2026-04-21".into(),
        };
        storage.save(&u1, &portfolio).unwrap();

        // 未刷新前：依然为空
        assert!(shared.load().watch_pool().is_empty());

        // refresh 后：新持仓可见，resolve 立即命中
        // 1 portfolio sub + 1 social/macro global sub(direct actor 汇总)
        let n = shared.refresh().unwrap();
        assert_eq!(n, 2);
        assert_eq!(shared.load().watch_pool(), vec!["AAPL"]);
        let hits = shared.load().resolve(&ev(
            "e1",
            "AAPL",
            Severity::High,
            EventKind::EarningsReleased,
        ));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0.user_id, "u1");
    }

    #[test]
    fn shared_registry_from_registry_has_no_refresh() {
        let reg = SubscriptionRegistry::new();
        let shared = SharedRegistry::from_registry(reg);
        assert!(
            shared.refresh().is_none(),
            "无 portfolio 目录时 refresh 应返回 None"
        );
    }

    #[test]
    fn registry_from_portfolios_skips_empty_holdings() {
        use hone_memory::PortfolioStorage;
        use hone_memory::portfolio::{Holding, Portfolio};
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path());
        let a = actor("imessage", "u1");
        let p = Portfolio {
            actor: Some(a.clone()),
            user_id: "u1".into(),
            holdings: vec![Holding {
                symbol: "AAPL".into(),
                asset_type: "stock".into(),
                shares: 10.0,
                avg_cost: 100.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                tracking_only: None,
            }],
            updated_at: "2026-04-21".into(),
        };
        storage.save(&a, &p).unwrap();

        // 再加一个空持仓 actor
        let a2 = actor("imessage", "u2");
        let p2 = Portfolio {
            actor: Some(a2.clone()),
            user_id: "u2".into(),
            holdings: vec![],
            updated_at: "2026-04-21".into(),
        };
        storage.save(&a2, &p2).unwrap();

        let reg = registry_from_portfolios(&storage);
        // 1 portfolio sub(a has AAPL)+ 1 social/macro global sub(包含 a 和 a2),
        // 空持仓 a2 不单独产生 PortfolioSubscription
        assert_eq!(reg.len(), 2, "空持仓不应产生 PortfolioSubscription");
    }

    /// 不变量：仅关注（tracking_only=true）的 symbol 也必须进入 watch_pool 与 resolve。
    /// 锁死"关注与持仓同级推送"的契约——未来若有人误给 registry_from_portfolios
    /// 加上 `!h.tracking_only.unwrap_or(false)` 过滤,这条测试会立即失败。
    #[test]
    fn registry_from_portfolios_includes_tracking_only_symbols() {
        use hone_memory::PortfolioStorage;
        use hone_memory::portfolio::{Holding, Portfolio};
        let dir = tempfile::tempdir().unwrap();
        let storage = PortfolioStorage::new(dir.path());

        let a = actor("imessage", "u_watch");
        storage
            .upsert_watch(&a, "NVDA", "stock")
            .expect("upsert watch");

        let reg = registry_from_portfolios(&storage);
        // 1 portfolio sub(仅关注 NVDA 也会创建 PortfolioSub)+ 1 social/macro global sub
        assert_eq!(reg.len(), 2, "仅关注的 actor 也应注册订阅");
        assert_eq!(reg.watch_pool(), vec!["NVDA"]);

        let hits = reg.resolve(&ev(
            "e-watchlist",
            "NVDA",
            Severity::High,
            EventKind::NewsCritical,
        ));
        assert_eq!(hits.len(), 1, "关注标的应命中推送路由");
        assert_eq!(hits[0].0.user_id, "u_watch");

        // 验证即使与真实持仓混用,关注项仍会贡献 symbol
        let a2 = actor("telegram", "u_mixed");
        let p_mixed = Portfolio {
            actor: Some(a2.clone()),
            user_id: "u_mixed".into(),
            holdings: vec![
                Holding {
                    symbol: "AAPL".into(),
                    asset_type: "stock".into(),
                    shares: 10.0,
                    avg_cost: 180.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: None,
                },
                Holding {
                    symbol: "TSLA".into(),
                    asset_type: "stock".into(),
                    shares: 0.0,
                    avg_cost: 0.0,
                    underlying: None,
                    option_type: None,
                    strike_price: None,
                    expiration_date: None,
                    contract_multiplier: None,
                    holding_horizon: None,
                    strategy_notes: None,
                    notes: None,
                    tracking_only: Some(true),
                },
            ],
            updated_at: "2026-04-22".into(),
        };
        storage.save(&a2, &p_mixed).unwrap();

        let reg = registry_from_portfolios(&storage);
        let mut pool = reg.watch_pool();
        pool.sort();
        assert_eq!(pool, vec!["AAPL", "NVDA", "TSLA"]);
    }
}
