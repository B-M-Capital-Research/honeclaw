# Global digest Pass1 model collapse + over-strict severity gate

- title: Global digest Pass1 model collapse + over-strict severity gate
- status: Fixed
- severity: P1
- created_at: 2026-04-27
- updated_at: 2026-04-27
- owner: Claude
- related_files:
  - `crates/hone-event-engine/src/global_digest/curator.rs`
  - `crates/hone-event-engine/src/global_digest/collector.rs`
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-core/src/config/event_engine.rs`
  - `config.yaml`
- verification:
  - `cargo test -p hone-event-engine --lib global_digest::`
  - `cargo build -p hone-event-engine -p hone-core -p hone-cli`
  - 下次 21:00 推送复核 picks 是否摆脱「4 条全宏观 + 3 条同主题重复」

## Evidence

2026-04-27 09:00 早间 global digest 推送给用户(actor `telegram::::8039067465`)4 条 picks,**全部** `[宏观]`,且 3/4 在重复同一主线(Iran-Hormuz-Oil-Fed):

| # | title | source |
|---|---|---|
| 1 | Oil Rises as Hormuz Stays Shut for Third Month After Talks Stall | rss:bloomberg_markets |
| 2 | Bond Traders Await Powell Update, Slate of US Treasury Auctions | rss:bloomberg_markets |
| 3 | Gold Declines as Attempts to Restart US-Iran Peace Talks Falter | rss:bloomberg_markets |
| 4 | Sen. Tillis Expected to Clear Way for Warsh as Fed Chair | rss:bloomberg_markets |

`data/daily_reports/2026-04-27-global-digest.md` 显示 candidates=24 / baseline_picks=7,event_dedupe 只合了 3 簇,但 4 条 personalize pick 仍高度同质。

## Root cause(POC 实测,2026-04-27)

两个独立缺陷叠加:

### 1. Pass1 模型(`amazon/nova-lite-v1`)在 42-61 候选量级下完全塌掉

POC 用同 prompt + 同 audience + 同候选池实测对比:

| 模型 | 候选 42 score 分布 | 候选 61 score 分布 |
|---|---|---|
| `amazon/nova-lite-v1`(线上) | 1/2/3 = 22/17/3,**无 4 / 无 5** | 1/2/3 = 28/25/8,**无 4 / 无 5** |
| `x-ai/grok-4.1-fast` | 1/2/3/4 = 16/13/4/9 | 1/2/3/4/5 = 21/16/5/15/3 |

curator.rs prompt 明确要求"5 分锚点 + 具体例子,避免两极化",nova-lite-v1 完全不遵守 → 所有候选挤进低分,top_n 排序失效。

更严重的是 cluster 行为:nova-lite 把 11 条 Iran/Hormuz/Oil/Gold/Stock-futures 全标成各自独立 cluster(`hormuz-blockade` / `frontier-markets-rally` / `imf-global-economy` / `bond-traders-powell-update` / ...),而 grok-4.1-fast 自动合到一个 `iran-hormuz-crisis` cluster → `rank_and_dedupe()` 自动只留 1 条代表。**这就是早间 4 条 picks 中 3 条同主题的根因。**

### 2. collector SQL 严重度门槛把 FMP trusted-Low 全砍光

`store.rs:list_global_digest_news_candidates` 原 SQL `severity IN ('high', 'medium')` 一刀切。`pollers::news::classify_severity` 对 FMP 的升级路径要求"trusted 域 AND 命中 distress/M&A 关键词"(bankruptcy / lawsuit / recall / CEO resigns / merger / 等 32 个词),routine business news 即便来自 reuters/wsj/cnbc 也只能保持 Low。

POC 抽 24h 窗口实测:trusted-Low FMP 共 19 条,其中 ~25% 是 thesis 真硬料:

- `marketwatch.com` "Wall Street's Super Bowl Wednesday: Alphabet, Amazon, Microsoft and Meta report along with Powell's last Fed meeting" —— 直接命中用户 GOOGL/AMD/MU 持仓
- `reuters.com` "Chip toolmaker Tokyo Electron cuts ties with executive linked to Chinese rivals" —— 半导体上下游
- `cnbc.com` 三条 Sun Pharma $11.75B Organon 并购
- `wsj.com` "Apple's New Boss"

放进 grok-4.1-fast Pass1 后,这些条目得到 5 分,直接以 `thesis_aligned` 进入 Pass2 personalize。

## Fix

1. `config.yaml:123` + `crates/hone-core/src/config/event_engine.rs:default_global_digest_pass1_model`:把 Pass1 模型默认值从 `amazon/nova-lite-v1` 切到 `x-ai/grok-4.1-fast`。成本从 ~$0.001/run → ~$0.003/run(2 次/天 = $0.006/天),延迟 +10s。
2. `crates/hone-event-engine/src/store.rs:list_global_digest_news_candidates`:SQL 改成非对称门槛 —— RSS 始终通过(信源已是 trusted),FMP 维持 high/medium **或** `source_class=trusted` 时也通过 Low。其它 source_class(opinion_blog / pr_wire / uncertain) 仍需 high/medium。
3. `crates/hone-event-engine/src/global_digest/curator.rs` 顶部 POC 注释更新,明确 nova-lite 不可用。
4. `crates/hone-event-engine/src/global_digest/collector.rs` 测试同步:`drops_low_severity` → `keeps_low_severity_when_fmp_source_class_is_trusted` + 新增 `still_drops_low_severity_for_pr_wire_or_opinion_blog`。

## Followups

- 周末 FMP 摄入量从 1400/天崩到 340/天的疑似 poller / quota 问题,需另开 ticker 单独跟。本次 fix 不在该范围。
- `event_dedupe` 现已与 Pass1 共用 grok-4.1-fast,功能上有重叠;若后续观察到 cluster 主题归并稳定,可评估是否退化为 PassThroughDeduper 节省一次 LLM 调用。
