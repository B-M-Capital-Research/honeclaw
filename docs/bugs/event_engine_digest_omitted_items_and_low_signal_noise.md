# Event-engine digest omitted items and low-signal noise

- title: Event-engine digest omitted items and low-signal noise
- status: Fixed
- severity: P2
- created_at: 2026-04-24
- updated_at: 2026-04-24
- owner: Codex
- related_files:
  - `crates/hone-event-engine/src/digest.rs`
  - `crates/hone-event-engine/src/router.rs`
  - `crates/hone-event-engine/src/event.rs`
  - `scripts/diagnose_event_engine_daily_pushes.py`
- verification:
  - `rtk cargo test -p hone-event-engine --lib`
  - `rtk cargo fmt --all -- --check`
  - `rtk bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
  - `rtk env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`

## Evidence

用户贴出的 2026-04-24 02:30 与 09:00 Telegram digest 都包含 `…… 另 N 条已省略`，但历史 `delivery_log` 只把展示出来的条目写成 `channel='digest_item' status='sent'`。省略项只能从 `data/digest_buffer/telegram__direct__8039067465.flushed-*` 反推，日常校准导出无法直接回答“省略的是什么”。

同时，实际 digest 中混入多类低信号内容：

- `Jim Cramer`、earnings preview、估值观点、Zacks/SeekingAlpha/Benzinga/247WallSt 等低优先级 stock news。
- 无持仓标的的 WatcherGuru 低优先级社交消息。
- 7 天后的低相关宏观日历长尾。
- `action=hold` 且 `previousGrade == newGrade` 的 analyst grade 被 `immediate_kinds=["analyst_grade"]` 强制即时推送。

本轮已生成 ignored 证据导出：

- `data/exports/event-engine-calibration/event_engine_calibration_telegram____8039067465_2026-04-24.json`
- `data/exports/event-engine-calibration/event_engine_calibration_telegram____8039067465_2026-04-24.md`
- `data/exports/event-engine-calibration/event_engine_digest_expanded_telegram____8039067465_2026-04-24.json`
- `data/exports/event-engine-calibration/event_engine_digest_expanded_telegram____8039067465_2026-04-24.md`
- `data/exports/event-engine-calibration/event_engine_delivery_raw_telegram____8039067465_2026-04-24.json`

## Root Cause

`DigestScheduler::tick_once` 在 curation/topic-memory/max-items 截断后只保留 `filtered`，随后仅对 `filtered` 写 `digest_item sent`。被 curation 或 truncation 丢掉的事件只体现在 footer 数字里，既没有 `digest_item omitted` 记录，也没有诊断导出分组。

降噪方面，`maybe_upgrade_news` 会对 `source_class=opinion_blog/pr_wire` 的 Low news 执行 window convergence 升级，导致观点/preview/listicle 文章在同日价格或评级硬信号附近升到 Medium 并挤入 digest。Digest curation 也没有把 Low news、远期 macro、no-op analyst hold 作为不可展示噪声处理。

## Fix

- Digest flush 现在保留 `omitted_events`，对 curation/topic-memory/max-items 省略项写入 `delivery_log`：`channel='digest_item' status='omitted'`。
- `scripts/diagnose_event_engine_daily_pushes.py` 新增 `digest_omitted` 分组和 summary 字段，后续导出可直接看到省略项。
- `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json` 从 30 条扩到 43 条，其中 2026-04-24 daily calibration 新增 13 条 stock-news 样本；真实 LLM 样本从 12 条扩到 15 条。
- Digest curation 过滤：
  - Low `news_critical`。
  - `source_class=opinion_blog/pr_wire` 的 news，即使被 convergence 升到 Medium。
  - Low 且无 symbols 的 WatcherGuru/Truth Social 社交消息。
  - Low macro，以及超过 48 小时 lookahead 的非 High macro。
  - no-op analyst hold/reiterate。
- Router window convergence 不再升级 `source_class=opinion_blog/pr_wire` 的 Low news。
- `immediate_kinds` 不再把 no-op analyst hold/reiterate 强制升 High。

## Verification

- `rtk cargo test -p hone-event-engine --lib`：219 passed, 13 ignored
- `rtk cargo fmt --all -- --check`：passed
- `rtk bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`：fixture loaded, 43 items, 15 LLM items
- `rtk env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`：15/15 matched, reported cost `0.000640`, avg latency `1.81s`

## Risks

- Historical digest batches before this fix still do not have `digest_item omitted` rows in SQLite; they must be reconstructed from `data/digest_buffer/*.flushed-*`.
- `portfolio_only=false` plus `min_severity=low` remains intentionally broad for non-news events. If future feedback says low price alerts should also disappear from digest, that should be a separate preference / policy change.
