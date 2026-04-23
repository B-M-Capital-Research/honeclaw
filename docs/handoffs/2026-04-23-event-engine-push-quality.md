# Event Engine 推送质量全量修复

- title: Event Engine 推送质量全量修复
- status: done
- created_at: 2026-04-23
- updated_at: 2026-04-23
- owner: Codex
- related_files:
  - `crates/hone-event-engine/src/router.rs`
  - `crates/hone-event-engine/src/digest.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-event-engine/src/news_classifier.rs`
  - `crates/hone-event-engine/src/pollers/`
  - `crates/hone-core/src/config/event_engine.rs`
  - `config.example.yaml`
  - `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
  - `tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
  - `scripts/diagnose_event_engine_daily_pushes.py`
  - `.agents/skills/event-engine-baseline-testing/SKILL.md`
- related_docs:
  - `docs/archive/plans/event-engine-push-quality.md`
  - `docs/archive/index.md`
  - `docs/bugs/event_engine_high_macro_events_unrouted.md`
  - `docs/bugs/event_engine_social_source_decode_failures.md`
  - `docs/bugs/event_engine_window_convergence_upgrade_burst.md`
- related_prs:
  - N/A

## Summary

本轮把 event engine 的 24 项推送质量清单完整收口，并将其从活跃计划移出。重点不是单点修 bug，而是把价格、新闻、宏观、财报、摘要和偏好几条链路一起拉回到更可解释、更低噪的默认行为。

## What Changed

- Digest 现在有去重、topic memory、source/domain/symbol curation、min-gap 与 per-category budget，减少同一窗口和相邻窗口重复消费同一批主题。
- 路由层补齐了价格方向性阈值、大仓位直推下限、macro immediate 时窗、legal/social/source 质量降噪、quiet mode 与 portfolio-first 默认行为。
- `earnings_call_transcript` 从原先混在财报类里拆成独立 kind；新闻不确定来源分类也补了离线基线与手工重跑脚本。
- delivery / digest observability 明显增强：buffer rotate、digest item 级 delivery rows、dryrun status 语义、poller degraded logs、daily calibration exporter 都已补齐。
- 默认不确定来源新闻分类模型更新为 `amazon/nova-lite-v1`；`config.example.yaml` 和测试 skill 已同步。

## Verification

- `rtk cargo fmt --all -- --check`
- `rtk cargo test -p hone-event-engine --lib`
- `rtk cargo test -p hone-core --lib`
- `rtk cargo check -p hone-web-api`
- `rtk bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- live OpenRouter/FMP manual eval with saved baseline: 12/12 parseable, classifier drift screened against non-Google/non-OpenAI/non-Anthropic candidates
- smoke run: `rtk python3 scripts/diagnose_event_engine_daily_pushes.py --actor 'telegram::::8039067465' --date 2026-04-23`

## Risks / Follow-ups

- 基线脚本仍是手工回归，不进默认 CI；后续改新闻分类模型时要先重跑基线再调整默认值。
- digest curation 和 topic memory 现在偏保守；如果后续用户反馈“重要重复提醒被压掉”，应优先调预算和窗口，而不是回退整套去重。
- 真实渠道的送达质量仍依赖各 sink 凭据和外部 API 健康度；代码侧已补 observability，但并不替代线上巡检。

## Next Entry Point

- `docs/archive/plans/event-engine-push-quality.md`
