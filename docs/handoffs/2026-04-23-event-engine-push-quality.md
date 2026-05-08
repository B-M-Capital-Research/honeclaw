# Event Engine 推送质量全量修复

- title: Event Engine 推送质量全量修复
- status: done
- created_at: 2026-04-23
- updated_at: 2026-05-08
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
  - `crates/hone-event-engine/src/pollers/analyst_grade.rs`
  - `crates/hone-event-engine/src/router/dispatch.rs`
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-event-engine/src/pollers/rss.rs`
- related_docs:
  - `docs/archive/plans/event-engine-push-quality.md`
  - `docs/archive/index.md`
  - `docs/bugs/event_engine_high_macro_events_unrouted.md`
  - `docs/bugs/event_engine_social_source_decode_failures.md`
  - `docs/bugs/event_engine_window_convergence_upgrade_burst.md`
  - `docs/archive/plans/event-engine-push-quality-hardening.md`
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

- `cargo fmt --all -- --check`
- `cargo test -p hone-event-engine --lib`
- `cargo test -p hone-core --lib`
- `cargo check -p hone-web-api`
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- live OpenRouter/FMP manual eval with saved baseline: 12/12 parseable, classifier drift screened against non-Google/non-OpenAI/non-Anthropic candidates
- smoke run: `python3 scripts/diagnose_event_engine_daily_pushes.py --actor 'telegram::::8039067465' --date 2026-04-23`

## Risks / Follow-ups

- 基线脚本仍是手工回归，不进默认 CI；后续改新闻分类模型时要先重跑基线再调整默认值。
- digest curation 和 topic memory 现在偏保守；如果后续用户反馈“重要重复提醒被压掉”，应优先调预算和窗口，而不是回退整套去重。
- 真实渠道的送达质量仍依赖各 sink 凭据和外部 API 健康度；代码侧已补 observability，但并不替代线上巡检。

## Next Entry Point

- `docs/archive/plans/event-engine-push-quality.md`
- `docs/archive/plans/event-engine-push-quality-hardening.md`

## 2026-05-08 POC 后续收口

近期 event review 的共性问题进一步收敛为三类：同一个 TheFly analyst 聚合页拆出多投行即时推送、Zacks 泛化模板混入候选、可信 RSS 因缺 ticker 落到 `no_actor`。本轮只落确定性 guardrail，没有引入新的 LLM 调用。

### What Changed

- AnalystGrade poller 会把同一 ticker + `newsURL` 的 fanout 按信号强度排序，优先让真实评级变化或目标价变化作为代表事件进入路由。
- Router 新增同 ticker + 同 analyst source article 的 cooldown 查询：第一条 High sink 送达后，同源文章后续投行行降级进 digest，避免一篇聚合页制造多条 immediate。
- RSS 只做标题级实体链接，覆盖 CoreWeave / Rocket Lab / Nebius / Broadcom / Nvidia / SanDisk / Micron / Vistra / AMD / Tempus AI / Coherent 等高置信 alias；summary-only 和 URL-only 命中不链接，链接到 ticker 的 RSS 事件保持 digest 级别。
- Zacks 泛化 stock attention 模板新增 poller 与 digest curation 回归测试，锁住 `opinion_blog -> Low -> omitted` 行为。

### Verification

- `cargo test -p hone-event-engine --lib`
- `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- `rustfmt --edition 2024 --check` on the changed event-engine Rust files
- `cargo fmt --all -- --check` was attempted but is blocked by unrelated existing formatting debt outside this task scope

### Risks / Follow-ups

- RSS alias 表保持小而保守；新增公司名应先从 event review 的真实 `no_actor` 证据或 POC fixture 出发。
- Analyst fanout guardrail 依赖 FMP 行里存在 `newsURL` 或 `event.url`；无 URL 行仍走既有 firm-aware cooldown。
