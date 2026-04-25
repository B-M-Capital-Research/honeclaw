# Bug: event-engine high news events had no sink delivery evidence

状态：`Fixed`

修复进展:2026-04-26 已修复。trusted-source 高优新闻即使没匹配到 portfolio ticker(即原始 router 路径会落 `router|no_actor`),现在通过新建的 `crates/hone-event-engine/src/global_digest/` 管道处理:collector 拉所有 trusted source(FMP + RSS Bloomberg/SpaceNews/STAT)的 High/Medium news 进候选池 → curator Pass 1 (nova-lite-v1) 批量打分聚类 → fetcher 抓原文(UA + google referer 绕反爬) → curator Pass 2 (grok-4.1-fast) 精读 + 写短评 → 每个 direct + global_digest_enabled 的 actor 单独跑 personalize(用其 thesis 重排 + macro_floor 兜底)→ broadcast。配置 `event_engine.global_digest.enabled=true` + 填 `schedules: ["HH:MM",...]` 即启用。POC 验证一天 2 次 cost ≈ $0.012/天/全用户。原始 `router|no_actor` skip 路径保留(仍是 LLM 仲裁不感兴趣的事件的归宿),但现在不再静默 —— 只是不再走即时 sink。

## Summary

Trusted-source `severity=high` stock news can still fall through to `router/no_actor` without any sink delivery, even when the active direct actor has `portfolio_only=false` and other High events in the same window deliver normally.

## Observed Symptoms

- `data/events.sqlite3` recorded three high news events created after `2026-04-22T06:09:38Z`:

```text
2026-04-22 07:37:26|fmp.stock_news:reuters.com|high|news:https://www.reuters.com/legal/litigation/ford-recall-over-140000-us-vehicles-over-damaged-wires-2026-04-22/|Ford to recall over 140,000 US vehicles over damaged wires
2026-04-22 08:40:54|fmp.stock_news:reuters.com|high|news:https://www.reuters.com/legal/transactional/deutsche-telekom-shares-slip-after-t-mobile-merger-talks-reports-2026-04-22/|Deutsche Telekom shares slip after T-Mobile merger talks reports
2026-04-22 09:40:54|fmp.stock_news:wsj.com|high|news:https://www.wsj.com/business/earnings/tui-cuts-guidance-amid-uncertainty-over-u-s-iran-war-e9417edd|TUI Cuts Guidance Amid Uncertainty Over U.S.-Iran War
```

- `data/runtime/logs/web.log` / `data/runtime/logs/web.log.2026-04-22` had no `sink delivered` lines for the incremental window. `data/events.sqlite3` also had no `delivery_log` rows with `channel='sink'` and `sent_at_ts>=1776838178`.
- The active backend did assemble a real sink before the affected events:

```text
data/runtime/logs/web.log:2207:[2026-04-22 14:37:25.025] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-22:239:[2026-04-22 17:55:52.293] INFO  event engine sink: MultiChannelSink 已装配
```

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/router.rs:323` resolves actor subscriptions before it can send high events. If `registry.resolve(event)` returns no hits, a high event can be stored with no delivery log and no explicit skip reason.

```rust
pub async fn dispatch(&self, event: &MarketEvent) -> anyhow::Result<(u32, u32)> {
    let tag = kind_tag(&event.kind);
    if self.disabled_kinds.contains(tag) {
        tracing::info!(
            event_id = %event.id,
            kind = %tag,
            "event kind globally disabled; dispatch skipped"
        );
        return Ok((0, 0));
    }
    let upgraded = self.maybe_upgrade_news(event);
    let event = &upgraded;
    // 每次 dispatch 都拿最新快照——用户持仓更新后下一条事件即可感知。
    let hits = self.registry.load().resolve(event);
    let mut sent = 0u32;
    let mut pending = 0u32;
    for (actor, sev) in hits {
```

`crates/hone-event-engine/src/router.rs:440` only records successful high sends inside the hit loop. With zero hits, the function can return `(0, 0)` without durable evidence that a high event was intentionally unmatched.

```rust
match effective_sev {
    Severity::High => {
        let default_body = renderer::render_immediate(event, self.sink.format());
        let body = match self.polisher.polish(event, &default_body).await {
            Some(polished) => polished,
            None => default_body,
        };
        if let Err(e) = self.sink.send(&actor, &body).await {
            tracing::warn!(
                actor = %actor_key(&actor),
                event_id = %event.id,
                kind = %kind_tag(&event.kind),
                body_len = body.chars().count(),
                body_preview = %body_preview(&body),
                "sink send failed: {e:#}"
            );
```

The subscription registry currently broadcasts only `social_post` globally. Non-social high news still depends on portfolio symbol matching, so high market news can have no delivery path if the symbol is absent from direct holdings or represented under another ticker.

```rust
/// - 每个有持仓的 direct actor → `PortfolioSubscription`（按 ticker 命中）
/// - 所有 direct actor 汇总后 → 一个 `GlobalSubscription`(kinds=[`social_post`])
///   用于把 Telegram / Truth Social 等"无 ticker"社交事件广播给所有 actor,
///   让 router 有机会调 LLM 仲裁。未来若加 macro 全员播报,在 kinds 里追加即可。
pub fn registry_from_portfolios(storage: &PortfolioStorage) -> SubscriptionRegistry {
    let mut reg = SubscriptionRegistry::new();
    let mut direct_actors: Vec<ActorIdentity> = Vec::new();
    for (actor, portfolio) in storage.list_all() {
```

## Evidence Gap

- Need structured dispatch metrics for `hits=0` high events, including event id, symbols, and loaded subscription count.
- Need a snapshot of active direct portfolios/subscriptions to determine whether `F`, `DTEGY`, or `TUIFF` should have matched a user, alias, ADR, or ETF exposure.
- This巡检 did not call any real channel API, so it cannot prove user non-delivery; it can only show absence of local sink success evidence.

## Latest巡检 Update

- 2026-04-22T14:14:09Z: the same pattern recurred after the previous巡检 window. `data/events.sqlite3` stored a new `severity=high` WSJ stock news event with no matching `delivery_log` row:

```text
created=2026-04-22 13:52:42
occurred=2026-04-22 09:34:00
source=fmp.stock_news:wsj.com
severity=high
id=news:https://www.wsj.com/business/telecom/deutsche-telekom-shares-fall-on-reports-of-potential-merger-with-t-mobile-us-1ed8e3ba
title=Deutsche Telekom Shares Fall on Reports of Potential Merger With T-Mobile US
symbols=["DTEGY"]
delivery_rows=0
```

- The same incremental window did record successful sink sends for other High events, so this is not a global sink outage:

```text
2026-04-22 10:30:10|sec:GEV:https://www.sec.gov/Archives/edgar/data/1996810/000199681026000063/gev-20260422.htm|sink|high|sent
2026-04-22 12:37:47|earnings_surprise:GEV:2026-04-22|sink|high|sent
2026-04-22 13:32:43|price:BE:2026-04-22|sink|high|sent
2026-04-22 13:32:43|price:GEV:2026-04-22|digest|high|cooled_down
```

- Local logs also show real sink assembly and successful delivery in the same runtime:

```text
data/runtime/logs/web.log.2026-04-22:239:[2026-04-22 17:55:52.293] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-22:339:[2026-04-22 18:30:10.570] INFO  sink delivered
data/runtime/logs/web.log.2026-04-22:563:[2026-04-22 20:37:47.227] INFO  sink delivered
data/runtime/logs/web.log.2026-04-22:623:[2026-04-22 21:32:43.777] INFO  sink delivered
```

- 2026-04-22T18:13:04Z: the pattern recurred again in the next incremental window. `data/events.sqlite3` stored two new `severity=high` Reuters stock-news events created after `2026-04-22T14:12:03Z`, both with `delivery_rows=0`:

```text
created=2026-04-22 16:37:42
occurred=2026-04-22 12:15:29
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/business/united-airlines-ceo-plays-down-merger-talk-white-house-signals-skepticism-2026-04-22/
title=United Airlines CEO plays down merger talk as White House signals skepticism
symbols=["UAL"]
delivery_rows=0

created=2026-04-22 18:07:42
occurred=2026-04-22 13:44:35
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/legal/litigation/how-deutsche-telecom-t-mobile-us-could-pull-off-worlds-biggest-ma-deal-2026-04-22/
title=Explainer: How Deutsche Telecom and T-Mobile US could pull off the world's biggest M&A deal
symbols=["TMUS"]
delivery_rows=0
```

- The same window had `delivery_log` rows only for digest/prefs outcomes and no `sink` channel rows:

```text
high|118
low|419
medium|16471
delivery|filtered|low|prefs|3
delivery|queued|low|digest|6
delivery|queued|medium|digest|16
```

- The local `web.log` interval after `2026-04-22 22:12:03` had no `sink delivered`, `sink send failed`, or `[dryrun sink]` lines, while `config.yaml` and `data/runtime/effective-config.yaml` both had `event_engine.dryrun=false`.

- 2026-04-22T22:16:25Z: the pattern recurred in the next incremental window after `2026-04-22T18:12:34Z`. `data/events.sqlite3` stored one new `severity=high` MarketWatch stock-news event with no matching `delivery_log` row:

```text
created=2026-04-22 18:52:42 UTC
occurred=2026-04-22 14:32:00 UTC
source=fmp.stock_news:marketwatch.com
severity=high
id=news:https://www.marketwatch.com/story/spirit-airlines-may-get-a-bailout-whats-in-it-for-u-s-taxpayers-39099d91
title=Spirit Airlines may get a bailout. What's in it for U.S. taxpayers?
symbols=["FLYYQ"]
delivery_rows=0
```

- The same incremental log window had no `sink delivered`, `sink send failed`, or `[dryrun sink]` lines. It did record `digest queued` rows and one `digest/sent/high` delivery row for a digest batch, but no `channel='sink' and status='sent'` row after `2026-04-22T18:12:34Z`.

## Latest巡检 Update

- 2026-04-23T06:18:33Z: the pattern recurred after `2026-04-23T02:16:05Z`. `data/events.sqlite3` stored one new `severity=high` Reuters stock-news event, and the only delivery row was `router/no_actor`; there was no `sink` send row:

```text
created=2026-04-23 05:43:39 UTC
occurred=2026-04-23 01:20:46 UTC
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/business/autos-transportation/hyundai-motor-reports-31-drop-q1-operating-profit-meets-forecasts-2026-04-23/
title=Hyundai Motor reports 31% drop in Q1 operating profit, meets forecasts
symbols=["HYMTF"]
delivery=router|no_actor|high|2026-04-23 05:43:39
```

- The new `router/no_actor` row comes from `crates/hone-event-engine/src/router.rs:545-563`, which is better evidence than the earlier zero-row cases but still means a High news event can be silently skipped when no portfolio/subscription matches:

```rust
let hits = self.registry.load().resolve(event);
if hits.is_empty() {
    let _ = self.store.log_delivery(
        &event.id,
        "event_engine::::no_actor",
        "router",
        event.severity,
        "no_actor",
        None,
    );
    info!(
        event_id = %event.id,
        kind = %kind_tag(&event.kind),
        source = %event.source,
        symbols = ?event.symbols,
        "dispatch skipped: no matching actor"
    );
    return Ok((0, 0));
}
```

- The same delivery window had no `sink|sent|high` rows after the cutoff; it had `digest` rows and one `router|no_actor|high` row. This keeps the issue scoped to subscription/routing coverage rather than a global sink assembly failure.

## Latest巡检 Update

- 2026-04-24T14:25:14Z: after `2026-04-24T10:23:05Z`, `data/events.sqlite3` stored three new High events. The AMD price alert still delivered immediately, but two trusted-source High news rows again ended at `router|no_actor` with no sink send evidence:

```text
2026-04-24 13:41:19|fmp.quote|high|price:AMD:2026-04-24|AMD +13.55%
2026-04-24 14:05:17|fmp.stock_news:reuters.com|high|news:https://www.reuters.com/world/union-says-federal-bailout-spirit-airlines-must-protect-employees-2026-04-24/|Union says federal bailout of Spirit Airlines must protect employees
2026-04-24 14:20:15|fmp.stock_news:cnbc.com|high|news:https://www.cnbc.com/2026/04/24/musk-v-altman-trial-openai-lawsuit-xai.html|Musk v. Altman heads to court next week. Here's what's at stake
```

```text
price:AMD:2026-04-24|2026-04-24 13:41:21|telegram::::8039067465|sink|high|sent
news:https://www.reuters.com/world/union-says-federal-bailout-spirit-airlines-must-protect-employees-2026-04-24/|2026-04-24 14:05:17|event_engine::::no_actor|router|high|no_actor
news:https://www.cnbc.com/2026/04/24/musk-v-altman-trial-openai-lawsuit-xai.html|2026-04-24 14:20:15|event_engine::::no_actor|router|high|no_actor
```

- The active actor prefs still allow non-portfolio-only delivery:

```text
data/notif_prefs/telegram__direct__8039067465.json:1-27
  "enabled": true
  "portfolio_only": false
  "min_severity": "low"
```

- The same actor portfolio snapshot only tracks `AMD` as an extra watch symbol and has no `TSLA` / `FLYYQ` entry:

```text
data/portfolio/portfolio_telegram__direct__8039067465.json:177-189
  "symbol": "AMD",
  "notes": "用户要求关注 AMD；作为关注列表标的，不计入真实持仓资金统计。",
  "tracking_only": true
```

- Current code still explains why `portfolio_only=false` cannot rescue those High news rows. `registry_from_portfolios` only adds global fanout for `social_post` and `macro_event`, so `news_critical` must hit a portfolio symbol before prefs are even consulted:

```rust
// crates/hone-event-engine/src/subscription.rs:288-325
/// - 每个有持仓的 direct actor → `PortfolioSubscription`（按 ticker 命中）
/// - 所有 direct actor 汇总后 → 一个 `GlobalSubscription`(kinds=[`social_post`,
/// `macro_event`])。社交事件默认进 digest/LLM 仲裁；宏观事件经 router 的
///   due-window 保护后，远期日历只进摘要，临近 high 才即时播报。
pub fn registry_from_portfolios(storage: &PortfolioStorage) -> SubscriptionRegistry {
    let mut reg = SubscriptionRegistry::new();
    let mut direct_actors: Vec<ActorIdentity> = Vec::new();
    ...
    if !direct_actors.is_empty() {
        reg.register(Box::new(
            GlobalSubscription::new("social_global", direct_actors)
                .with_kinds(["social_post".to_string(), "macro_event".to_string()]),
        ));
    }
```

```rust
// crates/hone-event-engine/src/router.rs:618-647
    info!(
        event_id = %event.id,
        kind = %kind_tag(&event.kind),
        source = %event.source,
        symbols = ?event.symbols,
        "dispatch skipped: no matching actor"
    );
    return Ok((0, 0));
}
...
let sev = self.apply_per_actor_severity_override(event, sev, &user_prefs);
let sev = self.apply_quiet_mode(event, sev, &user_prefs);
if !user_prefs.should_deliver(event) {
```

```rust
// crates/hone-event-engine/src/prefs.rs:109-132
pub fn should_deliver(&self, event: &MarketEvent) -> bool {
    if !self.enabled {
        return false;
    }
    if event.severity.rank() < self.min_severity.rank() {
        return false;
    }
    if self.portfolio_only && event.symbols.is_empty() {
        return false;
    }
    if self.source_blocked(&event.source) {
        return false;
    }
    if let Some(allow) = &self.allow_sources {
        if !allow.iter().any(|pat| source_matches(&event.source, pat)) {
            return false;
        }
    }
```

- This keeps the issue scoped to routing coverage, not sink assembly or Telegram channel health: the same incremental window had a real `sink|sent|high` row for AMD, but the Reuters Spirit bailout and CNBC Musk/OpenAI lawsuit high news still had no actor route at all.

## Latest巡检 Update

- 2026-04-24T22:26:46Z: after `2026-04-24T18:25:00Z`, the same routing gap recurred. `data/events.sqlite3` stored one new trusted-source High Reuters news row, and its only delivery evidence was `router|no_actor|high`:

```text
created=2026-04-24 20:36:52 UTC
occurred=2026-04-24 16:09:50 UTC
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/business/iheartmedia-holds-merger-talks-with-sirius-xm-bloomberg-news-reports-2026-04-24/
title=IHeartMedia holds merger talks with Sirius XM, Bloomberg News reports
symbols=["IHRT"]
delivery=router|no_actor|high|2026-04-24 20:36:52
```

- The same incremental window still had a working High sink path, so this was not a global sink outage:

```text
2026-04-24 19:36:54|price_band:RKLB:2026-04-24:down:600|sink|high|sent
2026-04-24 20:36:52|news:https://www.reuters.com/business/iheartmedia-holds-merger-talks-with-sirius-xm-bloomberg-news-reports-2026-04-24/|router|high|no_actor
```

- `data/runtime/logs/web.log.2026-04-24` shows the same contrast in local runtime logs: a successful sink send earlier in the same run, then the Reuters event falling through to `dispatch skipped: no matching actor` one hour later:

```text
data/runtime/logs/web.log.2026-04-24:4290:[2026-04-25 03:36:54.142] INFO  sink delivered
data/runtime/logs/web.log.2026-04-24:4408:[2026-04-25 04:36:52.405] INFO  dispatch skipped: no matching actor
```

- The event payload in `data/events.jsonl` remains a trusted-source merger headline with `severity="high"` and `symbols=["IHRT"]`, so the latest evidence still points to routing/subscription coverage rather than upstream classification noise.

## Latest巡检 Update

- 2026-04-25T02:32:11Z: after `2026-04-24T22:25:31Z`, the same routing gap recurred again. `data/events.sqlite3` stored one new trusted-source High Reuters news row, and its only delivery evidence was `router|no_actor|high`:

```text
created=2026-04-25 00:06:52 UTC
occurred=2026-04-24 19:36:21 UTC
source=fmp.stock_news:reuters.com
severity=high
id=news:https://www.reuters.com/world/us-judge-dismisses-musks-fraud-claims-openai-case-plans-proceed-trial-2026-04-24/
title=US judge dismisses Musk's fraud claims in OpenAI case, plans to proceed to trial
symbols=["P-OPEA"]
source_class=trusted
legal_ad_template=0
delivery=router|no_actor|high|2026-04-25 00:06:52
```

- The local log window around the same second again shows `dispatch skipped: no matching actor`, with no `sink delivered` or `[dryrun sink]` evidence in the incremental window:

```text
data/runtime/logs/web.log.2026-04-25:4:[2026-04-25 08:06:51.977] INFO  poller ok
data/runtime/logs/web.log.2026-04-25:5:[2026-04-25 08:06:52.231] INFO  dispatch skipped: no matching actor
data/runtime/logs/web.log.2026-04-25:6:[2026-04-25 08:06:52.251] INFO  dispatch skipped: no matching actor
data/runtime/logs/web.log.2026-04-25:7:[2026-04-25 08:06:52.262] INFO  dispatch skipped: no matching actor
data/runtime/logs/web.log.2026-04-25:8:[2026-04-25 08:06:52.273] INFO  dispatch skipped: no matching actor
data/runtime/logs/web.log.2026-04-25:9:[2026-04-25 08:06:52.286] INFO  dispatch skipped: no matching actor
data/runtime/logs/web.log.2026-04-25:10:[2026-04-25 08:06:52.288] INFO  poller ok
```

- This keeps the issue scoped to routing coverage rather than sink assembly or dryrun mode: the same incremental window later logged digest sends and a real sink assembly, while `config.yaml` still has `event_engine.dryrun=false`:

```text
data/runtime/logs/web.log.2026-04-25:476:[2026-04-25 09:00:52.600] INFO  digest delivered
data/runtime/logs/web.log.2026-04-25:478:[2026-04-25 09:00:53.188] INFO  digest delivered
data/runtime/logs/web.log.2026-04-25:729:[2026-04-25 10:00:58.732] INFO  event engine sink: MultiChannelSink 已装配
```

## Severity

sev2. The affected events are high severity and one is a safety recall while another is a guidance cut; if they should match the user, the current evidence trail makes the miss silent rather than auditable.

## Date Observed

2026-04-22T10:11:32Z
