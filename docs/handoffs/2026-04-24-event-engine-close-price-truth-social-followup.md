# Event Engine Close Price 与 Truth Social 后续修复

- title: Event Engine Close Price 与 Truth Social 后续修复
- status: done
- created_at: 2026-04-24
- updated_at: 2026-04-24
- owner: Codex
- related_files:
  - `crates/hone-event-engine/src/pollers/social/truth_social.rs`
  - `crates/hone-event-engine/src/pollers/price.rs`
  - `crates/hone-event-engine/src/router.rs`
  - `crates/hone-event-engine/src/digest.rs`
  - `crates/hone-event-engine/src/event.rs`
  - `scripts/diagnose_event_engine_daily_pushes.py`
  - `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
  - `docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md`
  - `docs/bugs/event_engine_close_price_alerts_never_immediate.md`
  - `docs/bugs/event_engine_digest_omitted_items_and_low_signal_noise.md`
  - `docs/bugs/event_engine_social_source_decode_failures.md`
  - `docs/bugs/README.md`
- related_docs:
  - `docs/archive/plans/event-engine-close-price-truth-social-followup.md`
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-23-event-engine-push-quality.md`
- related_prs:
  - N/A

## Summary

本轮补了 Truth Social poller 的失败诊断，修复 `price_close` 高波动不会即时推送的问题，并继续处理 2026-04-24 Telegram digest 反馈：省略项不可审计、低信号新闻/宏观/评级噪声挤入摘要。真实模型 baseline 已从 12 个 LLM 样本扩到 15 个，并使用 `amazon/nova-lite-v1` 全部匹配；同时导出并审阅了 `telegram::::8039067465` 在 2026-04-23 与 2026-04-24 的实际 event-engine 投递记录。

## What Changed

- Truth Social:
  - `TruthSocialPoller` 不再先 `resp.json()`；统一先读响应文本，再按 status 和 JSON parse 结果报错。
  - 非 2xx / 非 JSON 错误现在包含 `status`、`content_type` 和折叠空白后的前 240 字符 `body_prefix`。
  - 新增本地 mock HTTP 测试覆盖 `503 text/html` 和 `200 text/html` 两类 opaque decode 场景。
- Close price:
  - `price_close` 达到全局 high 阈值时生成 `Severity::High`。
  - per-actor `price_high_pct_override` 不再排除 `window="close"`，超过系统直推地板或满足大仓位敏感阈值的 close 事件可即时推送。
  - 普通 close 波动仍为 Low / digest，不会把全部收盘波动升成即时提醒。
- Bug 台账:
  - `event_engine_close_price_alerts_never_immediate.md` 标记 Fixed。
  - `event_engine_digest_omitted_items_and_low_signal_noise.md` 新增并标记 Fixed。
  - `truth_social_poller_opaque_json_decode_stalls_source.md` 标记 Fixing，等待下一条 live 补偿日志确认外部根因。
  - `event_engine_social_source_decode_failures.md` 补充 Truth Social 可观测性进展。
- Digest / news quality:
  - Digest flush 对 curation/topic-memory/max-items 省略项写 `delivery_log channel='digest_item' status='omitted'`。
  - 校准导出脚本新增 `digest_omitted` 分组。
  - Digest curation 过滤 Low `news_critical`、`source_class=opinion_blog/pr_wire` news、无 symbols 的低优先级 WatcherGuru/Truth Social、Low/远期 macro 和 no-op analyst hold/reiterate。
  - Router window convergence 不再把 opinion/pr-wire news 从 Low 升 Medium。
  - `immediate_kinds=["analyst_grade"]` 不再强制直推 no-op analyst hold/reiterate。
  - News classifier baseline fixture 从 30 条扩到 43 条：新增 13 条 2026-04-24 daily calibration stock-news 样本，其中 3 条进入真实 LLM baseline；非 stock-news 的 social/macro/analyst 通过 Rust 单测和 bug 文档固化。

## Telegram 2026-04-23 Calibration

导出命令：

```bash
rtk python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465
```

输出文件在 ignored 目录：

- `data/exports/event-engine-calibration/event_engine_calibration_telegram____8039067465_2026-04-23.json`
- `data/exports/event-engine-calibration/event_engine_calibration_telegram____8039067465_2026-04-23.md`
- `data/exports/event-engine-calibration/event_engine_delivery_raw_telegram____8039067465_2026-04-23.json`

摘要：121 条 delivery rows，其中 6 条即时、5 个 digest batch、14 条 digest item、96 条 queued/cooled_down、0 条 failed。`event_engine_delivery_raw_*` 额外保留了 `delivery_log.body` 作为 `sent_body`，并保留 `events.payload_json`，可直接追溯“发给用户的正文”和“触发该正文的原始事件 payload”。

应升级：

- `price_close:GEV:2026-04-22`、`price_close:AMD:2026-04-22`、`price_close:MU:2026-04-22`：均超过 6%，实际在 16:00 digest 才送达；本轮修复后同类 close 应走 High / immediate。
- 已记录 bug 中的 `price_close:AAOI/RKLB/TEM:2026-04-23`：同类下跌超过 6%，本轮修复后也应即时。

应降级 / 过滤：

- `price:BE:2026-04-23`：+4.00% 的 Low 价格波动实际被即时推送；若不是大仓位，应降级为 digest。当前代码已有系统直推地板测试覆盖；若 live 仍复现，优先查部署版本和 thresholds 配置。
- `grade:GEV:2026-04-23T10:48:30.000Z:Guggenheim`：`action=hold` 且 `Buy -> Buy`，实际因 `immediate_kinds=["analyst_grade"]` 即时推送；更合理是 digest，除非渲染中明确展示“目标价从 910 上调到 1300”这类可行动信息。
- 16:00 digest 中的 `[SA] M3 Money Supply YoY`、`[SA] Private sector loans YoY`、`[RS] Gross Domestic Product YoY`：低相关宏观长尾数据，建议 filter 或保留 queued 不入发出 digest。
- 21:00 digest 中的 `Jim Cramer sets Google stock price target`、`SpaceX IPO Rewrites The Playbook For Rocket Lab`：更像观点/话题流量，不应占据用户 digest 的前排；适合作为 future baseline candidate 继续压低。

保留 digest：

- `Rocket Lab Completes Second Dedicated Launch for Japan Aerospace Exploration Agency (JAXA)`：对 RKLB 持仓有用，但不是必须即时，建议保持 digest / baseline candidate。
- `Apple Supplier STMicroelectronics Logs Higher Sales on Strong AI Demand`：对 AAPL 是间接供应链信息，保留 digest 即可。
- `[US] FOMC Economic Projections`：实际落在未来窗口，保留 digest，不提前即时。

## Telegram 2026-04-24 Calibration

导出命令：

```bash
rtk python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-24 --actor telegram::::8039067465 --include-body
```

输出文件在 ignored 目录：

- `data/exports/event-engine-calibration/event_engine_calibration_telegram____8039067465_2026-04-24.json`
- `data/exports/event-engine-calibration/event_engine_calibration_telegram____8039067465_2026-04-24.md`
- `data/exports/event-engine-calibration/event_engine_delivery_raw_telegram____8039067465_2026-04-24.json`
- `data/exports/event-engine-calibration/event_engine_digest_expanded_telegram____8039067465_2026-04-24.json`
- `data/exports/event-engine-calibration/event_engine_digest_expanded_telegram____8039067465_2026-04-24.md`

摘要：105 条 delivery rows，其中 1 条即时、2 个 digest batch、31 条历史展示项、71 条 queued/cooled_down、0 条 failed。历史 02:30 digest 的 22 条可从 flushed buffer 展开为 14 displayed + 8 omitted；09:00 digest 的 63 条可展开为 17 displayed + 46 omitted。

应过滤 / 降级：

- Low stock news：Tim Cook 回顾、Cramer、earnings preview、估值观点、Zacks/SeekingAlpha/Benzinga/247WallSt 等低信号文章不再进入 digest。
- Opinion/pr-wire convergence：即使同 ticker 当天有价格/评级硬信号，opinion/pr-wire news 也不再从 Low 升 Medium。
- WatcherGuru 无 symbols 的低优先级社交消息不再进入 digest。
- 低优先级宏观与超过 48 小时 lookahead 的非 High macro 不再进入当前摘要。
- `hold/maintained/reiterated` 且评级未变化的 analyst grade 不再被 `immediate_kinds` 直推，也不再进入 digest。

应升级：

- `price_close:AAOI/RKLB/TEM:2026-04-23`：旧 09:00 digest 才展示；close price 修复后同类超过 6% 的收盘波动应即时。

## Verification

- `rtk cargo test -p hone-event-engine truth_social --lib`：7 passed
- `rtk cargo test -p hone-event-engine close_quote --lib`：4 passed
- `rtk cargo test -p hone-event-engine per_actor_price_threshold_can_promote_closing_move --lib`：1 passed
- `rtk cargo test -p hone-event-engine --lib`：219 passed, 13 ignored
- `rtk cargo fmt --all -- --check`：passed
- `rtk bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`：fixture loaded, 43 items, 15 LLM items
- `rtk cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`：1 passed
- `rtk env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`：15/15 matched, reported cost `0.000640`, avg latency `1.81s`
- `rtk git diff --check`：passed

## Risks / Follow-ups

- Truth Social 的真实断流根因仍未确认；需要 live 进程跑出新日志后查看 `body_prefix`。
- Historical digest batches before this fix still do not have `digest_item omitted` rows in SQLite；2026-04-24 已用 flushed buffer 反推出 expanded export。
- `portfolio_only=false` + `min_severity=low` 仍会保留低强度非新闻事件，例如小幅 price digest；如果后续要进一步减少低价波动，需要独立改 preference / policy。

## Next Entry Point

- Truth Social 后续看 `docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md`。
- 推送质量下一轮优先从 `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md` 的 Telegram calibration 样本继续。
