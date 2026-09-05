//! 行业树 59 家的公司基础数据刷新 worker。
//!
//! 写进 `hone_memory::CompanyFactsStorage`（配了 PG 走 PG，否则退回文件），
//! 对话侧的 `data_fetch` 与研究台的行业树页面都从那一层读——**同一份**。
//!
//! ## 名单来自哪里
//!
//! `hone_core::industry_map` 的 59 家，**不是** `company-cards.json` 的 52 家。
//! 两个集合只交 35 家：行业树里有 24 家（NVDA、ASML、ANET、CEG、ORCL、SMCI、OKLO…）
//! 压根没有研究卡，而每日评级 worker 按卡的名单跑，所以它们至今一条基础数据都没有。
//! 行业树是「本产品认为值得跟的公司」的权威名单，基础数据就该按它来。
//!
//! ## 为什么是 19:00
//!
//! 现有的日更时段已经排满：19:10 周报、19:20 估值实验室、19:30 公司评级、
//! 19:50 影响者摘要、19:55 关键事件链、20:00 每日信号与持仓新闻。
//! 19:00 是这串里**最早**的一个空位，这一点是刻意的——公司事实是别人的输入
//! （估值实验室要股本算市值、评级要基本面），必须在第一个消费者（19:20）之前跑完。
//! 一轮 59 家、每家 5 个 provider 请求加一次 SEC，配速下大约两三分钟，
//! 留 20 分钟余量足够，也不会挤到 19:10 的周报。
//!
//! ## 事件驱动的当天重刷
//!
//! 财报发布不等日更。worker 每小时用**一次**全市场财报日历请求（不是 59 次）
//! 找出「今天刚发了财报」的成员，只重刷那几家。发了新财季而不重刷，
//! 这张表就会在最需要准确的那一天恰好是旧的。
//!
//! ## 降级
//!
//! 每家独立：一家 SEC 404、一家 provider 超时，都只写进那一家的 `degraded`，
//! 不影响其余 58 家落库，也不让整轮标记失败。SEC 整条通道不可用（没配可联系的
//! User-Agent、连不上）时，官方股本这一段沿用上一轮，其余字段照常刷新。
//!
//! ## 逐段合并，不整行覆盖
//!
//! 每一轮都从**上一轮那一行**起手，取到的段覆盖、取不到的段连同它自己的
//! [`FactProvenance`] 原样留着，并记进 `carried_over`。理由就是这张表存在的理由：
//! 事故的根因是「全链路没有任何一处保存过官方股本」，如果一次 SEC 5xx 就能把昨天
//! 拿到的封面股数抹成空，这一处照样保存不住——上游抖一下变成我们自己的永久数据丢失，
//! 而下游（研究台市值、对话里的股本块）会原样复发 LITE 那个事故，持续到下一个 19:00。
//!
//! 与之配套的一条：**本轮一段都没成功时不推进 `refreshed_at`**。否则一行空数据会
//! 自称新鲜，冷启动补刷的守卫跳过它，研究台也不报 `pipeline_stale`——没数据这件事
//! 就再也没人看得见。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde_json::Value;
use tracing::{info, warn};

use hone_core::sec_shares::{
    SecCoverShares, SecSharesAbsence, SecSharesClient, SecSharesOutcome, build_shares_cross_check,
    cover_age_days, cover_is_usable_for_market_cap, form_is_us_domestic_periodic,
};
use hone_memory::company_facts::{
    BalanceSheetFacts, CashFlowFacts, CompanyFacts, CompanyFactsStorage, CompanyIdentity,
    EarningsCadence, FactProvenance, IncomeFacts, ShareCounts,
};

use crate::routes::public_finance_calendar::{fetch_fmp_json_once, stable_fmp_base_url};
use crate::state::AppState;

/// 日更时刻，理由见模块文档。
const REFRESH_HOUR: u32 = 19;
const REFRESH_MINUTE: u32 = 0;

/// 财报发现的轮询间隔。一次 tick 只花一个 provider 请求（全市场日历），
/// 一小时一次即可在发布当天把那几家重刷到位，又不会把日历请求打成噪音。
const EARNINGS_WATCH_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// 每家之间的间隔。SEC 建议 <10 req/s；实测 0.12s 跑完 59 家没有被限流。
const PER_COMPANY_DELAY: Duration = Duration::from_millis(120);

/// 一次日更允许跑多久。超时就把已经落库的留着、剩下的等下一轮——
/// 半份新数据比一份卡住的 worker 好。
const SWEEP_BUDGET: Duration = Duration::from_secs(15 * 60);

/// 事件驱动重刷的冷却期。财报日历里一条记录会在窗口里挂约三天，而
/// `latest_reported_date` 取不到（本轮 earnings 段失败）时判据恒为 due——两者相乘
/// 就是同一家被每小时刷一次、连刷约 72 次，每次 5 个 provider 请求 + 1 次 SEC。
/// 公司一个季度才发一次财报，6 小时内刚试过就没有再试的道理。
const EARNINGS_REFRESH_COOLDOWN: chrono::Duration = chrono::Duration::hours(6);

/// 批量行情一次带多少个代码。与 `hone-event-engine` 的 `FMP_QUOTE_BATCH_SIZE` 同值：
/// 一次打 59 个是在赌 provider 不截断，而截断的表现是**静默**返回少几行。
const QUOTE_BATCH_SIZE: usize = 25;

const SOURCE_SEC_COVER: &str = "SEC XBRL dei:EntityCommonStockSharesOutstanding";
const SOURCE_FMP_PROFILE: &str = "provider profile";
const SOURCE_FMP_INCOME: &str = "provider income-statement (quarter)";
const SOURCE_FMP_BALANCE: &str = "provider balance-sheet-statement (quarter)";
const SOURCE_FMP_CASH_FLOW: &str = "provider cash-flow-statement (quarter)";
const SOURCE_FMP_EARNINGS: &str = "provider earnings calendar";

/// 行业树里全部成员的代码 + 名字，去重后保持文件顺序。
pub(crate) fn tracked_members(data_root: &std::path::Path) -> Vec<(String, String)> {
    let (map, _) = hone_core::industry_map::load(data_root);
    let mut seen: Vec<(String, String)> = Vec::new();
    for industry in &map.industries {
        for member in &industry.members {
            let symbol = member.symbol.trim().to_ascii_uppercase();
            if symbol.is_empty() || seen.iter().any(|(existing, _)| existing == &symbol) {
                continue;
            }
            seen.push((symbol, member.name.clone()));
        }
    }
    seen
}

/// 冷启动时先尽力刷一轮，然后在日更时刻与每小时的财报发现之间轮流醒来。
pub(crate) async fn company_facts_worker(state: Arc<AppState>) {
    cold_start_refresh(&state).await;
    let mut next_daily = next_refresh(Utc::now());
    let mut next_watch = Utc::now()
        + chrono::Duration::from_std(EARNINGS_WATCH_INTERVAL)
            .unwrap_or_else(|_| chrono::Duration::hours(1));
    loop {
        let wake = next_daily.min(next_watch);
        let wait = (wake - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        info!(
            next_daily = %next_daily,
            next_earnings_watch = %next_watch,
            "company facts worker waiting"
        );
        tokio::time::sleep(wait).await;

        let now = Utc::now();
        if now >= next_daily {
            refresh_all(&state).await;
            next_daily = next_refresh(now);
            // 全量刚跑完，下一次财报发现从现在起算，不必紧接着再来一次。
            next_watch = now
                + chrono::Duration::from_std(EARNINGS_WATCH_INTERVAL)
                    .unwrap_or_else(|_| chrono::Duration::hours(1));
            continue;
        }
        refresh_newly_reported(&state).await;
        next_watch = now
            + chrono::Duration::from_std(EARNINGS_WATCH_INTERVAL)
                .unwrap_or_else(|_| chrono::Duration::hours(1));
    }
}

fn next_refresh(now: DateTime<Utc>) -> DateTime<Utc> {
    crate::routes::research_store::next_local_refresh(now, REFRESH_HOUR, REFRESH_MINUTE)
}

/// 进程起来时的第一轮。
///
/// 表里已经有一份**新鲜**的数据就不重刷：发一次版、崩溃重启一次、多副本各起一个，
/// 都不该各自把 59 家 × 5 个 provider 请求再打一遍。只有整张表还没建起来、
/// 或者已经陈旧（连着两天没刷到）时才在启动时补一轮。
async fn cold_start_refresh(state: &AppState) {
    let data_root = state.core.config.storage.data_root();
    let members = tracked_members(&data_root);
    if members.is_empty() {
        return;
    }
    let storage = CompanyFactsStorage::new(&data_root);
    let symbols: Vec<String> = members.iter().map(|(symbol, _)| symbol.clone()).collect();
    let stored = storage.load_many(&symbols).await;
    let now = Utc::now();
    let fresh = symbols
        .iter()
        .filter(|symbol| {
            stored
                .get(*symbol)
                .is_some_and(|facts| !facts.is_stale(now))
        })
        .count();
    if fresh == symbols.len() {
        info!(
            companies = fresh,
            "company facts already fresh at startup; waiting for the daily slot"
        );
        return;
    }
    info!(
        fresh,
        total = symbols.len(),
        "company facts running a cold-start refresh"
    );
    refresh_symbols(state, &members, &symbols, "cold_start").await;
}

/// 整棵树全量刷新。返回成功落库的家数，供日志与测试用。
pub(crate) async fn refresh_all(state: &AppState) -> usize {
    let data_root = state.core.config.storage.data_root();
    let members = tracked_members(&data_root);
    if members.is_empty() {
        warn!("company facts refresh skipped: industry map has no members");
        return 0;
    }
    let symbols: Vec<String> = members.iter().map(|(symbol, _)| symbol.clone()).collect();
    refresh_symbols(state, &members, &symbols, "daily").await
}

/// 只刷「今天刚发了财报」的那几家。
///
/// 一次全市场日历请求就能覆盖所有成员，因此这条路径的成本与成员数无关。
/// 没有新发布、日历取不到、或者一家都没匹配上时都是静默返回：这是巡检，不是告警源。
pub(crate) async fn refresh_newly_reported(state: &AppState) -> usize {
    let data_root = state.core.config.storage.data_root();
    let members = tracked_members(&data_root);
    if members.is_empty() {
        return 0;
    }
    let Some(calendar) = fetch_recent_earnings_calendar(state).await else {
        return 0;
    };
    let reported = reported_dates_from_calendar(&calendar);
    if reported.is_empty() {
        return 0;
    }

    let storage = CompanyFactsStorage::new(&data_root);
    let all_symbols: Vec<String> = members.iter().map(|(symbol, _)| symbol.clone()).collect();
    let stored = storage.load_many(&all_symbols).await;

    let due = earnings_due_symbols(&all_symbols, &reported, &stored, Utc::now());

    if due.is_empty() {
        return 0;
    }
    info!(count = due.len(), symbols = ?due, "company facts refreshing newly reported companies");
    refresh_symbols(state, &members, &due, "earnings_event").await
}

/// 哪几家该在这一轮事件驱动里重刷。
///
/// 纯函数，因为这里的每一条都踩过坑：日历滞后、earnings 段取数失败、以及
/// 「没有冷却」——三者相乘的后果是同一家被每小时刷一次、连刷约 72 次，
/// 每次 5 个 provider 请求 + 1 次 SEC。
fn earnings_due_symbols(
    all_symbols: &[String],
    reported: &HashMap<String, String>,
    stored: &HashMap<String, CompanyFacts>,
    now: DateTime<Utc>,
) -> Vec<String> {
    all_symbols
        .iter()
        .filter(|symbol| {
            let Some(latest) = reported.get(*symbol) else {
                return false;
            };
            match stored.get(*symbol) {
                Some(facts) => {
                    // 刚试过就别再试。日历里一条记录会在窗口里挂约三天。
                    if attempted_within(facts, now, EARNINGS_REFRESH_COOLDOWN) {
                        return false;
                    }
                    match facts.earnings.latest_reported_date.as_deref() {
                        // 存过的：只有日历上的发布日**严格晚于**我们记下的那一天才重刷。
                        // 用「晚于」而不是「不等于」，日历回填一条更早的历史行不该触发重刷。
                        Some(known) => latest.as_str() > known,
                        // 我们这边压根不知道上一次是什么时候发的。这通常不是「有新财报」，
                        // 而是这一行的 earnings 段一直没取到过——把「不知道」读成「有新财报」，
                        // 等于让一个坏掉的端点自己给自己派活。只在这一行本来就该刷时才刷。
                        None => facts.is_stale(now),
                    }
                }
                // 没存过的：日更还没轮到它，这次顺手补上。
                None => true,
            }
        })
        .cloned()
        .collect()
}

/// 这一行最近一次**尝试**刷新是不是还在 `window` 之内。尝试时刻读不出来就当成
/// 「很久没试过」——宁可多刷一轮，也不要因为一个坏时间戳把一家永远排除在外。
fn attempted_within(facts: &CompanyFacts, now: DateTime<Utc>, window: chrono::Duration) -> bool {
    facts
        .last_refresh_attempt_at
        .as_deref()
        .and_then(|at| DateTime::parse_from_rfc3339(at).ok())
        .is_some_and(|at| now - at.with_timezone(&Utc) < window)
}

async fn refresh_symbols(
    state: &AppState,
    members: &[(String, String)],
    symbols: &[String],
    trigger: &str,
) -> usize {
    let started = std::time::Instant::now();
    let data_root = state.core.config.storage.data_root();
    let storage = CompanyFactsStorage::new(&data_root);
    let pool = state.core.config.fmp.effective_key_pool();
    let keys = pool.keys().to_vec();
    let sec = SecSharesClient::new(
        &state
            .core
            .config
            .event_engine
            .sec_filings
            .enrichment
            .user_agent,
    );
    if sec.is_none() {
        // SEC 通道关掉不是错误：其余字段照刷，官方股本这一段沿用上一轮（没有上一轮
        // 就留空）并在每家的 degraded 里写明，读取方据此降级措辞。
        // 默认配置里的 `ops@honeclaw.local` 不算联系方式——SEC 拿它找不到人，
        // 发出去换来的是限流甚至封禁，所以宁可关着。
        warn!(
            "company facts: SEC channel disabled; set event_engine.sec_filings.enrichment.user_agent              to a real contact address to turn official share counts back on"
        );
    }
    let names: HashMap<&str, &str> = members
        .iter()
        .map(|(symbol, name)| (symbol.as_str(), name.as_str()))
        .collect();

    // 一次批量行情拿全部 provider 股本。这里只取 sharesOutstanding：
    // price / marketCap 不进这张表，市值一律用「现价 × 表里的股本」现算。
    let provider_shares = if keys.is_empty() {
        HashMap::new()
    } else {
        fetch_provider_shares(state, &keys, symbols).await
    };

    // 上一轮那一行。取到就是这一轮的底稿：本轮取不到的段沿用它，而不是抹成空。
    let previous = storage.load_many(symbols).await;

    // 日期基准统一走 `local_now`：worker 的排程、财报日历窗口都是本地时刻，
    // 混用 UTC 会在一天里的某几个小时把「今天」差出一天。
    let today = hone_core::local_now().date_naive();
    let mut stored = 0usize;
    let mut degraded_companies = 0usize;
    let mut carried_companies = 0usize;
    for symbol in symbols {
        if started.elapsed() > SWEEP_BUDGET {
            warn!(
                trigger,
                done = stored,
                total = symbols.len(),
                "company facts sweep hit its time budget; the rest waits for the next round"
            );
            break;
        }
        let facts = build_company_facts(
            state,
            &keys,
            sec.as_ref(),
            symbol,
            names.get(symbol.as_str()).copied(),
            provider_shares.get(symbol).copied(),
            previous.get(symbol),
            today,
        )
        .await;
        if !facts.degraded.is_empty() {
            degraded_companies += 1;
        }
        if !facts.carried_over.is_empty() {
            carried_companies += 1;
        }
        // 一家写失败也只是这一家：其余的已经落库，后面的继续跑。
        match storage.save(&facts).await {
            Ok(()) => stored += 1,
            Err(error) => warn!(%symbol, "company facts write failed: {error}"),
        }
        tokio::time::sleep(PER_COMPANY_DELAY).await;
    }

    info!(
        trigger,
        stored,
        requested = symbols.len(),
        degraded_companies,
        carried_companies,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "company facts refreshed"
    );
    stored
}

/// 本轮**取到了什么**。取数（有网络）与合并（纯函数）分开，合并那一半才盯得住：
/// 「第一轮全成功、第二轮 SEC 挂掉，官方股本还在不在」这种问题不该只能靠上线验证。
#[derive(Default)]
struct FetchedSegments {
    /// 一个 provider key 都没有：五个 provider 段落这一轮全都没发出去。
    provider_key_missing: bool,
    profile: Option<Value>,
    income: Option<Value>,
    balance_sheet: Option<Value>,
    cash_flow: Option<Value>,
    earnings: Option<Value>,
    provider_shares: Option<f64>,
    sec: SecSegment,
}

/// SEC 那一段的四种结局。**失败**与**这家确实没有**必须分开：前者下一轮可能就好了，
/// 后者是长期状态（8/59 家如此）。混在一起就会让「SEC 挂了」和「多类别股发行人」
/// 走同一条降级路径，而这两者一个该沿用旧值、一个不该。
#[derive(Default)]
enum SecSegment {
    /// 没配可联系的 User-Agent，通道本来就是关的。
    #[default]
    Disabled,
    Cover(Box<SecCoverShares>),
    Absent(SecSharesAbsence),
    Failed,
}

/// 单家取数 + 组装。任何一段失败都只往 `degraded` 里加一条，不返回错误。
#[allow(clippy::too_many_arguments)]
async fn build_company_facts(
    state: &AppState,
    keys: &[String],
    sec: Option<&SecSharesClient>,
    symbol: &str,
    name: Option<&str>,
    provider_shares: Option<f64>,
    previous: Option<&CompanyFacts>,
    today: NaiveDate,
) -> CompanyFacts {
    let mut segments = FetchedSegments {
        provider_key_missing: keys.is_empty(),
        provider_shares,
        ..FetchedSegments::default()
    };

    if !keys.is_empty() {
        let base = fmp_base_url(&state.core.config.fmp.base_url);
        // stable 端点挂在 host 根下，不在 `/api` 下面。配置里的 base_url 是
        // `https://financialmodelingprep.com/api`，用 v3 那个 base 去拼会得到
        // `/api/stable/earnings`——一个 404，而且是**静默**的 404：整段财报节奏
        // 恒空，事件驱动那条路又把「恒空」读成「有新财报」，于是每小时全量重刷。
        let stable = stable_fmp_base_url(&state.core.config.fmp.base_url);
        let encoded = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
        let profile_url = format!("{base}/v3/profile/{encoded}");
        let income_url = format!("{base}/v3/income-statement/{encoded}?period=quarter&limit=8");
        let balance_url =
            format!("{base}/v3/balance-sheet-statement/{encoded}?period=quarter&limit=2");
        let cash_flow_url =
            format!("{base}/v3/cash-flow-statement/{encoded}?period=quarter&limit=8");
        let earnings_url = stable_earnings_url(&stable, symbol);
        // 一家之内并发，家与家之间串行：并发是为了不让 5 次往返累加成一家的延迟，
        // 串行是为了给 provider 与 SEC 留出配速。
        let (profile, income, balance_sheet, cash_flow, earnings) = tokio::join!(
            fetch_fmp(state, keys, &profile_url),
            fetch_fmp(state, keys, &income_url),
            fetch_fmp(state, keys, &balance_url),
            fetch_fmp(state, keys, &cash_flow_url),
            fetch_fmp(state, keys, &earnings_url),
        );
        segments.profile = profile;
        segments.income = income;
        segments.balance_sheet = balance_sheet;
        segments.cash_flow = cash_flow;
        segments.earnings = earnings;
    }

    segments.sec = match sec {
        None => SecSegment::Disabled,
        Some(client) => match fetch_cover_shares(client, symbol, previous).await {
            Ok(SecSharesOutcome::Cover(cover)) => SecSegment::Cover(cover),
            Ok(SecSharesOutcome::Absent(absence)) => SecSegment::Absent(absence),
            Err(error) => {
                warn!(%symbol, "company facts SEC lookup failed: {error}");
                SecSegment::Failed
            }
        },
    };

    merge_company_facts(
        symbol,
        name,
        previous,
        segments,
        today,
        &Utc::now().to_rfc3339(),
    )
}

/// 把本轮取到的东西合进上一轮那一行。
///
/// **取到才覆盖，取不到就沿用**：沿用的段连同它自己的 `provenance`（`as_of` /
/// `fetched_at`）原样留着，并记进 `carried_over`，读取方据此知道自己拿到的是哪一轮的数。
/// 一次 SEC 5xx 不该把官方股本清空——那正是这张表要根治的那一类 bug 的翻版：
/// 把上游的瞬时抖动变成我们自己的永久数据丢失。
fn merge_company_facts(
    symbol: &str,
    name: Option<&str>,
    previous: Option<&CompanyFacts>,
    segments: FetchedSegments,
    today: NaiveDate,
    fetched_at: &str,
) -> CompanyFacts {
    let mut facts = CompanyFacts::new(symbol);
    facts.last_refresh_attempt_at = Some(fetched_at.to_string());
    // 身份先从旧行起手：`adr_ratio` / `home_symbol` 只能一手确认，没有任何自动来源，
    // 每轮重建就等于每轮丢掉。profile 取到时下面会覆盖掉能自动确定的那几项。
    if let Some(previous) = previous {
        facts.identity = previous.identity.clone();
    }
    if let Some(name) = name {
        facts.identity.company_name = Some(name.to_string());
    }
    // 本轮至少有一段成功。全军覆没时不推进 refreshed_at——一行空数据自称新鲜，
    // 会让冷启动补刷跳过它、让研究台不报陈旧，等于把没数据这件事藏起来。
    let mut fetched_any = false;

    if segments.provider_key_missing {
        facts.degraded.push("provider_key_missing".to_string());
        carry_income(&mut facts, previous);
        carry_balance_sheet(&mut facts, previous);
        carry_cash_flow(&mut facts, previous);
        carry_earnings(&mut facts, previous);
    } else {
        match segments.profile.as_ref() {
            Some(value) => {
                apply_profile(&mut facts.identity, value, fetched_at);
                fetched_any = true;
            }
            None => {
                facts.degraded.push("profile".to_string());
                // identity 已经是旧行那一份了，这里只是把「你读到的是上一轮」说出来。
                if previous.is_some_and(|old| old.identity != CompanyIdentity::default()) {
                    facts.carried_over.push("profile".to_string());
                }
            }
        }
        match segments.income.as_ref() {
            Some(value) => {
                facts.income = income_facts(value, fetched_at);
                apply_weighted_shares(&mut facts.shares, value, fetched_at);
                fetched_any = true;
            }
            None => {
                facts.degraded.push("income_statement".to_string());
                carry_income(&mut facts, previous);
            }
        }
        match segments.balance_sheet.as_ref() {
            Some(value) => {
                facts.balance_sheet = balance_sheet_facts(value, fetched_at);
                fetched_any = true;
            }
            None => {
                facts.degraded.push("balance_sheet".to_string());
                carry_balance_sheet(&mut facts, previous);
            }
        }
        match segments.cash_flow.as_ref() {
            Some(value) => {
                facts.cash_flow = cash_flow_facts(value, fetched_at);
                fetched_any = true;
            }
            None => {
                facts.degraded.push("cash_flow".to_string());
                carry_cash_flow(&mut facts, previous);
            }
        }
        match segments.earnings.as_ref() {
            Some(value) => {
                facts.earnings = earnings_cadence(value, today, fetched_at);
                fetched_any = true;
            }
            None => {
                facts.degraded.push("earnings_calendar".to_string());
                carry_earnings(&mut facts, previous);
            }
        }
    }

    // 财报日历给的是**发布日**，不是财季末。财季末在同一轮的季度利润表里，
    // 那才是「最近已发布财季」这个字段要的东西。
    if facts.earnings.latest_reported_period_end.is_none() {
        facts.earnings.latest_reported_period_end = facts.income.latest_quarter_end.clone();
    }

    match segments
        .provider_shares
        .filter(|value| value.is_finite() && *value > 0.0)
    {
        Some(value) => {
            facts.shares.provider_shares = Some(value);
            fetched_any = true;
        }
        None => {
            facts.degraded.push("provider_shares".to_string());
            if let Some(old) = previous.and_then(|old| old.shares.provider_shares) {
                facts.shares.provider_shares = Some(old);
                facts.carried_over.push("provider_shares".to_string());
            }
        }
    }

    match segments.sec {
        SecSegment::Cover(cover) => {
            apply_cover_shares(&mut facts, &cover, fetched_at);
            fetched_any = true;
        }
        SecSegment::Absent(absence) => {
            // `UnknownTicker` 是**我们这边**的对照表没查到这个代码，不是 SEC 说
            // 「这家没有封面股数」。拿一次查表失误去抹掉已经存下来的官方股本，
            // 就是把上游的瞬时问题变成我们自己的永久数据丢失。
            let lookup_miss = matches!(absence, SecSharesAbsence::UnknownTicker);
            if lookup_miss && previous.is_some_and(|old| old.shares.cover_shares.is_some()) {
                facts.degraded.push("sec_cover_shares".to_string());
                carry_cover_shares(&mut facts, previous);
            } else {
                // 404 / 没有可用行是 SEC 给出的**确定**答案（8/59 家长期如此），
                // 记下原因就是本轮的结论。
                facts.shares.cover_absent_reason = Some(absence.as_str().to_string());
                facts.shares.cover_absent_note = Some(absence.note().to_string());
                fetched_any = true;
            }
        }
        SecSegment::Failed => {
            // 真失败与「这家没有」分开记：前者下一轮可能就好了，后者是长期状态，
            // 混在一起会让人以为 SEC 天天在挂。
            facts.degraded.push("sec_cover_shares".to_string());
            carry_cover_shares(&mut facts, previous);
        }
        SecSegment::Disabled => {
            facts.degraded.push("sec_channel_disabled".to_string());
            carry_cover_shares(&mut facts, previous);
        }
    }

    // 新鲜度、与 provider 的差异、距申报天数都是**读取时**的派生量：沿用上一轮的
    // 封面数时它们必须按今天重算，否则昨天算出的 `usable_for_market_cap` 会把一份
    // 越来越旧的封面一直当成可用。
    refresh_cover_derivations(&mut facts, today);
    warn_on_provider_gap(&facts, today);

    facts.refreshed_at = match (fetched_any, previous) {
        (true, _) => fetched_at.to_string(),
        // 一段都没成功。保留上一轮的刷新时刻——这一行的数据确实还是那一轮的。
        (false, Some(previous)) => previous.refreshed_at.clone(),
        (false, None) => fetched_at.to_string(),
    };
    facts
}

/// SEC 取数。先按代码走对照表；对照表挂了就用上一轮存下来的 CIK 直取。
///
/// 那张 776 KB 的对照表是整条通道的单点：它一挂，59 家全都拿不到官方股本。
/// 而 CIK 一旦被 SEC 确认过就不会变，存着它就是为了在这一天还能绕过去。
/// 只用 `shares.cover_cik`（SEC 自己返回的），不用 `identity.cik`——后者可能来自
/// provider 的 profile，拿一个错 CIK 去取股本，错法和 TSM 那个 5 倍坑同级。
async fn fetch_cover_shares(
    client: &SecSharesClient,
    symbol: &str,
    previous: Option<&CompanyFacts>,
) -> Result<SecSharesOutcome, hone_core::sec_shares::SecSharesError> {
    let first = client.cover_shares(symbol).await;
    let Err(error) = first else {
        return first;
    };
    let Some(cik) = previous.and_then(|old| old.shares.cover_cik) else {
        return Err(error);
    };
    client.cover_shares_for_cik(cik).await
}

fn carry_income(facts: &mut CompanyFacts, previous: Option<&CompanyFacts>) {
    let Some(previous) = previous else {
        return;
    };
    let mut carried = false;
    if previous.income != IncomeFacts::default() {
        facts.income = previous.income.clone();
        carried = true;
    }
    // 加权平均股本和利润表来自同一个响应，沿用也要一起沿用（连同它自己的出处）。
    if previous.shares.basic_shares.is_some() || previous.shares.diluted_shares.is_some() {
        facts.shares.basic_shares = previous.shares.basic_shares;
        facts.shares.diluted_shares = previous.shares.diluted_shares;
        facts.shares.weighted_period_end = previous.shares.weighted_period_end.clone();
        facts.shares.diluted_collapsed = previous.shares.diluted_collapsed;
        facts.shares.weighted_provenance = previous.shares.weighted_provenance.clone();
        carried = true;
    }
    if carried {
        facts.carried_over.push("income_statement".to_string());
    }
}

fn carry_balance_sheet(facts: &mut CompanyFacts, previous: Option<&CompanyFacts>) {
    if let Some(previous) = previous
        && previous.balance_sheet != BalanceSheetFacts::default()
    {
        facts.balance_sheet = previous.balance_sheet.clone();
        facts.carried_over.push("balance_sheet".to_string());
    }
}

fn carry_cash_flow(facts: &mut CompanyFacts, previous: Option<&CompanyFacts>) {
    if let Some(previous) = previous
        && previous.cash_flow != CashFlowFacts::default()
    {
        facts.cash_flow = previous.cash_flow.clone();
        facts.carried_over.push("cash_flow".to_string());
    }
}

fn carry_earnings(facts: &mut CompanyFacts, previous: Option<&CompanyFacts>) {
    if let Some(previous) = previous
        && previous.earnings != EarningsCadence::default()
    {
        facts.earnings = previous.earnings.clone();
        facts.carried_over.push("earnings_calendar".to_string());
    }
}

/// 官方封面股数这一段的沿用。**这是整张表最不能丢的一段**：它一个季度才动一次，
/// 上一轮的值几乎总比空值正确，而空值会让研究台的市值当场回落到 provider 口径。
fn carry_cover_shares(facts: &mut CompanyFacts, previous: Option<&CompanyFacts>) {
    let Some(previous) = previous else {
        return;
    };
    if previous.shares.cover_shares.is_none() {
        // 上一轮也没有官方数：把「为什么没有」原样带过来，别让读取方以为是新故障。
        facts.shares.cover_absent_reason = previous.shares.cover_absent_reason.clone();
        facts.shares.cover_absent_note = previous.shares.cover_absent_note.clone();
        facts.shares.cover_cik = previous.shares.cover_cik;
        return;
    }
    let old = &previous.shares;
    let shares = &mut facts.shares;
    shares.cover_shares = old.cover_shares;
    shares.cover_cik = old.cover_cik;
    shares.cover_end = old.cover_end.clone();
    shares.cover_filed = old.cover_filed.clone();
    shares.cover_form = old.cover_form.clone();
    shares.cover_accession = old.cover_accession.clone();
    shares.cover_basis = old.cover_basis.clone();
    shares.previous_cover_shares = old.previous_cover_shares;
    shares.previous_cover_end = old.previous_cover_end.clone();
    shares.cover_absent_reason = None;
    shares.cover_absent_note = None;
    // 出处原样保留：`as_of`（封面日）与 `fetched_at`（上一轮取到的时刻）就是
    // 「这是上一轮的数」这句话本身。
    shares.cover_provenance = old.cover_provenance.clone();
    facts.carried_over.push("sec_cover_shares".to_string());
}

fn apply_cover_shares(facts: &mut CompanyFacts, cover: &SecCoverShares, fetched_at: &str) {
    let shares = &mut facts.shares;
    shares.cover_shares = Some(cover.latest.shares);
    shares.cover_cik = Some(cover.cik);
    shares.cover_end = Some(cover.latest.end.clone());
    shares.cover_filed = Some(cover.latest.filed.clone());
    shares.cover_form = Some(cover.latest.form.clone());
    shares.cover_accession = Some(cover.latest.accn.clone());
    shares.cover_basis = Some(cover.basis().to_string());
    shares.cover_absent_reason = None;
    shares.cover_absent_note = None;
    shares.previous_cover_shares = cover.previous.as_ref().map(|previous| previous.shares);
    shares.previous_cover_end = cover.previous.as_ref().map(|previous| previous.end.clone());
    // 封面股数有自己的出处；加权平均股本来自 provider 的利润表，出处是另一份。
    // 共用一个字段就会让落库那行声称加权股本也来自 SEC——这次事故的一半正是
    // 这两个概念被混着读。
    shares.cover_provenance =
        FactProvenance::new(Some(cover.latest.end.clone()), SOURCE_SEC_COVER, fetched_at);
    if facts.identity.cik.is_none() {
        facts.identity.cik = Some(cover.cik);
    }
    if facts.identity.company_name.is_none() {
        facts.identity.company_name = cover.entity_name.clone();
    }
}

/// 封面股数的**读取时**派生量：能不能拿来算市值、距申报多少天、与 provider 差多少。
///
/// 单独拎出来，是因为这三个都随「今天是哪天」变化，而封面数本身可能是上一轮沿用的。
/// 写入时算一次就存死，会让一份越来越旧的封面一直挂着当天算出的 `usable = true`。
fn refresh_cover_derivations(facts: &mut CompanyFacts, today: NaiveDate) {
    let Some(row) = cover_row(&facts.shares) else {
        facts.shares.cover_usable_for_market_cap = false;
        facts.shares.provider_difference_pct = None;
        facts.days_since_latest_filing = None;
        return;
    };
    facts.shares.cover_usable_for_market_cap = cover_is_usable_for_market_cap(&row, today);
    facts.days_since_latest_filing = cover_age_days(&row.filed, today);

    // 与 provider 的差异只在**口径成立**时算：20-F 的本土普通股股数与美股 ADR
    // 股数差整数倍，把它写成「差 400%」会让读的人以为 provider 错了。
    let usable = facts.shares.cover_usable_for_market_cap;
    facts.shares.provider_difference_pct = facts.shares.provider_shares.and_then(|provider| {
        if !form_is_us_domestic_periodic(&row.form) || !usable {
            return None;
        }
        let relative = (row.shares as f64 - provider) / provider;
        relative
            .is_finite()
            .then(|| (relative * 10_000.0).round() / 100.0)
    });
}

/// 把存下来的封面字段还原成 `sec_shares` 那边的行结构，好让口径规则只有一份实现。
fn cover_row(shares: &ShareCounts) -> Option<hone_core::sec_shares::CoverShareRow> {
    Some(hone_core::sec_shares::CoverShareRow {
        shares: shares.cover_shares?,
        end: shares.cover_end.clone()?,
        filed: shares.cover_filed.clone().unwrap_or_default(),
        form: shares.cover_form.clone()?,
        accn: shares.cover_accession.clone().unwrap_or_default(),
    })
}

/// 超过阈值的缺口写进日志：这是 LITE 事故那一类，值得在运维侧看得见。
fn warn_on_provider_gap(facts: &CompanyFacts, today: NaiveDate) {
    let Some(row) = cover_row(&facts.shares) else {
        return;
    };
    let Some(provider) = facts.shares.provider_shares else {
        return;
    };
    if let Some(block) = build_shares_cross_check(provider, None, None, &row, today) {
        warn!(
            symbol = %facts.symbol,
            status = %block["status"],
            difference_pct = %block["difference_pct"],
            official = row.shares,
            provider,
            cover_end = %row.end,
            "company facts: provider share count disagrees with the official cover page"
        );
    }
}

fn apply_profile(identity: &mut CompanyIdentity, value: &Value, fetched_at: &str) {
    let row = first_row(value);
    identity.exchange =
        string_field(row, "exchangeShortName").or_else(|| string_field(row, "exchange"));
    identity.is_adr = row
        .get("isAdr")
        .and_then(Value::as_bool)
        .unwrap_or(identity.is_adr);
    if let Some(name) = string_field(row, "companyName") {
        identity.company_name = Some(name);
    }
    if identity.cik.is_none() {
        identity.cik = string_field(row, "cik")
            .and_then(|cik| cik.trim_start_matches('0').parse::<u64>().ok());
    }
    // `adr_ratio` / `home_symbol` 刻意不由 provider 猜：这两个数错了就是 TSM 的
    // 5 倍坑，宁可留空让读取方拒绝换算。要填只能来自一手确认。
    identity.provenance = FactProvenance::new(None, SOURCE_FMP_PROFILE, fetched_at);
}

fn apply_weighted_shares(shares: &mut ShareCounts, income: &Value, fetched_at: &str) {
    let Some(latest) = income.as_array().and_then(|rows| rows.first()) else {
        return;
    };
    let basic = number(latest, "weightedAverageShsOut");
    let diluted = number(latest, "weightedAverageShsOutDil");
    shares.basic_shares = basic;
    shares.diluted_shares = diluted;
    shares.weighted_period_end = string_field(latest, "date").map(|date| date_only(&date));
    // 稀释 ≤ 基本 **且本季亏损**：GAAP 亏损季把潜在稀释证券全部排除，稀释股本会
    // 「塌」回基本股本甚至低于上一季一大截。标出来，免得有人拿它讲股本变化——
    // LITE 事故里正是这个数被读成「转股清偿」。
    //
    // 亏损这个条件不能省：很多成熟公司压根没有潜在稀释证券，稀释常年等于基本，
    // 只看第一条会把它们每一季都标成塌陷。一个恒真的标记等于没有标记。
    let loss_quarter = number(latest, "netIncome").is_some_and(|value| value < 0.0);
    shares.diluted_collapsed = loss_quarter
        && matches!((basic, diluted), (Some(basic), Some(diluted)) if diluted <= basic);
    shares.weighted_provenance = FactProvenance::new(
        shares.weighted_period_end.clone(),
        SOURCE_FMP_INCOME,
        fetched_at,
    );
}

fn income_facts(value: &Value, fetched_at: &str) -> IncomeFacts {
    let rows = value.as_array().map(Vec::as_slice).unwrap_or_default();
    let mut facts = IncomeFacts::default();
    let Some(latest) = rows.first() else {
        return facts;
    };

    let ttm: Vec<&Value> = rows.iter().take(4).collect();
    // 不足四季就不给 TTM：三季合计叫「最近四季」是在编数字。
    if ttm.len() == 4 {
        facts.ttm_period_ends = ttm
            .iter()
            .filter_map(|row| string_field(row, "date").map(|date| date_only(&date)))
            .collect();
        facts.ttm_revenue = sum_field(&ttm, "revenue");
        facts.ttm_gross_profit = sum_field(&ttm, "grossProfit");
        facts.ttm_operating_income = sum_field(&ttm, "operatingIncome");
        facts.ttm_net_income = sum_field(&ttm, "netIncome");
        facts.ttm_gross_margin_percent = match (facts.ttm_gross_profit, facts.ttm_revenue) {
            (Some(gross), Some(revenue)) if revenue > 0.0 => Some(gross / revenue * 100.0),
            _ => None,
        };
    }

    facts.latest_quarter_end = string_field(latest, "date").map(|date| date_only(&date));
    facts.latest_quarter_label = string_field(latest, "period");
    facts.latest_quarter_revenue = number(latest, "revenue");
    facts.latest_quarter_net_income = number(latest, "netIncome");
    facts.revenue_qoq_percent = percent_change(
        facts.latest_quarter_revenue,
        rows.get(1).and_then(|row| number(row, "revenue")),
    );
    facts.revenue_yoy_percent = percent_change(
        facts.latest_quarter_revenue,
        rows.get(4).and_then(|row| number(row, "revenue")),
    );
    facts.provenance = FactProvenance::new(
        facts.latest_quarter_end.clone(),
        SOURCE_FMP_INCOME,
        fetched_at,
    );
    facts
}

fn balance_sheet_facts(value: &Value, fetched_at: &str) -> BalanceSheetFacts {
    let mut facts = BalanceSheetFacts::default();
    let Some(latest) = value.as_array().and_then(|rows| rows.first()) else {
        return facts;
    };
    facts.period_end = string_field(latest, "date").map(|date| date_only(&date));
    facts.cash_and_short_term_investments = number(latest, "cashAndShortTermInvestments")
        .or_else(|| number(latest, "cashAndCashEquivalents"));
    facts.total_debt = number(latest, "totalDebt");
    facts.capital_lease_obligations = number(latest, "capitalLeaseObligations");
    facts.net_cash = match (facts.cash_and_short_term_investments, facts.total_debt) {
        (Some(cash), Some(debt)) => Some(cash - debt),
        _ => None,
    };
    // 口径跟着数字走。不写这一行，净现金拿去和同业比就是在比两把不同的尺。
    facts.debt_basis = facts.total_debt.map(|_| {
        "provider totalDebt：含短期借款、长期借款与融资租赁负债，通常不含经营租赁负债；净现金 = 现金及短期投资 − totalDebt。".to_string()
    });
    facts.provenance =
        FactProvenance::new(facts.period_end.clone(), SOURCE_FMP_BALANCE, fetched_at);
    facts
}

fn cash_flow_facts(value: &Value, fetched_at: &str) -> CashFlowFacts {
    let rows = value.as_array().map(Vec::as_slice).unwrap_or_default();
    let mut facts = CashFlowFacts::default();
    let ttm: Vec<&Value> = rows.iter().take(4).collect();
    if ttm.len() < 4 {
        return facts;
    }
    facts.ttm_period_ends = ttm
        .iter()
        .filter_map(|row| string_field(row, "date").map(|date| date_only(&date)))
        .collect();
    facts.ttm_operating_cash_flow = sum_field(&ttm, "operatingCashFlow")
        .or_else(|| sum_field(&ttm, "netCashProvidedByOperatingActivities"));
    facts.ttm_capital_expenditure = sum_field(&ttm, "capitalExpenditure");
    facts.ttm_free_cash_flow = sum_field(&ttm, "freeCashFlow").or_else(|| {
        match (facts.ttm_operating_cash_flow, facts.ttm_capital_expenditure) {
            // provider 的 capitalExpenditure 是负数（现金流出），所以是加不是减。
            (Some(ocf), Some(capex)) => Some(ocf + capex),
            _ => None,
        }
    });
    facts.provenance = FactProvenance::new(
        facts.ttm_period_ends.first().cloned(),
        SOURCE_FMP_CASH_FLOW,
        fetched_at,
    );
    facts
}

fn earnings_cadence(value: &Value, today: NaiveDate, fetched_at: &str) -> EarningsCadence {
    let mut facts = EarningsCadence::default();
    let today = today.format("%Y-%m-%d").to_string();
    let rows = value.as_array().map(Vec::as_slice).unwrap_or_default();
    for row in rows {
        let Some(date) = string_field(row, "date").map(|date| date_only(&date)) else {
            continue;
        };
        if row_has_actuals(row) {
            if facts
                .latest_reported_date
                .as_deref()
                .is_none_or(|known| date.as_str() > known)
            {
                facts.latest_reported_date = Some(date);
            }
        } else if date >= today
            && facts
                .next_earnings_date
                .as_deref()
                .is_none_or(|known| date.as_str() < known)
        {
            facts.next_earnings_date = Some(date);
        }
    }
    facts.provenance = FactProvenance::new(
        facts.latest_reported_date.clone(),
        SOURCE_FMP_EARNINGS,
        fetched_at,
    );
    facts
}

/// 一行财报日历是不是已经带上了实际数字。
/// stable 端点写 `epsActual` / `revenueActual`，v3 日历写 `eps` / `revenue`，两种都收。
fn row_has_actuals(row: &Value) -> bool {
    ["epsActual", "revenueActual", "eps", "revenue"]
        .iter()
        .any(|key| row.get(*key).is_some_and(|value| !value.is_null()))
}

/// 最近三天的全市场财报日历。**一次**请求覆盖全部成员，成本与成员数无关。
/// 窗口取三天而不是一天：日历按发布日期排，盘后发布的公司在时区两边可能落在
/// 相邻的那一天，而多看两天只是多几行要过滤。
async fn fetch_recent_earnings_calendar(state: &AppState) -> Option<Value> {
    let pool = state.core.config.fmp.effective_key_pool();
    let keys = pool.keys();
    if keys.is_empty() {
        return None;
    }
    let today = hone_core::local_now().date_naive();
    let from = today - chrono::Duration::days(2);
    let base = fmp_base_url(&state.core.config.fmp.base_url);
    let url = format!(
        "{base}/v3/earning_calendar?from={}&to={}",
        from.format("%Y-%m-%d"),
        today.format("%Y-%m-%d")
    );
    fetch_fmp(state, keys, &url).await
}

/// 日历里每个代码「已发布」的最新日期。
pub(crate) fn reported_dates_from_calendar(calendar: &Value) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    for row in calendar.as_array().map(Vec::as_slice).unwrap_or_default() {
        if !row_has_actuals(row) {
            continue;
        }
        let Some(symbol) = string_field(row, "symbol").map(|s| s.to_ascii_uppercase()) else {
            continue;
        };
        let Some(date) = string_field(row, "date").map(|date| date_only(&date)) else {
            continue;
        };
        out.entry(symbol)
            .and_modify(|known| {
                if date > *known {
                    *known = date.clone();
                }
            })
            .or_insert(date);
    }
    out
}

/// 批量取 provider 的 `sharesOutstanding`。取不到就整批留空——它只是用来对照的，
/// 缺了不影响官方股本落库。
async fn fetch_provider_shares(
    state: &AppState,
    keys: &[String],
    symbols: &[String],
) -> HashMap<String, f64> {
    let base = fmp_base_url(&state.core.config.fmp.base_url);
    let mut out: HashMap<String, f64> = HashMap::new();
    let batches = quote_batches(symbols);
    let total = batches.len();
    let mut failed = 0usize;
    for path in batches {
        match fetch_fmp(state, keys, &format!("{base}/v3/quote/{path}")).await {
            Some(value) => out.extend(provider_shares_from_quotes(&value)),
            None => failed += 1,
        }
    }
    if failed > 0 {
        warn!(
            failed_batches = failed,
            total_batches = total,
            "company facts: some provider share-count batches came back empty"
        );
    }
    out
}

/// 批量行情的路径片段。两条约定都照仓库既有的来：
///
/// 1. **逐个代码编码，用字面逗号 join**（`data_fetch::encode_fmp_symbols` 的形状）。
///    把整串一起 percent-encode 会把分隔符编成 `%2C`，路径变成
///    `/v3/quote/AAPL%2CMSFT%2C...`——多数框架解得开，但这是在赌路由的实现细节，
///    而赌输的表现是**静默**返回空，`provider_difference_pct` 整表消失只留一行 warn。
/// 2. **分批到 25 个**（`hone-event-engine::pollers::price::FMP_QUOTE_BATCH_SIZE`）。
///    一次打 59 个是在赌 provider 不截断，而截断同样是静默的。
fn quote_batches(symbols: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for symbol in symbols {
        let symbol = symbol.trim().to_ascii_uppercase();
        if symbol.is_empty() || seen.contains(&symbol) {
            continue;
        }
        seen.push(symbol);
    }
    seen.chunks(QUOTE_BATCH_SIZE)
        .map(|batch| {
            batch
                .iter()
                .map(|symbol| utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect()
}

/// stable 端点的 URL。单独一个函数是为了让它被测试盯住：这一段拼错过一次，
/// 表现是整段财报节奏恒空 + 事件驱动路径每小时全量重刷，而**没有任何报错**。
fn stable_earnings_url(stable_base: &str, symbol: &str) -> String {
    let encoded = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    format!("{stable_base}/stable/earnings?symbol={encoded}")
}

pub(crate) fn provider_shares_from_quotes(value: &Value) -> HashMap<String, f64> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let symbol = string_field(row, "symbol")?.to_ascii_uppercase();
            let shares = number(row, "sharesOutstanding")?;
            (shares > 0.0).then_some((symbol, shares))
        })
        .collect()
}

/// 多 key 轮询。全部 key 都失败才返回 `None`；调用方一律降级，不 panic、不重试到天亮。
async fn fetch_fmp(state: &AppState, keys: &[String], url: &str) -> Option<Value> {
    let connector = if url.contains('?') { '&' } else { '?' };
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let full = format!("{url}{connector}apikey={encoded_key}");
        match fetch_fmp_json_once(&state.http_client, &full, state.core.config.fmp.timeout).await {
            Ok(value) => return Some(value),
            Err(error) => last_error = error,
        }
    }
    if !last_error.is_empty() {
        tracing::debug!("company facts provider request failed: {last_error}");
    }
    None
}

/// **v3 端点**的 base：配置里的 base_url 可能是 `.../api` 或 `.../api/v3`，
/// 统一成不带 `/v3` 的形态（拼出来是 `{host}/api/v3/...`）。
///
/// stable 端点**不要**用它——那些挂在 host 根下，用 [`stable_fmp_base_url`]。
fn fmp_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    base.strip_suffix("/v3").unwrap_or(base).to_string()
}

fn first_row(value: &Value) -> &Value {
    value
        .as_array()
        .and_then(|rows| rows.first())
        .unwrap_or(value)
}

fn string_field(row: &Value, key: &str) -> Option<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn number(row: &Value, key: &str) -> Option<f64> {
    row.get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

/// provider 的 `date` 有时带时间部分；口径日期只到天。
fn date_only(raw: &str) -> String {
    raw.chars().take(10).collect()
}

/// 四季合计。任何一季缺这个科目就整体返回 `None`——补零求和会得到一个
/// 看起来正常、实际少了一季的「TTM」。
fn sum_field(rows: &[&Value], key: &str) -> Option<f64> {
    let mut total = 0.0;
    for row in rows {
        total += number(row, key)?;
    }
    total.is_finite().then_some(total)
}

fn percent_change(current: Option<f64>, base: Option<f64>) -> Option<f64> {
    match (current, base) {
        (Some(current), Some(base)) if base.abs() > f64::EPSILON => {
            let value = (current - base) / base.abs() * 100.0;
            value.is_finite().then_some(value)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn quarter(date: &str, revenue: f64, net_income: f64) -> Value {
        json!({
            "date": date,
            "period": "Q3",
            "revenue": revenue,
            "grossProfit": revenue * 0.35,
            "operatingIncome": revenue * 0.1,
            "netIncome": net_income,
            "weightedAverageShsOut": 74_600_000.0,
            "weightedAverageShsOutDil": 74_600_000.0,
        })
    }

    /// 一轮全成功的取数结果。用来当「昨天」。
    fn a_full_round(cover_shares: i64) -> FetchedSegments {
        FetchedSegments {
            provider_key_missing: false,
            profile: Some(json!([{
                "companyName": "Lumentum Holdings Inc.",
                "exchangeShortName": "NASDAQ",
                "cik": "0001633978",
            }])),
            income: Some(json!([
                quarter("2026-06-28", 100.0, -716.0),
                quarter("2026-03-29", 90.0, 8.0),
                quarter("2025-12-28", 80.0, 6.0),
                quarter("2025-09-28", 70.0, 4.0),
            ])),
            balance_sheet: Some(json!([{
                "date": "2026-06-28",
                "cashAndShortTermInvestments": 1.0e9,
                "totalDebt": 4.0e8,
            }])),
            cash_flow: Some(json!([
                {"date": "2026-06-28", "operatingCashFlow": 100.0, "capitalExpenditure": -30.0},
                {"date": "2026-03-29", "operatingCashFlow": 90.0, "capitalExpenditure": -20.0},
                {"date": "2025-12-28", "operatingCashFlow": 80.0, "capitalExpenditure": -10.0},
                {"date": "2025-09-28", "operatingCashFlow": 70.0, "capitalExpenditure": -10.0},
            ])),
            earnings: Some(json!([
                {"date": "2026-08-17", "epsActual": 1.2},
                {"date": "2026-11-05", "epsEstimated": 1.4},
            ])),
            provider_shares: Some(77_800_000.0),
            sec: SecSegment::Cover(Box::new(SecCoverShares {
                cik: 1_633_978,
                entity_name: Some("Lumentum Holdings Inc.".to_string()),
                latest: hone_core::sec_shares::CoverShareRow {
                    shares: cover_shares,
                    end: "2026-08-14".to_string(),
                    filed: "2026-08-17".to_string(),
                    form: "10-K".to_string(),
                    accn: "z".to_string(),
                },
                previous: Some(hone_core::sec_shares::CoverShareRow {
                    shares: 77_800_000,
                    end: "2026-04-30".to_string(),
                    filed: "2026-05-06".to_string(),
                    form: "10-Q".to_string(),
                    accn: "y".to_string(),
                }),
            })),
        }
    }

    /// **这一批里最伤的那条的回归测试。**
    ///
    /// 第一轮全成功、第二轮 SEC 挂掉（provider 也一起挂）。第二轮之后官方股本
    /// 必须还在——原实现每轮从 `CompanyFacts::new` 起手再无条件整行覆盖写，
    /// 一次 5xx 就把昨天拿到的 8,970 万抹成 None，研究台的市值当场回落到
    /// provider 那个落后一整份申报的 7,780 万，LITE 事故原样复发 24 小时。
    #[test]
    fn a_failed_round_keeps_the_official_share_count_it_already_had() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");

        let day_one = merge_company_facts(
            "LITE",
            Some("Lumentum Holdings Inc."),
            None,
            a_full_round(89_700_000),
            today,
            "2026-08-30T11:00:00Z",
        );
        assert_eq!(day_one.shares.cover_shares, Some(89_700_000));
        assert_eq!(day_one.shares.previous_cover_shares, Some(77_800_000));
        assert_eq!(day_one.income.ttm_revenue, Some(340.0));
        assert!(day_one.degraded.is_empty(), "{:?}", day_one.degraded);
        assert!(day_one.carried_over.is_empty());

        // 第二轮：SEC 5xx + provider 全线超时（key 还在，但每个请求都失败）。
        let day_two = merge_company_facts(
            "LITE",
            Some("Lumentum Holdings Inc."),
            Some(&day_one),
            FetchedSegments {
                sec: SecSegment::Failed,
                ..FetchedSegments::default()
            },
            today,
            "2026-08-31T11:00:00Z",
        );

        // 官方股本还在，连同它的口径日期与出处——出处说明这是上一轮取到的。
        assert_eq!(
            day_two.shares.cover_shares,
            Some(89_700_000),
            "一次 SEC 失败不得抹掉官方股本"
        );
        assert_eq!(day_two.shares.cover_end.as_deref(), Some("2026-08-14"));
        assert_eq!(day_two.shares.cover_form.as_deref(), Some("10-K"));
        assert_eq!(day_two.shares.previous_cover_shares, Some(77_800_000));
        assert_eq!(day_two.shares.market_cap_shares(), Some(89_700_000));
        assert_eq!(
            day_two.shares.cover_provenance.fetched_at.as_deref(),
            Some("2026-08-30T11:00:00Z"),
            "沿用的值必须带着它自己那一轮的取数时刻"
        );
        assert_eq!(
            day_two.shares.cover_provenance.as_of.as_deref(),
            Some("2026-08-14")
        );
        // 其余各段同样沿用，而不是清空。
        assert_eq!(day_two.income.ttm_revenue, Some(340.0));
        assert_eq!(day_two.balance_sheet.net_cash, Some(6.0e8));
        assert_eq!(day_two.cash_flow.ttm_free_cash_flow, Some(270.0));
        assert_eq!(
            day_two.earnings.latest_reported_date.as_deref(),
            Some("2026-08-17")
        );
        assert_eq!(day_two.identity.cik, Some(1_633_978));
        assert_eq!(day_two.shares.cover_cik, Some(1_633_978));

        // 沿用要说出来，而且要和「本轮没取到」分开说。
        assert!(
            day_two
                .carried_over
                .contains(&"sec_cover_shares".to_string())
        );
        assert!(
            day_two
                .carried_over
                .contains(&"income_statement".to_string())
        );
        assert!(day_two.degraded.contains(&"sec_cover_shares".to_string()));

        // 一段都没成功 → 不推进 refreshed_at，否则这一行会拿旧数据自称新鲜。
        assert_eq!(day_two.refreshed_at, "2026-08-30T11:00:00Z");
        assert_eq!(
            day_two.last_refresh_attempt_at.as_deref(),
            Some("2026-08-31T11:00:00Z"),
            "尝试过这件事本身要记下来，事件驱动的退避靠它"
        );

        // 第三轮 SEC 恢复并报出新封面数 → 用新值，沿用标记清空。
        let day_three = merge_company_facts(
            "LITE",
            Some("Lumentum Holdings Inc."),
            Some(&day_two),
            a_full_round(90_100_000),
            today,
            "2026-09-01T11:00:00Z",
        );
        assert_eq!(day_three.shares.cover_shares, Some(90_100_000));
        assert!(day_three.carried_over.is_empty());
        assert_eq!(day_three.refreshed_at, "2026-09-01T11:00:00Z");
    }

    /// 一行从来没有过 → 才是空。空行不许自称新鲜。
    #[test]
    fn a_first_round_that_fails_outright_leaves_an_empty_row_that_admits_it() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let facts = merge_company_facts(
            "OKLO",
            None,
            None,
            FetchedSegments {
                provider_key_missing: true,
                sec: SecSegment::Disabled,
                ..FetchedSegments::default()
            },
            today,
            "2026-08-30T11:00:00Z",
        );
        assert_eq!(facts.shares.cover_shares, None);
        assert!(facts.carried_over.is_empty(), "没有上一轮可沿用");
        assert!(facts.degraded.contains(&"provider_key_missing".to_string()));
        assert!(facts.degraded.contains(&"sec_channel_disabled".to_string()));
        assert!(facts.is_empty());
        assert!(
            facts.is_stale(Utc::now()),
            "空行必须自认陈旧，否则冷启动补刷会跳过它"
        );
    }

    /// SEC 说「这家没有封面股数」是一个**确定**答案（8/59 家长期如此），
    /// 与「SEC 挂了」不同：确定答案该覆盖，查表失误不该。
    #[test]
    fn a_definitive_404_overwrites_but_a_ticker_lookup_miss_does_not() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let day_one = merge_company_facts(
            "LITE",
            None,
            None,
            a_full_round(89_700_000),
            today,
            "2026-08-30T11:00:00Z",
        );

        // 对照表里查不到这个代码 —— 那是我们这边的问题，不是 SEC 说没有。
        let miss = merge_company_facts(
            "LITE",
            None,
            Some(&day_one),
            FetchedSegments {
                sec: SecSegment::Absent(SecSharesAbsence::UnknownTicker),
                ..FetchedSegments::default()
            },
            today,
            "2026-08-31T11:00:00Z",
        );
        assert_eq!(miss.shares.cover_shares, Some(89_700_000));
        assert!(miss.carried_over.contains(&"sec_cover_shares".to_string()));

        // concept 端点 404：多类别股发行人的长期状态，这就是本轮的结论。
        let absent = merge_company_facts(
            "LITE",
            None,
            Some(&day_one),
            FetchedSegments {
                sec: SecSegment::Absent(SecSharesAbsence::ConceptNotFound),
                ..FetchedSegments::default()
            },
            today,
            "2026-08-31T11:00:00Z",
        );
        assert_eq!(absent.shares.cover_shares, None);
        assert_eq!(
            absent.shares.cover_absent_reason.as_deref(),
            Some("concept_not_found")
        );
    }

    /// 沿用的封面数会**变旧**。新鲜度是读取时的判断，不能沿用写入那天算出的布尔值。
    #[test]
    fn a_carried_cover_is_re_aged_against_today_not_the_day_it_was_fetched() {
        let fetch_day = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let day_one = merge_company_facts(
            "LITE",
            None,
            None,
            a_full_round(89_700_000),
            fetch_day,
            "2026-08-30T11:00:00Z",
        );
        assert!(day_one.shares.cover_usable_for_market_cap);

        // 一年后还在沿用同一份封面（SEC 一直挂着）：它已经超过 200 天，
        // 不能再拿去乘股价算市值。
        let much_later = NaiveDate::from_ymd_opt(2027, 8, 30).expect("date");
        let carried = merge_company_facts(
            "LITE",
            None,
            Some(&day_one),
            FetchedSegments {
                sec: SecSegment::Failed,
                ..FetchedSegments::default()
            },
            much_later,
            "2027-08-30T11:00:00Z",
        );
        assert_eq!(carried.shares.cover_shares, Some(89_700_000));
        assert!(
            !carried.shares.cover_usable_for_market_cap,
            "过期的封面不许还挂着「可以算市值」"
        );
        assert_eq!(carried.shares.market_cap_shares(), None);
        assert_eq!(carried.shares.provider_difference_pct, None);
    }

    /// **URL 拼装的回归测试。** stable 端点挂在 host 根下，不在 `/api` 下面。
    /// 拼错的表现是一个静默 404：整段财报节奏恒空，而且事件驱动那条路会把
    /// 「恒空」读成「有新财报」，每小时把成员全量重刷一遍。
    #[test]
    fn the_stable_endpoint_hangs_off_the_host_not_under_api() {
        for configured in [
            "https://financialmodelingprep.com/api",
            "https://financialmodelingprep.com/api/",
            "https://financialmodelingprep.com/api/v3",
        ] {
            let stable = stable_fmp_base_url(configured);
            assert_eq!(
                stable_earnings_url(&stable, "LITE"),
                "https://financialmodelingprep.com/stable/earnings?symbol=LITE",
                "base_url = {configured}"
            );
        }

        // v3 端点走另一个 base，两者不能混用。
        let base = fmp_base_url("https://financialmodelingprep.com/api");
        assert_eq!(
            format!("{base}/v3/profile/LITE"),
            "https://financialmodelingprep.com/api/v3/profile/LITE"
        );
        assert_eq!(
            fmp_base_url("https://financialmodelingprep.com/api/v3"),
            "https://financialmodelingprep.com/api"
        );
    }

    #[test]
    fn the_batch_quote_path_uses_literal_commas_and_stays_under_the_batch_cap() {
        let symbols: Vec<String> = (0..59).map(|index| format!("SYM{index}")).collect();
        let batches = quote_batches(&symbols);
        assert_eq!(batches.len(), 3, "59 个代码按 25 分批");
        for batch in &batches {
            assert!(
                !batch.contains("%2C"),
                "分隔符必须是字面逗号，不是 %2C: {batch}"
            );
            assert!(batch.split(',').count() <= QUOTE_BATCH_SIZE);
        }
        assert!(batches[0].starts_with("SYM0,SYM1,"));

        // 代码本身还是要逐个编码；重复与空白不产生第二个条目。
        let batches = quote_batches(&[
            "brk-b".to_string(),
            " BRK-B ".to_string(),
            "BF.B".to_string(),
        ]);
        assert_eq!(batches, vec!["BRK%2DB,BF%2EB".to_string()]);
    }

    #[test]
    fn the_earnings_watch_backs_off_instead_of_refreshing_the_same_name_every_hour() {
        let now = Utc::now();
        let all: Vec<String> = ["LITE", "NVDA", "AMD", "CEG"]
            .iter()
            .map(|symbol| symbol.to_string())
            .collect();
        let reported: HashMap<String, String> = [
            ("LITE", "2026-08-30"),
            ("NVDA", "2026-08-30"),
            ("AMD", "2026-08-30"),
        ]
        .iter()
        .map(|(symbol, date)| (symbol.to_string(), date.to_string()))
        .collect();

        let mut stored: HashMap<String, CompanyFacts> = HashMap::new();

        // LITE：日历上的发布日比我们记下的新 → 该刷。
        let mut lite = CompanyFacts::new("LITE");
        lite.shares.cover_shares = Some(89_700_000);
        lite.earnings.latest_reported_date = Some("2026-05-06".to_string());
        lite.last_refresh_attempt_at = Some((now - chrono::Duration::hours(20)).to_rfc3339());
        stored.insert("LITE".to_string(), lite);

        // NVDA：一小时前刚试过 → 冷却期内不再刷。
        let mut nvda = CompanyFacts::new("NVDA");
        nvda.shares.cover_shares = Some(24_000_000_000);
        nvda.earnings.latest_reported_date = Some("2026-05-06".to_string());
        nvda.last_refresh_attempt_at = Some((now - chrono::Duration::hours(1)).to_rfc3339());
        stored.insert("NVDA".to_string(), nvda);

        // AMD：earnings 段一直取不到，latest_reported_date 为 None。这是失败态，
        // 不是「有新财报」——只有这一行本来就陈旧时才刷。
        let mut amd = CompanyFacts::new("AMD");
        amd.shares.cover_shares = Some(1_600_000_000);
        amd.refreshed_at = now.to_rfc3339();
        amd.last_refresh_attempt_at = Some((now - chrono::Duration::hours(20)).to_rfc3339());
        stored.insert("AMD".to_string(), amd);

        let due = earnings_due_symbols(&all, &reported, &stored, now);
        assert_eq!(due, vec!["LITE".to_string()]);

        // 同一个 AMD，如果这一行本来就陈旧（连着两天没刷到），那就该刷。
        let mut stale_amd = stored["AMD"].clone();
        stale_amd.refreshed_at = (now - chrono::Duration::hours(48)).to_rfc3339();
        stored.insert("AMD".to_string(), stale_amd);
        let due = earnings_due_symbols(&all, &reported, &stored, now);
        assert_eq!(due, vec!["LITE".to_string(), "AMD".to_string()]);

        // CEG 不在日历里 —— 永远不该被事件驱动这条路拉进来。
        assert!(!due.contains(&"CEG".to_string()));
    }

    #[test]
    fn the_refresh_slot_sits_ahead_of_every_consumer_of_this_table() {
        // 19:10 周报 / 19:20 估值实验室 / 19:30 公司评级 / 19:50 影响者 /
        // 19:55 关键事件链 / 20:00 每日信号与持仓新闻 —— 公司事实是它们的输入。
        assert_eq!((REFRESH_HOUR, REFRESH_MINUTE), (19, 0));
        let now = Utc::now();
        let next = hone_core::local_time_at(next_refresh(now));
        assert_eq!(
            (
                chrono::Timelike::hour(&next),
                chrono::Timelike::minute(&next)
            ),
            (19, 0)
        );
    }

    #[test]
    fn the_tracked_list_is_the_industry_tree_not_the_research_cards() {
        // 行业树 59 家；研究卡只有 52 家且与树只交 35 家。名单错了，
        // NVDA / ASML / CEG 这些树里有卡里没有的公司又会一条数据都没有。
        let members = tracked_members(std::path::Path::new("/nonexistent-data-root"));
        assert!(
            members.len() >= 50,
            "行业树成员数看起来不对: {}",
            members.len()
        );
        let symbols: Vec<&str> = members.iter().map(|(symbol, _)| symbol.as_str()).collect();
        for expected in ["NVDA", "ASML", "TSM", "LITE"] {
            assert!(symbols.contains(&expected), "行业树里应当有 {expected}");
        }
        let mut sorted = symbols.clone();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "成员代码去重后不应变少");
    }

    #[test]
    fn ttm_needs_four_quarters_and_refuses_to_pad_a_missing_one() {
        let three = json!([
            quarter("2026-06-28", 100.0, 10.0),
            quarter("2026-03-29", 90.0, 8.0),
            quarter("2025-12-28", 80.0, 6.0),
        ]);
        let facts = income_facts(&three, "2026-08-30T11:00:00Z");
        assert_eq!(facts.ttm_revenue, None, "三季不是 TTM");
        // 最近一季与环比仍然给得出来。
        assert_eq!(facts.latest_quarter_revenue, Some(100.0));
        assert!((facts.revenue_qoq_percent.expect("qoq") - 11.111).abs() < 0.01);
        assert_eq!(facts.revenue_yoy_percent, None);

        let five = json!([
            quarter("2026-06-28", 100.0, 10.0),
            quarter("2026-03-29", 90.0, 8.0),
            quarter("2025-12-28", 80.0, 6.0),
            quarter("2025-09-28", 70.0, 4.0),
            quarter("2025-06-29", 50.0, 2.0),
        ]);
        let facts = income_facts(&five, "2026-08-30T11:00:00Z");
        assert_eq!(facts.ttm_revenue, Some(340.0));
        assert_eq!(facts.ttm_period_ends.len(), 4);
        assert_eq!(facts.ttm_period_ends[0], "2026-06-28");
        assert_eq!(facts.revenue_yoy_percent, Some(100.0));
        assert!((facts.ttm_gross_margin_percent.expect("margin") - 35.0).abs() < 1e-6);

        // 某一季缺科目 → 整个 TTM 是 None，不补零。
        let mut holed = five.clone();
        holed[2]["revenue"] = Value::Null;
        assert_eq!(
            income_facts(&holed, "2026-08-30T11:00:00Z").ttm_revenue,
            None
        );
    }

    #[test]
    fn a_loss_quarter_collapse_of_the_diluted_count_is_flagged() {
        // LITE Q4 FY2026：净利 −71.6 亿，GAAP 下潜在稀释证券全部被排除，
        // 稀释股本从上一季的 96.2M 塌回 74.6M。不标出来就会被读成「转股清偿」。
        let mut shares = ShareCounts::default();
        let income = json!([{
            "date": "2026-06-28",
            "netIncome": -7_160_000_000.0_f64,
            "weightedAverageShsOut": 74_600_000.0,
            "weightedAverageShsOutDil": 74_600_000.0,
        }]);
        apply_weighted_shares(&mut shares, &income, "2026-08-30T11:00:00Z");
        assert!(shares.diluted_collapsed);
        assert_eq!(shares.basic_shares, Some(74_600_000.0));
        assert_eq!(shares.weighted_period_end.as_deref(), Some("2026-06-28"));
        // 加权股本有自己的出处，不许被随后的 SEC 段落改写成 SEC 的来源。
        assert_eq!(
            shares.weighted_provenance.source.as_deref(),
            Some(SOURCE_FMP_INCOME)
        );
        assert_eq!(
            shares.weighted_provenance.as_of.as_deref(),
            Some("2026-06-28")
        );

        let mut normal = ShareCounts::default();
        let income = json!([{
            "date": "2026-03-29",
            "netIncome": 5_000_000.0,
            "weightedAverageShsOut": 91_000_000.0,
            "weightedAverageShsOutDil": 96_200_000.0,
        }]);
        apply_weighted_shares(&mut normal, &income, "2026-08-30T11:00:00Z");
        assert!(!normal.diluted_collapsed);

        // 盈利季里「稀释 == 基本」是常态（压根没有潜在稀释证券），不是塌陷。
        // 只看「稀释 ≤ 基本」会把这一类每季都标一次，标记就成了背景噪音。
        let mut no_dilutives = ShareCounts::default();
        let income = json!([{
            "date": "2026-03-29",
            "netIncome": 5_000_000.0,
            "weightedAverageShsOut": 91_000_000.0,
            "weightedAverageShsOutDil": 91_000_000.0,
        }]);
        apply_weighted_shares(&mut no_dilutives, &income, "2026-08-30T11:00:00Z");
        assert!(
            !no_dilutives.diluted_collapsed,
            "盈利季的稀释=基本不是亏损季塌陷"
        );
    }

    #[test]
    fn a_20f_cover_never_produces_a_provider_difference() {
        // TSM：官方 259.3 亿是台股口径，provider 报的是 ADR 股数。
        // 写出「差 400%」比不写更危险。
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let mut facts = CompanyFacts::new("TSM");
        facts.shares.provider_shares = Some(5_186_504_904.0);
        let cover = SecCoverShares {
            cik: 1_046_179,
            entity_name: Some("TSMC".to_string()),
            latest: hone_core::sec_shares::CoverShareRow {
                shares: 25_932_524_521,
                end: "2025-12-31".to_string(),
                filed: "2026-04-15".to_string(),
                form: "20-F".to_string(),
                accn: "z".to_string(),
            },
            previous: None,
        };
        apply_cover_shares(&mut facts, &cover, "2026-08-30T11:00:00Z");
        refresh_cover_derivations(&mut facts, today);
        assert_eq!(facts.shares.cover_shares, Some(25_932_524_521));
        assert_eq!(
            facts.shares.cover_basis.as_deref(),
            Some("foreign_or_non_periodic_filing")
        );
        assert!(!facts.shares.cover_usable_for_market_cap);
        assert_eq!(facts.shares.market_cap_shares(), None);
        assert_eq!(facts.shares.provider_difference_pct, None);
    }

    #[test]
    fn the_lite_gap_is_recorded_as_a_percentage_against_the_provider() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let mut facts = CompanyFacts::new("LITE");
        facts.shares.provider_shares = Some(77_800_000.0);
        let cover = SecCoverShares {
            cik: 1_633_978,
            entity_name: Some("Lumentum Holdings Inc.".to_string()),
            latest: hone_core::sec_shares::CoverShareRow {
                shares: 89_700_000,
                end: "2026-08-14".to_string(),
                filed: "2026-08-17".to_string(),
                form: "10-K".to_string(),
                accn: "z".to_string(),
            },
            previous: Some(hone_core::sec_shares::CoverShareRow {
                shares: 77_800_000,
                end: "2026-04-30".to_string(),
                filed: "2026-05-06".to_string(),
                form: "10-Q".to_string(),
                accn: "y".to_string(),
            }),
        };
        apply_cover_shares(&mut facts, &cover, "2026-08-30T11:00:00Z");
        refresh_cover_derivations(&mut facts, today);
        assert_eq!(facts.shares.market_cap_shares(), Some(89_700_000));
        assert_eq!(facts.shares.provider_difference_pct, Some(15.3));
        assert_eq!(facts.shares.previous_cover_shares, Some(77_800_000));
        assert_eq!(facts.identity.cik, Some(1_633_978));
        assert_eq!(facts.shares.cover_cik, Some(1_633_978));
        // 距申报天数不再是一个声明了却没人填的字段。
        assert_eq!(facts.days_since_latest_filing, Some(13));
        assert_eq!(
            facts.shares.cover_provenance.source.as_deref(),
            Some(SOURCE_SEC_COVER)
        );
        assert_eq!(
            facts.shares.cover_provenance.as_of.as_deref(),
            Some("2026-08-14")
        );
    }

    #[test]
    fn only_rows_with_actuals_count_as_a_published_report() {
        let calendar = json!([
            {"symbol": "lite", "date": "2026-08-17", "epsActual": 1.2},
            {"symbol": "LITE", "date": "2026-08-18", "epsActual": null, "revenueActual": null},
            {"symbol": "NVDA", "date": "2026-08-27", "eps": 1.05, "revenue": 5.0e10},
            {"symbol": "AMD", "date": "2026-08-29", "epsEstimated": 0.9},
            {"date": "2026-08-29", "epsActual": 1.0}
        ]);
        let reported = reported_dates_from_calendar(&calendar);
        assert_eq!(reported.get("LITE").map(String::as_str), Some("2026-08-17"));
        assert_eq!(reported.get("NVDA").map(String::as_str), Some("2026-08-27"));
        assert!(!reported.contains_key("AMD"), "只有预期没有实际不算发布");
        assert_eq!(reported.len(), 2, "没有代码的行必须被丢掉");
    }

    #[test]
    fn provider_share_counts_come_out_of_a_batch_quote_and_ignore_price() {
        let quotes = json!([
            {"symbol": "LITE", "price": 895.0, "marketCap": 6.96e10, "sharesOutstanding": 77_800_000.0},
            {"symbol": "NVDA", "price": 200.0},
            {"symbol": "ZERO", "sharesOutstanding": 0.0}
        ]);
        let shares = provider_shares_from_quotes(&quotes);
        assert_eq!(shares.get("LITE").copied(), Some(77_800_000.0));
        assert_eq!(
            shares.len(),
            1,
            "没有股本的行不进表；price/marketCap 不落库"
        );
    }

    #[test]
    fn the_balance_sheet_always_ships_its_debt_basis() {
        let balance = json!([{
            "date": "2026-06-28",
            "cashAndShortTermInvestments": 1.0e9,
            "totalDebt": 4.0e8,
            "capitalLeaseObligations": 5.0e7,
        }]);
        let facts = balance_sheet_facts(&balance, "2026-08-30T11:00:00Z");
        assert_eq!(facts.net_cash, Some(6.0e8));
        assert!(facts.debt_basis.expect("basis").contains("经营租赁"));
        assert_eq!(facts.period_end.as_deref(), Some("2026-06-28"));

        // 缺 totalDebt 时不许把净现金当成现金。
        let partial = json!([{"date": "2026-06-28", "cashAndShortTermInvestments": 1.0e9}]);
        let facts = balance_sheet_facts(&partial, "2026-08-30T11:00:00Z");
        assert_eq!(facts.net_cash, None);
        assert_eq!(facts.debt_basis, None);
    }

    #[test]
    fn free_cash_flow_falls_back_to_ocf_plus_a_negative_capex() {
        let rows = json!([
            {"date": "2026-06-28", "operatingCashFlow": 100.0, "capitalExpenditure": -30.0},
            {"date": "2026-03-29", "operatingCashFlow": 90.0, "capitalExpenditure": -20.0},
            {"date": "2025-12-28", "operatingCashFlow": 80.0, "capitalExpenditure": -10.0},
            {"date": "2025-09-28", "operatingCashFlow": 70.0, "capitalExpenditure": -10.0},
        ]);
        let facts = cash_flow_facts(&rows, "2026-08-30T11:00:00Z");
        assert_eq!(facts.ttm_operating_cash_flow, Some(340.0));
        assert_eq!(facts.ttm_capital_expenditure, Some(-70.0));
        assert_eq!(facts.ttm_free_cash_flow, Some(270.0));
    }

    #[test]
    fn earnings_cadence_separates_the_last_report_from_the_next_one() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 30).expect("date");
        let earnings = json!([
            {"date": "2026-08-17", "epsActual": 1.2},
            {"date": "2026-05-06", "epsActual": 0.9},
            {"date": "2026-11-05", "epsEstimated": 1.4},
            {"date": "2027-02-04", "epsEstimated": 1.5}
        ]);
        let facts = earnings_cadence(&earnings, today, "2026-08-30T11:00:00Z");
        assert_eq!(facts.latest_reported_date.as_deref(), Some("2026-08-17"));
        assert_eq!(facts.next_earnings_date.as_deref(), Some("2026-11-05"));
        assert_eq!(facts.provenance.as_of.as_deref(), Some("2026-08-17"));
    }
}
